#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "optparse"

AUTHORITY = {
  "source_read" => "denied",
  "source_write" => "denied",
  "host_discovery" => "denied",
  "process_execution" => "denied",
  "network" => "denied",
  "credential_access" => "denied",
  "privilege_use" => "denied",
  "service_mutation" => "denied",
  "background_monitoring" => "denied",
  "analyzer_execution" => "denied",
}.freeze

REQUIRED_CONTROLLERS = %w[cpu memory pids].freeze
SELECTED_PROFILES = %w[rootless_user_manager externally_managed].freeze
PROFILES = (SELECTED_PROFILES + ["administrator_provisioned"]).freeze
SAFE_NEXT_STEPS = {
  "feasible_candidate" => "Retain only the source-free topology candidate; run the complete synthetic confinement corpus before any production review.",
  "unavailable" => "Report the selected topology unavailable without starting services, requesting privilege, or weakening the confinement claim.",
  "unsupported" => "Report unsupported; do not install a privileged service or fall back to application-only confinement while claiming IAR-1B.",
  "insufficient_delegation" => "Report the failed delegation prerequisite and keep production and analyzer execution closed.",
  "invalid_contract" => "Reject the launch contract; accept no raw path or authority beyond the selected profile.",
}.freeze

EXPECTED_POLICY = {
  "schema_name" => "linux-isolation-production-topology-policy",
  "schema_version" => "1.0.0",
  "policy_id" => "linux-iar-1b-production-topology-v1",
  "backend" => "landlock-seccomp-cgroup-v2",
  "decision" => {
    "default_profile" => "rootless_user_manager",
    "selected_profiles" => SELECTED_PROFILES,
    "administrator_profile" => "deferred",
    "automatic_sudo" => "denied",
    "privileged_daemon" => "denied",
  },
  "profiles" => {
    "rootless_user_manager" => {
      "launch_contract" => "transient_user_service_or_scope",
      "parent_manager" => "existing_systemd_user_manager",
      "privileged_installation" => false,
      "raw_path_authority" => false,
    },
    "externally_managed" => {
      "launch_contract" => "inherited_directory_fd",
      "provisioner" => "administrator_or_orchestrator",
      "privileged_installation_by_impresari" => false,
      "raw_path_authority" => false,
    },
  },
  "fallback" => "unsupported",
  "reconsideration" => {
    "unsupported_attempt_rate_gt" => "0.10",
    "recoverable_share_gte" => "0.50",
    "evidence_kind" => "documented_aggregate_preflight",
    "automatic_transition" => false,
  },
  "claim" => {
    "source_free_feasibility_only" => true,
    "production_admitted" => false,
    "real_analyzer_authorized" => false,
  },
}.freeze

class ContractError < StandardError; end

def load_policy(path)
  bytes = File.binread(path)
  policy = JSON.parse(bytes)
  raise ContractError, "policy has an unsupported shape or authority" unless policy == EXPECTED_POLICY

  [policy, bytes]
rescue JSON::ParserError
  raise ContractError, "policy is not valid JSON"
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "policy unavailable: #{error.class}"
end

def receipt(policy, policy_bytes, options, status, reason, checks)
  {
    "schema_name" => "linux-isolation-production-topology-receipt",
    "schema_version" => "1.0.0",
    "policy_id" => policy.fetch("policy_id"),
    "policy_identity" => Digest::SHA256.hexdigest(policy_bytes),
    "profile" => options.fetch(:profile),
    "status" => status,
    "reason_code" => reason,
    "observed" => {
      "cgroup_mode" => options.fetch(:cgroup_mode),
      "user_manager_available" => options.fetch(:user_manager_available),
      "delegation_marker" => options.fetch(:delegation_marker),
      "controllers" => options.fetch(:controllers).sort,
      "process_contained" => options.fetch(:process_contained),
      "descendant_ownership_exclusive" => options.fetch(:descendant_ownership_exclusive),
      "synthetic_child_cycle" => options.fetch(:synthetic_child_cycle),
      "external_capability" => options.fetch(:external_capability),
      "external_owner_verified" => options.fetch(:external_owner_verified),
      "external_containment_verified" => options.fetch(:external_containment_verified),
    },
    "checks" => checks,
    "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
    "feasibility_claim_active" => status == "feasible_candidate",
    "production_admitted" => false,
    "real_analyzer_authorized" => false,
    "privileged_installation_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(policy, policy_bytes, options)
  checks = ["policy_valid", "authority_closed"]
  profile = options.fetch(:profile)
  if profile == "administrator_provisioned"
    return receipt(policy, policy_bytes, options, "unsupported", "administrator_profile_deferred", checks + ["administrator_profile_deferred"])
  end
  raise ContractError, "profile is unsupported" unless SELECTED_PROFILES.include?(profile)

  if profile == "rootless_user_manager"
    unless options.fetch(:external_capability) == "none" &&
        !options.fetch(:external_owner_verified) &&
        !options.fetch(:external_containment_verified)
      return receipt(policy, policy_bytes, options, "invalid_contract", "rootless_external_authority_rejected", checks + ["external_authority_rejected"])
    end
    checks << "rootless_contract_valid"
  else
    capability = options.fetch(:external_capability)
    return receipt(policy, policy_bytes, options, "invalid_contract", "raw_path_rejected", checks + ["raw_path_rejected"]) if capability == "raw_path"
    return receipt(policy, policy_bytes, options, "unavailable", "external_capability_missing", checks + ["external_capability_missing"]) if capability == "none"
    checks << "directory_fd_received"
    unless options.fetch(:external_owner_verified) && options.fetch(:external_containment_verified)
      return receipt(policy, policy_bytes, options, "insufficient_delegation", "external_boundary_unverified", checks + ["external_boundary_unverified"])
    end
    checks << "external_boundary_verified"
  end

  cgroup_mode = options.fetch(:cgroup_mode)
  return receipt(policy, policy_bytes, options, "unavailable", "cgroup_interface_unavailable", checks + ["cgroup_interface_unavailable"]) if cgroup_mode == "unavailable"
  return receipt(policy, policy_bytes, options, "unsupported", "unified_cgroup_v2_required", checks + ["legacy_or_hybrid_rejected"]) unless cgroup_mode == "unified_v2"
  checks << "unified_cgroup_v2"

  if profile == "rootless_user_manager" && !options.fetch(:user_manager_available)
    return receipt(policy, policy_bytes, options, "unavailable", "user_manager_unavailable", checks + ["user_manager_unavailable"])
  end
  checks << "parent_manager_available"

  unless options.fetch(:delegation_marker)
    return receipt(policy, policy_bytes, options, "insufficient_delegation", "delegation_marker_missing", checks + ["delegation_marker_missing"])
  end
  checks << "delegation_marker_verified"

  missing = REQUIRED_CONTROLLERS - options.fetch(:controllers)
  unless missing.empty?
    return receipt(policy, policy_bytes, options, "insufficient_delegation", "required_controller_missing", checks + missing.map { |controller| "#{controller}_controller_missing" })
  end
  checks << "required_controllers_available"

  unless options.fetch(:process_contained)
    return receipt(policy, policy_bytes, options, "insufficient_delegation", "process_not_contained", checks + ["process_not_contained"])
  end
  checks << "process_contained"

  unless options.fetch(:descendant_ownership_exclusive)
    return receipt(policy, policy_bytes, options, "insufficient_delegation", "descendant_ownership_not_exclusive", checks + ["descendant_ownership_not_exclusive"])
  end
  checks << "descendant_ownership_exclusive"

  unless options.fetch(:synthetic_child_cycle)
    return receipt(policy, policy_bytes, options, "insufficient_delegation", "synthetic_child_cycle_failed", checks + ["synthetic_child_cycle_failed"])
  end

  receipt(policy, policy_bytes, options, "feasible_candidate", "selected_topology_prerequisites_met", checks + ["synthetic_child_cycle_passed"])
end

def boolean(value)
  value == "yes"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.banner = "Usage: ruby scripts/linux-isolation-topology-feasibility.rb --policy FILE --profile PROFILE --cgroup-mode MODE --user-manager yes|no --delegation-marker yes|no --controllers LIST --process-contained yes|no --exclusive-descendants yes|no --synthetic-child-cycle yes|no --external-capability KIND --external-owner-verified yes|no --external-containment-verified yes|no"
  arguments.on("--policy FILE") { |value| options[:policy] = value }
  arguments.on("--profile PROFILE", PROFILES) { |value| options[:profile] = value }
  arguments.on("--cgroup-mode MODE", %w[unified_v2 legacy_or_hybrid unavailable]) { |value| options[:cgroup_mode] = value }
  arguments.on("--user-manager VALUE", %w[yes no]) { |value| options[:user_manager_available] = boolean(value) }
  arguments.on("--delegation-marker VALUE", %w[yes no]) { |value| options[:delegation_marker] = boolean(value) }
  arguments.on("--controllers LIST") { |value| options[:controllers] = value.empty? ? [] : value.split(",") }
  arguments.on("--process-contained VALUE", %w[yes no]) { |value| options[:process_contained] = boolean(value) }
  arguments.on("--exclusive-descendants VALUE", %w[yes no]) { |value| options[:descendant_ownership_exclusive] = boolean(value) }
  arguments.on("--synthetic-child-cycle VALUE", %w[yes no]) { |value| options[:synthetic_child_cycle] = boolean(value) }
  arguments.on("--external-capability KIND", %w[none inherited_directory_fd raw_path]) { |value| options[:external_capability] = value }
  arguments.on("--external-owner-verified VALUE", %w[yes no]) { |value| options[:external_owner_verified] = boolean(value) }
  arguments.on("--external-containment-verified VALUE", %w[yes no]) { |value| options[:external_containment_verified] = boolean(value) }
end

begin
  parser.parse!
  required = %i[policy profile cgroup_mode user_manager_available delegation_marker controllers process_contained descendant_ownership_exclusive synthetic_child_cycle external_capability external_owner_verified external_containment_verified]
  missing = required.reject { |key| options.key?(key) && options[key] != "" }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  unknown_controllers = options.fetch(:controllers) - REQUIRED_CONTROLLERS
  raise ContractError, "controller list is unsupported" unless unknown_controllers.empty? && options.fetch(:controllers).uniq == options.fetch(:controllers)

  policy, policy_bytes = load_policy(options.fetch(:policy))
  puts JSON.pretty_generate(assess(policy, policy_bytes, options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
