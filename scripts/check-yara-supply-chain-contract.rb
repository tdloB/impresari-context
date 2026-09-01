#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "572ccba51d019feb6c4a4d69e786e9d12094c35b3acfd647e43083f037388170"
EXPECTED_TAG_COMMIT = "84b0e3cc0e42f8f8e6b84d19c97ec3ac6ff8aee8"
FIXTURE_DIGESTS = {
  "invalid/yara-supply-chain-candidate-overclaim.json" => "4417656be1f399cecb1b00731559d6c2b6af095a5eda7c469c8f84e6a6dabe48",
  "invalid/yara-supply-chain-profile-tag-moved.json" => "140708329ef49169afd58e65d1006a0097c116e95a97f3ca2075261c2d2d29a9",
  "valid/yara-supply-chain-admission-profile.json" => PROFILE_DIGEST,
  "valid/yara-supply-chain-candidate-receipt.json" => "dde8853a061518f72b00c55fb54bcf48bbbb687519f010a636d9d10452d95b72"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid YARA supply-chain JSON: #{path}: #{e.message}"
end

def exact(path, digest)
  abort "missing or symlinked YARA supply-chain input: #{path}" unless path.file? && !path.symlink?
  abort "YARA supply-chain input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

def candidate_state(as_of:, expires_at:, observed_tag_commit:, revoked:, artifacts_complete:)
  return "revoked" if revoked
  return "source_tag_moved" unless observed_tag_commit == EXPECTED_TAG_COMMIT
  return "expired" if as_of >= expires_at
  return "contract_fixture_only" unless artifacts_complete

  "requires_separate_activation_review"
end

profile_path = ROOT.join("profiles/v1/yara-supply-chain-admission-v1.json")
exact(profile_path, PROFILE_DIGEST)
sidecar = ROOT.join("profiles/v1/yara-supply-chain-admission-v1.sha256").read.strip
abort "YARA supply-chain profile checksum record changed" unless
  sidecar == "#{PROFILE_DIGEST}  yara-supply-chain-admission-v1.json"

fixture_profile = ROOT.join("tests/conformance/v1/valid/yara-supply-chain-admission-profile.json")
abort "YARA supply-chain profile fixture drifted" unless profile_path.binread == fixture_profile.binread
profile = json(profile_path)

source = profile.fetch("source_candidate")
abort "YARA source selection changed" unless source == {
  "repository" => "https://github.com/VirusTotal/yara",
  "release_tag" => "v4.5.8",
  "release_url" => "https://github.com/VirusTotal/yara/releases/tag/v4.5.8",
  "tag_commit_sha1" => EXPECTED_TAG_COMMIT,
  "published_at" => "2026-07-28T07:12:15Z",
  "source_archive_api_url" => "https://api.github.com/repos/VirusTotal/yara/tarball/v4.5.8",
  "upstream_release_asset_count" => "0",
  "license_spdx" => "BSD-3-Clause",
  "license_path" => "COPYING",
  "license_git_blob_sha1" => "81b0eed4fe55ab6a33432b140b0f98e61085a5ea"
}

observed_at = Time.iso8601(profile.dig("freshness", "observed_at"))
expires_at = Time.iso8601(profile.dig("freshness", "expires_at"))
abort "YARA source-selection expiry is not exactly 30 days" unless expires_at - observed_at == 30 * 86_400

executable = profile.fetch("executable_admission")
abort "YARA upstream binary became accepted" unless executable.fetch("upstream_binary_accepted") == false
required_executable_controls = executable.reject { |key, _| key == "upstream_binary_accepted" }
abort "YARA executable admission lost a required evidence gate" unless required_executable_controls.values.all?

ruleset = profile.fetch("ruleset_admission")
allowed = ruleset.select { |key, _| key.end_with?("_allowed") }
abort "YARA ruleset gained repository, include, module, update, network, or credential authority" unless allowed.values.none?
required_ruleset_controls = ruleset.reject { |key, _| key.end_with?("_allowed") }
abort "YARA ruleset admission lost a required evidence gate" unless required_ruleset_controls.values.all?

controls = profile.fetch("checkpoint_controls")
abort "contract-only YARA checkpoint gained artifact, execution, network, credential, or claim authority" unless controls.values.none?

provenance_path = ROOT.join("tests/conformance/v1/yara-supply-chain-fixture-provenance.json")
provenance = json(provenance_path)
recorded = provenance.fetch("fixtures").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "YARA supply-chain fixture provenance is incomplete" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(ROOT.join("tests/conformance/v1", relative), digest) }
%w[yara_source_content yara_executable_content yara_rule_content malware_content repository_source_content credential_content network_capture_content authority_added].each do |key|
  abort "YARA supply-chain fixture provenance crossed #{key}" unless provenance.fetch(key) == false
end

receipt = json(ROOT.join("tests/conformance/v1/valid/yara-supply-chain-candidate-receipt.json"))
abort "YARA supply-chain receipt does not bind the profile" unless receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}"
abort "YARA supply-chain receipt admitted an absent artifact or authority" unless
  %w[source_downloaded executable_present executable_admitted ruleset_present ruleset_admitted analyzer_executed network_used credentials_used os_confined production_admitted iar_2_admitted safety_claimed ordinary_host_execution_authorized authority_added].none? { |key| receipt.fetch(key) }

state_vectors = [
  [observed_at, EXPECTED_TAG_COMMIT, false, false, "contract_fixture_only"],
  [expires_at, EXPECTED_TAG_COMMIT, false, false, "expired"],
  [observed_at, "0" * 40, false, false, "source_tag_moved"],
  [observed_at, EXPECTED_TAG_COMMIT, true, false, "revoked"],
  [observed_at, EXPECTED_TAG_COMMIT, false, true, "requires_separate_activation_review"]
]
state_vectors.each do |as_of, tag_commit, revoked, artifacts_complete, expected|
  actual = candidate_state(as_of: as_of, expires_at: expires_at, observed_tag_commit: tag_commit, revoked: revoked, artifacts_complete: artifacts_complete)
  abort "YARA supply-chain state precedence changed: expected #{expected}, got #{actual}" unless actual == expected
end

yara_artifacts = Dir.glob(ROOT.join("{platform,profiles,rules,crates}/**/*.{yar,yara}").to_s)
  .map { |path| Pathname.new(path).relative_path_from(ROOT).to_s }
  .sort
allowed_yara_x_artifacts = [
  "rules/yara-x/production-v1-candidate.yar",
  "rules/yara-x/synthetic-compatibility-v1.yar"
]
abort "unexpected YARA source rule artifact entered the repository" unless yara_artifacts == allowed_yara_x_artifacts

puts "YARA supply-chain contract verified: source=v4.5.8@#{EXPECTED_TAG_COMMIT} legacy_artifacts=absent yara_x_rule_sources=2 state=contract_fixture_only"
