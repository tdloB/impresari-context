#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Isolated Gemini CLI and GitHub Copilot CLI MCP rehearsal. Its normal flow
# never writes either client's persistent configuration. The preview-first
# Copilot preparation mode writes only an explicitly named disposable project
# directory under /private/tmp.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
SERVER = "impresari-context"
CORE_LIFECYCLE_TOOLS = %w[context_session_open context_build context_packet_resolve context_session_close].freeze
COPILOT_CORE_TOOLS = CORE_LIFECYCLE_TOOLS.map { |tool| "#{SERVER}-#{tool}" }.freeze
FIXED_TIME = "2026-08-23T12:00:00Z"
SESSION = "session_copilot_conformance01"
REQUEST = "req_copilot_conformance01"
EVENT = "evt_copilot_conformance01"
PURPOSE = "copilot_cli_conformance"
QUERY = "__impresari_copilot_conformance_probe__"

options = {
  gemini: "gemini",
  copilot: "copilot",
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  malformed_copilot_config_only: false,
  prepared_copilot_project_root: nil,
  apply: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-gemini-copilot-preadmission.rb [options]"
  parser.on("--gemini PATH", "Gemini CLI executable") { |value| options[:gemini] = value }
  parser.on("--skip-gemini", "Skip Gemini discovery after an external client failure") { options[:gemini] = nil }
  parser.on("--copilot PATH", "Copilot CLI executable") { |value| options[:copilot] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--malformed-copilot-config-only", "Verify Copilot rejects malformed temporary MCP configuration") do
    options[:malformed_copilot_config_only] = true
  end
  parser.on("--prepare-copilot-project-root PATH", "Preview or prepare a disposable Copilot project root under /private/tmp") do |value|
    options[:prepared_copilot_project_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable-project preparation") { options[:apply] = true }
end.parse!

abort("missing executable: #{options[:mcp]}") unless File.file?(options[:mcp]) && File.executable?(options[:mcp])
abort("missing executable: #{options[:cli]}") unless File.file?(options[:cli]) && File.executable?(options[:cli])

def temporary_project_root(path, allow_absent: false)
  candidate = Pathname.new(path).expand_path
  temporary_parent = Pathname.new("/private/tmp").realpath
  resolved = if candidate.exist?
               candidate.realpath
             elsif allow_absent
               candidate.parent.realpath.join(candidate.basename)
             else
               abort("prepared project root does not exist: #{candidate}")
             end
  abort("prepared project root must be under /private/tmp") unless resolved.to_s.start_with?("#{temporary_parent}/")
  abort("prepared project root must not be a symbolic link") if candidate.symlink?
  resolved
end

def prepared_layout(root)
  {
    workspace: root.join("workspace"),
    cache: root.join("cache"),
  }
end

if options[:prepared_copilot_project_root]
  root = temporary_project_root(options[:prepared_copilot_project_root], allow_absent: true)
  layout = prepared_layout(root)
  if !options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_copilot_project",
      "project_root" => root.to_s,
      "workspace" => layout.fetch(:workspace).to_s,
      "cache" => layout.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. A user may install only the rendered project .mcp.json entry in the reported workspace before deciding whether to trust that folder in Copilot.",
    })
    exit(0)
  end
  abort("prepared project root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p([layout.fetch(:workspace), layout.fetch(:cache)])
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_copilot_project",
    "project_root" => root.to_s,
    "workspace" => layout.fetch(:workspace).to_s,
    "cache" => layout.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Review the reported paths. Any Copilot folder-trust decision and exact project-entry removal remain user-owned actions.",
  })
  exit(0)
end

def tree_digest(root)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH)
       .select { |path| File.file?(path) }
       .sort
       .map { |path| "#{path.delete_prefix(root)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def run(*command, chdir:)
  stdout, stderr, status = Open3.capture3(*command, chdir: chdir)
  abort("command failed: #{command.join(" ")}\n#{stderr}\n#{stdout}") unless status.success?
  stdout
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
      "clientInfo" => { "name" => "impresari-context-copilot-conformance", "version" => "1.0" },
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

def copilot_events(output)
  output.lines.map { |line| JSON.parse(line) }
rescue JSON::ParserError => error
  raise "Copilot emitted invalid JSONL: #{error.message}"
end

Dir.mktmpdir("impresari-gemini-copilot-preadmission-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  direct_mcp_cache = File.join(temporary, "direct-mcp-cache")
  FileUtils.mkdir_p([workspace, cache, direct_mcp_cache, File.join(workspace, ".gemini")])
  abort("could not initialize temporary Gemini project") unless system("git", "init", "-q", workspace)
  File.write(File.join(workspace, "probe.ts"), "export const __impresari_agent_probe__ = true;\n")
  entry = {
    "command" => options[:mcp],
    "args" => ["--workspace", workspace, "--cache", cache, "--consumer-id", "consumer_phase2_conformance", "--role", "local_user"],
  }
  File.write(File.join(workspace, ".gemini", "settings.json"), JSON.generate({
    "mcpServers" => { SERVER => entry.merge("trust" => false, "includeTools" => CORE_LIFECYCLE_TOOLS) },
  }))
  copilot_config = File.join(temporary, "copilot-mcp.json")
  malformed_copilot_config = File.join(temporary, "malformed-copilot-mcp.json")
  before = tree_digest(workspace)
  File.write(malformed_copilot_config, "{\"mcpServers\":")
  malformed_stdout, malformed_stderr, malformed_status = Open3.capture3(
    options[:copilot],
    "--additional-mcp-config", "@#{malformed_copilot_config}",
    "--disable-builtin-mcps",
    "--no-remote",
    "--no-auto-update",
    "--no-custom-instructions",
    "--log-dir", temporary,
    "--output-format", "json",
    "--prompt", "Do not call any tool.",
    chdir: workspace,
  )
  if malformed_status.success?
    abort("Copilot accepted malformed temporary MCP configuration")
  end
  malformed_output = "#{malformed_stdout}\n#{malformed_stderr}"
  if malformed_output.include?("__impresari_agent_probe__")
    abort("Copilot malformed configuration diagnostic exposed fixture source")
  end
  abort("temporary workspace was altered during malformed Copilot configuration rehearsal") unless before == tree_digest(workspace)
  if options[:malformed_copilot_config_only]
    puts JSON.generate({
      "status" => "passed",
      "copilot_malformed_configuration_rejected" => true,
      "workspace_immutable_after_configuration" => true,
      "persistent_mcp_configuration_changed" => false,
    })
    next
  end
  install_stdout, install_stderr, install_status = Open3.capture3(
    options[:cli], "client", "kit", "install", "copilot", options[:mcp], workspace, cache, copilot_config, "--apply",
  )
  abort("managed Copilot configuration install failed:\n#{install_stderr}\n#{install_stdout}") unless install_status.success?
  install = JSON.parse(install_stdout)
  abort("managed Copilot configuration did not report an explicit write") unless install["external_write_performed"] == true
  validate_stdout, validate_stderr, validate_status = Open3.capture3(
    options[:cli], "client", "kit", "validate", "copilot", options[:mcp], workspace, cache, copilot_config,
  )
  abort("managed Copilot configuration validation failed:\n#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
  direct_packet = direct_mcp_packet(options[:mcp], [
    "--workspace", workspace,
    "--cache", direct_mcp_cache,
    "--consumer-id", "consumer_copilot_managed",
    "--role", "local_user",
    "--occurred-at", FIXED_TIME,
  ])
  gemini_discovered = false
  unless options[:gemini].nil?
    gemini = run(
      options[:gemini],
      "--approval-mode", "default",
      "--allowed-mcp-server-names", SERVER,
      "--output-format", "stream-json",
      "--prompt", "Use the configured Impresari Context MCP server to call context_session_open exactly once. Do not use any other tool.",
      chdir: workspace,
    )
    abort("Gemini did not discover temporary server:\n#{gemini}") unless gemini.include?(SERVER)
    gemini_discovered = true
  end
  copilot = run(
    options[:copilot],
    "--additional-mcp-config", "@#{copilot_config}",
    "--disable-builtin-mcps",
    "--no-remote",
    "--no-auto-update",
    "--no-custom-instructions",
    "--log-dir", temporary,
    "--available-tools", COPILOT_CORE_TOOLS.join(","),
    "--allow-all-tools",
    "--output-format", "json",
    "--prompt", <<~PROMPT,
      Perform this exact Impresari Context MCP lifecycle. Use no other tools.
      1. Call context_session_open with {"session_id":"#{SESSION}"}.
      2. Call context_build with {"request_id":"#{REQUEST}","event_id":"#{EVENT}","purpose":"#{PURPOSE}","occurred_at":"#{FIXED_TIME}","steps":[{"kind":"literal","query":"#{QUERY}"}],"budget":#{JSON.generate(conservative_budget)},"session_id":"#{SESSION}"}.
      3. Call context_packet_resolve with the same session_id and the packet_id returned by context_build.
      4. Call context_session_close with {"session_id":"#{SESSION}"}.
      Reply only after all four calls complete.
    PROMPT
    chdir: workspace,
  )
  events = copilot_events(copilot)
  connected = events.any? do |event|
    event["type"] == "session.mcp_servers_loaded" &&
      Array(event.dig("data", "servers")).any? { |server| server["name"] == SERVER && server["status"] == "connected" }
  end
  abort("Copilot did not connect the temporary MCP server:\n#{copilot}") unless connected
  started = events.filter { |event| event["type"] == "tool.execution_start" }
  completed = events.filter { |event| event["type"] == "tool.execution_complete" }
  observed_tools = started.map { |event| event.dig("data", "toolName") }
  unless observed_tools == COPILOT_CORE_TOOLS
    abort("Copilot lifecycle differed from the required order: #{observed_tools.join(', ')}\n#{copilot}")
  end
  completed_by_tool_call = completed.to_h { |event| [event.dig("data", "toolCallId"), event.fetch("data")] }
  completed_in_order = started.map do |event|
    completed_by_tool_call.fetch(event.dig("data", "toolCallId"))
  end
  if completed_in_order.any? { |result| result["success"] != true }
    abort("Copilot reported an MCP tool failure:\n#{copilot}")
  end
  results_by_tool = started.zip(completed_in_order).to_h do |start, completion|
    [start.dig("data", "toolName"), completion.dig("result", "structuredContent")]
  end
  if results_by_tool.values.any? { |result| !result.is_a?(Hash) }
    abort("Copilot did not expose structured MCP results:\n#{copilot}")
  end
  built_packet = results_by_tool.fetch("#{SERVER}-context_build").fetch("packet")
  resolved_packet = results_by_tool.fetch("#{SERVER}-context_packet_resolve").fetch("packet")
  abort("Copilot resolved packet differs from its delivered packet") unless resolved_packet == built_packet
  abort("direct MCP packet differs from Copilot-delivered packet") unless direct_packet == built_packet
  remove_stdout, remove_stderr, remove_status = Open3.capture3(
    options[:cli], "client", "kit", "remove", "copilot", options[:mcp], workspace, cache, copilot_config, "--apply",
  )
  abort("managed Copilot configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
  abort("managed Copilot configuration target was not removed") if File.exist?(copilot_config)
  abort("temporary workspace was altered") unless before == tree_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "gemini_discovered": gemini_discovered,
    "copilot_discovered": true,
    "copilot_malformed_configuration_rejected": true,
    "copilot_managed_install_validate_remove": true,
    "copilot_direct_mcp_packet_equivalence": true,
    "copilot_tool_lifecycle": observed_tools,
    "workspace_immutable_after_configuration" => true,
    "persistent_mcp_configuration_changed" => false,
  })
end
