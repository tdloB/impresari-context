#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CLI = ROOT.join("target/debug/impresari-context").to_s
DEFAULT_CODE = "/usr/local/bin/code"
CLIENT_VERSION = "1.134.0"
FIXED_TIME = "2026-08-29T12:00:00Z"
FIXED_CUTOFF = "2026-08-01T00:00:00Z"
FIXTURE = "authentication.rs"
FIXTURE_BYTES = "pub fn authenticate() {}\n"

options = { cli: DEFAULT_CLI, code: DEFAULT_CODE, user_home: nil, temporary_root: nil, run: nil, apply: false, observed_packet_id: nil }
OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-vscode-copilot-guided-delivery.rb [options]"
  parser.on("--cli PATH", "Impresari Context CLI executable") { |value| options[:cli] = value }
  parser.on("--code PATH", "VS Code CLI executable") { |value| options[:code] = value }
  parser.on("--user-home PATH", "Existing signed-in VS Code user home used in place") { |value| options[:user_home] = value }
  parser.on("--temporary-root PATH", "Caller-created empty rehearsal root") { |value| options[:temporary_root] = value }
  parser.on("--run NUMBER", Integer, "Synthetic run number (1 or 2)") { |value| options[:run] = value }
  parser.on("--apply", "Launch the exact preview in a new VS Code Ask-mode chat") { options[:apply] = true }
  parser.on("--observed-packet-id ID", "Exact packet ID visibly acknowledged by Copilot") { |value| options[:observed_packet_id] = value }
end.parse!

abort("--user-home is required") unless options[:user_home]
abort("--temporary-root is required") unless options[:temporary_root]
abort("--run must be 1 or 2") unless [1, 2].include?(options[:run])
[options[:cli], options[:code]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end
abort("user home must be a real directory") unless File.directory?(options[:user_home]) && !File.symlink?(options[:user_home])
abort("temporary root must be a real directory") unless File.directory?(options[:temporary_root]) && !File.symlink?(options[:temporary_root])

def run_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError => error
  abort("command returned invalid JSON: #{error.message}\n#{stdout}")
end

version_stdout, version_stderr, version_status = Open3.capture3(options[:code], "--version")
abort("VS Code version check failed: #{version_stderr}") unless version_status.success?
abort("unsupported VS Code version") unless version_stdout.lines.first.to_s.strip == CLIENT_VERSION

root = File.realpath(options[:temporary_root])
workspace = File.join(root, "workspace")
cache = File.join(root, "cache")
runtime = File.join(root, "runtime")
artifacts = File.join(root, "artifacts")
fixture = File.join(workspace, FIXTURE)
intent_path = File.join(artifacts, "intent.json")
preview_path = File.join(artifacts, "preview.json")
receipt_path = File.join(artifacts, "launch-receipt.json")
source_hash_path = File.join(artifacts, "source.sha256")

if options[:observed_packet_id]
  abort("launch receipt is missing") unless File.file?(receipt_path)
  receipt = JSON.parse(File.binread(receipt_path))
  packet_id = receipt.fetch("packet_id")
  confirmed = run_json(
    options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF,
    "client", "delivery", "vscode", "confirm", receipt_path,
    packet_id, options[:observed_packet_id]
  )
  abort("VS Code delivery was not confirmed") unless confirmed["outcome"] == "delivered"
  abort("confirmation inferred provider delivery") unless confirmed["provider_delivery_inferred"] == false
  abort("confirmation claimed machine-readable response") unless confirmed["model_response_machine_observable"] == false
  abort("confirmation claimed observable tools") unless confirmed["tool_execution_machine_observable"] == false
  abort("source workspace was exposed") unless confirmed["source_workspace_exposed"] == false
  abort("credentials were touched") unless %w[credential_state_inspected credential_state_copied credential_state_deleted].all? { |key| confirmed[key] == false }
  abort("authority was added") unless confirmed["authority_added"] == false
  abort("source changed") unless Digest::SHA256.file(fixture).hexdigest == File.binread(source_hash_path).strip
  abort("runtime cleanup failed") unless Dir.children(runtime).empty?
  preview = JSON.parse(File.binread(preview_path))
  puts JSON.pretty_generate({
    "status" => "passed",
    "run" => options[:run],
    "client" => "vscode_copilot",
    "client_version" => CLIENT_VERSION,
    "packet_id" => packet_id,
    "plan_id" => preview.dig("value", "prepared", "plan", "plan_id"),
    "workspace_snapshot" => confirmed.fetch("workspace_snapshot"),
    "source_sha256" => File.binread(source_hash_path).strip,
    "source_immutable" => true,
    "runtime_clean" => true,
    "operator_confirmation_observed" => true,
    "source_workspace_exposed" => false,
    "provider_delivery_inferred" => false,
    "authority_added" => false
  })
  exit
end

if options[:apply]
  abort("preview is missing") unless File.file?(preview_path)
  abort("runtime must be empty") unless File.directory?(runtime) && Dir.children(runtime).empty?
  preview = JSON.parse(File.binread(preview_path))
  packet_id = preview.dig("value", "delivery_envelope", "packet_id") or abort("packet ID missing")
  receipt = run_json(
    options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF, "--apply",
    "client", "delivery", "vscode", "apply", preview_path, runtime,
    options[:code], File.realpath(options[:user_home]), packet_id
  )
  abort("VS Code launch did not require confirmation") unless receipt["outcome"] == "confirmation_required"
  abort("launcher inferred provider delivery") unless receipt["provider_delivery_inferred"] == false
  abort("source changed during launch") unless Digest::SHA256.file(fixture).hexdigest == File.binread(source_hash_path).strip
  abort("runtime cleanup failed") unless Dir.children(runtime).empty?
  File.binwrite(receipt_path, JSON.generate(receipt))
  puts JSON.pretty_generate({
    "status" => "confirmation_required",
    "run" => options[:run],
    "packet_id" => packet_id,
    "instruction" => "In the new VS Code Ask-mode chat, verify Copilot visibly acknowledged this exact packet ID. Then rerun with --observed-packet-id set to that exact value."
  })
  exit
end

abort("temporary root must be empty before preparation") unless Dir.children(root).empty?
[workspace, cache, runtime, artifacts].each { |path| FileUtils.mkdir_p(path) }
File.binwrite(fixture, FIXTURE_BYTES)
source_before = Digest::SHA256.file(fixture).hexdigest
File.binwrite(source_hash_path, "#{source_before}\n")
common = [options[:cli], "--at", FIXED_TIME, "--cutoff", FIXED_CUTOFF]
snapshot = run_json(*common, "--id-seed", "vscodeci3fsnapshot#{options[:run]}", "snapshot", "build", workspace, cache)
intent = {
  "adapter_contract_version" => "1.0.0",
  "client" => "vscode_copilot",
  "scope" => "chat_cli_ask",
  "client_version" => CLIENT_VERSION,
  "lifecycle_point" => "chat_open",
  "consent" => true,
  "request_id" => "req_vscodeci3fdelivery#{options[:run]}",
  "event_id" => "evt_vscodeci3fdelivery#{options[:run]}",
  "consumer_id" => "consumer_vscode_ci3f_admission",
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
File.binwrite(intent_path, JSON.generate(intent))
preview = run_json(*common, "--id-seed", "vscodeci3fpreview#{options[:run]}", "client", "delivery", "vscode", "preview", workspace, cache, intent_path)
abort("delivery was not prepared") unless preview["state"] == "prepared"
File.binwrite(preview_path, JSON.generate(preview))
packet_id = preview.dig("value", "delivery_envelope", "packet_id")
puts JSON.pretty_generate({
  "status" => "prepared",
  "run" => options[:run],
  "packet_id" => packet_id,
  "source_sha256" => source_before,
  "next_step" => "Review the preview, then rerun this command with --apply. That action opens a new VS Code Ask-mode chat and sends the bounded synthetic packet to Copilot."
})
