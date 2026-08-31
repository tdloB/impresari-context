#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "date"
require "digest"
require "json"
require "optparse"

POLICY_SHA256 = "fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c"
FALSE_CLAIMS = %w[
  artifact_retained executable_admitted ruleset_admitted production_admitted
  iar_2_admitted repository_scan_authorized credential_access_authorized
  artifact_upload_authorized detection_quality_claimed safety_claimed
  malware_free_claimed authority_added
].freeze
OBSERVATION_KEYS = %w[
  schema_name schema_version observation_id policy_id policy_digest as_of
  revoked evidence_present evidence_exact evidence_fresh release_published
  target_supported engine_bundle_complete ruleset_bundle_complete
  binding_complete lifecycle_complete activation_approved
].freeze

def read_regular_bytes(path, label)
  stat = File.lstat(path)
  abort "#{label} must be a regular non-symlink file" unless stat.file? && !stat.symlink?
  File.binread(path)
rescue Errno::ENOENT, Errno::EACCES => e
  abort "invalid #{label}: #{e.message}"
end

def parse_json(bytes, label)
  JSON.parse(bytes)
rescue JSON::ParserError => e
  abort "invalid #{label}: #{e.message}"
end

def exact_keys!(object, keys, label)
  abort "#{label} must be an object" unless object.is_a?(Hash)
  abort "#{label} fields changed" unless object.keys.sort == keys.sort
end

def boolean!(value, label)
  abort "#{label} must be boolean" unless value == true || value == false
end

def state_for(observation, policy)
  return "revoked" if observation.fetch("revoked")
  return "missing_evidence" unless observation.fetch("evidence_present")
  return "changed" unless observation.fetch("evidence_exact")

  as_of = Date.iso8601(observation.fetch("as_of"))
  fresh_through = Date.iso8601(policy.dig("target_scope", "support_evidence_fresh_through"))
  return "stale" unless observation.fetch("evidence_fresh") && as_of <= fresh_through
  return "release_pending" unless observation.fetch("release_published")
  return "unsupported" unless observation.fetch("target_supported")

  complete = %w[
    engine_bundle_complete ruleset_bundle_complete binding_complete
    lifecycle_complete activation_approved
  ].all? { |key| observation.fetch(key) }
  return "compatible_not_activated" unless complete

  # The v1 policy itself is activation-closed. A later reviewed policy and
  # digest are required before any evaluator can emit an active claim.
  return "compatible_not_activated" unless policy.dig("release_binding", "activated")

  "active"
end

def missing_requirements(observation, policy)
  missing = []
  missing << "fresh-compatible-evidence" unless observation.fetch("evidence_present") && observation.fetch("evidence_exact") && observation.fetch("evidence_fresh")
  missing << "immutable-release" unless observation.fetch("release_published")
  missing << "supported-target" unless observation.fetch("target_supported")
  missing << "retained-admitted-engine-bundle" unless observation.fetch("engine_bundle_complete")
  missing << "reviewed-admitted-ruleset-bundle" unless observation.fetch("ruleset_bundle_complete")
  missing << "complete-release-binding" unless observation.fetch("binding_complete")
  missing << "production-lifecycle-evidence" unless observation.fetch("lifecycle_complete")
  missing << "separate-activation-review" unless observation.fetch("activation_approved") && policy.dig("release_binding", "activated")
  missing
end

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby scripts/yara-x-production-admission.rb --policy FILE --observations FILE"
  parser.on("--policy FILE") { |value| options[:policy] = value }
  parser.on("--observations FILE") { |value| options[:observations] = value }
end.parse!
abort "policy and observations are required" unless options.keys.sort == %i[observations policy]

policy_bytes = read_regular_bytes(options.fetch(:policy), "production-admission policy")
abort "production-admission policy digest changed" unless Digest::SHA256.hexdigest(policy_bytes) == POLICY_SHA256
policy = parse_json(policy_bytes, "production-admission policy")
observation = parse_json(
  read_regular_bytes(options.fetch(:observations), "production-admission observations"),
  "production-admission observations"
)

exact_keys!(policy, %w[
  schema_name schema_version policy_id policy_version prior_evidence
  target_scope engine_bundle ruleset_bundle release_binding state_precedence
  claims
], "policy")
abort "unexpected production-admission policy" unless
  policy.fetch("schema_name") == "yara-x-production-admission-policy" &&
  policy.fetch("schema_version") == "1.0.0" &&
  policy.fetch("policy_id") == "yara-x-production-admission-v1" &&
  policy.fetch("policy_version") == "1.0.0"
abort "policy precedence changed" unless policy.fetch("state_precedence") == %w[
  revoked missing_evidence changed stale release_pending unsupported
  compatible_not_activated active
]
exact_keys!(policy.fetch("claims"), FALSE_CLAIMS, "policy claims")
abort "v1 production-admission policy gained authority" unless
  FALSE_CLAIMS.none? { |key| policy.dig("claims", key) }
abort "v1 production-admission policy became active" if
  policy.dig("engine_bundle", "admitted") ||
  policy.dig("ruleset_bundle", "admitted") ||
  policy.dig("release_binding", "activated")

exact_keys!(observation, OBSERVATION_KEYS, "observations")
abort "unexpected observations contract" unless
  observation.fetch("schema_name") == "yara-x-production-admission-observations" &&
  observation.fetch("schema_version") == "1.0.0" &&
  observation.fetch("policy_id") == policy.fetch("policy_id") &&
  observation.fetch("policy_digest") == "sha256:#{POLICY_SHA256}"
(OBSERVATION_KEYS - %w[schema_name schema_version observation_id policy_id policy_digest as_of]).each do |key|
  boolean!(observation.fetch(key), key)
end
Date.iso8601(observation.fetch("as_of"))

state = state_for(observation, policy)
abort "activation is closed by the v1 policy" if state == "active"
receipt = {
  "schema_name" => "yara-x-production-admission-receipt",
  "schema_version" => "1.0.0",
  "receipt_id" => "#{observation.fetch('observation_id')}_receipt",
  "policy_id" => policy.fetch("policy_id"),
  "policy_digest" => "sha256:#{POLICY_SHA256}",
  "evaluated_as_of" => observation.fetch("as_of"),
  "state" => state,
  "eligible_targets" => [],
  "missing_requirements" => missing_requirements(observation, policy),
  "limitations" => [
    "contract-and-source-free-evaluation-only",
    "no-retained-production-artifact",
    "no-production-ruleset",
    "not-production",
    "not-iar-2",
    "not-a-safety-verdict"
  ],
  "artifact_retained" => false,
  "executable_admitted" => false,
  "ruleset_admitted" => false,
  "production_admitted" => false,
  "iar_2_admitted" => false,
  "repository_scan_authorized" => false,
  "credentials_used" => false,
  "artifact_uploaded" => false,
  "detection_quality_claimed" => false,
  "safety_claimed" => false,
  "malware_free_claimed" => false,
  "authority_added" => false
}

STDOUT.write(JSON.generate(receipt))
STDOUT.write("\n")
