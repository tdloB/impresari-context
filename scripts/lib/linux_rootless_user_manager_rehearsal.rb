# frozen_string_literal: true
# SPDX-License-Identifier: Apache-2.0

require "digest"

module LinuxRootlessUserManagerRehearsal
  AUTHORITY = {
    "policy_read" => "fixed_bundled_policy",
    "host_metadata_read" => "fixed_linux_platform_files",
    "bundled_synthetic_source_read" => "fixed_rehearsal_only",
    "workspace_source_read" => "denied",
    "source_write" => "synthetic_target_only",
    "process_execution" => "fixed_synthetic_rehearsal_only",
    "user_manager_request" => "one_transient_unit_only",
    "service_mutation" => "transient_user_unit_only",
    "cgroup_mutation" => "delegated_descendants_only",
    "network" => "denied",
    "credential_access" => "denied",
    "privilege_use" => "denied",
    "persistent_service" => "denied",
    "analyzer_execution" => "denied",
  }.freeze
  REQUIRED_PREFLIGHT = "ready_for_synthetic_rehearsal"
  REQUIRED_CONTROLLERS = %w[cpu memory pids].freeze
  REQUIRED_COMPOSITE_CHECKS = %w[
    atomic_cgroup_placement resource_profile_applied no_new_privs_effective
    landlock_read_only_input external_filesystem_denial credential_denial
    device_denial network_denial unrelated_descriptors_closed descendant_denial
    zero_writable_filesystem cpu_limit memory_limit process_count_limit
    exact_cgroup_kill cgroup_empty_after_job bounded_output timeout crash_relaunch
    cleanup cross_job_isolation
  ].freeze
  SAFE_NEXT_STEPS = {
    "candidate_passed" => "Retain this exact-host rootless synthetic candidate; do not admit production or run a real analyzer until release and maintenance gates pass.",
    "skipped_preflight" => "Report the rootless profile unsupported on this host without creating a unit, requesting privilege, or using a privileged fallback.",
    "launch_failed" => "Withdraw the rootless candidate for this host and inspect only bounded service metadata; do not retry with sudo or a system service.",
    "composite_failed" => "Withdraw the rootless candidate for this host; keep production and real analyzers closed until the complete synthetic corpus passes.",
    "cleanup_failed" => "Withdraw the rootless candidate and treat the user-unit lifecycle as failed until collection is independently verified.",
  }.freeze

  module_function

  def valid_composite?(composite)
    return false unless composite.is_a?(Hash)
    return false unless composite["schema_name"] == "linux-isolation-feasibility"
    return false unless composite["result"] == "candidate_passed"
    return false unless composite["os_confined"] == true
    return false unless composite["production_admitted"] == false
    return false unless composite["source_retained"] == false
    return false unless composite["authority_added"] == false
    return false unless composite["limitations"] == ["single-host-evidence", "synthetic-probe-no-analysis"]

    preflight = composite["preflight"]
    checks = composite["checks"]
    preflight.is_a?(Hash) && preflight.values.all? { |value| value == true } &&
      checks.is_a?(Hash) && REQUIRED_COMPOSITE_CHECKS.all? { |name| checks[name] == true }
  end

  def architecture(value)
    return value if %w[x86_64 aarch64].include?(value)

    "other"
  end

  def build(policy_identity:, preflight_bytes:, preflight:, architecture:, attempted:, created:, collected:, composite_bytes: nil, composite: nil)
    preflight_status = preflight.fetch("status")
    if preflight_status != REQUIRED_PREFLIGHT
      status = "skipped_preflight"
      reason = "preflight_#{preflight_status}"
    elsif !attempted || !created
      status = "launch_failed"
      reason = "transient_user_unit_launch_failed"
    elsif !valid_composite?(composite)
      status = "composite_failed"
      reason = "synthetic_composite_failed"
    elsif !collected
      status = "cleanup_failed"
      reason = "transient_user_unit_not_collected"
    else
      status = "candidate_passed"
      reason = "rootless_synthetic_corpus_passed"
    end

    composite_executed = !composite.nil?
    composite_passed = valid_composite?(composite)
    {
      "schema_name" => "linux-rootless-user-manager-rehearsal",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-topology-v1",
      "policy_identity" => policy_identity,
      "profile" => "rootless_user_manager",
      "status" => status,
      "reason_code" => reason,
      "observed_host" => {
        "operating_system" => preflight.dig("observed", "platform"),
        "kernel_release" => preflight.dig("observed", "kernel_release"),
        "architecture" => architecture(architecture),
      },
      "preflight" => {
        "status" => preflight_status,
        "receipt_identity" => Digest::SHA256.hexdigest(preflight_bytes),
        "raw_cgroup_path_recorded" => preflight.dig("observed", "raw_cgroup_path_recorded"),
      },
      "transient_user_unit" => {
        "attempted" => attempted,
        "created" => created,
        "manager" => "existing_user_manager_only",
        "delegated_controllers" => REQUIRED_CONTROLLERS,
        "sudo_used" => false,
        "privileged_service_used" => false,
        "persistent" => false,
        "unit_name_recorded" => false,
        "collected" => collected,
      },
      "composite" => {
        "executed" => composite_executed,
        "result" => composite_executed ? (composite_passed ? "candidate_passed" : "failed") : "not_executed",
        "receipt_identity" => composite_bytes ? Digest::SHA256.hexdigest(composite_bytes) : "not_executed",
        "source_kind" => "original_synthetic_only",
        "real_analyzer_used" => false,
      },
      "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
      "rootless_candidate_active" => status == "candidate_passed",
      "os_confined" => status == "candidate_passed",
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "authority" => AUTHORITY,
    }
  end
end
