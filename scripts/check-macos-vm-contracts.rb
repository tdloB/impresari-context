#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path

def read(relative)
  ROOT.join(relative).read
end

def json(relative)
  JSON.parse(read(relative))
end

profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-feasibility-v1.json")
profile_digest = Digest::SHA256.file(profile_path).hexdigest
abort "macOS VM profile digest changed" unless profile_digest == "a082df092d5180058f732d47ae99164316f3bfd3b12f4079de43575834314757"
sha_record = read("profiles/v1/iar-macos-local-vm-feasibility-v1.sha256").strip
abort "macOS VM profile checksum record mismatch" unless sha_record == "#{profile_digest}  iar-macos-local-vm-feasibility-v1.json"
abort "macOS VM profile fixture differs from the frozen profile" unless
  profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-feasibility-profile.json").binread

profile = json("profiles/v1/iar-macos-local-vm-feasibility-v1.json")
abort "macOS VM profile admits analyzer execution" unless profile.dig("controls", "analyzer_execution") == false
abort "macOS VM profile admits production" unless profile.dig("controls", "production_admitted") == false
abort "macOS VM profile adds authority" unless profile.dig("controls", "authority_added") == false
abort "macOS VM profile includes network or host sharing" unless
  profile.dig("devices", "network_devices") == "0" && profile.dig("devices", "directory_shares") == "0"

matrix_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-synthetic-matrix-v2.json")
matrix_profile_digest = Digest::SHA256.file(matrix_profile_path).hexdigest
abort "macOS VM matrix profile digest changed" unless matrix_profile_digest == "090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888"
matrix_sha_record = read("profiles/v1/iar-macos-local-vm-synthetic-matrix-v2.sha256").strip
abort "macOS VM matrix profile checksum record mismatch" unless
  matrix_sha_record == "#{matrix_profile_digest}  iar-macos-local-vm-synthetic-matrix-v2.json"
abort "macOS VM matrix profile fixture differs from the frozen profile" unless
  matrix_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-synthetic-matrix-profile-v2.json").binread

matrix_profile = json("profiles/v1/iar-macos-local-vm-synthetic-matrix-v2.json")
abort "macOS VM matrix guest identity is not exact" unless
  matrix_profile.dig("guest", "guest_init_sha256") == "68d5be977b2bd1bc7df2bcfc8bdb077bb03f9afc390d7c099f23437ced1598bf" &&
    matrix_profile.dig("guest", "initramfs_sha256") == "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b"
abort "macOS VM matrix expands authority" unless
  matrix_profile.dig("controls", "analyzer_execution") == false &&
    matrix_profile.dig("controls", "production_admitted") == false &&
    matrix_profile.dig("controls", "authority_added") == false

supervisor_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-supervisor-v2.json")
supervisor_profile_digest = Digest::SHA256.file(supervisor_profile_path).hexdigest
abort "macOS VM supervisor profile digest changed" unless
  supervisor_profile_digest == "614b9da42f051518e6a6d54f15e75c492e233e2ed653bfcbf69285d130967b88"
supervisor_sha_record = read("profiles/v1/iar-macos-local-vm-supervisor-v2.sha256").strip
abort "macOS VM supervisor profile checksum record mismatch" unless
  supervisor_sha_record == "#{supervisor_profile_digest}  iar-macos-local-vm-supervisor-v2.json"
abort "macOS VM supervisor profile fixture differs from the frozen profile" unless
  supervisor_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-supervisor-profile-v2.json").binread
supervisor_profile = json("profiles/v1/iar-macos-local-vm-supervisor-v2.json")
abort "macOS VM supervisor profile expands authority" unless
  supervisor_profile.dig("controls", "analyzer_execution") == false &&
    supervisor_profile.dig("controls", "production_admitted") == false &&
    supervisor_profile.dig("controls", "authority_added") == false

resource_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-resource-canary-v2.json")
resource_profile_digest = Digest::SHA256.file(resource_profile_path).hexdigest
abort "macOS VM resource/canary profile digest changed" unless
  resource_profile_digest == "82d3cbf4b68866b92794a06e86948ccaf2492b3b4cb38e7e70503562c61d1de0"
resource_sha_record = read("profiles/v1/iar-macos-local-vm-resource-canary-v2.sha256").strip
abort "macOS VM resource/canary profile checksum record mismatch" unless
  resource_sha_record == "#{resource_profile_digest}  iar-macos-local-vm-resource-canary-v2.json"
abort "macOS VM resource/canary profile fixture differs from the frozen profile" unless
  resource_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-resource-canary-profile-v2.json").binread
resource_profile = json("profiles/v1/iar-macos-local-vm-resource-canary-v2.json")
abort "macOS VM resource/canary profile expands authority" unless
  resource_profile.dig("controls", "analyzer_execution") == false &&
    resource_profile.dig("controls", "production_admitted") == false &&
    resource_profile.dig("controls", "authority_added") == false
abort "macOS VM resource/canary profile lost exact cgroup limits" unless
  resource_profile.dig("limits", "job_memory_bytes") == "33554432" &&
    resource_profile.dig("limits", "job_cpu_quota_usec") == "10000" &&
    resource_profile.dig("limits", "job_cpu_period_usec") == "100000" &&
    resource_profile.dig("limits", "job_pids") == "8"

interruption_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-interruption-v2.json")
interruption_profile_digest = Digest::SHA256.file(interruption_profile_path).hexdigest
abort "macOS VM interruption profile digest changed" unless
  interruption_profile_digest == "f1b57b17d9de3b2b4de885732b6bef0f3cbf637bcba08dc1dda34724e9b18c4f"
interruption_sha_record = read("profiles/v1/iar-macos-local-vm-interruption-v2.sha256").strip
abort "macOS VM interruption profile checksum record mismatch" unless
  interruption_sha_record == "#{interruption_profile_digest}  iar-macos-local-vm-interruption-v2.json"
abort "macOS VM interruption profile fixture differs from the frozen profile" unless
  interruption_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-interruption-profile-v2.json").binread
interruption_profile = json("profiles/v1/iar-macos-local-vm-interruption-v2.json")
abort "macOS VM interruption profile confuses simulation with real sleep" unless
  interruption_profile.fetch("automated_evidence") == "synthetic-job-private-trigger" &&
    interruption_profile.fetch("manual_evidence_required") == "os-will-sleep" &&
    interruption_profile.dig("controls", "real_host_sleep_observed") == false
abort "macOS VM interruption profile expands authority" unless
  interruption_profile.dig("controls", "analyzer_execution") == false &&
    interruption_profile.dig("controls", "production_admitted") == false &&
    interruption_profile.dig("controls", "authority_added") == false

assets = json("platform/macos-vm-feasibility/guest-assets-v2.json")
abort "macOS VM guest asset source is not exact Alpine HTTPS" unless assets.fetch("artifacts").all? do |artifact|
  artifact.fetch("url").start_with?("https://dl-cdn.alpinelinux.org/alpine/v3.24/main/aarch64/") &&
    artifact.fetch("sha256").match?(/\A[0-9a-f]{64}\z/) &&
    artifact.fetch("bytes").match?(/\A[1-9][0-9]*\z/)
end
abort "macOS VM guest asset publisher authentication is incomplete" unless
  assets.dig("publisher_authentication", "key_sha256") == "d11f6b21c61b4274e182eb888883a8ba8acdbf820dcc7a6d82a7d9fc2fd2836d" &&
    assets.dig("publisher_authentication", "package_datahash") == "e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263"
abort "macOS VM guest asset record enables guest networking" unless assets.fetch("network_available_to_guest") == false
abort "macOS VM guest asset record overclaims production" unless assets.fetch("production_admitted") == false
abort "macOS VM guest asset record adds authority" unless assets.fetch("authority_added") == false
abort "macOS VM guest asset record is already expired" unless Time.iso8601(assets.fetch("expires_at")) > Time.now.utc

entitlements = read("platform/macos-vm-feasibility/Resources/Controller.entitlements")
keys = entitlements.scan(%r{<key>([^<]+)</key>}).flatten
abort "macOS VM controller entitlement set changed" unless keys == ["com.apple.security.virtualization"]

controller = read("platform/macos-vm-feasibility/Sources/Controller/main.swift")
%w[
  configuration.networkDevices\ =\ []
  configuration.directorySharingDevices\ =\ []
  configuration.graphicsDevices\ =\ []
  configuration.audioDevices\ =\ []
  configuration.keyboards\ =\ []
  configuration.pointingDevices\ =\ []
].each do |required|
  abort "macOS VM controller lost an absent-device assertion: #{required}" unless controller.include?(required.gsub("\\ ", " "))
end
%w[VZVirtioNetworkDeviceConfiguration VZSharedDirectory URLSession NWConnection Process(].each do |forbidden|
  abort "macOS VM controller acquired forbidden surface: #{forbidden}" if controller.include?(forbidden)
end
abort "macOS VM controller no longer limits serial output" unless controller.include?("maximumSerialBytes = 65_536")
abort "macOS VM controller lost bounded in-memory serial capture" unless
  controller.include?("BoundedSerialCapture") && controller.include?("serialOverflow") && !controller.include?("serial.log")
abort "macOS VM controller accepts caller-selected guest identity" unless
  controller.include?("initramfsDigest = \"89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b\"")
abort "macOS VM controller lost the exact external cancellation marker" unless
  controller.include?("cancel.request") && controller.include?("IMPRESARI_VM_CANCEL_V1") == false
abort "macOS VM controller can overclaim VM confinement" unless controller.include?("vmConfined: false")

runner = read("crates/context-analyzer-runner/src/lib.rs")
abort "Rust supervisor profile identity drifted" unless
  runner.include?("iar-macos-local-vm-supervisor-v2") &&
    runner.include?("sha256:614b9da42f051518e6a6d54f15e75c492e233e2ed653bfcbf69285d130967b88")
abort "Rust supervisor acquired a second child-process launch site" unless runner.scan("Command::new").length == 1
abort "Rust supervisor lost forced termination or exact cleanup" unless
  runner.include?("ForcedTerminationRecovery") && runner.include?("force_kill_and_collect") &&
    runner.include?("remove_exact_job(&action_job)")
abort "Rust supervisor lost resource/canary validation" unless
  runner.include?("execute_resource_canary") && runner.include?("validate_vm_resource_canary") &&
    runner.include?("MACOS_VM_RESOURCE_INITRAMFS_DIGEST")

guest = read("platform/macos-vm-feasibility/Sources/GuestInit/main.c")
%w[execve( execl( system( popen( socket( connect(].each do |forbidden|
  abort "synthetic guest init acquired forbidden execution/network surface: #{forbidden}" if guest.include?(forbidden)
end
abort "synthetic guest descendant probe changed" unless guest.scan("fork(").length == 1 && guest.include?("SCENARIO_DESCENDANT_TIMEOUT")
abort "synthetic guest init lost raw scratch ceiling" unless guest.include?("#define SCRATCH_BYTES 1048576")
abort "synthetic guest init lost read-only input probe" unless guest.include?("input_write_denied")
abort "synthetic guest init lost network-device absence probe" unless guest.include?("network_device_absent")

resource_guest = read("platform/macos-vm-feasibility/Sources/GuestResourceInit/main.c")
%w[execve( execl( system( popen( socket( connect(].each do |forbidden|
  abort "resource guest acquired forbidden execution/network surface: #{forbidden}" if resource_guest.include?(forbidden)
end
abort "resource guest lost exact cgroup controls" unless
  resource_guest.include?("memory.max") && resource_guest.include?("cpu.max") &&
    resource_guest.include?("pids.max") && resource_guest.include?("cgroup.kill")
abort "resource guest lost host canary/path/process probes" unless
  resource_guest.include?("canary_markers") && resource_guest.include?("host_paths_absent") &&
    resource_guest.include?("host_process_invisible") && resource_guest.include?("exact_block_devices")

initramfs_builder = read("scripts/build-macos-vm-initramfs.rb")
abort "macOS VM initramfs build timestamp is not reproducible" unless initramfs_builder.include?("gzip.mtime = 1")

v2_provenance = json("tests/conformance/v1/macos-local-vm-current-guest-fixture-provenance.json")
abort "current guest fixture review status changed" unless
  v2_provenance.fetch("review_status") == "approved_original_synthetic_and_public_metadata_only" &&
    v2_provenance.fetch("contains_executable_artifacts") == false &&
    v2_provenance.fetch("contains_malware_or_live_signatures") == false &&
    v2_provenance.fetch("contains_third_party_source") == false &&
    v2_provenance.fetch("contains_private_or_customer_source") == false &&
    v2_provenance.fetch("network_or_provider_data_used") == true
v2_provenance_paths = v2_provenance.fetch("cases").map do |entry|
  relative = entry.fetch("path")
  abort "current guest fixture provenance escaped its root" if relative.start_with?("/") || relative.include?("..")
  path = ROOT.join("tests/conformance/v1", relative)
  abort "current guest fixture provenance digest changed: #{relative}" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
  abort "current guest fixture provenance origin or license changed: #{relative}" unless
    entry.fetch("origin") == "original_synthetic" && entry.fetch("license") == "Apache-2.0"
  relative
end.sort
conformance = json("tests/conformance/v1/manifest.json")
declared_v2_paths = conformance.fetch("cases").each_with_object([]) do |entry, paths|
  paths << entry.fetch("fixture") if entry.fetch("schema").split("#").first.end_with?("-v2.schema.json")
end.sort
abort "current guest fixture provenance is incomplete" unless v2_provenance_paths == declared_v2_paths

seal_provenance = json("tests/conformance/v1/macos-local-vm-release-metadata-seal-fixture-provenance.json")
abort "release-metadata seal fixture review status changed" unless
  seal_provenance.fetch("review_status") == "approved_original_synthetic_and_public_metadata_only" &&
    seal_provenance.fetch("contains_executable_artifacts") == false &&
    seal_provenance.fetch("contains_malware_or_live_signatures") == false &&
    seal_provenance.fetch("contains_third_party_source") == false &&
    seal_provenance.fetch("contains_private_or_customer_source") == false &&
    seal_provenance.fetch("network_or_provider_data_used") == false
seal_fixture_paths = seal_provenance.fetch("cases").map do |entry|
  relative = entry.fetch("path")
  abort "release-metadata seal fixture provenance escaped its root" if relative.start_with?("/") || relative.include?("..")
  path = ROOT.join("tests/conformance/v1", relative)
  abort "release-metadata seal fixture provenance digest changed: #{relative}" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
  abort "release-metadata seal fixture license changed: #{relative}" unless entry.fetch("license") == "Apache-2.0"
  relative
end.sort
declared_seal_paths = conformance.fetch("cases").each_with_object([]) do |entry, paths|
  paths << entry.fetch("fixture") if entry.fetch("schema").start_with?("macos-local-vm-release-metadata-seal")
end.sort
abort "release-metadata seal fixture provenance is incomplete" unless seal_fixture_paths == declared_seal_paths

platform_root = ROOT.join("platform/macos-vm-feasibility")
Dir.glob(platform_root.join("**/*")).select { |path| File.file?(path) }.each do |path|
  prefix = File.binread(path, 4)
  abort "executable artifact committed under macOS VM feasibility source: #{path}" if
    prefix.start_with?("\x7FELF".b) || prefix.start_with?("MZ".b) ||
    ["\xCF\xFA\xED\xFE".b, "\xFE\xED\xFA\xCF".b].include?(prefix)
end

puts "macOS local-VM static contracts passed"
