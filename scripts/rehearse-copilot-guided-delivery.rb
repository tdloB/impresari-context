#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CLI = ROOT.join("target/debug/impresari-context").to_s
DEFAULT_COPILOT = "/opt/homebrew/bin/copilot"
CLIENT_VERSION = "1.0.80"
FIXED_TIME = "2026-08-28T12:00:00Z"
FIXED_CUTOFF = "2026-08-01T00:00:00Z"
FIXTURE = "authentication.rs"
FIXTURE_BYTES = "pub fn authenticate() {}\n"

options = { cli: DEFAULT_CLI, copilot: DEFAULT_COPILOT, runs: 2, copilot_home: nil, github_auth_config: nil }
OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-copilot-guided-delivery.rb [options]"
  parser.on("--cli PATH", "Impresari Context CLI executable") { |value| options[:cli] = value }
  parser.on("--copilot PATH", "GitHub Copilot CLI executable") { |value| options[:copilot] = value }
  parser.on("--copilot-home PATH", "Dedicated authenticated Copilot home") { |value| options[:copilot_home] = value }
  parser.on("--github-auth-config PATH", "Existing GitHub CLI auth directory used in place") { |value| options[:github_auth_config] = value }
  parser.on("--runs COUNT", Integer, "Successful runs required (default: 2)") { |value| options[:runs] = value }
end.parse!

abort("runs must be between 2 and 10") unless (2..10).cover?(options[:runs])
abort("--copilot-home is required") unless options[:copilot_home]
abort("--github-auth-config is required") unless options[:github_auth_config]
[options[:cli], options[:copilot]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end
abort("Copilot home must be a real directory") unless File.directory?(options[:copilot_home]) && !File.symlink?(options[:copilot_home])
options[:copilot_home] = File.realpath(options[:copilot_home])
abort("GitHub auth config must be a real directory") unless File.directory?(options[:github_auth_config]) && !File.symlink?(options[:github_auth_config])
options[:github_auth_config] = File.realpath(options[:github_auth_config])

def run_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError => error
  abort("command returned invalid JSON: #{command.join(' ')}\n#{error.message}\n#{stdout}")
end

version_stdout, version_stderr, version_status = Open3.capture3(options[:copilot], "--version")
abort("Copilot version check failed: #{version_stderr}") unless version_status.success?
version = version_stdout.lines.first.to_s.strip.delete_prefix("GitHub Copilot CLI ").delete_suffix(".")
abort("unsupported Copilot version: #{version}") unless version == CLIENT_VERSION

records = []
options[:runs].times do |index|
  Dir.mktmpdir("impresari-copilot-ci3c-", "/private/tmp") do |temporary_root|
    workspace = File.join(temporary_root, "workspace")
    cache = File.join(temporary_root, "cache")
    runtime = File.join(temporary_root, "runtime")
    artifacts = File.join(temporary_root, "artifacts")
    [workspace, cache, runtime, artifacts].each { |path| FileUtils.mkdir_p(path) }
    fixture = File.join(workspace, FIXTURE)
    File.binwrite(fixture, FIXTURE_BYTES)
    source_before = Digest::SHA256.file(fixture).hexdigest
    common = [options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF]
    snapshot = run_json(*common, "--id-seed", "copilotci3csnapshot#{index}", "snapshot", "build", workspace, cache)
    intent = {
      "adapter_contract_version" => "1.0.0",
      "client" => "github_copilot_cli",
      "scope" => "programmatic_prompt",
      "client_version" => CLIENT_VERSION,
      "lifecycle_point" => "prompt_start",
      "consent" => true,
      "request_id" => "req_copilotci3cdelivery#{index}",
      "event_id" => "evt_copilotci3cdelivery#{index}",
      "consumer_id" => "consumer_copilot_ci3c_admission",
      "role" => "local_user",
      "purpose" => "implementation",
      "occurred_at" => FIXED_TIME,
      "workspace_identity" => snapshot.fetch("workspace_identity"),
      "workspace_snapshot" => snapshot.fetch("snapshot_id"),
      "task_profile" => "implementation",
      "query" => "authenticate",
      "budget" => {
        "unit_kind" => "utf8_bytes", "requested" => "8192", "hard" => true,
        "max_evidence_items" => "20", "max_files" => "100",
        "max_excerpt_bytes_per_item" => "128", "max_matches" => "100",
        "max_traversal_depth" => "16", "max_elapsed_ms" => "30000",
        "max_memory_bytes" => "67108864",
        "policy_profile" => "sha256:aba86621046ccc86cff7aabb81f4eab1020ab6db53ae1b649ea3977dec9649e8"
      }
    }
    intent_path = File.join(artifacts, "intent.json")
    File.write(intent_path, JSON.generate(intent))
    preview = run_json(*common, "--id-seed", "copilotci3cpreview#{index}", "client", "delivery", "copilot", "preview", workspace, cache, intent_path)
    abort("delivery was not prepared: #{JSON.generate(preview)}") unless preview["state"] == "prepared"
    packet_id = preview.fetch("value").fetch("delivery_envelope").fetch("packet_id")
    preview_path = File.join(artifacts, "preview.json")
    File.write(preview_path, JSON.generate(preview))
    receipt = run_json(*common, "--apply", "client", "delivery", "copilot", "apply", preview_path, runtime, options[:copilot], options[:copilot_home], options[:github_auth_config], packet_id)
    abort("Copilot delivery did not complete: #{JSON.generate(receipt)}") unless receipt["outcome"] == "delivered"
    abort("receipt did not bind the dedicated Copilot home") unless receipt["authenticated_copilot_home_used"] == true
    abort("receipt did not select GitHub auth in place") unless receipt["github_auth_config_used_in_place"] == true
    abort("terminal result was not observed") unless receipt["terminal_result_observed"] == true
    abort("tool execution was observed") unless receipt["tool_executions_observed"] == 0
    abort("source workspace was exposed") unless receipt["source_workspace_exposed"] == false
    abort("receipt claimed credential copying") unless receipt["credential_state_copied"] == false
    abort("receipt claimed credential deletion") unless receipt["credential_state_deleted"] == false
    abort("receipt claimed added authority") unless receipt["authority_added"] == false
    abort("source changed during delivery") unless Digest::SHA256.file(fixture).hexdigest == source_before
    abort("runtime cleanup failed") unless Dir.children(runtime).empty?
    records << {
      "run" => index + 1,
      "packet_id" => packet_id,
      "plan_id" => preview.dig("value", "prepared", "plan", "plan_id"),
      "workspace_snapshot" => snapshot.fetch("snapshot_id"),
      "outcome" => receipt.fetch("outcome"),
      "source_sha256" => source_before,
      "source_immutable" => true,
      "runtime_clean" => true,
      "terminal_result_observed" => receipt.fetch("terminal_result_observed"),
      "tool_executions_observed" => receipt.fetch("tool_executions_observed"),
      "provider_network_required" => receipt.fetch("provider_network_required"),
      "source_workspace_exposed" => receipt.fetch("source_workspace_exposed"),
      "authority_added" => receipt.fetch("authority_added"),
      "authenticated_copilot_home_used" => receipt.fetch("authenticated_copilot_home_used"),
      "github_auth_config_used_in_place" => receipt.fetch("github_auth_config_used_in_place"),
      "credential_state_copied" => receipt.fetch("credential_state_copied"),
      "credential_state_deleted" => receipt.fetch("credential_state_deleted")
    }
  end
end

puts JSON.pretty_generate({
  "status" => "passed",
  "client" => "github_copilot_cli",
  "client_version" => version,
  "platform" => RUBY_PLATFORM,
  "successful_runs" => records.length,
  "records" => records
})
