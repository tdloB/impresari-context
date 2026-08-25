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
SERVER = "impresari-context"

options = {
  cursor: DEFAULT_CURSOR,
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-cursor-preadmission.rb [options]"
  parser.on("--cursor PATH", "Cursor CLI executable") { |value| options[:cursor] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
end.parse!

[options[:cursor], options[:cli], options[:mcp]].each do |path|
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
  before = tree_digest(workspace)
  config_path = File.join(cursor_directory, "mcp.json")
  install_stdout, install_stderr, install_status = Open3.capture3(
    options[:cli], "client", "kit", "install", "cursor", options[:mcp], workspace, cache, config_path, "--apply",
  )
  abort("managed Cursor configuration install failed:\n#{install_stderr}\n#{install_stdout}") unless install_status.success?
  install = JSON.parse(install_stdout)
  abort("managed Cursor configuration did not report an explicit write") unless install["external_write_performed"] == true
  stdout, stderr, status = Open3.capture3(
    options[:cursor], "agent", "mcp", "list", chdir: workspace,
  )
  abort("Cursor Agent MCP listing failed:\n#{stderr}\n#{stdout}") unless status.success?
  abort("Cursor Agent did not discover the temporary MCP server:\n#{stdout}") unless stdout.include?(SERVER)
  validate_stdout, validate_stderr, validate_status = Open3.capture3(
    options[:cli], "client", "kit", "validate", "cursor", options[:mcp], workspace, cache, config_path,
  )
  abort("managed Cursor configuration validation failed:\n#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
  remove_stdout, remove_stderr, remove_status = Open3.capture3(
    options[:cli], "client", "kit", "remove", "cursor", options[:mcp], workspace, cache, config_path, "--apply",
  )
  abort("managed Cursor configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
  abort("managed Cursor configuration target was not removed") if File.exist?(config_path)
  abort("Cursor Agent unexpectedly altered the temporary workspace") unless before == tree_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "cursor" => options[:cursor],
    "server" => SERVER,
    "workspace_immutable_after_configuration" => true,
    "managed_install_validate_remove" => true,
    "mcp_approval_granted" => false,
  })
end
