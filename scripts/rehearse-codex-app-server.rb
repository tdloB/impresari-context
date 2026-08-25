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

options = {
  codex: DEFAULT_CODEX,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  project_config: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-codex-app-server.rb [options]"
  parser.on("--codex PATH", "Codex CLI executable") { |value| options[:codex] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--project-config", "Load the temporary project's .codex/config.toml instead of one-use overrides") do
    options[:project_config] = true
  end
end.parse!

[options[:codex], options[:mcp], options[:cli]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

def abort_with(message, stderr = nil)
  detail = stderr&.strip
  abort(detail.nil? || detail.empty? ? message : "#{message}\n#{detail}")
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
    raise "direct MCP context build did not return a packet" unless build["packet"].is_a?(Hash)
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

Dir.mktmpdir("impresari-codex-app-server-") do |temporary|
  workspace = File.join(temporary, "workspace")
  direct_mcp_cache = File.join(temporary, "direct-mcp-cache")
  codex_mcp_cache = File.join(temporary, "codex-mcp-cache")
  cli_cache = File.join(temporary, "cli-cache")
  FileUtils.mkdir_p([workspace, direct_mcp_cache, codex_mcp_cache, cli_cache])
  fixture = File.join(workspace, "probe.ts")
  File.write(fixture, "export const __impresari_codex_conformance_probe__ = true;\n")

  direct_server_args = [
    "--workspace", workspace,
    "--cache", direct_mcp_cache,
    "--consumer-id", CONSUMER,
    "--role", "local_user",
    "--occurred-at", FIXED_TIME,
  ]
  codex_server_args = direct_server_args.dup
  codex_server_args[codex_server_args.index(direct_mcp_cache)] = codex_mcp_cache
  if options[:project_config]
    config_directory = File.join(workspace, ".codex")
    FileUtils.mkdir_p(config_directory)
    config_path = File.join(config_directory, "config.toml")
    install_stdout, install_stderr, install_status = Open3.capture3(
      options[:cli], "client", "kit", "install", "codex", options[:mcp], workspace, codex_mcp_cache, config_path, "--apply",
    )
    abort_with("managed Codex project configuration install failed", "#{install_stderr}\n#{install_stdout}") unless install_status.success?
    validate_stdout, validate_stderr, validate_status = Open3.capture3(
      options[:cli], "client", "kit", "validate", "codex", options[:mcp], workspace, codex_mcp_cache, config_path,
    )
    abort_with("managed Codex project configuration validation failed", "#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
  end
  before = source_digest(workspace)
  direct_packet = direct_mcp_packet(options[:mcp], direct_server_args)
  command = if options[:project_config]
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
  stdin, stdout, stderr, wait = Open3.popen3(*command, chdir: workspace)
  stderr_buffer = +""
  stderr_reader = Thread.new { stderr.each_line { |line| stderr_buffer << line } }

  begin
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
    status_json = JSON.generate(status)
    unless status_json.include?(SERVER)
      detail = options[:project_config] ?
        "Codex did not load the temporary project configuration; trust the project through Codex before claiming project-scope admission" :
        "Codex did not expose the dedicated MCP server"
      raise detail
    end

    open = tool_payload(rpc.call("mcpServer/tool/call", {
      "server" => SERVER,
      "threadId" => thread_id,
      "tool" => "context_session_open",
      "arguments" => { "session_id" => SESSION },
    }))
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
      "configuration_source" => options[:project_config] ? "temporary_project" : "one_use_override",
      "server" => SERVER,
      "packet_id" => packet_id,
      "source_immutable" => true,
      "direct_engine_mcp_equivalence" => true,
      "raw_mcp_codex_packet_equivalence" => true,
      "tool_lifecycle" => ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"],
    })
  ensure
    stdin.close unless stdin.closed?
    stdout.close unless stdout.closed?
    wait.join(10)
    stderr_reader.join(10)
    if options[:project_config] && defined?(config_path) && File.exist?(config_path)
      remove_stdout, remove_stderr, remove_status = Open3.capture3(
        options[:cli], "client", "kit", "remove", "codex", options[:mcp], workspace, codex_mcp_cache, config_path, "--apply",
      )
      abort_with("managed Codex project configuration removal failed", "#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
      abort("managed Codex project configuration target was not removed") if File.exist?(config_path)
    end
  end
end
