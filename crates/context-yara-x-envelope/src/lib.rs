// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "ADR-0101 test-only synthetic runner-to-adapter envelope composition."]
#![allow(clippy::struct_excessive_bools)]

use std::{
    error::Error,
    fmt,
    fmt::Write as _,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use context_analyzer_runner::{
    YaraXSyntheticProcessCapture, YaraXSyntheticProcessConfig, capture_yara_x_synthetic_process,
};
use context_yara_x_adapter::{AdapterControl, NormalizedResult, normalize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Frozen synthetic envelope profile identifier.
pub const PROFILE_ID: &str = "yara-x-synthetic-runner-envelope-v1";
/// Digest of the exact committed profile bytes.
pub const PROFILE_DIGEST: &str =
    "sha256:356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b";
/// Exact original-synthetic match record embedded in the emitter.
pub const VALID_MATCH: &[u8] = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"SyntheticMarker\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$marker\",\"match\":\" ... 12 more bytes\",\"offset\":8}],\"tags\":[\"synthetic\",\"contract\"]}]}\n";
/// Exact original-synthetic no-match record embedded in the emitter.
pub const VALID_NO_MATCH: &[u8] = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n";

const MAX_CONTROL_BYTES: usize = 32_768;
const MAX_OUTPUT_BYTES: usize = 2_228_224;
const RECEIPT_ID_DOMAIN: &[u8] = b"impresari-context/yara-x-synthetic-envelope/v1\0";

/// Stable source-free envelope failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnvelopeErrorCode {
    /// Control metadata or a closed identity was invalid.
    InvalidControl,
    /// Synthetic process launch or confinement failed.
    Process,
    /// Captured output identity or accounting did not match.
    CaptureMismatch,
    /// The pure adapter rejected the complete captured record.
    Adapter,
    /// Exact job or cgroup cleanup failed.
    Cleanup,
    /// Serialization or bounded transport failed.
    Serialization,
}

/// Source-free ADR-0101 error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnvelopeError(EnvelopeErrorCode);

impl EnvelopeError {
    const fn new(code: EnvelopeErrorCode) -> Self {
        Self(code)
    }

    /// Returns the stable error category.
    #[must_use]
    pub const fn code(self) -> EnvelopeErrorCode {
        self.0
    }
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            EnvelopeErrorCode::InvalidControl => "invalid synthetic envelope control",
            EnvelopeErrorCode::Process => "synthetic envelope process failed",
            EnvelopeErrorCode::CaptureMismatch => "synthetic envelope capture mismatch",
            EnvelopeErrorCode::Adapter => "synthetic envelope adapter failed",
            EnvelopeErrorCode::Cleanup => "synthetic envelope cleanup failed",
            EnvelopeErrorCode::Serialization => "synthetic envelope serialization failed",
        })
    }
}

impl Error for EnvelopeError {}

/// Source-free adapter control carried by the closed envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterControlRecord {
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact synthetic manifest identity.
    pub manifest_id: String,
    /// Exact synthetic artifact identity.
    pub artifact_hash: String,
    /// Exact synthetic artifact length.
    pub artifact_bytes: String,
    /// Exact vendor-shaped staged path.
    pub expected_staged_path: String,
    /// Exact YARA-X compatibility-evidence identity; not an admitted artifact.
    pub executable_digest: String,
    /// Exact synthetic ruleset evidence identity; not an admitted ruleset.
    pub ruleset_digest: String,
    /// Caller-supplied canonical completion time.
    pub completed_at: String,
}

/// Closed source-free envelope control.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvelopeControl {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Fresh synthetic job identifier.
    pub job_id: String,
    /// Frozen envelope profile identifier.
    pub profile_id: String,
    /// Exact envelope profile digest.
    pub profile_digest: String,
    /// Exact synthetic emitter identity.
    pub emitter_digest: String,
    /// Exact isolation launcher identity.
    pub launcher_digest: String,
    /// Closed original-synthetic case identifier.
    pub case_id: String,
    /// Expected exact stdout bytes.
    pub expected_stdout_bytes: String,
    /// Expected exact stdout digest.
    pub expected_stdout_digest: String,
    /// Pure adapter control identities.
    pub adapter: AdapterControlRecord,
    /// Always false.
    pub authority_added: bool,
}

/// Test-only live path bindings supplied to the coordinator over stdin.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveEnvelopeControl {
    /// Source-free envelope control.
    pub envelope: EnvelopeControl,
    /// Exact staged launcher path.
    pub launcher_path: PathBuf,
    /// Exact staged emitter path.
    pub emitter_path: PathBuf,
    /// Exact synthetic job root.
    pub job_root: PathBuf,
    /// Fresh delegated cgroup leaf.
    pub cgroup_leaf: PathBuf,
    /// Synthetic external canary.
    pub external_canary: PathBuf,
    /// Synthetic credential-shaped canary.
    pub credential_canary: PathBuf,
    /// Nonexistent in-job write probe.
    pub write_probe: PathBuf,
}

/// Source-free successful composition receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionReceipt {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated receipt identity.
    pub receipt_id: String,
    /// Fresh synthetic job identifier.
    pub job_id: String,
    /// Frozen envelope profile identifier.
    pub profile_id: String,
    /// Exact envelope profile digest.
    pub profile_digest: String,
    /// Exact synthetic emitter identity.
    pub emitter_digest: String,
    /// Exact isolation launcher identity.
    pub launcher_digest: String,
    /// Exact captured stdout identity.
    pub stdout_digest: String,
    /// Exact captured stdout byte length.
    pub stdout_bytes: String,
    /// Exact normalized result identity.
    pub normalized_result_id: String,
    /// Closed original-synthetic case identifier.
    pub case_id: String,
    /// The Impresari synthetic emitter executed.
    pub synthetic_emitter_executed: bool,
    /// The emitter process was OS-confined by the admitted Linux boundary.
    pub synthetic_emitter_os_confined: bool,
    /// The emitter produced no stderr bytes.
    pub emitter_stderr_empty: bool,
    /// Complete captured bytes were parsed in memory.
    pub in_memory_composition_complete: bool,
    /// Raw stdout was not retained.
    pub raw_output_retained: bool,
    /// Exact job storage was removed.
    pub job_removed: bool,
    /// Exact cgroup leaf was removed.
    pub cgroup_removed: bool,
    /// Always false; the emitter is not YARA-X.
    pub yara_x_executed: bool,
    /// Always false; no analyzer ran.
    pub analyzer_executed: bool,
    /// Always false.
    pub production_admitted: bool,
    /// Always false.
    pub iar_2_admitted: bool,
    /// Always false.
    pub detection_quality_claimed: bool,
    /// Always false.
    pub safety_claimed: bool,
    /// Always false.
    pub authority_added: bool,
}

/// Complete source-free coordinator output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionOutput {
    /// Exact receipt.
    pub receipt: CompositionReceipt,
    /// Exact pure normalized result.
    pub normalized_result: NormalizedResult,
}

/// Returns the exact embedded original-synthetic record for one closed case.
///
/// # Errors
///
/// Returns `InvalidControl` for every value outside the two frozen cases.
pub fn synthetic_record(case_id: &str) -> Result<&'static [u8], EnvelopeError> {
    match case_id {
        "valid-match" => Ok(VALID_MATCH),
        "valid-no-match" => Ok(VALID_NO_MATCH),
        _ => Err(EnvelopeError::new(EnvelopeErrorCode::InvalidControl)),
    }
}

/// Composes one already captured synthetic process result with the pure adapter.
///
/// # Errors
///
/// Fails without a partial result for every control, capture, or parser mismatch.
pub fn compose_capture(
    control: &EnvelopeControl,
    capture: &YaraXSyntheticProcessCapture,
    job_removed: bool,
    cgroup_removed: bool,
) -> Result<CompositionOutput, EnvelopeError> {
    validate_control(control)?;
    let expected_bytes = control
        .expected_stdout_bytes
        .parse::<u64>()
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::InvalidControl))?;
    if capture.emitter_digest != control.emitter_digest
        || capture.launcher_digest != control.launcher_digest
        || capture.stdout_bytes != expected_bytes
        || capture.stdout_digest != control.expected_stdout_digest
        || sha256(&capture.stdout) != control.expected_stdout_digest
        || capture.stdout.as_slice() != synthetic_record(&control.case_id)?
        || !capture.emitter_succeeded
        || !capture.emitter_stderr_empty
        || !capture.atomic_cgroup_placement
        || !capture.landlock_read_only_job
        || !capture.network_denied
        || !capture.unrelated_descriptors_closed
        || !job_removed
        || !cgroup_removed
    {
        return Err(EnvelopeError::new(EnvelopeErrorCode::CaptureMismatch));
    }
    let artifact_bytes = control
        .adapter
        .artifact_bytes
        .parse::<u64>()
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::InvalidControl))?;
    let normalized_result = normalize(
        &capture.stdout,
        &AdapterControl {
            profile_id: context_yara_x_adapter::PROFILE_ID.to_owned(),
            profile_digest: context_yara_x_adapter::PROFILE_DIGEST.to_owned(),
            workspace_snapshot: control.adapter.workspace_snapshot.clone(),
            manifest_id: control.adapter.manifest_id.clone(),
            artifact_hash: control.adapter.artifact_hash.clone(),
            artifact_bytes,
            expected_staged_path: control.adapter.expected_staged_path.clone(),
            executable_digest: control.adapter.executable_digest.clone(),
            ruleset_digest: control.adapter.ruleset_digest.clone(),
            completed_at: control.adapter.completed_at.clone(),
        },
    )
    .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Adapter))?;

    let receipt_id = receipt_identity(control, &normalized_result.result_id)?;
    Ok(CompositionOutput {
        receipt: CompositionReceipt {
            schema_name: "yara-x-synthetic-runner-envelope-receipt".to_owned(),
            schema_version: "1.0.0".to_owned(),
            receipt_id,
            job_id: control.job_id.clone(),
            profile_id: PROFILE_ID.to_owned(),
            profile_digest: PROFILE_DIGEST.to_owned(),
            emitter_digest: control.emitter_digest.clone(),
            launcher_digest: control.launcher_digest.clone(),
            stdout_digest: control.expected_stdout_digest.clone(),
            stdout_bytes: control.expected_stdout_bytes.clone(),
            normalized_result_id: normalized_result.result_id.clone(),
            case_id: control.case_id.clone(),
            synthetic_emitter_executed: true,
            synthetic_emitter_os_confined: true,
            emitter_stderr_empty: true,
            in_memory_composition_complete: true,
            raw_output_retained: false,
            job_removed: true,
            cgroup_removed: true,
            yara_x_executed: false,
            analyzer_executed: false,
            production_admitted: false,
            iar_2_admitted: false,
            detection_quality_claimed: false,
            safety_claimed: false,
            authority_added: false,
        },
        normalized_result,
    })
}

/// Runs one live synthetic envelope inside the configured admitted Linux boundary.
///
/// # Errors
///
/// Returns a source-free category and removes the exact synthetic job and cgroup
/// leaf on every path where they remain removable.
pub fn execute_live(control: &LiveEnvelopeControl) -> Result<CompositionOutput, EnvelopeError> {
    validate_live_paths(control)?;
    let cleanup = CleanupGuard::new(control.job_root.clone(), control.cgroup_leaf.clone());
    let capture = capture_yara_x_synthetic_process(&YaraXSyntheticProcessConfig {
        launcher: control.launcher_path.clone(),
        launcher_digest: control.envelope.launcher_digest.clone(),
        emitter: control.emitter_path.clone(),
        emitter_digest: control.envelope.emitter_digest.clone(),
        job_root: control.job_root.clone(),
        cgroup_leaf: control.cgroup_leaf.clone(),
        external_canary: control.external_canary.clone(),
        credential_canary: control.credential_canary.clone(),
        write_probe: control.write_probe.clone(),
        case_id: control.envelope.case_id.clone(),
        timeout: Duration::from_secs(10),
    })
    .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Process))?;
    cleanup.finish()?;
    compose_capture(&control.envelope, &capture, true, true)
}

/// Runs the bounded coordinator JSON transport on stdin/stdout.
///
/// # Errors
///
/// Returns a source-free category for framing, execution, composition, or output failure.
pub fn run_coordinator_stdio() -> Result<(), EnvelopeError> {
    let mut input = Vec::new();
    io::stdin()
        .lock()
        .take((MAX_CONTROL_BYTES + 1) as u64)
        .read_to_end(&mut input)
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::InvalidControl))?;
    if input.is_empty() || input.len() > MAX_CONTROL_BYTES {
        return Err(EnvelopeError::new(EnvelopeErrorCode::InvalidControl));
    }
    let control = serde_json::from_slice::<LiveEnvelopeControl>(&input)
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::InvalidControl))?;
    let output = execute_live(&control)?;
    let bytes = serde_json::to_vec(&output)
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Serialization))?;
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(EnvelopeError::new(EnvelopeErrorCode::Serialization));
    }
    io::stdout()
        .lock()
        .write_all(&bytes)
        .and_then(|()| io::stdout().lock().write_all(b"\n"))
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Serialization))
}

fn validate_control(control: &EnvelopeControl) -> Result<(), EnvelopeError> {
    let record = synthetic_record(&control.case_id)?;
    if control.schema_name != "yara-x-synthetic-runner-envelope-control"
        || control.schema_version != "1.0.0"
        || control.profile_id != PROFILE_ID
        || control.profile_digest != PROFILE_DIGEST
        || !valid_identifier(&control.job_id)
        || !valid_sha256(&control.emitter_digest)
        || !valid_sha256(&control.launcher_digest)
        || control.expected_stdout_bytes != record.len().to_string()
        || control.expected_stdout_digest != sha256(record)
        || control.adapter.expected_staged_path != "/staged/artifact.bin"
        || control.authority_added
    {
        return Err(EnvelopeError::new(EnvelopeErrorCode::InvalidControl));
    }
    Ok(())
}

fn validate_live_paths(control: &LiveEnvelopeControl) -> Result<(), EnvelopeError> {
    let expected_job = format!("job-{}", control.envelope.job_id);
    if !control.job_root.is_absolute()
        || control
            .job_root
            .file_name()
            .and_then(|value| value.to_str())
            != Some(&expected_job)
        || control.launcher_path.parent() != Some(control.job_root.as_path())
        || control.emitter_path.parent() != Some(control.job_root.as_path())
        || control.write_probe.parent() != Some(control.job_root.as_path())
        || control.external_canary.starts_with(&control.job_root)
        || control.credential_canary.starts_with(&control.job_root)
    {
        return Err(EnvelopeError::new(EnvelopeErrorCode::InvalidControl));
    }
    Ok(())
}

fn receipt_identity(control: &EnvelopeControl, result_id: &str) -> Result<String, EnvelopeError> {
    let bytes = serde_json::to_vec(&(control, result_id))
        .map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Serialization))?;
    let mut hasher = Sha256::new();
    hasher.update(RECEIPT_ID_DOMAIN);
    hasher.update(bytes);
    Ok(hex_digest(&hasher.finalize()))
}

fn sha256(bytes: &[u8]) -> String {
    hex_digest(&Sha256::digest(bytes))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

struct CleanupGuard {
    job: PathBuf,
    cgroup: PathBuf,
    finished: bool,
}

impl CleanupGuard {
    fn new(job: PathBuf, cgroup: PathBuf) -> Self {
        Self {
            job,
            cgroup,
            finished: false,
        }
    }

    fn finish(mut self) -> Result<(), EnvelopeError> {
        remove_job(&self.job)?;
        fs::remove_dir(&self.cgroup).map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Cleanup))?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if !self.finished {
            let _ = remove_job(&self.job);
            let _ = fs::remove_dir(&self.cgroup);
        }
    }
}

fn remove_job(path: &Path) -> Result<(), EnvelopeError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Cleanup))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(EnvelopeError::new(EnvelopeErrorCode::Cleanup));
    }
    fs::remove_dir_all(path).map_err(|_| EnvelopeError::new(EnvelopeErrorCode::Cleanup))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control(case_id: &str) -> EnvelopeControl {
        let record = synthetic_record(case_id).expect("closed case");
        EnvelopeControl {
            schema_name: "yara-x-synthetic-runner-envelope-control".to_owned(),
            schema_version: "1.0.0".to_owned(),
            job_id: format!("synthetic-{case_id}"),
            profile_id: PROFILE_ID.to_owned(),
            profile_digest: PROFILE_DIGEST.to_owned(),
            emitter_digest: format!("sha256:{}", "e".repeat(64)),
            launcher_digest: format!("sha256:{}", "f".repeat(64)),
            case_id: case_id.to_owned(),
            expected_stdout_bytes: record.len().to_string(),
            expected_stdout_digest: sha256(record),
            adapter: AdapterControlRecord {
                workspace_snapshot: format!("sha256:{}", "d".repeat(64)),
                manifest_id: format!("sha256:{}", "a".repeat(64)),
                artifact_hash: format!("sha256:{}", "1".repeat(64)),
                artifact_bytes: "64".to_owned(),
                expected_staged_path: "/staged/artifact.bin".to_owned(),
                executable_digest: format!("sha256:{}", "b".repeat(64)),
                ruleset_digest: format!("sha256:{}", "c".repeat(64)),
                completed_at: "2026-08-31T00:00:00Z".to_owned(),
            },
            authority_added: false,
        }
    }

    fn capture(control: &EnvelopeControl) -> YaraXSyntheticProcessCapture {
        let stdout = synthetic_record(&control.case_id).expect("record").to_vec();
        YaraXSyntheticProcessCapture {
            stdout_bytes: stdout.len() as u64,
            stdout_digest: sha256(&stdout),
            stdout,
            emitter_digest: control.emitter_digest.clone(),
            launcher_digest: control.launcher_digest.clone(),
            emitter_succeeded: true,
            emitter_stderr_empty: true,
            atomic_cgroup_placement: true,
            landlock_read_only_job: true,
            network_denied: true,
            unrelated_descriptors_closed: true,
        }
    }

    #[test]
    fn composes_both_closed_cases_without_execution_overclaim() {
        for case_id in ["valid-match", "valid-no-match"] {
            let control = control(case_id);
            let output = compose_capture(&control, &capture(&control), true, true)
                .expect("complete synthetic composition");
            assert!(output.receipt.synthetic_emitter_executed);
            assert!(output.receipt.synthetic_emitter_os_confined);
            assert!(!output.receipt.yara_x_executed);
            assert!(!output.receipt.analyzer_executed);
            assert!(!output.receipt.production_admitted);
            assert!(!output.receipt.iar_2_admitted);
            assert!(!output.receipt.safety_claimed);
            assert!(!output.receipt.authority_added);
        }
    }

    #[test]
    fn capture_and_cleanup_mismatches_fail_without_partial_output() {
        let control = control("valid-match");
        let mut changed = capture(&control);
        changed.stdout.push(b'x');
        assert_eq!(
            compose_capture(&control, &changed, true, true)
                .expect_err("mutated output")
                .code(),
            EnvelopeErrorCode::CaptureMismatch
        );
        assert_eq!(
            compose_capture(&control, &capture(&control), false, true)
                .expect_err("missing cleanup")
                .code(),
            EnvelopeErrorCode::CaptureMismatch
        );
    }

    #[test]
    fn unknown_case_and_authority_fail_closed() {
        assert_eq!(
            synthetic_record("other").expect_err("closed cases").code(),
            EnvelopeErrorCode::InvalidControl
        );
        let mut changed = control("valid-no-match");
        changed.authority_added = true;
        assert_eq!(
            compose_capture(&changed, &capture(&changed), true, true)
                .expect_err("authority")
                .code(),
            EnvelopeErrorCode::InvalidControl
        );
    }
}
