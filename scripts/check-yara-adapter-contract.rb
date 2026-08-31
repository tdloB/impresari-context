#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/yara-adapter-contract-v1.json"
PROFILE_DIGEST = "686c812f098c38e5b347d53279904744b5e937dbc9539c228a47f358b69ab3d7"
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
INPUT_RELATIVE = "valid/yara-adapter-synthetic-result.json"
RECEIPT_RELATIVE = "valid/yara-adapter-normalization-receipt.json"

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def canonical_strings?(values)
  values == values.sort && values.uniq == values
end

profile_path = ROOT.join(PROFILE_RELATIVE)
abort "missing or symlinked YARA adapter profile" unless profile_path.file? && !profile_path.symlink?
abort "YARA adapter profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/yara-adapter-contract-v1.sha256").read.strip
abort "YARA adapter checksum record mismatch" unless sidecar == "#{PROFILE_DIGEST}  yara-adapter-contract-v1.json"
fixture_profile = FIXTURE_ROOT.join("valid/yara-adapter-contract-profile.json")
abort "YARA adapter profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile = read_json(profile_path)
abort "YARA adapter profile identity changed" unless
  profile.fetch("profile_id") == "yara-adapter-contract-v1" &&
    profile.dig("adapter", "analyzer_id") == "impresari.yara" &&
    profile.dig("adapter", "result_origin") == "original_synthetic_fixture" &&
    profile.dig("adapter", "normalization_method") == "yara_rule_observation_v1" &&
    profile.dig("adapter", "match_identity_domain") == "impresari-context/yara-adapter-match/v1"
abort "YARA adapter limits changed" unless profile.fetch("limits") == {
  "max_artifacts" => "64",
  "max_observations" => "256",
  "max_tags_per_observation" => "32",
  "max_ranges_per_observation" => "32",
  "max_rule_identifier_bytes" => "128",
  "max_namespace_bytes" => "128",
  "max_total_input_bytes" => "262144"
}
abort "YARA adapter input boundary gained data or authority" unless profile.fetch("input_contract").values.none?
abort "YARA adapter claims gained authority" unless profile.fetch("claims").values.none?

provenance = read_json(FIXTURE_ROOT.join("yara-adapter-fixture-provenance.json"))
expected_paths = %w[
  invalid/yara-adapter-normalization-overclaim.json
  invalid/yara-adapter-synthetic-result-path.json
  valid/yara-adapter-contract-profile.json
  valid/yara-adapter-normalization-receipt.json
  valid/yara-adapter-synthetic-result.json
]
entries = provenance.fetch("fixtures")
abort "YARA fixture provenance is not closed and sorted" unless entries.map { |entry| entry.fetch("path") } == expected_paths
entries.each do |entry|
  path = FIXTURE_ROOT.join(entry.fetch("path")).cleanpath
  abort "YARA fixture escapes the fixture root" unless path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked YARA fixture" unless path.file? && !path.symlink?
  abort "YARA fixture provenance digest changed: #{entry.fetch('path')}" unless
    Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
end
%w[malware_content third_party_content executable_content repository_source_content credential_content network_capture_content authority_added].each do |key|
  abort "YARA fixture provenance crossed #{key}" if provenance.fetch(key)
end

input_path = FIXTURE_ROOT.join(INPUT_RELATIVE)
abort "YARA synthetic input exceeds frozen byte limit" if input_path.size > Integer(profile.dig("limits", "max_total_input_bytes"), 10)
input = read_json(input_path)
abort "YARA adapter accepted a non-synthetic origin" unless
  input.fetch("result_origin") == "original_synthetic_fixture" &&
    !input.fetch("analyzer_executed") && !input.fetch("authority_added")
abort "YARA adapter input identity mismatch" unless
  input.fetch("profile_id") == profile.fetch("profile_id") &&
    input.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    input.fetch("analyzer_id") == profile.dig("adapter", "analyzer_id")

artifacts = input.fetch("artifacts")
observations = input.fetch("observations")
abort "YARA artifacts are not canonical" unless
  artifacts.map { |item| item.fetch("artifact_hash") } == artifacts.map { |item| item.fetch("artifact_hash") }.sort &&
    artifacts.map { |item| item.fetch("artifact_hash") }.uniq.length == artifacts.length
observation_keys = observations.map do |item|
  [item.fetch("artifact_hash"), item.fetch("namespace"), item.fetch("rule_identifier")]
end
abort "YARA observations are not canonical and unique" unless
  observation_keys == observation_keys.sort && observation_keys.uniq == observation_keys

artifacts_by_hash = artifacts.to_h { |item| [item.fetch("artifact_hash"), item] }
observations.each do |observation|
  artifact = artifacts_by_hash.fetch(observation.fetch("artifact_hash")) { abort "YARA observation references an unknown artifact" }
  abort "YARA observation attached to a non-match artifact" unless artifact.fetch("status") == "synthetic_match_fixture"
  abort "YARA observation tags are not canonical" unless canonical_strings?(observation.fetch("tags"))
  ranges = observation.fetch("ranges")
  expected_range_order = ranges.sort_by { |range| [Integer(range.fetch("offset"), 10), range.fetch("string_identifier"), Integer(range.fetch("length"), 10)] }
  abort "YARA observation ranges are not canonical" unless ranges == expected_range_order && ranges.uniq == ranges
  ranges.each do |range|
    offset = Integer(range.fetch("offset"), 10)
    length = Integer(range.fetch("length"), 10)
    abort "YARA observation has an empty or out-of-bounds range" unless length.positive? && offset + length <= Integer(artifact.fetch("bytes"), 10)
  end
end

observation_counts = observations.each_with_object(Hash.new(0)) do |item, counts|
  counts[item.fetch("artifact_hash")] += 1
end
artifacts.each do |artifact|
  count = observation_counts.fetch(artifact.fetch("artifact_hash"), 0)
  expected_match = artifact.fetch("status") == "synthetic_match_fixture"
  abort "YARA artifact accounting and observations disagree" unless expected_match == count.positive?
end

matches = observations.map do |observation|
  range_identity = observation.fetch("ranges").map do |range|
    "#{range.fetch('string_identifier')}:#{range.fetch('offset')}:#{range.fetch('length')}"
  end.join(",")
  identity_parts = [
    profile.dig("adapter", "match_identity_domain"),
    input.fetch("workspace_snapshot"),
    observation.fetch("artifact_hash"),
    observation.fetch("namespace"),
    observation.fetch("rule_identifier"),
    observation.fetch("tags").join(","),
    range_identity
  ]
  {
    "match_id" => "sha256:#{Digest::SHA256.hexdigest(identity_parts.join("\0"))}",
    "artifact_hash" => observation.fetch("artifact_hash"),
    "namespace" => observation.fetch("namespace"),
    "rule_identifier" => observation.fetch("rule_identifier"),
    "tags" => observation.fetch("tags"),
    "evidence_ranges" => observation.fetch("ranges"),
    "classification" => "untrusted_derived_data",
    "method" => profile.dig("adapter", "normalization_method"),
    "trust" => "untrusted_derived_data",
    "limitations" => ["synthetic-fixture", "rule-match-is-not-a-safety-verdict"],
    "authority_added" => false
  }
end

expected_receipt = {
  "schema_name" => "yara-adapter-normalization-receipt",
  "schema_version" => "1.0.0",
  "fixture_id" => input.fetch("fixture_id"),
  "profile_id" => input.fetch("profile_id"),
  "profile_digest" => input.fetch("profile_digest"),
  "workspace_snapshot" => input.fetch("workspace_snapshot"),
  "manifest_id" => input.fetch("manifest_id"),
  "analyzer_id" => input.fetch("analyzer_id"),
  "executable_digest" => input.fetch("executable_digest"),
  "ruleset_digest" => input.fetch("ruleset_digest"),
  "completed_at" => input.fetch("completed_at"),
  "artifact_statuses" => artifacts.map do |artifact|
    artifact.merge("observation_count" => observation_counts.fetch(artifact.fetch("artifact_hash"), 0).to_s)
  end,
  "matches" => matches,
  "completeness" => "synthetic_fixture_complete",
  "limitations" => ["contract-only", "no-live-analyzer", "not-a-safety-verdict", "original-synthetic-fixture"],
  "raw_output_retained" => false,
  "source_bytes_retained" => false,
  "analyzer_executed" => false,
  "os_confined" => false,
  "production_admitted" => false,
  "iar_2_admitted" => false,
  "safety_claimed" => false,
  "ordinary_host_execution_authorized" => false,
  "authority_added" => false
}
receipt = read_json(FIXTURE_ROOT.join(RECEIPT_RELATIVE))
abort "committed YARA normalization receipt is not the deterministic result" unless receipt == expected_receipt

rust_sources = ROOT.join("crates").glob("**/*.rs").reject { |path| path.to_s.include?("/target/") }
abort "production Rust code gained a YARA implementation or launch reference" if
  rust_sources.any? { |path| path.read.match?(/\byara\b/i) }

puts "YARA adapter contract verified: profile=sha256:#{PROFILE_DIGEST} artifacts=#{artifacts.length} " \
  "matches=#{matches.length} analyzer_executed=false production_admitted=false iar_2_admitted=false"
