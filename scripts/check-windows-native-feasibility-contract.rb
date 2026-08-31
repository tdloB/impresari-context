#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/iar-windows-native-feasibility-v1.json"
PROFILE_DIGEST = "6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73"
PROBE_RELATIVE = "platform/windows-native-feasibility/windows-native-capability-probe.rs"
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")

options = {receipt: nil}
OptionParser.new do |parser|
  parser.banner = "Usage: check-windows-native-feasibility-contract.rb [--receipt FILE]"
  parser.on("--receipt FILE", "Validate a live hosted Windows receipt") do |value|
    options[:receipt] = Pathname.new(value).expand_path
  end
end.parse!
abort "unexpected arguments" unless ARGV.empty?

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

profile_path = ROOT.join(PROFILE_RELATIVE)
abort "missing Windows native profile" unless profile_path.file? && !profile_path.symlink?
abort "Windows native profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/iar-windows-native-feasibility-v1.sha256").read.strip
abort "Windows native profile checksum record mismatch" unless
  sidecar == "#{PROFILE_DIGEST}  iar-windows-native-feasibility-v1.json"
fixture_profile = FIXTURE_ROOT.join("valid/iar-windows-native-feasibility-profile.json")
abort "Windows native profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile = read_json(profile_path)
abort "Windows native target identity changed" unless
  profile.fetch("profile_id") == "iar-windows-native-feasibility-v1" &&
    profile.dig("target", "os_family") == "windows" &&
    profile.dig("target", "runner_label") == "windows-2025" &&
    profile.dig("target", "architecture") == "x86_64" &&
    profile.dig("target", "filesystem") == "ntfs"
abort "Windows native zero-capability identity changed" unless
  profile.dig("identity", "fresh_identity_per_job") &&
    profile.dig("identity", "capability_count") == "0" &&
    profile.dig("identity", "lpac_required_for_worker") &&
    profile.dig("identity", "all_application_packages_opt_out")
abort "Windows native resource ceilings changed" unless profile.fetch("limits") == {
  "active_processes" => "1",
  "process_memory_bytes" => "67108864",
  "job_memory_bytes" => "134217728",
  "cpu_rate_percent" => "25",
  "wall_time_ms" => "30000",
  "stdout_bytes" => "65536",
  "stderr_bytes" => "16384",
  "staged_artifacts" => "64",
  "staged_total_bytes" => "4194304",
  "writable_path_backed_bytes" => "0"
}
abort "Windows native profile crossed a preflight gate" unless profile.fetch("preflight").values.none?

probe = ROOT.join(PROBE_RELATIVE).read
abort "native probe profile identity drifted" unless
  probe.include?(%[const PROFILE_ID: &str = "#{profile.fetch('profile_id')}";]) &&
    probe.include?(%["sha256:#{PROFILE_DIGEST}";])
abort "native probe no longer uses zero capabilities" unless
  probe.include?("CreateAppContainerProfile(") &&
    probe.include?("null(),\n            0,\n            &raw mut created")
abort "native probe gained child-process launch syntax" if
  probe.match?(/std::process::Command|Command::new|CreateProcessW\s*\(/)

provenance_path = FIXTURE_ROOT.join("windows-native-fixture-provenance.json")
provenance = read_json(provenance_path)
expected_fixture_paths = %w[
  invalid/windows-native-capability-preflight-overclaim.json
  valid/iar-windows-native-feasibility-profile.json
  valid/windows-native-capability-preflight.json
]
entries = provenance.fetch("fixtures")
abort "Windows native fixture provenance is not closed or sorted" unless
  entries.map { |entry| entry.fetch("path") } == expected_fixture_paths
entries.each do |entry|
  path = FIXTURE_ROOT.join(entry.fetch("path")).cleanpath
  abort "Windows native fixture escapes root" unless path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked Windows native fixture" unless path.file? && !path.symlink?
  abort "Windows native fixture provenance digest changed" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
end
abort "Windows native fixture provenance added authority" unless
  %w[third_party_content executable_content repository_source_content credential_content network_capture_content authority_added]
    .none? { |key| provenance.fetch(key) }

receipt_path = options[:receipt] || FIXTURE_ROOT.join("valid/windows-native-capability-preflight.json")
abort "missing or symlinked Windows native receipt" unless receipt_path.file? && !receipt_path.symlink?
receipt = read_json(receipt_path)
expected_keys = %w[
  schema_name schema_version profile_id profile_digest runner_environment runner_label
  os_family windows_build architecture filesystem required_launch_apis_present
  required_mitigation_apis_present job_object_created job_limits_set job_limits_queried
  job_kill_on_close_configurable active_process_limit_configurable breakaway_disabled
  appcontainer_profile_created appcontainer_sid_derived appcontainer_sid_matched
  appcontainer_profile_deleted capability_count synthetic_worker_launched
  appcontainer_worker_launched network_denial_verified path_boundary_verified
  resource_limits_verified descendant_containment_verified complete_cleanup_verified
  os_confined production_admitted analyzer_execution authority_added
].sort
abort "Windows native receipt shape changed" unless receipt.keys.sort == expected_keys
abort "Windows native receipt identity changed" unless
  receipt.fetch("schema_name") == "windows-native-capability-preflight" &&
    receipt.fetch("schema_version") == "1.0.0" &&
    receipt.fetch("profile_id") == profile.fetch("profile_id") &&
    receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    receipt.fetch("runner_environment") == "github-hosted" &&
    receipt.fetch("runner_label") == "windows-2025" &&
    receipt.fetch("os_family") == "windows" &&
    receipt.fetch("architecture") == "x86_64" &&
    receipt.fetch("filesystem") == "ntfs" &&
    receipt.fetch("windows_build").match?(/\A[0-9]{1,10}\z/) &&
    receipt.fetch("capability_count") == "0"
measured_true = %w[
  required_launch_apis_present required_mitigation_apis_present job_object_created
  job_limits_set job_limits_queried job_kill_on_close_configurable
  active_process_limit_configurable breakaway_disabled appcontainer_profile_created
  appcontainer_sid_derived appcontainer_sid_matched appcontainer_profile_deleted
]
abort "Windows native measured preflight is incomplete" unless measured_true.all? { |key| receipt.fetch(key) }
unmeasured_false = %w[
  synthetic_worker_launched appcontainer_worker_launched network_denial_verified
  path_boundary_verified resource_limits_verified descendant_containment_verified
  complete_cleanup_verified os_confined production_admitted analyzer_execution authority_added
]
abort "Windows native receipt crossed an unmeasured gate" unless unmeasured_false.none? { |key| receipt.fetch(key) }

puts "Windows native contract verified: build=#{receipt.fetch('windows_build')} profile=sha256:#{PROFILE_DIGEST} worker_launched=false os_confined=false"
