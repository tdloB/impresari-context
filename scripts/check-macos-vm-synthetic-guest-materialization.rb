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
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-synthetic-guest-materialization-v1.json"
PROFILE_DIGEST = "32a4f3ba1c719dc84207ecf0463e25313dea1f72df89f6dbacb3525c1b64c0a4"
RECORD_RELATIVE = "platform/macos-vm-feasibility/synthetic-guest-materialization-record-v1.json"
RECORD_DIGEST = "cb75f03b0934365f8cbc431403df52695595efc55d7592d418a777d0181b5998"
MATERIALIZER_RELATIVE = "scripts/materialize-macos-vm-synthetic-guest-candidate.rb"
MATERIALIZER_DIGEST = "6e3264d421d2866c444ae480f787eba82eb487bbe3313be3cdf5babaea9779c8"
EXPECTED_MEMBERS = {
  "Image" => ["36175872", "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5"],
  "impresari-initramfs.gz" => ["38207", "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b"]
}.freeze
FALSE_CLAIMS = %w[
  guest_payload_executed app_assembled apple_identity_access signed notarized
  cask_created bundle_installed vm_launch analyzer_execution
  release_identity_bound production_admitted macos_iar_1b_admitted
  authority_added
].freeze
FIXTURE_DIGESTS = {
  "valid/iar-macos-local-vm-synthetic-guest-materialization-profile.json" => "0f25d49153412882e3644b06e76f70695349761a098ccf78faae7520b1e4d99f",
  "valid/macos-local-vm-synthetic-guest-materialization-record.json" => "086463985889891b5bdf6b797b8891768a7b6a8f519c689336d55ae95dc18399",
  "valid/macos-local-vm-synthetic-guest-materialization-receipt.json" => "05f228149f41eeb9b919de6614bf114d43e3e76ec3617cd34d2977d5badb2a4f",
  "invalid/macos-local-vm-synthetic-guest-materialization-overclaim.json" => "0eb7913e5188dc3aa327b89f022aa54943f4cef113e031b6e44424ec48988892"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(relative, digest, label)
  path = ROOT.join(relative).cleanpath
  abort "#{label} escapes repository: #{relative}" unless path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked #{label}: #{relative}" unless path.file? && !path.symlink?
  abort "#{label} digest changed: #{relative}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

contract = json(exact(CONTRACT_RELATIVE, CONTRACT_DIGEST, "guest payload contract"))
profile = json(exact(PROFILE_RELATIVE, PROFILE_DIGEST, "materialization profile"))
record = json(exact(RECORD_RELATIVE, RECORD_DIGEST, "materialization record"))
materializer = exact(MATERIALIZER_RELATIVE, MATERIALIZER_DIGEST, "materializer").read

sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-synthetic-guest-materialization-v1.sha256").read.strip
abort "materialization profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-synthetic-guest-materialization-v1.json"
abort "profile fixture drifted" unless profile == json(FIXTURE_ROOT.join("valid/iar-macos-local-vm-synthetic-guest-materialization-profile.json"))
abort "record fixture drifted" unless record == json(FIXTURE_ROOT.join("valid/macos-local-vm-synthetic-guest-materialization-record.json"))
abort "profile binding changed" unless
  profile.fetch("contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    record.fetch("contract_id") == contract.fetch("contract_id") &&
    record.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    record.fetch("profile_id") == profile.fetch("profile_id") &&
    record.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}"

abort "rehearsal host or toolchain changed" unless
  record.fetch("host") == {"operating_system" => "macos", "architecture" => "arm64", "zig_version" => "0.16.0"}
public_input = record.fetch("public_input")
abort "authenticated public input changed" unless
  public_input.fetch("url") == "https://dl-cdn.alpinelinux.org/alpine/v3.24/main/aarch64/linux-virt-6.18.48-r0.apk" &&
    public_input.fetch("bytes") == "41557960" &&
    public_input.fetch("sha256") == "sha256:c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616" &&
    public_input.fetch("publisher_signature_verified") && public_input.fetch("signed_datahash_verified") &&
    public_input.fetch("package_name") == "linux-virt" && public_input.fetch("package_version") == "6.18.48-r0" &&
    public_input.fetch("package_architecture") == "aarch64" &&
    public_input.fetch("package_commit") == "c83b91e0fde4c1bada9b80d4e67c395b5335597b"

members = record.fetch("payload_members")
abort "materialized payload member order or count changed" unless members.map { |entry| entry.fetch("relative_path") } == EXPECTED_MEMBERS.keys
members.each do |member|
  bytes, digest = EXPECTED_MEMBERS.fetch(member.fetch("relative_path"))
  abort "materialized payload identity changed" unless
    member.fetch("mode") == "0644" && member.fetch("bytes") == bytes && member.fetch("sha256") == "sha256:#{digest}"
end

cleanup = record.fetch("cleanup")
abort "cleanup evidence weakened" unless
  cleanup.fetch("private_root_mode") == "0700" && cleanup.fetch("private_root_name_retained") == false &&
    %w[download_deleted extracted_inputs_deleted build_outputs_deleted compiler_caches_deleted metadata_only_retained].all? { |key| cleanup.fetch(key) == true } &&
    %w[raw_logs_retained runnable_guest_artifacts_retained].all? { |key| cleanup.fetch(key) == false }
controls = record.fetch("controls")
abort "materialization controls changed" unless
  controls.fetch("network_access") && controls.fetch("network_scope") == "exact-public-apk-only" &&
    controls.fetch("credential_access") == false && controls.fetch("compiler_process_launch") &&
    controls.fetch("guest_payload_materialized") && FALSE_CLAIMS.all? { |claim| controls.fetch(claim) == false }

required_source_fragments = [
  "ARGV.empty?", "Dir.mktmpdir", "File.chmod(0o700, root)", "--max-redirs", '"0"',
  "verify-alpine-apkv2.rb", "extract-macos-vm-kernel.rb", "build-macos-vm-initramfs.rb",
  "aarch64-linux-musl", "remove_entry_secure", "private materialization root was not deleted"
]
abort "materializer boundary changed" unless required_source_fragments.all? { |fragment| materializer.include?(fragment) }
abort "materializer acquired shell execution" if materializer.match?(/system\s*\(|IO\.popen|Open3\.capture3\([^\n]*sh\b/)

receipt = {
  "schema_name" => "macos-local-vm-synthetic-guest-materialization-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "record_id" => record.fetch("record_id"),
  "record_digest" => "sha256:#{RECORD_DIGEST}",
  "status" => "authenticated_synthetic_guest_materialized_and_deleted",
  "publisher_authentication_verified" => true,
  "payload_members_exact" => true,
  "cleanup_verified" => true,
  "runnable_guest_artifacts_retained" => false,
  "guest_payload_executed" => false,
  "app_assembled" => false,
  "apple_identity_access" => false,
  "signed" => false,
  "notarized" => false,
  "cask_created" => false,
  "bundle_installed" => false,
  "vm_launch" => false,
  "analyzer_execution" => false,
  "release_identity_bound" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "authority_added" => false
}
abort "materialization receipt fixture drifted" unless receipt == json(FIXTURE_ROOT.join("valid/macos-local-vm-synthetic-guest-materialization-receipt.json"))

provenance = json(FIXTURE_ROOT.join("macos-local-vm-synthetic-guest-materialization-fixture-provenance.json"))
abort "fixture provenance acquired unsafe content" unless
  provenance.fetch("review_status") == "approved_original_project_metadata_only" &&
    provenance.fetch("network_or_provider_data_used") == true &&
    %w[contains_executable_artifacts contains_guest_payload_bytes contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source].all? { |key| provenance.fetch(key) == false }
FIXTURE_DIGESTS.each { |relative, digest| exact("tests/conformance/v1/#{relative}", digest, "materialization fixture") }
abort "fixture provenance inventory changed" unless
  provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] } == FIXTURE_DIGESTS

puts JSON.generate(receipt)
