#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-release-identity-v1.json"
PROFILE_DIGEST = "3bf687ea0c9acc5a2e381d64343a9704b97b6467f3f4016a81bfb091df886076"
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/release-identity-contract-v1.json"
CONTRACT_DIGEST = "ebf78abf0a8b1609cf891b96f092065a3e957d4b819e221d47440bacc4f9cf9c"
PACKAGE_RELATIVE = "platform/macos-vm-feasibility/cask-package-contract-v1.json"
PACKAGE_DIGEST = "4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676"
SEAL_RELATIVE = "platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
CANDIDATE_SCHEMA_RELATIVE = "schemas/v1/macos-local-vm-unsigned-release-candidate.schema.json"
CANDIDATE_SCHEMA_DIGEST = "b1d0e93ce5917825018913017796ae1c9fbb84b824f28feb6fd37a38c04e2e41"
SOURCE_SET_DIGEST = "d5e98d46ba5294f147bdd44a5eb8fb307247472f69b9fed0b482f33feeef733e"

FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-release-identity-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-release-identity-receipt.json" => "ffa4f72092c1b18850c2a1386d2be180bce64fe273d1bf19363062ab0481b772",
  "invalid/macos-local-vm-release-identity-overclaim.json" => "07e29e2528497e39a48f770c2d2b0b726d87e9924dc8c9d066e0f181b75b0b39"
}.freeze

EXPECTED_BUILD_UNITS = {
  "cli-supervisor-entrypoint" => "Contents/MacOS/impresari-context",
  "local-stdio-mcp-server" => "Contents/Helpers/impresari-context-mcp",
  "isolated-structural-worker" => "Contents/Helpers/impresari-context-structural-worker",
  "local-vm-controller" => "Contents/Helpers/impresari-context-vm-controller",
  "closed-guest-payload-root" => "Contents/Resources/macos-vm/guest"
}.freeze

FALSE_CLAIMS = %w[
  workspace_scanned network_access credential_access process_launch
  candidate_materialized release_bundle_assembled archive_created
  bundle_installed cask_created release_identity_bound
  developer_id_signature_verified apple_notarization_verified
  github_publication_attestation_verified cask_lifecycle_verified
  sealed_distribution vm_launch analyzer_execution production_admitted
  macos_iar_1b_admitted authority_added
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

def clean_project_path(path)
  value = Pathname.new(path)
  !value.absolute? && value.cleanpath.to_s == path && !value.each_filename.include?("..")
end

profile_path = exact(ROOT.join(PROFILE_RELATIVE), PROFILE_DIGEST, "release-identity profile")
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-release-identity-v1.sha256").read.strip
abort "release-identity profile checksum record mismatch" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-release-identity-v1.json"
profile = json(profile_path)
abort "release-identity profile fixture drifted" unless profile_path.binread == FIXTURE_ROOT.join("valid/iar-macos-local-vm-release-identity-profile.json").binread

contract_path = exact(ROOT.join(CONTRACT_RELATIVE), CONTRACT_DIGEST, "release-identity contract")
contract = json(contract_path)
package = json(exact(ROOT.join(PACKAGE_RELATIVE), PACKAGE_DIGEST, "cask package contract"))
seal = json(exact(ROOT.join(SEAL_RELATIVE), SEAL_DIGEST, "guest metadata seal"))
exact(ROOT.join(CANDIDATE_SCHEMA_RELATIVE), CANDIDATE_SCHEMA_DIGEST, "unsigned candidate schema")

abort "profile bindings changed" unless
  profile.fetch("contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    profile.fetch("package_contract_path") == PACKAGE_RELATIVE &&
    profile.fetch("package_contract_digest") == "sha256:#{PACKAGE_DIGEST}" &&
    profile.fetch("metadata_seal_path") == SEAL_RELATIVE &&
    profile.fetch("metadata_seal_digest") == "sha256:#{SEAL_DIGEST}" &&
    profile.fetch("candidate_schema_path") == CANDIDATE_SCHEMA_RELATIVE &&
    profile.fetch("candidate_schema_digest") == "sha256:#{CANDIDATE_SCHEMA_DIGEST}" &&
    profile.fetch("required_build_units") == "5" &&
    profile.fetch("required_source_inputs") == "15" &&
    profile.fetch("required_product_artifacts") == "4" &&
    profile.fetch("required_guest_payload_sets") == "1" &&
    profile.fetch("offline_only") && !profile.fetch("candidate_materialized")

abort "release identity package binding changed" unless
  contract.fetch("package_contract_id") == package.fetch("contract_id") &&
    contract.fetch("package_contract_digest") == "sha256:#{PACKAGE_DIGEST}"
abort "release identity guest binding changed" unless
  contract.fetch("guest_release_id") == seal.fetch("guest_release_id") &&
    contract.fetch("guest_metadata_set_digest") == seal.fetch("metadata_set_digest") &&
    contract.fetch("guest_metadata_seal_digest") == "sha256:#{SEAL_DIGEST}"

inputs = contract.fetch("source_inputs")
paths = inputs.map { |entry| entry.fetch("path") }
abort "source input inventory is not closed, sorted, and unique" unless paths == paths.sort && paths.uniq == paths && paths.length == 15
abort "source input path escaped project" unless paths.all? { |path| clean_project_path(path) }
inputs.each do |entry|
  path = exact(ROOT.join(entry.fetch("path")), entry.fetch("sha256").delete_prefix("sha256:"), "release build input")
  abort "release build input byte count changed: #{entry.fetch('path')}" unless path.size.to_s == entry.fetch("bytes")
end
canonical_sources = inputs.map do |entry|
  [entry.fetch("path"), entry.fetch("bytes"), entry.fetch("sha256")].join("\t") + "\n"
end.join
abort "source input set digest changed" unless Digest::SHA256.hexdigest(canonical_sources) == SOURCE_SET_DIGEST
abort "contract source set binding changed" unless contract.fetch("source_set_digest") == "sha256:#{SOURCE_SET_DIGEST}"

cargo_workspace = ROOT.join("Cargo.toml").read
toolchain = ROOT.join("rust-toolchain.toml").read
abort "workspace product version changed" unless cargo_workspace.include?("version = \"#{contract.fetch('product_version')}\"")
abort "Rust toolchain changed" unless toolchain.include?("channel = \"#{contract.fetch('rust_toolchain')}\"")
abort "Rust release target changed" unless contract.fetch("rust_target") == "aarch64-apple-darwin"

units = contract.fetch("build_units")
unit_map = units.to_h { |unit| [unit.fetch("unit_id"), unit.fetch("bundle_path")] }
abort "release build units changed" unless unit_map == EXPECTED_BUILD_UNITS
abort "release build unit identifiers repeat" unless units.map { |unit| unit.fetch("unit_id") }.uniq.length == units.length
abort "release build outputs escape target" unless units.all? { |unit| unit.fetch("unsigned_output").start_with?("target/") && clean_project_path(unit.fetch("unsigned_output")) }

package_roles = package.fetch("bundle_layout").reject do |entry|
  entry.fetch("role") == "release-metadata-seal"
end.to_h { |entry| [entry.fetch("role"), entry.fetch("path")] }
abort "build units do not exactly cover cask material roles" unless unit_map == package_roles

rust_units = units.select { |unit| unit.fetch("build_system") == "cargo" }
abort "Rust build-unit count changed" unless rust_units.length == 3
abort "Rust commands are not locked release arm64 Apple builds" unless rust_units.all? do |unit|
  command = unit.fetch("command")
  command.first == "cargo" && command.include?("--locked") && command.include?("--release") &&
    command.each_cons(2).any? { |left, right| left == "--target" && right == "aarch64-apple-darwin" }
end
controller = units.fetch(units.index { |unit| unit.fetch("unit_id") == "local-vm-controller" })
abort "Swift controller build boundary changed" unless
  controller.fetch("build_system") == "xcrun-swiftc" &&
    controller.fetch("command").first(2) == %w[xcrun swiftc] &&
    controller.fetch("command").include?("Virtualization") &&
    !controller.fetch("command").include?("codesign")
guest = units.fetch(units.index { |unit| unit.fetch("unit_id") == "closed-guest-payload-root" })
abort "guest candidate was made buildable by the contract" unless
  guest.fetch("build_system") == "separate-authenticated-guest-candidate" &&
    guest.fetch("command") == ["not-authorized-by-adr-0109"] &&
    guest.fetch("candidate_identity") == "metadata-bound-but-unmaterialized"

requirements = contract.fetch("candidate_record_requirements")
abort "future build environment requirements changed" unless requirements.fetch("build_environment").sort == %w[
  apple_sdk_version cargo_version macos_build_version macos_product_version
  rustc_verbose_version swift_version target_triple xcode_build_version xcode_version
].sort
abort "future artifact identity requirements changed" unless requirements.fetch("per_artifact").sort == %w[
  architectures build_log_sha256 bundle_path bytes file_format sha256 unit_id unsigned
].sort
abort "future product evidence requirements changed" unless requirements.fetch("product_evidence").sort == %w[
  license_inventory_sha256 reproducibility_disposition_sha256 spdx_2_3_sbom_sha256
  vulnerability_assessment_sha256
].sort
abort "future candidate evidence became optional" unless requirements.fetch("all_fields_required_before_substitution")

rollback = contract.fetch("rollback")
abort "release rollback boundary changed" unless
  rollback.fetch("application_predecessor") == "none-first-cask-release" &&
    rollback.fetch("guest_predecessor_release_id") == seal.fetch("cross_bindings").fetch("rollback_predecessor_release_id") &&
    rollback.fetch("whole_bundle_only") && !rollback.fetch("mixed_version_rollback")

controls = contract.fetch("controls")
abort "release identity contract is not frozen offline metadata" unless
  controls.fetch("contract_frozen") && controls.fetch("offline_validation") &&
    controls.fetch("repository_build_input_metadata_read")
abort "release identity contract overclaimed a later gate" unless FALSE_CLAIMS.all? { |key| !controls.fetch(key) }

receipt = {
  "schema_name" => "macos-local-vm-release-identity-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "contract_id" => contract.fetch("contract_id"),
  "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
  "contract_baseline_revision" => contract.fetch("contract_baseline_revision"),
  "product_version" => contract.fetch("product_version"),
  "source_set_digest" => contract.fetch("source_set_digest"),
  "package_contract_digest" => contract.fetch("package_contract_digest"),
  "guest_release_id" => contract.fetch("guest_release_id"),
  "guest_metadata_set_digest" => contract.fetch("guest_metadata_set_digest"),
  "status" => "release_identity_contract_frozen",
  "source_inputs_exact" => true,
  "build_units_closed" => true,
  "bundle_roles_exact" => true,
  "candidate_schema_frozen" => true,
  "future_candidate_evidence_required" => true,
  "rollback_closed" => true,
  "contract_frozen" => true,
  "candidate_materialized" => false,
  "release_bundle_assembled" => false,
  "archive_created" => false,
  "bundle_installed" => false,
  "cask_created" => false,
  "release_identity_bound" => false,
  "network_access" => false,
  "credential_access" => false,
  "process_launch" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "github_publication_attestation_verified" => false,
  "cask_lifecycle_verified" => false,
  "sealed_distribution" => false,
  "vm_launch" => false,
  "analyzer_execution" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "authority_added" => false
}
fixture_receipt = json(FIXTURE_ROOT.join("valid/macos-local-vm-release-identity-receipt.json"))
abort "release identity receipt fixture drifted" unless receipt == fixture_receipt

provenance = json(FIXTURE_ROOT.join("macos-local-vm-release-identity-fixture-provenance.json"))
abort "release identity fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" &&
    %w[contains_executable_artifacts contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].none? { |key| provenance.fetch(key) }
recorded = provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "release identity fixture provenance inventory changed" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(FIXTURE_ROOT.join(relative), digest, "release identity fixture") }

puts "macOS local-VM release identity contract frozen: source_inputs=15 build_units=5 candidate=false signed=false"
