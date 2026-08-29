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
DEFAULT_CURSOR = "/Users/aaronboldt/.local/bin/cursor-agent"
CLIENT_VERSION = "2026.08.25-3e8eec8"
FIXED_TIME = "2026-08-29T12:00:00Z"
FIXED_CUTOFF = "2026-08-01T00:00:00Z"
FIXTURE = "authentication.rs"
FIXTURE_BYTES = "pub fn authenticate() {}\n"

options = { cli: DEFAULT_CLI, cursor: DEFAULT_CURSOR, runs: 2, user_home: nil }
OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-cursor-guided-delivery.rb [options]"
  parser.on("--cli PATH", "Impresari Context CLI executable") { |value| options[:cli] = value }
  parser.on("--cursor PATH", "Cursor Agent executable") { |value| options[:cursor] = value }
  parser.on("--user-home PATH", "Existing authenticated user home used in place") { |value| options[:user_home] = value }
  parser.on("--runs COUNT", Integer, "Successful runs required (default: 2)") { |value| options[:runs] = value }
end.parse!

abort("runs must be between 2 and 10") unless (2..10).cover?(options[:runs])
abort("--user-home is required") unless options[:user_home]
[options[:cli], options[:cursor]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end
abort("user home must be a real directory") unless File.directory?(options[:user_home]) && !File.symlink?(options[:user_home])
options[:user_home] = File.realpath(options[:user_home])

def run_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError => error
  abort("command returned invalid JSON: #{command.join(' ')}\n#{error.message}\n#{stdout}")
end

version_stdout, version_stderr, version_status = Open3.capture3(options[:cursor], "--version")
abort("Cursor version check failed: #{version_stderr}") unless version_status.success?
version_output = version_stdout.lines.first.to_s.strip
abort("unsupported Cursor version: #{version_output}") unless version_output == CLIENT_VERSION
version = CLIENT_VERSION

records = []
options[:runs].times do |index|
  Dir.mktmpdir("impresari-cursor-ci3e-", "/private/tmp") do |temporary_root|
    workspace = File.join(temporary_root, "workspace")
    cache = File.join(temporary_root, "cache")
    runtime = File.join(temporary_root, "runtime")
    artifacts = File.join(temporary_root, "artifacts")
    [workspace, cache, runtime, artifacts].each { |path| FileUtils.mkdir_p(path) }
    fixture = File.join(workspace, FIXTURE)
    File.binwrite(fixture, FIXTURE_BYTES)
    source_before = Digest::SHA256.file(fixture).hexdigest
    common = [options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF]
    snapshot = run_json(*common, "--id-seed", "cursorci3esnapshot#{index}", "snapshot", "build", workspace, cache)
    intent = {
      "adapter_contract_version" => "1.0.0",
      "client" => "cursor_agent",
      "scope" => "ask_mode_print",
      "client_version" => CLIENT_VERSION,
      "lifecycle_point" => "prompt_start",
      "consent" => true,
      "request_id" => "req_cursorci3edelivery#{index}",
      "event_id" => "evt_cursorci3edelivery#{index}",
      "consumer_id" => "consumer_cursor_ci3e_admission",
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
    preview = run_json(*common, "--id-seed", "cursorci3epreview#{index}", "client", "delivery", "cursor", "preview", workspace, cache, intent_path)
    abort("delivery was not prepared: #{JSON.generate(preview)}") unless preview["state"] == "prepared"
    packet_id = preview.fetch("value").fetch("delivery_envelope").fetch("packet_id")
    preview_path = File.join(artifacts, "preview.json")
    File.write(preview_path, JSON.generate(preview))
    receipt = run_json(*common, "--apply", "client", "delivery", "cursor", "apply", preview_path, runtime, options[:cursor], options[:user_home], packet_id)
    abort("Cursor delivery did not complete: #{JSON.generate(receipt)}") unless receipt["outcome"] == "delivered"
    abort("receipt did not bind the authenticated Cursor home") unless receipt["authenticated_cursor_home_used"] == true
    abort("receipt did not select the user home in place") unless receipt["authenticated_user_home_used_in_place"] == true
    abort("Cursor authentication status was not verified") unless receipt["authentication_status_verified"] == true
    expected_environment_inheritance = ENV.key?("CURSOR_API_KEY")
    unless receipt["provider_auth_environment_inherited"] == expected_environment_inheritance
      abort("provider authentication environment evidence mismatch")
    end
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
      "authenticated_cursor_home_used" => receipt.fetch("authenticated_cursor_home_used"),
      "authenticated_user_home_used_in_place" => receipt.fetch("authenticated_user_home_used_in_place"),
      "authentication_status_verified" => receipt.fetch("authentication_status_verified"),
      "provider_auth_environment_inherited" => receipt.fetch("provider_auth_environment_inherited"),
      "credential_state_copied" => receipt.fetch("credential_state_copied"),
      "credential_state_deleted" => receipt.fetch("credential_state_deleted")
    }
  end
end

puts JSON.pretty_generate({
  "status" => "passed",
  "client" => "cursor_agent",
  "client_version" => version,
  "platform" => RUBY_PLATFORM,
  "successful_runs" => records.length,
  "records" => records
})
