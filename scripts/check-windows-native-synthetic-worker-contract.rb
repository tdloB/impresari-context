#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-windows-native-synthetic-worker-matrix-v1.json"
PROFILE_DIGEST = "82ab5c5c0cff76079ae19925b92da23b2d86e3a31e7cfc58626e17cb01c14678"
BASE_PROFILE_RELATIVE = "profiles/v1/iar-windows-native-feasibility-v1.json"
BASE_PROFILE_DIGEST = "6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73"
ZERO_DIGEST = "sha256:" + ("0" * 64)
BROKER_RELATIVE = "platform/windows-native-feasibility/windows-native-synthetic-broker.rs"
WORKER_RELATIVE = "platform/windows-native-feasibility/windows-native-synthetic-worker.rs"

options = {receipt: nil}
OptionParser.new do |parser|
  parser.banner = "Usage: check-windows-native-synthetic-worker-contract.rb [--receipt FILE]"
  parser.on("--receipt FILE", "Validate a live hosted Windows worker-matrix receipt") do |value|
    options[:receipt] = Pathname.new(value).expand_path
  end
end.parse!
abort "unexpected arguments" unless ARGV.empty?

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def boolean_values(value, skip = [])
  value.each_with_object({}) do |(key, member), result|
    next if skip.include?(key)
    abort "non-boolean contract member: #{key}" unless member == true || member == false
    result[key] = member
  end
end

base_profile_path = ROOT.join(BASE_PROFILE_RELATIVE)
abort "Windows base profile digest changed" unless
  Digest::SHA256.file(base_profile_path).hexdigest == BASE_PROFILE_DIGEST

profile_path = ROOT.join(PROFILE_RELATIVE)
abort "missing or symlinked Windows worker profile" unless profile_path.file? && !profile_path.symlink?
abort "Windows worker profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/iar-windows-native-synthetic-worker-matrix-v1.sha256").read.strip
abort "Windows worker profile checksum record mismatch" unless
  sidecar == "#{PROFILE_DIGEST}  iar-windows-native-synthetic-worker-matrix-v1.json"
fixture_profile = FIXTURE_ROOT.join("valid/iar-windows-native-synthetic-worker-matrix-profile.json")
abort "Windows worker profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile = read_json(profile_path)
abort "Windows worker profile identity changed" unless
  profile.fetch("profile_id") == "iar-windows-native-synthetic-worker-matrix-v1" &&
    profile.fetch("base_profile_id") == "iar-windows-native-feasibility-v1" &&
    profile.fetch("base_profile_digest") == "sha256:#{BASE_PROFILE_DIGEST}" &&
    profile.dig("target", "runner_label") == "windows-2025" &&
    profile.dig("target", "architecture") == "x86_64" &&
    profile.dig("target", "filesystem") == "ntfs"
abort "Windows worker launch order or authority changed" unless
  profile.dig("launch", "create_suspended") &&
    profile.dig("launch", "assign_job_before_resume") &&
    profile.dig("launch", "lpac_required") &&
    profile.dig("launch", "capability_count") == "0" &&
    profile.dig("launch", "all_application_packages_opt_out") &&
    profile.dig("launch", "exact_inherited_handles") == "3" &&
    profile.dig("launch", "child_process_policy") == "restricted" &&
    !profile.dig("launch", "caller_environment_inherited") &&
    !profile.dig("launch", "arbitrary_arguments")
expected_scenarios = %w[
  success input-mutation worker-mutation sibling-read user-profile-canary-read
  profile-storage-write synthetic-registry-canary-read loopback-connect
  unrelated-handle unrelated-process child-process cpu-pressure memory-pressure
  timeout output-flood crash cancellation malformed-result cross-job-read
]
abort "Windows worker scenario corpus changed" unless profile.fetch("scenarios") == expected_scenarios
abort "Windows worker controls crossed an authority gate" unless
  profile.dig("controls", "synthetic_worker_only") &&
    profile.dig("controls", "worker_identity_pinned") &&
    profile.dig("controls", "profile_storage_write_denied") &&
    %w[external_network_destination existing_credentials_inspected repository_input real_analyzer os_confined production_admitted authority_added]
      .none? { |key| profile.dig("controls", key) }

broker_path = ROOT.join(BROKER_RELATIVE)
worker_path = ROOT.join(WORKER_RELATIVE)
abort "missing or symlinked Windows synthetic broker" unless broker_path.file? && !broker_path.symlink?
abort "missing or symlinked Windows synthetic worker" unless worker_path.file? && !worker_path.symlink?
broker = broker_path.read
worker = worker_path.read
abort "Windows broker launch sequence drifted" unless
  broker.include?("PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES") &&
    broker.include?("PROC_THREAD_ATTRIBUTE_ALL_APPLICATION_PACKAGES_POLICY") &&
    broker.include?("PROC_THREAD_ATTRIBUTE_HANDLE_LIST") &&
    broker.include?("PROC_THREAD_ATTRIBUTE_CHILD_PROCESS_POLICY") &&
    broker.include?("PROC_THREAD_ATTRIBUTE_MITIGATION_POLICY") &&
    broker.include?("size_of::<[u64; 2]>()") &&
    broker.include?("CREATE_NO_WINDOW") &&
    broker.include?("CREATE_SUSPENDED") &&
    broker.include?("CREATE_UNICODE_ENVIRONMENT") &&
    broker.include?("EXTENDED_STARTUPINFO_PRESENT") &&
    broker.include?("AssignProcessToJobObject(job.0, process_handle.0)") &&
    broker.include?("ResumeThread(thread_handle.0)") &&
    broker.index("AssignProcessToJobObject(job.0, process_handle.0)") < broker.index("ResumeThread(thread_handle.0)")
abort "Windows broker boundary corpus drifted" unless
  expected_scenarios.all? { |scenario| broker.include?(%Q{"#{scenario}"}) } &&
    broker.include?("GetAppContainerFolderPath") &&
    broker.include?("path == profile.path") &&
    broker.include?("profile.sid_string.as_str()") &&
    broker.include?("let stage = first.path.join") &&
    broker.include?("let second_worker = second_stage.join") &&
    broker.include?("EnvironmentBlock::exact_system(stage)") &&
    broker.include?("CreateEnvironmentBlock(&raw mut clean, token.0, 0)") &&
    broker.include?(%q{"LOCALAPPDATA"}) &&
    broker.include?(%q{"USERPROFILE"}) &&
    broker.include?("GetWindowsDirectoryW") &&
    broker.include?(%q{entries.insert("SystemDrive".into(), system_drive)}) &&
    broker.include?(%q{entries.insert("SystemRoot".into(), system_root.clone())}) &&
    broker.include?(%q{entries.insert("windir".into(), system_root)}) &&
    broker.include?("PROTECTED_DACL_SECURITY_INFORMATION") &&
    broker.include?("EXPECTED_WORKER_SHA256") &&
    broker.include?("EXPECTED_BROKER_SHA256") &&
    broker.include?("TerminateJobObject") &&
    broker.include?("DeleteAppContainerProfile")
abort "Windows worker protocol drifted" unless
  expected_scenarios.all? { |scenario| worker.include?(%Q{"#{scenario}"}) } &&
    worker.include?("TOKEN_IS_LESS_PRIVILEGED_APP_CONTAINER") &&
    worker.include?("TOKEN_APP_CONTAINER_SID") &&
    worker.include?("control frame exceeded 16384 bytes")
abort "Windows synthetic sources added shell launch authority" if
  [broker, worker].any? { |source| source.match?(/std::process::Command|Command::new/) }

ci = ROOT.join(".github/workflows/ci.yml").read
abort "Windows hosted live matrix job drifted" unless
  ci.include?("name: Windows native IAR-1B synthetic candidate") &&
    ci.include?("windows-native-synthetic-broker.rs") &&
    ci.include?("windows-native-synthetic-worker.rs") &&
    ci.include?("EXPECTED_WORKER_SHA256") &&
    ci.include?("EXPECTED_BROKER_SHA256") &&
    ci.include?("check-windows-native-synthetic-worker-contract.rb --receipt")

provenance = read_json(FIXTURE_ROOT.join("windows-native-synthetic-worker-fixture-provenance.json"))
expected_fixture_paths = %w[
  invalid/windows-native-synthetic-worker-matrix-overclaim.json
  valid/iar-windows-native-synthetic-worker-matrix-profile.json
  valid/windows-native-synthetic-worker-matrix-contract.json
]
entries = provenance.fetch("fixtures")
abort "Windows worker fixture provenance is not closed or sorted" unless
  entries.map { |entry| entry.fetch("path") } == expected_fixture_paths
entries.each do |entry|
  path = FIXTURE_ROOT.join(entry.fetch("path")).cleanpath
  abort "Windows worker fixture escapes root" unless path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked Windows worker fixture" unless path.file? && !path.symlink?
  abort "Windows worker fixture provenance digest changed" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
end
abort "Windows worker fixture provenance added authority" unless
  %w[third_party_content executable_content repository_source_content credential_content network_capture_content authority_added]
    .none? { |key| provenance.fetch(key) }

receipt_path = options[:receipt] || FIXTURE_ROOT.join("valid/windows-native-synthetic-worker-matrix-contract.json")
abort "missing or symlinked Windows worker receipt" unless receipt_path.file? && !receipt_path.symlink?
receipt = read_json(receipt_path)
abort "Windows worker receipt identity changed" unless
  receipt.fetch("schema_name") == "windows-native-synthetic-worker-matrix-receipt" &&
    receipt.fetch("schema_version") == "1.0.0" &&
    receipt.fetch("profile_id") == profile.fetch("profile_id") &&
    receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    receipt.fetch("base_profile_id") == profile.fetch("base_profile_id") &&
    receipt.fetch("base_profile_digest") == profile.fetch("base_profile_digest") &&
    receipt.dig("host", "runner_environment") == "github-hosted" &&
    receipt.dig("host", "runner_label") == "windows-2025" &&
    receipt.dig("host", "os_family") == "windows" &&
    receipt.dig("host", "architecture") == "x86_64" &&
    receipt.dig("host", "filesystem") == "ntfs" &&
    receipt.dig("identity", "capability_count") == "0" &&
    receipt.dig("observations", "scenario_count") == "19"
abort "Windows worker receipt added authority" unless receipt.fetch("claims").values.none?

identity = boolean_values(receipt.fetch("identity"), ["capability_count"])
launch = boolean_values(receipt.fetch("launch"))
observations = boolean_values(receipt.fetch("observations"), ["scenario_count"])
cleanup = boolean_values(receipt.fetch("cleanup"))
measured = identity.merge(launch).merge(observations).merge(cleanup)

if options[:receipt]
  abort "Windows worker live matrix did not pass" unless
    receipt.fetch("status") == "candidate_passed" &&
      receipt.fetch("reason_code") == "synthetic_matrix_passed" &&
      receipt.dig("host", "windows_build").match?(/\A[1-9][0-9]{0,9}\z/) &&
      receipt.dig("host", "broker_digest") != ZERO_DIGEST &&
      receipt.dig("host", "worker_digest") != ZERO_DIGEST &&
      measured.values.all?
else
  abort "Windows worker contract fixture overclaimed execution" unless
    receipt.fetch("status") == "contract_fixture" &&
      receipt.fetch("reason_code") == "contract_fixture" &&
      receipt.dig("host", "windows_build") == "0" &&
      receipt.dig("host", "broker_digest") == ZERO_DIGEST &&
      receipt.dig("host", "worker_digest") == ZERO_DIGEST &&
      measured.values.none?
end

puts "Windows synthetic-worker contract verified: profile=sha256:#{PROFILE_DIGEST} status=#{receipt.fetch('status')} os_confined=false"
