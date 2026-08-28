//! Offline executable-level coverage for production adapter boundaries.

#![forbid(unsafe_code)]

use context_engine::{ContextPlanStep, QueryKind};
use context_evaluation::agent_eval::{AdapterRequest, Arm, PacketResponse, PricingSchedule};
use context_evaluation::production_adapter::source_fingerprint;
use serde_json::Value;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "impresari-production-adapters-{}-{nonce}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create test directory");
        Self(path)
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request(root: &Path, model: &str, arm: Arm) -> AdapterRequest {
    let source_files = vec!["one.rs".to_owned()];
    AdapterRequest {
        task_id: "one".into(),
        prompt: "Explain ONE and cite its definition.".into(),
        arm,
        workspace_root: root.display().to_string(),
        source_fingerprint_sha256: source_fingerprint(root, &source_files)
            .expect("source fingerprint"),
        source_files,
        context_plan: vec![ContextPlanStep {
            kind: QueryKind::ExactPath,
            query: "one.rs".into(),
        }],
        model_identifier: model.into(),
        pricing_schedule: PricingSchedule::default(),
        container_image: "offline-test".into(),
        operation_timestamp: "1970-01-01T00:00:00Z".into(),
        turn_limit: 3,
        packet: (arm == Arm::Treatment).then(|| "test packet".into()),
    }
}

fn invoke(path: &str, request: &AdapterRequest) -> std::process::Output {
    let mut child = Command::new(path)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adapter");
    child
        .stdin
        .as_mut()
        .expect("adapter stdin")
        .write_all(&serde_json::to_vec(request).expect("serialize request"))
        .expect("write request");
    drop(child.stdin.take());
    child.wait_with_output().expect("adapter output")
}

#[test]
fn real_packet_adapter_builds_from_only_frozen_source() {
    let directory = TestDirectory::new();
    fs::write(directory.0.join("one.rs"), "pub const ONE: u8 = 1;\n").expect("write source");
    fs::write(
        directory.0.join("not-allowed.txt"),
        "must not enter packet\n",
    )
    .expect("write unlisted source");
    let mut request = request(&directory.0, "offline", Arm::Treatment);
    request.packet = None;
    let output = invoke(
        env!("CARGO_BIN_EXE_impresari-context-production-packet-adapter"),
        &request,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let response: PacketResponse =
        serde_json::from_slice(&output.stdout).expect("parse packet response");
    assert_eq!(
        response.source_fingerprint_sha256,
        request.source_fingerprint_sha256
    );
    assert!(!response.packet.contains("must not enter packet"));
    let packet: Value = serde_json::from_str(&response.packet).expect("parse packet");
    assert_eq!(packet["schema_name"], "context-packet");
    assert!(
        !packet["observed_evidence"]
            .as_array()
            .expect("packet evidence")
            .is_empty()
    );
}

#[test]
fn provider_adapters_fail_before_network_when_credentials_are_absent() {
    let directory = TestDirectory::new();
    fs::write(directory.0.join("one.rs"), "pub const ONE: u8 = 1;\n").expect("write source");
    for (binary, model, expected) in [
        (
            env!("CARGO_BIN_EXE_impresari-context-openai-agent-adapter"),
            "gpt-5.6-sol",
            "OPENAI_API_KEY is required",
        ),
        (
            env!("CARGO_BIN_EXE_impresari-context-anthropic-agent-adapter"),
            "claude-opus-5",
            "ANTHROPIC_API_KEY is required",
        ),
    ] {
        let output = invoke(binary, &request(&directory.0, model, Arm::BaselineA));
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
        assert!(output.stdout.is_empty());
    }
}
