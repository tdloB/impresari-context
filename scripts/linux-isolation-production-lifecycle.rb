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
  "automatic_repair" => "denied",
}.freeze

PHASE_ORDER = {
  "rootless_user_manager" => %w[clean_install upgrade rollback logout_login cancellation crash_recovery health_withdrawal uninstall],
  "externally_managed" => %w[clean_install upgrade rollback operator_relaunch cancellation crash_recovery health_withdrawal uninstall],
}.freeze

OPERATION_EVIDENCE = {
  "clean_install" => "clean_install_identity_verified",
  "upgrade" => "upgrade_replacement_verified",
  "rollback" => "rollback_identity_restored",
  "logout_login" => "logout_login_reentry_verified",
  "operator_relaunch" => "operator_relaunch_verified",
  "cancellation" => "cancellation_cleanup_verified",
  "crash_recovery" => "crash_recovery_cleanup_verified",
  "health_withdrawal" => "health_withdrawal_verified",
  "uninstall" => "uninstall_cleanup_verified",
}.freeze

SAFE_NEXT_STEPS = {
  "lifecycle_candidate" => "Retain only the source-free lifecycle candidate and proceed to independently hosted package-lifecycle rehearsals; production and analyzers remain closed.",
  "incomplete" => "Complete every profile-specific lifecycle phase with independently bound source-free evidence before advancing the candidate.",
  "lifecycle_failed" => "Withdraw the lifecycle candidate, preserve the failed phase evidence, and repair the package or cleanup behavior without privileged fallback.",
  "withdrawal_failed" => "Withdraw the lifecycle candidate immediately; changed prerequisites must disable the claim without repair, fallback, or background authority.",
  "invalid_contract" => "Reject the observation set and rerun the exact selected-profile lifecycle matrix without changing phase order or evidence semantics.",
}.freeze

EXPECTED_POLICY = {
  "schema_name" => "linux-isolation-production-lifecycle-policy",
  "schema_version" => "1.0.0",
  "policy_id" => "linux-iar-1b-production-lifecycle-v1",
  "decision" => {
    "selected_profiles" => %w[rootless_user_manager externally_managed],
    "administrator_profile" => "deferred",
    "automatic_sudo" => "denied",
    "persistent_service" => "denied",
    "automatic_repair" => "denied",
  },
  "package_scope" => {
    "artifacts" => %w[cli mcp_server structural_worker],
    "service_unit" => "absent",
    "authorization_policy" => "absent",
    "external_operator_contract" => "documentation_only",
  },
  "phase_order" => PHASE_ORDER,
  "clean_state" => {
    "persistent_service" => "absent",
    "privileged_policy" => "absent",
    "stale_cgroup" => "absent",
    "descendants" => "absent",
    "staged_source" => "absent",
  },
  "health" => {
    "check" => "explicit_source_free",
    "changed_prerequisite" => "withdraw",
    "claim_behavior" => "fail_closed",
    "fallback" => "unsupported",
  },
  "claim" => {
    "contract_only" => true,
    "lifecycle_candidate_only" => true,
    "production_admitted" => false,
    "real_analyzer_authorized" => false,
    "release_packaging_authorized" => false,
  },
}.freeze

OBSERVATION_KEYS = %w[
  phase outcome operation_evidence package_identity_verified
  topology_revalidated claim_withdrawn clean_state
].freeze
ROOT_KEYS = %w[
  schema_name schema_version policy_id profile package_artifact_identity
  source_free phases
].freeze
CLEAN_STATE_KEYS = %w[
  persistent_service_absent privileged_policy_absent stale_cgroup_absent
  descendants_absent staged_source_absent
].freeze

class ContractError < StandardError; end

def exact_keys?(value, keys)
  value.is_a?(Hash) && value.keys.sort == keys.sort
end

def sha256?(value)
  value.is_a?(String) && value.match?(/\A[0-9a-f]{64}\z/)
end

def load_json(path, name)
  bytes = File.binread(path)
  [JSON.parse(bytes), bytes]
rescue JSON::ParserError
  raise ContractError, "#{name} is not valid JSON"
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "#{name} unavailable: #{error.class}"
end

def load_policy(path)
  policy, bytes = load_json(path, "policy")
  raise ContractError, "policy has an unsupported shape or authority" unless policy == EXPECTED_POLICY

  [policy, bytes]
end

def load_observations(path)
  observations, bytes = load_json(path, "observations")
  raise ContractError, "observations have an unsupported root shape" unless exact_keys?(observations, ROOT_KEYS)
  valid_root = observations["schema_name"] == "linux-isolation-production-lifecycle-observations" &&
    observations["schema_version"] == "1.0.0" &&
    observations["policy_id"] == "linux-iar-1b-production-lifecycle-v1" &&
    PHASE_ORDER.key?(observations["profile"]) && observations["source_free"] == true &&
    sha256?(observations["package_artifact_identity"]) &&
    observations["phases"].is_a?(Array) && observations["phases"].length == 8
  raise ContractError, "observations have unsupported identity or bounds" unless valid_root

  observations["phases"].each do |phase|
    raise ContractError, "phase observation has an unsupported shape" unless exact_keys?(phase, OBSERVATION_KEYS)
    raise ContractError, "phase name is unsupported" unless OPERATION_EVIDENCE.key?(phase["phase"])
    raise ContractError, "phase outcome is unsupported" unless %w[passed failed not_observed].include?(phase["outcome"])
    raise ContractError, "operation evidence is unsupported" unless OPERATION_EVIDENCE.value?(phase["operation_evidence"])
    raise ContractError, "phase booleans are malformed" unless %w[package_identity_verified topology_revalidated claim_withdrawn].all? { |key| [true, false].include?(phase[key]) }
    clean = phase["clean_state"]
    raise ContractError, "clean-state observation has an unsupported shape" unless exact_keys?(clean, CLEAN_STATE_KEYS)
    raise ContractError, "clean-state observation is malformed" unless clean.values.all? { |value| [true, false].include?(value) }
  end

  [observations, bytes]
end

def receipt(policy_bytes, observation_bytes, observations, status, reason, checks, failed_phase, evaluated)
  health = observations.fetch("phases").find { |phase| phase["phase"] == "health_withdrawal" }
  health_verified = !health.nil? && health["outcome"] == "passed" && health["claim_withdrawn"] == true &&
    health["topology_revalidated"] == false && health["clean_state"].values.all?
  {
    "schema_name" => "linux-isolation-production-lifecycle-receipt",
    "schema_version" => "1.0.0",
    "policy_id" => "linux-iar-1b-production-lifecycle-v1",
    "policy_identity" => Digest::SHA256.hexdigest(policy_bytes),
    "observation_identity" => Digest::SHA256.hexdigest(observation_bytes),
    "profile" => observations.fetch("profile"),
    "status" => status,
    "reason_code" => reason,
    "evaluated_phases" => evaluated,
    "failed_phase" => failed_phase,
    "checks" => checks,
    "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
    "lifecycle_candidate_active" => status == "lifecycle_candidate",
    "health_withdrawal_verified" => health_verified,
    "production_admitted" => false,
    "real_analyzer_authorized" => false,
    "release_packaging_authorized" => false,
    "privileged_installation_authorized" => false,
    "persistent_service_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(policy_bytes, observation_bytes, observations)
  checks = %w[policy_valid observations_bounded authority_closed]
  profile = observations.fetch("profile")
  expected_phases = PHASE_ORDER.fetch(profile)
  actual_phases = observations.fetch("phases").map { |phase| phase.fetch("phase") }
  unless actual_phases == expected_phases
    return receipt(policy_bytes, observation_bytes, observations, "invalid_contract", "phase_order_invalid", checks + ["phase_order_invalid"], "none", [])
  end
  checks << "phase_order_valid"

  evaluated = []
  observations.fetch("phases").each do |phase|
    name = phase.fetch("phase")
    unless phase.fetch("operation_evidence") == OPERATION_EVIDENCE.fetch(name)
      return receipt(policy_bytes, observation_bytes, observations, "invalid_contract", "operation_evidence_mismatch", checks + ["operation_evidence_mismatch"], name, evaluated)
    end
    if phase.fetch("outcome") == "not_observed"
      zeroed = !phase.fetch("package_identity_verified") && !phase.fetch("topology_revalidated") &&
        !phase.fetch("claim_withdrawn") && phase.fetch("clean_state").values.none?
      unless zeroed
        return receipt(policy_bytes, observation_bytes, observations, "invalid_contract", "unobserved_phase_claimed_evidence", checks + ["unobserved_phase_claimed_evidence"], name, evaluated)
      end
      next
    end

    evaluated << name
    clean = phase.fetch("clean_state").values.all?
    identity = phase.fetch("package_identity_verified")
    if name == "health_withdrawal"
      withdrawal = phase.fetch("outcome") == "passed" && identity && !phase.fetch("topology_revalidated") &&
        phase.fetch("claim_withdrawn") && clean
      unless withdrawal
        return receipt(policy_bytes, observation_bytes, observations, "withdrawal_failed", "health_withdrawal_not_proven", checks + ["health_withdrawal_failed"], name, evaluated)
      end
      checks << "health_withdrawal_passed"
      next
    end

    passed = phase.fetch("outcome") == "passed" && identity && phase.fetch("topology_revalidated") &&
      !phase.fetch("claim_withdrawn") && clean
    unless passed
      return receipt(policy_bytes, observation_bytes, observations, "lifecycle_failed", "#{name}_not_proven", checks + ["#{name}_failed"], name, evaluated)
    end
    checks << "#{name}_passed"
  end

  unless evaluated == expected_phases
    missing = expected_phases - evaluated
    return receipt(policy_bytes, observation_bytes, observations, "incomplete", "lifecycle_evidence_incomplete", checks + missing.map { |phase| "#{phase}_not_observed" }, missing.first, evaluated)
  end

  receipt(policy_bytes, observation_bytes, observations, "lifecycle_candidate", "complete_source_free_matrix_passed", checks + ["complete_matrix_passed"], "none", evaluated)
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.banner = "Usage: ruby scripts/linux-isolation-production-lifecycle.rb --policy FILE --observations FILE"
  arguments.on("--policy FILE") { |value| options[:policy] = value }
  arguments.on("--observations FILE") { |value| options[:observations] = value }
end

begin
  parser.parse!
  missing = %i[policy observations].reject { |key| options.key?(key) && !options[key].empty? }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  _policy, policy_bytes = load_policy(options.fetch(:policy))
  observations, observation_bytes = load_observations(options.fetch(:observations))
  puts JSON.pretty_generate(assess(policy_bytes, observation_bytes, observations))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
