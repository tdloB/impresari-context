#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURES = ROOT.join("tests/conformance/v1")
PROFILE_PATH = ROOT.join("profiles/v1/iar-macos-historical-release-evidence-integrity-v1.json")
PROFILE_DIGEST = "2d68fd49dbd91b6a95cf9e41a1e386a54d51908217e3a005c6de5087cd3eaa98"
CONTRACT_DIGEST = "ebf78abf0a8b1609cf891b96f092065a3e957d4b819e221d47440bacc4f9cf9c"
OLD_PROFILE_DIGEST = "3bf687ea0c9acc5a2e381d64343a9704b97b6467f3f4016a81bfb091df886076"
PACKAGE_DIGEST = "4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
CANDIDATE_SCHEMA_DIGEST = "b1d0e93ce5917825018913017796ae1c9fbb84b824f28feb6fd37a38c04e2e41"
SOURCE_SET_DIGEST = "d5e98d46ba5294f147bdd44a5eb8fb307247472f69b9fed0b482f33feeef733e"
PROVENANCE_DIGEST = "5127312aef6813cbaccf5c62d5868a5adef4935493bd270ca61ce9b4fe7a59d9"

HISTORICAL_FIXTURES = {
  "valid/iar-macos-local-vm-release-identity-profile.json" => OLD_PROFILE_DIGEST,
  "valid/macos-local-vm-release-identity-receipt.json" => "ffa4f72092c1b18850c2a1386d2be180bce64fe273d1bf19363062ab0481b772",
  "invalid/macos-local-vm-release-identity-overclaim.json" => "07e29e2528497e39a48f770c2d2b0b726d87e9924dc8c9d066e0f181b75b0b39"
}.freeze

def parse(path)
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

def clean_path?(raw)
  path = Pathname.new(raw)
  !path.absolute? && path.cleanpath.to_s == raw && !path.each_filename.include?("..")
end

profile = parse(exact(PROFILE_PATH, PROFILE_DIGEST, "historical integrity profile"))
sidecar = PROFILE_PATH.sub_ext(".sha256").read.strip
abort "historical integrity profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  #{PROFILE_PATH.basename}"
abort "historical integrity profile fixture drifted" unless PROFILE_PATH.binread == FIXTURES.join("valid/iar-macos-historical-release-evidence-integrity-profile.json").binread

contract_path = exact(ROOT.join(profile.fetch("historical_contract_path")), CONTRACT_DIGEST, "historical release contract")
old_profile_path = exact(ROOT.join(profile.fetch("historical_profile_path")), OLD_PROFILE_DIGEST, "historical release profile")
provenance_path = exact(ROOT.join(profile.fetch("historical_fixture_provenance_path")), PROVENANCE_DIGEST, "historical fixture provenance")
contract = parse(contract_path)
old_profile = parse(old_profile_path)
package = parse(exact(ROOT.join("platform/macos-vm-feasibility/cask-package-contract-v1.json"), PACKAGE_DIGEST, "historical package contract"))
seal = parse(exact(ROOT.join("platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"), SEAL_DIGEST, "historical metadata seal"))
exact(ROOT.join("schemas/v1/macos-local-vm-unsigned-release-candidate.schema.json"), CANDIDATE_SCHEMA_DIGEST, "historical candidate schema")

abort "historical lineage changed" unless profile.fetch("lineage_id") == contract.fetch("contract_id")
abort "historical profile binding changed" unless old_profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}"
abort "historical package binding changed" unless contract.fetch("package_contract_id") == package.fetch("contract_id") && contract.fetch("package_contract_digest") == "sha256:#{PACKAGE_DIGEST}"
abort "historical guest binding changed" unless contract.fetch("guest_release_id") == seal.fetch("guest_release_id") && contract.fetch("guest_metadata_set_digest") == seal.fetch("metadata_set_digest") && contract.fetch("guest_metadata_seal_digest") == "sha256:#{SEAL_DIGEST}"

# Validate only the recorded inventory. Deliberately never open these paths in
# the current checkout: doing so would confuse metadata integrity with source reproduction.
inputs = contract.fetch("source_inputs")
paths = inputs.map { |entry| entry.fetch("path") }
abort "historical source inventory changed" unless paths.length == 15 && paths == paths.sort && paths.uniq == paths && paths.all? { |path| clean_path?(path) }
canonical = inputs.map { |entry| [entry.fetch("path"), entry.fetch("bytes"), entry.fetch("sha256")].join("\t") + "\n" }.join
abort "historical source-set identity changed" unless Digest::SHA256.hexdigest(canonical) == SOURCE_SET_DIGEST && contract.fetch("source_set_digest") == "sha256:#{SOURCE_SET_DIGEST}"

provenance = parse(provenance_path)
recorded = provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "historical provenance inventory changed" unless recorded == HISTORICAL_FIXTURES
HISTORICAL_FIXTURES.each { |relative, digest| exact(FIXTURES.join(relative), digest, "historical fixture") }

receipt = parse(FIXTURES.join("valid/macos-historical-release-evidence-integrity-receipt.json"))
abort "historical checker emitted a current/release claim" unless receipt.fetch("status") == "historical_not_current" && receipt.fetch("historical") && %w[historical_source_reproduced current_source_verified release_candidate_present release_identity_bound publication_authorized production_admitted independent_review_satisfied signing_verified notarization_verified network_access credential_access process_launch authority_added].none? { |key| receipt.fetch(key) }

puts JSON.generate(receipt)
