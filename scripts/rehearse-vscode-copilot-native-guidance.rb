#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Disposable VS Code Copilot L2 native-guidance rehearsal. It prepares only a
# named workspace below /private/tmp. A signed-in operator makes every VS Code
# trust and tool-approval decision. This script never launches VS Code, edits a
# profile, reads a real repository, or promotes a conversational smoke test.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_NAME = "__impresari_vscode_native_guidance_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_vscode_native_guidance_probe__ = true;\n"
RECORDED_VSCODE_VERSION = "1.134.0"

options = {
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  prepare_root: nil,
  temporary_root: nil,
  apply: false,
  confirmed_discovery: false,
  confirmed_guidance_reference: false,
  confirmed_session_lifecycle: false,
  confirmed_packet_build: false,
  confirmed_packet_resolve: false,
  vscode_version: nil,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-vscode-copilot-native-guidance.rb [options]"
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-root PATH", "Preview or create an empty disposable root below /private/tmp") do |value|
    options[:prepare_root] = value
  end
  parser.on("--temporary-root PATH", "Verify and clean up an explicitly prepared disposable root") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Permit creation or removal in the named disposable root") { options[:apply] = true }
  parser.on("--confirmed-discovery", "Record visible Impresari server discovery in VS Code") do
    options[:confirmed_discovery] = true
  end
  parser.on("--confirmed-guidance-reference", "Record visible application of the owned guidance file") do
    options[:confirmed_guidance_reference] = true
  end
  parser.on("--confirmed-session-lifecycle", "Record visible context_session_open and context_session_close calls") do
    options[:confirmed_session_lifecycle] = true
  end
  parser.on("--confirmed-packet-build", "Record one successful context_build result") do
    options[:confirmed_packet_build] = true
  end
  parser.on("--confirmed-packet-resolve", "Record one successful context_packet_resolve result") do
    options[:confirmed_packet_resolve] = true
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

def extension_host_config(workspace)
  workspace.join(".vscode", "mcp.json")
end

def guidance_file(workspace)
  workspace.join(".github", "instructions", "impresari-context.instructions.md")
end

def source_digest(workspace)
  excluded = [".vscode/mcp.json", ".github/instructions/impresari-context.instructions.md"]
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(workspace, "**", "*"), File::FNM_DOTMATCH)
       .select do |path|
         relative = path.delete_prefix("#{workspace}/")
         File.file?(path) && !excluded.include?(relative) && !relative.start_with?(".git/")
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
  config = extension_host_config(paths.fetch(:workspace))
  guidance = guidance_file(paths.fetch(:workspace))
  unless options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_vscode_copilot_native_guidance",
      "root" => root.to_s,
      "workspace" => paths.fetch(:workspace).to_s,
      "cache" => paths.fetch(:cache).to_s,
      "configuration" => config.to_s,
      "guidance" => guidance.to_s,
      "configuration_surface" => "workspace_extension_host",
      "external_write_performed" => false,
      "next_step" => "Review the disposable paths, then rerun with --apply. No VS Code profile, real source workspace, trust state, or server process is selected.",
    })
    exit(0)
  end
  abort("prepared root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p(paths.values)
  FileUtils.mkdir_p(config.dirname)
  FileUtils.mkdir_p(guidance.dirname)
  File.write(paths.fetch(:workspace).join(FIXTURE_NAME), FIXTURE_CONTENT)
  before = source_digest(paths.fetch(:workspace))
  installed = run_json(
    options[:cli], "client", "kit", "install", "vscode", options[:mcp],
    paths.fetch(:workspace).to_s, paths.fetch(:cache).to_s, config.to_s, "--apply",
  )
  abort("managed VS Code entry was not explicitly installed") unless installed["external_write_performed"] == true
  guidance_install = run_json(
    options[:cli], "client", "guidance", "install", "copilot", paths.fetch(:workspace).to_s, "--apply",
  )
  abort("owned Copilot guidance was not explicitly installed") unless guidance_install["external_write_performed"] == true
  doctor = run_json(
    options[:cli], "doctor", "vscode-config", paths.fetch(:workspace).to_s,
    paths.fetch(:cache).to_s, config.to_s,
  )
  abort("extension-host VS Code configuration was rejected by doctor") unless doctor.dig("checks", 4, "status") == "passed"
  guidance_validation = run_json(
    options[:cli], "client", "guidance", "validate", "copilot", paths.fetch(:workspace).to_s,
  )
  abort("owned Copilot guidance was rejected by validation") unless guidance_validation["state"] == "owned"
  abort("source changed during native-guidance preparation") unless source_digest(paths.fetch(:workspace)) == before
  puts JSON.generate({
    "status" => "prepared_for_manual_native_guidance_evidence",
    "operation" => "prepare_disposable_vscode_copilot_native_guidance",
    "root" => root.to_s,
    "workspace" => paths.fetch(:workspace).to_s,
    "cache" => paths.fetch(:cache).to_s,
    "configuration" => config.to_s,
    "guidance" => guidance.to_s,
    "configuration_surface" => "workspace_extension_host",
    "source_digest" => before,
    "external_write_performed" => true,
    "required_manual_steps" => [
      "Open the named workspace in a new signed-in VS Code window and make VS Code's own trust decision.",
      "Use MCP: List Servers and confirm the local impresari-context server is visible and started. Do not enable MCP sandboxing or automatic approvals.",
      "Confirm that .github/instructions/impresari-context.instructions.md is listed in chat references or diagnostics for the probe request.",
      "In Agent Chat, ask exactly for one bounded packet for #{FIXTURE_NAME}: open a session, call context_build using its live-schema direct filename example with current schema values, resolve the returned packet ID in the same session, then close it. Do not accept a direct file read as packet evidence.",
      "Approve only each visible no-authority Impresari tool call for this chat session, retain the chat tool-result record, then close the temporary workspace/server.",
      "Run this script with --temporary-root, all five confirmation flags, the exact observed VS Code version, and --apply to validate and remove only the owned configuration and guidance files.",
    ],
    "launch_command" => "'/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code' --new-window '#{paths.fetch(:workspace)}'",
  })
  exit(0)
end

root = disposable_root(options[:temporary_root])
paths = layout(root)
paths.each_value { |path| abort("prepared layout is missing: #{path}") unless path.directory? }
config = extension_host_config(paths.fetch(:workspace))
guidance = guidance_file(paths.fetch(:workspace))
abort("extension-host workspace configuration is missing") unless config.file? && !config.symlink?
abort("owned Copilot guidance is missing") unless guidance.file? && !guidance.symlink?
abort("--confirmed-discovery is required to record the manual VS Code observation") unless options[:confirmed_discovery]
abort("--confirmed-guidance-reference is required to record the manual VS Code observation") unless options[:confirmed_guidance_reference]
abort("--confirmed-session-lifecycle is required to record the manual VS Code observation") unless options[:confirmed_session_lifecycle]
abort("--confirmed-packet-build is required to record the manual VS Code observation") unless options[:confirmed_packet_build]
abort("--confirmed-packet-resolve is required to record the manual VS Code observation") unless options[:confirmed_packet_resolve]
abort("--vscode-version must equal #{RECORDED_VSCODE_VERSION}") unless options[:vscode_version] == RECORDED_VSCODE_VERSION
abort("--apply is required before removing the owned disposable artifacts") unless options[:apply]

before = source_digest(paths.fetch(:workspace))
doctor = run_json(
  options[:cli], "doctor", "vscode-config", paths.fetch(:workspace).to_s,
  paths.fetch(:cache).to_s, config.to_s,
)
abort("extension-host VS Code configuration became invalid before removal") unless doctor.dig("checks", 4, "status") == "passed"
guidance_validation = run_json(
  options[:cli], "client", "guidance", "validate", "copilot", paths.fetch(:workspace).to_s,
)
abort("owned Copilot guidance became invalid before removal") unless guidance_validation["state"] == "owned"
guidance_removed = run_json(
  options[:cli], "client", "guidance", "remove", "copilot", paths.fetch(:workspace).to_s, "--apply",
)
abort("owned Copilot guidance was not explicitly removed") unless guidance_removed["external_write_performed"] == true
removed = run_json(
  options[:cli], "client", "kit", "remove", "vscode", options[:mcp],
  paths.fetch(:workspace).to_s, paths.fetch(:cache).to_s, config.to_s, "--apply",
)
abort("managed VS Code entry was not explicitly removed") unless removed["external_write_performed"] == true
abort("extension-host workspace configuration still exists after exact owned removal") if config.exist?
abort("owned Copilot guidance still exists after exact removal") if guidance.exist?
abort("source changed during native-guidance evidence capture") unless source_digest(paths.fetch(:workspace)) == before
puts JSON.generate({
  "status" => "manual_native_guidance_evidence_recorded",
  "operation" => "verify_disposable_vscode_copilot_native_guidance",
  "configuration_surface" => "workspace_extension_host",
  "vscode_version" => options[:vscode_version],
  "confirmed_server_discovery" => true,
  "confirmed_guidance_reference" => true,
  "confirmed_session_lifecycle" => true,
  "confirmed_packet_build" => true,
  "confirmed_packet_resolve" => true,
  "source_unchanged" => true,
  "owned_configuration_removed" => true,
  "owned_guidance_removed" => true,
  "external_write_performed" => true,
  "first_class_l2_claim" => false,
  "next_step" => "Review the retained chat tool-result record and the source-free cleanup record before deciding whether the manual observation supports public L2 admission.",
})
