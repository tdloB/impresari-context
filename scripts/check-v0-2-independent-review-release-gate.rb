#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
GATE = ROOT.join("scripts/enforce-v0-2-independent-review-release-gate.rb")
CANDIDATE_SCOPE = ROOT.join("release-review/v0.2.0-independent-review-candidate-scope.json")
HISTORICAL_SCOPE = ROOT.join("release-review/v0.2.0-independent-review-scope.json")
SOURCE_SHA = "1a9923c0e5d671581f6b7da3bc4248b604971d63"

def run_gate(tag, source_sha, scope, record = nil)
  command = [RbConfig.ruby, GATE.to_s, tag, source_sha, scope.to_s]
  command << record.to_s if record
  Open3.capture3(*command)
end

_stdout, stderr, status = run_gate("v0.2.0", SOURCE_SHA, CANDIDATE_SCOPE)
abort("unreviewed candidate passed the release gate") if status.success?
abort("unreviewed candidate did not fail on missing review record") unless stderr.include?("independent review record is unavailable")

legacy_stdout, legacy_stderr, legacy_status = run_gate("v0.1.0", SOURCE_SHA, HISTORICAL_SCOPE)
abort("v0.1.0 legacy policy was broken: #{legacy_stderr}") unless legacy_status.success? && legacy_stdout.include?("legacy policy applies")

Dir.mktmpdir("impresari-review-release-gate-") do |directory|
  record = {
    "schema_name" => "independent-security-review-record",
    "schema_version" => "1.0.0",
    "scope_id" => "impresari-context-v0-2-0-candidate-review-v1",
    "target_version" => "0.2.0",
    "scope_identity" => Digest::SHA256.file(CANDIDATE_SCOPE).hexdigest,
    "product_source_commit" => SOURCE_SHA,
    "status" => "review_recorded",
    "reviewer_reference" => "Independent application-security reviewer",
    "independence_statement" => "Did not implement the reviewed source.",
    "conflict_disclosure" => "No conflict disclosed.",
    "report_sha256" => "a" * 64,
    "reviewed_at" => "2026-08-30",
    "critical_open" => 0,
    "high_open" => 0,
    "unknown_open" => 0,
    "medium_dispositions_complete" => true,
    "low_documentation_complete" => true,
    "claim" => {
      "review_gate_satisfied" => true,
      "release_ready" => false,
      "publication_authorized" => false,
      "production_support_admitted" => false,
      "real_analyzer_authorized" => false,
    },
    "safe_next_step" => "Run the remaining release gates and obtain separate owner approval before tagging or publication.",
  }
  record_path = Pathname.new(directory).join("record.json")
  record_path.write(JSON.pretty_generate(record) + "\n")

  stdout, stderr, status = run_gate("v0.2.0", SOURCE_SHA, CANDIDATE_SCOPE, record_path)
  abort("exact admitted review was rejected: #{stderr}") unless status.success? && stdout.include?("release gate passed")

  record["scope_identity"] = "0" * 64
  record_path.write(JSON.pretty_generate(record) + "\n")
  _stdout, stderr, status = run_gate("v0.2.0", SOURCE_SHA, CANDIDATE_SCOPE, record_path)
  abort("scope-mismatched review passed") if status.success?
  abort("scope mismatch reason drifted") unless stderr.include?("not bound to the candidate scope")

  _stdout, stderr, status = run_gate("v0.3.0", SOURCE_SHA, CANDIDATE_SCOPE, record_path)
  abort("unrecorded release policy passed") if status.success?
  abort("unrecorded policy reason drifted") unless stderr.include?("no independent review release policy")
end

puts "v0.2 independent review release gate checks passed: legacy preserved, current blocked, immutable scope plus exact future record accepted"
