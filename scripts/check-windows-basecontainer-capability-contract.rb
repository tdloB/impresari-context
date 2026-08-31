#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/iar-windows-basecontainer-capability-v1.json"
PROFILE_DIGEST = "9f5c8f589cf5f7ce3e6d87b6b7752aeac4da530a81edbb1bf036bf5eb7e84305"
PROBE_RELATIVE = "platform/windows-native-feasibility/windows-basecontainer-capability-probe.rs"
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")

options = {receipt: nil}
OptionParser.new do |parser|
  parser.banner = "Usage: check-windows-basecontainer-capability-contract.rb [--receipt FILE]"
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
abort "missing Windows BaseContainer capability profile" unless profile_path.file? && !profile_path.symlink?
abort "Windows BaseContainer capability profile digest changed" unless
  Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/iar-windows-basecontainer-capability-v1.sha256").read.strip
abort "Windows BaseContainer capability checksum record mismatch" unless
  sidecar == "#{PROFILE_DIGEST}  iar-windows-basecontainer-capability-v1.json"
fixture_profile = FIXTURE_ROOT.join("valid/iar-windows-basecontainer-capability-profile.json")
abort "Windows BaseContainer profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile = read_json(profile_path)
abort "Windows BaseContainer target identity changed" unless
  profile.fetch("profile_id") == "iar-windows-basecontainer-capability-v1" &&
    profile.dig("target", "os_family") == "windows" &&
    profile.dig("target", "os_product_type") == "workstation" &&
    profile.dig("target", "runner_environment") == "github-hosted" &&
    profile.dig("target", "runner_label") == "windows-11-arm" &&
    profile.dig("target", "architecture") == "arm64" &&
    profile.dig("target", "filesystem") == "ntfs" &&
    profile.dig("target", "minimum_windows_build") == "26600"
abort "Windows BaseContainer export contract changed" unless profile.fetch("basecontainer") == {
  "module" => "processmodel.dll",
  "sandbox_specification_version" => "0.1.0",
  "required_exports" => [
    "Experimental_CreateProcessAsUserInSandbox",
    "Experimental_CreateProcessInSandbox"
  ]
}
abort "Windows BaseContainer observation crossed an authority boundary" unless
  profile.dig("observation", "system_module_export_inspection") &&
    profile.fetch("observation").reject { |key, _| key == "system_module_export_inspection" }.values.none? &&
    profile.fetch("claims").values.none?

probe = ROOT.join(PROBE_RELATIVE).read
abort "BaseContainer probe profile identity drifted" unless
  probe.include?(%[const PROFILE_ID: &str = "#{profile.fetch('profile_id')}";]) &&
    probe.include?(%["sha256:#{PROFILE_DIGEST}";]) &&
    probe.include?("LOAD_LIBRARY_SEARCH_SYSTEM32") &&
    probe.include?("Experimental_CreateProcessInSandbox\\0") &&
    probe.include?("Experimental_CreateProcessAsUserInSandbox\\0")
forbidden_probe_patterns = [
  /std::process::Command/,
  /Command::new/,
  /CreateProcessW\s*\(/,
  /fn\s+Experimental_CreateProcess(?:AsUser)?InSandbox/,
  /CreateAppContainerProfile\s*\(/,
  /SetNamedSecurityInfo/,
  /SetSecurityInfo/,
  /SetEntriesInAcl/,
  /ShellExecute/,
  /OpenSCManager/,
  /CreateService/,
  /Enable-WindowsOptionalFeature/,
  /Containers-DisposableClientVM/,
  /runas/i
]
abort "BaseContainer probe gained launch, mutation, elevation, or service syntax" if
  forbidden_probe_patterns.any? { |pattern| probe.match?(pattern) }

provenance_path = FIXTURE_ROOT.join("windows-basecontainer-capability-fixture-provenance.json")
provenance = read_json(provenance_path)
expected_fixture_paths = %w[
  invalid/windows-basecontainer-capability-overclaim.json
  valid/iar-windows-basecontainer-capability-profile.json
  valid/windows-basecontainer-capability-unsupported-build.json
]
entries = provenance.fetch("fixtures")
abort "Windows BaseContainer fixture provenance is not closed or sorted" unless
  entries.map { |entry| entry.fetch("path") } == expected_fixture_paths
entries.each do |entry|
  path = FIXTURE_ROOT.join(entry.fetch("path")).cleanpath
  abort "Windows BaseContainer fixture escapes root" unless
    path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked Windows BaseContainer fixture" unless path.file? && !path.symlink?
  abort "Windows BaseContainer fixture provenance digest changed" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
end
abort "Windows BaseContainer fixture provenance added authority" unless
  %w[third_party_content executable_content repository_source_content credential_content network_capture_content authority_added]
    .none? { |key| provenance.fetch(key) }

receipt_path = options[:receipt] ||
  FIXTURE_ROOT.join("valid/windows-basecontainer-capability-unsupported-build.json")
abort "missing or symlinked Windows BaseContainer receipt" unless receipt_path.file? && !receipt_path.symlink?
receipt = read_json(receipt_path)
expected_keys = %w[
  schema_name schema_version profile_id profile_digest runner_environment runner_label
  os_family os_product_type windows_build architecture filesystem system_module_inspected
  processmodel_dll_present create_process_in_sandbox_export_present
  create_process_as_user_in_sandbox_export_present status reason_code
  synthetic_worker_launched appcontainer_profile_created host_acl_modified
  windows_feature_modified elevation_requested os_confined production_admitted
  analyzer_execution authority_added
].sort
abort "Windows BaseContainer receipt shape changed" unless receipt.keys.sort == expected_keys
abort "Windows BaseContainer receipt identity changed" unless
  receipt.fetch("schema_name") == "windows-basecontainer-capability-receipt" &&
    receipt.fetch("schema_version") == "1.0.0" &&
    receipt.fetch("profile_id") == profile.fetch("profile_id") &&
    receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    receipt.fetch("runner_environment") == "github-hosted" &&
    receipt.fetch("runner_label") == "windows-11-arm" &&
    receipt.fetch("os_family") == "windows" &&
    %w[workstation domain_controller server].include?(receipt.fetch("os_product_type")) &&
    receipt.fetch("windows_build").match?(/\A[0-9]{1,10}\z/) &&
    receipt.fetch("architecture") == "arm64" &&
    %w[ntfs other].include?(receipt.fetch("filesystem")) &&
    receipt.fetch("system_module_inspected")
abort "Windows BaseContainer receipt crossed a no-worker authority gate" unless
  %w[
    synthetic_worker_launched appcontainer_profile_created host_acl_modified
    windows_feature_modified elevation_requested os_confined production_admitted
    analyzer_execution authority_added
  ].none? { |key| receipt.fetch(key) }
abort "Windows BaseContainer export appeared without its module" if
  !receipt.fetch("processmodel_dll_present") &&
    (receipt.fetch("create_process_in_sandbox_export_present") ||
      receipt.fetch("create_process_as_user_in_sandbox_export_present"))

expected = if receipt.fetch("os_product_type") != "workstation"
  ["unsupported", "unsupported_host_family"]
elsif receipt.fetch("filesystem") != "ntfs"
  ["unsupported", "unsupported_filesystem"]
elsif receipt.fetch("windows_build").to_i < 26_600
  ["unsupported", "unsupported_build"]
elsif !receipt.fetch("processmodel_dll_present") ||
    !receipt.fetch("create_process_in_sandbox_export_present") ||
    !receipt.fetch("create_process_as_user_in_sandbox_export_present")
  ["unsupported", "unsupported_api_absent"]
else
  ["ready_for_basecontainer_rehearsal", "candidate_capability_present"]
end
abort "Windows BaseContainer routing decision is inconsistent with observed facts" unless
  [receipt.fetch("status"), receipt.fetch("reason_code")] == expected

puts "Windows BaseContainer capability verified: build=#{receipt.fetch('windows_build')} " \
  "arch=arm64 status=#{receipt.fetch('status')} reason=#{receipt.fetch('reason_code')} " \
  "worker_launched=false os_confined=false"
