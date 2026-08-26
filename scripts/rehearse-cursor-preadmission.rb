#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Cursor Agent MCP preadmission rehearsal. Its default flow exercises Cursor's
# project configuration parser without enabling an MCP server or starting a
# model session. The optional preview-first preparation mode writes only an
# explicitly named disposable directory under /private/tmp.

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
  prepared_project_root: nil,
  apply: false,
  malformed_config_only: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-cursor-preadmission.rb [options]"
  parser.on("--cursor PATH", "Cursor CLI executable") { |value| options[:cursor] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-project-root PATH", "Preview or prepare a disposable Cursor project root under /private/tmp") do |value|
    options[:prepared_project_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable-project preparation") { options[:apply] = true }
  parser.on("--malformed-config-only", "Verify Cursor does not load malformed temporary MCP configuration without starting a model") do
    options[:malformed_config_only] = true
  end
end.parse!

[options[:cursor], options[:cli], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
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

if options[:prepared_project_root]
  root = temporary_project_root(options[:prepared_project_root], allow_absent: true)
  layout = prepared_layout(root)
  if !options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_cursor_project",
      "project_root" => root.to_s,
      "workspace" => layout.fetch(:workspace).to_s,
      "cache" => layout.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. Install only the rendered .cursor/mcp.json entry in the reported workspace before deciding whether to enable it in Cursor.",
    })
    exit(0)
  end
  abort("prepared project root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p([layout.fetch(:workspace), layout.fetch(:cache)])
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_cursor_project",
    "project_root" => root.to_s,
    "workspace" => layout.fetch(:workspace).to_s,
    "cache" => layout.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Install and inspect only the generated .cursor/mcp.json entry in the reported workspace. A user must explicitly decide whether to run Cursor's mcp enable command for that one temporary entry.",
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

Dir.mktmpdir("impresari-cursor-preadmission-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  malformed_workspace = File.join(temporary, "malformed-workspace")
  cursor_directory = File.join(workspace, ".cursor")
  malformed_cursor_directory = File.join(malformed_workspace, ".cursor")
  FileUtils.mkdir_p([cache, cursor_directory, malformed_cursor_directory])
  File.write(
    File.join(workspace, "probe.ts"),
    "export const __impresari_cursor_preadmission_probe__ = true;\n",
  )
  File.write(
    File.join(malformed_workspace, "probe.ts"),
    "export const __impresari_cursor_malformed_probe__ = true;\n",
  )
  malformed_config_path = File.join(malformed_cursor_directory, "mcp.json")
  File.write(malformed_config_path, "{\"mcpServers\":")
  malformed_before = tree_digest(malformed_workspace)
  malformed_stdout, malformed_stderr, _malformed_status = Open3.capture3(
    options[:cursor], "agent", "mcp", "list", chdir: malformed_workspace,
  )
  malformed_output = "#{malformed_stdout}\n#{malformed_stderr}"
  if malformed_output.include?(SERVER)
    abort("Cursor Agent loaded a server from malformed temporary MCP configuration:\n#{malformed_output}")
  end
  if malformed_output.include?("__impresari_cursor_malformed_probe__")
    abort("Cursor malformed configuration diagnostic exposed fixture source")
  end
  unless malformed_before == tree_digest(malformed_workspace)
    abort("Cursor Agent altered the malformed temporary workspace")
  end
  if options[:malformed_config_only]
    puts JSON.generate({
      "status" => "passed",
      "cursor" => options[:cursor],
      "malformed_configuration_fails_closed" => true,
      "source_immutable" => true,
      "mcp_approval_granted" => false,
    })
    next
  end
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
    "malformed_configuration_fails_closed" => true,
    "mcp_approval_granted" => false,
  })
end
