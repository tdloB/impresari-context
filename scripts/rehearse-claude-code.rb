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
SERVER = "impresari_context_conformance"
CONSUMER = "consumer_claude_conformance"
SESSION = "session_claude_conformance01"
REQUEST = "req_claude_conformance01"
EVENT = "evt_claude_conformance01"
PURPOSE = "claude_code_conformance"
QUERY = "__impresari_claude_conformance_probe__"

options = {
  claude: DEFAULT_CLAUDE,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  malformed_config_only: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-claude-code.rb [options]"
  parser.on("--claude PATH", "Claude Code CLI executable") { |value| options[:claude] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--malformed-config-only", "Verify strict temporary MCP configuration rejection without a model request") do
    options[:malformed_config_only] = true
  end
end.parse!

[options[:claude], options[:mcp]].each do |path|
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

Dir.mktmpdir("impresari-claude-code-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  config_path = File.join(temporary, "mcp.json")
  malformed_config_path = File.join(temporary, "malformed-mcp.json")
  FileUtils.mkdir_p([workspace, cache])
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
  File.write(config_path, JSON.generate({
    "mcpServers" => {
      SERVER => {
        "command" => options[:mcp],
        "args" => [
          "--workspace", workspace,
          "--cache", cache,
          "--consumer-id", CONSUMER,
          "--role", "local_user",
          "--occurred-at", FIXED_TIME,
        ],
      },
    },
  }))

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
  _persistent_stdout, persistent_stderr, persistent_status = Open3.capture3(
    options[:claude], "mcp", "get", SERVER, chdir: workspace,
  )
  if persistent_status.success?
    abort("Claude Code persistent configuration unexpectedly contains #{SERVER}:\n#{persistent_stderr}")
  end
  abort("source workspace changed during Claude Code rehearsal") unless before == source_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "claude" => options[:claude],
    "server" => SERVER,
    "source_immutable" => true,
    "persistent_mcp_registration" => false,
    "malformed_configuration_rejected" => true,
    "tool_lifecycle" => observed_tools,
  })
end
