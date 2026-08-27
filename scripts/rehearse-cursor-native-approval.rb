#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Native Cursor MCP approval lifecycle rehearsal. It uses only a caller-named
# disposable project under /private/tmp. Cursor's local approved-list state is
# changed only for the exact server identifier and is disabled again before the
# project configuration is removed.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CURSOR = "/Applications/Cursor.app/Contents/Resources/app/bin/cursor"
SERVER = "impresari-context"
FIXTURE_NAME = "__impresari_cursor_native_approval_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_cursor_native_approval_probe__ = true;\n"
FIXED_TIME = "2026-08-26T00:00:00Z"
SESSION = "session_cursor_conformance01"
REQUEST = "req_cursor_conformance01"
EVENT = "evt_cursor_conformance01"
PURPOSE = "cursor_agent_conformance"
QUERY = "__impresari_cursor_native_approval_probe__"
EXPECTED_TOOLS = %w[
  context_session_open
  context_build
  context_packet_resolve
  context_session_close
].freeze

options = {
  cursor: DEFAULT_CURSOR,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  prepared_root: nil,
  temporary_root: nil,
  apply: false,
  model_smoke: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-cursor-native-approval.rb [options]"
  parser.on("--cursor PATH", "Cursor CLI executable") { |value| options[:cursor] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-root PATH", "Preview or prepare a disposable project root under /private/tmp") do |value|
    options[:prepared_root] = value
  end
  parser.on("--temporary-root PATH", "Run against a prepared disposable project root under /private/tmp") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable-root preparation") { options[:apply] = true }
  parser.on("--model-smoke", "Run a bounded read-only model-directed MCP lifecycle smoke check") { options[:model_smoke] = true }
end.parse!

[options[:cursor], options[:cli], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

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
    workspace: root.join("workspace"),
    cache: root.join("cache"),
  }
end

def source_digest(workspace)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(workspace, "**", "*"), File::FNM_DOTMATCH)
       .select { |path| File.file?(path) && !path.delete_prefix("#{workspace}/").start_with?(".cursor/") }
       .sort
       .map { |path| "#{path.delete_prefix(workspace)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def run_command(*command, chdir:)
  stdout, stderr, status = Open3.capture3(*command, chdir: chdir)
  abort("command failed (#{command.join(' ')}):\n#{stderr}\n#{stdout}") unless status.success?
  [stdout, stderr]
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
      "clientInfo" => { "name" => "impresari-context-cursor-conformance", "version" => "1.0" },
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

def cursor_mcp_calls(events)
  starts = events.filter do |event|
    event["type"] == "tool_call" && event["subtype"] == "started" && event.dig("tool_call", "mcpToolCall", "args", "toolName").is_a?(String)
  end
  started = starts.map do |event|
    [event.fetch("call_id"), event.dig("tool_call", "mcpToolCall", "args")]
  end.to_h
  completed = events.filter do |event|
    event["type"] == "tool_call" && event["subtype"] == "completed" && started.key?(event["call_id"])
  end
  completed.map do |event|
    [
      started.fetch(event.fetch("call_id")).fetch("toolName"),
      event.dig("tool_call", "mcpToolCall", "result"),
    ]
  end
end

def cursor_tool_result_payload(result)
  success = result["success"]
  abort("Cursor MCP tool returned an error: #{JSON.generate(result)}") unless success.is_a?(Hash) && success["isError"] == false
  content = success["content"]
  text = if content.is_a?(String)
           content
         elsif content.is_a?(Array)
           content.map { |part| part.dig("text", "text") if part.is_a?(Hash) }.compact.join
         end
  abort("Cursor MCP tool result is missing structured content: #{JSON.generate(result)}") unless text.is_a?(String)
  JSON.parse(text)
rescue JSON::ParserError => error
  abort("Cursor MCP tool result was not JSON: #{error.message}: #{JSON.generate(result)}")
end

abort("choose either --prepare-root or --temporary-root, not both") if options[:prepared_root] && options[:temporary_root]

if options[:prepared_root]
  root = temporary_root(options[:prepared_root], allow_absent: true)
  paths = layout(root)
  unless options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_cursor_native_approval",
      "root" => root.to_s,
      "workspace" => paths.fetch(:workspace).to_s,
      "cache" => paths.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. The native rehearsal will add and later disable only the exact Impresari server in Cursor's local approved list.",
    })
    exit(0)
  end
  abort("prepared root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p(paths.values)
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_cursor_native_approval",
    "root" => root.to_s,
    "workspace" => paths.fetch(:workspace).to_s,
    "cache" => paths.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Run this rehearsal with --temporary-root #{root}.",
  })
  exit(0)
end

abort("--temporary-root is required for the native approval rehearsal") unless options[:temporary_root]
root = temporary_root(options[:temporary_root])
paths = layout(root)
paths.each_value { |path| abort("prepared layout is missing: #{path}") unless path.directory? }

workspace = paths.fetch(:workspace).to_s
cache = paths.fetch(:cache).to_s
server_cache = File.join(cache, "cursor-mcp-#{Process.pid}")
fixture = File.join(workspace, FIXTURE_NAME)
if File.file?(fixture) && File.binread(fixture) == FIXTURE_CONTENT && Dir.children(workspace) == [FIXTURE_NAME]
  File.delete(fixture)
end
abort("disposable Cursor workspace must be empty before registration") unless Dir.children(workspace).empty?
cursor_directory = File.join(workspace, ".cursor")
config_path = File.join(workspace, ".cursor", "mcp.json")
permissions_path = File.join(workspace, ".cursor", "cli.json")
File.write(fixture, FIXTURE_CONTENT)
before = source_digest(workspace)
installed = false
enabled = false
created_cursor_directory = false
model_smoke_passed = false
permissions_content = JSON.generate({
  "permissions" => {
    "allow" => EXPECTED_TOOLS.map { |name| "Mcp(#{SERVER}:#{name})" },
    "deny" => ["Shell(*)", "Read(*)", "Write(*)", "WebFetch(*)"],
  },
})

begin
  FileUtils.mkdir_p(cursor_directory)
  FileUtils.mkdir_p(server_cache)
  created_cursor_directory = true
  File.write(permissions_path, permissions_content)
  install_stdout, = run_command(
    options[:cli], "client", "kit", "install", "cursor", options[:mcp], workspace, server_cache, config_path, "--apply",
    chdir: workspace,
  )
  install = JSON.parse(install_stdout)
  abort("managed Cursor configuration did not report an explicit write") unless install["external_write_performed"] == true
  installed = true

  run_command(
    options[:cli], "client", "kit", "validate", "cursor", options[:mcp], workspace, server_cache, config_path,
    chdir: workspace,
  )
  listed_stdout, = run_command(options[:cursor], "agent", "mcp", "list", chdir: workspace)
  abort("Cursor did not discover the disposable MCP entry") unless listed_stdout.include?(SERVER)

  run_command(options[:cursor], "agent", "mcp", "enable", SERVER, chdir: workspace)
  enabled = true
  tools_stdout, = run_command(options[:cursor], "agent", "mcp", "list-tools", SERVER, chdir: workspace)
  missing_tools = EXPECTED_TOOLS.reject { |name| tools_stdout.include?(name) }
  abort("Cursor did not expose the fixed Impresari tool set: #{missing_tools.join(', ')}") unless missing_tools.empty?
  if options[:model_smoke]
    direct_mcp_cache = File.join(cache, "direct-mcp-#{Process.pid}")
    FileUtils.mkdir_p(direct_mcp_cache)
    direct_packet = direct_mcp_packet(options[:mcp], [
      "--workspace", workspace,
      "--cache", direct_mcp_cache,
      "--consumer-id", "consumer_cursor_managed",
      "--role", "local_user",
      "--occurred-at", FIXED_TIME,
    ])
    prompt = <<~PROMPT
      Perform this exact read-only Impresari Context MCP lifecycle. Do not use
      any shell, file, web, codebase, or non-MCP tool, and do not change files.
      Call these MCP tools in this order: context_session_open with session_id
      #{SESSION}; context_build with request_id #{REQUEST}, event_id #{EVENT},
      purpose #{PURPOSE}, occurred_at #{FIXED_TIME}, steps
      [{"kind":"literal","query":"#{QUERY}"}], budget
      #{JSON.generate(conservative_budget)}, and the same session_id;
      context_packet_resolve with that session_id and the returned packet_id;
      then context_session_close with that session_id.
      Reply only after the four MCP calls finish.
    PROMPT
    model_stdout, model_stderr, model_status = Open3.capture3(
      options[:cursor], "agent", "--print",
      "--output-format", "stream-json", "--approve-mcps", "--sandbox", "enabled", "--trust", prompt,
      chdir: workspace,
    )
    abort("Cursor model-directed MCP smoke check failed:\n#{model_stderr}\n#{model_stdout}") unless model_status.success?
    events = model_stdout.lines.map do |line|
      JSON.parse(line)
    rescue JSON::ParserError
      nil
    end.compact
    model_results = cursor_mcp_calls(events)
    observed_tools = model_results.map(&:first)
    unless observed_tools == EXPECTED_TOOLS
      abort("Cursor model-directed MCP smoke check did not expose the expected lifecycle: #{observed_tools.join(', ')}\n#{model_stdout}")
    end
    payloads = model_results.to_h { |name, result| [name, cursor_tool_result_payload(result)] }
    built_packet = payloads.fetch("context_build").fetch("packet")
    resolved_packet = payloads.fetch("context_packet_resolve").fetch("packet")
    abort("Cursor resolved packet differs from its delivered packet") unless resolved_packet == built_packet
    abort("direct MCP packet differs from Cursor-delivered packet") unless direct_packet == built_packet
    model_smoke_passed = true
  end
  abort("source workspace changed during Cursor approval lifecycle") unless before == source_digest(workspace)
ensure
  if enabled
    disable_stdout, disable_stderr, disable_status = Open3.capture3(
      options[:cursor], "agent", "mcp", "disable", SERVER, chdir: workspace,
    )
    abort("Cursor native approval removal failed:\n#{disable_stderr}\n#{disable_stdout}") unless disable_status.success?
  end
  if installed
    remove_stdout, remove_stderr, remove_status = Open3.capture3(
      options[:cli], "client", "kit", "remove", "cursor", options[:mcp], workspace, server_cache, config_path, "--apply",
      chdir: workspace,
    )
    abort("managed Cursor configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
    abort("managed Cursor configuration target was not removed") if File.exist?(config_path)
  end
  if created_cursor_directory
    abort("refusing to remove an unexpected Cursor permissions file") unless File.file?(permissions_path) && File.binread(permissions_path) == permissions_content
    File.delete(permissions_path)
    unexpected = Dir.children(cursor_directory)
    abort("Cursor rehearsal configuration directory contains unexpected files: #{unexpected.join(', ')}") unless unexpected.empty?
    Dir.rmdir(cursor_directory)
  end
  abort("source workspace changed during Cursor approval removal") unless before == source_digest(workspace)
  abort("refusing to remove an unexpected Cursor rehearsal fixture") unless File.file?(fixture) && File.binread(fixture) == FIXTURE_CONTENT
  File.delete(fixture)
end

abort("disposable Cursor workspace is not empty after exact rehearsal cleanup") unless Dir.children(workspace).empty?

puts JSON.generate({
  "status" => "passed",
  "cursor" => options[:cursor],
  "configuration_scope" => "project",
  "native_enable_list_tools_disable" => true,
  "managed_install_validate_remove" => true,
  "source_immutable" => true,
  "expected_tools" => EXPECTED_TOOLS,
  "model_directed_smoke" => model_smoke_passed,
})
