#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-unsigned-candidate-composition-v1.json"
PROFILE_DIGEST = "70b44b73a8f1412cae5be5d94a944333ba1a80e65ac362da87b4cd2f391fb45a"
COMPOSITION_RELATIVE = "platform/macos-vm-feasibility/unsigned-release-candidate-composition-v1.json"
COMPOSITION_DIGEST = "88ee55c39b6735d645b285ca43ba203f4517f3d695aeaafc29d604c25eb6a167"
PROSPECTIVE_IDENTITY = "39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e"
SOURCE_DIGESTS = {
  "product-identities" => ["platform/macos-vm-feasibility/ephemeral-product-candidate-record-v1.json", "4b850177a1ab70eb2cb0330fa3c18ccae4fb00d29e7ba4f4c3b6b772c3ebea69"],
  "synthetic-tree-metadata" => ["platform/macos-vm-feasibility/unsigned-synthetic-bundle-assembly-v1.json", "36978dfd1f475d219ed7168d7f00c17fca1dcd5951e771e6dd81a5cfff7058d9"],
  "guest-payload-contract" => ["platform/macos-vm-feasibility/synthetic-guest-payload-contract-v1.json", "4e43e28f325d7ab67ff2bb23595eb9273320ff5e8597553b9a681bfdc51033d4"],
  "guest-materialization" => ["platform/macos-vm-feasibility/synthetic-guest-materialization-record-v1.json", "cb75f03b0934365f8cbc431403df52695595efc55d7592d418a777d0181b5998"],
  "guest-metadata-seal" => ["platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json", "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"]
}.freeze
EXPECTED_MEMBERS = {
  "Contents/Helpers/impresari-context-mcp" => ["0755", "4496400", "4324a95f4a6ceeb506f659bda8d8a6cb54cb00cbfa0248e81f6b98bb815e086c", "product-identities"],
  "Contents/Helpers/impresari-context-structural-worker" => ["0755", "35820544", "ab2efcae9c89c2a3cf8543c5be5cf6a63650e0ef689ec2be95df5b48aad103a7", "product-identities"],
  "Contents/Helpers/impresari-context-vm-controller" => ["0755", "274704", "48689796ad27aa4413a95d23ebb318d14c64a786cf0c5ab1b12553d5d656b7a5", "product-identities"],
  "Contents/Info.plist" => ["0644", "603", "e79b647082d33cdb39f003ae85d36f1ae39b1e2d3efc001f157eb7c2e8b6fc67", "synthetic-tree-metadata"],
  "Contents/MacOS/impresari-context" => ["0755", "8261920", "fa1992cd02678c03888a4a5f5a42849880dba42ef9e2b59153c5e66749499bd9", "product-identities"],
  "Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json" => ["0644", "5947", "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1", "guest-metadata-seal"],
  "Contents/Resources/macos-vm/guest/Image" => ["0644", "36175872", "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5", "guest-materialization"],
  "Contents/Resources/macos-vm/guest/impresari-initramfs.gz" => ["0644", "38207", "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b", "guest-materialization"]
}.freeze
FALSE_CLAIMS = %w[candidate_materialized app_assembled runnable_artifacts_retained apple_identity_access developer_id_signature_verified apple_notarization_verified cask_created bundle_installed vm_launch analyzer_execution production_admitted macos_iar_1b_admitted authority_added].freeze
FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-unsigned-candidate-composition-profile.json" => "a6f17b6492842bf9982031b38a064c2d54cb80e025c3a73d8ff15e875b6ad226",
  "valid/macos-local-vm-unsigned-candidate-composition.json" => COMPOSITION_DIGEST,
  "valid/macos-local-vm-unsigned-candidate-composition-receipt.json" => "d665973a8e94e936b059294d1cb7f05d7ba589541282743f5c312682cb0f0a61",
  "invalid/macos-local-vm-unsigned-candidate-composition-overclaim.json" => "cd369817eabfd5edcd786174d607650cb51b4e27f40c894ae47f199abc2661c4"
}.freeze

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

profile = json(exact(PROFILE_RELATIVE, PROFILE_DIGEST, "composition profile"))
composition = json(exact(COMPOSITION_RELATIVE, COMPOSITION_DIGEST, "composition record"))
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-unsigned-candidate-composition-v1.sha256").read.strip
abort "composition profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-unsigned-candidate-composition-v1.json"
abort "profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-unsigned-candidate-composition-profile.json"))
abort "composition fixture drifted" unless composition == json(FIXTURE_ROOT.join("valid/macos-local-vm-unsigned-candidate-composition.json"))
abort "profile composition binding changed" unless
  profile.fetch("composition_path") == COMPOSITION_RELATIVE && profile.fetch("composition_digest") == "sha256:#{COMPOSITION_DIGEST}" &&
    profile.fetch("required_source_records") == "5" && profile.fetch("required_material_members") == "8" &&
    profile.fetch("offline_only") && profile.fetch("metadata_only") && profile.fetch("claims").values.all? { |value| value == false }

release = composition.fetch("release_contract")
package = composition.fetch("package_contract")
exact(release.fetch("path"), release.fetch("sha256").delete_prefix("sha256:"), "release contract")
exact(package.fetch("path"), package.fetch("sha256").delete_prefix("sha256:"), "package contract")
abort "contract identifiers changed" unless
  release.fetch("id") == "iar-macos-local-vm-release-identity-2026-09-01.1" &&
    package.fetch("id") == "iar-macos-local-vm-cask-contract-2026-09-01.1"

sources = composition.fetch("evidence_sources").to_h { |entry| [entry.fetch("role"), [entry.fetch("path"), entry.fetch("sha256").delete_prefix("sha256:")]] }
abort "composition source inventory changed" unless sources == SOURCE_DIGESTS
SOURCE_DIGESTS.each { |role, (path, digest)| exact(path, digest, role) }

members = composition.fetch("material_projection")
abort "material projection is not closed and sorted" unless members.map { |entry| entry.fetch("path") } == EXPECTED_MEMBERS.keys
members.each do |member|
  mode, bytes, digest, role = EXPECTED_MEMBERS.fetch(member.fetch("path"))
  abort "material member identity changed" unless
    member.fetch("kind") == "file" && member.fetch("required_mode") == mode && member.fetch("bytes") == bytes &&
      member.fetch("sha256") == "sha256:#{digest}" && member.fetch("evidence_role") == role && member.fetch("co_materialized") == false
end
rows = [
  ["candidate-source-revision", composition.fetch("candidate_source_revision")],
  ["product-version", composition.fetch("product_version")],
  ["target", composition.fetch("target")]
]
rows.concat(members.map { |member| [member.fetch("path"), member.fetch("kind"), member.fetch("required_mode"), member.fetch("bytes"), member.fetch("sha256")] })
computed = Digest::SHA256.hexdigest(rows.map { |row| row.join("\t") + "\n" }.join)
abort "prospective compound identity changed" unless computed == PROSPECTIVE_IDENTITY && composition.fetch("prospective_compound_identity") == "sha256:#{PROSPECTIVE_IDENTITY}"
abort "source identity changed" unless
  composition.fetch("candidate_source_revision") == "aca656771f9286b13fbcc046b133ade62b58da2a" &&
    composition.fetch("candidate_source_archive_sha256") == "sha256:f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b" &&
    composition.fetch("product_version") == "0.2.0" && composition.fetch("target") == "aarch64-apple-darwin"
abort "unresolved gate was hidden" unless composition.fetch("unresolved_gates").values.all? { |value| value == false }
controls = composition.fetch("controls")
abort "source-free controls changed" unless controls.fetch("offline_validation") && controls.fetch("repository_metadata_read") && controls.fetch("network_access") == false && controls.fetch("credential_access") == false && controls.fetch("process_launch") == false && FALSE_CLAIMS.all? { |claim| controls.fetch(claim) == false }

receipt = {
  "schema_name" => "macos-local-vm-unsigned-candidate-composition-receipt", "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"), "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "composition_id" => composition.fetch("composition_id"), "composition_digest" => "sha256:#{COMPOSITION_DIGEST}",
  "status" => "source_free_candidate_projection_ready_for_complete_rehearsal",
  "source_bindings_exact" => true, "material_projection_exact" => true, "prospective_identity_exact" => true, "unresolved_gates_explicit" => true,
  "candidate_materialized" => false, "app_assembled" => false, "release_identity_bound" => false,
  "runnable_artifacts_retained" => false, "apple_identity_access" => false,
  "developer_id_signature_verified" => false, "apple_notarization_verified" => false,
  "cask_created" => false, "bundle_installed" => false, "vm_launch" => false,
  "analyzer_execution" => false, "production_admitted" => false, "macos_iar_1b_admitted" => false, "authority_added" => false
}
abort "composition receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-unsigned-candidate-composition-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-unsigned-candidate-composition-fixture-provenance.json"))
abort "fixture provenance boundary changed" unless provenance.fetch("review_status") == "approved_original_project_metadata_only" && %w[contains_executable_artifacts contains_guest_payload_bytes contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].all? { |key| provenance.fetch(key) == false }
FIXTURE_DIGESTS.each { |relative, digest| exact("tests/conformance/v1/#{relative}", digest, "composition fixture") }
abort "fixture provenance inventory changed" unless provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] } == FIXTURE_DIGESTS

puts JSON.generate(receipt)
