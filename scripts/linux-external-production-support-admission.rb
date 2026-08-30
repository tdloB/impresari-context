#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "date"
require "digest"
require "json"
require "optparse"

PINNED_MANIFEST_SHA256 = "1b0d35034d7020629a65e4e8358fae850c94f03644a87850b4bede8a4926e05d"
AUTHORITY = %w[source_read source_write host_discovery process_execution network credential_access privilege_use service_mutation background_monitoring analyzer_execution].to_h { |key| [key, "denied"] }.freeze
STATUSES = %w[release_pending compatible_supported stale_evidence changed missing_evidence unsupported unavailable].freeze

class ContractError < StandardError; end

def exact_keys!(value, keys, description)
  raise ContractError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && value.keys.sort == keys.sort
end

def date(value, description)
  Date.iso8601(value)
rescue Date::Error
  raise ContractError, "#{description} must be an ISO date"
end

def load_manifest(path)
  bytes = File.binread(path)
  raise ContractError, "manifest identity is not the tracked admission" unless Digest::SHA256.hexdigest(bytes) == PINNED_MANIFEST_SHA256
  manifest = JSON.parse(bytes)
  exact_keys!(manifest, %w[schema_name schema_version manifest_id profile support_surface target evidence release claim withdraw_on safe_next_steps], "manifest")
  raise ContractError, "manifest contract is unsupported" unless manifest.values_at("schema_name", "schema_version", "manifest_id", "profile", "support_surface") == [
    "linux-external-production-support-manifest", "1.0.0", "linux-external-github-hosted-ubuntu-24-04-x86-64-v1", "externally_managed", "github_actions_hosted"
  ]
  exact_keys!(manifest.fetch("target"), %w[runner_label runner_image_version os_release kernel_release architecture landlock_abi], "target")
  exact_keys!(manifest.fetch("evidence"), %w[run_id job_id observed_at fresh_through source_commit candidate_archive_sha256 composition_receipt_sha256], "evidence")
  exact_keys!(manifest.fetch("claim"), %w[level exact_scope_only broad_linux_support production_admitted real_analyzer_authorized], "claim")
  raise ContractError, "manifest claim overreaches" unless manifest.fetch("claim") == {
    "level" => "IAR-1B", "exact_scope_only" => true, "broad_linux_support" => false,
    "production_admitted" => false, "real_analyzer_authorized" => false,
  }
  raise ContractError, "evidence dates are inverted" if date(manifest.dig("evidence", "observed_at"), "observed-at") > date(manifest.dig("evidence", "fresh_through"), "fresh-through")
  release = manifest.fetch("release")
  raise ContractError, "tracked release gate is not pending" unless release["status"] == "pending_publication" && release["baseline_version"] == "0.1.0" && release["candidate_project_version"] == "0.1.0"
  exact_keys!(release, %w[status baseline_version candidate_project_version required_action], "release")
  safe_steps = manifest.fetch("safe_next_steps")
  exact_keys!(safe_steps, STATUSES, "safe next steps")
  raise ContractError, "safe next step is malformed" unless safe_steps.values.all? { |value| value.is_a?(String) && !value.empty? && value.bytesize <= 300 }
  [manifest, bytes]
rescue JSON::ParserError
  raise ContractError, "manifest is not valid JSON"
end

def receipt(manifest, bytes, options, status, reason, checks)
  {
    "schema_name" => "linux-external-production-support-receipt",
    "schema_version" => "1.0.0",
    "status" => status,
    "reason_code" => reason,
    "manifest_id" => manifest.fetch("manifest_id"),
    "profile" => "externally_managed",
    "support_surface" => "github_actions_hosted",
    "observed" => {
      "target_available" => options.fetch(:target_available),
      "evidence_available" => options.fetch(:evidence_available),
      "release_available" => options.fetch(:release_available),
      "runner_label" => options.fetch(:runner_label),
      "runner_image_version" => options.fetch(:runner_image_version),
      "os_release" => options.fetch(:os_release),
      "kernel_release" => options.fetch(:kernel_release),
      "architecture" => options.fetch(:architecture),
      "landlock_abi" => options.fetch(:landlock_abi),
      "release_version" => options.fetch(:release_version),
      "release_tag" => options.fetch(:release_tag),
      "release_archive_sha256" => options.fetch(:release_archive_sha256),
      "as_of" => options.fetch(:as_of),
    },
    "manifest_identity" => Digest::SHA256.hexdigest(bytes),
    "evidence_identity" => manifest.dig("evidence", "composition_receipt_sha256"),
    "checks" => checks,
    "safe_next_step" => manifest.fetch("safe_next_steps").fetch(status),
    "support_claim_active" => status == "compatible_supported",
    "production_admitted" => status == "compatible_supported",
    "real_analyzer_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(path, options)
  manifest, bytes = load_manifest(path)
  checks = %w[manifest_valid manifest_identity_pinned profile_exact support_surface_exact]
  return receipt(manifest, bytes, options, "unsupported", "support_surface_unrecorded", checks + ["support_surface_unrecorded"]) unless options.fetch(:support_surface) == manifest.fetch("support_surface")
  return receipt(manifest, bytes, options, "unavailable", "target_unavailable", checks + ["target_unavailable"]) unless options.fetch(:target_available)
  checks << "target_available"
  return receipt(manifest, bytes, options, "missing_evidence", "evidence_missing", checks + ["evidence_missing"]) unless options.fetch(:evidence_available)
  checks << "evidence_available"
  return receipt(manifest, bytes, options, "stale_evidence", "evidence_expired", checks + ["evidence_stale"]) if date(options.fetch(:as_of), "as-of") > date(manifest.dig("evidence", "fresh_through"), "fresh-through")
  checks << "evidence_current"
  mapping = {
    runner_label: "runner_label", runner_image_version: "runner_image_version", os_release: "os_release",
    kernel_release: "kernel_release", architecture: "architecture", landlock_abi: "landlock_abi",
  }
  changed = mapping.each_with_object([]) do |(option_key, manifest_key), fields|
    fields << manifest_key unless options.fetch(option_key) == manifest.dig("target", manifest_key)
  end
  return receipt(manifest, bytes, options, "changed", "target_identity_changed", checks + changed.map { |field| "#{field}_changed" }) unless changed.empty?
  checks << "target_identity_exact"
  release = manifest.fetch("release")
  return receipt(manifest, bytes, options, "release_pending", "immutable_release_not_published", checks + ["release_publication_pending"]) if release.fetch("status") == "pending_publication"
  return receipt(manifest, bytes, options, "missing_evidence", "release_missing", checks + ["release_missing"]) unless options.fetch(:release_available)
  release_changed = {
    release_version: "version", release_tag: "tag", release_archive_sha256: "archive_sha256",
  }.each_with_object([]) do |(option_key, manifest_key), fields|
    fields << manifest_key unless options.fetch(option_key) == release.fetch(manifest_key)
  end
  return receipt(manifest, bytes, options, "changed", "release_identity_changed", checks + release_changed.map { |field| "release_#{field}_changed" }) unless release_changed.empty?
  receipt(manifest, bytes, options, "compatible_supported", "exact_published_release_current", checks + %w[release_available release_identity_exact])
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "manifest unavailable: #{error.class}"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.banner = "Usage: ruby scripts/linux-external-production-support-admission.rb [options]"
  arguments.on("--manifest FILE") { |value| options[:manifest] = value }
  arguments.on("--support-surface VALUE") { |value| options[:support_surface] = value }
  arguments.on("--target-available VALUE", %w[yes no]) { |value| options[:target_available] = value == "yes" }
  arguments.on("--evidence-available VALUE", %w[yes no]) { |value| options[:evidence_available] = value == "yes" }
  arguments.on("--release-available VALUE", %w[yes no]) { |value| options[:release_available] = value == "yes" }
  arguments.on("--runner-label VALUE") { |value| options[:runner_label] = value }
  arguments.on("--runner-image-version VALUE") { |value| options[:runner_image_version] = value }
  arguments.on("--os-release VALUE") { |value| options[:os_release] = value }
  arguments.on("--kernel-release VALUE") { |value| options[:kernel_release] = value }
  arguments.on("--arch VALUE") { |value| options[:architecture] = value }
  arguments.on("--landlock-abi VALUE") { |value| options[:landlock_abi] = value }
  arguments.on("--release-version VALUE") { |value| options[:release_version] = value }
  arguments.on("--release-tag VALUE") { |value| options[:release_tag] = value }
  arguments.on("--release-archive-sha256 VALUE") { |value| options[:release_archive_sha256] = value }
  arguments.on("--as-of DATE") { |value| options[:as_of] = value }
end

begin
  parser.parse!
  required = %i[manifest support_surface target_available evidence_available release_available runner_label runner_image_version os_release kernel_release architecture landlock_abi release_version release_tag release_archive_sha256 as_of]
  missing = required.reject { |key| options.key?(key) }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  puts JSON.pretty_generate(assess(options.fetch(:manifest), options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
