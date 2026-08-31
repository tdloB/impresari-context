// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Application-enforced synthetic staging and supervision for ADR-0074 IAR-1A."]
// Independent measured controls must remain explicit so unsupported isolation
// claims cannot be inferred from one aggregate state.
#![allow(clippy::struct_excessive_bools)]

use std::{
    collections::BTreeMap,
    error::Error,
    ffi::OsStr,
    fmt,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use context_analyzer_protocol::{
    AnalyzerExecutionManifest, AnalyzerRunnerRequest, SyntheticOutcome, run_synthetic,
    validate_failure, validate_manifest, validate_request, validate_result,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const MAX_CONTROL_BYTES: usize = 262_144;
const MAX_OUTPUT_BYTES: usize = 262_144;
const STDERR_BYTES: usize = 16_384;
const SUPERVISOR_PROFILE_ID: &str = "iar-application-supervisor-v1";
const SUPERVISOR_PROFILE_DIGEST: &str =
    "sha256:3a81a84d2658c0a2a33214e2424daf84696c2692d73f52f6dfd46700ab2e51fa";
/// Frozen macOS hybrid XPC resource-profile identity.
pub const MACOS_XPC_PROFILE_ID: &str = "iar-macos-xpc-hybrid-v1";
/// Digest of the exact committed macOS hybrid XPC resource-profile bytes.
pub const MACOS_XPC_PROFILE_DIGEST: &str =
    "sha256:7b33023031e84ac63e686054837cc20416e5e82cee333d7007fa1e1788581acf";
/// Exact production bundle identifier selected by ADR-0076.
pub const MACOS_XPC_BUNDLE_IDENTIFIER: &str = "studio.boldthaus.impresari-context";
/// Exact background host location inside the sealed bundle.
pub const MACOS_XPC_HOST_RELATIVE_PATH: &str = "Contents/MacOS/impresari-context-xpc-host";
/// Exact private XPC service name.
pub const MACOS_XPC_SERVICE_NAME: &str = "studio.boldthaus.impresari-context.Analyzer";
/// Exact private XPC service location inside the sealed bundle.
pub const MACOS_XPC_SERVICE_RELATIVE_PATH: &str =
    "Contents/XPCServices/studio.boldthaus.impresari-context.Analyzer.xpc";
const MACOS_XPC_MAX_ARTIFACTS: u64 = 64;
const MACOS_XPC_MAX_TOTAL_ARTIFACT_BYTES: u64 = 4_194_304;
/// Frozen macOS local-VM supervisor lifecycle profile.
pub const MACOS_VM_SUPERVISOR_PROFILE_ID: &str = "iar-macos-local-vm-supervisor-v2";
/// Digest of the exact committed local-VM supervisor lifecycle profile.
pub const MACOS_VM_SUPERVISOR_PROFILE_DIGEST: &str =
    "sha256:614b9da42f051518e6a6d54f15e75c492e233e2ed653bfcbf69285d130967b88";
/// Frozen profile exposed by the synthetic macOS VM controller.
pub const MACOS_VM_CONTROLLER_PROFILE_ID: &str = "iar-macos-local-vm-synthetic-matrix-v2";
/// Exact profile digest exposed by the synthetic macOS VM controller.
pub const MACOS_VM_CONTROLLER_PROFILE_DIGEST: &str =
    "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888";
/// Frozen macOS local-VM synthetic resource and host-canary profile.
pub const MACOS_VM_RESOURCE_CANARY_PROFILE_ID: &str = "iar-macos-local-vm-resource-canary-v2";
/// Digest of the exact committed resource and host-canary profile bytes.
pub const MACOS_VM_RESOURCE_CANARY_PROFILE_DIGEST: &str =
    "sha256:82d3cbf4b68866b92794a06e86948ccaf2492b3b4cb38e7e70503562c61d1de0";
/// Frozen macOS local-VM host-interruption profile.
pub const MACOS_VM_HOST_INTERRUPTION_PROFILE_ID: &str = "iar-macos-local-vm-interruption-v2";
/// Digest of the exact committed host-interruption profile bytes.
pub const MACOS_VM_HOST_INTERRUPTION_PROFILE_DIGEST: &str =
    "sha256:f1b57b17d9de3b2b4de885732b6bef0f3cbf637bcba08dc1dda34724e9b18c4f";
const MACOS_VM_KERNEL_DIGEST: &str =
    "sha256:4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5";
const MACOS_VM_INITRAMFS_DIGEST: &str =
    "sha256:89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b";
const MACOS_VM_RESOURCE_INITRAMFS_DIGEST: &str =
    "sha256:1a4029b781020260e4cb8c18271e3a01e1920f1448d87a71678e12cc617a1ec3";
const MACOS_VM_SYNTHETIC_INPUT_DIGEST: &str =
    "sha256:3050d67653f05f1db0dcef073a64f6fc9f9ac2e55c7b1881e7372151b3e4fd99";
const MACOS_VM_STDOUT_BYTES: usize = 65_536;
const MACOS_VM_STDERR_BYTES: usize = 16_384;

/// Stable source-free supervisor failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunnerErrorCode {
    /// Runner configuration or a closed contract was invalid.
    InvalidConfiguration,
    /// The pinned worker executable did not match the manifest.
    WorkerIdentity,
    /// Private staging could not be created, verified, or removed safely.
    Staging,
    /// Exact staged artifact bytes changed or did not match.
    ArtifactMismatch,
    /// Worker launch, pipe, or reap failed.
    Process,
    /// The worker exceeded the fixed wall-time ceiling.
    Timeout,
    /// The worker exceeded a bounded output channel.
    OutputLimit,
    /// The worker returned malformed, partial, or mismatched output.
    InvalidOutput,
    /// The worker exited unsuccessfully.
    WorkerFailure,
}

/// Source-free IAR-1A supervisor error.
#[derive(Debug)]
pub struct RunnerError(RunnerErrorCode);

impl RunnerError {
    const fn new(code: RunnerErrorCode) -> Self {
        Self(code)
    }

    /// Returns the stable source-free category.
    #[must_use]
    pub const fn code(&self) -> RunnerErrorCode {
        self.0
    }
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            RunnerErrorCode::InvalidConfiguration => "invalid analyzer runner configuration",
            RunnerErrorCode::WorkerIdentity => "analyzer worker identity mismatch",
            RunnerErrorCode::Staging => "analyzer staging failed",
            RunnerErrorCode::ArtifactMismatch => "staged artifact mismatch",
            RunnerErrorCode::Process => "analyzer worker process failed",
            RunnerErrorCode::Timeout => "analyzer worker timed out",
            RunnerErrorCode::OutputLimit => "analyzer worker output limit exceeded",
            RunnerErrorCode::InvalidOutput => "invalid analyzer worker output",
            RunnerErrorCode::WorkerFailure => "analyzer worker failed",
        })
    }
}

impl Error for RunnerError {}

/// Honest measured confinement posture for IAR-1A.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfinementPosture {
    /// Fixed IAR-1A application-supervisor profile.
    pub profile_id: String,
    /// Exact committed profile bytes digest.
    pub profile_digest: String,
    /// Portable supervisor controls are active.
    pub application_enforced: bool,
    /// IAR-1A makes no OS sandbox claim.
    pub os_confined: bool,
    /// IAR-1A makes no VM isolation claim.
    pub vm_confined: bool,
    /// Worker environment is cleared before launch.
    pub environment_cleared: bool,
    /// Worker current directory is private per job.
    pub private_working_directory: bool,
    /// Input is checked before and after the worker.
    pub input_rehashed: bool,
    /// Worker transport is bounded and all-or-nothing.
    pub bounded_transport: bool,
    /// Network denial is not established by portable IAR-1A controls.
    pub network_denial_verified: bool,
    /// Stable limitations that prevent a sandbox overclaim.
    pub limitations: Vec<String>,
}

impl ConfinementPosture {
    fn iar1() -> Self {
        Self {
            profile_id: SUPERVISOR_PROFILE_ID.to_owned(),
            profile_digest: SUPERVISOR_PROFILE_DIGEST.to_owned(),
            application_enforced: true,
            os_confined: false,
            vm_confined: false,
            environment_cleared: true,
            private_working_directory: true,
            input_rehashed: true,
            bounded_transport: true,
            network_denial_verified: false,
            limitations: vec![
                "no-os-sandbox".to_owned(),
                "network-denial-unverified".to_owned(),
                "descendant-containment-unverified".to_owned(),
                "executable-substitution-race-unresolved".to_owned(),
                "staged-input-immutability-unverified".to_owned(),
                "synthetic-worker-no-analysis".to_owned(),
            ],
        }
    }
}

/// Complete validated supervisor output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SupervisedOutcome {
    /// Exact synthetic result or source-free synthetic failure.
    pub outcome: SyntheticOutcome,
    /// Measured IAR-1A posture; never an OS/VM sandbox claim.
    pub confinement: ConfinementPosture,
    /// Source-free completion audit.
    pub audit: SupervisorAudit,
}

/// Bounded source-free supervisor audit for a promoted complete result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SupervisorAudit {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Caller-owned request identifier.
    pub request_id: String,
    /// Exact application-supervisor profile digest.
    pub profile_digest: String,
    /// Canonical artifact count.
    pub artifacts_staged: String,
    /// Canonical total staged bytes.
    pub bytes_staged: String,
    /// Complete outcome category.
    pub outcome: String,
    /// Exact job storage was removed before this record was returned.
    pub job_removed: bool,
    /// No source bytes are retained in this record.
    pub source_retained: bool,
    /// No authority is added.
    pub authority_added: bool,
}

/// Explicit supervisor configuration.
#[derive(Clone, Debug)]
pub struct Supervisor {
    /// Absolute path to the exact first-party synthetic worker.
    pub executable: PathBuf,
    /// Existing private root beneath which one new job directory is created.
    pub staging_root: PathBuf,
    /// Existing source/cache roots that must be disjoint from staging.
    pub excluded_roots: Vec<PathBuf>,
    /// Fixed wall-time limit, no more than one minute.
    pub timeout: Duration,
}

/// Source-free Rust-to-host launch request for the selected macOS XPC backend.
///
/// This is a closed preparation handshake. It carries only immutable identity
/// and accounting data; source paths, commands, arguments, environment,
/// credentials, network authority, and analyzer execution are prohibited.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsXpcLaunchRequest {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Caller-owned request identifier.
    pub request_id: String,
    /// Frozen resource-profile identifier.
    pub profile_id: String,
    /// Exact resource-profile bytes digest.
    pub profile_digest: String,
    /// Digest of the canonical source-free job description.
    pub job_digest: String,
    /// Canonical artifact count.
    pub artifacts: String,
    /// Canonical total artifact bytes.
    pub total_artifact_bytes: String,
    /// Expected sealed application identity.
    pub bundle_identifier: String,
    /// Expected background host path relative to the bundle root.
    pub host_relative_path: String,
    /// Expected private XPC service name.
    pub service_name: String,
    /// Expected private XPC bundle path relative to the bundle root.
    pub service_relative_path: String,
    /// A repository path is never admitted across this boundary.
    pub repository_path_present: bool,
    /// Arbitrary launch arguments are never admitted across this boundary.
    pub arguments_present: bool,
    /// Caller-controlled environment is never admitted across this boundary.
    pub environment_present: bool,
    /// Credentials are never admitted across this boundary.
    pub credentials_present: bool,
    /// The worker has no network authority.
    pub network_authorized: bool,
    /// The contract remains synthetic-only until IAR-1B admission.
    pub analyzer_execution: bool,
    /// The handshake cannot add authority.
    pub authority_added: bool,
}

/// Source-free preparation record returned by the selected macOS XPC host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsXpcLaunchPreparation {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Exact request identity copied from the launch request.
    pub request_id: String,
    /// Frozen resource-profile identifier.
    pub profile_id: String,
    /// Exact resource-profile bytes digest.
    pub profile_digest: String,
    /// Exact source-free job identity copied from the request.
    pub job_digest: String,
    /// Exact prepared host process identifier.
    pub host_process_id: String,
    /// Exact prepared private-service process identifier.
    pub service_process_id: String,
    /// The sealed application identity matched the request.
    pub bundle_identity_verified: bool,
    /// The private service identity matched the request.
    pub service_identity_verified: bool,
    /// Effective resource limits matched the frozen profile.
    pub effective_profile_verified: bool,
    /// The effective private-service entitlements contain no network grant.
    pub network_entitlement_absent: bool,
    /// The prepared identity may receive the separately bounded job only when true.
    pub ready: bool,
    /// Preparation alone never establishes the complete IAR-1B claim.
    pub os_confined: bool,
    /// Preparation alone never admits a production backend.
    pub production_admitted: bool,
    /// No source bytes are retained in this preparation record.
    pub source_retained: bool,
    /// The preparation record cannot add authority.
    pub authority_added: bool,
}

/// Validates the closed source-free macOS XPC launch request.
///
/// # Errors
///
/// Returns `InvalidConfiguration` for any identity mismatch, malformed
/// accounting value, exceeded profile limit, or authority-bearing field.
pub fn validate_macos_xpc_launch_request(
    request: &MacOsXpcLaunchRequest,
) -> Result<(), RunnerError> {
    let artifacts = parse_canonical_u64(&request.artifacts)?;
    let total_artifact_bytes = parse_canonical_u64(&request.total_artifact_bytes)?;
    if request.schema_name != "macos-xpc-launch-request"
        || request.schema_version != "1.0.0"
        || !valid_identifier(&request.request_id)
        || request.profile_id != MACOS_XPC_PROFILE_ID
        || request.profile_digest != MACOS_XPC_PROFILE_DIGEST
        || !valid_sha256(&request.job_digest)
        || artifacts > MACOS_XPC_MAX_ARTIFACTS
        || total_artifact_bytes > MACOS_XPC_MAX_TOTAL_ARTIFACT_BYTES
        || (artifacts == 0) != (total_artifact_bytes == 0)
        || request.bundle_identifier != MACOS_XPC_BUNDLE_IDENTIFIER
        || request.host_relative_path != MACOS_XPC_HOST_RELATIVE_PATH
        || request.service_name != MACOS_XPC_SERVICE_NAME
        || request.service_relative_path != MACOS_XPC_SERVICE_RELATIVE_PATH
        || request.repository_path_present
        || request.arguments_present
        || request.environment_present
        || request.credentials_present
        || request.network_authorized
        || request.analyzer_execution
        || request.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
    }
    Ok(())
}

/// Validates a macOS host preparation record against its exact request.
///
/// # Errors
///
/// Returns `InvalidOutput` unless every identity and effective-profile check
/// passes while all admission, retention, and authority claims remain false.
pub fn validate_macos_xpc_launch_preparation(
    request: &MacOsXpcLaunchRequest,
    preparation: &MacOsXpcLaunchPreparation,
) -> Result<(), RunnerError> {
    validate_macos_xpc_launch_request(request)?;
    let host_process_id = parse_canonical_u64(&preparation.host_process_id)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let service_process_id = parse_canonical_u64(&preparation.service_process_id)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if preparation.schema_name != "macos-xpc-launch-preparation"
        || preparation.schema_version != "1.0.0"
        || preparation.request_id != request.request_id
        || preparation.profile_id != request.profile_id
        || preparation.profile_digest != request.profile_digest
        || preparation.job_digest != request.job_digest
        || host_process_id <= 1
        || service_process_id <= 1
        || !preparation.bundle_identity_verified
        || !preparation.service_identity_verified
        || !preparation.effective_profile_verified
        || !preparation.network_entitlement_absent
        || !preparation.ready
        || preparation.os_confined
        || preparation.production_admitted
        || preparation.source_retained
        || preparation.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(())
}

/// Synthetic-only lifecycle action exercised by the macOS VM supervisor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MacOsVmSupervisorAction {
    /// Request cancellation through the exact job-private control marker.
    ExternalCancellation,
    /// Kill and reap the controller, remove its exact stale job, then recover.
    ForcedTerminationRecovery,
}

/// Exact synthetic macOS VM supervisor configuration.
#[derive(Clone, Debug)]
pub struct MacOsVmSyntheticSupervisor {
    /// Absolute canonical path to the ad hoc signed feasibility controller.
    pub controller: PathBuf,
    /// Runtime digest of the exact controller bytes selected for this run.
    pub expected_controller_digest: String,
    /// Absolute canonical root containing only the pinned VM assets.
    pub asset_root: PathBuf,
    /// Fixed readiness and completion ceiling, no more than one minute.
    pub timeout: Duration,
}

/// Source-free result of one external lifecycle action and recovery job.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsVmSupervisorReceipt {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Frozen supervisor profile.
    pub profile_id: String,
    /// Exact supervisor profile bytes digest.
    pub profile_digest: String,
    /// Frozen profile exposed by the child controller.
    pub controller_profile_id: String,
    /// Exact frozen child-controller profile digest.
    pub controller_profile_digest: String,
    /// Runtime digest of the exact launched controller bytes.
    pub controller_digest: String,
    /// Caller-owned source-free job identifier.
    pub job_id: String,
    /// Exact synthetic action completed by the supervisor.
    pub action: MacOsVmSupervisorAction,
    /// The controller digest matched immediately before child launch.
    pub controller_digest_verified_before_launch: bool,
    /// The controller reached the job-private ready state.
    pub controller_ready: bool,
    /// The job-private external cancellation request was written.
    pub external_cancellation_requested: bool,
    /// The controller returned its exact cancellation receipt.
    pub controller_cancellation_verified: bool,
    /// The supervisor forcibly terminated the exact child controller.
    pub controller_forcibly_terminated: bool,
    /// The exact child controller was reaped.
    pub controller_reaped: bool,
    /// Exact stale job state was absent before recovery.
    pub stale_job_removed: bool,
    /// A fresh post-action VM job completed successfully.
    pub recovery_job_succeeded: bool,
    /// Both the action and recovery job roots were absent before return.
    pub all_job_state_removed: bool,
    /// Partial feasibility never establishes VM confinement.
    pub vm_confined: bool,
    /// Partial feasibility never admits production execution.
    pub production_admitted: bool,
    /// No analyzer runs in this synthetic checkpoint.
    pub analyzer_execution: bool,
    /// The source-free receipt retains no source bytes.
    pub source_retained: bool,
    /// The supervisor adds no authority.
    pub authority_added: bool,
}

/// Source-free Rust-supervised guest-resource and host-canary evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsVmResourceCanaryReceipt {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Frozen resource/canary profile.
    pub profile_id: String,
    /// Exact resource/canary profile bytes digest.
    pub profile_digest: String,
    /// Runtime digest of the exact controller bytes.
    pub controller_digest: String,
    /// Caller-owned source-free job identifier.
    pub job_id: String,
    /// Exact frozen guest kernel.
    pub kernel_digest: String,
    /// Exact separate resource-test initramfs.
    pub initramfs_digest: String,
    /// Exact synthetic input identity.
    pub input_digest: String,
    /// The controller digest matched immediately before child launch.
    pub controller_digest_verified_before_launch: bool,
    /// The Virtualization configuration passed framework validation.
    pub configuration_validated: bool,
    /// Exact virtual CPU count.
    pub cpu_count: String,
    /// Exact VM memory bytes.
    pub memory_bytes: String,
    /// Exact attached storage device count.
    pub storage_devices: String,
    /// Exact attached network device count.
    pub network_devices: String,
    /// Exact directory-share count.
    pub directory_shares: String,
    /// The synthetic host-only canary corpus was created.
    pub host_canary_corpus_created: bool,
    /// The host-only corpus remained byte-exact after the guest stopped.
    pub host_canary_corpus_unchanged: bool,
    /// The guest observed only the two exact attached block devices.
    pub attached_device_set_exact: bool,
    /// No host-canary marker occurred on either attached disk.
    pub host_canary_bytes_absent: bool,
    /// Prohibited host path families were absent inside the guest.
    pub host_paths_absent: bool,
    /// No guest process entry exposed the host controller identity.
    pub host_process_invisible: bool,
    /// A child exceeding the frozen memory limit was contained and killed.
    pub memory_pressure_contained: bool,
    /// Guest cgroup OOM-kill count.
    pub memory_oom_kills: String,
    /// A CPU-bound child was measurably throttled under the frozen quota.
    pub cpu_pressure_bounded: bool,
    /// CPU microseconds charged during the pressure interval.
    pub cpu_usage_usec: String,
    /// Throttled cgroup periods during the pressure interval.
    pub cpu_throttled_periods: String,
    /// Maximum task count observed in the job cgroup.
    pub pids_peak: String,
    /// The exact guest job cgroup was removed.
    pub job_cgroup_removed: bool,
    /// The exact host job state was removed.
    pub job_removed: bool,
    /// Partial feasibility never establishes VM confinement.
    pub vm_confined: bool,
    /// Partial feasibility never admits production execution.
    pub production_admitted: bool,
    /// No analyzer runs in this synthetic checkpoint.
    pub analyzer_execution: bool,
    /// The source-free receipt retains no source bytes.
    pub source_retained: bool,
    /// The supervisor adds no authority.
    pub authority_added: bool,
}

/// Source-free Rust-supervised synthetic host-interruption evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MacOsVmHostInterruptionReceipt {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Frozen interruption profile.
    pub profile_id: String,
    /// Exact interruption profile bytes digest.
    pub profile_digest: String,
    /// Frozen profile exposed by the child controller.
    pub controller_profile_id: String,
    /// Exact frozen child-controller profile digest.
    pub controller_profile_digest: String,
    /// Runtime digest of the exact launched controller bytes.
    pub controller_digest: String,
    /// Caller-owned source-free job identifier.
    pub job_id: String,
    /// Exact automated trigger; never represents a real host sleep.
    pub interruption_source: String,
    /// The controller installed the macOS will-sleep observer.
    pub sleep_observer_installed: bool,
    /// Synthetic and operating-system events enter one stop handler.
    pub shared_stop_handler_used: bool,
    /// The job-private synthetic interruption request was created.
    pub synthetic_interruption_requested: bool,
    /// The virtual machine stopped through the shared handler.
    pub virtual_machine_stopped: bool,
    /// The exact child controller was reaped.
    pub controller_reaped: bool,
    /// Exact stale job state was absent before recovery.
    pub stale_job_removed: bool,
    /// A fresh post-interruption VM job completed successfully.
    pub recovery_job_succeeded: bool,
    /// Both interruption and recovery job roots were absent before return.
    pub all_job_state_removed: bool,
    /// Automated evidence intentionally does not claim actual system sleep.
    pub real_host_sleep_observed: bool,
    /// Partial feasibility never establishes VM confinement.
    pub vm_confined: bool,
    /// Partial feasibility never admits production execution.
    pub production_admitted: bool,
    /// No analyzer runs in this synthetic checkpoint.
    pub analyzer_execution: bool,
    /// The source-free receipt retains no source bytes.
    pub source_retained: bool,
    /// The supervisor adds no authority.
    pub authority_added: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MacOsVmControllerFailure {
    schema_name: String,
    schema_version: String,
    profile_id: String,
    profile_digest: String,
    category: String,
    vm_confined: bool,
    production_admitted: bool,
    analyzer_execution: bool,
    source_retained: bool,
    authority_added: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MacOsVmControllerJobReceipt {
    schema_name: String,
    schema_version: String,
    profile_id: String,
    profile_digest: String,
    job_id: String,
    result: String,
    kernel_digest: String,
    initramfs_digest: String,
    input_digest: String,
    virtualization_supported: bool,
    configuration_validated: bool,
    cpu_count: String,
    memory_bytes: String,
    serial_ports: String,
    storage_devices: String,
    network_devices: String,
    directory_shares: String,
    graphics_devices: String,
    audio_devices: String,
    input_devices: String,
    exact_input_verified: bool,
    read_only_input_verified: bool,
    scratch_initially_clean: bool,
    scratch_capacity_verified: bool,
    network_device_absent: bool,
    job_removed: bool,
    vm_confined: bool,
    production_admitted: bool,
    source_retained: bool,
    authority_added: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MacOsVmControllerResourceCanaryReceipt {
    schema_name: String,
    schema_version: String,
    profile_id: String,
    profile_digest: String,
    job_id: String,
    result: String,
    kernel_digest: String,
    initramfs_digest: String,
    input_digest: String,
    virtualization_supported: bool,
    configuration_validated: bool,
    cpu_count: String,
    memory_bytes: String,
    storage_devices: String,
    network_devices: String,
    directory_shares: String,
    host_canary_corpus_created: bool,
    host_canary_corpus_unchanged: bool,
    attached_device_set_exact: bool,
    host_canary_bytes_absent: bool,
    host_paths_absent: bool,
    host_process_invisible: bool,
    memory_pressure_contained: bool,
    memory_oom_kills: String,
    cpu_pressure_bounded: bool,
    cpu_usage_usec: String,
    cpu_throttled_periods: String,
    pids_peak: String,
    job_cgroup_removed: bool,
    job_removed: bool,
    vm_confined: bool,
    production_admitted: bool,
    analyzer_execution: bool,
    source_retained: bool,
    authority_added: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MacOsVmControllerInterruptionReceipt {
    schema_name: String,
    schema_version: String,
    profile_id: String,
    profile_digest: String,
    job_id: String,
    result: String,
    interruption_source: String,
    sleep_observer_installed: bool,
    shared_stop_handler_used: bool,
    virtualization_supported: bool,
    configuration_validated: bool,
    virtual_machine_stopped: bool,
    job_removed: bool,
    real_host_sleep_observed: bool,
    vm_confined: bool,
    production_admitted: bool,
    analyzer_execution: bool,
    source_retained: bool,
    authority_added: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerControl {
    manifest: AnalyzerExecutionManifest,
    request: AnalyzerRunnerRequest,
    completed_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkerOutput {
    outcome: SyntheticOutcome,
    confinement: ConfinementPosture,
}

struct JobDirectory {
    path: PathBuf,
    cleaned: bool,
}

struct ChildGuard(Option<Child>);

impl ChildGuard {
    fn new(child: Child) -> Self {
        Self(Some(child))
    }

    fn child_mut(&mut self) -> Result<&mut Child, RunnerError> {
        self.0
            .as_mut()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))
    }

    fn mark_reaped(&mut self) {
        self.0 = None;
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for JobDirectory {
    fn drop(&mut self) {
        if !self.cleaned {
            let _ = remove_exact_job(&self.path);
        }
    }
}

impl JobDirectory {
    fn cleanup(mut self) -> Result<(), RunnerError> {
        remove_exact_job(&self.path)?;
        self.cleaned = true;
        Ok(())
    }
}

impl Supervisor {
    /// Stages exact bytes, supervises one worker, and validates one complete response.
    ///
    /// The caller must supply already-authorized bytes. No repository path is
    /// accepted or discovered by this API.
    ///
    /// # Errors
    ///
    /// Returns a source-free failure for invalid configuration, identity or
    /// artifact mismatch, unsafe staging, timeout, abnormal exit, excessive
    /// output, invalid framing, or a mismatched complete response.
    pub fn execute(
        &self,
        manifest: &AnalyzerExecutionManifest,
        request: &AnalyzerRunnerRequest,
        artifacts: &BTreeMap<String, Vec<u8>>,
        completed_at: &str,
    ) -> Result<SupervisedOutcome, RunnerError> {
        validate_manifest(manifest)
            .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        validate_request(request)
            .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        let _ = run_synthetic(request, manifest, artifacts, completed_at)
            .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        validate_configuration(self, manifest, request)?;
        let job = create_job_directory(&self.staging_root, &request.request_id)?;
        let execution = (|| {
            stage_artifacts(&job.path, request, artifacts)?;
            let worker = supervise_worker(self, &job.path, manifest, request, completed_at);
            verify_staged_artifacts(&job.path, request, artifacts)?;
            worker
        })();
        job.cleanup()?;
        let output = execution?;
        let bytes_staged = request
            .artifacts
            .iter()
            .try_fold(0_u64, |total, artifact| {
                artifact
                    .bytes
                    .parse::<u64>()
                    .ok()
                    .and_then(|bytes| total.checked_add(bytes))
            })
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        Ok(SupervisedOutcome {
            audit: SupervisorAudit {
                schema_name: "analyzer-supervisor-audit".to_owned(),
                schema_version: "1.0.0".to_owned(),
                request_id: request.request_id.clone(),
                profile_digest: SUPERVISOR_PROFILE_DIGEST.to_owned(),
                artifacts_staged: request.artifacts.len().to_string(),
                bytes_staged: bytes_staged.to_string(),
                outcome: match output.outcome {
                    SyntheticOutcome::Result(_) => "complete_result",
                    SyntheticOutcome::Failure(_) => "complete_failure",
                }
                .to_owned(),
                job_removed: true,
                source_retained: false,
                authority_added: false,
            },
            outcome: output.outcome,
            confinement: output.confinement,
        })
    }
}

impl MacOsVmSupervisorAction {
    /// Parses the exact source-free CLI action name.
    #[must_use]
    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "external-cancellation" => Some(Self::ExternalCancellation),
            "forced-termination-recovery" => Some(Self::ForcedTerminationRecovery),
            _ => None,
        }
    }
}

struct CapturedChild {
    child: ChildGuard,
    stdout: thread::JoinHandle<Result<Vec<u8>, RunnerError>>,
    stderr: thread::JoinHandle<Result<Vec<u8>, RunnerError>>,
}

struct VmJobCleanupGuard {
    path: PathBuf,
}

impl Drop for VmJobCleanupGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = remove_exact_job(&self.path);
        }
    }
}

impl MacOsVmSyntheticSupervisor {
    /// Runs one exact synthetic VM lifecycle action and a clean recovery job.
    ///
    /// No repository path, source bytes, command, environment, credential, or
    /// network destination is accepted by this API.
    ///
    /// # Errors
    ///
    /// Returns a source-free category if identity, readiness, cancellation,
    /// reaping, exact cleanup, controller output, or recovery validation fails.
    pub fn execute(
        &self,
        job_id: &str,
        action: MacOsVmSupervisorAction,
    ) -> Result<MacOsVmSupervisorReceipt, RunnerError> {
        self.validate(job_id)?;
        let output_root = self
            .asset_root
            .parent()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        let jobs_root = output_root.join("jobs");
        let action_job = jobs_root.join(job_id);
        let recovery_id = format!("{job_id}-recovery");
        let recovery_job = jobs_root.join(&recovery_id);
        if action_job.exists() || recovery_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }
        let _action_cleanup = VmJobCleanupGuard {
            path: action_job.clone(),
        };
        let _recovery_cleanup = VmJobCleanupGuard {
            path: recovery_job.clone(),
        };

        let mut process = self.spawn_controller(job_id, "timeout")?;
        let ready = action_job.join("controller.ready");
        wait_for_controller_ready(&mut process.child, &ready, self.timeout)?;

        let (
            external_cancellation_requested,
            controller_cancellation_verified,
            controller_forcibly_terminated,
        ) = match action {
            MacOsVmSupervisorAction::ExternalCancellation => {
                let cancellation = action_job.join("cancel.request");
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(cancellation)
                    .and_then(|mut file| file.write_all(b"IMPRESARI_VM_CANCEL_V1\n"))
                    .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
                let (status, stdout, _stderr) = wait_for_captured_child(process, self.timeout)?;
                if status.success() {
                    return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
                }
                validate_vm_cancellation(&stdout)?;
                (true, true, false)
            }
            MacOsVmSupervisorAction::ForcedTerminationRecovery => {
                force_kill_and_collect(process)?;
                (false, false, true)
            }
        };

        if action_job.exists() {
            remove_exact_job(&action_job)?;
        }
        if action_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }

        let recovery = self.spawn_controller(&recovery_id, "success")?;
        let (status, stdout, _stderr) = wait_for_captured_child(recovery, self.timeout)?;
        if !status.success() {
            return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
        }
        validate_vm_recovery(&stdout, &recovery_id)?;
        if action_job.exists() || recovery_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }

        Ok(MacOsVmSupervisorReceipt {
            schema_name: "macos-local-vm-supervisor-lifecycle-receipt".to_owned(),
            schema_version: "1.0.0".to_owned(),
            profile_id: MACOS_VM_SUPERVISOR_PROFILE_ID.to_owned(),
            profile_digest: MACOS_VM_SUPERVISOR_PROFILE_DIGEST.to_owned(),
            controller_profile_id: MACOS_VM_CONTROLLER_PROFILE_ID.to_owned(),
            controller_profile_digest: MACOS_VM_CONTROLLER_PROFILE_DIGEST.to_owned(),
            controller_digest: self.expected_controller_digest.clone(),
            job_id: job_id.to_owned(),
            action,
            controller_digest_verified_before_launch: true,
            controller_ready: true,
            external_cancellation_requested,
            controller_cancellation_verified,
            controller_forcibly_terminated,
            controller_reaped: true,
            stale_job_removed: true,
            recovery_job_succeeded: true,
            all_job_state_removed: true,
            vm_confined: false,
            production_admitted: false,
            analyzer_execution: false,
            source_retained: false,
            authority_added: false,
        })
    }

    /// Runs the exact source-free guest-resource and host-canary scenario.
    ///
    /// # Errors
    ///
    /// Returns a source-free category if identity, execution, complete output,
    /// exact resource accounting, canary denial, or cleanup validation fails.
    pub fn execute_resource_canary(
        &self,
        job_id: &str,
    ) -> Result<MacOsVmResourceCanaryReceipt, RunnerError> {
        self.validate(job_id)?;
        let output_root = self
            .asset_root
            .parent()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        let job = output_root.join("jobs").join(job_id);
        if job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }
        let _cleanup = VmJobCleanupGuard { path: job.clone() };
        let process = self.spawn_controller(job_id, "resource-canary")?;
        let (status, stdout, _stderr) = wait_for_captured_child(process, self.timeout)?;
        if !status.success() {
            return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
        }
        let receipt = validate_vm_resource_canary(&stdout, job_id)?;
        if job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }
        Ok(MacOsVmResourceCanaryReceipt {
            schema_name: "macos-local-vm-resource-canary-supervisor-receipt".to_owned(),
            schema_version: "1.0.0".to_owned(),
            profile_id: receipt.profile_id,
            profile_digest: receipt.profile_digest,
            controller_digest: self.expected_controller_digest.clone(),
            job_id: receipt.job_id,
            kernel_digest: receipt.kernel_digest,
            initramfs_digest: receipt.initramfs_digest,
            input_digest: receipt.input_digest,
            controller_digest_verified_before_launch: true,
            configuration_validated: receipt.configuration_validated,
            cpu_count: receipt.cpu_count,
            memory_bytes: receipt.memory_bytes,
            storage_devices: receipt.storage_devices,
            network_devices: receipt.network_devices,
            directory_shares: receipt.directory_shares,
            host_canary_corpus_created: receipt.host_canary_corpus_created,
            host_canary_corpus_unchanged: receipt.host_canary_corpus_unchanged,
            attached_device_set_exact: receipt.attached_device_set_exact,
            host_canary_bytes_absent: receipt.host_canary_bytes_absent,
            host_paths_absent: receipt.host_paths_absent,
            host_process_invisible: receipt.host_process_invisible,
            memory_pressure_contained: receipt.memory_pressure_contained,
            memory_oom_kills: receipt.memory_oom_kills,
            cpu_pressure_bounded: receipt.cpu_pressure_bounded,
            cpu_usage_usec: receipt.cpu_usage_usec,
            cpu_throttled_periods: receipt.cpu_throttled_periods,
            pids_peak: receipt.pids_peak,
            job_cgroup_removed: receipt.job_cgroup_removed,
            job_removed: receipt.job_removed,
            vm_confined: false,
            production_admitted: false,
            analyzer_execution: false,
            source_retained: false,
            authority_added: false,
        })
    }

    /// Exercises the production-shaped host-interruption stop path with a
    /// source-free synthetic trigger, then completes a fresh recovery VM.
    ///
    /// This automated method never claims that the host actually slept. The
    /// same controller handler is registered for macOS will-sleep delivery,
    /// but that operating-system path requires a separate manual rehearsal.
    ///
    /// # Errors
    ///
    /// Returns a source-free category if identity, readiness, interruption,
    /// reaping, exact cleanup, complete output, or recovery validation fails.
    pub fn execute_host_interruption(
        &self,
        job_id: &str,
    ) -> Result<MacOsVmHostInterruptionReceipt, RunnerError> {
        self.validate(job_id)?;
        let output_root = self
            .asset_root
            .parent()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        let jobs_root = output_root.join("jobs");
        let action_job = jobs_root.join(job_id);
        let recovery_id = format!("{job_id}-recovery");
        let recovery_job = jobs_root.join(&recovery_id);
        if action_job.exists() || recovery_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }
        let _action_cleanup = VmJobCleanupGuard {
            path: action_job.clone(),
        };
        let _recovery_cleanup = VmJobCleanupGuard {
            path: recovery_job.clone(),
        };

        let mut process = self.spawn_controller(job_id, "host-interruption")?;
        let ready = action_job.join("controller.ready");
        wait_for_controller_ready(&mut process.child, &ready, self.timeout)?;
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(action_job.join("host-interruption.request"))
            .and_then(|mut file| file.write_all(b"IMPRESARI_VM_HOST_INTERRUPTION_V1\n"))
            .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
        let (status, stdout, _stderr) = wait_for_captured_child(process, self.timeout)?;
        if !status.success() {
            return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
        }
        let interruption = validate_vm_host_interruption(&stdout, job_id)?;
        if action_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }

        let recovery = self.spawn_controller(&recovery_id, "success")?;
        let (status, stdout, _stderr) = wait_for_captured_child(recovery, self.timeout)?;
        if !status.success() {
            return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
        }
        validate_vm_recovery(&stdout, &recovery_id)?;
        if action_job.exists() || recovery_job.exists() {
            return Err(RunnerError::new(RunnerErrorCode::Staging));
        }

        Ok(MacOsVmHostInterruptionReceipt {
            schema_name: "macos-local-vm-host-interruption-receipt".to_owned(),
            schema_version: "1.0.0".to_owned(),
            profile_id: MACOS_VM_HOST_INTERRUPTION_PROFILE_ID.to_owned(),
            profile_digest: MACOS_VM_HOST_INTERRUPTION_PROFILE_DIGEST.to_owned(),
            controller_profile_id: MACOS_VM_CONTROLLER_PROFILE_ID.to_owned(),
            controller_profile_digest: MACOS_VM_CONTROLLER_PROFILE_DIGEST.to_owned(),
            controller_digest: self.expected_controller_digest.clone(),
            job_id: job_id.to_owned(),
            interruption_source: interruption.interruption_source,
            sleep_observer_installed: interruption.sleep_observer_installed,
            shared_stop_handler_used: interruption.shared_stop_handler_used,
            synthetic_interruption_requested: true,
            virtual_machine_stopped: interruption.virtual_machine_stopped,
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
        })
    }

    fn validate(&self, job_id: &str) -> Result<(), RunnerError> {
        if !cfg!(target_os = "macos")
            || self.timeout.is_zero()
            || self.timeout > Duration::from_mins(1)
            || !valid_vm_job_id(job_id)
            || !valid_sha256(&self.expected_controller_digest)
            || !self.controller.is_absolute()
            || !self.asset_root.is_absolute()
        {
            return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
        }
        let controller = fs::symlink_metadata(&self.controller)
            .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?;
        let assets = fs::symlink_metadata(&self.asset_root)
            .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        if !controller.is_file()
            || controller.file_type().is_symlink()
            || !assets.is_dir()
            || assets.file_type().is_symlink()
            || fs::canonicalize(&self.controller)
                .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?
                != self.controller
            || fs::canonicalize(&self.asset_root)
                .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?
                != self.asset_root
            || sha256(
                &fs::read(&self.controller)
                    .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?,
            ) != self.expected_controller_digest
        {
            return Err(RunnerError::new(RunnerErrorCode::WorkerIdentity));
        }
        Ok(())
    }

    fn spawn_controller(&self, job_id: &str, scenario: &str) -> Result<CapturedChild, RunnerError> {
        let output_root = self
            .asset_root
            .parent()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        let arguments = [
            self.asset_root.as_os_str(),
            OsStr::new(job_id),
            OsStr::new(scenario),
        ];
        let child = spawn_exact_process(
            &self.controller,
            output_root,
            &arguments,
            Stdio::null(),
            Stdio::piped(),
            Stdio::piped(),
        )?;
        let mut child = ChildGuard::new(child);
        let stdout = child
            .child_mut()?
            .stdout
            .take()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))?;
        let stderr = child
            .child_mut()?
            .stderr
            .take()
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))?;
        Ok(CapturedChild {
            child,
            stdout: thread::spawn(move || read_capped(stdout, MACOS_VM_STDOUT_BYTES)),
            stderr: thread::spawn(move || read_capped(stderr, MACOS_VM_STDERR_BYTES)),
        })
    }
}

fn valid_vm_job_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 20
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn wait_for_controller_ready(
    child: &mut ChildGuard,
    ready: &Path,
    timeout: Duration,
) -> Result<(), RunnerError> {
    let started = Instant::now();
    loop {
        if ready.is_file() {
            return Ok(());
        }
        if child
            .child_mut()?
            .try_wait()
            .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?
            .is_some()
        {
            return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
        }
        if started.elapsed() >= timeout {
            return Err(RunnerError::new(RunnerErrorCode::Timeout));
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn wait_for_captured_child(
    mut process: CapturedChild,
    timeout: Duration,
) -> Result<(std::process::ExitStatus, Vec<u8>, Vec<u8>), RunnerError> {
    let started = Instant::now();
    let status = loop {
        if let Some(status) = process
            .child
            .child_mut()?
            .try_wait()
            .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = process.child.child_mut()?.kill();
            let _ = process.child.child_mut()?.wait();
            process.child.mark_reaped();
            let _ = process.stdout.join();
            let _ = process.stderr.join();
            return Err(RunnerError::new(RunnerErrorCode::Timeout));
        }
        thread::sleep(Duration::from_millis(5));
    };
    process.child.mark_reaped();
    let stdout = process
        .stdout
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    let stderr = process
        .stderr
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    Ok((status, stdout, stderr))
}

fn force_kill_and_collect(mut process: CapturedChild) -> Result<(), RunnerError> {
    process
        .child
        .child_mut()?
        .kill()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?;
    process
        .child
        .child_mut()?
        .wait()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?;
    process.child.mark_reaped();
    let _ = process
        .stdout
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    let _ = process
        .stderr
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    Ok(())
}

fn validate_vm_cancellation(stdout: &[u8]) -> Result<(), RunnerError> {
    let receipt = serde_json::from_slice::<MacOsVmControllerFailure>(stdout)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if receipt.schema_name != "macos-local-vm-matrix-failure"
        || receipt.schema_version != "1.0.0"
        || receipt.profile_id != MACOS_VM_CONTROLLER_PROFILE_ID
        || receipt.profile_digest != MACOS_VM_CONTROLLER_PROFILE_DIGEST
        || receipt.category != "cancelled"
        || receipt.vm_confined
        || receipt.production_admitted
        || receipt.analyzer_execution
        || receipt.source_retained
        || receipt.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(())
}

fn validate_vm_recovery(stdout: &[u8], job_id: &str) -> Result<(), RunnerError> {
    let receipt = serde_json::from_slice::<MacOsVmControllerJobReceipt>(stdout)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if receipt.schema_name != "macos-local-vm-matrix-job-receipt"
        || receipt.schema_version != "1.0.0"
        || receipt.profile_id != MACOS_VM_CONTROLLER_PROFILE_ID
        || receipt.profile_digest != MACOS_VM_CONTROLLER_PROFILE_DIGEST
        || receipt.job_id != job_id
        || receipt.result != "feasibility_passed"
        || receipt.kernel_digest != MACOS_VM_KERNEL_DIGEST
        || receipt.initramfs_digest != MACOS_VM_INITRAMFS_DIGEST
        || !valid_sha256(&receipt.input_digest)
        || !receipt.virtualization_supported
        || !receipt.configuration_validated
        || receipt.cpu_count != "1"
        || receipt.memory_bytes != "268435456"
        || receipt.serial_ports != "1"
        || receipt.storage_devices != "2"
        || receipt.network_devices != "0"
        || receipt.directory_shares != "0"
        || receipt.graphics_devices != "0"
        || receipt.audio_devices != "0"
        || receipt.input_devices != "0"
        || !receipt.exact_input_verified
        || !receipt.read_only_input_verified
        || !receipt.scratch_initially_clean
        || !receipt.scratch_capacity_verified
        || !receipt.network_device_absent
        || !receipt.job_removed
        || receipt.vm_confined
        || receipt.production_admitted
        || receipt.source_retained
        || receipt.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(())
}

fn validate_vm_resource_canary(
    stdout: &[u8],
    job_id: &str,
) -> Result<MacOsVmControllerResourceCanaryReceipt, RunnerError> {
    let receipt = serde_json::from_slice::<MacOsVmControllerResourceCanaryReceipt>(stdout)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let oom_kills = parse_canonical_u64(&receipt.memory_oom_kills)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let cpu_usage = parse_canonical_u64(&receipt.cpu_usage_usec)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let throttled = parse_canonical_u64(&receipt.cpu_throttled_periods)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let pids_peak = parse_canonical_u64(&receipt.pids_peak)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if receipt.schema_name != "macos-local-vm-resource-canary-receipt"
        || receipt.schema_version != "1.0.0"
        || receipt.profile_id != MACOS_VM_RESOURCE_CANARY_PROFILE_ID
        || receipt.profile_digest != MACOS_VM_RESOURCE_CANARY_PROFILE_DIGEST
        || receipt.job_id != job_id
        || receipt.result != "partial_resource_canary_passed"
        || receipt.kernel_digest != MACOS_VM_KERNEL_DIGEST
        || receipt.initramfs_digest != MACOS_VM_RESOURCE_INITRAMFS_DIGEST
        || receipt.input_digest != MACOS_VM_SYNTHETIC_INPUT_DIGEST
        || !receipt.virtualization_supported
        || !receipt.configuration_validated
        || receipt.cpu_count != "1"
        || receipt.memory_bytes != "268435456"
        || receipt.storage_devices != "2"
        || receipt.network_devices != "0"
        || receipt.directory_shares != "0"
        || !receipt.host_canary_corpus_created
        || !receipt.host_canary_corpus_unchanged
        || !receipt.attached_device_set_exact
        || !receipt.host_canary_bytes_absent
        || !receipt.host_paths_absent
        || !receipt.host_process_invisible
        || !receipt.memory_pressure_contained
        || oom_kills != 1
        || !receipt.cpu_pressure_bounded
        || !(50_000..=400_000).contains(&cpu_usage)
        || throttled == 0
        || pids_peak != 1
        || !receipt.job_cgroup_removed
        || !receipt.job_removed
        || receipt.vm_confined
        || receipt.production_admitted
        || receipt.analyzer_execution
        || receipt.source_retained
        || receipt.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(receipt)
}

fn validate_vm_host_interruption(
    stdout: &[u8],
    job_id: &str,
) -> Result<MacOsVmControllerInterruptionReceipt, RunnerError> {
    let receipt = serde_json::from_slice::<MacOsVmControllerInterruptionReceipt>(stdout)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if receipt.schema_name != "macos-local-vm-interruption-controller-receipt"
        || receipt.schema_version != "1.0.0"
        || receipt.profile_id != MACOS_VM_CONTROLLER_PROFILE_ID
        || receipt.profile_digest != MACOS_VM_CONTROLLER_PROFILE_DIGEST
        || receipt.job_id != job_id
        || receipt.result != "synthetic_interruption_handled"
        || receipt.interruption_source != "synthetic-job-private-trigger"
        || !receipt.sleep_observer_installed
        || !receipt.shared_stop_handler_used
        || !receipt.virtualization_supported
        || !receipt.configuration_validated
        || !receipt.virtual_machine_stopped
        || !receipt.job_removed
        || receipt.real_host_sleep_observed
        || receipt.vm_confined
        || receipt.production_admitted
        || receipt.analyzer_execution
        || receipt.source_retained
        || receipt.authority_added
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(receipt)
}

fn supervise_worker(
    supervisor: &Supervisor,
    job: &Path,
    manifest: &AnalyzerExecutionManifest,
    request: &AnalyzerRunnerRequest,
    completed_at: &str,
) -> Result<WorkerOutput, RunnerError> {
    let control = WorkerControl {
        manifest: manifest.clone(),
        request: request.clone(),
        completed_at: completed_at.to_owned(),
    };
    let payload = serde_json_canonicalizer::to_vec(&control)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
    if payload.len() > MAX_CONTROL_BYTES {
        return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
    }
    let child = spawn_exact_process(
        &supervisor.executable,
        job,
        &[],
        Stdio::piped(),
        Stdio::piped(),
        Stdio::piped(),
    )?;
    let mut child = ChildGuard::new(child);
    let mut stdin = child
        .child_mut()?
        .stdin
        .take()
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))?;
    write_frame(&mut stdin, &payload, MAX_CONTROL_BYTES)?;
    drop(stdin);
    let stdout = child
        .child_mut()?
        .stdout
        .take()
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))?;
    let stderr = child
        .child_mut()?
        .stderr
        .take()
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::Process))?;
    let stdout_reader = thread::spawn(move || read_capped(stdout, MAX_OUTPUT_BYTES + 5));
    let stderr_reader = thread::spawn(move || read_capped(stderr, STDERR_BYTES));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .child_mut()?
            .try_wait()
            .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?
        {
            break status;
        }
        if started.elapsed() >= supervisor.timeout {
            let _ = child.child_mut()?.kill();
            let _ = child.child_mut()?.wait();
            child.mark_reaped();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(RunnerError::new(RunnerErrorCode::Timeout));
        }
        thread::sleep(Duration::from_millis(2));
    };
    child.mark_reaped();
    let stdout = stdout_reader
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    let _stderr = stderr_reader
        .join()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))??;
    if !status.success() {
        return Err(RunnerError::new(RunnerErrorCode::WorkerFailure));
    }
    let payload = read_single_frame(&stdout, MAX_OUTPUT_BYTES)?;
    let output = serde_json::from_slice::<WorkerOutput>(&payload)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if output.confinement != ConfinementPosture::iar1() {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    match &output.outcome {
        SyntheticOutcome::Result(result) => validate_result(result, request, manifest),
        SyntheticOutcome::Failure(failure) => validate_failure(failure, request),
    }
    .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    Ok(output)
}

fn spawn_exact_process(
    executable: &Path,
    current_directory: &Path,
    arguments: &[&OsStr],
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
) -> Result<Child, RunnerError> {
    let mut command = Command::new(executable);
    command
        .current_dir(current_directory)
        .env_clear()
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr);
    command.args(arguments);
    command
        .spawn()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))
}

/// Runs one short-lived synthetic worker request over standard input/output.
///
/// The worker reads only the job-private `input` directory under its current
/// directory. It accepts no arguments, environment configuration, repository
/// path, command, endpoint, credential, analyzer, or parser selection.
///
/// # Errors
///
/// Returns a source-free I/O or validation error. The binary maps every error
/// to a nonzero exit without printing details.
pub fn run_worker_stdio() -> Result<(), RunnerError> {
    let frame = read_stream_capped(io::stdin().lock(), MAX_CONTROL_BYTES + 4)?;
    let payload = read_single_frame(&frame, MAX_CONTROL_BYTES)?;
    let control = serde_json::from_slice::<WorkerControl>(&payload)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
    validate_manifest(&control.manifest)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
    validate_request(&control.request)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
    let artifacts = read_staged_artifacts(Path::new("."), &control.request)?;

    match control.request.synthetic_behavior.as_str() {
        "crash" => return Err(RunnerError::new(RunnerErrorCode::WorkerFailure)),
        "timeout" => thread::sleep(Duration::from_mins(1)),
        "output_flood" => {
            io::stdout()
                .lock()
                .write_all(&vec![b'x'; MAX_OUTPUT_BYTES + 6])
                .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?;
            return Ok(());
        }
        "malformed_output" => {
            write_frame(&mut io::stdout().lock(), b"{}", MAX_OUTPUT_BYTES)?;
            return Ok(());
        }
        "input_mutation" => {
            let first = control
                .request
                .artifacts
                .first()
                .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
            let path = staged_path(Path::new("."), &first.artifact_hash)?;
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(path)
                .and_then(|mut file| file.write_all(b"mutated"))
                .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
            return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
        }
        "no_op" => {}
        _ => return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration)),
    }

    let outcome = run_synthetic(
        &control.request,
        &control.manifest,
        &artifacts,
        &control.completed_at,
    )
    .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let output = WorkerOutput {
        outcome,
        confinement: ConfinementPosture::iar1(),
    };
    let payload = serde_json_canonicalizer::to_vec(&output)
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    write_frame(&mut io::stdout().lock(), &payload, MAX_OUTPUT_BYTES)
}

fn validate_configuration(
    supervisor: &Supervisor,
    manifest: &AnalyzerExecutionManifest,
    request: &AnalyzerRunnerRequest,
) -> Result<(), RunnerError> {
    if supervisor.timeout.is_zero()
        || supervisor.timeout > Duration::from_mins(1)
        || !supervisor.executable.is_absolute()
        || !supervisor.staging_root.is_absolute()
        || request.manifest_id != manifest.manifest_id
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
    }
    let executable = fs::symlink_metadata(&supervisor.executable)
        .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?;
    if !executable.is_file() || executable.file_type().is_symlink() {
        return Err(RunnerError::new(RunnerErrorCode::WorkerIdentity));
    }
    if fs::canonicalize(&supervisor.executable)
        .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?
        != supervisor.executable
    {
        return Err(RunnerError::new(RunnerErrorCode::WorkerIdentity));
    }
    let executable_bytes = fs::read(&supervisor.executable)
        .map_err(|_| RunnerError::new(RunnerErrorCode::WorkerIdentity))?;
    if sha256(&executable_bytes) != manifest.executable_digest {
        return Err(RunnerError::new(RunnerErrorCode::WorkerIdentity));
    }
    validate_staging_root(&supervisor.staging_root, &supervisor.excluded_roots)
}

fn validate_staging_root(root: &Path, excluded_roots: &[PathBuf]) -> Result<(), RunnerError> {
    let metadata =
        fs::symlink_metadata(root).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(RunnerError::new(RunnerErrorCode::Staging));
    }
    if excluded_roots.is_empty() {
        return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
    }
    let canonical_root =
        fs::canonicalize(root).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
    if canonical_root != root {
        return Err(RunnerError::new(RunnerErrorCode::Staging));
    }
    for excluded in excluded_roots {
        if !excluded.is_absolute() {
            return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
        }
        let canonical_excluded = fs::canonicalize(excluded)
            .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
        if canonical_root.starts_with(&canonical_excluded)
            || canonical_excluded.starts_with(&canonical_root)
        {
            return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
        }
    }
    private_root_permissions(root)?;
    Ok(())
}

fn create_job_directory(root: &Path, request_id: &str) -> Result<JobDirectory, RunnerError> {
    let path = root.join(format!("job-{request_id}"));
    fs::create_dir(&path).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
    let job = JobDirectory {
        path,
        cleaned: false,
    };
    set_private_directory(&job.path)?;
    let input = job.path.join("input");
    fs::create_dir(&input).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
    set_private_directory(&input)?;
    Ok(job)
}

fn remove_exact_job(path: &Path) -> Result<(), RunnerError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
    if metadata.file_type().is_symlink() {
        fs::remove_file(path).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
        return Err(RunnerError::new(RunnerErrorCode::Staging));
    }
    if !metadata.is_dir() {
        return Err(RunnerError::new(RunnerErrorCode::Staging));
    }
    fs::remove_dir_all(path).map_err(|_| RunnerError::new(RunnerErrorCode::Staging))
}

fn stage_artifacts(
    job: &Path,
    request: &AnalyzerRunnerRequest,
    artifacts: &BTreeMap<String, Vec<u8>>,
) -> Result<(), RunnerError> {
    if artifacts.len() != request.artifacts.len() {
        return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
    }
    for descriptor in &request.artifacts {
        let bytes = artifacts
            .get(&descriptor.artifact_hash)
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
        if descriptor.bytes.parse::<usize>().ok() != Some(bytes.len())
            || sha256(bytes) != descriptor.artifact_hash
        {
            return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
        }
        let path = staged_path(job, &descriptor.artifact_hash)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?;
        set_private_file(&path)?;
    }
    verify_staged_artifacts(job, request, artifacts)
}

fn verify_staged_artifacts(
    job: &Path,
    request: &AnalyzerRunnerRequest,
    artifacts: &BTreeMap<String, Vec<u8>>,
) -> Result<(), RunnerError> {
    for descriptor in &request.artifacts {
        let expected = artifacts
            .get(&descriptor.artifact_hash)
            .ok_or_else(|| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
        let path = staged_path(job, &descriptor.artifact_hash)?;
        let metadata = fs::symlink_metadata(&path)
            .map_err(|_| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
        }
        let actual =
            fs::read(path).map_err(|_| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
        if actual != *expected || sha256(&actual) != descriptor.artifact_hash {
            return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
        }
    }
    Ok(())
}

fn read_staged_artifacts(
    job: &Path,
    request: &AnalyzerRunnerRequest,
) -> Result<BTreeMap<String, Vec<u8>>, RunnerError> {
    request
        .artifacts
        .iter()
        .map(|descriptor| {
            let path = staged_path(job, &descriptor.artifact_hash)?;
            let metadata = fs::symlink_metadata(&path)
                .map_err(|_| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
            }
            let bytes =
                fs::read(path).map_err(|_| RunnerError::new(RunnerErrorCode::ArtifactMismatch))?;
            if descriptor.bytes.parse::<usize>().ok() != Some(bytes.len())
                || sha256(&bytes) != descriptor.artifact_hash
            {
                return Err(RunnerError::new(RunnerErrorCode::ArtifactMismatch));
            }
            Ok((descriptor.artifact_hash.clone(), bytes))
        })
        .collect()
}

fn staged_path(job: &Path, digest: &str) -> Result<PathBuf, RunnerError> {
    let name = digest
        .strip_prefix("sha256:")
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidConfiguration))?;
    Ok(job.join("input").join(name))
}

fn write_frame(writer: &mut impl Write, payload: &[u8], maximum: usize) -> Result<(), RunnerError> {
    if payload.len() > maximum {
        return Err(RunnerError::new(RunnerErrorCode::OutputLimit));
    }
    let length =
        u32::try_from(payload.len()).map_err(|_| RunnerError::new(RunnerErrorCode::OutputLimit))?;
    writer
        .write_all(&length.to_be_bytes())
        .and_then(|()| writer.write_all(payload))
        .and_then(|()| writer.flush())
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))
}

fn read_single_frame(frame: &[u8], maximum: usize) -> Result<Vec<u8>, RunnerError> {
    let prefix: [u8; 4] = frame
        .get(..4)
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidOutput))?
        .try_into()
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    let length = usize::try_from(u32::from_be_bytes(prefix))
        .map_err(|_| RunnerError::new(RunnerErrorCode::OutputLimit))?;
    if length > maximum {
        return Err(RunnerError::new(RunnerErrorCode::OutputLimit));
    }
    let payload = frame
        .get(4..)
        .ok_or_else(|| RunnerError::new(RunnerErrorCode::InvalidOutput))?;
    if payload.len() != length {
        return Err(RunnerError::new(RunnerErrorCode::InvalidOutput));
    }
    Ok(payload.to_vec())
}

fn read_capped(mut reader: impl Read, maximum: usize) -> Result<Vec<u8>, RunnerError> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take(u64::try_from(maximum).unwrap_or(u64::MAX).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?;
    if bytes.len() > maximum {
        return Err(RunnerError::new(RunnerErrorCode::OutputLimit));
    }
    Ok(bytes)
}

fn read_stream_capped(reader: impl Read, maximum: usize) -> Result<Vec<u8>, RunnerError> {
    read_capped(reader, maximum)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex = digest
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            use fmt::Write as _;
            write!(output, "{byte:02x}").expect("writing to a string cannot fail");
            output
        });
    format!("sha256:{hex}")
}

fn parse_canonical_u64(value: &str) -> Result<u64, RunnerError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(RunnerError::new(RunnerErrorCode::InvalidConfiguration));
    }
    value
        .parse::<u64>()
        .map_err(|_| RunnerError::new(RunnerErrorCode::InvalidConfiguration))
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn valid_identifier(value: &str) -> bool {
    value.match_indices('_').any(|(delimiter, _)| {
        let prefix = &value[..delimiter];
        let suffix = &value[delimiter + 1..];
        !prefix.is_empty()
            && prefix.len() <= 32
            && prefix
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_lowercase())
            && prefix.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
            })
            && (8..=128).contains(&suffix.len())
            && suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    })
}

#[cfg(unix)]
fn set_private_directory(path: &Path) -> Result<(), RunnerError> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn set_private_directory(_path: &Path) -> Result<(), RunnerError> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file(path: &Path) -> Result<(), RunnerError> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn set_private_file(_path: &Path) -> Result<(), RunnerError> {
    Ok(())
}

#[cfg(unix)]
fn private_root_permissions(path: &Path) -> Result<(), RunnerError> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode = fs::metadata(path)
        .map_err(|_| RunnerError::new(RunnerErrorCode::Staging))?
        .permissions()
        .mode();
    if mode.trailing_zeros() >= 6 {
        Ok(())
    } else {
        Err(RunnerError::new(RunnerErrorCode::Staging))
    }
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn private_root_permissions(_path: &Path) -> Result<(), RunnerError> {
    Ok(())
}
