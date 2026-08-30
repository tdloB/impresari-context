# SPDX-License-Identifier: Apache-2.0

require "digest"

module LinuxExternalDelegationLiveRehearsal
  POLICY_IDENTITY = "03ff04052dae6f7990805011fe454774c3f2ba209a9cf0eea083097eacb7bac4"
  REQUIRED_CONTROLLERS = %w[cpu memory pids].freeze

  module_function

  def build(facts:, provisioner_collected:)
    revalidation = facts.fetch("revalidation")
    composite = facts.fetch("composite")
    cleanup = facts.fetch("cleanup")
    status, reason = classify(facts, provisioner_collected)
    candidate = status == "candidate_passed"
    {
      "schema_name" => "linux-external-delegation-live-rehearsal",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-topology-v1",
      "policy_identity" => POLICY_IDENTITY,
      "profile" => "externally_managed",
      "status" => status,
      "reason_code" => reason,
      "observed_host" => facts.fetch("observed_host"),
      "provisioner" => {
        "provider" => "ephemeral_ci_transient_system_service",
        "service_created" => facts.fetch("service_created"),
        "service_collected" => provisioner_collected,
        "operator_privilege_used" => true,
        "impresari_privilege_used" => false,
        "persistent" => false,
        "unit_name_recorded" => false,
      },
      "capability" => facts.fetch("capability"),
      "revalidation" => revalidation,
      "composite" => composite,
      "cleanup" => cleanup,
      "safe_next_step" => safe_next_step(status),
      "external_candidate_active" => candidate,
      "os_confined" => candidate,
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "authority" => {
        "workspace_source_read" => "denied",
        "source_write" => "synthetic_target_only",
        "process_execution" => "fixed_synthetic_composite_only",
        "cgroup_mutation" => "inherited_delegated_descendants_only",
        "service_mutation_by_impresari" => "denied",
        "operator_provisioning" => "one_ephemeral_ci_service_only",
        "network" => "denied",
        "credential_access" => "denied",
        "impresari_privilege_use" => "denied",
        "persistent_service" => "denied",
        "analyzer_execution" => "denied",
      },
    }
  end

  def classify(facts, provisioner_collected)
    capability = facts.fetch("capability")
    revalidation = facts.fetch("revalidation")
    composite = facts.fetch("composite")
    cleanup = facts.fetch("cleanup")
    return ["capability_failed", "fixed_descriptor_contract_failed"] unless capability.values_at("received", "directory_verified", "close_on_exec_set").all? && capability.fetch("raw_path_received") == false
    required = %w[unified_cgroup_v2 owner_verified process_contained exclusive_descendants delegation_writable]
    return ["revalidation_failed", "external_boundary_revalidation_failed"] unless required.all? { |name| revalidation.fetch(name) } && revalidation.fetch("controllers") == REQUIRED_CONTROLLERS
    return ["composite_failed", "synthetic_composite_failed"] unless composite.fetch("executed") && composite.fetch("result") == "candidate_passed"
    return ["cleanup_failed", "delegated_descendant_cleanup_failed"] unless cleanup.fetch("descendants_removed")
    return ["provisioner_cleanup_failed", "ephemeral_service_not_collected"] unless provisioner_collected

    ["candidate_passed", "external_synthetic_corpus_passed"]
  end

  def safe_next_step(status)
    if status == "candidate_passed"
      "Retain this exact-host external synthetic candidate; production still requires frozen operator and package lifecycle evidence."
    else
      "Reject the external profile on this host without a path fallback, privilege request by Impresari, or weaker confinement claim."
    end
  end

  def identity(path)
    Digest::SHA256.file(path).hexdigest
  end
end
