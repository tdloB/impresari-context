#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-ephemeral-unsigned-release-candidate-v1.json"
PROFILE_DIGEST = "0b9990f604eae2d3fec50d9e78d596aff822b7c702d7c80569e6d58059058942"
CANDIDATE_RELATIVE = "platform/macos-vm-feasibility/ephemeral-unsigned-release-candidate-v1.json"
CANDIDATE_DIGEST = "54257bb6a89f0930d7d80a87b15e302eac3756411fc719d100e9c2f5bfcfc362"
REHEARSAL_RELATIVE = "platform/macos-vm-feasibility/ephemeral-unsigned-release-candidate-rehearsal-v1.json"
REHEARSAL_DIGEST = "0df3cf194a4a97245b8523f598593ede1b9d80e18411a1356f7110bee7fb9b8b"
COMPOSITION_RELATIVE = "platform/macos-vm-feasibility/unsigned-release-candidate-composition-v1.json"
COMPOSITION_DIGEST = "88ee55c39b6735d645b285ca43ba203f4517f3d695aeaafc29d604c25eb6a167"
SCRIPT_RELATIVE = "scripts/rehearse-macos-vm-ephemeral-unsigned-release-candidate.rb"
SCRIPT_DIGEST = "5b86107ae92c59a21743e8405803f6159706bfa0b72d4c772267231b3ceaf6be"
CONTRACT_IDENTITY = "8d3da788a95c6cf638537218722e5fe32629710a10a3b25c0ac282280ed5720e"
MATERIAL_IDENTITY = "39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e"

FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-ephemeral-unsigned-release-candidate-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-ephemeral-unsigned-release-candidate.json" => "6f0814349c1e7a462cf1aeb73cf3cdc115df2df105ff0d15fc0441635c9404d7",
  "valid/macos-local-vm-ephemeral-unsigned-release-candidate-rehearsal.json" => REHEARSAL_DIGEST,
  "valid/macos-local-vm-ephemeral-unsigned-release-candidate-receipt.json" => "b8f66616bc9db9e1f2c74be209f003795ab530c320217163804ddbe7c2c503d1",
  "invalid/macos-local-vm-ephemeral-unsigned-release-candidate-overclaim.json" => "dd5f1e1aa6d15be066386d22b8fa9fbefb76d29120488b8eccd1379524a0d3d9"
}.freeze

FALSE_CLAIMS = %w[
  credential_access apple_identity_access developer_id_signature_verified
  apple_notarization_verified archive_created cask_created bundle_installed
  vm_launch analyzer_execution production_admitted macos_iar_1b_admitted
  authority_added
].freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(relative, digest, label)
  path = ROOT.join(relative).cleanpath
  abort "#{label} escapes repository" unless path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked #{label}: #{relative}" unless path.file? && !path.symlink?
  abort "#{label} digest changed: #{relative}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

profile = json(exact(PROFILE_RELATIVE, PROFILE_DIGEST, "ADR-0114 profile"))
candidate = json(exact(CANDIDATE_RELATIVE, CANDIDATE_DIGEST, "unsigned candidate record"))
rehearsal = json(exact(REHEARSAL_RELATIVE, REHEARSAL_DIGEST, "candidate rehearsal record"))
composition = json(exact(COMPOSITION_RELATIVE, COMPOSITION_DIGEST, "ADR-0113 composition"))
script = exact(SCRIPT_RELATIVE, SCRIPT_DIGEST, "ADR-0114 operator script").read

sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-ephemeral-unsigned-release-candidate-v1.sha256").read.strip
abort "profile checksum sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-ephemeral-unsigned-release-candidate-v1.json"
abort "profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-ephemeral-unsigned-release-candidate-profile.json"))
abort "candidate fixture drifted" unless candidate == json(FIXTURE_ROOT.join("valid/macos-local-vm-ephemeral-unsigned-release-candidate.json"))
abort "rehearsal fixture drifted" unless rehearsal == json(FIXTURE_ROOT.join("valid/macos-local-vm-ephemeral-unsigned-release-candidate-rehearsal.json"))

abort "profile bindings changed" unless
  profile.fetch("candidate_record_path") == CANDIDATE_RELATIVE && profile.fetch("candidate_record_digest") == "sha256:#{CANDIDATE_DIGEST}" &&
    profile.fetch("rehearsal_record_path") == REHEARSAL_RELATIVE && profile.fetch("rehearsal_record_digest") == "sha256:#{REHEARSAL_DIGEST}" &&
    profile.fetch("composition_record_path") == COMPOSITION_RELATIVE && profile.fetch("composition_record_digest") == "sha256:#{COMPOSITION_DIGEST}" &&
    profile.fetch("operator_script_path") == SCRIPT_RELATIVE && profile.fetch("operator_script_digest") == "sha256:#{SCRIPT_DIGEST}" &&
    profile.fetch("required_file_count") == "8" && profile.fetch("required_directory_count") == "6" &&
    profile.fetch("network_scope") == "exact-public-apk-only"
abort "operator script boundary changed" unless
  script.include?("abort \"usage: rehearse-macos-vm-ephemeral-unsigned-release-candidate.rb\" unless ARGV.empty?") &&
    script.include?("--offline") && script.include?("--max-redirs\", \"0\"") &&
    script.include?("FileUtils.remove_entry_secure(root)") && script.include?("produced_artifacts_executed\" => false")

abort "candidate source identity changed" unless
  candidate.fetch("candidate_source_revision") == "aca656771f9286b13fbcc046b133ade62b58da2a" &&
    candidate.fetch("candidate_source_archive_sha256") == "sha256:f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b" &&
    candidate.fetch("product_version") == "0.2.0" && candidate.fetch("target") == "aarch64-apple-darwin"
abort "candidate host identity incomplete" unless
  candidate.fetch("build_environment").fetch("target_triple") == "aarch64-apple-darwin" &&
    candidate.fetch("build_environment").values.all? { |value| value.is_a?(String) && !value.empty? }

projection = composition.fetch("material_projection")
projection_by_path = projection.to_h { |entry| [entry.fetch("path"), entry] }
artifacts = candidate.fetch("artifacts")
abort "product artifact inventory changed" unless artifacts.length == 4 && artifacts.map { |entry| entry.fetch("unit_id") } == artifacts.map { |entry| entry.fetch("unit_id") }.sort
artifacts.each do |artifact|
  member = projection_by_path.fetch(artifact.fetch("bundle_path"))
  abort "product artifact does not match assembled projection" unless
    artifact.fetch("bytes") == member.fetch("bytes") && artifact.fetch("sha256") == member.fetch("sha256") &&
      artifact.fetch("file_format") == "mach-o-64-arm64" && artifact.fetch("architectures") == ["arm64"] && artifact.fetch("unsigned")
end
guest = candidate.fetch("guest")
abort "guest evidence changed" unless
  guest.fetch("guest_release_id") == "iar-macos-local-vm-guest-2026-08-31.1" &&
    guest.fetch("guest_metadata_set_digest") == "sha256:ea29c43f36493f7e61935f33a64822805c8275d804c5384c3e8becea849fc54b" &&
    guest.fetch("guest_metadata_seal_digest") == "sha256:c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1" &&
    guest.fetch("payload_inventory_sha256") == "sha256:1e0097751227d9cd442c0fc8bfe2ef3c83973d1679a4da19eba92004a9d4de1f"

contract_rows = [["candidate-source-revision", candidate.fetch("candidate_source_revision")], ["product-version", candidate.fetch("product_version")]]
contract_rows.concat(projection.map { |entry| [entry.fetch("path"), entry.fetch("bytes"), entry.fetch("sha256")] })
computed_contract = Digest::SHA256.hexdigest(contract_rows.map { |row| row.join("\t") + "\n" }.join)
abort "contract compound identity changed" unless computed_contract == CONTRACT_IDENTITY && candidate.fetch("compound_identity") == "sha256:#{CONTRACT_IDENTITY}"
material_rows = [["candidate-source-revision", candidate.fetch("candidate_source_revision")], ["product-version", candidate.fetch("product_version")], ["target", candidate.fetch("target")]]
material_rows.concat(projection.map { |entry| [entry.fetch("path"), entry.fetch("kind"), entry.fetch("required_mode"), entry.fetch("bytes"), entry.fetch("sha256")] })
computed_material = Digest::SHA256.hexdigest(material_rows.map { |row| row.join("\t") + "\n" }.join)
abort "material projection identity changed" unless computed_material == MATERIAL_IDENTITY

candidate_controls = candidate.fetch("controls")
abort "candidate historical claims changed" unless candidate_controls.fetch("candidate_materialized") && candidate_controls.fetch("release_identity_bound")
abort "candidate overclaimed distribution or runtime" unless %w[developer_id_signature_verified apple_notarization_verified bundle_installed cask_created github_publication_attestation_verified cask_lifecycle_verified sealed_distribution vm_launch analyzer_execution production_admitted macos_iar_1b_admitted authority_added].all? { |key| candidate_controls.fetch(key) == false }

abort "rehearsal identity or closure changed" unless
  rehearsal.fetch("candidate_record_sha256") == "sha256:#{CANDIDATE_DIGEST}" &&
    rehearsal.fetch("composition_record_sha256") == "sha256:#{COMPOSITION_DIGEST}" &&
    rehearsal.fetch("contract_compound_identity") == "sha256:#{CONTRACT_IDENTITY}" &&
    rehearsal.fetch("material_projection_identity") == "sha256:#{MATERIAL_IDENTITY}" &&
    rehearsal.fetch("source_archive_verified") && rehearsal.fetch("publisher_signature_verified") && rehearsal.fetch("signed_datahash_verified") &&
    rehearsal.fetch("product_identities_exact") && rehearsal.fetch("guest_identities_exact") && rehearsal.fetch("app_tree_exact") &&
    rehearsal.fetch("filesystem_modes_exact") && rehearsal.fetch("simultaneous_component_custody") &&
    rehearsal.fetch("candidate_materialized") && rehearsal.fetch("app_assembled") && rehearsal.fetch("release_identity_bound") &&
    rehearsal.fetch("cleanup_verified") && !rehearsal.fetch("runnable_artifacts_retained") && !rehearsal.fetch("raw_build_logs_retained") &&
    !rehearsal.fetch("produced_artifacts_executed") && rehearsal.fetch("network_access") && rehearsal.fetch("network_scope") == "exact-public-apk-only"
abort "rehearsal overclaimed a later gate" unless FALSE_CLAIMS.all? { |key| rehearsal.fetch(key) == false }

receipt = {
  "schema_name" => "macos-local-vm-ephemeral-unsigned-release-candidate-receipt", "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"), "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "candidate_id" => candidate.fetch("candidate_id"), "candidate_record_digest" => "sha256:#{CANDIDATE_DIGEST}",
  "rehearsal_id" => rehearsal.fetch("rehearsal_id"), "rehearsal_record_digest" => "sha256:#{REHEARSAL_DIGEST}",
  "status" => "ephemeral_unsigned_candidate_assembled_verified_and_deleted",
  "source_archive_verified" => true, "publisher_authentication_verified" => true,
  "product_identities_exact" => true, "guest_identities_exact" => true,
  "app_tree_exact" => true, "filesystem_modes_exact" => true, "simultaneous_component_custody" => true,
  "contract_compound_identity" => "sha256:#{CONTRACT_IDENTITY}", "material_projection_identity" => "sha256:#{MATERIAL_IDENTITY}",
  "candidate_materialized" => true, "app_assembled" => true, "release_identity_bound" => true,
  "cleanup_verified" => true, "runnable_artifacts_retained" => false,
  "credential_access" => false, "apple_identity_access" => false,
  "developer_id_signature_verified" => false, "apple_notarization_verified" => false,
  "archive_created" => false, "cask_created" => false, "bundle_installed" => false,
  "vm_launch" => false, "analyzer_execution" => false, "production_admitted" => false,
  "macos_iar_1b_admitted" => false, "authority_added" => false
}
abort "receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-ephemeral-unsigned-release-candidate-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-ephemeral-unsigned-release-candidate-fixture-provenance.json"))
abort "fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" && provenance.fetch("network_or_provider_data_used") &&
    %w[contains_executable_artifacts contains_guest_payload_bytes contains_raw_build_logs contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source].all? { |key| provenance.fetch(key) == false }
abort "fixture provenance inventory changed" unless provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] } == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact("tests/conformance/v1/#{relative}", digest, "ADR-0114 fixture") }

puts JSON.generate(receipt)
