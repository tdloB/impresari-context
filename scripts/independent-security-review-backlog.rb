#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "optparse"

PINNED_BACKLOG_SHA256 = "4d1a1a30094ed69e2cb5bd48d6600bcbba310ebf8b0e65b2a319d8af9bc9de88"
PINNED_PREPARED_SCOPE_SHA256 = "98a248d7133c85366a16b0a443dab15f131529d1bc4e3d8587b0adfc7925a45c"
AUTHORITY = %w[source_read source_write network credential_access process_execution tag_creation release_publication risk_acceptance].to_h { |key| [key, "denied"] }.freeze

class ContractError < StandardError; end

def exact_keys!(value, keys, description)
  raise ContractError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && value.keys.sort == keys.sort
end

def load_backlog(path)
  bytes = File.binread(path)
  raise ContractError, "backlog identity is not the tracked scheduling decision" unless Digest::SHA256.hexdigest(bytes) == PINNED_BACKLOG_SHA256
  backlog = JSON.parse(bytes)
  exact_keys!(backlog, %w[schema_name schema_version backlog_id target_version decision decision_date prepared_scope_sha256 prepared_product_source_commit refresh_triggers review_requirement claim safe_next_step], "backlog")
  raise ContractError, "backlog contract is unsupported" unless backlog.values_at("schema_name", "schema_version", "backlog_id", "target_version", "decision") == [
    "independent-security-review-backlog", "1.0.0", "impresari-context-v0-2-0-review-backlog-v1", "0.2.0", "deferred_to_release_candidate"
  ]
  raise ContractError, "prepared scope identity drifted" unless backlog.fetch("prepared_scope_sha256") == PINNED_PREPARED_SCOPE_SHA256
  raise ContractError, "backlog requirement drifted" unless backlog.fetch("review_requirement") == {
    "mandatory_before_tag" => true, "mandatory_before_publication" => true,
    "development_may_continue" => true, "prepared_scope_may_not_admit_changed_source" => true,
  }
  raise ContractError, "backlog claim overreaches" unless backlog.fetch("claim") == {
    "roadmap_development_blocked" => false, "review_gate_satisfied" => false,
    "release_ready" => false, "tag_authorized" => false, "publication_authorized" => false,
    "production_support_admitted" => false, "real_analyzer_authorized" => false,
  }
  [backlog, bytes]
rescue JSON::ParserError
  raise ContractError, "backlog is not valid JSON"
end

def receipt(backlog, bytes, options, status, reason, checks, safe_next_step = nil)
  {
    "schema_name" => "independent-security-review-backlog-receipt",
    "schema_version" => "1.0.0",
    "status" => status,
    "reason_code" => reason,
    "backlog_id" => backlog.fetch("backlog_id"),
    "target_version" => backlog.fetch("target_version"),
    "observed" => {
      "backlog_available" => options.fetch(:backlog_available),
      "prepared_scope_available" => options.fetch(:prepared_scope_available),
      "target_version" => options.fetch(:target_version),
      "current_product_commit" => options.fetch(:current_product_commit),
      "release_requested" => options.fetch(:release_requested),
    },
    "backlog_identity" => Digest::SHA256.hexdigest(bytes),
    "checks" => checks,
    "safe_next_step" => safe_next_step || backlog.fetch("safe_next_step"),
    "roadmap_development_blocked" => false,
    "review_gate_satisfied" => false,
    "release_ready" => false,
    "tag_authorized" => false,
    "publication_authorized" => false,
    "production_support_admitted" => false,
    "real_analyzer_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(options)
  backlog, bytes = load_backlog(options.fetch(:backlog))
  checks = %w[backlog_valid backlog_identity_pinned]
  return receipt(backlog, bytes, options, "unsupported", "target_version_unrecorded", checks + ["target_version_unrecorded"], "Prepare a separately reviewed schedule for the requested version.") unless options.fetch(:target_version) == backlog.fetch("target_version")
  checks << "target_version_exact"
  return receipt(backlog, bytes, options, "missing_evidence", "backlog_unavailable", checks + ["backlog_unavailable"]) unless options.fetch(:backlog_available)
  checks << "backlog_available"
  return receipt(backlog, bytes, options, "missing_evidence", "prepared_scope_unavailable", checks + ["prepared_scope_unavailable"]) unless options.fetch(:prepared_scope_available)
  prepared_scope_bytes = File.binread(options.fetch(:prepared_scope))
  return receipt(backlog, bytes, options, "changed", "prepared_scope_changed", checks + ["prepared_scope_changed"], "Restore the exact prepared scope or record a separately reviewed scheduling decision.") unless Digest::SHA256.hexdigest(prepared_scope_bytes) == backlog.fetch("prepared_scope_sha256")
  checks << "prepared_scope_exact"
  current_commit = options.fetch(:current_product_commit)
  return receipt(backlog, bytes, options, "changed", "current_product_identity_invalid", checks + ["current_product_identity_invalid"]) unless current_commit.match?(/\A[0-9a-f]{40}\z/)
  if options.fetch(:release_requested)
    return receipt(backlog, bytes, options, "review_required_before_release", "independent_review_not_admitted", checks + ["review_required_before_release"], "Freeze the exact release candidate, refresh the scope and brief, and admit the independent review before tagging or publication.")
  end
  if current_commit != backlog.fetch("prepared_product_source_commit")
    return receipt(backlog, bytes, options, "scope_refresh_required", "product_advanced_since_prepared_scope", checks + ["scope_refresh_required"], "Continue roadmap development and refresh the exact review scope only after the final v0.2.0 release candidate is frozen.")
  end
  receipt(backlog, bytes, options, "development_continues", "review_deferred_to_release_candidate", checks + ["development_continues"])
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "review scheduling evidence unavailable: #{error.class}"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.on("--backlog FILE") { |value| options[:backlog] = value }
  arguments.on("--prepared-scope FILE") { |value| options[:prepared_scope] = value }
  arguments.on("--backlog-available VALUE", %w[yes no]) { |value| options[:backlog_available] = value == "yes" }
  arguments.on("--prepared-scope-available VALUE", %w[yes no]) { |value| options[:prepared_scope_available] = value == "yes" }
  arguments.on("--target-version VALUE") { |value| options[:target_version] = value }
  arguments.on("--current-product-commit VALUE") { |value| options[:current_product_commit] = value }
  arguments.on("--release-requested VALUE", %w[yes no]) { |value| options[:release_requested] = value == "yes" }
end

begin
  parser.parse!
  required = %i[backlog prepared_scope backlog_available prepared_scope_available target_version current_product_commit release_requested]
  missing = required.reject { |key| options.key?(key) }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  puts JSON.pretty_generate(assess(options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
