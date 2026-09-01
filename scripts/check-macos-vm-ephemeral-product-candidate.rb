#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-ephemeral-product-candidate-v1.json"
PROFILE_DIGEST = "df0536a8d19eacfc3175074b4a557643526a80353e3e31927a51effd9aea02c5"
RECORD_RELATIVE = "platform/macos-vm-feasibility/ephemeral-product-candidate-record-v1.json"
RECORD_DIGEST = "4b850177a1ab70eb2cb0330fa3c18ccae4fb00d29e7ba4f4c3b6b772c3ebea69"
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/release-identity-contract-v1.json"
CONTRACT_DIGEST = "ebf78abf0a8b1609cf891b96f092065a3e957d4b819e221d47440bacc4f9cf9c"
SOURCE_REVISION = "aca656771f9286b13fbcc046b133ade62b58da2a"
SOURCE_ARCHIVE_DIGEST = "f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b"
PRODUCT_IDENTITY = "7bd280339e2a8cf30c26fc2ad96225f52cad5593c63ea621e7e44ba62b9bd5ca"

EVIDENCE_DIGESTS = {
  "artifacts/sbom.spdx.json" => "bb249501b6d693edaff188edc2344d1d1a62a94bd13ace8488f4a03e5273a3bb",
  "platform/macos-vm-feasibility/product-license-disposition-v1.json" => "6f7183c6b0c46d7121c371df536f810677ed843a9f282d454859c3ab04a4c219",
  "platform/macos-vm-feasibility/product-vulnerability-disposition-v1.json" => "73a56792d4a09d3cf12329e3d46f289ace496eaab42c391c57689726197daea1",
  "platform/macos-vm-feasibility/product-reproducibility-disposition-v1.json" => "b74119c2acebfdc919c7852cee904016483f93113471bae3a23fd5f56135b59b"
}.freeze

FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-ephemeral-product-candidate-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-ephemeral-product-candidate-record.json" => "01f63f748a3b0ad4fd34bcf4556d6d22ee79ac3655f7f330e89fa1c08815ffc0",
  "valid/macos-local-vm-ephemeral-product-candidate-receipt.json" => "b696f0b120480650d816eeb08e0895bb91cf483d68ecd1d3aad0c23410ce38bd",
  "invalid/macos-local-vm-ephemeral-product-candidate-overclaim.json" => "1897011cd1a1959ef351f02b46ad181aa7766a398fb01b589ed2fb4bb856e8d9"
}.freeze

EXPECTED_ARTIFACTS = {
  "cli-supervisor-entrypoint" => ["Contents/MacOS/impresari-context", "8261920", "fa1992cd02678c03888a4a5f5a42849880dba42ef9e2b59153c5e66749499bd9"],
  "isolated-structural-worker" => ["Contents/Helpers/impresari-context-structural-worker", "35820544", "ab2efcae9c89c2a3cf8543c5be5cf6a63650e0ef689ec2be95df5b48aad103a7"],
  "local-stdio-mcp-server" => ["Contents/Helpers/impresari-context-mcp", "4496400", "4324a95f4a6ceeb506f659bda8d8a6cb54cb00cbfa0248e81f6b98bb815e086c"],
  "local-vm-controller" => ["Contents/Helpers/impresari-context-vm-controller", "274704", "48689796ad27aa4413a95d23ebb318d14c64a786cf0c5ab1b12553d5d656b7a5"]
}.freeze

FALSE_CLAIMS = %w[
  workspace_scanned network_access credential_access product_candidates_executed
  guest_materialized release_bundle_assembled archive_created
  developer_id_signature_verified apple_notarization_verified bundle_installed
  cask_created github_publication_attestation_verified cask_lifecycle_verified
  sealed_distribution vm_launch analyzer_execution release_identity_bound
  production_admitted macos_iar_1b_admitted authority_added
].freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(path, digest, label)
  abort "missing #{label}: #{path}" unless path.file?
  abort "refusing symlinked #{label}: #{path}" if path.symlink?
  abort "#{label} digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

profile_path = exact(ROOT.join(PROFILE_RELATIVE), PROFILE_DIGEST, "ephemeral product profile")
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-ephemeral-product-candidate-v1.sha256").read.strip
abort "ephemeral product profile checksum record mismatch" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-ephemeral-product-candidate-v1.json"
profile = json(profile_path)
abort "ephemeral product profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-ephemeral-product-candidate-profile.json"))

record_path = exact(ROOT.join(RECORD_RELATIVE), RECORD_DIGEST, "ephemeral product record")
record = json(record_path)
abort "ephemeral product record fixture drifted" unless record == json(FIXTURE_ROOT.join("valid/macos-local-vm-ephemeral-product-candidate-record.json"))
contract = json(exact(ROOT.join(CONTRACT_RELATIVE), CONTRACT_DIGEST, "release identity contract"))

abort "profile record binding changed" unless
  profile.fetch("candidate_record_path") == RECORD_RELATIVE &&
    profile.fetch("candidate_record_digest") == "sha256:#{RECORD_DIGEST}" &&
    profile.fetch("release_identity_contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("release_identity_contract_digest") == "sha256:#{CONTRACT_DIGEST}"
abort "candidate source identity changed" unless
  record.fetch("candidate_source_revision") == SOURCE_REVISION &&
    profile.fetch("candidate_source_revision") == SOURCE_REVISION &&
    record.fetch("candidate_source_archive_sha256") == "sha256:#{SOURCE_ARCHIVE_DIGEST}" &&
    profile.fetch("candidate_source_archive_sha256") == "sha256:#{SOURCE_ARCHIVE_DIGEST}" &&
    record.fetch("product_version") == contract.fetch("product_version") &&
    record.fetch("target") == contract.fetch("rust_target") &&
    record.fetch("source_date_epoch") == "1788243888"

environment = record.fetch("build_environment")
abort "candidate host identity is incomplete" unless
  environment.fetch("host_architecture") == "arm64" &&
    environment.fetch("target_triple") == "aarch64-apple-darwin" &&
    %w[macos_product_version macos_build_version xcode_version xcode_build_version apple_sdk_version apple_sdk_build_version swift_version swift_target rustc_version rustc_commit llvm_version cargo_version].all? { |key| !environment.fetch(key).empty? }

build_controls = record.fetch("build_controls")
abort "candidate build controls weakened" unless
  build_controls.fetch("independent_private_temp_roots") == "2" &&
    !build_controls.fetch("root_names_retained") &&
    build_controls.fetch("cargo_locked") && build_controls.fetch("cargo_offline") &&
    !build_controls.fetch("cargo_incremental") && build_controls.fetch("swift_module_cache_private") &&
    build_controls.fetch("locale") == "C" && build_controls.fetch("source_date_epoch_from_candidate_commit")

evidence = record.fetch("dependency_evidence")
EVIDENCE_DIGESTS.each do |relative, digest|
  exact(ROOT.join(relative), digest, "candidate product evidence")
end
abort "candidate evidence bindings changed" unless
  evidence.fetch("cargo_lock_sha256") == "sha256:d04f92d689b5d92fba1b49442258b9db82c0f141f07bc3b45dd04b6883278add" &&
    evidence.fetch("cargo_metadata_stdout_sha256") == "sha256:5a63c27b8e0eba2cbcfc842adca388118e725a0aea8883d11881e6c2f08ba44c" &&
    evidence.fetch("spdx_2_3_sbom_path") == "artifacts/sbom.spdx.json" &&
    evidence.fetch("spdx_2_3_sbom_sha256") == "sha256:#{EVIDENCE_DIGESTS.fetch('artifacts/sbom.spdx.json')}" &&
    evidence.fetch("license_disposition_sha256") == "sha256:#{EVIDENCE_DIGESTS.fetch('platform/macos-vm-feasibility/product-license-disposition-v1.json')}" &&
    evidence.fetch("vulnerability_disposition_sha256") == "sha256:#{EVIDENCE_DIGESTS.fetch('platform/macos-vm-feasibility/product-vulnerability-disposition-v1.json')}" &&
    evidence.fetch("reproducibility_disposition_sha256") == "sha256:#{EVIDENCE_DIGESTS.fetch('platform/macos-vm-feasibility/product-reproducibility-disposition-v1.json')}"

license = json(ROOT.join("platform/macos-vm-feasibility/product-license-disposition-v1.json"))
abort "license disposition overclaimed" unless license.fetch("status") == "pass_with_unmatched_allowance_warnings" && !license.fetch("network_access") && !license.fetch("production_approved")
vulnerability = json(ROOT.join("platform/macos-vm-feasibility/product-vulnerability-disposition-v1.json"))
abort "vulnerability disposition overclaimed" unless
  vulnerability.fetch("status") == "no_known_advisory_match_in_recorded_database" &&
    vulnerability.fetch("vulnerabilities_found") == "0" && !vulnerability.fetch("network_access") &&
    !vulnerability.fetch("vulnerability_free_claimed") && !vulnerability.fetch("production_approved")
reproducibility = json(ROOT.join("platform/macos-vm-feasibility/product-reproducibility-disposition-v1.json"))
abort "reproducibility disposition overclaimed" unless
  reproducibility.fetch("same_host_reproducibility_observed") &&
    !reproducibility.fetch("cross_run_reproducibility_established") &&
    !reproducibility.fetch("cross_host_reproducibility_established") &&
    !reproducibility.fetch("production_reproducibility_established") &&
    !reproducibility.fetch("runnable_artifacts_retained") && !reproducibility.fetch("production_approved")

artifacts = record.fetch("artifacts")
abort "candidate artifact count changed" unless artifacts.length == 4
abort "candidate artifact order or identity changed" unless artifacts.map { |entry| entry.fetch("unit_id") } == EXPECTED_ARTIFACTS.keys
artifacts.each do |artifact|
  path, bytes, digest = EXPECTED_ARTIFACTS.fetch(artifact.fetch("unit_id"))
  abort "candidate artifact binding changed" unless
    artifact.fetch("bundle_path") == path && artifact.fetch("bytes") == bytes &&
      artifact.fetch("build_a_sha256") == "sha256:#{digest}" &&
      artifact.fetch("build_b_sha256") == "sha256:#{digest}" && artifact.fetch("byte_identical") &&
      artifact.fetch("file_format") == "mach-o-64-arm64" && artifact.fetch("architectures") == ["arm64"] &&
      artifact.fetch("signature_kind") == "linker-adhoc" && !artifact.fetch("team_identifier_present") &&
      !artifact.fetch("developer_id_signed")
  libraries = artifact.fetch("dynamic_libraries")
  abort "candidate dynamic-library inventory is unsafe or nondeterministic" unless
    libraries == libraries.sort && libraries.uniq == libraries && libraries.all? { |library| library.start_with?("/") }
end

canonical = [
  ["candidate-source-revision", SOURCE_REVISION],
  ["product-version", record.fetch("product_version")],
  ["target", record.fetch("target")]
]
canonical.concat(artifacts.map do |artifact|
  [artifact.fetch("unit_id"), artifact.fetch("bundle_path"), artifact.fetch("bytes"), artifact.fetch("build_a_sha256")]
end)
computed_identity = Digest::SHA256.hexdigest(canonical.map { |row| row.join("\t") + "\n" }.join)
abort "canonical product identity changed" unless computed_identity == PRODUCT_IDENTITY && record.fetch("canonical_product_identity") == "sha256:#{PRODUCT_IDENTITY}"

cleanup = record.fetch("cleanup")
abort "candidate cleanup was not closed" unless
  cleanup.fetch("superseded_build_root_count") == "2" && cleanup.fetch("accepted_build_root_count") == "2" &&
    cleanup.fetch("all_build_roots_deleted") && !cleanup.fetch("runnable_artifacts_retained") &&
    !cleanup.fetch("raw_build_logs_retained") && cleanup.fetch("metadata_only_retained")
controls = record.fetch("controls")
abort "candidate rehearsal did not record compiler launches" unless controls.fetch("repository_source_read") && controls.fetch("compiler_process_launch") && controls.fetch("product_candidates_materialized")
abort "candidate rehearsal overclaimed a later gate" unless FALSE_CLAIMS.all? { |key| !controls.fetch(key) }

receipt = {
  "schema_name" => "macos-local-vm-ephemeral-product-candidate-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "record_id" => record.fetch("record_id"),
  "record_digest" => "sha256:#{RECORD_DIGEST}",
  "candidate_source_revision" => SOURCE_REVISION,
  "candidate_source_archive_sha256" => "sha256:#{SOURCE_ARCHIVE_DIGEST}",
  "product_version" => record.fetch("product_version"),
  "target" => record.fetch("target"),
  "status" => "product_candidates_built_and_deleted",
  "independent_builds" => "2",
  "product_artifacts_per_build" => "4",
  "byte_identical_artifacts" => "4",
  "canonical_product_identity" => "sha256:#{PRODUCT_IDENTITY}",
  "dependency_evidence_complete" => true,
  "all_build_roots_deleted" => true,
  "runnable_artifacts_retained" => false,
  "product_candidates_executed" => false,
  "guest_candidate_complete" => false,
  "release_candidate_complete" => false,
  "release_bundle_assembled" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "bundle_installed" => false,
  "cask_created" => false,
  "vm_launch" => false,
  "analyzer_execution" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "authority_added" => false
}
abort "ephemeral product receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-ephemeral-product-candidate-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-ephemeral-product-candidate-fixture-provenance.json"))
abort "ephemeral product fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" &&
    %w[contains_executable_artifacts contains_raw_build_logs contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].none? { |key| provenance.fetch(key) }
recorded = provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "ephemeral product fixture provenance inventory changed" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(FIXTURE_ROOT.join(relative), digest, "ephemeral product fixture") }

puts "macOS ephemeral product candidate verified: builds=2 artifacts=4 identical=4 retained=false release=false"
