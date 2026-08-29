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
DEFAULT_CODEX = "/Applications/ChatGPT.app/Contents/Resources/codex"
CLIENT_VERSION = "0.150.0-alpha.8"
FIXED_TIME = "2026-08-28T12:00:00Z"
FIXED_CUTOFF = "2026-08-01T00:00:00Z"
FIXTURE = "authentication.rs"
FIXTURE_BYTES = "pub fn authenticate() {}\n"

options = { cli: DEFAULT_CLI, codex: DEFAULT_CODEX, runs: 2 }
OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-codex-guided-delivery.rb [options]"
  parser.on("--cli PATH", "Impresari Context CLI executable") { |value| options[:cli] = value }
  parser.on("--codex PATH", "Codex CLI executable") { |value| options[:codex] = value }
  parser.on("--runs COUNT", Integer, "Successful runs required (default: 2)") { |value| options[:runs] = value }
end.parse!

abort("runs must be between 2 and 10") unless (2..10).cover?(options[:runs])
[options[:cli], options[:codex]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

def run_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError => error
  abort("command returned invalid JSON: #{command.join(' ')}\n#{error.message}\n#{stdout}")
end

version_stdout, version_stderr, version_status = Open3.capture3(options[:codex], "--version")
abort("Codex version check failed: #{version_stderr}") unless version_status.success?
version = version_stdout.strip.delete_prefix("codex-cli ")
abort("unsupported Codex version: #{version}") unless version == CLIENT_VERSION

records = []
options[:runs].times do |index|
  Dir.mktmpdir("impresari-codex-l3-", "/private/tmp") do |temporary_root|
    workspace = File.join(temporary_root, "workspace")
    cache = File.join(temporary_root, "cache")
    runtime = File.join(temporary_root, "runtime")
    artifacts = File.join(temporary_root, "artifacts")
    [workspace, cache, runtime, artifacts].each { |path| FileUtils.mkdir_p(path) }
    fixture = File.join(workspace, FIXTURE)
    File.binwrite(fixture, FIXTURE_BYTES)
    source_before = Digest::SHA256.file(fixture).hexdigest
    common = [options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF]
    snapshot = run_json(*common, "--id-seed", "codexl3snapshot#{index}", "snapshot", "build", workspace, cache)
    intent = {
      "adapter_contract_version" => "1.0.0",
      "client" => "codex",
      "scope" => "app_server_ephemeral",
      "client_version" => CLIENT_VERSION,
      "lifecycle_point" => "turn_start",
      "consent" => true,
      "request_id" => "req_codexl3delivery#{index}",
      "event_id" => "evt_codexl3delivery#{index}",
      "consumer_id" => "consumer_codex_l3_admission",
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
    preview = run_json(*common, "--id-seed", "codexl3preview#{index}", "client", "delivery", "codex", "preview", workspace, cache, intent_path)
    abort("delivery was not prepared: #{JSON.generate(preview)}") unless preview["state"] == "prepared"
    packet_id = preview.fetch("value").fetch("delivery_envelope").fetch("packet_id")
    preview_path = File.join(artifacts, "preview.json")
    File.write(preview_path, JSON.generate(preview))
    receipt = run_json(*common, "--apply", "client", "delivery", "codex", "apply", preview_path, runtime, options[:codex], packet_id)
    abort("Codex delivery did not complete: #{JSON.generate(receipt)}") unless receipt["outcome"] == "delivered"
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
      "authority_added" => receipt.fetch("authority_added"),
      "approval_requests_declined" => receipt.fetch("approval_requests_declined")
    }
  end
end

puts JSON.pretty_generate({
  "status" => "passed",
  "client" => "codex",
  "client_version" => version,
  "platform" => RUBY_PLATFORM,
  "successful_runs" => records.length,
  "records" => records
})
