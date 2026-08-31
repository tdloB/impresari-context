// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Closed-contract tests for ADR-0087's synthetic macOS VM supervisor."]

use context_analyzer_runner::{
    MACOS_VM_CONTROLLER_PROFILE_DIGEST, MACOS_VM_CONTROLLER_PROFILE_ID,
    MACOS_VM_SUPERVISOR_PROFILE_DIGEST, MACOS_VM_SUPERVISOR_PROFILE_ID, MacOsVmSupervisorAction,
    MacOsVmSupervisorReceipt,
};

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
