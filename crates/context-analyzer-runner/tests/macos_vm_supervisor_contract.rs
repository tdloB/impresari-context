// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Closed-contract tests for ADR-0087's synthetic macOS VM supervisor."]

use context_analyzer_runner::{
    MACOS_VM_CONTROLLER_PROFILE_DIGEST, MACOS_VM_CONTROLLER_PROFILE_ID,
    MACOS_VM_HOST_INTERRUPTION_PROFILE_DIGEST, MACOS_VM_HOST_INTERRUPTION_PROFILE_ID,
    MACOS_VM_RESOURCE_CANARY_PROFILE_DIGEST, MACOS_VM_RESOURCE_CANARY_PROFILE_ID,
    MACOS_VM_SUPERVISOR_PROFILE_DIGEST, MACOS_VM_SUPERVISOR_PROFILE_ID,
    MacOsVmHostInterruptionReceipt, MacOsVmResourceCanaryReceipt, MacOsVmSupervisorAction,
    MacOsVmSupervisorReceipt,
};

#[test]
fn host_interruption_receipt_distinguishes_simulation_from_real_sleep() {
    let receipt = MacOsVmHostInterruptionReceipt {
        schema_name: "macos-local-vm-host-interruption-receipt".into(),
        schema_version: "1.0.0".into(),
        profile_id: MACOS_VM_HOST_INTERRUPTION_PROFILE_ID.into(),
        profile_digest: MACOS_VM_HOST_INTERRUPTION_PROFILE_DIGEST.into(),
        controller_profile_id: MACOS_VM_CONTROLLER_PROFILE_ID.into(),
        controller_profile_digest: MACOS_VM_CONTROLLER_PROFILE_DIGEST.into(),
        controller_digest:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        job_id: "interrupt-contract".into(),
        interruption_source: "synthetic-job-private-trigger".into(),
        sleep_observer_installed: true,
        shared_stop_handler_used: true,
        synthetic_interruption_requested: true,
        virtual_machine_stopped: true,
        controller_reaped: true,
        stale_job_removed: true,
        recovery_job_succeeded: true,
        all_job_state_removed: true,
        real_host_sleep_observed: false,
        vm_confined: false,
        production_admitted: false,
        analyzer_execution: false,
        source_retained: false,
        authority_added: false,
    };
    let value = serde_json::to_value(&receipt).expect("serialize receipt");
    assert_eq!(
        value["interruption_source"],
        "synthetic-job-private-trigger"
    );
    assert_eq!(value["real_host_sleep_observed"], false);
    assert_eq!(value["vm_confined"], false);
    assert_eq!(value["production_admitted"], false);
    assert_eq!(value["analyzer_execution"], false);
    assert!(value.get("workspace_path").is_none());

    let mut expanded = value;
    expanded
        .as_object_mut()
        .expect("receipt object")
        .insert("host_sleep_observed".into(), serde_json::json!(true));
    assert!(serde_json::from_value::<MacOsVmHostInterruptionReceipt>(expanded).is_err());
}

fn receipt(action: MacOsVmSupervisorAction) -> MacOsVmSupervisorReceipt {
    let cancellation = action == MacOsVmSupervisorAction::ExternalCancellation;
    MacOsVmSupervisorReceipt {
        schema_name: "macos-local-vm-supervisor-lifecycle-receipt".into(),
        schema_version: "1.0.0".into(),
        profile_id: MACOS_VM_SUPERVISOR_PROFILE_ID.into(),
        profile_digest: MACOS_VM_SUPERVISOR_PROFILE_DIGEST.into(),
        controller_profile_id: MACOS_VM_CONTROLLER_PROFILE_ID.into(),
        controller_profile_digest: MACOS_VM_CONTROLLER_PROFILE_DIGEST.into(),
        controller_digest:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        job_id: "supervisor-contract".into(),
        action,
        controller_digest_verified_before_launch: true,
        controller_ready: true,
        external_cancellation_requested: cancellation,
        controller_cancellation_verified: cancellation,
        controller_forcibly_terminated: !cancellation,
        controller_reaped: true,
        stale_job_removed: true,
        recovery_job_succeeded: true,
        all_job_state_removed: true,
        vm_confined: false,
        production_admitted: false,
        analyzer_execution: false,
        source_retained: false,
        authority_added: false,
    }
}

#[test]
fn resource_canary_receipt_is_bounded_source_free_evidence() {
    let receipt = MacOsVmResourceCanaryReceipt {
        schema_name: "macos-local-vm-resource-canary-supervisor-receipt".into(),
        schema_version: "1.0.0".into(),
        profile_id: MACOS_VM_RESOURCE_CANARY_PROFILE_ID.into(),
        profile_digest: MACOS_VM_RESOURCE_CANARY_PROFILE_DIGEST.into(),
        controller_digest:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        job_id: "resource-contract".into(),
        kernel_digest: "sha256:8b216f74e7f89def4604adf69e2345437363aff4819101bb1551c9e83cd35cdd"
            .into(),
        initramfs_digest: "sha256:f75a3bc10d569622f84c557e88bbc9ce65a157e7bb410f412c8ab39dedc5c80c"
            .into(),
        input_digest: "sha256:3050d67653f05f1db0dcef073a64f6fc9f9ac2e55c7b1881e7372151b3e4fd99"
            .into(),
        controller_digest_verified_before_launch: true,
        configuration_validated: true,
        cpu_count: "1".into(),
        memory_bytes: "268435456".into(),
        storage_devices: "2".into(),
        network_devices: "0".into(),
        directory_shares: "0".into(),
        host_canary_corpus_created: true,
        host_canary_corpus_unchanged: true,
        attached_device_set_exact: true,
        host_canary_bytes_absent: true,
        host_paths_absent: true,
        host_process_invisible: true,
        memory_pressure_contained: true,
        memory_oom_kills: "1".into(),
        cpu_pressure_bounded: true,
        cpu_usage_usec: "150000".into(),
        cpu_throttled_periods: "15".into(),
        pids_peak: "1".into(),
        job_cgroup_removed: true,
        job_removed: true,
        vm_confined: false,
        production_admitted: false,
        analyzer_execution: false,
        source_retained: false,
        authority_added: false,
    };
    let value = serde_json::to_value(&receipt).expect("serialize receipt");
    assert_eq!(value["vm_confined"], false);
    assert_eq!(value["production_admitted"], false);
    assert_eq!(value["analyzer_execution"], false);
    assert_eq!(value["source_retained"], false);
    assert_eq!(value["authority_added"], false);
    assert!(value.get("workspace_path").is_none());

    let mut expanded = value;
    expanded
        .as_object_mut()
        .expect("receipt object")
        .insert("host_home".into(), serde_json::json!("/Users/example"));
    assert!(serde_json::from_value::<MacOsVmResourceCanaryReceipt>(expanded).is_err());
}

#[test]
fn exact_action_names_are_closed() {
    assert_eq!(
        MacOsVmSupervisorAction::from_name("external-cancellation"),
        Some(MacOsVmSupervisorAction::ExternalCancellation)
    );
    assert_eq!(
        MacOsVmSupervisorAction::from_name("forced-termination-recovery"),
        Some(MacOsVmSupervisorAction::ForcedTerminationRecovery)
    );
    assert_eq!(MacOsVmSupervisorAction::from_name("run-analyzer"), None);
}

#[test]
fn receipts_are_source_free_and_reject_unknown_authority() {
    for action in [
        MacOsVmSupervisorAction::ExternalCancellation,
        MacOsVmSupervisorAction::ForcedTerminationRecovery,
    ] {
        let receipt = receipt(action);
        let value = serde_json::to_value(&receipt).expect("serialize receipt");
        assert_eq!(value["vm_confined"], false);
        assert_eq!(value["production_admitted"], false);
        assert_eq!(value["analyzer_execution"], false);
        assert_eq!(value["source_retained"], false);
        assert_eq!(value["authority_added"], false);
        assert!(value.get("workspace_path").is_none());

        let mut expanded = value;
        expanded
            .as_object_mut()
            .expect("receipt object")
            .insert("workspace_path".into(), serde_json::json!("/untrusted"));
        assert!(serde_json::from_value::<MacOsVmSupervisorReceipt>(expanded).is_err());
    }
}
