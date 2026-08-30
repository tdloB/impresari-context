#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "optparse"

PINNED_SCOPE_SHA256 = "98a248d7133c85366a16b0a443dab15f131529d1bc4e3d8587b0adfc7925a45c"
AUTHORITY = %w[source_read source_write network credential_access process_execution tag_creation release_publication risk_acceptance].to_h { |key| [key, "denied"] }.freeze

class ContractError < StandardError; end

def exact_keys!(value, keys, description)
  raise ContractError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && value.keys.sort == keys.sort
end

def load_scope(path)
  bytes = File.binread(path)
  raise ContractError, "scope identity is not the tracked review package" unless Digest::SHA256.hexdigest(bytes) == PINNED_SCOPE_SHA256
  scope = JSON.parse(bytes)
  exact_keys!(scope, %w[schema_name schema_version scope_id target_version product_source_commit status triggered_boundaries review_areas required_artifacts reviewer_requirements finding_policy claim safe_next_step], "scope")
  raise ContractError, "scope contract is unsupported" unless scope.values_at("schema_name", "schema_version", "scope_id", "target_version", "status") == [
    "independent-security-review-scope", "1.0.0", "impresari-context-v0-2-0-review-v1", "0.2.0", "manual_review_required"
  ]
  raise ContractError, "scope claim overreaches" unless scope.fetch("claim") == {
    "review_gate_satisfied" => false, "release_ready" => false, "publication_authorized" => false,
    "production_support_admitted" => false, "real_analyzer_authorized" => false,
  }
  [scope, bytes]
rescue JSON::ParserError
  raise ContractError, "scope is not valid JSON"
end

def receipt(scope, bytes, options, status, reason, checks, safe_next_step = nil)
  {
    "schema_name" => "independent-security-review-receipt",
    "schema_version" => "1.0.0",
    "status" => status,
    "reason_code" => reason,
    "scope_id" => scope.fetch("scope_id"),
    "target_version" => scope.fetch("target_version"),
    "observed" => {
      "scope_available" => options.fetch(:scope_available),
      "report_available" => options.fetch(:report_available),
      "reviewer_independent" => options.fetch(:reviewer_independent),
      "target_version" => options.fetch(:target_version),
      "product_source_commit" => options.fetch(:product_source_commit),
      "report_sha256" => options.fetch(:report_sha256),
      "critical_open" => options.fetch(:critical_open),
      "high_open" => options.fetch(:high_open),
    },
    "scope_identity" => Digest::SHA256.hexdigest(bytes),
    "checks" => checks,
    "safe_next_step" => safe_next_step || scope.fetch("safe_next_step"),
    "review_gate_satisfied" => status == "review_admitted",
    "release_ready" => false,
    "publication_authorized" => false,
    "production_support_admitted" => false,
    "real_analyzer_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(path, options)
  scope, bytes = load_scope(path)
  checks = %w[scope_valid scope_identity_pinned]
  return receipt(scope, bytes, options, "unsupported", "target_version_unrecorded", checks + ["target_version_unrecorded"], "Prepare a separately reviewed scope for the requested version.") unless options.fetch(:target_version) == scope.fetch("target_version")
  checks << "target_version_exact"
  return receipt(scope, bytes, options, "missing_evidence", "scope_unavailable", checks + ["scope_unavailable"]) unless options.fetch(:scope_available)
  checks << "scope_available"
  return receipt(scope, bytes, options, "changed", "product_source_changed", checks + ["product_source_changed"], "Refresh the exact review scope after every product-source change before asking a reviewer to proceed.") unless options.fetch(:product_source_commit) == scope.fetch("product_source_commit")
  checks << "product_source_exact"
  if options.fetch(:report_available)
    return receipt(scope, bytes, options, "invalid_review", "reviewer_not_independent", checks + ["reviewer_not_independent"]) unless options.fetch(:reviewer_independent)
    report_sha = options.fetch(:report_sha256)
    return receipt(scope, bytes, options, "invalid_review", "report_identity_invalid", checks + ["report_identity_invalid"]) unless report_sha.match?(/\A[0-9a-f]{64}\z/)
    return receipt(scope, bytes, options, "invalid_review", "blocking_findings_open", checks + ["blocking_findings_open"]) unless options.fetch(:critical_open).zero? && options.fetch(:high_open).zero?
    checks += %w[reviewer_independent report_identity_present blocking_findings_closed]
  end
  return receipt(scope, bytes, options, "manual_review_required", "attributable_independent_report_not_recorded", checks + ["manual_review_required"]) if scope.fetch("status") == "manual_review_required"
  receipt(scope, bytes, options, "review_admitted", "exact_independent_review_recorded", checks + ["review_record_exact"])
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "scope unavailable: #{error.class}"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.on("--scope FILE") { |value| options[:scope] = value }
  arguments.on("--scope-available VALUE", %w[yes no]) { |value| options[:scope_available] = value == "yes" }
  arguments.on("--report-available VALUE", %w[yes no]) { |value| options[:report_available] = value == "yes" }
  arguments.on("--reviewer-independent VALUE", %w[yes no]) { |value| options[:reviewer_independent] = value == "yes" }
  arguments.on("--target-version VALUE") { |value| options[:target_version] = value }
  arguments.on("--product-source-commit VALUE") { |value| options[:product_source_commit] = value }
  arguments.on("--report-sha256 VALUE") { |value| options[:report_sha256] = value }
  arguments.on("--critical-open VALUE", Integer) { |value| options[:critical_open] = value }
  arguments.on("--high-open VALUE", Integer) { |value| options[:high_open] = value }
end

begin
  parser.parse!
  required = %i[scope scope_available report_available reviewer_independent target_version product_source_commit report_sha256 critical_open high_open]
  missing = required.reject { |key| options.key?(key) }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  raise ContractError, "finding counts must be nonnegative" if options[:critical_open].negative? || options[:high_open].negative?
  puts JSON.pretty_generate(assess(options.fetch(:scope), options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
