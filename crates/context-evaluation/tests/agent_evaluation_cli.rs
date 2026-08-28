//! End-to-end CLI coverage for the bounded agent-context study runner.

#![forbid(unsafe_code)]

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "impresari-agent-eval-cli-{}-{nonce}",
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

#[test]
fn deterministic_five_language_study_requires_consent_and_omits_payloads() {
    let directory = TestDirectory::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../evaluation/agent-context/v1");
    let source = directory.0.join("source");
    fs::create_dir_all(&source).expect("create source directory");
    for name in [
        "go.go",
        "python.py",
        "rust.rs",
        "strict.json",
        "typescript.ts",
    ] {
        fs::copy(fixture.join("source").join(name), source.join(name))
            .expect("copy fixture source");
    }

    let mut spec: Value =
        serde_json::from_slice(&fs::read(fixture.join("study.json")).expect("read fixture study"))
            .expect("parse fixture study");
    let adapter = env!("CARGO_BIN_EXE_impresari-context-deterministic-adapter");
    spec["agent_command"] = serde_json::json!([adapter, "agent"]);
    spec["packet_command"] = serde_json::json!([adapter, "packet"]);
    let spec_path = directory.0.join("study.json");
    fs::write(
        &spec_path,
        serde_json::to_vec_pretty(&spec).expect("encode study"),
    )
    .expect("write study");
    let output = directory.0.join("records");
    let cli = env!("CARGO_BIN_EXE_impresari-context-agent-eval");

    let denied = Command::new(cli)
        .args(["run"])
        .arg(&spec_path)
        .arg(&output)
        .output()
        .expect("run CLI without consent");
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("--allow-adapter-execution"));

    let completed = Command::new(cli)
        .args(["run"])
        .arg(&spec_path)
        .arg(&output)
        .arg("--allow-adapter-execution")
        .output()
        .expect("run deterministic study");
    assert!(
        completed.status.success(),
        "{}",
        String::from_utf8_lossy(&completed.stderr)
    );

    let validated = Command::new(cli)
        .arg("validate-runs")
        .arg(&spec_path)
        .arg(&output)
        .output()
        .expect("validate deterministic records");
    assert!(
        validated.status.success(),
        "{}",
        String::from_utf8_lossy(&validated.stderr)
    );

    let records = fs::read_dir(&output)
        .expect("read record directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("run-"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 15);
    let summary: Value =
        serde_json::from_slice(&fs::read(output.join("summary.json")).expect("read summary"))
            .expect("parse summary");
    assert_eq!(summary["arms"][0]["correctness_rate"], 0.0);
    assert_eq!(summary["arms"][1]["correctness_rate"], 1.0);
    assert_eq!(summary["arms"][2]["correctness_rate"], 0.0);
    let persisted = records
        .iter()
        .map(|entry| fs::read_to_string(entry.path()).expect("read record"))
        .collect::<String>();
    for forbidden in [
        "Return the TypeScript greeting.",
        "hello-typescript",
        "intentionally incorrect cold baseline",
        "deterministic Impresari Context fixture packet",
    ] {
        assert!(
            !persisted.contains(forbidden),
            "persisted sensitive payload: {forbidden}"
        );
    }
}

#[test]
fn specification_rejects_shell_command_strings() {
    let directory = TestDirectory::new();
    let source = directory.0.join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("one.rs"), "const ONE: u8 = 1;\n").expect("write source");
    let spec = serde_json::json!({
        "schema_version": "1.0",
        "study_id": "reject-shell",
        "repository": "source",
        "source_files": ["one.rs"],
        "workspace_revision": "fixture-v1",
        "execution": {
            "agent_adapter_identifier": "test-agent",
            "agent_adapter_version": "1",
            "packet_adapter_identifier": "test-packet",
            "packet_adapter_version": "1",
            "model_identifier": "test-model",
            "container_image": "none",
            "turn_limit": 1,
            "pricing_basis": "test"
        },
        "repetitions": 1,
        "agent_command": ["/bin/sh", "-c", "echo unsafe"],
        "packet_command": ["/bin/false"],
        "environment": {},
        "command_timeout_seconds": 1,
        "max_stdout_bytes": 1024,
        "max_stderr_bytes": 1024,
        "tasks": [{
            "id": "one",
            "prompt": "one",
            "expected_answer_fragments": ["one"],
            "required_evidence": [{"path":"one.rs","line_start":1,"line_end":1}]
        }]
    });
    let path = directory.0.join("study.json");
    fs::write(&path, serde_json::to_vec(&spec).expect("encode spec")).expect("write spec");
    let output = Command::new(env!("CARGO_BIN_EXE_impresari-context-agent-eval"))
        .arg("validate-spec")
        .arg(path)
        .output()
        .expect("validate unsafe spec");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("shell command string"));
}

#[test]
fn adapter_failures_timeouts_and_output_limits_fail_closed() {
    let directory = TestDirectory::new();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../evaluation/agent-context/v1");
    let source = directory.0.join("source");
    fs::create_dir_all(&source).expect("create source directory");
    for name in [
        "go.go",
        "python.py",
        "rust.rs",
        "strict.json",
        "typescript.ts",
    ] {
        fs::copy(fixture.join("source").join(name), source.join(name))
            .expect("copy fixture source");
    }
    let original: Value =
        serde_json::from_slice(&fs::read(fixture.join("study.json")).expect("read fixture study"))
            .expect("parse fixture study");
    let adapter = env!("CARGO_BIN_EXE_impresari-context-deterministic-adapter");
    let cli = env!("CARGO_BIN_EXE_impresari-context-agent-eval");

    for (mode, expected) in [
        ("malformed", "parse adapter response"),
        ("oversize", "stdout exceeded"),
        ("stderr-oversize", "stderr exceeded"),
        ("fail", "adapter exited"),
        ("sleep", "adapter exceeded"),
        ("mutate", "evaluated source changed"),
    ] {
        let mut spec = original.clone();
        spec["agent_command"] = serde_json::json!([adapter, mode]);
        spec["packet_command"] = serde_json::json!([adapter, "packet"]);
        spec["command_timeout_seconds"] = serde_json::json!(1);
        if mode == "oversize" {
            spec["max_stdout_bytes"] = serde_json::json!(1024);
        }
        if mode == "stderr-oversize" {
            spec["max_stderr_bytes"] = serde_json::json!(1024);
        }
        let spec_path = directory.0.join(format!("{mode}.json"));
        fs::write(
            &spec_path,
            serde_json::to_vec(&spec).expect("encode failure spec"),
        )
        .expect("write failure spec");
        let output = Command::new(cli)
            .arg("run")
            .arg(&spec_path)
            .arg(directory.0.join(format!("output-{mode}")))
            .arg("--allow-adapter-execution")
            .env("IMPRESARI_PARENT_SECRET", "must-not-reach-adapter")
            .output()
            .expect("run failing adapter");
        assert!(
            !output.status.success(),
            "mode {mode} unexpectedly succeeded"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "mode {mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn specification_rejects_repository_and_source_root_escape() {
    let directory = TestDirectory::new();
    let source = directory.0.join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("one.rs"), "const ONE: u8 = 1;\n").expect("write source");
    let base = serde_json::json!({
        "schema_version": "1.0",
        "study_id": "reject-escape",
        "repository": "source",
        "source_files": ["one.rs"],
        "workspace_revision": "fixture-v1",
        "execution": {
            "agent_adapter_identifier": "test-agent",
            "agent_adapter_version": "1",
            "packet_adapter_identifier": "test-packet",
            "packet_adapter_version": "1",
            "model_identifier": "test-model",
            "container_image": "none",
            "turn_limit": 1,
            "pricing_basis": "test"
        },
        "repetitions": 1,
        "agent_command": ["/bin/false"],
        "packet_command": ["/bin/false"],
        "environment": {},
        "command_timeout_seconds": 1,
        "max_stdout_bytes": 1024,
        "max_stderr_bytes": 1024,
        "tasks": [{
            "id": "one",
            "prompt": "one",
            "expected_answer_fragments": ["one"],
            "required_evidence": [{"path":"one.rs","line_start":1,"line_end":1}]
        }]
    });
    for (label, field, escaped) in [
        ("repository", "repository", "../outside"),
        ("source", "source_files", "../outside.rs"),
    ] {
        let mut spec = base.clone();
        if field == "repository" {
            spec[field] = serde_json::json!(escaped);
        } else {
            spec[field] = serde_json::json!([escaped]);
        }
        let path = directory.0.join(format!("escape-{label}.json"));
        fs::write(&path, serde_json::to_vec(&spec).expect("encode spec")).expect("write spec");
        let output = Command::new(env!("CARGO_BIN_EXE_impresari-context-agent-eval"))
            .arg("validate-spec")
            .arg(path)
            .output()
            .expect("validate escaped spec");
        assert!(
            !output.status.success(),
            "{label} escape unexpectedly accepted"
        );
    }
}

#[cfg(unix)]
#[test]
fn specification_rejects_symlinked_source_files() {
    use std::os::unix::fs::symlink;

    let directory = TestDirectory::new();
    let source = directory.0.join("source");
    fs::create_dir_all(&source).expect("create source");
    fs::write(source.join("real.rs"), "const ONE: u8 = 1;\n").expect("write source");
    symlink("real.rs", source.join("linked.rs")).expect("create source symlink");
    let spec = serde_json::json!({
        "schema_version": "1.0",
        "study_id": "reject-symlink",
        "repository": "source",
        "source_files": ["linked.rs"],
        "workspace_revision": "fixture-v1",
        "execution": {
            "agent_adapter_identifier": "test-agent",
            "agent_adapter_version": "1",
            "packet_adapter_identifier": "test-packet",
            "packet_adapter_version": "1",
            "model_identifier": "test-model",
            "container_image": "none",
            "turn_limit": 1,
            "pricing_basis": "test"
        },
        "repetitions": 1,
        "agent_command": ["/bin/false"],
        "packet_command": ["/bin/false"],
        "environment": {},
        "command_timeout_seconds": 1,
        "max_stdout_bytes": 1024,
        "max_stderr_bytes": 1024,
        "tasks": [{
            "id": "one",
            "prompt": "one",
            "expected_answer_fragments": ["one"],
            "required_evidence": [{"path":"linked.rs","line_start":1,"line_end":1}]
        }]
    });
    let path = directory.0.join("study.json");
    fs::write(&path, serde_json::to_vec(&spec).expect("encode spec")).expect("write spec");
    let output = Command::new(env!("CARGO_BIN_EXE_impresari-context-agent-eval"))
        .arg("validate-spec")
        .arg(path)
        .output()
        .expect("validate symlink spec");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink is not allowed"));
}
