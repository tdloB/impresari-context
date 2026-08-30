#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"

class GateError < StandardError; end

def exact_keys!(value, keys, description)
  raise GateError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && value.keys.sort == keys.sort
end

tag = ARGV.fetch(0) { abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG SOURCE_SHA [SCOPE]" }
source_sha = ARGV.fetch(1) { abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG SOURCE_SHA [SCOPE]" }
scope_path = ARGV.fetch(2, File.expand_path("../release-review/v0.2.0-independent-review-scope.json", __dir__))
abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG SOURCE_SHA [SCOPE]" unless ARGV.length.between?(2, 3)

begin
  raise GateError, "release source identity is invalid" unless source_sha.match?(/\A[0-9a-f]{40}\z/)
  if tag == "v0.1.0"
    puts "independent review release gate: v0.1.0 legacy policy applies"
    exit 0
  end
  raise GateError, "no independent review release policy is recorded for #{tag}" unless tag == "v0.2.0"
  raise GateError, "review scope is unavailable" unless File.file?(scope_path) && !File.symlink?(scope_path)

  scope = JSON.parse(File.binread(scope_path))
  exact_keys!(scope, %w[schema_name schema_version scope_id target_version product_source_commit status triggered_boundaries review_areas required_artifacts reviewer_requirements finding_policy claim report safe_next_step], "review scope")
  raise GateError, "review scope identity is unsupported" unless scope.values_at("schema_name", "schema_version", "target_version", "status") == [
    "independent-security-review-scope", "1.0.0", "0.2.0", "review_recorded"
  ]
  raise GateError, "reviewed source does not match the release tag" unless scope.fetch("product_source_commit") == source_sha
  claim = scope.fetch("claim")
  raise GateError, "review claim is not narrowly admitted" unless claim == {
    "review_gate_satisfied" => true, "release_ready" => false, "publication_authorized" => false,
    "production_support_admitted" => false, "real_analyzer_authorized" => false,
  }
  report = scope.fetch("report")
  exact_keys!(report, %w[reviewer_reference independence_statement conflict_disclosure report_sha256 reviewed_commit reviewed_at critical_open high_open medium_dispositions_complete low_documentation_complete], "review report")
  raise GateError, "review report is not bound to the release source" unless report.fetch("reviewed_commit") == source_sha
  raise GateError, "review report identity is invalid" unless report.fetch("report_sha256").match?(/\A[0-9a-f]{64}\z/)
  raise GateError, "reviewer attribution or independence is missing" if %w[reviewer_reference independence_statement conflict_disclosure].any? { |key| report.fetch(key).to_s.empty? }
  raise GateError, "blocking review findings remain open" unless report.fetch("critical_open").zero? && report.fetch("high_open").zero?
  raise GateError, "review finding dispositions are incomplete" unless report.fetch("medium_dispositions_complete") && report.fetch("low_documentation_complete")
  puts "independent review release gate passed for #{tag} at #{source_sha}"
rescue JSON::ParserError
  warn "error: review scope is not valid JSON"
  exit 1
rescue GateError, KeyError => error
  warn "error: #{error.message}"
  exit 1
end
