// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Closed-contract tests for the source-free macOS XPC launch handshake."]

use context_analyzer_runner::{
    MACOS_XPC_BUNDLE_IDENTIFIER, MACOS_XPC_HOST_RELATIVE_PATH, MACOS_XPC_PROFILE_DIGEST,
    MACOS_XPC_PROFILE_ID, MACOS_XPC_SERVICE_NAME, MACOS_XPC_SERVICE_RELATIVE_PATH,
    MacOsXpcLaunchPreparation, MacOsXpcLaunchRequest, RunnerErrorCode,
    validate_macos_xpc_launch_preparation, validate_macos_xpc_launch_request,
};

fn request() -> MacOsXpcLaunchRequest {
    MacOsXpcLaunchRequest {
        schema_name: "macos-xpc-launch-request".into(),
        schema_version: "1.0.0".into(),
        request_id: "req_macos_xpc_launch".into(),
        profile_id: MACOS_XPC_PROFILE_ID.into(),
        profile_digest: MACOS_XPC_PROFILE_DIGEST.into(),
        job_digest: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
            .into(),
        artifacts: "1".into(),
        total_artifact_bytes: "4096".into(),
        bundle_identifier: MACOS_XPC_BUNDLE_IDENTIFIER.into(),
        host_relative_path: MACOS_XPC_HOST_RELATIVE_PATH.into(),
        service_name: MACOS_XPC_SERVICE_NAME.into(),
        service_relative_path: MACOS_XPC_SERVICE_RELATIVE_PATH.into(),
        repository_path_present: false,
        arguments_present: false,
        environment_present: false,
        credentials_present: false,
        network_authorized: false,
        analyzer_execution: false,
        authority_added: false,
    }
}

fn preparation(request: &MacOsXpcLaunchRequest) -> MacOsXpcLaunchPreparation {
    MacOsXpcLaunchPreparation {
        schema_name: "macos-xpc-launch-preparation".into(),
        schema_version: "1.0.0".into(),
        request_id: request.request_id.clone(),
        profile_id: request.profile_id.clone(),
        profile_digest: request.profile_digest.clone(),
        job_digest: request.job_digest.clone(),
        host_process_id: "1200".into(),
        service_process_id: "1201".into(),
        bundle_identity_verified: true,
        service_identity_verified: true,
        effective_profile_verified: true,
        network_entitlement_absent: true,
        ready: true,
        os_confined: false,
        production_admitted: false,
        source_retained: false,
        authority_added: false,
    }
}

#[test]
fn exact_source_free_launch_handshake_is_accepted() {
    let request = request();
    let preparation = preparation(&request);
    validate_macos_xpc_launch_request(&request).expect("closed request");
    validate_macos_xpc_launch_preparation(&request, &preparation)
        .expect("matching source-free preparation");
}

#[test]
fn launch_request_rejects_authority_and_noncanonical_accounting() {
    let mut authority = request();
    authority.repository_path_present = true;
    assert_eq!(
        validate_macos_xpc_launch_request(&authority)
            .expect_err("repository authority must fail")
            .code(),
        RunnerErrorCode::InvalidConfiguration
    );

    let mut leading_zero = request();
    leading_zero.total_artifact_bytes = "04096".into();
    assert_eq!(
        validate_macos_xpc_launch_request(&leading_zero)
            .expect_err("noncanonical decimal must fail")
            .code(),
        RunnerErrorCode::InvalidConfiguration
    );

    let mut oversized = request();
    oversized.total_artifact_bytes = "4194305".into();
    assert_eq!(
        validate_macos_xpc_launch_request(&oversized)
            .expect_err("profile overflow must fail")
            .code(),
        RunnerErrorCode::InvalidConfiguration
    );
}

#[test]
fn preparation_rejects_wrong_identity_partial_readiness_and_overclaim() {
    let request = request();
    let mut wrong_job = preparation(&request);
    wrong_job.job_digest =
        "sha256:2222222222222222222222222222222222222222222222222222222222222222".into();
    assert_eq!(
        validate_macos_xpc_launch_preparation(&request, &wrong_job)
            .expect_err("wrong job identity must fail")
            .code(),
        RunnerErrorCode::InvalidOutput
    );

    let mut not_verified = preparation(&request);
    not_verified.effective_profile_verified = false;
    assert_eq!(
        validate_macos_xpc_launch_preparation(&request, &not_verified)
            .expect_err("partial preparation must fail")
            .code(),
        RunnerErrorCode::InvalidOutput
    );

    let mut overclaim = preparation(&request);
    overclaim.os_confined = true;
    assert_eq!(
        validate_macos_xpc_launch_preparation(&request, &overclaim)
            .expect_err("preparation cannot claim IAR-1B admission")
            .code(),
        RunnerErrorCode::InvalidOutput
    );
}

#[test]
fn serde_contract_rejects_unknown_fields() {
    let mut value = serde_json::to_value(request()).expect("serialize request");
    value
        .as_object_mut()
        .expect("request object")
        .insert("workspace_path".into(), serde_json::json!("/untrusted"));
    assert!(
        serde_json::from_value::<MacOsXpcLaunchRequest>(value).is_err(),
        "unknown authority-bearing fields must be rejected"
    );
}
