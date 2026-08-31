#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tempfile"

ROOT = Pathname.new(__dir__).join("..").expand_path
POLICY_DIGEST = "fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c"
SCHEMA_DIGEST = "eda3497fcc6a56a07ded32c5bec3b3f2f922af6d1d4c02792827fb425d2deb54"
FIXTURE_DIGESTS = {
  "observations-release-pending.json" => "656aee370c5d3b0f3a2441a99b9b9c4bb59e32b254927c128a0a10f005c5dce8",
  "receipt-release-pending.json" => "80e9a068e4a4b4fce77198167967e3268d5a2c3b6e7bfac3af19cbbe4ca59303"
}.freeze
CONFORMANCE_FIXTURE_DIGESTS = {
  "valid/yara-x-engine-bundle-candidate.json" => "6fe4210371c0f67707248c7586edbd0651e2207dbf7a2543a70563c52ede1894",
  "valid/yara-x-ruleset-bundle-candidate.json" => "7167b00abb5adb80687691910f9688608dc17ba296531cda7ca29bc97f3f76ad",
  "valid/yara-x-release-binding-candidate.json" => "1aaf442a42737908162394463cc44167ed722a7fc153e2a2fdd0eb7a6e9f9275",
  "invalid/yara-x-engine-bundle-admitted.json" => "132af96762cbc574597156a6c8578dc7ae8c247d26faf2ea8236b515179b9105",
  "invalid/yara-x-ruleset-bundle-synthetic.json" => "51e9fb9a51f828e46e381a49d7b9449f8a06485cd6db4ed2984dbd11e20c26ec",
  "invalid/yara-x-release-binding-activated.json" => "05b0381a9df85d5d217724bc8243e79c9cea0acf76f322a4e444b15cb2b0c476"
}.freeze
FALSE_RECEIPT_CLAIMS = %w[
  artifact_retained executable_admitted ruleset_admitted production_admitted
  iar_2_admitted repository_scan_authorized credentials_used artifact_uploaded
  detection_quality_claimed safety_claimed malware_free_claimed authority_added
].freeze

def exact(path, digest)
  abort "missing or symlinked production-admission input: #{path}" unless path.file? && !path.symlink?
  abort "production-admission input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid production-admission JSON: #{path}: #{e.message}"
end

def evaluate(evaluator, policy, observation)
  Tempfile.create(["yara-x-production-observation-", ".json"]) do |file|
    file.write(JSON.generate(observation))
    file.flush
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      evaluator.to_s,
      "--policy", policy.to_s,
      "--observations", file.path
    )
    abort "production-admission evaluator failed: #{stderr}" unless status.success?
    abort "production-admission evaluator wrote stderr" unless stderr.empty?
    JSON.parse(stdout)
  end
end

policy_path = ROOT.join("profiles/v1/yara-x-production-admission-v1.json")
sidecar_path = ROOT.join("profiles/v1/yara-x-production-admission-v1.sha256")
evaluator = ROOT.join("scripts/yara-x-production-admission.rb")
fixture_root = ROOT.join("tests/yara-x-production-admission")
observations_path = fixture_root.join("observations-release-pending.json")
receipt_path = fixture_root.join("receipt-release-pending.json")
schema_path = ROOT.join("schemas/v1/yara-x-production-admission.schema.json")
registry_path = ROOT.join("schemas/v1/registry.json")
manifest_path = ROOT.join("tests/conformance/v1/manifest.json")
conformance_root = ROOT.join("tests/conformance/v1")

exact(policy_path, POLICY_DIGEST)
exact(schema_path, SCHEMA_DIGEST)
abort "production-admission sidecar changed" unless
  sidecar_path.read.strip == "#{POLICY_DIGEST}  yara-x-production-admission-v1.json"
FIXTURE_DIGESTS.each { |path, digest| exact(fixture_root.join(path), digest) }
CONFORMANCE_FIXTURE_DIGESTS.each { |path, digest| exact(conformance_root.join(path), digest) }

schema = json(schema_path)
definitions = schema.fetch("$defs")
abort "production-admission schema lost a closed bundle" unless
  %w[engineBundle rulesetBundle releaseBinding].all? do |name|
    definitions.fetch(name).fetch("additionalProperties") == false
  end
abort "engine candidate schema gained admission authority" unless
  definitions.dig("engineBundle", "properties", "admitted", "const") == false
abort "ruleset candidate schema gained admission authority" unless
  definitions.dig("rulesetBundle", "properties", "admitted", "const") == false
abort "synthetic rules became production-bundle provenance" if
  definitions.dig("rulesetBundle", "properties", "source", "properties", "provenance").to_s.include?("synthetic")
abort "release-binding candidate schema gained activation authority" unless
  definitions.dig("releaseBinding", "properties", "activated", "const") == false &&
  definitions.dig("releaseBinding", "properties", "gates", "properties", "activation_review", "const") == false

registry = json(registry_path).fetch("schemas")
abort "production-admission schema is not registered exactly once" unless
  registry.count { |entry| entry.fetch("name") == "yara-x-production-admission" &&
    entry.fetch("path") == "yara-x-production-admission.schema.json" &&
    entry.fetch("identity_object_kind") == nil } == 1

manifest = json(manifest_path).fetch("cases")
expected_manifest = {
  "valid/yara-x-engine-bundle-candidate.json" => ["yara-x-production-admission.schema.json#/$defs/engineBundle", true],
  "valid/yara-x-ruleset-bundle-candidate.json" => ["yara-x-production-admission.schema.json#/$defs/rulesetBundle", true],
  "valid/yara-x-release-binding-candidate.json" => ["yara-x-production-admission.schema.json#/$defs/releaseBinding", true],
  "invalid/yara-x-engine-bundle-admitted.json" => ["yara-x-production-admission.schema.json#/$defs/engineBundle", false],
  "invalid/yara-x-ruleset-bundle-synthetic.json" => ["yara-x-production-admission.schema.json#/$defs/rulesetBundle", false],
  "invalid/yara-x-release-binding-activated.json" => ["yara-x-production-admission.schema.json#/$defs/releaseBinding", false]
}
expected_manifest.each do |fixture, (schema_ref, valid)|
  matches = manifest.select { |entry| entry.fetch("fixture") == fixture }
  abort "production-admission fixture is not declared exactly once: #{fixture}" unless
    matches == [{"fixture" => fixture, "schema" => schema_ref, "valid" => valid}]
end

source = evaluator.read
abort "production-admission evaluator gained process or network capability" if
  source.match?(/(?:Open3|IO\.popen|Net::HTTP|TCPSocket|UDPSocket|Socket\.|Kernel\.(?:system|exec|spawn)|`)/)
abort "production-admission evaluator gained ambient time" if source.match?(/(?:Time|Date)\.(?:now|today)/)

policy = json(policy_path)
abort "production-admission policy lost the exact live evidence" unless
  policy.dig("prior_evidence", "live_run_id") == "33432469614" &&
  policy.dig("prior_evidence", "live_job_id") == "99620875408" &&
  policy.dig("prior_evidence", "live_candidate_passed") &&
  !policy.dig("prior_evidence", "live_candidate_production_admitted")
abort "production-admission target expanded" unless
  policy.dig("target_scope", "target_triple") == "x86_64-unknown-linux-gnu" &&
  policy.dig("target_scope", "isolation_profile") == "externally_managed" &&
  policy.dig("target_scope", "support_release_state") == "pending_publication" &&
  !policy.dig("target_scope", "broad_linux_support") &&
  !policy.dig("target_scope", "macos_eligible") &&
  !policy.dig("target_scope", "windows_eligible")
abort "synthetic rules became production eligible" if
  policy.dig("ruleset_bundle", "synthetic_compatibility_rules_eligible")
abort "production bundle entered the closed policy" if
  policy.dig("engine_bundle", "retained_candidate_present") ||
  policy.dig("engine_bundle", "signature_present") ||
  policy.dig("engine_bundle", "admitted") ||
  policy.dig("ruleset_bundle", "production_source_present") ||
  policy.dig("ruleset_bundle", "human_review_present") ||
  policy.dig("ruleset_bundle", "signature_present") ||
  policy.dig("ruleset_bundle", "admitted") ||
  policy.dig("release_binding", "binding_present") ||
  policy.dig("release_binding", "activated")
abort "production-admission policy overclaims" if policy.fetch("claims").values.any?

base = json(observations_path)
expected = json(receipt_path)
actual = evaluate(evaluator, policy_path, base)
abort "release-pending receipt changed" unless actual == expected
abort "release-pending receipt overclaims" unless
  FALSE_RECEIPT_CLAIMS.none? { |key| actual.fetch(key) }

vectors = [
  [{"revoked" => true}, "revoked"],
  [{"evidence_present" => false}, "missing_evidence"],
  [{"evidence_exact" => false}, "changed"],
  [{"evidence_fresh" => false}, "stale"],
  [{"as_of" => "2026-09-14"}, "stale"],
  [{"release_published" => true, "target_supported" => false}, "unsupported"],
  [{"release_published" => true}, "compatible_not_activated"],
  [{
    "release_published" => true,
    "engine_bundle_complete" => true,
    "ruleset_bundle_complete" => true,
    "binding_complete" => true,
    "lifecycle_complete" => true,
    "activation_approved" => true
  }, "compatible_not_activated"]
]
vectors.each do |changes, expected_state|
  receipt = evaluate(evaluator, policy_path, base.merge(changes))
  abort "state precedence changed: expected #{expected_state}, got #{receipt.fetch('state')}" unless
    receipt.fetch("state") == expected_state
  abort "closed evaluator emitted authority for #{expected_state}" unless
    FALSE_RECEIPT_CLAIMS.none? { |key| receipt.fetch(key) }
end

puts "YARA-X production admission contract verified: state=release_pending active=false production=false iar_2=false"
