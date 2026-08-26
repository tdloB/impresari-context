#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Real-client, non-mutating Claude Code MCP admission rehearsal. It uses a
# one-run --mcp-config file in a temporary directory and never registers a
# persistent Claude Code MCP server.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tempfile"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CLAUDE = "/Users/aaronboldt/.local/bin/claude"
FIXED_TIME = "2026-08-23T12:00:00Z"
SERVER = "impresari-context"
SESSION = "session_claude_conformance01"
REQUEST = "req_claude_conformance01"
EVENT = "evt_claude_conformance01"
PURPOSE = "claude_code_conformance"
QUERY = "__impresari_claude_conformance_probe__"

options = {
  claude: DEFAULT_CLAUDE,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  malformed_config_only: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-claude-code.rb [options]"
  parser.on("--claude PATH", "Claude Code CLI executable") { |value| options[:claude] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--malformed-config-only", "Verify strict temporary MCP configuration rejection without a model request") do
    options[:malformed_config_only] = true
  end
end.parse!

[options[:claude], options[:cli], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
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

def tool_input(tool, input)
  JSON.generate({ "tool" => tool, "input" => input })
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
      "clientInfo" => { "name" => "impresari-context-claude-conformance", "version" => "1.0" },
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

def claude_tool_result_payload(block)
  content = block.fetch("content")
  text = if content.is_a?(String)
           content
         elsif content.is_a?(Array)
           content.map { |part| part["text"] if part.is_a?(Hash) }.compact.join
         elsif content.is_a?(Hash)
           content["text"]
         end
  raise "Claude Code tool result has no JSON text payload: #{JSON.generate(block)}" unless text.is_a?(String)
  JSON.parse(text)
rescue JSON::ParserError => error
  raise "Claude Code tool result was not JSON: #{error.message}: #{JSON.generate(block)}"
end

Dir.mktmpdir("impresari-claude-code-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  direct_mcp_cache = File.join(temporary, "direct-mcp-cache")
  config_path = File.join(temporary, "mcp.json")
  malformed_config_path = File.join(temporary, "malformed-mcp.json")
  FileUtils.mkdir_p([workspace, cache, direct_mcp_cache])
  File.write(
    File.join(workspace, "probe.ts"),
    "export const __impresari_claude_conformance_probe__ = true;\n",
  )
  before = source_digest(workspace)
  File.write(malformed_config_path, "{\"mcpServers\":")
  malformed_stdout, malformed_stderr, malformed_status = Open3.capture3(
    options[:claude], "-p", "Do not call any tool.",
    "--mcp-config", malformed_config_path,
    "--strict-mcp-config",
    "--max-turns", "1",
    "--output-format", "json",
    chdir: workspace,
  )
  if malformed_status.success?
    abort("Claude Code accepted malformed MCP configuration")
  end
  malformed_output = "#{malformed_stdout}\n#{malformed_stderr}"
  if malformed_output.include?("__impresari_claude_conformance_probe__")
    abort("Claude Code malformed configuration diagnostic exposed fixture source")
  end
  if options[:malformed_config_only]
    abort("source workspace changed during malformed-config rehearsal") unless before == source_digest(workspace)
    puts JSON.generate({
      "status" => "passed",
      "claude" => options[:claude],
      "malformed_configuration_rejected" => true,
      "source_immutable" => true,
    })
    next
  end
  install_stdout, install_stderr, install_status = Open3.capture3(
    options[:cli], "client", "kit", "install", "claude", options[:mcp], workspace, cache, config_path, "--apply",
  )
  abort("managed Claude configuration install failed:\n#{install_stderr}\n#{install_stdout}") unless install_status.success?
  install = JSON.parse(install_stdout)
  abort("managed Claude configuration did not report an explicit write") unless install["external_write_performed"] == true
  validate_stdout, validate_stderr, validate_status = Open3.capture3(
    options[:cli], "client", "kit", "validate", "claude", options[:mcp], workspace, cache, config_path,
  )
  abort("managed Claude configuration validation failed:\n#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
  direct_packet = direct_mcp_packet(options[:mcp], [
    "--workspace", workspace,
    "--cache", direct_mcp_cache,
    "--consumer-id", "consumer_claude_managed",
    "--role", "local_user",
    "--occurred-at", FIXED_TIME,
  ])

  tools = [
    "mcp__#{SERVER}__context_session_open",
    "mcp__#{SERVER}__context_build",
    "mcp__#{SERVER}__context_packet_resolve",
    "mcp__#{SERVER}__context_session_close",
  ]
  prompt = <<~PROMPT
    Perform this exact MCP conformance lifecycle and do not describe a plan.
    Use only the available MCP tools, in this order.
    1. Call #{tool_input("context_session_open", { "session_id" => SESSION })}.
    2. Call #{tool_input("context_build", {
      "request_id" => REQUEST,
      "event_id" => EVENT,
      "purpose" => PURPOSE,
      "occurred_at" => FIXED_TIME,
      "steps" => [{ "kind" => "literal", "query" => QUERY }],
      "budget" => conservative_budget,
      "session_id" => SESSION,
    })}.
    3. From the build result, call context_packet_resolve with the same session_id
       and its returned packet_id.
    4. Call #{tool_input("context_session_close", { "session_id" => SESSION })}.
    Reply only after all four calls complete.
  PROMPT
  command = [
    options[:claude], "-p", prompt,
    "--mcp-config", config_path,
    "--strict-mcp-config",
    "--tools", "",
    "--allowedTools", tools.join(","),
    "--permission-mode", "dontAsk",
    "--no-session-persistence",
    "--max-turns", "6",
    "--output-format", "stream-json",
    "--verbose",
  ]
  stdout, stderr, status = Open3.capture3(*command, chdir: workspace)
  abort("Claude Code rehearsal failed:\n#{stderr}\n#{stdout}") unless status.success?
  events = stdout.lines.map { |line| JSON.parse(line) }
  observed_tools = events.flat_map do |event|
    content = event.dig("message", "content")
    Array(content).map { |block| block["name"] if block["type"] == "tool_use" }.compact
  end
  tool_results = events.flat_map do |event|
    content = event.dig("message", "content")
    Array(content).select { |block| block["type"] == "tool_result" }
  end
  unless observed_tools == tools
    abort("Claude Code MCP lifecycle differed from the required order: #{observed_tools.join(', ')}\n#{stdout}")
  end
  if tool_results.length != tools.length || tool_results.any? { |result| result["is_error"] == true }
    abort("Claude Code received an MCP tool error or incomplete result set:\n#{stdout}")
  end
  tool_use_names = events.flat_map do |event|
    Array(event.dig("message", "content")).map do |block|
      [block["id"], block["name"]] if block["type"] == "tool_use"
    end.compact
  end.to_h
  results_by_name = tool_results.to_h do |result|
    tool_name = tool_use_names.fetch(result.fetch("tool_use_id"))
    [tool_name, claude_tool_result_payload(result)]
  end
  built_packet = results_by_name.fetch("mcp__#{SERVER}__context_build").fetch("packet")
  resolved_packet = results_by_name.fetch("mcp__#{SERVER}__context_packet_resolve").fetch("packet")
  abort("Claude Code resolved packet differs from its delivered packet") unless resolved_packet == built_packet
  abort("direct MCP packet differs from Claude Code-delivered packet") unless direct_packet == built_packet
  _persistent_stdout, persistent_stderr, persistent_status = Open3.capture3(
    options[:claude], "mcp", "get", SERVER, chdir: workspace,
  )
  if persistent_status.success?
    abort("Claude Code persistent configuration unexpectedly contains #{SERVER}:\n#{persistent_stderr}")
  end
  remove_stdout, remove_stderr, remove_status = Open3.capture3(
    options[:cli], "client", "kit", "remove", "claude", options[:mcp], workspace, cache, config_path, "--apply",
  )
  abort("managed Claude configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
  abort("managed Claude configuration target was not removed") if File.exist?(config_path)
  abort("source workspace changed during Claude Code rehearsal") unless before == source_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "claude" => options[:claude],
    "server" => SERVER,
    "source_immutable" => true,
    "persistent_mcp_registration" => false,
    "malformed_configuration_rejected" => true,
    "managed_install_validate_remove" => true,
    "direct_mcp_packet_equivalence" => true,
    "tool_lifecycle" => observed_tools,
  })
end
