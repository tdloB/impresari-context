#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Deterministic local conformance rehearsal for Codex App Server's MCP bridge.
# It calls the MCP tools through Codex's explicit app-server RPC surface rather
# than asking a conversational model to decide whether to call a tool.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tempfile"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CODEX = "/Applications/ChatGPT.app/Contents/Resources/codex"
FIXED_TIME = "2026-08-23T12:00:00Z"
FIXED_CUTOFF = "2026-08-16T12:00:00Z"
SERVER = "impresari-context"
CONSUMER = "consumer_codex_conformance"
SESSION = "session_codex_conformance01"
REQUEST = "req_codex_conformance01"
EVENT = "evt_codex_conformance01"
PURPOSE = "codex_app_server_conformance"
QUERY = "__impresari_codex_conformance_probe__"
FIXTURE_NAME = "__impresari_codex_conformance_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_codex_conformance_probe__ = true;\n"

options = {
  codex: DEFAULT_CODEX,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  project_config: false,
  isolated_home: nil,
  prepared_project_root: nil,
  temporary_root: nil,
  apply: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-codex-app-server.rb [options]"
  parser.on("--codex PATH", "Codex CLI executable") { |value| options[:codex] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--project-config", "Exercise the observed unsupported repository-config boundary (not an admission path)") do
    options[:project_config] = true
  end
  parser.on("--isolated-home PATH", "Use an empty disposable CODEX_HOME under /private/tmp for the managed configuration rehearsal") do |value|
    options[:isolated_home] = value
  end
  parser.on("--prepare-project-root PATH", "Preview or prepare a disposable workspace/cache root under /private/tmp") do |value|
    options[:prepared_project_root] = value
  end
  parser.on("--temporary-root PATH", "Use a prepared disposable workspace/cache root under /private/tmp") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable workspace/cache preparation") { options[:apply] = true }
end.parse!

[options[:codex], options[:mcp], options[:cli]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

def abort_with(message, stderr = nil)
  detail = stderr&.strip
  abort(detail.nil? || detail.empty? ? message : "#{message}\n#{detail}")
end

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

if options[:prepared_project_root] && options[:temporary_root]
  abort("choose either --prepare-project-root or --temporary-root, not both")
end

if options[:prepared_project_root]
  root = temporary_project_root(options[:prepared_project_root], allow_absent: true)
  layout = prepared_layout(root)
  if !options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_workspace",
      "project_root" => root.to_s,
      "workspace" => layout.fetch(:workspace).to_s,
      "cache" => layout.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. For managed admission, create an empty explicit /private/tmp CODEX_HOME and run with --temporary-root plus --isolated-home.",
    })
    exit(0)
  end
  abort("prepared project root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p([layout.fetch(:workspace), layout.fetch(:cache)])
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_workspace",
    "project_root" => root.to_s,
    "workspace" => layout.fetch(:workspace).to_s,
    "cache" => layout.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Create an empty explicit /private/tmp CODEX_HOME, then run this rehearsal with --temporary-root #{root} and --isolated-home <CODEX_HOME>.",
  })
  exit(0)
end

if options[:project_config] && options[:isolated_home]
  abort("choose either --project-config or --isolated-home, not both")
end

if options[:isolated_home]
  isolated_home = temporary_project_root(options[:isolated_home])
  abort("isolated Codex home must be an empty directory") unless Dir.children(isolated_home).empty?
  options[:isolated_home] = isolated_home.to_s
end

if options[:temporary_root] && !options[:project_config] && !options[:isolated_home]
  abort("--temporary-root requires --project-config or --isolated-home")
end

def source_digest(root)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH)
       .select { |path| File.file?(path) }
       .sort
       .map { |path| "#{path.delete_prefix(root)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def remove_owned_workspace_artifacts(workspace, fixture)
  if File.exist?(fixture)
    abort("refusing to remove an unexpected Codex rehearsal fixture") unless File.file?(fixture) && File.binread(fixture) == FIXTURE_CONTENT
    File.delete(fixture)
  end

  config_directory = File.join(workspace, ".codex")
  return unless Dir.exist?(config_directory)
  return unless Dir.children(config_directory).empty?

  Dir.rmdir(config_directory)
end

def clean_prepared_workspace(workspace)
  return if Dir.children(workspace).empty?

  legacy_fixture = File.join(workspace, "probe.ts")
  fixture = File.join(workspace, FIXTURE_NAME)
  allowed = ["probe.ts", FIXTURE_NAME, ".codex"]
  unexpected = Dir.children(workspace) - allowed
  abort("prepared disposable workspace contains unowned files; refusing to modify it") unless unexpected.empty?

  [legacy_fixture, fixture].each do |candidate|
    next unless File.exist?(candidate)

    abort("prepared disposable workspace contains an unexpected fixture; refusing to modify it") unless File.file?(candidate) && File.binread(candidate) == FIXTURE_CONTENT
    File.delete(candidate)
  end

  config_directory = File.join(workspace, ".codex")
  if Dir.exist?(config_directory)
    abort("prepared disposable workspace contains a nonempty .codex directory; refusing to modify it") unless Dir.children(config_directory).empty?
    Dir.rmdir(config_directory)
  end

  abort("prepared disposable workspace could not be restored to empty") unless Dir.children(workspace).empty?
end

def literal_plan
  [{ "kind" => "literal", "query" => QUERY }]
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
  def initialize(stdin, stdout, stderr)
    @stdin = stdin
    @stdout = stdout
    @stderr = stderr
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
      raise "timed out waiting for Codex App Server response to #{method}" if remaining <= 0
      ready = IO.select([@stdout], nil, nil, remaining)
      raise "Codex App Server closed stdout during #{method}" unless ready
      line = @stdout.gets
      raise "Codex App Server closed stdout during #{method}" if line.nil?
      value = JSON.parse(line)
      next unless value["id"] == id
      raise "Codex App Server error for #{method}: #{JSON.generate(value.fetch('error'))}" if value.key?("error")
      return value.fetch("result")
    end
  rescue JSON::ParserError => e
    raise "Codex App Server emitted invalid JSON: #{e.message}"
  end
end

def tool_payload(result)
  structured = result.dig("content", 0, "text")
  return result.fetch("structuredContent") if result["structuredContent"].is_a?(Hash)
  return JSON.parse(structured) if structured.is_a?(String)
  raise "Codex App Server tool result did not expose a structured payload: #{JSON.generate(result)}"
end

def mcp_server_status_summary(status, server)
  matches = Array(status["data"]).select { |entry| entry.is_a?(Hash) && entry["name"] == server }
  {
    "server" => server,
    "matching_server_count" => matches.length,
    "matches" => matches.map do |entry|
      server_info = entry["serverInfo"]
      {
        "name" => entry["name"],
        "server_info" => server_info.is_a?(Hash) ? {
          "name" => server_info["name"],
          "version" => server_info["version"],
        } : nil,
        "tool_names" => entry.fetch("tools", {}).keys.sort,
        "auth_status" => entry["authStatus"],
      }
    end,
  }
end

def assert_codex_rejects_malformed_configuration(codex, isolated_home)
  config_path = File.join(isolated_home, "config.toml")
  abort("isolated Codex home already contains a configuration file") if File.exist?(config_path)
  File.write(config_path, "[mcp_servers.\"impresari-context\"]\ncommand = [\n")
  stdout, stderr, status = Open3.capture3(
    { "CODEX_HOME" => isolated_home }, codex, "mcp", "list",
  )
  File.delete(config_path) if File.exist?(config_path)
  abort_with("Codex accepted a malformed isolated-home configuration", "#{stderr}\n#{stdout}") if status.success?
end

def direct_mcp_packet(executable, server_args)
  stdin, stdout, stderr, wait = Open3.popen3(executable, *server_args)
  stderr_buffer = +""
  stderr_reader = Thread.new { stderr.each_line { |line| stderr_buffer << line } }
  begin
    rpc = Rpc.new(stdin, stdout, stderr)
    rpc.call("initialize", {
      "protocolVersion" => "2025-11-25",
      "capabilities" => {},
      "clientInfo" => { "name" => "impresari-context-direct-conformance", "version" => "1.0" },
    })
    stdin.puts(JSON.generate({ "jsonrpc" => "2.0", "method" => "notifications/initialized" }))
    stdin.flush
    open = tool_payload(rpc.call("tools/call", {
      "name" => "context_session_open", "arguments" => { "session_id" => SESSION },
    }))
    raise "direct MCP session open was not acknowledged" unless open["opened"] == true
    build = tool_payload(rpc.call("tools/call", {
      "name" => "context_build",
      "arguments" => {
        "request_id" => REQUEST,
        "event_id" => EVENT,
        "purpose" => PURPOSE,
        "occurred_at" => FIXED_TIME,
        "steps" => literal_plan,
        "budget" => conservative_budget,
        "session_id" => SESSION,
      },
    }))
    raise "direct MCP context build did not return a packet: #{JSON.generate(build)}" unless build["packet"].is_a?(Hash)
    packet = build.fetch("packet")
    close = tool_payload(rpc.call("tools/call", {
      "name" => "context_session_close", "arguments" => { "session_id" => SESSION },
    }))
    raise "direct MCP session close was not acknowledged" unless close["closed"] == true
    packet
  ensure
    stdin.close unless stdin.closed?
    stdout.close unless stdout.closed?
    wait.join(10)
    stderr_reader.join(10)
    abort_with("direct MCP process failed", stderr_buffer) if wait.value && !wait.value.success?
  end
end

def with_rehearsal_root(options)
  if options[:temporary_root]
    root = temporary_project_root(options[:temporary_root])
    layout = prepared_layout(root)
    [root, layout.fetch(:workspace), layout.fetch(:cache)].each do |path|
      abort("prepared project layout is missing: #{path}") unless path.directory?
    end
    clean_prepared_workspace(layout.fetch(:workspace))
    yield(root.to_s)
  else
    Dir.mktmpdir("impresari-codex-app-server-") { |temporary| yield(temporary) }
  end
end

with_rehearsal_root(options) do |temporary|
  workspace = File.join(temporary, "workspace")
  run_nonce = "#{Process.pid}-#{Process.clock_gettime(Process::CLOCK_MONOTONIC, :nanosecond)}"
  direct_mcp_cache = File.join(temporary, "direct-mcp-cache-#{run_nonce}")
  codex_mcp_cache = if options[:temporary_root]
                      File.join(temporary, "cache", run_nonce)
                    else
                      File.join(temporary, "codex-mcp-cache-#{run_nonce}")
                    end
  cli_cache = File.join(temporary, "cli-cache-#{run_nonce}")
  FileUtils.mkdir_p([workspace, direct_mcp_cache, codex_mcp_cache, cli_cache])
  fixture = File.join(workspace, FIXTURE_NAME)
  File.write(fixture, FIXTURE_CONTENT)

  direct_server_args = [
    "--workspace", workspace,
    "--cache", direct_mcp_cache,
    "--consumer-id", CONSUMER,
    "--role", "local_user",
    "--occurred-at", FIXED_TIME,
  ]
  if options[:project_config] || options[:isolated_home]
    direct_server_args[direct_server_args.index("--consumer-id") + 1] = "consumer_codex_managed"
  end
  codex_server_args = direct_server_args.dup
  codex_server_args[codex_server_args.index(direct_mcp_cache)] = codex_mcp_cache
  stdin = stdout = stderr = wait = stderr_reader = nil
  stderr_buffer = +""
  begin
  assert_codex_rejects_malformed_configuration(options[:codex], options[:isolated_home]) if options[:isolated_home]
  if options[:project_config] || options[:isolated_home]
    config_directory = options[:isolated_home] || File.join(workspace, ".codex")
    FileUtils.mkdir_p(config_directory)
    config_path = File.join(config_directory, "config.toml")
    install_stdout, install_stderr, install_status = Open3.capture3(
      options[:cli], "client", "kit", "install", "codex", options[:mcp], workspace, codex_mcp_cache, config_path, "--apply",
    )
    abort_with("managed Codex configuration install failed", "#{install_stderr}\n#{install_stdout}") unless install_status.success?
    validate_stdout, validate_stderr, validate_status = Open3.capture3(
      options[:cli], "client", "kit", "validate", "codex", options[:mcp], workspace, codex_mcp_cache, config_path,
    )
    abort_with("managed Codex configuration validation failed", "#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
    if options[:isolated_home]
      registration_stdout, registration_stderr, registration_status = Open3.capture3(
        { "CODEX_HOME" => options[:isolated_home] }, options[:codex], "mcp", "get", SERVER,
      )
      abort_with("Codex did not accept the managed isolated-home configuration", "#{registration_stderr}\n#{registration_stdout}") unless registration_status.success?
    end
  end
  before = source_digest(workspace)
  direct_packet = direct_mcp_packet(options[:mcp], direct_server_args)
  command = if options[:project_config] || options[:isolated_home]
              [options[:codex], "app-server", "--stdio"]
            else
              config = [
                "mcp_servers.#{SERVER}.command=#{options[:mcp].to_json}",
                "mcp_servers.#{SERVER}.args=#{codex_server_args.to_json}",
                "mcp_servers.#{SERVER}.enabled=true",
                "mcp_servers.#{SERVER}.required=true",
              ]
              [options[:codex]] + config.flat_map { |entry| ["-c", entry] } + ["app-server", "--stdio"]
            end
  codex_environment = options[:isolated_home] ? { "CODEX_HOME" => options[:isolated_home] } : {}
  stdin, stdout, stderr, wait = Open3.popen3(codex_environment, *command, chdir: workspace)
  stderr_reader = Thread.new { stderr.each_line { |line| stderr_buffer << line } }

    rpc = Rpc.new(stdin, stdout, stderr)
    rpc.call("initialize", {
      "clientInfo" => { "name" => "impresari-context-conformance", "version" => "1.0" },
      "capabilities" => {},
    })
    thread = rpc.call("thread/start", {
      "cwd" => workspace,
      "approvalPolicy" => "never",
      "sandbox" => "read-only",
      "ephemeral" => true,
    })
    thread_id = thread.fetch("thread").fetch("id")
    status = rpc.call("mcpServerStatus/list", { "threadId" => thread_id, "detail" => "toolsAndAuthOnly" })
    status_summary = mcp_server_status_summary(status, SERVER)
    status_json = JSON.generate(status_summary)
    if status_summary.fetch("matching_server_count").zero?
      detail = if options[:isolated_home]
                 "Codex did not load the temporary isolated-home configuration"
               elsif options[:project_config]
                 "Codex did not load the temporary project configuration"
               else
        "Codex did not expose the dedicated MCP server"
               end
      raise "#{detail}; observed MCP status: #{status_json}"
    end

    begin
      open = tool_payload(rpc.call("mcpServer/tool/call", {
        "server" => SERVER,
        "threadId" => thread_id,
        "tool" => "context_session_open",
        "arguments" => { "session_id" => SESSION },
      }))
    rescue RuntimeError => error
      raise "Codex registered the temporary server but did not make it callable: #{error.message}; observed MCP status: #{status_json}"
    end
    raise "session open was not acknowledged" unless open["opened"] == true

    build = tool_payload(rpc.call("mcpServer/tool/call", {
      "server" => SERVER,
      "threadId" => thread_id,
      "tool" => "context_build",
      "arguments" => {
        "request_id" => REQUEST,
        "event_id" => EVENT,
        "purpose" => PURPOSE,
        "occurred_at" => FIXED_TIME,
        "steps" => literal_plan,
        "budget" => conservative_budget,
        "session_id" => SESSION,
      },
    }))
    raise "context build did not return a packet: #{JSON.generate(build)}" unless build["packet"].is_a?(Hash)
    packet = build.fetch("packet")
    packet_id = packet.fetch("packet_id")
    reference = build.fetch("reference")
    raise "packet reference was not attached to the owning session" unless reference.fetch("packet_id") == packet_id

    resolved = tool_payload(rpc.call("mcpServer/tool/call", {
      "server" => SERVER,
      "threadId" => thread_id,
      "tool" => "context_packet_resolve",
      "arguments" => { "session_id" => SESSION, "packet_id" => packet_id },
    }))
    raise "resolved packet differs from Codex-delivered packet" unless resolved.fetch("packet") == packet

    close = tool_payload(rpc.call("mcpServer/tool/call", {
      "server" => SERVER,
      "threadId" => thread_id,
      "tool" => "context_session_close",
      "arguments" => { "session_id" => SESSION },
    }))
    raise "session close was not acknowledged" unless close["closed"] == true

    raise "direct MCP packet differs from the packet delivered through Codex" unless direct_packet == packet
    doctor_command = [
      options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF,
      "doctor", "mcp", workspace, cli_cache,
    ]
    doctor_stdout, doctor_stderr, doctor_status = Open3.capture3(*doctor_command)
    abort_with("direct-engine/MCP equivalence check failed", doctor_stderr) unless doctor_status.success?
    doctor = JSON.parse(doctor_stdout)
    doctor_checks = doctor.fetch("checks")
    equivalent = doctor_checks.find { |check| check["id"] == "direct_engine_mcp_packet_equivalence" }
    unless equivalent && equivalent["status"] == "passed" && doctor_checks.none? { |check| check["status"] == "failed" }
      raise "direct-engine/MCP equivalence check reported failure: #{JSON.generate(doctor)}"
    end
    raise "source workspace changed during the conformance rehearsal" unless before == source_digest(workspace)

    puts JSON.generate({
      "status" => "passed",
      "codex" => options[:codex],
      "configuration_source" => if options[:isolated_home]
                                    "temporary_isolated_codex_home"
                                  elsif options[:project_config]
                                    "temporary_project"
                                  else
                                    "one_use_override"
                                  end,
      "server" => SERVER,
      "mcp_server_status" => status_summary,
      "packet_id" => packet_id,
      "source_immutable" => true,
      "direct_engine_mcp_equivalence" => true,
      "raw_mcp_codex_packet_equivalence" => true,
      "malformed_configuration_rejected" => !options[:isolated_home].nil?,
      "tool_lifecycle" => ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"],
    })
  ensure
    stdin.close if stdin && !stdin.closed?
    stdout.close if stdout && !stdout.closed?
    wait.join(10) if wait
    stderr_reader.join(10) if stderr_reader
    if (options[:project_config] || options[:isolated_home]) && defined?(config_path) && File.exist?(config_path)
      remove_stdout, remove_stderr, remove_status = Open3.capture3(
        options[:cli], "client", "kit", "remove", "codex", options[:mcp], workspace, codex_mcp_cache, config_path, "--apply",
      )
      abort_with("managed Codex configuration removal failed", "#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
      abort("managed Codex configuration target was not removed") if File.exist?(config_path)
    end
    if options[:isolated_home]
      removed_stdout, removed_stderr, removed_status = Open3.capture3(
        { "CODEX_HOME" => options[:isolated_home] }, options[:codex], "mcp", "get", SERVER,
      )
      abort_with("Codex retained the removed isolated-home MCP entry", "#{removed_stderr}\n#{removed_stdout}") if removed_status.success?
    end
    remove_owned_workspace_artifacts(workspace, fixture) if options[:temporary_root] && defined?(fixture)
  end
end
