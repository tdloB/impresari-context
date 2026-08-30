// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Application-enforced synthetic staging and supervision for ADR-0074 IAR-1A."]
// Independent measured controls must remain explicit so unsupported isolation
// claims cannot be inferred from one aggregate state.
#![allow(clippy::struct_excessive_bools)]

use std::{
    collections::BTreeMap,
    error::Error,
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
    let child = Command::new(&supervisor.executable)
        .current_dir(job)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| RunnerError::new(RunnerErrorCode::Process))?;
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
