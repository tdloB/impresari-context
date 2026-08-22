// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Packaged CLI smoke tests."]

use std::{fs, process::Command};

#[test]
fn packaged_binary_emits_only_machine_json_on_stdout() {
    let root = std::env::temp_dir().join(format!("impresari-cli-binary-{}", std::process::id()));
    let source = root.join("source");
    let cache = root.join("cache");
    fs::create_dir_all(&source).expect("source root");
    fs::write(source.join("a.rs"), b"fn main() {}\n").expect("source");
    let output = Command::new(env!("CARGO_BIN_EXE_impresari-context"))
        .args([
            "--at",
            "2026-08-21T12:00:00Z",
            "--cutoff",
            "2026-08-14T12:00:00Z",
            "--id-seed",
            "binarytest",
            "workspace",
            "open",
        ])
        .arg(&source)
        .arg(&cache)
        .output()
        .expect("run binary");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    assert_eq!(value["schema_name"], "workspace-handle");
    assert_eq!(fs::read_dir(&source).expect("source entries").count(), 1);
    let _ = fs::remove_dir_all(root);
}
