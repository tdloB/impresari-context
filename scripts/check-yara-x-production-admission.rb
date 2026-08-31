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
FIXTURE_DIGESTS = {
  "observations-release-pending.json" => "656aee370c5d3b0f3a2441a99b9b9c4bb59e32b254927c128a0a10f005c5dce8",
  "receipt-release-pending.json" => "80e9a068e4a4b4fce77198167967e3268d5a2c3b6e7bfac3af19cbbe4ca59303"
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

exact(policy_path, POLICY_DIGEST)
abort "production-admission sidecar changed" unless
  sidecar_path.read.strip == "#{POLICY_DIGEST}  yara-x-production-admission-v1.json"
FIXTURE_DIGESTS.each { |path, digest| exact(fixture_root.join(path), digest) }

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
