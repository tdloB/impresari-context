#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-cask-lifecycle-v1.json"
PROFILE_DIGEST = "1373511a5ed419337df562bd66a9bfd57441bd58c5ae9f0d0d9333fc64fb5213"
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/cask-package-contract-v1.json"
CONTRACT_DIGEST = "4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676"
SEAL_RELATIVE = "platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
METADATA_SET_DIGEST = "ea29c43f36493f7e61935f33a64822805c8275d804c5384c3e8becea849fc54b"

EXPECTED_LAYOUT = {
  "Contents/Helpers/impresari-context-mcp" => ["local-stdio-mcp-server", false, "required-at-assembly"],
  "Contents/Helpers/impresari-context-structural-worker" => ["isolated-structural-worker", false, "required-at-assembly"],
  "Contents/Helpers/impresari-context-vm-controller" => ["local-vm-controller", false, "required-at-assembly"],
  "Contents/MacOS/impresari-context" => ["cli-supervisor-entrypoint", true, "required-at-assembly"],
  "Contents/Resources/macos-vm/guest" => ["closed-guest-payload-root", false, "required-at-assembly"],
  "Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json" => ["release-metadata-seal", false, "bound-by-contract"]
}.freeze

FIXTURE_DIGESTS = {
  "valid/macos-local-vm-cask-package-contract.json" => CONTRACT_DIGEST,
  "valid/iar-macos-local-vm-cask-lifecycle-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-cask-lifecycle-receipt.json" => "693a1d1016f905db023830f5573f8ef7d091e9591458107a2ca0a4b2e8b05bc3",
  "invalid/macos-local-vm-cask-lifecycle-overclaim.json" => "12f3ecfaa81625c706c58b7cc7ca5ac1f6428054668c42685b26f7b729491b06"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(path, digest, label)
  abort "missing #{label}: #{path}" unless path.file?
  abort "refusing symlinked #{label}: #{path}" if path.symlink?
  actual = Digest::SHA256.file(path).hexdigest
  abort "#{label} digest changed: #{path}" unless actual == digest
  path
end

profile_path = exact(ROOT.join(PROFILE_RELATIVE), PROFILE_DIGEST, "cask lifecycle profile")
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-cask-lifecycle-v1.sha256").read.strip
abort "cask lifecycle profile checksum record mismatch" unless
  sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-cask-lifecycle-v1.json"
abort "cask lifecycle profile fixture drifted" unless
  profile_path.binread == FIXTURE_ROOT.join("valid/iar-macos-local-vm-cask-lifecycle-profile.json").binread
profile = json(profile_path)

contract_path = exact(ROOT.join(CONTRACT_RELATIVE), CONTRACT_DIGEST, "cask package contract")
abort "cask package contract fixture drifted" unless
  contract_path.binread == FIXTURE_ROOT.join("valid/macos-local-vm-cask-package-contract.json").binread
contract = json(contract_path)
abort "profile does not bind the exact package contract" unless
  profile.fetch("package_contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("package_contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    profile.fetch("metadata_seal_digest") == "sha256:#{SEAL_DIGEST}"

seal_path = exact(ROOT.join(SEAL_RELATIVE), SEAL_DIGEST, "release-metadata seal")
seal = json(seal_path)
bindings = contract.fetch("release_bindings")
abort "package contract release-metadata binding changed" unless
  bindings.fetch("metadata_seal_path") == SEAL_RELATIVE &&
    bindings.fetch("metadata_seal_digest") == "sha256:#{SEAL_DIGEST}" &&
    bindings.fetch("metadata_set_digest") == "sha256:#{METADATA_SET_DIGEST}" &&
    bindings.fetch("metadata_set_digest") == seal.fetch("metadata_set_digest") &&
    bindings.fetch("guest_release_id") == seal.fetch("guest_release_id")
abort "package contract resolved release identities too early" unless
  bindings.fetch("source_revision") == "required-at-assembly" &&
    bindings.fetch("product_version") == "required-at-assembly" &&
    bindings.fetch("app_bundle_digest") == "required-after-assembly"

distribution = contract.fetch("distribution")
abort "package distribution shape changed" unless
  distribution == {
    "mechanism" => "homebrew-cask",
    "cask_token" => "impresari-context",
    "app_bundle" => "Impresari Context.app",
    "terminal_command" => "impresari-context",
    "architecture" => "arm64",
    "minimum_macos" => "unresolved-before-live-rehearsal"
  }

layout = contract.fetch("bundle_layout")
paths = layout.map { |entry| entry.fetch("path") }
abort "bundle layout is not closed and sorted" unless paths == EXPECTED_LAYOUT.keys.sort
abort "bundle layout contains duplicate destinations" unless paths.uniq == paths
paths.each do |relative|
  path = Pathname.new(relative)
  abort "bundle destination is not app-relative: #{relative}" if
    path.absolute? || path.cleanpath.to_s != relative || path.each_filename.include?("..") || !relative.start_with?("Contents/")
end
layout.each do |entry|
  expected = EXPECTED_LAYOUT.fetch(entry.fetch("path"))
  actual = [entry.fetch("role"), entry.fetch("public_command"), entry.fetch("release_identity")]
  abort "bundle role binding changed: #{entry.fetch('path')}" unless actual == expected
end
public_entries = layout.select { |entry| entry.fetch("public_command") }
abort "public CLI entrypoint changed" unless
  public_entries.length == 1 && public_entries.first.fetch("path") == "Contents/MacOS/impresari-context"

ownership = contract.fetch("cask_ownership")
abort "cask ownership set changed" unless
  ownership.fetch("installed_artifacts") == ["Impresari Context.app", "impresari-context-cli-link"] &&
    ownership.fetch("internal_helper_links") == "none"
abort "package scripts or broad removal were added" unless
  %w[postflight uninstall_script zap].none? { |key| ownership.fetch(key) }
abort "privileged or background service was added" unless
  %w[privileged_helper launch_daemon launch_agent login_item].none? { |key| ownership.fetch(key) }

lifecycle = contract.fetch("lifecycle")
abort "whole-bundle lifecycle changed" unless
  lifecycle == {
    "install" => "atomic-whole-bundle-and-cli-link",
    "upgrade" => "explicit-homebrew-whole-bundle-replacement",
    "rollback" => "explicit-previous-accepted-whole-bundle",
    "migration" => "reject-coexisting-formula-and-cask-before-mutation",
    "uninstall" => "remove-only-cask-owned-app-and-cli-link",
    "mixed_versions" => "rejected",
    "automatic_update" => false
  }

controls = contract.fetch("controls")
abort "contract evaluator authority changed" unless
  controls.fetch("contract_frozen") && controls.fetch("offline_validation") &&
    %w[network_access credential_access repository_source_access filesystem_mutation process_launch].none? { |key| controls.fetch(key) }
abort "package contract crossed an unproven gate" unless
  %w[app_bundle_assembled github_publication_attestation_verified developer_id_signature_verified apple_notarization_verified cask_lifecycle_verified sealed_distribution production_admitted macos_iar_1b_admitted analyzer_execution authority_added].none? { |key| controls.fetch(key) }

receipt = {
  "schema_name" => "macos-local-vm-cask-lifecycle-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "contract_id" => contract.fetch("contract_id"),
  "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
  "guest_release_id" => bindings.fetch("guest_release_id"),
  "metadata_set_digest" => bindings.fetch("metadata_set_digest"),
  "status" => "contract_frozen",
  "package_contract_exact" => true,
  "release_metadata_binding_exact" => true,
  "bundle_layout_closed" => true,
  "paths_app_relative" => true,
  "destinations_unique" => true,
  "public_cli_entrypoint_exact" => true,
  "cask_ownership_exact" => true,
  "whole_bundle_lifecycle_exact" => true,
  "formula_conflict_rejected" => true,
  "uninstall_scope_narrow" => true,
  "package_scripts_absent" => true,
  "privileged_services_absent" => true,
  "automatic_update_authority_absent" => true,
  "offline_validation" => true,
  "contract_frozen" => true,
  "app_bundle_assembled" => false,
  "github_publication_attestation_verified" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "cask_lifecycle_verified" => false,
  "sealed_distribution" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "analyzer_execution" => false,
  "authority_added" => false
}
fixture_receipt = json(FIXTURE_ROOT.join("valid/macos-local-vm-cask-lifecycle-receipt.json"))
abort "cask lifecycle receipt fixture drifted" unless receipt == fixture_receipt

provenance = json(FIXTURE_ROOT.join("macos-local-vm-cask-lifecycle-fixture-provenance.json"))
abort "cask fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_synthetic_and_project_metadata_only" &&
    %w[contains_executable_artifacts contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].none? { |key| provenance.fetch(key) }
recorded = provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "cask fixture provenance inventory changed" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(FIXTURE_ROOT.join(relative), digest, "cask lifecycle fixture") }

puts "macOS local-VM cask lifecycle contract frozen: contract=#{contract.fetch('contract_id')} assembled=false signed=false notarized=false"
