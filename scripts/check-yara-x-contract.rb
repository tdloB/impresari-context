#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "827c4e1f99e5175369fc4347bbc20c21a63b45884313c8292feb55dcd0ab5574"
EXPECTED_TAG_COMMIT = "60ad06971467029e77967e59d580cbbe85a1474d"
EXPECTED_ASSETS = {
  "yara-x-capi-v1.20.0-x86_64-pc-windows-msvc.zip" => ["19762002", "sha256:e626902731aa665518c281c2219990885a68882aca06c26e74a2bbd00e865b90", "c_api", false],
  "yara-x-v1.20.0-aarch64-apple-darwin.tar.gz" => ["8119789", "sha256:33685a589133c5112611c06b66a14f759c8788347dc438795ec895b214e2897a", "cli", true],
  "yara-x-v1.20.0-aarch64-unknown-linux-gnu.tar.gz" => ["8787612", "sha256:c1d6f63a6fe55c17b5ddbfcb89d34599b737226a683ee492b12d92d1d541f304", "cli", true],
  "yara-x-v1.20.0-x86_64-apple-darwin.tar.gz" => ["9065029", "sha256:4d06b46eea0231897a4e3561148a95d5895f74286f6dfe5f9da90e0e4fd36007", "cli", true],
  "yara-x-v1.20.0-x86_64-pc-windows-msvc.zip" => ["8640562", "sha256:b1e2840bac593aea353d2b2b341f5a862c9d61c0c406d9abbbad9e1fa35163a1", "cli", true],
  "yara-x-v1.20.0-x86_64-unknown-linux-gnu.tar.gz" => ["9930108", "sha256:cabb8df46492fff59c51261302c71ed9cb2cef393d3f0ca560801a34a8e24cbe", "cli", true]
}.freeze
FIXTURE_DIGESTS = {
  "invalid/yara-x-contract-candidate-command.json" => "a92d0e6122a9072f1da13553ef8452bf33318079c3b78688851dab18c8fac5a1",
  "invalid/yara-x-contract-candidate-overclaim.json" => "10cf1b579df121e0d0de46aab2e03f609c6a2952d4ef66fc8bb5660e4b897a74",
  "invalid/yara-x-contract-profile-asset-drift.json" => "a2ce99bb0105bbb40f2b1f392eee9889032a09744dcd90840ce3294ab1df60d4",
  "valid/yara-x-contract-candidate-receipt.json" => "09b6f2c089f064a4ff348525df1f7833c8882cbbf2990070edae031cd2f43703",
  "valid/yara-x-contract-profile.json" => PROFILE_DIGEST
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid YARA-X contract JSON: #{path}: #{e.message}"
end

def exact(path, digest)
  abort "missing or symlinked YARA-X contract input: #{path}" unless path.file? && !path.symlink?
  abort "YARA-X contract input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

def contract_state(as_of:, expires_at:, tag_commit:, assets_exact:, revoked:, activation_inputs_complete:)
  return "revoked" if revoked
  return "release_tag_moved" unless tag_commit == EXPECTED_TAG_COMMIT
  return "uploaded_asset_metadata_changed" unless assets_exact
  return "expired" if as_of >= expires_at
  return "contract_fixture_only" unless activation_inputs_complete

  "requires_separate_activation_review"
end

profile_path = ROOT.join("profiles/v1/yara-x-contract-v1.json")
exact(profile_path, PROFILE_DIGEST)
sidecar = ROOT.join("profiles/v1/yara-x-contract-v1.sha256").read.strip
abort "YARA-X profile checksum record changed" unless
  sidecar == "#{PROFILE_DIGEST}  yara-x-contract-v1.json"

profile = json(profile_path)
fixture_profile = ROOT.join("tests/conformance/v1/valid/yara-x-contract-profile.json")
abort "YARA-X profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile_schema = json(ROOT.join("schemas/v1/yara-x-contract-profile.schema.json"))
abort "YARA-X profile schema no longer freezes the exact profile" unless profile_schema.fetch("const") == profile

engine = profile.fetch("engine")
abort "YARA-X engine source selection changed" unless engine == {
  "repository" => "https://github.com/VirusTotal/yara-x",
  "release_tag" => "v1.20.0",
  "release_url" => "https://github.com/VirusTotal/yara-x/releases/tag/v1.20.0",
  "tag_commit_sha1" => EXPECTED_TAG_COMMIT,
  "published_at" => "2026-08-24T12:32:25Z",
  "source_archive_api_url" => "https://api.github.com/repos/VirusTotal/yara-x/tarball/v1.20.0",
  "license_spdx" => "BSD-3-Clause",
  "license_path" => "LICENSE",
  "license_git_blob_sha1" => "124f2e84d967b00246443561b7d927c56030308a",
  "cli_binary" => "yr",
  "minimum_rust_version" => "1.93.0"
}

assets = profile.fetch("uploaded_assets").to_h do |asset|
  [asset.fetch("name"), [asset.fetch("bytes"), asset.fetch("sha256"), asset.fetch("kind"), asset.fetch("candidate")]]
end
abort "YARA-X uploaded asset metadata changed" unless assets == EXPECTED_ASSETS

observed_at = Time.iso8601(profile.dig("freshness", "observed_at"))
expires_at = Time.iso8601(profile.dig("freshness", "expires_at"))
abort "YARA-X observation expiry is not exactly 30 days" unless expires_at - observed_at == 30 * 86_400

strategy = profile.fetch("artifact_strategy")
abort "YARA-X official assets gained admission" unless
  strategy.fetch("selected") == "rebuild_from_pinned_source_then_impresari_sign" &&
  strategy.fetch("upstream_cli_assets_candidate_only") &&
  !strategy.fetch("upstream_assets_admitted") &&
  !strategy.fetch("source_archive_sha256_recorded") &&
  !strategy.fetch("upstream_per_asset_signature_present") &&
  !strategy.fetch("upstream_slsa_provenance_present")
required_artifact_controls = strategy.reject do |key, _|
  %w[selected upstream_assets_admitted source_archive_sha256_recorded upstream_per_asset_signature_present upstream_slsa_provenance_present].include?(key)
end
abort "YARA-X artifact strategy lost a required gate" unless required_artifact_controls.values.all?

ruleset = profile.fetch("ruleset_contract")
abort "YARA-X first ruleset gained module authority" unless ruleset.fetch("allowed_modules").empty?
closed_rule_features = %w[includes_allowed external_variables_allowed regular_expressions_allowed base64_allowed xor_allowed repository_provided_rules_allowed relaxed_regex_syntax_allowed ignore_invalid_rules_allowed in_job_update_allowed network_retrieval_allowed worker_update_credentials_allowed]
abort "YARA-X first ruleset gained a closed feature" unless closed_rule_features.none? { |key| ruleset.fetch(key) }

expected_arguments = [
  "--compiled-rules", "--output-format=ndjson", "--print-namespace",
  "--print-tags", "--print-strings=0", "--disable-console-logs",
  "--no-mmap", "--max-matches-per-pattern=32", "--threads=1",
  "--timeout=5", "--skip-larger=262144", "<compiled_ruleset_path>",
  "<staged_artifact_path>"
]
invocation = profile.fetch("invocation_contract")
abort "YARA-X invocation expanded" unless
  invocation.fetch("executable") == "yr" &&
  invocation.fetch("subcommand") == "scan" &&
  invocation.fetch("arguments") == expected_arguments
closed_invocation_features = %w[configuration_file_allowed inherited_environment_allowed recursive_scan_allowed scan_list_allowed module_data_allowed include_directory_allowed arbitrary_arguments_allowed]
abort "YARA-X invocation gained ambient or arbitrary authority" unless closed_invocation_features.none? { |key| invocation.fetch(key) }

output = profile.fetch("output_contract")
abort "YARA-X output boundary no longer strips matched bytes" unless
  output.fetch("format") == "ndjson" &&
  output.fetch("exact_lines") == "1" &&
  output.fetch("match_marker_regex") == "^ \\.\\.\\. ([1-9][0-9]*) more bytes$" &&
  !output.fetch("matched_bytes_retained") &&
  !output.fetch("raw_output_retained") &&
  !output.fetch("raw_error_retained") &&
  !output.fetch("normalized_paths_emitted") &&
  !output.fetch("xor_fields_allowed") &&
  !output.fetch("plaintext_fields_allowed")

resources = profile.fetch("resource_contract")
abort "YARA-X resource contract expanded" unless resources == {
  "maximum_artifact_bytes" => "262144",
  "maximum_ruleset_source_bytes" => "262144",
  "maximum_compiled_ruleset_bytes" => "2097152",
  "maximum_rules" => "256",
  "maximum_patterns_per_rule" => "32",
  "maximum_tags_per_rule" => "32",
  "maximum_identifier_bytes" => "128",
  "maximum_matches_per_pattern" => "32",
  "maximum_normalized_observations" => "256",
  "maximum_ranges_per_observation" => "32",
  "maximum_total_ranges" => "8192",
  "maximum_output_bytes" => "131072",
  "engine_timeout_seconds" => "5",
  "external_runner_timeout_seconds" => "10",
  "threads" => "1"
}

controls = profile.fetch("checkpoint_controls")
abort "contract-only YARA-X checkpoint gained artifact, execution, network, credential, or claim authority" unless controls.values.none?

receipt_path = ROOT.join("tests/conformance/v1/valid/yara-x-contract-candidate-receipt.json")
receipt = json(receipt_path)
receipt_schema = json(ROOT.join("schemas/v1/yara-x-contract-candidate-receipt.schema.json"))
abort "YARA-X receipt schema no longer freezes the exact receipt" unless receipt_schema.fetch("const") == receipt
abort "YARA-X receipt does not bind the exact profile" unless receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}"
receipt_false_fields = %w[source_downloaded release_asset_downloaded executable_present executable_admitted ruleset_present ruleset_admitted live_parser_implemented compatibility_corpus_executed analyzer_executed network_used credentials_used os_confined production_admitted iar_2_admitted safety_claimed ordinary_host_execution_authorized authority_added]
abort "YARA-X contract receipt overclaims authority" unless receipt_false_fields.none? { |key| receipt.fetch(key) }
abort "YARA-X contract receipt exposed an activation target" unless receipt.fetch("eligible_activation_targets").empty?

provenance = json(ROOT.join("tests/conformance/v1/yara-x-contract-fixture-provenance.json"))
recorded = provenance.fetch("fixtures").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "YARA-X fixture provenance is incomplete" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(ROOT.join("tests/conformance/v1", relative), digest) }
%w[yara_x_source_content release_asset_content executable_content rule_content malware_content repository_source_content matched_byte_content credential_content network_capture_content authority_added].each do |key|
  abort "YARA-X fixture provenance crossed #{key}" unless provenance.fetch(key) == false
end

state_vectors = [
  [observed_at, EXPECTED_TAG_COMMIT, true, false, false, "contract_fixture_only"],
  [expires_at, EXPECTED_TAG_COMMIT, true, false, false, "expired"],
  [observed_at, "0" * 40, true, false, false, "release_tag_moved"],
  [observed_at, EXPECTED_TAG_COMMIT, false, false, false, "uploaded_asset_metadata_changed"],
  [observed_at, EXPECTED_TAG_COMMIT, true, true, false, "revoked"],
  [observed_at, EXPECTED_TAG_COMMIT, true, false, true, "requires_separate_activation_review"]
]
state_vectors.each do |as_of, tag_commit, assets_exact, revoked, complete, expected|
  actual = contract_state(as_of: as_of, expires_at: expires_at, tag_commit: tag_commit, assets_exact: assets_exact, revoked: revoked, activation_inputs_complete: complete)
  abort "YARA-X contract state precedence changed: expected #{expected}, got #{actual}" unless actual == expected
end

artifact_globs = [
  ROOT.join("{platform,profiles,crates}/**/*.{yar,yara,yarc}"),
  ROOT.join("{platform,profiles,rules,crates}/**/yr"),
  ROOT.join("{platform,profiles,rules,crates}/**/yr.exe")
]
abort "YARA-X executable or rule artifact entered the repository" unless artifact_globs.flat_map { |glob| Dir.glob(glob.to_s) }.empty?

allowed_rules = Dir.glob(ROOT.join("rules/**/*.{yar,yara,yarc}").to_s).map do |path|
  Pathname.new(path).relative_path_from(ROOT).to_s
end
abort "unexpected YARA-X rule artifact entered the repository" unless
  allowed_rules == ["rules/yara-x/synthetic-compatibility-v1.yar"]

production_refs = Dir.glob(ROOT.join("crates/**/*.rs").to_s).select do |path|
  File.read(path).match?(/yara[_-]x|\byr\s+scan\b/i)
end
abort "YARA-X implementation or launch reference entered production Rust: #{production_refs.join(', ')}" unless production_refs.empty?

puts "YARA-X contract verified: engine=v1.20.0@#{EXPECTED_TAG_COMMIT} uploaded_assets=#{assets.length} artifacts=absent state=contract_fixture_only"
