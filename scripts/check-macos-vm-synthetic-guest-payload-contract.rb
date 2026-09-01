#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/synthetic-guest-payload-contract-v1.json"
CONTRACT_DIGEST = "4e43e28f325d7ab67ff2bb23595eb9273320ff5e8597553b9a681bfdc51033d4"
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-synthetic-guest-payload-v1.json"
PROFILE_DIGEST = "3cbcd2a65c268477ab0d01207f0a97da3bbc5543c6415c67140ff9f19b6b10cc"
MANIFEST_RELATIVE = "platform/macos-vm-feasibility/guest-release-manifest-v2.json"
MANIFEST_DIGEST = "d0aad27ee855cac8969b189ab24cd10b58d6ceffae42f43ff0fbf4952c1785ff"
SEAL_RELATIVE = "platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"

FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-synthetic-guest-payload-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-synthetic-guest-payload-contract.json" => "b852d058a89a32d4c701ae0e1bfc2bfd9e0308d65d123334516fc175fff1431c",
  "valid/macos-local-vm-synthetic-guest-payload-receipt.json" => "bb3e0d3c24c062bd01e5eb955768145c353863e61132edb05c174f5e2365e5c1",
  "invalid/macos-local-vm-synthetic-guest-payload-overclaim.json" => "7cb3b7d7bd1c7f005b472e376b519e7358ed0320582c05e17426acb5cc9cc5a0"
}.freeze

EXPECTED_MEMBERS = {
  "Image" => ["36175872", "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5", "linux-kernel-image"],
  "impresari-initramfs.gz" => ["38207", "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b", "synthetic-guest-initramfs"]
}.freeze
EXPECTED_EXCLUDED = %w[
  synthetic-guest-init
  synthetic-resource-guest-init
  synthetic-resource-guest-initramfs
  virtio-blk-kernel-module
].freeze
FALSE_CLAIMS = %w[
  workspace_scanned network_access credential_access process_launch
  guest_payload_materialized runnable_guest_artifacts_retained
  release_bundle_assembled archive_created developer_id_signature_verified
  apple_notarization_verified bundle_installed cask_created vm_launch
  analyzer_execution release_identity_bound production_admitted
  macos_iar_1b_admitted authority_added
].freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(relative, digest, label)
  path = ROOT.join(relative).cleanpath
  abort "#{label} escapes repository: #{relative}" unless path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  abort "missing #{label}: #{relative}" unless path.file?
  abort "refusing symlinked #{label}: #{relative}" if path.symlink?
  abort "#{label} digest changed: #{relative}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

contract = json(exact(CONTRACT_RELATIVE, CONTRACT_DIGEST, "guest payload contract"))
profile = json(exact(PROFILE_RELATIVE, PROFILE_DIGEST, "guest payload profile"))
manifest = json(exact(MANIFEST_RELATIVE, MANIFEST_DIGEST, "guest release manifest"))
seal = json(exact(SEAL_RELATIVE, SEAL_DIGEST, "guest metadata seal"))

sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-synthetic-guest-payload-v1.sha256").read.strip
abort "guest payload profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-synthetic-guest-payload-v1.json"
abort "profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-synthetic-guest-payload-profile.json"))
abort "contract fixture drifted" unless contract == json(FIXTURE_ROOT.join("valid/macos-local-vm-synthetic-guest-payload-contract.json"))
abort "profile contract binding changed" unless
  profile.fetch("contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}"

abort "guest release binding changed" unless
  contract.fetch("guest_release_id") == manifest.fetch("release_id") &&
    contract.fetch("guest_release_manifest_path") == MANIFEST_RELATIVE &&
    contract.fetch("guest_release_manifest_digest") == "sha256:#{MANIFEST_DIGEST}" &&
    contract.fetch("guest_component_set_digest") == manifest.fetch("component_set_digest") &&
    contract.fetch("guest_metadata_seal_path") == SEAL_RELATIVE &&
    contract.fetch("guest_metadata_seal_digest") == "sha256:#{SEAL_DIGEST}" &&
    contract.fetch("guest_metadata_set_digest") == seal.fetch("metadata_set_digest")

members = contract.fetch("payload_members")
abort "payload member order or count changed" unless members.map { |entry| entry.fetch("relative_path") } == EXPECTED_MEMBERS.keys
manifest_by_id = manifest.fetch("guest_components").to_h { |entry| [entry.fetch("component_id"), entry] }
members.each do |member|
  bytes, digest, component_id = EXPECTED_MEMBERS.fetch(member.fetch("relative_path"))
  source = manifest_by_id.fetch(component_id)
  abort "payload member identity changed" unless
    member.fetch("mode") == "0644" && member.fetch("bytes") == bytes &&
      member.fetch("sha256") == "sha256:#{digest}" &&
      member.fetch("source_component_id") == component_id &&
      source.fetch("bytes") == bytes && source.fetch("sha256") == "sha256:#{digest}" &&
      member.fetch("bundle_path") == "#{contract.fetch('payload_root')}/#{member.fetch('relative_path')}" &&
      member.fetch("controller_asset_name") == member.fetch("relative_path")
end

excluded = contract.fetch("excluded_manifest_components")
abort "excluded guest component set changed" unless excluded.map { |entry| entry.fetch("component_id") } == EXPECTED_EXCLUDED
abort "manifest projection is not closed" unless
  (members.map { |entry| entry.fetch("source_component_id") } + EXPECTED_EXCLUDED).sort == manifest_by_id.keys.sort

controller = contract.fetch("controller_binding")
controller_path = exact(controller.fetch("source_path"), controller.fetch("source_sha256").delete_prefix("sha256:"), "controller source")
controller_source = controller_path.read
abort "controller asset-name binding changed" unless
  controller.fetch("kernel_asset_name") == "Image" &&
    controller.fetch("ordinary_initramfs_asset_name") == "impresari-initramfs.gz" &&
    controller_source.include?('appendingPathComponent("Image", isDirectory: false)') &&
    controller_source.include?('? "impresari-resource-initramfs.gz"') &&
    controller_source.include?(': "impresari-initramfs.gz"') &&
    controller_source.include?("private let maximumInitramfsBytes: UInt64 = 2_097_152")

recipe = contract.fetch("future_materialization_recipe")
public_input = recipe.fetch("public_input")
assets = json(exact("platform/macos-vm-feasibility/guest-assets-v2.json", "5cbf2eb61efc744e7e1ebf969641ec3f15cd76f16d0bfba6c1a5cbb580730275", "guest asset record"))
apk = assets.fetch("artifacts").find { |entry| entry.fetch("name") == "linux-virt-6.18.48-r0.apk" }
abort "authenticated public input changed" unless
  public_input.fetch("url") == apk.fetch("url") && public_input.fetch("bytes") == apk.fetch("bytes") &&
    public_input.fetch("sha256") == "sha256:#{apk.fetch('sha256')}" &&
    public_input.fetch("verification_key_sha256") == "sha256:#{assets.dig('publisher_authentication', 'key_sha256')}" &&
    public_input.fetch("package_signature_sha256") == "sha256:#{assets.dig('publisher_authentication', 'package_signature_sha256')}" &&
    public_input.fetch("package_datahash") == "sha256:#{assets.dig('publisher_authentication', 'package_datahash')}" &&
    public_input.fetch("package_name") == assets.dig("package", "name") &&
    public_input.fetch("package_version") == assets.dig("package", "version") &&
    public_input.fetch("package_commit") == assets.dig("package", "commit")

recipe.fetch("build_inputs").each do |input|
  exact(input.fetch("path"), input.fetch("sha256").delete_prefix("sha256:"), "future materialization input")
end
abort "materialization custody controls weakened" unless
  recipe.fetch("fresh_private_temp_root_required") &&
    recipe.fetch("symlinked_input_or_output_denied") &&
    recipe.fetch("output_identity_must_match_payload_members") &&
    recipe.fetch("all_download_build_cache_and_output_paths_deleted_before_receipt") &&
    recipe.fetch("metadata_only_retained") &&
    recipe.fetch("zig_version") == "0.16.0" &&
    recipe.fetch("zig_target") == "aarch64-linux-musl" &&
    recipe.fetch("initramfs_builder") == "ruby-stdlib-canonical-cpio-gzip-mtime-1"

controls = contract.fetch("controls")
abort "source-free contract controls weakened" unless
  controls.fetch("contract_frozen") && controls.fetch("offline_validation") &&
    controls.fetch("repository_metadata_read") && FALSE_CLAIMS.all? { |claim| controls.fetch(claim) == false }

receipt = {
  "schema_name" => "macos-local-vm-synthetic-guest-payload-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "contract_id" => contract.fetch("contract_id"),
  "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
  "status" => "source_free_guest_payload_contract_exact",
  "payload_member_count" => "2",
  "payload_members_exact" => true,
  "excluded_component_count" => "4",
  "excluded_components_exact" => true,
  "controller_binding_exact" => true,
  "authenticated_future_input_frozen" => true,
  "build_recipe_frozen" => true,
  "source_free_validation" => true,
  "network_access" => false,
  "credential_access" => false,
  "process_launch" => false,
  "guest_payload_materialized" => false,
  "runnable_guest_artifacts_retained" => false,
  "release_candidate_complete" => false,
  "release_bundle_assembled" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "cask_created" => false,
  "bundle_installed" => false,
  "vm_launch" => false,
  "analyzer_execution" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "authority_added" => false
}
abort "guest payload receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-synthetic-guest-payload-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-synthetic-guest-payload-fixture-provenance.json"))
abort "fixture provenance acquired unsafe content" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" &&
    %w[contains_executable_artifacts contains_guest_payload_bytes contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].all? { |key| provenance.fetch(key) == false }
FIXTURE_DIGESTS.each { |relative, digest| exact("tests/conformance/v1/#{relative}", digest, "guest payload fixture") }
abort "fixture provenance inventory changed" unless
  provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] } == FIXTURE_DIGESTS

puts JSON.generate(receipt)
