// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Process-boundary tests for the IAR-1A synthetic supervisor."]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use context_analyzer_protocol::{
    AnalyzerExecutionManifest, AnalyzerRunnerRequest, ArtifactDescriptor, SyntheticOutcome,
    manifest_identity,
};
use context_analyzer_runner::{RunnerErrorCode, Supervisor};
use sha2::{Digest, Sha256};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "impresari-iar1-{label}-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create isolated staging root");
        set_private_directory(&path);
        Self(path.canonicalize().expect("canonical test root"))
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn digest(bytes: &[u8]) -> String {
    let value = Sha256::digest(bytes);
    let hex = value
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            use std::fmt::Write as _;
            write!(output, "{byte:02x}").expect("string write");
            output
        });
    format!("sha256:{hex}")
}

fn executable() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_impresari-context-analyzer-synthetic-worker"
    ))
    .canonicalize()
    .expect("canonical synthetic worker path")
}

fn manifest(executable: &Path) -> AnalyzerExecutionManifest {
    let executable_bytes = fs::read(executable).expect("worker bytes");
    let mut manifest = AnalyzerExecutionManifest {
        schema_name: "analyzer-execution-manifest".into(),
        schema_version: "1.0.0".into(),
        manifest_id: String::new(),
        analyzer_id: "impresari.synthetic".into(),
        analyzer_version: "1.0.0".into(),
        publisher: "BoldtHaus Studio, LLC".into(),
        executable_digest: digest(&executable_bytes),
        ruleset_digest: "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
            .into(),
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
    manifest.manifest_id = manifest_identity(&manifest).expect("manifest identity");
    manifest
}

fn request(
    manifest: &AnalyzerExecutionManifest,
    behavior: &str,
    request_id: &str,
) -> (AnalyzerRunnerRequest, BTreeMap<String, Vec<u8>>) {
    let bytes = b"synthetic artifact bytes".to_vec();
    let artifact_hash = digest(&bytes);
    let request = AnalyzerRunnerRequest {
        schema_name: "analyzer-runner-request".into(),
        schema_version: "1.0.0".into(),
        request_id: request_id.into(),
        occurred_at: "2026-08-29T12:00:00Z".into(),
        deadline_at: "2026-08-29T12:01:00Z".into(),
        workspace_snapshot:
            "sha256:1111111111111111111111111111111111111111111111111111111111111111".into(),
        assessment_plan_id:
            "sha256:2222222222222222222222222222222222222222222222222222222222222222".into(),
        policy_id: "sha256:3333333333333333333333333333333333333333333333333333333333333333".into(),
        manifest_id: manifest.manifest_id.clone(),
        capability_ids: vec!["synthetic.accounting".into()],
        artifacts: vec![ArtifactDescriptor {
            artifact_hash: artifact_hash.clone(),
            bytes: bytes.len().to_string(),
            media_type: "application.octet-stream".into(),
            target_platform: "any".into(),
        }],
        resource_profile_id: "iar-protocol-synthetic-v1".into(),
        resource_profile_digest:
            "sha256:f4e05f583e5af4719703e1178546d625bccb8efde1527143d55e32a9bfcb00b0".into(),
        synthetic_behavior: behavior.into(),
        source_paths_included: false,
        commands_included: false,
        network_destinations_included: false,
        credentials_included: false,
        authority_added: false,
    };
    (request, BTreeMap::from([(artifact_hash, bytes)]))
}

fn supervisor(root: &Path, timeout: Duration) -> Supervisor {
    Supervisor {
        executable: executable(),
        staging_root: root.to_path_buf(),
        excluded_roots: vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root"),
        ],
        timeout,
    }
}

#[cfg(unix)]
fn set_private_directory(path: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("private test root");
}

#[cfg(not(unix))]
fn set_private_directory(_path: &Path) {}

fn assert_root_empty(root: &Path) {
    assert!(
        fs::read_dir(root)
            .expect("read staging root")
            .next()
            .is_none(),
        "job bytes must be removed"
    );
}

#[test]
fn no_op_runs_in_a_private_job_and_reports_only_application_enforcement() {
    let root = TestRoot::new("no-op");
    let executable = executable();
    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_noop");
    let result = supervisor(&root.0, Duration::from_secs(2))
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect("supervised no-op");
    assert!(matches!(result.outcome, SyntheticOutcome::Result(_)));
    assert!(result.confinement.application_enforced);
    assert!(!result.confinement.os_confined);
    assert!(!result.confinement.vm_confined);
    assert!(!result.confinement.network_denial_verified);
    assert!(result.audit.job_removed);
    assert!(!result.audit.source_retained);
    assert!(!result.audit.authority_added);
    assert_root_empty(&root.0);
}

#[test]
fn every_process_fault_fails_closed_and_cleans_the_exact_job() {
    for (index, (behavior, expected)) in [
        ("crash", RunnerErrorCode::WorkerFailure),
        ("timeout", RunnerErrorCode::Timeout),
        ("input_mutation", RunnerErrorCode::ArtifactMismatch),
        ("output_flood", RunnerErrorCode::OutputLimit),
        ("malformed_output", RunnerErrorCode::InvalidOutput),
    ]
    .into_iter()
    .enumerate()
    {
        let root = TestRoot::new(behavior);
        let executable = executable();
        let manifest = manifest(&executable);
        let (request, artifacts) = request(&manifest, behavior, &format!("req_iar1_fault_{index}"));
        let timeout = if behavior == "timeout" {
            Duration::from_millis(75)
        } else {
            Duration::from_secs(2)
        };
        let error = supervisor(&root.0, timeout)
            .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
            .expect_err("fault must fail closed");
        assert_eq!(error.code(), expected, "unexpected {behavior} failure");
        assert_root_empty(&root.0);
    }
}

#[test]
fn executable_pin_and_fresh_job_identity_are_mandatory() {
    let root = TestRoot::new("pin");
    let executable = executable();
    let mut altered_manifest = manifest(&executable);
    let (mut altered_request, artifacts) = request(&altered_manifest, "no_op", "req_iar1_pin");
    altered_manifest.executable_digest =
        "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".into();
    altered_manifest.manifest_id = manifest_identity(&altered_manifest).expect("altered identity");
    altered_request.manifest_id = altered_manifest.manifest_id.clone();
    let error = supervisor(&root.0, Duration::from_secs(1))
        .execute(
            &altered_manifest,
            &altered_request,
            &artifacts,
            "2026-08-29T12:00:01Z",
        )
        .expect_err("wrong worker pin");
    assert_eq!(error.code(), RunnerErrorCode::WorkerIdentity);
    assert_root_empty(&root.0);

    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_collision");
    fs::create_dir(root.0.join("job-req_iar1_collision")).expect("collision directory");
    let error = supervisor(&root.0, Duration::from_secs(1))
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect_err("preexisting job must fail");
    assert_eq!(error.code(), RunnerErrorCode::Staging);
}

#[test]
fn invalid_completion_time_fails_before_staging_or_process_launch() {
    let root = TestRoot::new("completion-time");
    let executable = executable();
    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_invalid_completion_time");
    let error = supervisor(&root.0, Duration::from_secs(1))
        .execute(&manifest, &request, &artifacts, "not-a-timestamp")
        .expect_err("invalid completion time must fail during preflight");
    assert_eq!(error.code(), RunnerErrorCode::InvalidConfiguration);
    assert_root_empty(&root.0);
}

#[cfg(unix)]
#[test]
fn symlinked_worker_executable_is_rejected_before_staging() {
    use std::os::unix::fs::symlink;

    let root = TestRoot::new("worker-symlink");
    let link_parent = TestRoot::new("worker-symlink-parent");
    let worker = executable();
    let worker_link = link_parent.0.join("synthetic-worker-link");
    symlink(&worker, &worker_link).expect("create worker symlink");
    let manifest = manifest(&worker);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_worker_symlink");
    let supervisor = Supervisor {
        executable: worker_link,
        staging_root: root.0.clone(),
        excluded_roots: vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .expect("repository root"),
        ],
        timeout: Duration::from_secs(1),
    };
    let error = supervisor
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect_err("symlinked worker must fail identity validation");
    assert_eq!(error.code(), RunnerErrorCode::WorkerIdentity);
    assert_root_empty(&root.0);
}

#[cfg(unix)]
#[test]
fn symlinked_staging_root_is_rejected() {
    use std::os::unix::fs::symlink;

    let target = TestRoot::new("symlink-target");
    let link_parent = TestRoot::new("symlink-parent");
    let link = link_parent.0.join("staging-link");
    symlink(&target.0, &link).expect("create symlink");
    let executable = executable();
    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_symlink");
    let error = supervisor(&link, Duration::from_secs(1))
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect_err("symlink root must fail");
    assert_eq!(error.code(), RunnerErrorCode::Staging);
}

#[test]
fn staging_must_be_disjoint_from_declared_source_and_cache_roots() {
    let root = TestRoot::new("overlap");
    let executable = executable();
    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_overlap");
    let supervisor = Supervisor {
        executable,
        staging_root: root.0.clone(),
        excluded_roots: vec![root.0.clone()],
        timeout: Duration::from_secs(1),
    };
    let error = supervisor
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect_err("overlapping staging root must fail");
    assert_eq!(error.code(), RunnerErrorCode::InvalidConfiguration);
    assert_root_empty(&root.0);
}

#[cfg(unix)]
#[test]
fn staging_root_must_not_be_group_or_world_accessible() {
    use std::os::unix::fs::PermissionsExt as _;

    let root = TestRoot::new("permissions");
    fs::set_permissions(&root.0, fs::Permissions::from_mode(0o755)).expect("open permissions");
    let executable = executable();
    let manifest = manifest(&executable);
    let (request, artifacts) = request(&manifest, "no_op", "req_iar1_permissions");
    let error = supervisor(&root.0, Duration::from_secs(1))
        .execute(&manifest, &request, &artifacts, "2026-08-29T12:00:01Z")
        .expect_err("non-private root must fail");
    assert_eq!(error.code(), RunnerErrorCode::Staging);
    assert_root_empty(&root.0);
}
