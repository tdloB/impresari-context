// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Packaged CLI smoke tests."]

use std::{
    fs,
    io::Write,
    process::{Command, Output, Stdio},
};

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

fn run_hook(words: &[&str], input: &[u8], credentials: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_impresari-context"));
    command
        .arg("hook")
        .args(words)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if credentials {
        command
            .env("ANTHROPIC_API_KEY", "sk-ant-hook-test-credential")
            .env("OPENAI_API_KEY", "sk-hook-test-credential")
            .env("GITHUB_TOKEN", "ghp_hook_test_credential");
    }
    let mut child = command.spawn().expect("run binary");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input)
        .expect("hook input");
    child.wait_with_output().expect("hook output")
}

#[test]
fn hook_commands_ignore_provider_credentials_in_their_environment() {
    // "error: boom" in base64url without padding.
    let request = br#"{"schema_name":"impresari_context_output_reduction","schema_version":"1.0","offered_base64url":"ZXJyb3I6IGJvb20","maximum_returned_bytes":1024,"context_lines":2}"#;
    let clean = run_hook(&["output-reduction"], request, false);
    let exposed = run_hook(&["output-reduction"], request, true);
    assert!(clean.status.success());
    assert!(clean.stderr.is_empty());
    assert_eq!(clean.status.code(), exposed.status.code());
    assert_eq!(clean.stdout, exposed.stdout);
    assert_eq!(clean.stderr, exposed.stderr);
    let text = String::from_utf8(exposed.stdout).expect("utf8");
    assert!(!text.contains("credential"));
    let value: serde_json::Value = serde_json::from_str(&text).expect("JSON line");
    assert_eq!(value["selected_base64url"], "ZXJyb3I6IGJvb20");

    let other_tool = br#"{"hook_event_name":"PostToolUse","tool_name":"Read","tool_response":{}}"#;
    for credentials in [false, true] {
        let output = run_hook(&["claude-code", "post-tool-use"], other_tool, credentials);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}
