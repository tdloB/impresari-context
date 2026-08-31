#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "8118ca67549f145531697fbc4ffb60f571f7960f4d1e3ddf954d7931454b6785"
RECORD_DIGEST = "f1ef35d52427a15179777bafd67eddd2e545c8e853ddc425377544732b955476"
MANIFEST_DIGEST = "02e5ba57ef2bb3be02cef4e978d3e518ec39a5db014988036164d2821e19b7e6"
EVIDENCE_PROVENANCE_DIGEST = "0ce18fc19393c1a14e2e15912b3684047b7266a3ec5283a790bb5a734b10f001"

def exact(path, digest, bytes = nil)
  abort "missing or symlinked upstream-authentication input: #{path}" unless path.file? && !path.symlink?
  abort "upstream-authentication input size changed: #{path}" if bytes && path.size.to_s != bytes
  abort "upstream-authentication input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid upstream-authentication JSON: #{path}: #{e.message}"
end

profile_path = ROOT.join("profiles/v1/iar-macos-local-vm-upstream-auth-v1.json")
exact(profile_path, PROFILE_DIGEST)
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-upstream-auth-v1.sha256").read.strip
abort "upstream-authentication profile sidecar changed" unless
  sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-upstream-auth-v1.json"
abort "upstream-authentication profile fixture changed" unless
  profile_path.binread ==
    ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-upstream-auth-profile.json").binread
profile = json(profile_path)

record_path = ROOT.join("platform/macos-vm-feasibility/alpine-upstream-authentication.json")
exact(record_path, RECORD_DIGEST)
abort "upstream-authentication record fixture changed" unless
  record_path.binread ==
    ROOT.join("tests/conformance/v1/valid/macos-local-vm-upstream-auth-record.json").binread
record = json(record_path)
abort "profile does not bind the exact authentication record" unless
  profile.fetch("authentication_record_digest") == "sha256:#{RECORD_DIGEST}" &&
    profile.fetch("authentication_record_path") == record_path.relative_path_from(ROOT).to_s

manifest_path = ROOT.join("platform/macos-vm-feasibility/guest-release-manifest.json")
exact(manifest_path, MANIFEST_DIGEST)
manifest = json(manifest_path)
abort "profile does not bind the exact guest release manifest" unless
  profile.fetch("guest_release_manifest_digest") == "sha256:#{MANIFEST_DIGEST}"

key = record.fetch("release_key")
key_path = ROOT.join(key.fetch("path"))
exact(key_path, key.fetch("sha256").delete_prefix("sha256:"), key.fetch("bytes"))
abort "release-key fingerprint differs from the official pinned fingerprint" unless
  key.fetch("fingerprint") == "0482D84022F52DF1C4E7CD43293ACD0907D9495A" &&
    profile.fetch("release_key_fingerprint") == key.fetch("fingerprint")

archive = record.fetch("signed_archive")
signature_path = ROOT.join(archive.fetch("signature_path"))
exact(signature_path, archive.fetch("signature_sha256").delete_prefix("sha256:"), archive.fetch("signature_bytes"))
abort "signed archive URL is not exact and versioned" unless
  archive.fetch("url") == "https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64/alpine-netboot-3.24.1-aarch64.tar.gz"

evidence_provenance_path =
  ROOT.join("platform/macos-vm-feasibility/alpine-upstream-evidence-provenance.json")
exact(evidence_provenance_path, EVIDENCE_PROVENANCE_DIGEST)
evidence_provenance = json(evidence_provenance_path)
provenance_paths = evidence_provenance.fetch("files").to_h do |entry|
  [entry.fetch("path"), entry.fetch("sha256")]
end
abort "third-party release evidence lacks exact provenance" unless
  provenance_paths == {
    key_path.relative_path_from(ROOT).to_s => key.fetch("sha256").delete_prefix("sha256:"),
    signature_path.relative_path_from(ROOT).to_s => archive.fetch("signature_sha256").delete_prefix("sha256:")
  }

manifest_assets = manifest.fetch("upstream_artifacts").to_h do |asset|
  [asset.fetch("name"), [asset.fetch("bytes"), asset.fetch("sha256")]]
end
authenticated_assets = record.fetch("authenticated_embedded_artifacts").to_h do |asset|
  [asset.fetch("guest_asset_name"), [asset.fetch("bytes"), asset.fetch("sha256")]]
end
abort "authenticated Alpine assets do not exactly bind the guest release manifest" unless
  authenticated_assets == manifest_assets

verification = record.fetch("verification")
abort "upstream authentication record lost verified evidence" unless
  verification.fetch("archive_digest_verified") == true &&
    verification.fetch("detached_signature_verified") == true &&
    verification.fetch("signing_fingerprint_verified") == true &&
    verification.fetch("embedded_artifact_identities_verified") == true &&
    verification.fetch("upstream_publisher_authentication_verified") == true
abort "upstream authentication crossed a release or runtime boundary" unless
  verification.fetch("archive_committed") == false &&
    verification.fetch("runtime_network_required") == false &&
    verification.fetch("release_metadata_sealed") == false &&
    verification.fetch("vulnerability_assessment_complete") == false &&
    verification.fetch("production_admitted") == false &&
    verification.fetch("analyzer_execution") == false &&
    verification.fetch("authority_added") == false

live_check = ROOT.join("scripts/verify-macos-vm-alpine-archive.sh").read
%w[gpgv boot/vmlinuz-virt boot/initramfs-virt 0482D84022F52DF1C4E7CD43293ACD0907D9495A].each do |required|
  abort "live upstream-authentication check lost #{required}" unless live_check.include?(required)
end
%w[curl wget].each do |forbidden|
  abort "live upstream-authentication check acquired network retrieval: #{forbidden}" if live_check.include?(forbidden)
end

committed_archives = Dir.glob(ROOT.join("platform/macos-vm-feasibility/*.tar.gz"))
abort "signed Alpine archive was committed" unless committed_archives.empty?

puts "macOS local-VM upstream authentication contracts passed: fingerprint=#{key.fetch('fingerprint')} release_metadata_sealed=false"
