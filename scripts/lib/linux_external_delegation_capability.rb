# SPDX-License-Identifier: Apache-2.0

module LinuxExternalDelegationCapability
  FIXED_DESCRIPTOR = 3
  POLICY_IDENTITY = "03ff04052dae6f7990805011fe454774c3f2ba209a9cf0eea083097eacb7bac4"

  module_function

  def build(received:, directory:, close_on_exec:, raw_path_received: false, descriptor_slot: FIXED_DESCRIPTOR)
    status, reason = classify(
      received: received,
      directory: directory,
      close_on_exec: close_on_exec,
      raw_path_received: raw_path_received,
      descriptor_slot: descriptor_slot,
    )
    ready = status == "transport_ready_for_host_rehearsal"
    {
      "schema_name" => "linux-external-delegation-capability",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-topology-v1",
      "policy_identity" => POLICY_IDENTITY,
      "profile" => "externally_managed",
      "status" => status,
      "reason_code" => reason,
      "capability" => {
        "transport" => "inherited_directory_fd",
        "descriptor_slot" => descriptor_slot,
        "descriptor_slot_configurable" => false,
        "received" => received,
        "directory_verified" => directory,
        "close_on_exec_set" => close_on_exec,
        "raw_path_received" => raw_path_received,
        "descriptor_identity_recorded" => false,
        "raw_cgroup_path_recorded" => false,
      },
      "host_revalidation" => {
        "required_before_mutation" => true,
        "executed" => false,
        "unified_cgroup_v2" => "not_observed",
        "owner_verified" => "not_observed",
        "process_contained" => "not_observed",
        "exclusive_descendants" => "not_observed",
        "required_controllers" => "not_observed",
      },
      "safe_next_step" => safe_next_step(status),
      "transport_contract_active" => ready,
      "os_confined" => false,
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "authority" => {
        "workspace_source_read" => "denied",
        "source_write" => "denied",
        "process_execution" => "fixed_source_free_transport_probe_only",
        "cgroup_mutation" => "denied",
        "service_mutation" => "denied",
        "network" => "denied",
        "credential_access" => "denied",
        "privilege_use" => "denied",
        "persistent_service" => "denied",
        "analyzer_execution" => "denied",
      },
    }
  end

  def classify(received:, directory:, close_on_exec:, raw_path_received:, descriptor_slot:)
    return ["invalid_contract", "raw_path_rejected"] if raw_path_received
    return ["invalid_contract", "descriptor_slot_rejected"] unless descriptor_slot == FIXED_DESCRIPTOR
    return ["unavailable", "inherited_descriptor_missing"] unless received
    return ["invalid_contract", "inherited_descriptor_not_directory"] unless directory
    return ["invalid_contract", "descriptor_leakage_not_closed"] unless close_on_exec

    ["transport_ready_for_host_rehearsal", "fixed_inherited_directory_fd_verified"]
  end

  def safe_next_step(status)
    if status == "transport_ready_for_host_rehearsal"
      "Revalidate cgroup v2, ownership, containment, controllers, and exclusive descendants on an operator-provided synthetic host before any mutation."
    else
      "Reject the external launch without accepting a path, changing privilege, mutating cgroups, or weakening the confinement claim."
    end
  end
end
