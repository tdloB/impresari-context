// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Process-boundary tests for the structural worker launcher."]

use std::{
    fmt::Write as _,
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_structural::{
    FactClass, GRAPH_VERSION, PROTOCOL_VERSION, RESOLVER_VERSION, StructuralError,
    StructuralLanguage, WorkerLauncher, WorkerPath, WorkerRequest,
};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .fold(String::from("sha256:"), |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a string cannot fail");
            output
        })
}

fn temporary_empty_directory() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("impresari-context-worker-test-{nonce}"));
    fs::create_dir(&path).expect("create empty directory");
    path
}

fn request(source: &[u8]) -> WorkerRequest {
    WorkerRequest {
        schema_name: "structural-worker-request".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: "req_worker_process_01".into(),
        language: StructuralLanguage::TypeScript,
        path: WorkerPath {
            display_path: "src/example.ts".into(),
            platform_family: "unix".into(),
            unit_encoding: "unix_bytes".into(),
            relative_units_base64url: "c3JjL2V4YW1wbGUudHM".into(),
        },
        content_hash: sha256(source),
        source_base64url: URL_SAFE_NO_PAD.encode(source),
        fact_classes: vec![
            FactClass::Declaration,
            FactClass::Contains,
            FactClass::Import,
            FactClass::Export,
        ],
        max_facts: 100,
        max_nesting_depth: 128,
        max_response_bytes: 1_048_576,
        parser_version: "tree-sitter-0.26.12".into(),
        grammar_version: "tree-sitter-typescript-0.23.2".into(),
        resolver_version: RESOLVER_VERSION.into(),
        graph_version: GRAPH_VERSION.into(),
    }
}

#[test]
fn fresh_worker_runs_with_pinned_identity_and_empty_environment() {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_impresari-context-structural-worker"));
    let expected_sha256 = sha256(&fs::read(&executable).expect("read worker"));
    let empty = temporary_empty_directory();
    let launcher = WorkerLauncher {
        executable,
        expected_sha256,
        empty_working_directory: empty.clone(),
        timeout: Duration::from_secs(5),
    };
    let output = launcher
        .execute(&request(b"export function value() { return 1; }"))
        .expect("worker success");
    assert!(
        output
            .facts
            .iter()
            .any(|fact| fact.name.as_deref() == Some("value"))
    );
    fs::remove_dir(empty).expect("remove empty directory");
}

#[test]
fn executable_identity_mismatch_is_rejected_before_launch() {
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_impresari-context-structural-worker"));
    let empty = temporary_empty_directory();
    let launcher = WorkerLauncher {
        executable,
        expected_sha256: sha256(b"not the worker"),
        empty_working_directory: empty.clone(),
        timeout: Duration::from_secs(5),
    };
    assert_eq!(
        launcher.execute(&request(b"const value = 1;")),
        Err(StructuralError::WorkerIdentity)
    );
    fs::remove_dir(empty).expect("remove empty directory");
}
