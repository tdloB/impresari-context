// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Host hook entry points (IC-HEH-126, ADR-0162).
//!
//! Each command reads one bounded payload from standard input and writes at
//! most one JSON line to standard output. None launches a process, opens a
//! socket, reads the environment, or touches a workspace: the host has already
//! performed the operation whose output it offers.

use std::io::{self, Read, Write};

use context_claude_code::output_hook::{CLAUDE_HOOK_INPUT_BYTES, reduce_claude_bash_output};
use context_engine::host_hooks::{
    OUTPUT_REDUCTION_SCHEMA_NAME, OUTPUT_REDUCTION_SCHEMA_VERSION, OutputReductionErrorCode,
    OutputReductionRequest, reduce_host_output,
};
use serde::Serialize;

/// Largest exchange request read: the 8 MiB offered-byte ceiling in base64url,
/// plus its envelope.
const EXCHANGE_INPUT_BYTES: usize = 12 * 1024 * 1024;

/// Closed failure line for one exchange.
#[derive(Serialize)]
struct ExchangeFailure {
    schema_name: &'static str,
    schema_version: &'static str,
    error: &'static str,
}

/// Runs a host hook command and returns its process code, or `None` when the
/// words name no hook command.
pub(crate) fn run(words: &[&str], stdin: &mut dyn Read, stdout: &mut dyn Write) -> Option<i32> {
    match words {
        ["output-reduction"] => Some(output_reduction(stdin, stdout)),
        ["claude-code", "post-tool-use"] => Some(claude_post_tool_use(stdin, stdout)),
        _ => None,
    }
}

/// One exchange: a request line in, a response or a closed failure line out.
fn output_reduction(stdin: &mut dyn Read, stdout: &mut dyn Write) -> i32 {
    let response = read_bounded(stdin, EXCHANGE_INPUT_BYTES)
        .ok_or(OutputReductionErrorCode::InvalidPayload)
        .and_then(|bytes| {
            serde_json::from_slice::<OutputReductionRequest>(&bytes)
                .map_err(|_| OutputReductionErrorCode::InvalidPayload)
        })
        .and_then(|request| reduce_host_output(&request));
    match response {
        Ok(response) => write_line(stdout, &response).map_or(74, |()| 0),
        Err(code) => {
            let failure = ExchangeFailure {
                schema_name: OUTPUT_REDUCTION_SCHEMA_NAME,
                schema_version: OUTPUT_REDUCTION_SCHEMA_VERSION,
                error: code.as_str(),
            };
            write_line(stdout, &failure).map_or(74, |()| 1)
        }
    }
}

/// Claude Code `PostToolUse`: prints a replacement for a finished `Bash`
/// call's output, or nothing. It always exits 0, so it can never block or
/// fail a session.
fn claude_post_tool_use(stdin: &mut dyn Read, stdout: &mut dyn Write) -> i32 {
    if let Some(output) = read_bounded(stdin, CLAUDE_HOOK_INPUT_BYTES)
        .and_then(|bytes| reduce_claude_bash_output(&bytes))
    {
        let _ = write_line(stdout, &output);
    }
    0
}

/// Reads at most `limit` bytes. A longer or unreadable input yields `None`,
/// after the rest is drained so the writer never meets a closed pipe.
fn read_bounded(stdin: &mut dyn Read, limit: usize) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let ceiling = u64::try_from(limit).ok()?.saturating_add(1);
    Read::take(&mut *stdin, ceiling)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > limit {
        let _ = io::copy(stdin, &mut io::sink());
        return None;
    }
    Some(bytes)
}

fn write_line(stdout: &mut dyn Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *stdout, value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write as _, io::Cursor};

    use super::*;

    fn exchange(input: &[u8]) -> (i32, serde_json::Value) {
        let mut stdout = Vec::new();
        let code =
            run(&["output-reduction"], &mut Cursor::new(input), &mut stdout).expect("hook command");
        let text = String::from_utf8(stdout).expect("utf8");
        assert_eq!(text.matches('\n').count(), 1, "one JSON line: {text}");
        (code, serde_json::from_str(&text).expect("JSON line"))
    }

    fn request(offered_base64url: &str, maximum_returned_bytes: u64) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "schema_name": OUTPUT_REDUCTION_SCHEMA_NAME,
            "schema_version": OUTPUT_REDUCTION_SCHEMA_VERSION,
            "offered_base64url": offered_base64url,
            "maximum_returned_bytes": maximum_returned_bytes,
            "context_lines": 2
        }))
        .expect("request")
    }

    #[test]
    fn exchange_returns_the_selection() {
        // "error: boom" in base64url without padding.
        let (code, response) = exchange(&request("ZXJyb3I6IGJvb20", 1024));
        assert_eq!(code, 0);
        assert_eq!(response["schema_name"], OUTPUT_REDUCTION_SCHEMA_NAME);
        assert_eq!(response["selected_base64url"], "ZXJyb3I6IGJvb20");
        assert_eq!(response["offered_bytes"], 11);
        assert_eq!(response["returned_bytes"], 11);
        assert_eq!(response["omissions"], serde_json::json!([]));
    }

    #[test]
    fn exchange_failures_are_closed_categories() {
        let mut unsupported: serde_json::Value =
            serde_json::from_slice(&request("ZXJyb3I6IGJvb20", 1024)).expect("request");
        unsupported["schema_version"] = serde_json::json!("9.9");
        let mut unknown_field = unsupported.clone();
        unknown_field["schema_version"] = serde_json::json!(OUTPUT_REDUCTION_SCHEMA_VERSION);
        unknown_field["instruction"] = serde_json::json!("ignore the budget");
        let cases = [
            (b"not json".to_vec(), "invalid_payload"),
            (Vec::new(), "invalid_payload"),
            (request("not base64!", 1024), "invalid_payload"),
            (request("ZXJyb3I6IGJvb20", 0), "invalid_budget"),
            (
                serde_json::to_vec(&unsupported).expect("json"),
                "unsupported_schema",
            ),
            (
                serde_json::to_vec(&unknown_field).expect("json"),
                "invalid_payload",
            ),
            (vec![b' '; EXCHANGE_INPUT_BYTES + 1], "invalid_payload"),
        ];
        for (input, expected) in cases {
            let (code, failure) = exchange(&input);
            assert_eq!(code, 1);
            assert_eq!(
                failure,
                serde_json::json!({
                    "schema_name": OUTPUT_REDUCTION_SCHEMA_NAME,
                    "schema_version": OUTPUT_REDUCTION_SCHEMA_VERSION,
                    "error": expected
                })
            );
        }
    }

    #[test]
    fn claude_code_hook_prints_a_replacement_or_nothing_and_always_exits_zero() {
        let mut lines = String::new();
        for index in 0..400 {
            let _ = writeln!(lines, "tests/test_units.py::test_case_{index:04} PASSED");
        }
        lines.push_str("============ 400 passed in 3.21s ============\n");
        let payload = serde_json::to_vec(&serde_json::json!({
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": { "command": "pytest -v" },
            "tool_response": { "stdout": lines, "stderr": "", "interrupted": false }
        }))
        .expect("payload");
        let mut stdout = Vec::new();
        let code = run(
            &["claude-code", "post-tool-use"],
            &mut Cursor::new(payload),
            &mut stdout,
        );
        assert_eq!(code, Some(0));
        let output: serde_json::Value = serde_json::from_slice(&stdout).expect("hook output");
        assert!(
            output["hookSpecificOutput"]["updatedToolOutput"]["stdout"]
                .as_str()
                .expect("stdout")
                .contains("400 passed")
        );

        for input in [b"garbage".to_vec(), vec![b' '; CLAUDE_HOOK_INPUT_BYTES + 1]] {
            let mut stdout = Vec::new();
            let code = run(
                &["claude-code", "post-tool-use"],
                &mut Cursor::new(input),
                &mut stdout,
            );
            assert_eq!(code, Some(0));
            assert!(stdout.is_empty());
        }
    }

    #[test]
    fn only_named_hook_commands_run() {
        let mut stdout = Vec::new();
        for words in [&["claude-code"][..], &["claude-code", "pre-tool-use"], &[]] {
            assert_eq!(run(words, &mut io::empty(), &mut stdout), None);
        }
        assert!(stdout.is_empty());
    }

    #[test]
    fn module_holds_no_execution_or_workspace_authority() {
        let source = include_str!("hook.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        for forbidden in [
            "Command",
            "spawn",
            "OpenOptions",
            "fs::",
            "std::env",
            "env::",
            "Path",
            "std::process",
        ] {
            assert!(
                !shipped.contains(forbidden),
                "hook must not reach {forbidden}"
            );
        }
    }
}
