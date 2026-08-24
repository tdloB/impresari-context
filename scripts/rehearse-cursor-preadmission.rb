#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Non-mutating Cursor Agent MCP preadmission rehearsal. It exercises Cursor's
# own project configuration parser without enabling an MCP server or starting a
# model session.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CURSOR = "/Applications/Cursor.app/Contents/Resources/app/bin/cursor"
SERVER = "impresari_context_conformance"

options = {
  cursor: DEFAULT_CURSOR,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-cursor-preadmission.rb [options]"
  parser.on("--cursor PATH", "Cursor CLI executable") { |value| options[:cursor] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
end.parse!

[options[:cursor], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
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

Dir.mktmpdir("impresari-cursor-preadmission-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  cursor_directory = File.join(workspace, ".cursor")
  FileUtils.mkdir_p([cache, cursor_directory])
  File.write(
    File.join(workspace, "probe.ts"),
    "export const __impresari_cursor_preadmission_probe__ = true;\n",
  )
  File.write(File.join(cursor_directory, "mcp.json"), JSON.generate({
    "mcpServers" => {
      SERVER => {
        "command" => options[:mcp],
        "args" => [
          "--workspace", workspace,
          "--cache", cache,
          "--consumer-id", "consumer_cursor_conformance",
          "--role", "local_user",
        ],
      },
    },
  }))
  before = tree_digest(workspace)
  stdout, stderr, status = Open3.capture3(
    options[:cursor], "agent", "mcp", "list", chdir: workspace,
  )
  abort("Cursor Agent MCP listing failed:\n#{stderr}\n#{stdout}") unless status.success?
  abort("Cursor Agent did not discover the temporary MCP server:\n#{stdout}") unless stdout.include?(SERVER)
  abort("Cursor Agent unexpectedly altered the temporary workspace") unless before == tree_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "cursor" => options[:cursor],
    "server" => SERVER,
    "workspace_immutable_after_configuration" => true,
    "mcp_approval_granted" => false,
  })
end
