#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/developer-id-notarization-preparation-v1.json"
CONTRACT_DIGEST = "a580be3e137e412c4a450c6377e4dbaba57abca49b68910e26ef4ad7690a86d1"
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-developer-id-notarization-preparation-v1.json"
PROFILE_DIGEST = "e02319909a0d4393c33833ac29d5e349edf2fe24fcd0d97ac5a327e3c442d850"
CANDIDATE_RELATIVE = "platform/macos-vm-feasibility/ephemeral-unsigned-release-candidate-v1.json"
CANDIDATE_DIGEST = "54257bb6a89f0930d7d80a87b15e302eac3756411fc719d100e9c2f5bfcfc362"

FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-developer-id-notarization-preparation-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-developer-id-notarization-preparation-receipt.json" => "2bba22f5bd72c1a7bc662c909288384d3525aa5f0fcb577ae7d05417d4858fef",
  "invalid/macos-local-vm-developer-id-notarization-preparation-overclaim.json" => "9284aa30365193304165bda939edc498054b29c4572f2af304c38629e473bc1e"
}.freeze

FALSE_CLAIMS = %w[
  network_access credential_access process_launch candidate_materialized
  developer_id_signature_verified apple_notarization_verified archive_created
  cask_created bundle_installed signed_or_notarized_artifact_retained vm_launch
  analyzer_execution production_admitted macos_iar_1b_admitted authority_added
].freeze

SIGNING_ORDER = [
  ["Contents/Helpers/impresari-context-mcp", "none"],
  ["Contents/Helpers/impresari-context-structural-worker", "none"],
  ["Contents/Helpers/impresari-context-vm-controller", "platform/macos-vm-feasibility/Resources/Controller.entitlements"],
  ["Contents/MacOS/impresari-context", "none"],
  ["Impresari Context.app", "none"]
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

contract_path = exact(CONTRACT_RELATIVE, CONTRACT_DIGEST, "ADR-0115 contract")
contract = json(contract_path)
profile = json(exact(PROFILE_RELATIVE, PROFILE_DIGEST, "ADR-0115 profile"))
candidate = json(exact(CANDIDATE_RELATIVE, CANDIDATE_DIGEST, "ADR-0114 candidate record"))

sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-developer-id-notarization-preparation-v1.sha256").read.strip
abort "profile checksum sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-developer-id-notarization-preparation-v1.json"
abort "profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-developer-id-notarization-preparation-profile.json"))
abort "profile bindings changed" unless
  profile.fetch("contract_path") == CONTRACT_RELATIVE && profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    profile.fetch("candidate_record_path") == CANDIDATE_RELATIVE && profile.fetch("candidate_record_digest") == "sha256:#{CANDIDATE_DIGEST}" &&
    profile.fetch("source_free_validation") && profile.fetch("manual_owner_action_required")

candidate_binding = contract.fetch("candidate")
abort "unsigned candidate binding changed" unless
  candidate_binding.fetch("record_path") == CANDIDATE_RELATIVE && candidate_binding.fetch("record_digest") == "sha256:#{CANDIDATE_DIGEST}" &&
    candidate_binding.fetch("candidate_id") == candidate.fetch("candidate_id") && candidate_binding.fetch("synthetic_identity_only") &&
    candidate_binding.fetch("material_projection_identity") == "sha256:39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e"

boundary = contract.fetch("credential_boundary")
abort "manual credential boundary changed" unless
  boundary.fetch("manual_owner_action_required") && boundary.fetch("signing_identity_class") == "Developer ID Application" &&
    boundary.fetch("signing_identity_reference") == "{developer_id_application_identity_from_keychain}" &&
    boundary.fetch("notary_credential_reference") == "{notarytool_keychain_profile_name}" &&
    boundary.fetch("credential_access_mode") == "reference-existing-keychain-items-in-place" &&
    %w[credential_values_in_arguments credential_values_in_environment credential_values_in_source credential_values_in_logs inspect_copy_print_export_or_delete_credentials].all? { |key| boundary.fetch(key) == false } &&
    boundary.fetch("keychain_creation_unlock_or_mutation") == "manual-outside-operator"

signing = contract.fetch("signing_plan")
abort "signing controls changed" unless signing.fetch("hardened_runtime") && signing.fetch("secure_timestamp") && !signing.fetch("deep_signing")
abort "inside-out signing order changed" unless signing.fetch("order").map { |entry| [entry.fetch("path"), entry.fetch("entitlements")] } == SIGNING_ORDER
abort "signing command changed" unless signing.fetch("nested_command") == ["codesign", "--force", "--sign", "{developer_id_application_identity_from_keychain}", "--options", "runtime", "--timestamp", "{optional_entitlements_pair}", "{code_path}"]
abort "signature verification changed" unless signing.fetch("verification_command") == ["codesign", "--verify", "--deep", "--strict", "--verbose=4", "Impresari Context.app"]

notary = contract.fetch("notarization_plan")
abort "notary submission changed" unless
  notary.fetch("submit_command") == ["xcrun", "notarytool", "submit", "Impresari-Context-0.2.0-macos-arm64.zip", "--keychain-profile", "{notarytool_keychain_profile_name}", "--wait", "--output-format", "json"] &&
    notary.fetch("required_submission_status") == "Accepted" && notary.fetch("submission_log_required_even_when_accepted") &&
    notary.fetch("error_severity_issue_count") == "0" && notary.fetch("warning_disposition_required") &&
    notary.fetch("final_archive_recreated_after_stapling")
abort "staple or Gatekeeper plan changed" unless
  notary.fetch("staple_command") == ["xcrun", "stapler", "staple", "Impresari Context.app"] &&
    notary.fetch("staple_validation_command") == ["xcrun", "stapler", "validate", "Impresari Context.app"] &&
    notary.fetch("gatekeeper_command") == ["spctl", "--assess", "--type", "execute", "--verbose=4", "Impresari Context.app"]

custody = contract.fetch("custody_and_cleanup")
abort "artifact custody or cleanup weakened" unless
  %w[fresh_private_root_required unsigned_input_rebuilt_in_same_root signed_app_never_launched archive_never_launched notarization_network_only_through_notarytool_and_stapler complete_private_root_deleted_before_receipt metadata_only_receipt].all? { |key| custody.fetch(key) } &&
    !custody.fetch("raw_notary_log_retained") && !custody.fetch("signed_or_notarized_artifact_retained")

abort "source-free contract overclaimed" unless contract.fetch("controls").fetch("contract_frozen") && contract.fetch("controls").fetch("source_free_validation") && FALSE_CLAIMS.all? { |key| contract.fetch("controls").fetch(key) == false }
abort "profile overclaimed" unless profile.fetch("claims").values_at("credential_boundary_frozen", "signing_plan_frozen", "notarization_plan_frozen") == [true, true, true] && FALSE_CLAIMS.all? { |key| profile.fetch("claims").fetch(key) == false }

receipt = {
  "schema_name" => "macos-local-vm-developer-id-notarization-preparation-receipt", "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"), "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "contract_id" => contract.fetch("contract_id"), "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
  "candidate_id" => candidate.fetch("candidate_id"), "candidate_record_digest" => "sha256:#{CANDIDATE_DIGEST}",
  "status" => "source_free_signing_notarization_contract_exact",
  "credential_boundary_exact" => true, "signing_plan_exact" => true, "notarization_plan_exact" => true,
  "cleanup_plan_exact" => true, "manual_owner_action_required" => true, "source_free_validation" => true,
  "network_access" => false, "credential_access" => false, "process_launch" => false,
  "candidate_materialized" => false, "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false, "archive_created" => false, "cask_created" => false,
  "bundle_installed" => false, "signed_or_notarized_artifact_retained" => false, "vm_launch" => false,
  "analyzer_execution" => false, "production_admitted" => false, "macos_iar_1b_admitted" => false,
  "authority_added" => false
}
abort "receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-developer-id-notarization-preparation-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-developer-id-notarization-preparation-fixture-provenance.json"))
abort "fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" &&
    %w[contains_executable_artifacts contains_guest_payload_bytes contains_apple_credentials_or_identity_subjects contains_private_or_customer_source network_or_provider_data_used].all? { |key| provenance.fetch(key) == false }
abort "fixture provenance inventory changed" unless provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] } == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact("tests/conformance/v1/#{relative}", digest, "ADR-0115 fixture") }

puts JSON.generate(receipt)
