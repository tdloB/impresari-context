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

matrix_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-synthetic-matrix-v1.json")
matrix_profile_digest = Digest::SHA256.file(matrix_profile_path).hexdigest
abort "macOS VM matrix profile digest changed" unless matrix_profile_digest == "a411dc8d896b9b516cb535786fe2d12f17c6bfed3b39b2104c040e7556507522"
matrix_sha_record = read("profiles/v1/iar-macos-local-vm-synthetic-matrix-v1.sha256").strip
abort "macOS VM matrix profile checksum record mismatch" unless
  matrix_sha_record == "#{matrix_profile_digest}  iar-macos-local-vm-synthetic-matrix-v1.json"
abort "macOS VM matrix profile fixture differs from the frozen profile" unless
  matrix_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-synthetic-matrix-profile.json").binread

matrix_profile = json("profiles/v1/iar-macos-local-vm-synthetic-matrix-v1.json")
abort "macOS VM matrix guest identity is not exact" unless
  matrix_profile.dig("guest", "guest_init_sha256") == "68d5be977b2bd1bc7df2bcfc8bdb077bb03f9afc390d7c099f23437ced1598bf" &&
    matrix_profile.dig("guest", "initramfs_sha256") == "cc87a9a68d06826277dd759befd318272a7876540b4287cfd6fe0ac67552bfbf"
abort "macOS VM matrix expands authority" unless
  matrix_profile.dig("controls", "analyzer_execution") == false &&
    matrix_profile.dig("controls", "production_admitted") == false &&
    matrix_profile.dig("controls", "authority_added") == false

supervisor_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-supervisor-v1.json")
supervisor_profile_digest = Digest::SHA256.file(supervisor_profile_path).hexdigest
abort "macOS VM supervisor profile digest changed" unless
  supervisor_profile_digest == "f82de32acc12d6cad53c9a8c4a225ea4a352bd91d39222969e9e1efa40035e85"
supervisor_sha_record = read("profiles/v1/iar-macos-local-vm-supervisor-v1.sha256").strip
abort "macOS VM supervisor profile checksum record mismatch" unless
  supervisor_sha_record == "#{supervisor_profile_digest}  iar-macos-local-vm-supervisor-v1.json"
abort "macOS VM supervisor profile fixture differs from the frozen profile" unless
  supervisor_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-supervisor-profile.json").binread
supervisor_profile = json("profiles/v1/iar-macos-local-vm-supervisor-v1.json")
abort "macOS VM supervisor profile expands authority" unless
  supervisor_profile.dig("controls", "analyzer_execution") == false &&
    supervisor_profile.dig("controls", "production_admitted") == false &&
    supervisor_profile.dig("controls", "authority_added") == false

resource_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-resource-canary-v1.json")
resource_profile_digest = Digest::SHA256.file(resource_profile_path).hexdigest
abort "macOS VM resource/canary profile digest changed" unless
  resource_profile_digest == "b711c69b7a46ad26bb7181622edc69366557886cfe43ef3ca2ef05283d861e7e"
resource_sha_record = read("profiles/v1/iar-macos-local-vm-resource-canary-v1.sha256").strip
abort "macOS VM resource/canary profile checksum record mismatch" unless
  resource_sha_record == "#{resource_profile_digest}  iar-macos-local-vm-resource-canary-v1.json"
abort "macOS VM resource/canary profile fixture differs from the frozen profile" unless
  resource_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-resource-canary-profile.json").binread
resource_profile = json("profiles/v1/iar-macos-local-vm-resource-canary-v1.json")
abort "macOS VM resource/canary profile expands authority" unless
  resource_profile.dig("controls", "analyzer_execution") == false &&
    resource_profile.dig("controls", "production_admitted") == false &&
    resource_profile.dig("controls", "authority_added") == false
abort "macOS VM resource/canary profile lost exact cgroup limits" unless
  resource_profile.dig("limits", "job_memory_bytes") == "33554432" &&
    resource_profile.dig("limits", "job_cpu_quota_usec") == "10000" &&
    resource_profile.dig("limits", "job_cpu_period_usec") == "100000" &&
    resource_profile.dig("limits", "job_pids") == "8"

interruption_profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-interruption-v1.json")
interruption_profile_digest = Digest::SHA256.file(interruption_profile_path).hexdigest
abort "macOS VM interruption profile digest changed" unless
  interruption_profile_digest == "e5f54da3e1fbce7ea7f839dc723e4b288ff7113fd9c85950df3970ae18737fd1"
interruption_sha_record = read("profiles/v1/iar-macos-local-vm-interruption-v1.sha256").strip
abort "macOS VM interruption profile checksum record mismatch" unless
  interruption_sha_record == "#{interruption_profile_digest}  iar-macos-local-vm-interruption-v1.json"
abort "macOS VM interruption profile fixture differs from the frozen profile" unless
  interruption_profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-interruption-profile.json").binread
interruption_profile = json("profiles/v1/iar-macos-local-vm-interruption-v1.json")
abort "macOS VM interruption profile confuses simulation with real sleep" unless
  interruption_profile.fetch("automated_evidence") == "synthetic-job-private-trigger" &&
    interruption_profile.fetch("manual_evidence_required") == "os-will-sleep" &&
    interruption_profile.dig("controls", "real_host_sleep_observed") == false
abort "macOS VM interruption profile expands authority" unless
  interruption_profile.dig("controls", "analyzer_execution") == false &&
    interruption_profile.dig("controls", "production_admitted") == false &&
    interruption_profile.dig("controls", "authority_added") == false

assets = json("platform/macos-vm-feasibility/guest-assets.json")
abort "macOS VM guest asset source is not exact Alpine HTTPS" unless assets.fetch("artifacts").all? do |artifact|
  artifact.fetch("url").start_with?("https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64/netboot/") &&
    artifact.fetch("sha256").match?(/\A[0-9a-f]{64}\z/) &&
    artifact.fetch("bytes").match?(/\A[1-9][0-9]*\z/)
end
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
  controller.include?("initramfsDigest = \"cc87a9a68d06826277dd759befd318272a7876540b4287cfd6fe0ac67552bfbf\"")
abort "macOS VM controller lost the exact external cancellation marker" unless
  controller.include?("cancel.request") && controller.include?("IMPRESARI_VM_CANCEL_V1") == false
abort "macOS VM controller can overclaim VM confinement" unless controller.include?("vmConfined: false")

runner = read("crates/context-analyzer-runner/src/lib.rs")
abort "Rust supervisor profile identity drifted" unless
  runner.include?("iar-macos-local-vm-supervisor-v1") &&
    runner.include?("sha256:f82de32acc12d6cad53c9a8c4a225ea4a352bd91d39222969e9e1efa40035e85")
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

platform_root = ROOT.join("platform/macos-vm-feasibility")
Dir.glob(platform_root.join("**/*")).select { |path| File.file?(path) }.each do |path|
  prefix = File.binread(path, 4)
  abort "executable artifact committed under macOS VM feasibility source: #{path}" if
    prefix.start_with?("\x7FELF".b) || prefix.start_with?("MZ".b) ||
    ["\xCF\xFA\xED\xFE".b, "\xFE\xED\xFA\xCF".b].include?(prefix)
end

puts "macOS local-VM static contracts passed"
