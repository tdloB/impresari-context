#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "optparse"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-guest-supply-chain-v2.json"
PROFILE_DIGEST = "389e5c55dbc3573a69561a9b999eb173e7f15e46a9a77e007d7fdc6dec8e48a2"
MANIFEST_RELATIVE = "platform/macos-vm-feasibility/guest-release-manifest-v2.json"
MANIFEST_DIGEST = "d0aad27ee855cac8969b189ab24cd10b58d6ceffae42f43ff0fbf4952c1785ff"

options = {prepared_assets: nil, output: nil}
OptionParser.new do |parser|
  parser.banner = "Usage: check-macos-vm-guest-supply-chain.rb [--prepared-assets DIR] [--output FILE]"
  parser.on("--prepared-assets DIR", "Verify every built guest component under DIR") do |value|
    options[:prepared_assets] = Pathname.new(value).expand_path
  end
  parser.on("--output FILE", "Write the deterministic receipt to FILE") do |value|
    options[:output] = Pathname.new(value).expand_path
  end
end.parse!
abort "unexpected arguments" unless ARGV.empty?

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact_digest(path, expected, label)
  abort "missing #{label}: #{path}" unless path.file?
  abort "refusing symlinked #{label}: #{path}" if path.symlink?
  actual = Digest::SHA256.file(path).hexdigest
  abort "#{label} digest changed: #{path}" unless actual == expected
  actual
end

profile_path = ROOT.join(PROFILE_RELATIVE)
exact_digest(profile_path, PROFILE_DIGEST, "guest supply-chain profile")
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-guest-supply-chain-v2.sha256").read.strip
expected_sidecar = "#{PROFILE_DIGEST}  iar-macos-local-vm-guest-supply-chain-v2.json"
abort "guest supply-chain profile checksum record mismatch" unless sidecar == expected_sidecar
profile = read_json(profile_path)

manifest_path = ROOT.join(MANIFEST_RELATIVE)
exact_digest(manifest_path, MANIFEST_DIGEST, "guest release manifest")
abort "guest release manifest fixture differs from the frozen manifest" unless
  manifest_path.binread ==
    ROOT.join("tests/conformance/v1/valid/macos-local-vm-guest-release-manifest-v2.json").binread
manifest = read_json(manifest_path)
abort "profile does not bind the exact manifest" unless
  profile.fetch("manifest_path") == MANIFEST_RELATIVE &&
    profile.fetch("manifest_digest") == "sha256:#{MANIFEST_DIGEST}"

record_keys = %w[sbom licenses provenance vulnerability_policy]
abort "guest release record set changed" unless manifest.fetch("records").keys.sort == record_keys.sort
record_keys.each do |key|
  record = manifest.fetch("records").fetch(key)
  relative = record.fetch("path")
  path = ROOT.join(relative).cleanpath
  abort "guest release record escapes repository: #{relative}" unless
    path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  exact_digest(path, record.fetch("sha256").delete_prefix("sha256:"), "#{key} record")
end

provenance = read_json(ROOT.join(manifest.dig("records", "provenance", "path")))
provenance.fetch("build_inputs").each do |input|
  relative = input.fetch("path")
  path = ROOT.join(relative).cleanpath
  abort "build input escapes repository: #{relative}" unless
    path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  exact_digest(path, input.fetch("sha256"), "build input")
end
abort "guest provenance lost publisher authentication" unless
  provenance.fetch("upstream_publisher_authentication") == "verified-apkv2-from-openpgp-authenticated-key"
abort "offline admission acquired network authority" unless
  provenance.fetch("network_used_by_offline_admission") == false

components = manifest.fetch("guest_components")
expected_component_ids = %w[
  linux-kernel-image
  virtio-blk-kernel-module
  synthetic-guest-init
  synthetic-guest-initramfs
  synthetic-resource-guest-init
  synthetic-resource-guest-initramfs
]
abort "guest component inventory is incomplete" unless
  components.map { |component| component.fetch("component_id") }.sort == expected_component_ids.sort
component_set_material = components.map { |component| component.fetch("sha256").delete_prefix("sha256:") }.sort.join("\n") + "\n"
component_set_digest = Digest::SHA256.hexdigest(component_set_material)
abort "guest component set digest mismatch" unless
  manifest.fetch("component_set_digest") == "sha256:#{component_set_digest}"

sbom = read_json(ROOT.join(manifest.dig("records", "sbom", "path")))
abort "guest SBOM is not SPDX 2.3" unless sbom.fetch("spdxVersion") == "SPDX-2.3"
sbom_checksums = sbom.fetch("packages").flat_map do |package|
  package.fetch("checksums").select { |checksum| checksum.fetch("algorithm") == "SHA256" }
    .map { |checksum| checksum.fetch("checksumValue") }
end
%w[
  c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616
  68d5be977b2bd1bc7df2bcfc8bdb077bb03f9afc390d7c099f23437ced1598bf
  7e5add284e63a059c5df32b66a2bd18b0d96a5e4b5809c00a02bd9a82e7fa3f6
].each { |digest| abort "guest SBOM omits component #{digest}" unless sbom_checksums.include?(digest) }

licenses = read_json(ROOT.join(manifest.dig("records", "licenses", "path")))
abort "guest license record is incomplete" unless
  licenses.fetch("components").map { |component| component.fetch("component_id") }.sort ==
    %w[alpine-linux-virt-kernel impresari-context-synthetic-guest-init impresari-context-synthetic-resource-guest-init].sort
abort "guest license record overclaims legal advice" unless licenses.fetch("legal_advice") == false

vulnerability = read_json(ROOT.join(manifest.dig("records", "vulnerability_policy", "path")))
now = Time.now.utc
valid_from = Time.iso8601(manifest.fetch("valid_from"))
expires_at = Time.iso8601(manifest.fetch("expires_at"))
abort "guest release is not currently valid" unless valid_from <= now && now < expires_at
abort "vulnerability policy expiry differs from the release" unless
  Time.iso8601(vulnerability.fetch("expires_at")) == expires_at
abort "incomplete vulnerability coverage does not deny production" unless
  vulnerability.fetch("assessment_state") == "current-package-reviewed-advisory-coverage-incomplete" &&
    vulnerability.fetch("admission_when_assessment_incomplete") == "deny-production-admission"

rollback = manifest.fetch("rollback")
abort "guest update may occur during a job" unless rollback.fetch("updates_between_jobs_only") == true
abort "replacement candidate must bind the previous release" unless
  rollback.fetch("current_release_id") == manifest.fetch("release_id") &&
    rollback.fetch("previous_release_id") == "iar-macos-local-vm-guest-2026-08-30.1" &&
    rollback.fetch("anti_rollback_enforced") == false

controls = manifest.fetch("controls")
abort "guest release acquired self-update or network authority" unless
  controls.fetch("guest_self_update") == false &&
    controls.fetch("guest_network_available") == false &&
    controls.fetch("offline_admission") == true
abort "candidate release crossed an incomplete production gate" unless
  controls.fetch("publisher_authentication_verified") == true &&
    controls.fetch("cryptographic_signature_verified") == true &&
    controls.fetch("notarized_distribution_verified") == false &&
    controls.fetch("vulnerability_assessment_complete") == false &&
    controls.fetch("production_admitted") == false &&
    controls.fetch("analyzer_execution") == false &&
    controls.fetch("authority_added") == false

prepared_verified = false
if options[:prepared_assets]
  root = options.fetch(:prepared_assets)
  abort "prepared-asset root is missing or symlinked: #{root}" unless root.directory? && !root.symlink?
  components.each do |component|
    relative = component.fetch("path")
    path = root.join(relative).cleanpath
    abort "prepared component escapes root: #{relative}" unless
      path.to_s.start_with?(root.to_s + File::SEPARATOR)
    exact_digest(path, component.fetch("sha256").delete_prefix("sha256:"), "prepared guest component")
    abort "prepared component size changed: #{relative}" unless path.size.to_s == component.fetch("bytes")
  end
  prepared_verified = true
end

receipt = {
  "schema_name" => "macos-local-vm-guest-supply-chain-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "manifest_release_id" => manifest.fetch("release_id"),
  "manifest_digest" => "sha256:#{MANIFEST_DIGEST}",
  "component_set_digest" => manifest.fetch("component_set_digest"),
  "manifest_exact" => true,
  "record_identities_exact" => true,
  "build_input_identities_exact" => true,
  "component_inventory_complete" => true,
  "sbom_recorded" => true,
  "licenses_recorded" => true,
  "provenance_recorded" => true,
  "vulnerability_policy_recorded" => true,
  "expiry_valid" => true,
  "rollback_identity_bound" => true,
  "offline_validation" => true,
  "prepared_artifacts_verified" => prepared_verified,
  "publisher_authentication_verified" => true,
  "vulnerability_assessment_complete" => false,
  "cryptographic_signature_verified" => true,
  "notarized_distribution_verified" => false,
  "sealed_distribution" => false,
  "production_admitted" => false,
  "analyzer_execution" => false,
  "authority_added" => false
}

if options[:output]
  options.fetch(:output).dirname.mkpath
  options.fetch(:output).write(JSON.pretty_generate(receipt) + "\n")
end
puts "macOS local-VM guest supply-chain contract passed: release=#{manifest.fetch('release_id')} prepared_artifacts_verified=#{prepared_verified} sealed_distribution=false"
