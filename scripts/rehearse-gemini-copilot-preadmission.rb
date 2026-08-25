#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Isolated, non-mutating Gemini CLI and GitHub Copilot CLI MCP discovery.
# This never starts a model session, grants tool permissions, or writes either
# client's persistent configuration.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
SERVER = "impresari-context"
TOOLS = %w[context_session_open context_build context_packet_resolve context_session_close].freeze

options = {
  gemini: "gemini",
  copilot: "copilot",
  cli: ROOT.join("target/debug/impresari-context").to_s,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  malformed_copilot_config_only: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-gemini-copilot-preadmission.rb [options]"
  parser.on("--gemini PATH", "Gemini CLI executable") { |value| options[:gemini] = value }
  parser.on("--skip-gemini", "Skip Gemini discovery after an external client failure") { options[:gemini] = nil }
  parser.on("--copilot PATH", "Copilot CLI executable") { |value| options[:copilot] = value }
  parser.on("--cli PATH", "Impresari CLI executable") { |value| options[:cli] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--malformed-copilot-config-only", "Verify Copilot rejects malformed temporary MCP configuration") do
    options[:malformed_copilot_config_only] = true
  end
end.parse!

abort("missing executable: #{options[:mcp]}") unless File.file?(options[:mcp]) && File.executable?(options[:mcp])
abort("missing executable: #{options[:cli]}") unless File.file?(options[:cli]) && File.executable?(options[:cli])

def tree_digest(root)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH)
       .select { |path| File.file?(path) }
       .sort
       .map { |path| "#{path.delete_prefix(root)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

def run(*command, chdir:)
  stdout, stderr, status = Open3.capture3(*command, chdir: chdir)
  abort("command failed: #{command.join(" ")}\n#{stderr}\n#{stdout}") unless status.success?
  stdout
end

Dir.mktmpdir("impresari-gemini-copilot-preadmission-") do |temporary|
  workspace = File.join(temporary, "workspace")
  cache = File.join(temporary, "cache")
  FileUtils.mkdir_p([workspace, cache, File.join(workspace, ".gemini")])
  abort("could not initialize temporary Gemini project") unless system("git", "init", "-q", workspace)
  File.write(File.join(workspace, "probe.ts"), "export const __impresari_agent_probe__ = true;\n")
  entry = {
    "command" => options[:mcp],
    "args" => ["--workspace", workspace, "--cache", cache, "--consumer-id", "consumer_phase2_conformance", "--role", "local_user"],
  }
  File.write(File.join(workspace, ".gemini", "settings.json"), JSON.generate({
    "mcpServers" => { SERVER => entry.merge("trust" => false, "includeTools" => TOOLS) },
  }))
  copilot_config = File.join(temporary, "copilot-mcp.json")
  malformed_copilot_config = File.join(temporary, "malformed-copilot-mcp.json")
  before = tree_digest(workspace)
  File.write(malformed_copilot_config, "{\"mcpServers\":")
  malformed_stdout, malformed_stderr, malformed_status = Open3.capture3(
    options[:copilot],
    "--additional-mcp-config", "@#{malformed_copilot_config}",
    "--disable-builtin-mcps",
    "--no-remote",
    "--no-auto-update",
    "--no-custom-instructions",
    "--log-dir", temporary,
    "--output-format", "json",
    "--prompt", "Do not call any tool.",
    chdir: workspace,
  )
  if malformed_status.success?
    abort("Copilot accepted malformed temporary MCP configuration")
  end
  malformed_output = "#{malformed_stdout}\n#{malformed_stderr}"
  if malformed_output.include?("__impresari_agent_probe__")
    abort("Copilot malformed configuration diagnostic exposed fixture source")
  end
  abort("temporary workspace was altered during malformed Copilot configuration rehearsal") unless before == tree_digest(workspace)
  if options[:malformed_copilot_config_only]
    puts JSON.generate({
      "status" => "passed",
      "copilot_malformed_configuration_rejected" => true,
      "workspace_immutable_after_configuration" => true,
      "persistent_mcp_configuration_changed" => false,
    })
    next
  end
  install_stdout, install_stderr, install_status = Open3.capture3(
    options[:cli], "client", "kit", "install", "copilot", options[:mcp], workspace, cache, copilot_config, "--apply",
  )
  abort("managed Copilot configuration install failed:\n#{install_stderr}\n#{install_stdout}") unless install_status.success?
  install = JSON.parse(install_stdout)
  abort("managed Copilot configuration did not report an explicit write") unless install["external_write_performed"] == true
  validate_stdout, validate_stderr, validate_status = Open3.capture3(
    options[:cli], "client", "kit", "validate", "copilot", options[:mcp], workspace, cache, copilot_config,
  )
  abort("managed Copilot configuration validation failed:\n#{validate_stderr}\n#{validate_stdout}") unless validate_status.success?
  gemini_discovered = false
  unless options[:gemini].nil?
    gemini = run(
      options[:gemini],
      "--approval-mode", "default",
      "--allowed-mcp-server-names", SERVER,
      "--output-format", "stream-json",
      "--prompt", "Use the configured Impresari Context MCP server to call context_session_open exactly once. Do not use any other tool.",
      chdir: workspace,
    )
    abort("Gemini did not discover temporary server:\n#{gemini}") unless gemini.include?(SERVER)
    gemini_discovered = true
  end
  copilot = run(
    options[:copilot],
    "--additional-mcp-config", "@#{copilot_config}",
    "--disable-builtin-mcps",
    "--no-remote",
    "--no-auto-update",
    "--no-custom-instructions",
    "--log-dir", temporary,
    "--available-tools", "#{SERVER}(context_session_open)",
    "--allow-tool", "#{SERVER}(context_session_open)",
    "--output-format", "json",
    "--prompt", "Use the configured Impresari Context MCP tool context_session_open exactly once. Do not use any other tool.",
    chdir: workspace,
  )
  abort("Copilot did not discover temporary server:\n#{copilot}") unless copilot.include?(SERVER)
  abort("Copilot did not call the permitted temporary MCP tool:\n#{copilot}") unless copilot.include?("context_session_open")
  remove_stdout, remove_stderr, remove_status = Open3.capture3(
    options[:cli], "client", "kit", "remove", "copilot", options[:mcp], workspace, cache, copilot_config, "--apply",
  )
  abort("managed Copilot configuration removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
  abort("managed Copilot configuration target was not removed") if File.exist?(copilot_config)
  abort("temporary workspace was altered") unless before == tree_digest(workspace)
  puts JSON.generate({
    "status" => "passed",
    "gemini_discovered": gemini_discovered,
    "copilot_discovered": true,
    "copilot_malformed_configuration_rejected": true,
    "copilot_managed_install_validate_remove": true,
    "workspace_immutable_after_configuration" => true,
    "persistent_mcp_configuration_changed" => false,
  })
end
