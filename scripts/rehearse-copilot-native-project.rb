#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Native project-scope GitHub Copilot CLI MCP rehearsal. It uses an explicit
# disposable project and COPILOT_HOME under /private/tmp. The CLI's real home
# and any real source repository are never selected.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_COPILOT = "copilot"
SERVER = "impresari-context"
FIXED_TIME = "2026-08-26T00:00:00Z"
SESSION = "session_copilot_native_project01"
REQUEST = "req_copilot_native_project01"
EVENT = "evt_copilot_native_project01"
PURPOSE = "copilot_cli_native_project_conformance"
QUERY = "__impresari_copilot_native_project_probe__"
FIXTURE_NAME = "__impresari_copilot_native_project_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_copilot_native_project_probe__ = true;\n"
CORE_TOOLS = %w[context_session_open context_build context_packet_resolve context_session_close].freeze
COPILOT_TOOLS = CORE_TOOLS.map { |tool| "#{SERVER}-#{tool}" }.freeze

options = {
  copilot: DEFAULT_COPILOT,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  prepared_root: nil,
  temporary_root: nil,
  apply: false,
  native_guidance_smoke: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-copilot-native-project.rb [options]"
  parser.on("--copilot PATH", "GitHub Copilot CLI executable") { |value| options[:copilot] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-root PATH", "Preview or prepare a disposable root under /private/tmp") do |value|
    options[:prepared_root] = value
  end
  parser.on("--temporary-root PATH", "Run against a prepared disposable root under /private/tmp") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable-root preparation") { options[:apply] = true }
  parser.on("--native-guidance-smoke", "Install the owned Copilot repository instruction and run the bounded MCP lifecycle smoke") do
    options[:native_guidance_smoke] = true
  end
end.parse!

[options[:cli], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

def executable_available?(path)
  return File.file?(path) && File.executable?(path) if path.include?(File::SEPARATOR)

  _stdout, _stderr, status = Open3.capture3("which", path)
  status.success?
end

abort("Copilot CLI was not found on PATH: #{options[:copilot]}") unless executable_available?(options[:copilot])

def temporary_root(path, allow_absent: false)
  candidate = Pathname.new(path).expand_path
  parent = Pathname.new("/private/tmp").realpath
  resolved = if candidate.exist?
               candidate.realpath
             elsif allow_absent
               candidate.parent.realpath.join(candidate.basename)
             else
               abort("prepared root does not exist: #{candidate}")
             end
  abort("prepared root must be under /private/tmp") unless resolved.to_s.start_with?("#{parent}/")
  abort("prepared root must not be a symbolic link") if candidate.symlink?
  resolved
end

def layout(root)
  {
    home: root.join("copilot-home"),
    workspace: root.join("workspace"),
    cache: root.join("cache"),
  }
end

def source_digest(workspace)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(workspace, "**", "*"), File::FNM_DOTMATCH)
       .select do |path|
         relative = path.delete_prefix("#{workspace}/")
         File.file?(path) && relative != ".mcp.json" &&
           relative != ".github/instructions/impresari-context.instructions.md" &&
           !relative.start_with?(".git/")
       end
       .sort
       .map { |path| "#{path.delete_prefix(workspace)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def read_copilot_config(path)
  raw = File.binread(path)
  JSON.parse(raw.gsub(/^\s*\/\/.*\n/, ""))
end

def write_copilot_config(path, configuration)
  File.write(path, "// User settings belong in settings.json.\n// This file is managed automatically.\n#{JSON.pretty_generate(configuration)}\n")
end

def conservative_budget
  {
    "unit_kind" => "utf8_bytes",
    "requested" => "65536",
    "hard" => true,
    "max_evidence_items" => "100",
    "max_files" => "10000",
    "max_excerpt_bytes_per_item" => "4096",
    "max_matches" => "1000",
    "max_traversal_depth" => "32",
    "max_elapsed_ms" => "30000",
    "max_memory_bytes" => "536870912",
    "policy_profile" => "sha256:aba86621046ccc86cff7aabb81f4eab1020ab6db53ae1b649ea3977dec9649e8",
  }
end

class Rpc
  def initialize(stdin, stdout)
    @stdin = stdin
    @stdout = stdout
    @next_id = 1
  end

  def call(method, params)
    id = @next_id
    @next_id += 1
    @stdin.puts(JSON.generate({ "jsonrpc" => "2.0", "id" => id, "method" => method, "params" => params }))
    @stdin.flush
    deadline = Process.clock_gettime(Process::CLOCK_MONOTONIC) + 30
    loop do
      remaining = deadline - Process.clock_gettime(Process::CLOCK_MONOTONIC)
      raise "timed out waiting for direct MCP response to #{method}" if remaining <= 0
      ready = IO.select([@stdout], nil, nil, remaining)
      raise "direct MCP closed stdout during #{method}" unless ready
      line = @stdout.gets
      raise "direct MCP closed stdout during #{method}" if line.nil?
      value = JSON.parse(line)
      next unless value["id"] == id
      raise "direct MCP error for #{method}: #{JSON.generate(value.fetch('error'))}" if value.key?("error")
      return value.fetch("result")
    end
  end
end

def mcp_tool_payload(result)
  return result.fetch("structuredContent") if result["structuredContent"].is_a?(Hash)
  text = result.dig("content", 0, "text")
  return JSON.parse(text) if text.is_a?(String)
  raise "direct MCP tool result did not expose a structured payload: #{JSON.generate(result)}"
end

def direct_mcp_packet(executable, server_args)
  stdin, stdout, stderr, wait = Open3.popen3(executable, *server_args)
  stderr_buffer = +""
  stderr_reader = Thread.new { stderr.each_line { |line| stderr_buffer << line } }
  begin
    rpc = Rpc.new(stdin, stdout)
    rpc.call("initialize", {
      "protocolVersion" => "2025-11-25",
      "capabilities" => {},
      "clientInfo" => { "name" => "impresari-context-copilot-native-project", "version" => "1.0" },
    })
    stdin.puts(JSON.generate({ "jsonrpc" => "2.0", "method" => "notifications/initialized" }))
    stdin.flush
    open = mcp_tool_payload(rpc.call("tools/call", {
      "name" => "context_session_open", "arguments" => { "session_id" => SESSION },
    }))
    raise "direct MCP session open was not acknowledged" unless open["opened"] == true
    build = mcp_tool_payload(rpc.call("tools/call", {
      "name" => "context_build",
      "arguments" => {
        "request_id" => REQUEST,
        "event_id" => EVENT,
        "purpose" => PURPOSE,
        "occurred_at" => FIXED_TIME,
        "steps" => [{ "kind" => "literal", "query" => QUERY }],
        "budget" => conservative_budget,
        "session_id" => SESSION,
      },
    }))
    packet = build.fetch("packet")
    resolved = mcp_tool_payload(rpc.call("tools/call", {
      "name" => "context_packet_resolve",
      "arguments" => { "session_id" => SESSION, "packet_id" => packet.fetch("packet_id") },
    }))
    raise "direct MCP resolved packet differs from its built packet" unless resolved.fetch("packet") == packet
    close = mcp_tool_payload(rpc.call("tools/call", {
      "name" => "context_session_close", "arguments" => { "session_id" => SESSION },
    }))
    raise "direct MCP session close was not acknowledged" unless close["closed"] == true
    packet
  ensure
    stdin.close unless stdin.closed?
    stdout.close unless stdout.closed?
    wait.join(10)
    stderr_reader.join(10)
    abort("direct MCP process failed:\n#{stderr_buffer}") if wait.value && !wait.value.success?
  end
end

abort("choose either --prepare-root or --temporary-root, not both") if options[:prepared_root] && options[:temporary_root]

if options[:prepared_root]
  root = temporary_root(options[:prepared_root], allow_absent: true)
  paths = layout(root)
  unless options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_copilot_native_project",
      "root" => root.to_s,
      "copilot_home" => paths.fetch(:home).to_s,
      "workspace" => paths.fetch(:workspace).to_s,
      "cache" => paths.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. The subsequent rehearsal uses only this disposable Copilot home and project entry.",
    })
    exit(0)
  end
  abort("prepared root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p(paths.values)
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_copilot_native_project",
    "root" => root.to_s,
    "copilot_home" => paths.fetch(:home).to_s,
    "workspace" => paths.fetch(:workspace).to_s,
    "cache" => paths.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Run this rehearsal with --temporary-root #{root}.",
  })
  exit(0)
end

abort("--temporary-root is required for the native Copilot project rehearsal") unless options[:temporary_root]
root = temporary_root(options[:temporary_root])
paths = layout(root)
paths.each_value { |path| abort("prepared layout is missing: #{path}") unless path.directory? }
abort("disposable Copilot home must be empty before registration") unless Dir.children(paths.fetch(:home)).empty?
abort("disposable Copilot workspace must be empty before registration") unless Dir.children(paths.fetch(:workspace)).empty?

home = paths.fetch(:home).to_s
workspace = paths.fetch(:workspace).to_s
cache = paths.fetch(:cache).to_s
server_cache = File.join(cache, "copilot-mcp-#{Process.pid}")
direct_cache = File.join(cache, "direct-mcp-#{Process.pid}")
config_path = File.join(workspace, ".mcp.json")
github_directory = File.join(workspace, ".github")
instructions_directory = File.join(github_directory, "instructions")
trust_config_path = File.join(home, "config.json")
fixture = File.join(workspace, FIXTURE_NAME)
environment = { "COPILOT_HOME" => home }
installed = false
guidance_installed = false
trusted = false

begin
  abort("could not initialize disposable Copilot Git project") unless system("git", "init", "-q", workspace)
  File.write(fixture, FIXTURE_CONTENT)
  before = source_digest(workspace)
  FileUtils.mkdir_p([server_cache, direct_cache])
  write_copilot_config(trust_config_path, { "trustedFolders" => [workspace] })
  trusted_folders = read_copilot_config(trust_config_path).fetch("trustedFolders")
  abort("disposable Copilot trust configuration was not exact") unless trusted_folders == [workspace]
  trusted = true
  install_stdout, install_stderr, install_status = Open3.capture3(
    options[:cli], "client", "kit", "install", "copilot", options[:mcp], workspace, server_cache, config_path, "--apply",
  )
  abort("managed Copilot project configuration install failed:\n#{install_stderr}\n#{install_stdout}") unless install_status.success?
  installed = true
  install = JSON.parse(install_stdout)
  abort("managed Copilot project configuration did not report an explicit write") unless install["external_write_performed"] == true
  if options[:native_guidance_smoke]
    FileUtils.mkdir_p(instructions_directory)
    guidance_stdout, guidance_stderr, guidance_status = Open3.capture3(
      options[:cli], "client", "guidance", "install", "copilot", workspace, "--apply",
    )
    abort("Copilot native guidance install failed:\n#{guidance_stderr}\n#{guidance_stdout}") unless guidance_status.success?
    guidance = JSON.parse(guidance_stdout)
    abort("Copilot native guidance did not report an explicit write") unless guidance["external_write_performed"] == true
    guidance_installed = true
    validate_guidance_stdout, validate_guidance_stderr, validate_guidance_status = Open3.capture3(
      options[:cli], "client", "guidance", "validate", "copilot", workspace,
    )
    abort("Copilot native guidance validation failed:\n#{validate_guidance_stderr}\n#{validate_guidance_stdout}") unless validate_guidance_status.success?
  end
  validate_stdout, validate_stderr, validate_status = Open3.capture3(
    options[:cli], "client", "kit", "validate", "copilot", options[:mcp], workspace, server_cache, config_path,
  )
  abort("managed Copilot project configuration validation failed:\n#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?

  list_stdout, list_stderr, list_status = Open3.capture3(
    environment, options[:copilot], "mcp", "list", "--json", chdir: workspace,
  )
  abort("Copilot did not inspect the disposable project entry:\n#{list_stderr}\n#{list_stdout}") unless list_status.success?
  listed = JSON.parse(list_stdout)
  entry = if listed["mcpServers"].is_a?(Hash)
            listed.fetch("mcpServers").fetch(SERVER, nil)
          elsif listed.is_a?(Array)
            listed.find { |candidate| candidate["name"] == SERVER }
          end
  abort("Copilot did not discover the disposable project MCP entry") unless entry
  get_stdout, get_stderr, get_status = Open3.capture3(
    environment, options[:copilot], "mcp", "get", SERVER, "--json", chdir: workspace,
  )
  abort("Copilot did not recognize the disposable project MCP entry:\n#{get_stderr}\n#{get_stdout}") unless get_status.success?
  get = JSON.parse(get_stdout)
  get_entry = if get["mcpServers"].is_a?(Hash)
                get.fetch("mcpServers").fetch(SERVER, nil)
              elsif get[SERVER].is_a?(Hash)
                get.fetch(SERVER)
              else
                get
              end
  unless get_entry.is_a?(Hash) && get_entry["command"] == options[:mcp]
    abort("Copilot did not retain the fixed MCP executable:\n#{JSON.generate(get)}")
  end

  direct_packet = direct_mcp_packet(options[:mcp], [
    "--workspace", workspace,
    "--cache", direct_cache,
    "--consumer-id", "consumer_copilot_managed",
    "--role", "local_user",
    "--occurred-at", FIXED_TIME,
  ])
  native_guidance_request = if options[:native_guidance_smoke]
                              "The task concerns #{FIXTURE_NAME}; apply the owned Impresari Context project instruction. "
                            else
                              ""
                            end
  prompt = <<~PROMPT
    #{native_guidance_request}Perform this exact Impresari Context MCP lifecycle. Use no other tools.
    1. Call context_session_open with {"session_id":"#{SESSION}"}.
    2. Call context_build with {"request_id":"#{REQUEST}","event_id":"#{EVENT}","purpose":"#{PURPOSE}","occurred_at":"#{FIXED_TIME}","steps":[{"kind":"literal","query":"#{QUERY}"}],"budget":#{JSON.generate(conservative_budget)},"session_id":"#{SESSION}"}.
    3. Call context_packet_resolve with the same session_id and the packet_id returned by context_build.
    4. Call context_session_close with {"session_id":"#{SESSION}"}.
    Reply only after all four calls complete.
  PROMPT
  copilot_arguments = [
    "--disable-builtin-mcps", "--no-remote", "--no-auto-update",
    "--log-dir", root.to_s, "--available-tools", COPILOT_TOOLS.join(","),
    "--allow-all-tools", "--allow-all-paths", "--output-format", "json", "--prompt", prompt,
  ]
  copilot_arguments << "--no-custom-instructions" unless options[:native_guidance_smoke]
  copilot_stdout, copilot_stderr, copilot_status = Open3.capture3(
    environment, options[:copilot],
    *copilot_arguments,
    chdir: workspace,
  )
  abort("Copilot project MCP lifecycle failed:\n#{copilot_stderr}\n#{copilot_stdout}") unless copilot_status.success?
  events = copilot_stdout.lines.map { |line| JSON.parse(line) }
  connected = events.any? do |event|
    event["type"] == "session.mcp_servers_loaded" &&
      Array(event.dig("data", "servers")).any? { |server| server["name"] == SERVER && server["status"] == "connected" }
  end
  abort("Copilot did not connect the project MCP entry:\n#{copilot_stdout}") unless connected
  started = events.filter { |event| event["type"] == "tool.execution_start" }
  completed = events.filter { |event| event["type"] == "tool.execution_complete" }
  observed_tools = started.map { |event| event.dig("data", "toolName") }
  abort("Copilot lifecycle differed from the required order: #{observed_tools.join(', ')}\n#{copilot_stdout}") unless observed_tools == COPILOT_TOOLS
  completed_by_call = completed.to_h { |event| [event.dig("data", "toolCallId"), event.fetch("data")] }
  results = started.map { |event| completed_by_call.fetch(event.dig("data", "toolCallId")) }
  abort("Copilot reported an MCP tool failure:\n#{copilot_stdout}") if results.any? { |result| result["success"] != true }
  results_by_tool = started.zip(results).to_h { |start, completion| [start.dig("data", "toolName"), completion.dig("result", "structuredContent")] }
  abort("Copilot did not expose structured MCP results:\n#{copilot_stdout}") if results_by_tool.values.any? { |result| !result.is_a?(Hash) }
  built_packet = results_by_tool.fetch("#{SERVER}-context_build").fetch("packet")
  resolved_packet = results_by_tool.fetch("#{SERVER}-context_packet_resolve").fetch("packet")
  abort("Copilot resolved packet differs from its delivered packet") unless resolved_packet == built_packet
  abort("direct MCP packet differs from Copilot-delivered packet") unless direct_packet == built_packet
  abort("source workspace changed during Copilot project lifecycle") unless before == source_digest(workspace)
ensure
  if guidance_installed
    guidance_stdout, guidance_stderr, guidance_status = Open3.capture3(
      options[:cli], "client", "guidance", "remove", "copilot", workspace, "--apply",
    )
    abort("Copilot native guidance removal failed:\n#{guidance_stderr}\n#{guidance_stdout}") unless guidance_status.success?
    guidance = JSON.parse(guidance_stdout)
    abort("Copilot native guidance removal did not report an explicit write") unless guidance["external_write_performed"] == true
  end
  if installed
    remove_stdout, remove_stderr, remove_status = Open3.capture3(
      options[:cli], "client", "kit", "remove", "copilot", options[:mcp], workspace, server_cache, config_path, "--apply",
    )
    abort("managed Copilot project configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
    abort("managed Copilot project configuration target was not removed") if File.exist?(config_path)
  end
  if trusted
    abort("disposable Copilot trust configuration disappeared before safe removal") unless File.file?(trust_config_path)
    configuration = read_copilot_config(trust_config_path)
    abort("refusing to remove an unexpected disposable Copilot trusted-folder entry") unless configuration["trustedFolders"] == [workspace]
    configuration.delete("trustedFolders")
    write_copilot_config(trust_config_path, configuration)
  end
  if options[:native_guidance_smoke] && File.directory?(instructions_directory)
    abort("Copilot native guidance directory contains unexpected files") unless Dir.children(instructions_directory).empty?
    Dir.rmdir(instructions_directory)
    abort("Copilot GitHub directory contains unexpected files") unless Dir.children(github_directory).empty?
    Dir.rmdir(github_directory)
  end
end

abort("source workspace changed during Copilot project removal") unless before == source_digest(workspace)
if File.exist?(trust_config_path)
  abort("disposable Copilot trusted-folder entry remained after removal") if read_copilot_config(trust_config_path).key?("trustedFolders")
end
abort("refusing to remove an unexpected Copilot rehearsal fixture") unless File.file?(fixture) && File.binread(fixture) == FIXTURE_CONTENT
File.delete(fixture)

puts JSON.generate({
  "status" => "passed",
  "copilot" => options[:copilot],
  "configuration_scope" => "project",
  "isolated_copilot_home" => home,
  "temporary_workspace_trust_removed" => true,
  "native_project_discovery" => true,
  "managed_install_validate_remove" => true,
  "source_immutable" => true,
  "direct_mcp_packet_equivalence" => true,
  "tool_lifecycle" => CORE_TOOLS,
  "native_guidance_smoke" => options[:native_guidance_smoke],
})
