#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-release-metadata-seal-v1.json"
PROFILE_DIGEST = "4f3b504dd20682de005d074f57fcb3807d721939a5e03fe801274aa6efbe47a5"
SEAL_RELATIVE = "platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
METADATA_SET_DIGEST = "ea29c43f36493f7e61935f33a64822805c8275d804c5384c3e8becea849fc54b"

EXPECTED_MEMBERS = {
  "platform/macos-vm-feasibility/alpine-devel-616ae350.rsa.pub" => "upstream-public-verification-key",
  "platform/macos-vm-feasibility/alpine-upstream-authentication-v2.json" => "upstream-authentication-record",
  "platform/macos-vm-feasibility/guest-assets-v2.json" => "guest-asset-identity",
  "platform/macos-vm-feasibility/guest-license-record-v2.json" => "license-record",
  "platform/macos-vm-feasibility/guest-provenance-v2.json" => "build-provenance-record",
  "platform/macos-vm-feasibility/guest-release-manifest-v2.json" => "guest-release-manifest",
  "platform/macos-vm-feasibility/guest-sbom-v2.spdx.json" => "spdx-sbom",
  "platform/macos-vm-feasibility/guest-vulnerability-assessment-v2.json" => "vulnerability-assessment",
  "platform/macos-vm-feasibility/guest-vulnerability-policy-v2.json" => "vulnerability-policy",
  "profiles/v1/iar-macos-local-vm-guest-supply-chain-v2.json" => "guest-supply-chain-profile",
  "profiles/v1/iar-macos-local-vm-interruption-v2.json" => "host-interruption-profile",
  "profiles/v1/iar-macos-local-vm-resource-canary-v2.json" => "resource-canary-profile",
  "profiles/v1/iar-macos-local-vm-supervisor-v2.json" => "supervisor-profile",
  "profiles/v1/iar-macos-local-vm-synthetic-matrix-v2.json" => "synthetic-matrix-profile",
  "profiles/v1/iar-macos-local-vm-upstream-auth-v2.json" => "upstream-authentication-profile",
  "profiles/v1/iar-macos-local-vm-vulnerability-review-v2.json" => "vulnerability-review-profile"
}.freeze

options = {output: nil}
OptionParser.new do |parser|
  parser.banner = "Usage: check-macos-vm-release-metadata-seal.rb [--output FILE]"
  parser.on("--output FILE", "Write the deterministic receipt") do |value|
    options[:output] = Pathname.new(value).expand_path
  end
end.parse!
abort "unexpected arguments" unless ARGV.empty?

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact_regular_file(root, relative, expected_digest, expected_bytes, label)
  path = root.join(relative).cleanpath
  abort "#{label} escapes repository: #{relative}" unless path.to_s.start_with?(root.to_s + File::SEPARATOR)
  abort "missing #{label}: #{relative}" unless path.file?
  abort "refusing symlinked #{label}: #{relative}" if path.symlink?
  abort "#{label} byte length changed: #{relative}" unless path.size.to_s == expected_bytes
  actual = Digest::SHA256.file(path).hexdigest
  abort "#{label} digest changed: #{relative}" unless actual == expected_digest
  path
end

profile_path = exact_regular_file(
  ROOT,
  PROFILE_RELATIVE,
  PROFILE_DIGEST,
  ROOT.join(PROFILE_RELATIVE).size.to_s,
  "release-metadata seal profile"
)
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-release-metadata-seal-v1.sha256").read.strip
abort "release-metadata seal profile checksum record mismatch" unless
  sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-release-metadata-seal-v1.json"
profile = read_json(profile_path)
abort "profile fixture drifted" unless
  profile_path.binread == ROOT.join("tests/conformance/v1/valid/iar-macos-local-vm-release-metadata-seal-profile.json").binread

seal_path = exact_regular_file(ROOT, SEAL_RELATIVE, SEAL_DIGEST, ROOT.join(SEAL_RELATIVE).size.to_s, "release-metadata seal")
abort "seal fixture drifted" unless
  seal_path.binread == ROOT.join("tests/conformance/v1/valid/macos-local-vm-release-metadata-seal.json").binread
seal = read_json(seal_path)
abort "profile does not bind the exact seal" unless
  profile.fetch("seal_path") == SEAL_RELATIVE &&
    profile.fetch("seal_digest") == "sha256:#{SEAL_DIGEST}" &&
    profile.fetch("required_member_count") == EXPECTED_MEMBERS.length.to_s

members = seal.fetch("members")
paths = members.map { |member| member.fetch("path") }
abort "release-metadata member inventory is not strictly path-sorted" unless paths == paths.sort && paths.uniq == paths
abort "release-metadata member inventory is not closed" unless paths == EXPECTED_MEMBERS.keys.sort

members.each do |member|
  relative = member.fetch("path")
  abort "release-metadata member role changed: #{relative}" unless
    member.fetch("role") == EXPECTED_MEMBERS.fetch(relative)
  exact_regular_file(
    ROOT,
    relative,
    member.fetch("sha256").delete_prefix("sha256:"),
    member.fetch("bytes"),
    "release-metadata member"
  )
end

material = members.map do |member|
  "#{member.fetch('path')}\t#{member.fetch('bytes')}\t#{member.fetch('sha256')}\n"
end.join
actual_set_digest = Digest::SHA256.hexdigest(material)
abort "release-metadata set digest changed" unless
  actual_set_digest == METADATA_SET_DIGEST &&
    seal.fetch("metadata_set_digest") == "sha256:#{METADATA_SET_DIGEST}"

manifest_path = ROOT.join("platform/macos-vm-feasibility/guest-release-manifest-v2.json")
auth_path = ROOT.join("platform/macos-vm-feasibility/alpine-upstream-authentication-v2.json")
assessment_path = ROOT.join("platform/macos-vm-feasibility/guest-vulnerability-assessment-v2.json")
manifest = read_json(manifest_path)
auth = read_json(auth_path)
assessment = read_json(assessment_path)
bindings = seal.fetch("cross_bindings")
abort "guest release identity changed" unless
  seal.fetch("guest_release_id") == manifest.fetch("release_id") &&
    bindings.fetch("guest_release_manifest_digest") == "sha256:#{Digest::SHA256.file(manifest_path).hexdigest}" &&
    bindings.fetch("guest_component_set_digest") == manifest.fetch("component_set_digest")
abort "upstream authentication binding changed" unless
  bindings.fetch("upstream_authentication_record_digest") == "sha256:#{Digest::SHA256.file(auth_path).hexdigest}" &&
    auth.dig("verification", "publisher_algorithm_deprecation_disclosed") == true &&
    auth.dig("verification", "package_signature_verified") == true &&
    auth.dig("signed_package", "package_version") == assessment.dig("candidate", "version")
abort "vulnerability assessment binding changed" unless
  bindings.fetch("vulnerability_assessment_digest") == "sha256:#{Digest::SHA256.file(assessment_path).hexdigest}" &&
    assessment.fetch("guest_release_id") == manifest.fetch("release_id") &&
    assessment.fetch("guest_release_manifest_digest") == bindings.fetch("guest_release_manifest_digest") &&
    assessment.fetch("upstream_authentication_record_digest") == bindings.fetch("upstream_authentication_record_digest")
abort "rollback predecessor binding changed" unless
  bindings.fetch("rollback_predecessor_release_id") == manifest.dig("rollback", "previous_release_id")

now = Time.now.utc
abort "release-metadata seal is not currently valid" unless
  Time.iso8601(seal.fetch("valid_from")) <= now && now < Time.iso8601(seal.fetch("expires_at"))
abort "release-metadata seal expiry differs from guest manifest" unless
  seal.fetch("valid_from") == manifest.fetch("valid_from") &&
    seal.fetch("expires_at") == manifest.fetch("expires_at")

controls = seal.fetch("controls")
abort "release-metadata seal added authority" unless
  controls.fetch("release_metadata_sealed") == true &&
    controls.fetch("offline_validation") == true &&
    controls.fetch("network_access") == false &&
    controls.fetch("credential_access") == false &&
    controls.fetch("repository_source_access") == false
abort "release-metadata seal crossed an unproven gate" unless
  %w[
    advisory_coverage_complete
    vulnerability_assessment_complete
    impresari_publication_attestation_verified
    developer_id_signature_verified
    apple_notarization_verified
    cask_lifecycle_verified
    sealed_distribution
    production_admitted
    analyzer_execution
    authority_added
  ].none? { |claim| controls.fetch(claim) }

receipt = {
  "schema_name" => "macos-local-vm-release-metadata-seal-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "seal_id" => seal.fetch("seal_id"),
  "seal_digest" => "sha256:#{SEAL_DIGEST}",
  "guest_release_id" => seal.fetch("guest_release_id"),
  "metadata_set_digest" => seal.fetch("metadata_set_digest"),
  "member_count" => members.length.to_s,
  "seal_exact" => true,
  "member_inventory_closed" => true,
  "member_identities_exact" => true,
  "metadata_set_digest_exact" => true,
  "cross_bindings_exact" => true,
  "expiry_valid" => true,
  "offline_validation" => true,
  "release_metadata_sealed" => true,
  "advisory_coverage_complete" => false,
  "vulnerability_assessment_complete" => false,
  "impresari_publication_attestation_verified" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "cask_lifecycle_verified" => false,
  "sealed_distribution" => false,
  "production_admitted" => false,
  "analyzer_execution" => false,
  "authority_added" => false
}

fixture_receipt = read_json(ROOT.join("tests/conformance/v1/valid/macos-local-vm-release-metadata-seal-receipt.json"))
abort "release-metadata seal receipt fixture drifted" unless receipt == fixture_receipt

if options[:output]
  options.fetch(:output).dirname.mkpath
  options.fetch(:output).write(JSON.pretty_generate(receipt) + "\n")
end

puts "macOS local-VM release metadata sealed: release=#{seal.fetch('guest_release_id')} members=#{members.length} sealed_distribution=false"
