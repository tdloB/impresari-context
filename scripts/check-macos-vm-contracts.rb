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
abort "macOS VM controller can overclaim VM confinement" unless controller.include?("vmConfined: false")

guest = read("platform/macos-vm-feasibility/Sources/GuestInit/main.c")
%w[fork( execve( execl( system( popen( socket( connect(].each do |forbidden|
  abort "synthetic guest init acquired forbidden execution/network surface: #{forbidden}" if guest.include?(forbidden)
end
abort "synthetic guest init lost raw scratch ceiling" unless guest.include?("#define SCRATCH_BYTES 1048576")
abort "synthetic guest init lost read-only input probe" unless guest.include?("input_write_denied")
abort "synthetic guest init lost network-device absence probe" unless guest.include?("network_device_absent")

platform_root = ROOT.join("platform/macos-vm-feasibility")
Dir.glob(platform_root.join("**/*")).select { |path| File.file?(path) }.each do |path|
  prefix = File.binread(path, 4)
  abort "executable artifact committed under macOS VM feasibility source: #{path}" if
    prefix.start_with?("\x7FELF".b) || prefix.start_with?("MZ".b) ||
    ["\xCF\xFA\xED\xFE".b, "\xFE\xED\xFA\xCF".b].include?(prefix)
end

puts "macOS local-VM static contracts passed"
