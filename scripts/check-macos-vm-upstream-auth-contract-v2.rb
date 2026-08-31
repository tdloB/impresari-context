#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "c4840caf688d47fbc6d42fe4005c59dc1fd25b6b4e5fd5f3cc29d529077e7e88"
RECORD_DIGEST = "0dfc2b183b60145de395074e0371213aadcf51c567d90dcf0b9ad68cc6df9483"
MANIFEST_DIGEST = "d0aad27ee855cac8969b189ab24cd10b58d6ceffae42f43ff0fbf4952c1785ff"
KEY_DIGEST = "d11f6b21c61b4274e182eb888883a8ba8acdbf820dcc7a6d82a7d9fc2fd2836d"

def exact(relative, digest, bytes = nil)
  path = ROOT.join(relative)
  abort "missing or symlinked APKv2 authentication input: #{relative}" unless path.file? && !path.symlink?
  abort "APKv2 authentication input size changed: #{relative}" if bytes && path.size.to_s != bytes
  abort "APKv2 authentication input digest changed: #{relative}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid APKv2 authentication JSON: #{path}: #{e.message}"
end

profile_path = exact("profiles/v1/iar-macos-local-vm-upstream-auth-v2.json", PROFILE_DIGEST)
abort "APKv2 profile sidecar changed" unless
  ROOT.join("profiles/v1/iar-macos-local-vm-upstream-auth-v2.sha256").read.strip ==
    "#{PROFILE_DIGEST}  iar-macos-local-vm-upstream-auth-v2.json"
abort "APKv2 profile fixture changed" unless
  profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-upstream-auth-profile-v2.json").binread
profile = json(profile_path)

record_path = exact("platform/macos-vm-feasibility/alpine-upstream-authentication-v2.json", RECORD_DIGEST)
abort "APKv2 authentication record fixture changed" unless
  record_path.binread == ROOT.join("tests/conformance/v1/valid/macos-local-vm-upstream-auth-record-v2.json").binread
record = json(record_path)
manifest_path = exact("platform/macos-vm-feasibility/guest-release-manifest-v2.json", MANIFEST_DIGEST)
manifest = json(manifest_path)
key = record.fetch("apk_signing_key")
exact(key.fetch("path"), KEY_DIGEST, key.fetch("bytes"))

abort "APKv2 profile lost exact cross-bindings" unless
  profile.fetch("authentication_record_digest") == "sha256:#{RECORD_DIGEST}" &&
    profile.fetch("guest_release_manifest_digest") == "sha256:#{MANIFEST_DIGEST}" &&
    profile.fetch("apk_signing_key_digest") == "sha256:#{KEY_DIGEST}"

index = record.fetch("signed_index")
package = record.fetch("signed_package")
manifest_assets = manifest.fetch("upstream_artifacts").to_h { |artifact| [artifact.fetch("name"), artifact] }
abort "signed APKINDEX is not exactly bound to the guest manifest" unless
  manifest_assets.fetch("APKINDEX.tar.gz").values_at("url", "bytes", "sha256") ==
    index.values_at("url", "bytes", "sha256")
abort "signed linux-virt package is not exactly bound to the guest manifest" unless
  manifest_assets.fetch("linux-virt-6.18.48-r0.apk").values_at("url", "bytes", "sha256") ==
    package.values_at("url", "bytes", "sha256")

abort "APKv2 package identity changed" unless
  package.values_at("package_name", "package_version", "architecture", "origin", "commit") ==
    ["linux-virt", "6.18.48-r0", "aarch64", "linux-lts", "c83b91e0fde4c1bada9b80d4e67c395b5335597b"]
abort "APKv2 signed datahash changed" unless
  package.fetch("datahash") == "sha256:e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263"

verification = record.fetch("verification")
abort "APKv2 authentication record lost verified evidence" unless
  %w[trust_anchor_verified index_signature_verified package_signature_verified package_datahash_verified index_package_identity_matched component_identities_verified publisher_algorithm_deprecation_disclosed]
    .all? { |field| verification.fetch(field) == true }
abort "APKv2 authentication crossed a release boundary" unless
  verification.fetch("runtime_network_required") == false &&
    verification.fetch("release_metadata_sealed") == false &&
    verification.fetch("vulnerability_assessment_complete") == false &&
    verification.fetch("production_admitted") == false &&
    verification.fetch("analyzer_execution") == false &&
    verification.fetch("authority_added") == false

live_check = ROOT.join("scripts/verify-macos-vm-alpine-package.sh").read
%w[verify-macos-vm-alpine-archive.sh verify-alpine-apkv2.rb APKINDEX linux-virt].each do |required|
  abort "live APKv2 authentication check lost #{required}" unless live_check.include?(required)
end
%w[curl wget].each do |forbidden|
  abort "live APKv2 authentication check acquired retrieval authority: #{forbidden}" if live_check.include?(forbidden)
end

committed_packages = Dir.glob(ROOT.join("platform/macos-vm-feasibility/*.apk"))
abort "signed Alpine package was committed" unless committed_packages.empty?

receipt = json(ROOT.join("tests/conformance/v1/valid/macos-local-vm-upstream-auth-receipt-v2.json"))
abort "APKv2 receipt lost exact non-production result" unless
  receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    receipt.fetch("authentication_record_digest") == "sha256:#{RECORD_DIGEST}" &&
    receipt.fetch("upstream_publisher_authentication_verified") == true &&
    receipt.fetch("deprecated_provider_algorithm_disclosed") == true &&
    receipt.fetch("production_admitted") == false

puts "macOS local-VM APKv2 upstream authentication contracts passed: package=linux-virt-6.18.48-r0 release_metadata_sealed=false"
