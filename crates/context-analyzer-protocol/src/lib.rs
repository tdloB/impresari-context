// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Pure closed contracts and synthetic fault behavior for ADR-0074 IAR-0."]
// The wire contracts intentionally preserve independent, explicit negative
// authority claims instead of collapsing them into an ambiguous state enum.
#![allow(clippy::struct_excessive_bools)]

use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Write as _},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CONTRACT_VERSION: &str = "1.0.0";
const PROFILE_ID: &str = "iar-protocol-synthetic-v1";
const PROFILE_DIGEST: &str =
    "sha256:f4e05f583e5af4719703e1178546d625bccb8efde1527143d55e32a9bfcb00b0";
const MAX_REQUEST_BYTES: usize = 262_144;
const MAX_ARTIFACTS: usize = 64;
const MAX_ARTIFACT_BYTES: u64 = 1_048_576;
const MAX_TOTAL_ARTIFACT_BYTES: u64 = 4_194_304;
const MAX_CAPABILITIES: usize = 32;

/// Stable content-free protocol failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolErrorCode {
    /// A closed contract is malformed, non-canonical, or authority-claiming.
    InvalidContract,
    /// A domain-separated identity does not match the exact record.
    IdentityMismatch,
    /// A fixed IAR-0 resource ceiling was exceeded.
    ResourceLimit,
    /// Supplied artifact bytes do not match their exact descriptors.
    ArtifactMismatch,
    /// Canonical serialization failed.
    Serialization,
}

/// Content-free IAR-0 protocol error.
#[derive(Debug)]
pub struct ProtocolError(ProtocolErrorCode);

impl ProtocolError {
    const fn new(code: ProtocolErrorCode) -> Self {
        Self(code)
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> ProtocolErrorCode {
        self.0
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            ProtocolErrorCode::InvalidContract => "invalid closed analyzer protocol contract",
            ProtocolErrorCode::IdentityMismatch => "analyzer protocol identity mismatch",
            ProtocolErrorCode::ResourceLimit => "analyzer protocol resource limit exceeded",
            ProtocolErrorCode::ArtifactMismatch => "synthetic artifact mismatch",
            ProtocolErrorCode::Serialization => "analyzer protocol serialization failed",
        })
    }
}

impl Error for ProtocolError {}

/// Closed IAR-0 synthetic capability declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerCapability {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Stable capability identifier.
    pub capability_id: String,
    /// Capability contract version.
    pub capability_version: String,
    /// Source-free description.
    pub description: String,
    /// Canonically ordered accepted media types.
    pub input_media_types: Vec<String>,
    /// Canonically ordered target platforms.
    pub target_platforms: Vec<String>,
    /// Canonically ordered possible result categories; empty in IAR-0.
    pub result_categories: Vec<String>,
    /// Always true in IAR-0.
    pub synthetic_only: bool,
    /// Always false.
    pub authority_added: bool,
}

/// Closed analyzer identity admitted only for the synthetic IAR-0 model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerExecutionManifest {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated exact manifest identity.
    pub manifest_id: String,
    /// Stable analyzer ID.
    pub analyzer_id: String,
    /// Exact analyzer version.
    pub analyzer_version: String,
    /// Source-free publisher statement; not cryptographic identity.
    pub publisher: String,
    /// Exact synthetic executable identity.
    pub executable_digest: String,
    /// Exact synthetic ruleset identity.
    pub ruleset_digest: String,
    /// Canonically ordered capabilities.
    pub capability_ids: Vec<String>,
    /// Canonically ordered supported hosts.
    pub host_platforms: Vec<String>,
    /// Canonically ordered artifact target platforms.
    pub target_platforms: Vec<String>,
    /// Always `none`.
    pub network: String,
    /// Always `deterministic`.
    pub determinism: String,
    /// Exact result contract name and version.
    pub result_schema: String,
    /// IAR-0 manifests must not be revoked.
    pub revoked: bool,
    /// Always true in IAR-0.
    pub synthetic_only: bool,
    /// Always false.
    pub repository_supplied: bool,
    /// Always false.
    pub authority_added: bool,
}

/// One path-free content-addressed artifact descriptor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDescriptor {
    /// Exact SHA-256 of supplied bytes.
    pub artifact_hash: String,
    /// Canonical unsigned decimal byte length.
    pub bytes: String,
    /// Closed opaque media type identifier.
    pub media_type: String,
    /// Closed target platform.
    pub target_platform: String,
}

/// Closed request for the pure IAR-0 synthetic model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerRunnerRequest {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Caller-owned request identifier.
    pub request_id: String,
    /// Canonical operation time.
    pub occurred_at: String,
    /// Canonical deadline.
    pub deadline_at: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact assessment-plan identity.
    pub assessment_plan_id: String,
    /// Exact policy identity.
    pub policy_id: String,
    /// Exact admitted manifest identity.
    pub manifest_id: String,
    /// Canonically ordered requested capabilities.
    pub capability_ids: Vec<String>,
    /// Canonically ordered path-free artifacts.
    pub artifacts: Vec<ArtifactDescriptor>,
    /// Fixed IAR-0 profile ID.
    pub resource_profile_id: String,
    /// Exact fixed profile bytes digest.
    pub resource_profile_digest: String,
    /// Closed deterministic synthetic behavior.
    pub synthetic_behavior: String,
    /// Always false.
    pub source_paths_included: bool,
    /// Always false.
    pub commands_included: bool,
    /// Always false.
    pub network_destinations_included: bool,
    /// Always false.
    pub credentials_included: bool,
    /// Always false.
    pub authority_added: bool,
}

/// Exact accounting status for one synthetic artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactStatus {
    /// Exact artifact identity.
    pub artifact_hash: String,
    /// Always `synthetic_accounted`.
    pub status: String,
    /// Exact observed byte length.
    pub bytes_observed: String,
}

/// Complete authority-neutral synthetic result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerRunnerResult {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated exact result identity.
    pub result_id: String,
    /// Exact request identifier.
    pub request_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact manifest identity.
    pub manifest_id: String,
    /// Exact analyzer identifier.
    pub analyzer_id: String,
    /// Exact synthetic executable identity.
    pub executable_digest: String,
    /// Exact synthetic ruleset identity.
    pub ruleset_digest: String,
    /// Exact requested capabilities.
    pub capability_ids: Vec<String>,
    /// Exact resource-profile digest.
    pub resource_profile_digest: String,
    /// Canonical completion time supplied as control data.
    pub completed_at: String,
    /// Complete artifact accounting.
    pub artifact_statuses: Vec<ArtifactStatus>,
    /// Always empty in IAR-0.
    pub findings: Vec<serde_json::Value>,
    /// Always `complete` for the no-op behavior.
    pub completeness: String,
    /// Exact limitation statement.
    pub limitations: Vec<String>,
    /// Always true in IAR-0.
    pub synthetic_only: bool,
    /// Always false.
    pub safety_claimed: bool,
    /// Always false.
    pub ordinary_host_execution_authorized: bool,
    /// Always false.
    pub authority_added: bool,
}

/// Source-free all-or-nothing synthetic failure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnalyzerRunnerFailure {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated exact failure identity.
    pub failure_id: String,
    /// Exact request identifier.
    pub request_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact manifest identity.
    pub manifest_id: String,
    /// Canonical completion time supplied as control data.
    pub completed_at: String,
    /// Closed failure category.
    pub failure_code: String,
    /// Always true.
    pub source_free: bool,
    /// Always false.
    pub partial_result_accepted: bool,
    /// Always false.
    pub coverage_completed: bool,
    /// Always false.
    pub authority_added: bool,
}

/// The complete possible outcome of the pure synthetic model.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", content = "record", rename_all = "snake_case")]
pub enum SyntheticOutcome {
    /// Complete no-op accounting result.
    Result(AnalyzerRunnerResult),
    /// Complete source-free simulated failure.
    Failure(AnalyzerRunnerFailure),
}

#[derive(Serialize)]
struct ManifestIdentity<'a> {
    analyzer_id: &'a str,
    analyzer_version: &'a str,
    publisher: &'a str,
    executable_digest: &'a str,
    ruleset_digest: &'a str,
    capability_ids: &'a [String],
    host_platforms: &'a [String],
    target_platforms: &'a [String],
    network: &'a str,
    determinism: &'a str,
    result_schema: &'a str,
    revoked: bool,
    synthetic_only: bool,
    repository_supplied: bool,
    authority_added: bool,
}

#[derive(Serialize)]
struct ResultIdentity<'a> {
    request_id: &'a str,
    workspace_snapshot: &'a str,
    manifest_id: &'a str,
    analyzer_id: &'a str,
    executable_digest: &'a str,
    ruleset_digest: &'a str,
    capability_ids: &'a [String],
    resource_profile_digest: &'a str,
    completed_at: &'a str,
    artifact_statuses: &'a [ArtifactStatus],
    findings: &'a [serde_json::Value],
    completeness: &'a str,
    limitations: &'a [String],
    synthetic_only: bool,
    safety_claimed: bool,
    ordinary_host_execution_authorized: bool,
    authority_added: bool,
}

#[derive(Serialize)]
struct FailureIdentity<'a> {
    request_id: &'a str,
    workspace_snapshot: &'a str,
    manifest_id: &'a str,
    completed_at: &'a str,
    failure_code: &'a str,
    source_free: bool,
    partial_result_accepted: bool,
    coverage_completed: bool,
    authority_added: bool,
}

/// Returns the exact committed IAR-0 profile digest.
#[must_use]
pub const fn synthetic_profile_digest() -> &'static str {
    PROFILE_DIGEST
}

/// Computes the exact domain-separated manifest identity.
///
/// # Errors
///
/// Returns an error if canonical serialization fails.
pub fn manifest_identity(manifest: &AnalyzerExecutionManifest) -> Result<String, ProtocolError> {
    structured_identity(
        "analyzer-execution-manifest",
        &ManifestIdentity {
            analyzer_id: &manifest.analyzer_id,
            analyzer_version: &manifest.analyzer_version,
            publisher: &manifest.publisher,
            executable_digest: &manifest.executable_digest,
            ruleset_digest: &manifest.ruleset_digest,
            capability_ids: &manifest.capability_ids,
            host_platforms: &manifest.host_platforms,
            target_platforms: &manifest.target_platforms,
            network: &manifest.network,
            determinism: &manifest.determinism,
            result_schema: &manifest.result_schema,
            revoked: manifest.revoked,
            synthetic_only: manifest.synthetic_only,
            repository_supplied: manifest.repository_supplied,
            authority_added: manifest.authority_added,
        },
    )
}

/// Validates the closed IAR-0 manifest and exact identity.
///
/// # Errors
///
/// Returns a content-free error for malformed, non-canonical, revoked, or
/// authority-claiming manifests.
pub fn validate_manifest(manifest: &AnalyzerExecutionManifest) -> Result<(), ProtocolError> {
    let fields_valid = manifest.schema_name == "analyzer-execution-manifest"
        && manifest.schema_version == CONTRACT_VERSION
        && is_schema_name(&manifest.analyzer_id)
        && is_version(&manifest.analyzer_version)
        && !manifest.publisher.is_empty()
        && manifest.publisher.len() <= 1024
        && is_sha256(&manifest.executable_digest)
        && is_sha256(&manifest.ruleset_digest)
        && canonical_strings(
            &manifest.capability_ids,
            1,
            MAX_CAPABILITIES,
            is_schema_name,
        )
        && canonical_strings(&manifest.host_platforms, 1, 3, |value| {
            matches!(value, "macos_arm64" | "linux_x86_64" | "windows_x86_64")
        })
        && canonical_strings(&manifest.target_platforms, 1, 4, |value| {
            matches!(value, "any" | "windows" | "macos" | "linux")
        })
        && manifest.network == "none"
        && manifest.determinism == "deterministic"
        && manifest.result_schema == "analyzer-runner-result@1.0.0"
        && !manifest.revoked
        && manifest.synthetic_only
        && !manifest.repository_supplied
        && !manifest.authority_added;
    if !fields_valid {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    if manifest_identity(manifest)? != manifest.manifest_id {
        return Err(ProtocolError::new(ProtocolErrorCode::IdentityMismatch));
    }
    Ok(())
}

/// Validates a closed authority-neutral IAR-0 capability declaration.
///
/// # Errors
///
/// Returns a content-free error for malformed, non-canonical, or
/// authority-claiming declarations.
pub fn validate_capability(capability: &AnalyzerCapability) -> Result<(), ProtocolError> {
    let fields_valid = capability.schema_name == "analyzer-runner-capability"
        && capability.schema_version == CONTRACT_VERSION
        && is_schema_name(&capability.capability_id)
        && is_version(&capability.capability_version)
        && !capability.description.is_empty()
        && capability.description.len() <= 1024
        && canonical_strings(&capability.input_media_types, 1, 16, is_schema_name)
        && canonical_strings(&capability.target_platforms, 1, 4, |value| {
            matches!(value, "any" | "windows" | "macos" | "linux")
        })
        && capability.result_categories.is_empty()
        && capability.synthetic_only
        && !capability.authority_added;
    if fields_valid {
        Ok(())
    } else {
        Err(ProtocolError::new(ProtocolErrorCode::InvalidContract))
    }
}

/// Validates the closed IAR-0 request without reading any ambient state.
///
/// # Errors
///
/// Returns a content-free error for malformed, non-canonical, oversized, or
/// authority-claiming requests.
pub fn validate_request(request: &AnalyzerRunnerRequest) -> Result<(), ProtocolError> {
    let serialized = serde_json_canonicalizer::to_vec(request)
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::Serialization))?;
    if serialized.len() > MAX_REQUEST_BYTES {
        return Err(ProtocolError::new(ProtocolErrorCode::ResourceLimit));
    }
    let fields_valid = request.schema_name == "analyzer-runner-request"
        && request.schema_version == CONTRACT_VERSION
        && is_identifier(&request.request_id)
        && is_canonical_utc(&request.occurred_at)
        && is_canonical_utc(&request.deadline_at)
        && request.occurred_at < request.deadline_at
        && is_sha256(&request.workspace_snapshot)
        && is_sha256(&request.assessment_plan_id)
        && is_sha256(&request.policy_id)
        && is_sha256(&request.manifest_id)
        && canonical_strings(&request.capability_ids, 1, MAX_CAPABILITIES, is_schema_name)
        && request.resource_profile_id == PROFILE_ID
        && request.resource_profile_digest == PROFILE_DIGEST
        && matches!(
            request.synthetic_behavior.as_str(),
            "no_op" | "crash" | "timeout" | "input_mutation" | "output_flood" | "malformed_output"
        )
        && !request.source_paths_included
        && !request.commands_included
        && !request.network_destinations_included
        && !request.credentials_included
        && !request.authority_added;
    if !fields_valid {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    validate_descriptors(&request.artifacts)
}

/// Runs the deterministic in-memory IAR-0 synthetic behavior.
///
/// This function has no filesystem, process, environment, network, credential,
/// analyzer, parser, model, policy, or quarantine capability.
///
/// # Errors
///
/// Returns a content-free error when the manifest, request, completion time, or
/// supplied in-memory artifact bytes fail exact validation.
pub fn run_synthetic(
    request: &AnalyzerRunnerRequest,
    manifest: &AnalyzerExecutionManifest,
    artifacts: &BTreeMap<String, Vec<u8>>,
    completed_at: &str,
) -> Result<SyntheticOutcome, ProtocolError> {
    validate_manifest(manifest)?;
    validate_request(request)?;
    if request.manifest_id != manifest.manifest_id
        || request
            .capability_ids
            .iter()
            .any(|capability| manifest.capability_ids.binary_search(capability).is_err())
        || !is_canonical_utc(completed_at)
        || completed_at < request.occurred_at.as_str()
        || completed_at > request.deadline_at.as_str()
    {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    validate_artifact_bytes(&request.artifacts, artifacts)?;

    if request.synthetic_behavior != "no_op" {
        return Ok(SyntheticOutcome::Failure(build_failure(
            request,
            completed_at,
            match request.synthetic_behavior.as_str() {
                "crash" => "simulated_crash",
                "timeout" => "simulated_timeout",
                "input_mutation" => "simulated_input_mutation",
                "output_flood" => "simulated_output_flood",
                "malformed_output" => "simulated_malformed_output",
                _ => return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract)),
            },
        )?));
    }

    let statuses = request
        .artifacts
        .iter()
        .map(|artifact| ArtifactStatus {
            artifact_hash: artifact.artifact_hash.clone(),
            status: "synthetic_accounted".to_owned(),
            bytes_observed: artifact.bytes.clone(),
        })
        .collect::<Vec<_>>();
    let mut result = AnalyzerRunnerResult {
        schema_name: "analyzer-runner-result".to_owned(),
        schema_version: CONTRACT_VERSION.to_owned(),
        result_id: String::new(),
        request_id: request.request_id.clone(),
        workspace_snapshot: request.workspace_snapshot.clone(),
        manifest_id: manifest.manifest_id.clone(),
        analyzer_id: manifest.analyzer_id.clone(),
        executable_digest: manifest.executable_digest.clone(),
        ruleset_digest: manifest.ruleset_digest.clone(),
        capability_ids: request.capability_ids.clone(),
        resource_profile_digest: request.resource_profile_digest.clone(),
        completed_at: completed_at.to_owned(),
        artifact_statuses: statuses,
        findings: Vec::new(),
        completeness: "complete".to_owned(),
        limitations: vec!["synthetic-worker-no-analysis".to_owned()],
        synthetic_only: true,
        safety_claimed: false,
        ordinary_host_execution_authorized: false,
        authority_added: false,
    };
    result.result_id = result_identity(&result)?;
    Ok(SyntheticOutcome::Result(result))
}

/// Validates an exact complete IAR-0 result against its request and manifest.
///
/// # Errors
///
/// Returns a content-free error for partial, mismatched, authority-claiming, or
/// non-canonical results.
pub fn validate_result(
    result: &AnalyzerRunnerResult,
    request: &AnalyzerRunnerRequest,
    manifest: &AnalyzerExecutionManifest,
) -> Result<(), ProtocolError> {
    validate_manifest(manifest)?;
    validate_request(request)?;
    let expected_statuses = request
        .artifacts
        .iter()
        .map(|artifact| ArtifactStatus {
            artifact_hash: artifact.artifact_hash.clone(),
            status: "synthetic_accounted".to_owned(),
            bytes_observed: artifact.bytes.clone(),
        })
        .collect::<Vec<_>>();
    let fields_valid = result.schema_name == "analyzer-runner-result"
        && result.schema_version == CONTRACT_VERSION
        && result.request_id == request.request_id
        && result.workspace_snapshot == request.workspace_snapshot
        && result.manifest_id == manifest.manifest_id
        && result.analyzer_id == manifest.analyzer_id
        && result.executable_digest == manifest.executable_digest
        && result.ruleset_digest == manifest.ruleset_digest
        && result.capability_ids == request.capability_ids
        && result.resource_profile_digest == PROFILE_DIGEST
        && is_canonical_utc(&result.completed_at)
        && result.completed_at >= request.occurred_at
        && result.completed_at <= request.deadline_at
        && result.artifact_statuses == expected_statuses
        && result.findings.is_empty()
        && result.completeness == "complete"
        && result.limitations == ["synthetic-worker-no-analysis"]
        && result.synthetic_only
        && !result.safety_claimed
        && !result.ordinary_host_execution_authorized
        && !result.authority_added;
    if !fields_valid {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    if result_identity(result)? != result.result_id {
        return Err(ProtocolError::new(ProtocolErrorCode::IdentityMismatch));
    }
    Ok(())
}

/// Validates a source-free, all-or-nothing IAR-0 failure.
///
/// # Errors
///
/// Returns a content-free error for malformed, partial, mismatched, or
/// authority-claiming failures.
pub fn validate_failure(
    failure: &AnalyzerRunnerFailure,
    request: &AnalyzerRunnerRequest,
) -> Result<(), ProtocolError> {
    validate_request(request)?;
    let fields_valid = failure.schema_name == "analyzer-runner-failure"
        && failure.schema_version == CONTRACT_VERSION
        && failure.request_id == request.request_id
        && failure.workspace_snapshot == request.workspace_snapshot
        && failure.manifest_id == request.manifest_id
        && is_canonical_utc(&failure.completed_at)
        && failure.completed_at.as_str() >= request.occurred_at.as_str()
        && failure.completed_at.as_str() <= request.deadline_at.as_str()
        && matches!(
            failure.failure_code.as_str(),
            "simulated_crash"
                | "simulated_timeout"
                | "simulated_input_mutation"
                | "simulated_output_flood"
                | "simulated_malformed_output"
        )
        && failure.source_free
        && !failure.partial_result_accepted
        && !failure.coverage_completed
        && !failure.authority_added;
    if !fields_valid {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    if failure_identity(failure)? != failure.failure_id {
        return Err(ProtocolError::new(ProtocolErrorCode::IdentityMismatch));
    }
    Ok(())
}

/// Encodes one request as a bounded four-byte big-endian length-prefixed frame.
///
/// # Errors
///
/// Returns a content-free error if the request is invalid or exceeds the fixed
/// IAR-0 frame ceiling.
pub fn encode_request_frame(request: &AnalyzerRunnerRequest) -> Result<Vec<u8>, ProtocolError> {
    validate_request(request)?;
    let payload = serde_json_canonicalizer::to_vec(request)
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::Serialization))?;
    let length = u32::try_from(payload.len())
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::ResourceLimit))?;
    let mut frame = Vec::with_capacity(payload.len() + 4);
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decodes exactly one bounded request frame with no trailing bytes.
///
/// # Errors
///
/// Returns a content-free error for incomplete, oversized, trailing, malformed,
/// or invalid input.
pub fn decode_request_frame(frame: &[u8]) -> Result<AnalyzerRunnerRequest, ProtocolError> {
    let prefix: [u8; 4] = frame
        .get(..4)
        .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::InvalidContract))?
        .try_into()
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::InvalidContract))?;
    let length = usize::try_from(u32::from_be_bytes(prefix))
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::ResourceLimit))?;
    if length > MAX_REQUEST_BYTES {
        return Err(ProtocolError::new(ProtocolErrorCode::ResourceLimit));
    }
    let payload = frame
        .get(4..)
        .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::InvalidContract))?;
    if payload.len() != length {
        return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
    }
    let request = serde_json::from_slice::<AnalyzerRunnerRequest>(payload)
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::InvalidContract))?;
    validate_request(&request)?;
    Ok(request)
}

fn build_failure(
    request: &AnalyzerRunnerRequest,
    completed_at: &str,
    code: &str,
) -> Result<AnalyzerRunnerFailure, ProtocolError> {
    let mut failure = AnalyzerRunnerFailure {
        schema_name: "analyzer-runner-failure".to_owned(),
        schema_version: CONTRACT_VERSION.to_owned(),
        failure_id: String::new(),
        request_id: request.request_id.clone(),
        workspace_snapshot: request.workspace_snapshot.clone(),
        manifest_id: request.manifest_id.clone(),
        completed_at: completed_at.to_owned(),
        failure_code: code.to_owned(),
        source_free: true,
        partial_result_accepted: false,
        coverage_completed: false,
        authority_added: false,
    };
    failure.failure_id = failure_identity(&failure)?;
    Ok(failure)
}

fn validate_descriptors(descriptors: &[ArtifactDescriptor]) -> Result<(), ProtocolError> {
    if descriptors.is_empty() || descriptors.len() > MAX_ARTIFACTS {
        return Err(ProtocolError::new(ProtocolErrorCode::ResourceLimit));
    }
    let mut previous = None;
    let mut total = 0_u64;
    for descriptor in descriptors {
        let bytes = canonical_decimal(&descriptor.bytes)
            .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::InvalidContract))?;
        if bytes > MAX_ARTIFACT_BYTES {
            return Err(ProtocolError::new(ProtocolErrorCode::ResourceLimit));
        }
        total = total
            .checked_add(bytes)
            .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::ResourceLimit))?;
        if !is_sha256(&descriptor.artifact_hash)
            || !is_schema_name(&descriptor.media_type)
            || !matches!(
                descriptor.target_platform.as_str(),
                "any" | "windows" | "macos" | "linux"
            )
            || previous.is_some_and(|value: &str| value >= descriptor.artifact_hash.as_str())
        {
            return Err(ProtocolError::new(ProtocolErrorCode::InvalidContract));
        }
        previous = Some(&descriptor.artifact_hash);
    }
    if total > MAX_TOTAL_ARTIFACT_BYTES {
        return Err(ProtocolError::new(ProtocolErrorCode::ResourceLimit));
    }
    Ok(())
}

fn validate_artifact_bytes(
    descriptors: &[ArtifactDescriptor],
    artifacts: &BTreeMap<String, Vec<u8>>,
) -> Result<(), ProtocolError> {
    if artifacts.len() != descriptors.len() {
        return Err(ProtocolError::new(ProtocolErrorCode::ArtifactMismatch));
    }
    for descriptor in descriptors {
        let bytes = artifacts
            .get(&descriptor.artifact_hash)
            .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::ArtifactMismatch))?;
        let expected = canonical_decimal(&descriptor.bytes)
            .ok_or_else(|| ProtocolError::new(ProtocolErrorCode::InvalidContract))?;
        if usize::try_from(expected).ok() != Some(bytes.len())
            || sha256(bytes) != descriptor.artifact_hash
        {
            return Err(ProtocolError::new(ProtocolErrorCode::ArtifactMismatch));
        }
    }
    Ok(())
}

fn result_identity(result: &AnalyzerRunnerResult) -> Result<String, ProtocolError> {
    structured_identity(
        "analyzer-runner-result",
        &ResultIdentity {
            request_id: &result.request_id,
            workspace_snapshot: &result.workspace_snapshot,
            manifest_id: &result.manifest_id,
            analyzer_id: &result.analyzer_id,
            executable_digest: &result.executable_digest,
            ruleset_digest: &result.ruleset_digest,
            capability_ids: &result.capability_ids,
            resource_profile_digest: &result.resource_profile_digest,
            completed_at: &result.completed_at,
            artifact_statuses: &result.artifact_statuses,
            findings: &result.findings,
            completeness: &result.completeness,
            limitations: &result.limitations,
            synthetic_only: result.synthetic_only,
            safety_claimed: result.safety_claimed,
            ordinary_host_execution_authorized: result.ordinary_host_execution_authorized,
            authority_added: result.authority_added,
        },
    )
}

fn failure_identity(failure: &AnalyzerRunnerFailure) -> Result<String, ProtocolError> {
    structured_identity(
        "analyzer-runner-failure",
        &FailureIdentity {
            request_id: &failure.request_id,
            workspace_snapshot: &failure.workspace_snapshot,
            manifest_id: &failure.manifest_id,
            completed_at: &failure.completed_at,
            failure_code: &failure.failure_code,
            source_free: failure.source_free,
            partial_result_accepted: failure.partial_result_accepted,
            coverage_completed: failure.coverage_completed,
            authority_added: failure.authority_added,
        },
    )
}

fn structured_identity(kind: &str, value: &impl Serialize) -> Result<String, ProtocolError> {
    let payload = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| ProtocolError::new(ProtocolErrorCode::Serialization))?;
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(CONTRACT_VERSION.as_bytes());
    hasher.update(b"\0");
    hasher.update(payload);
    Ok(format_digest(hasher.finalize()))
}

fn sha256(bytes: &[u8]) -> String {
    format_digest(Sha256::digest(bytes))
}

fn format_digest(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .fold(String::from("sha256:"), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a string cannot fail");
            hex
        })
}

fn canonical_strings(
    values: &[String],
    minimum: usize,
    maximum: usize,
    validate: impl Fn(&str) -> bool,
) -> bool {
    values.len() >= minimum
        && values.len() <= maximum
        && values.windows(2).all(|pair| pair[0] < pair[1])
        && values.iter().all(|value| validate(value))
}

fn canonical_decimal(value: &str) -> Option<u64> {
    if value == "0" || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        value.parse().ok()
    } else {
        None
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_schema_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
}

fn is_identifier(value: &str) -> bool {
    let Some((prefix, suffix)) = value.split_once('_') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.len() <= 32
        && prefix.as_bytes()[0].is_ascii_lowercase()
        && prefix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && (8..=128).contains(&suffix.len())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn is_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3 && parts.iter().all(|part| canonical_decimal(part).is_some())
}

fn is_canonical_utc(value: &str) -> bool {
    if value.len() != 20 || !value.ends_with('Z') {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes.iter().enumerate().any(|(index, byte)| {
            !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit()
        })
    {
        return false;
    }
    let parse = |start: usize, end: usize| value[start..end].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        parse(0, 4),
        parse(5, 7),
        parse(8, 10),
        parse(11, 13),
        parse(14, 16),
        parse(17, 19),
    ) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day > 0 && day <= days && hour < 24 && minute < 60 && second < 60
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

    fn manifest() -> AnalyzerExecutionManifest {
        let mut value = AnalyzerExecutionManifest {
            schema_name: "analyzer-execution-manifest".into(),
            schema_version: CONTRACT_VERSION.into(),
            manifest_id: String::new(),
            analyzer_id: "impresari.synthetic".into(),
            analyzer_version: "1.0.0".into(),
            publisher: "BoldtHaus Studio, LLC".into(),
            executable_digest: HASH_B.into(),
            ruleset_digest: HASH_C.into(),
            capability_ids: vec!["synthetic.accounting".into()],
            host_platforms: vec![
                "linux_x86_64".into(),
                "macos_arm64".into(),
                "windows_x86_64".into(),
            ],
            target_platforms: vec!["any".into()],
            network: "none".into(),
            determinism: "deterministic".into(),
            result_schema: "analyzer-runner-result@1.0.0".into(),
            revoked: false,
            synthetic_only: true,
            repository_supplied: false,
            authority_added: false,
        };
        value.manifest_id = manifest_identity(&value).expect("manifest identity");
        value
    }

    fn request(
        manifest: &AnalyzerExecutionManifest,
        behavior: &str,
        bytes: &[u8],
    ) -> AnalyzerRunnerRequest {
        AnalyzerRunnerRequest {
            schema_name: "analyzer-runner-request".into(),
            schema_version: CONTRACT_VERSION.into(),
            request_id: "req_synthetic01".into(),
            occurred_at: "2026-08-29T00:00:00Z".into(),
            deadline_at: "2026-08-29T00:00:01Z".into(),
            workspace_snapshot: HASH_D.into(),
            assessment_plan_id: HASH_A.into(),
            policy_id: HASH_B.into(),
            manifest_id: manifest.manifest_id.clone(),
            capability_ids: vec!["synthetic.accounting".into()],
            artifacts: vec![ArtifactDescriptor {
                artifact_hash: sha256(bytes),
                bytes: bytes.len().to_string(),
                media_type: "application.octet-stream".into(),
                target_platform: "any".into(),
            }],
            resource_profile_id: PROFILE_ID.into(),
            resource_profile_digest: PROFILE_DIGEST.into(),
            synthetic_behavior: behavior.into(),
            source_paths_included: false,
            commands_included: false,
            network_destinations_included: false,
            credentials_included: false,
            authority_added: false,
        }
    }

    #[test]
    fn no_op_accounts_for_exact_bytes_without_findings_or_authority() {
        let manifest = manifest();
        let bytes = b"safe".to_vec();
        let request = request(&manifest, "no_op", &bytes);
        let artifacts = BTreeMap::from([(request.artifacts[0].artifact_hash.clone(), bytes)]);
        let outcome = run_synthetic(&request, &manifest, &artifacts, "2026-08-29T00:00:01Z")
            .expect("synthetic result");
        let SyntheticOutcome::Result(result) = outcome else {
            panic!("expected result")
        };
        validate_result(&result, &request, &manifest).expect("valid exact result");
        assert!(result.findings.is_empty());
        assert!(!result.safety_claimed);
        assert!(!result.authority_added);
    }

    #[test]
    fn every_fault_is_source_free_and_all_or_nothing() {
        let manifest = manifest();
        for (behavior, code) in [
            ("crash", "simulated_crash"),
            ("timeout", "simulated_timeout"),
            ("input_mutation", "simulated_input_mutation"),
            ("output_flood", "simulated_output_flood"),
            ("malformed_output", "simulated_malformed_output"),
        ] {
            let bytes = b"safe".to_vec();
            let request = request(&manifest, behavior, &bytes);
            let artifacts = BTreeMap::from([(request.artifacts[0].artifact_hash.clone(), bytes)]);
            let SyntheticOutcome::Failure(failure) =
                run_synthetic(&request, &manifest, &artifacts, "2026-08-29T00:00:01Z")
                    .expect("synthetic failure")
            else {
                panic!("expected failure")
            };
            assert_eq!(failure.failure_code, code);
            assert!(failure.source_free);
            assert!(!failure.partial_result_accepted);
            assert!(!failure.coverage_completed);
            assert!(!failure.authority_added);
            validate_failure(&failure, &request).expect("valid exact failure");
        }
    }

    #[test]
    fn request_framing_is_exact_and_fail_closed() {
        let manifest = manifest();
        let request = request(&manifest, "no_op", b"safe");
        let frame = encode_request_frame(&request).expect("valid frame");
        assert_eq!(
            decode_request_frame(&frame).expect("valid request"),
            request
        );

        let mut trailing = frame.clone();
        trailing.push(0);
        assert_eq!(
            decode_request_frame(&trailing).unwrap_err().code(),
            ProtocolErrorCode::InvalidContract
        );
        assert_eq!(
            decode_request_frame(&frame[..frame.len() - 1])
                .unwrap_err()
                .code(),
            ProtocolErrorCode::InvalidContract
        );
        assert_eq!(
            decode_request_frame(&[0, 4, 0, 1]).unwrap_err().code(),
            ProtocolErrorCode::ResourceLimit
        );
    }

    #[test]
    fn request_authority_and_noncanonical_inputs_fail_closed() {
        let manifest = manifest();
        let mut request = request(&manifest, "no_op", b"safe");
        request.commands_included = true;
        assert_eq!(
            validate_request(&request).unwrap_err().code(),
            ProtocolErrorCode::InvalidContract
        );
        request.commands_included = false;
        request.capability_ids.push("synthetic.accounting".into());
        assert_eq!(
            validate_request(&request).unwrap_err().code(),
            ProtocolErrorCode::InvalidContract
        );
    }

    #[test]
    fn altered_or_missing_artifact_bytes_never_produce_a_result() {
        let manifest = manifest();
        let request = request(&manifest, "no_op", b"safe");
        let missing = BTreeMap::new();
        assert_eq!(
            run_synthetic(&request, &manifest, &missing, "2026-08-29T00:00:01Z")
                .unwrap_err()
                .code(),
            ProtocolErrorCode::ArtifactMismatch
        );
        let altered =
            BTreeMap::from([(request.artifacts[0].artifact_hash.clone(), b"evil".to_vec())]);
        assert_eq!(
            run_synthetic(&request, &manifest, &altered, "2026-08-29T00:00:01Z")
                .unwrap_err()
                .code(),
            ProtocolErrorCode::ArtifactMismatch
        );
    }

    #[test]
    fn manifest_identity_and_profile_are_exact() {
        let mut manifest = manifest();
        validate_manifest(&manifest).expect("valid manifest");
        manifest.publisher.push('!');
        assert_eq!(
            validate_manifest(&manifest).unwrap_err().code(),
            ProtocolErrorCode::IdentityMismatch
        );
        assert_eq!(synthetic_profile_digest(), PROFILE_DIGEST);
    }
}
