#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Disposable VS Code Copilot L1 admission rehearsal. It prepares only a named
# workspace below /private/tmp. A signed-in operator performs the VS Code trust,
# discovery, and tool-invocation checks; this script never launches VS Code,
# changes a user profile, or claims admission from an unchecked prompt.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_NAME = "__impresari_vscode_agent_host_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_vscode_agent_host_probe__ = true;\n"
RECORDED_VSCODE_VERSION = "1.134.0"

options = {
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  prepare_root: nil,
  temporary_root: nil,
  apply: false,
  confirmed_discovery: false,
  confirmed_tool_invocation: false,
  vscode_version: nil,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-vscode-copilot-portable-agent-host.rb [options]"
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-root PATH", "Preview or create an empty disposable root below /private/tmp") do |value|
    options[:prepare_root] = value
  end
  parser.on("--temporary-root PATH", "Verify and remove an explicitly prepared disposable root") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Permit creation or removal in the named disposable root") { options[:apply] = true }
  parser.on("--confirmed-discovery", "Record that the operator saw the named server in VS Code MCP UI") do
    options[:confirmed_discovery] = true
  end
  parser.on("--confirmed-tool-invocation", "Record that the operator saw one Impresari MCP tool invocation") do
    options[:confirmed_tool_invocation] = true
  end
  parser.on("--vscode-version VERSION", "Exact VS Code version observed by the operator") do |value|
    options[:vscode_version] = value
  end
end.parse!

[options[:cli], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

abort("choose either --prepare-root or --temporary-root, not both") if options[:prepare_root] && options[:temporary_root]
abort("one of --prepare-root or --temporary-root is required") unless options[:prepare_root] || options[:temporary_root]

def disposable_root(path, allow_absent: false)
  candidate = Pathname.new(path).expand_path
  parent = Pathname.new("/private/tmp").realpath
  resolved = if candidate.exist?
               candidate.realpath
             elsif allow_absent
               candidate.parent.realpath.join(candidate.basename)
             else
               abort("prepared root does not exist: #{candidate}")
             end
  abort("disposable root must be below /private/tmp") unless resolved.to_s.start_with?("#{parent}/")
  abort("disposable root must not be a symbolic link") if candidate.symlink?
  resolved
end

def layout(root)
  { workspace: root.join("workspace"), cache: root.join("cache") }
end

def source_digest(workspace)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(workspace, "**", "*"), File::FNM_DOTMATCH)
       .select do |path|
         relative = path.delete_prefix("#{workspace}/")
         File.file?(path) && relative != ".mcp.json" && !relative.start_with?(".git/")
       end
       .sort
       .map { |path| "#{path.delete_prefix(workspace.to_s)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def run_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  JSON.parse(stdout)
end

if options[:prepare_root]
  root = disposable_root(options[:prepare_root], allow_absent: true)
  paths = layout(root)
  config = paths.fetch(:workspace).join(".mcp.json")
  unless options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_vscode_copilot_agent_host",
      "root" => root.to_s,
      "workspace" => paths.fetch(:workspace).to_s,
      "cache" => paths.fetch(:cache).to_s,
      "configuration" => config.to_s,
      "external_write_performed" => false,
      "next_step" => "Review the disposable paths, then rerun with --apply. No VS Code profile, real source workspace, trust state, or server process is selected.",
    })
    exit(0)
  end
  abort("prepared root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p(paths.values)
  File.write(paths.fetch(:workspace).join(FIXTURE_NAME), FIXTURE_CONTENT)
  before = source_digest(paths.fetch(:workspace))
  installed = run_json(
    options[:cli], "client", "kit", "install", "vscode", options[:mcp],
    paths.fetch(:workspace).to_s, paths.fetch(:cache).to_s, config.to_s, "--apply",
  )
  abort("managed VS Code entry was not explicitly installed") unless installed["external_write_performed"] == true
  doctor = run_json(
    options[:cli], "doctor", "vscode-config", paths.fetch(:workspace).to_s,
    paths.fetch(:cache).to_s, config.to_s,
  )
  abort("portable VS Code configuration was rejected by doctor") unless doctor.dig("checks", 4, "status") == "passed"
  abort("source changed during portable configuration preparation") unless source_digest(paths.fetch(:workspace)) == before
  puts JSON.generate({
    "status" => "prepared_for_manual_client_evidence",
    "operation" => "prepare_disposable_vscode_copilot_agent_host",
    "root" => root.to_s,
    "workspace" => paths.fetch(:workspace).to_s,
    "cache" => paths.fetch(:cache).to_s,
    "configuration" => config.to_s,
    "source_digest" => before,
    "external_write_performed" => true,
    "required_manual_steps" => [
      "Open the named workspace in a new VS Code window using the signed-in default VS Code profile.",
      "Review the exact .mcp.json entry and make VS Code's own trust/enable decision; do not enable MCP sandboxing or change automatic approvals.",
      "Use MCP: List Servers and verify the server named impresari-context and its fixed local command are visible.",
      "In Agent Chat, enable only the Impresari tools, ask for the probe's context, and record whether one named Impresari MCP tool was invoked. Model tool choice is not required to be repeatable.",
      "Close the temporary workspace/server, then run this script with --temporary-root, both confirmation flags, the exact observed VS Code version, and --apply to validate and remove only the owned entry.",
    ],
    "launch_command" => "'/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code' --new-window '#{paths.fetch(:workspace)}'",
  })
  exit(0)
end

root = disposable_root(options[:temporary_root])
paths = layout(root)
paths.each_value { |path| abort("prepared layout is missing: #{path}") unless path.directory? }
config = paths.fetch(:workspace).join(".mcp.json")
abort("portable workspace configuration is missing") unless config.file? && !config.symlink?
abort("--confirmed-discovery is required to record the manual VS Code observation") unless options[:confirmed_discovery]
abort("--confirmed-tool-invocation is required to record the manual VS Code observation") unless options[:confirmed_tool_invocation]
abort("--vscode-version must equal #{RECORDED_VSCODE_VERSION}") unless options[:vscode_version] == RECORDED_VSCODE_VERSION
abort("--apply is required before removing the owned disposable entry") unless options[:apply]

before = source_digest(paths.fetch(:workspace))
doctor = run_json(
  options[:cli], "doctor", "vscode-config", paths.fetch(:workspace).to_s,
  paths.fetch(:cache).to_s, config.to_s,
)
abort("portable VS Code configuration became invalid before removal") unless doctor.dig("checks", 4, "status") == "passed"
removed = run_json(
  options[:cli], "client", "kit", "remove", "vscode", options[:mcp],
  paths.fetch(:workspace).to_s, paths.fetch(:cache).to_s, config.to_s, "--apply",
)
abort("managed VS Code entry was not explicitly removed") unless removed["external_write_performed"] == true
abort("portable workspace configuration still exists after exact owned removal") if config.exist?
abort("source changed during VS Code client evidence capture") unless source_digest(paths.fetch(:workspace)) == before
puts JSON.generate({
  "status" => "manual_client_evidence_recorded",
  "operation" => "verify_disposable_vscode_copilot_agent_host",
  "vscode_version" => options[:vscode_version],
  "confirmed_server_discovery" => true,
  "confirmed_impresari_tool_invocation" => true,
  "source_unchanged" => true,
  "owned_configuration_removed" => true,
  "external_write_performed" => true,
  "first_class_admission_claim" => false,
  "next_step" => "Review the retained source-free record and separately decide whether the manually observed client evidence is sufficient for the public L1 admission claim.",
})
