// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Claude Code `PostToolUse` output reduction (IC-HEH-126, ADR-0162).
//!
//! Maps Claude Code's hook payload for a finished `Bash` call onto the host
//! output-reduction rule. Claude Code ran the command; this module only selects
//! lines from the output it was handed and removes their terminal escape
//! sequences (ADR-0164). It launches nothing, opens nothing, and reads no
//! environment.
//!
//! Claude Code checks a replacement against the tool's own output shape and
//! keeps the original output when it does not match. The replacement is
//! therefore the payload's own `tool_response`, with only `stdout` and
//! `stderr` rewritten.

use std::fmt::Write as _;

use context_engine::host_hooks::{reduce_host_text, remove_terminal_escapes};
use serde_json::{Value, json};

/// Bytes of command output kept for the model, across `stdout` and `stderr`.
/// ADR-0160 measured 8 KB as the budget at which every scored fact survived
/// on its 33-log corpus.
pub const CLAUDE_OUTPUT_RETURNED_BYTES: usize = 8 * 1024;
/// Lines kept around each retained line, as ADR-0160 measured.
pub const CLAUDE_OUTPUT_CONTEXT_LINES: u32 = 2;
/// Largest hook payload the adapter reads.
pub const CLAUDE_HOOK_INPUT_BYTES: usize = 16 * 1024 * 1024;

/// Omission recorded when a line worth keeping did not fit the budget.
const LIMIT_REACHED: &str = "output_returned_byte_limit_reached";

/// Hook output that shortens one finished `Bash` call's output, or `None` to
/// leave it as it is.
///
/// `None` covers every case the adapter does not own: another event or tool;
/// an interrupted, backgrounded, or image result; output already within the
/// budget; a selection no smaller than the original; and any payload it cannot
/// read. The hook then prints nothing and Claude Code keeps the original.
#[must_use]
pub fn reduce_claude_bash_output(input: &[u8]) -> Option<Value> {
    if input.len() > CLAUDE_HOOK_INPUT_BYTES {
        return None;
    }
    let payload: Value = serde_json::from_slice(input).ok()?;
    if payload.get("hook_event_name")?.as_str()? != "PostToolUse"
        || payload.get("tool_name")?.as_str()? != "Bash"
    {
        return None;
    }
    let response = payload.get("tool_response")?.as_object()?;
    if response.get("interrupted")?.as_bool()?
        || response.get("isImage").and_then(Value::as_bool) == Some(true)
        || response.contains_key("backgroundTaskId")
    {
        return None;
    }
    let stdout = response.get("stdout")?.as_str()?;
    let stderr = response.get("stderr")?.as_str()?;
    let offered = stdout.len().checked_add(stderr.len())?;
    if offered <= CLAUDE_OUTPUT_RETURNED_BYTES {
        return None;
    }

    let (stdout_share, stderr_share) = shares(stdout.len(), stderr.len());
    let stdout_kept = kept(stdout, stdout_share)?;
    let stderr_kept = kept(stderr, stderr_share)?;
    let returned = stdout_kept.text.len() + stderr_kept.text.len();
    if returned >= offered {
        return None;
    }

    let mut note = format!(
        "Impresari Context shortened this command's output to {returned} of {offered} bytes \
         ({} of {} lines, in their original order, none reworded).",
        stdout_kept.returned_lines + stderr_kept.returned_lines,
        stdout_kept.offered_lines + stderr_kept.offered_lines,
    );
    let removed = stdout_kept.escape_bytes_removed + stderr_kept.escape_bytes_removed;
    if removed > 0 {
        let _ = write!(
            note,
            " It removed {removed} bytes of terminal color and cursor codes."
        );
    }
    if stdout_kept.limit_reached || stderr_kept.limit_reached {
        note.push_str(" Not every line reporting a result, failure, warning, or location fit.");
    }
    note.push_str(" Rerun the command more narrowly to see what was left out.");

    let mut updated = response.clone();
    updated.insert("stdout".to_owned(), Value::String(stdout_kept.text));
    updated.insert("stderr".to_owned(), Value::String(stderr_kept.text));
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "updatedToolOutput": Value::Object(updated),
            "additionalContext": note,
        }
    }))
}

/// One stream after selection.
struct Kept {
    text: String,
    offered_lines: u64,
    returned_lines: u64,
    escape_bytes_removed: usize,
    limit_reached: bool,
}

/// Keeps a stream whole when it fits its share, and selects from it otherwise.
/// Either way, its terminal escape sequences are removed (ADR-0164).
fn kept(text: &str, share: usize) -> Option<Kept> {
    if text.len() <= share {
        let lines = u64::try_from(text.lines().count()).ok()?;
        let cleaned = remove_terminal_escapes(text);
        return Some(Kept {
            escape_bytes_removed: text.len().saturating_sub(cleaned.len()),
            text: cleaned.into_owned(),
            offered_lines: lines,
            returned_lines: lines,
            limit_reached: false,
        });
    }
    let reduced = reduce_host_text(
        text,
        u64::try_from(share).ok()?,
        CLAUDE_OUTPUT_CONTEXT_LINES,
    )
    .ok()?;
    Some(Kept {
        limit_reached: reduced
            .omissions
            .iter()
            .any(|omission| omission == LIMIT_REACHED),
        offered_lines: reduced.offered_lines,
        returned_lines: reduced.returned_lines,
        escape_bytes_removed: usize::try_from(reduced.escape_bytes_removed).ok()?,
        text: reduced.text,
    })
}

/// Splits the budget between the two streams. A stream that fits in half of
/// it is kept whole, and the other stream gets the rest.
const fn shares(stdout: usize, stderr: usize) -> (usize, usize) {
    let budget = CLAUDE_OUTPUT_RETURNED_BYTES;
    let half = budget / 2;
    if stdout <= half {
        (stdout, budget - stdout)
    } else if stderr <= half {
        (budget - stderr, stderr)
    } else {
        (half, budget - half)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(stdout: &str, stderr: &str) -> Value {
        json!({
            "session_id": "session",
            "transcript_path": "/tmp/transcript.jsonl",
            "cwd": "/tmp/project",
            "permission_mode": "default",
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": { "command": "pytest -v", "description": "Run the tests" },
            "tool_response": {
                "stdout": stdout,
                "stderr": stderr,
                "interrupted": false,
                "isImage": false,
                "noOutputExpected": false
            },
            "tool_use_id": "toolu_01"
        })
    }

    fn run(value: &Value) -> Option<Value> {
        reduce_claude_bash_output(&serde_json::to_vec(value).expect("payload"))
    }

    fn passing_run(tests: usize) -> String {
        let mut log = String::from(
            "============================= test session starts ==============================\n\
             platform linux -- Python 3.11.4, pytest-7.4.0, pluggy-1.2.0\n\
             rootdir: /testbed\n\n",
        );
        for index in 0..tests {
            let _ = writeln!(
                log,
                "tests/test_units.py::test_case_{index:04} PASSED{:>30}[{:3}%]",
                "",
                index * 100 / tests
            );
        }
        let _ = writeln!(
            log,
            "============================== {tests} passed in 3.21s =============================="
        );
        log
    }

    fn deprecations(count: usize) -> String {
        let mut log = String::new();
        for index in 0..count {
            let _ = writeln!(
                log,
                "tests/test_units.py:{index}: DeprecationWarning: `old_call` is deprecated"
            );
        }
        log
    }

    fn replacement(output: &Value) -> (&str, &str) {
        let updated = &output["hookSpecificOutput"]["updatedToolOutput"];
        (
            updated["stdout"].as_str().expect("stdout"),
            updated["stderr"].as_str().expect("stderr"),
        )
    }

    /// Whether `kept` is offered lines, in order, each without its terminal
    /// escape sequences.
    fn is_subsequence(kept: &str, offered: &str) -> bool {
        let mut offered = offered.lines().map(remove_terminal_escapes);
        kept.lines()
            .all(|line| offered.any(|candidate| candidate == line))
    }

    /// `log` as a terminal would color it: passing tests and counts in green.
    fn colored(log: &str) -> String {
        log.replace("PASSED", "\u{1b}[32mPASSED\u{1b}[0m")
            .replace(" passed in ", "\u{1b}[32m passed\u{1b}[0m in ")
    }

    #[test]
    fn removes_terminal_color_codes_and_says_how_many_bytes() {
        let log = colored(&passing_run(400));
        let warning = "\u{1b}[33mwarning: unused manifest key: package.readme-file\u{1b}[0m\n";
        let output = run(&payload(&log, warning)).expect("shortened");
        let (stdout, stderr) = replacement(&output);
        assert!(!stdout.contains('\u{1b}'));
        assert!(stdout.contains("400 passed in 3.21s"));
        assert!(stdout.len() <= CLAUDE_OUTPUT_RETURNED_BYTES);
        assert!(is_subsequence(stdout, &log));
        // A stream kept whole loses its codes too.
        assert_eq!(
            stderr,
            "warning: unused manifest key: package.readme-file\n"
        );
        let note = output["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .expect("note");
        assert!(note.contains(" bytes of terminal color and cursor codes."));
    }

    #[test]
    fn leaves_output_within_the_budget_alone() {
        assert_eq!(run(&payload("3 passed in 0.10s\n", "")), None);
        let exact = "x".repeat(CLAUDE_OUTPUT_RETURNED_BYTES);
        assert_eq!(run(&payload(&exact, "")), None);
    }

    #[test]
    fn shortens_a_long_passing_run_and_keeps_its_result() {
        let log = passing_run(400);
        assert!(log.len() > 3 * CLAUDE_OUTPUT_RETURNED_BYTES);
        let output = run(&payload(&log, "")).expect("shortened");
        let hook = &output["hookSpecificOutput"];
        assert_eq!(hook["hookEventName"], "PostToolUse");
        let (stdout, stderr) = replacement(&output);
        assert!(stdout.contains("400 passed in 3.21s"));
        assert!(stdout.len() <= CLAUDE_OUTPUT_RETURNED_BYTES);
        assert_eq!(stderr, "");
        assert!(is_subsequence(stdout, &log));
        let updated = &hook["updatedToolOutput"];
        assert_eq!(updated["interrupted"], false);
        assert_eq!(updated["isImage"], false);
        assert_eq!(updated["noOutputExpected"], false);
        let note = hook["additionalContext"].as_str().expect("note");
        assert!(note.starts_with("Impresari Context shortened"));
        assert!(note.contains(&format!("of {} bytes", log.len())));
        assert!(!note.contains("test_case_"));
        assert!(!note.contains("terminal"));
    }

    #[test]
    fn splits_the_budget_between_two_long_streams() {
        let log = passing_run(400);
        let warnings = deprecations(300);
        let output = run(&payload(&log, &warnings)).expect("shortened");
        let (stdout, stderr) = replacement(&output);
        assert!(stdout.len() <= CLAUDE_OUTPUT_RETURNED_BYTES / 2);
        assert!(stderr.len() <= CLAUDE_OUTPUT_RETURNED_BYTES / 2);
        assert!(stdout.contains("400 passed in 3.21s"));
        assert!(!stderr.is_empty());
        assert!(is_subsequence(stdout, &log));
        assert!(is_subsequence(stderr, &warnings));
    }

    #[test]
    fn keeps_a_short_stream_whole() {
        let log = passing_run(400);
        let warning = "warning: unused manifest key: package.readme-file\n";
        let output = run(&payload(&log, warning)).expect("shortened");
        let (stdout, stderr) = replacement(&output);
        assert_eq!(stderr, warning);
        assert!(stdout.len() + stderr.len() <= CLAUDE_OUTPUT_RETURNED_BYTES);
    }

    #[test]
    fn ignores_what_it_does_not_own() {
        let log = passing_run(400);
        let mut failure = payload(&log, "");
        failure["hook_event_name"] = json!("PostToolUseFailure");
        let mut read = payload(&log, "");
        read["tool_name"] = json!("Read");
        let mut interrupted = payload(&log, "");
        interrupted["tool_response"]["interrupted"] = json!(true);
        let mut unknown_interruption = payload(&log, "");
        unknown_interruption["tool_response"]
            .as_object_mut()
            .expect("response")
            .remove("interrupted");
        let mut image = payload(&log, "");
        image["tool_response"]["isImage"] = json!(true);
        let mut background = payload(&log, "");
        background["tool_response"]["backgroundTaskId"] = json!("task_1");
        let mut not_text = payload(&log, "");
        not_text["tool_response"]["stdout"] = json!(["not", "text"]);
        for value in [
            failure,
            read,
            interrupted,
            unknown_interruption,
            image,
            background,
            not_text,
        ] {
            assert_eq!(run(&value), None);
        }
        assert_eq!(reduce_claude_bash_output(b"{not json"), None);
        assert_eq!(
            reduce_claude_bash_output(&vec![b' '; CLAUDE_HOOK_INPUT_BYTES + 1]),
            None
        );
    }

    #[test]
    fn module_holds_no_execution_or_workspace_authority() {
        let source = include_str!("output_hook.rs");
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
