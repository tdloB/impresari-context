//! Offline deterministic adapter used only by the frozen evaluation fixture.

#![forbid(unsafe_code)]

use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::env;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("deterministic adapter: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    if env::var_os("IMPRESARI_PARENT_SECRET").is_some() {
        return Err("inherited environment was not cleared".to_owned());
    }
    let mode = env::args()
        .nth(1)
        .ok_or_else(|| "expected packet or agent mode".to_owned())?;
    if mode == "packet"
        && (env::var_os("OPENAI_API_KEY").is_some() || env::var_os("ANTHROPIC_API_KEY").is_some())
    {
        return Err("agent credential reached packet adapter".to_owned());
    }
    let mut input = Vec::new();
    std::io::stdin()
        .take(256 * 1024)
        .read_to_end(&mut input)
        .map_err(|error| format!("read request: {error}"))?;
    let request: Value =
        serde_json::from_slice(&input).map_err(|error| format!("parse request: {error}"))?;
    match mode.as_str() {
        "malformed" => {
            std::io::stdout()
                .write_all(b"{")
                .map_err(|error| format!("write malformed response: {error}"))?;
            return Ok(());
        }
        "oversize" => {
            std::io::stdout()
                .write_all(&vec![b'x'; 2048])
                .map_err(|error| format!("write oversized response: {error}"))?;
            return Ok(());
        }
        "stderr-oversize" => {
            std::io::stderr()
                .write_all(&vec![b'e'; 2048])
                .map_err(|error| format!("write oversized stderr: {error}"))?;
        }
        "fail" => return Err("intentional adapter failure".to_owned()),
        "sleep" => std::thread::sleep(std::time::Duration::from_secs(2)),
        "mutate" => {
            let workspace = required_string(&request, "workspace_root")?;
            fs::write(
                Path::new(workspace).join("typescript.ts"),
                "export const greeting = \"mutated\";\n",
            )
            .map_err(|error| format!("mutate fixture source: {error}"))?;
        }
        _ => {}
    }
    let fingerprint = required_string(&request, "source_fingerprint_sha256")?;
    let response = match mode.as_str() {
        "packet" => json!({
            "packet": "deterministic Impresari Context fixture packet",
            "usage": usage(20, 10, 1, 1),
            "source_fingerprint_sha256": fingerprint,
        }),
        "agent" | "require-openai-key" | "sleep" | "stderr-oversize" | "unlisted-evidence"
        | "mutate" => {
            if mode == "require-openai-key" && env::var_os("OPENAI_API_KEY").is_none() {
                return Err("OPENAI_API_KEY did not reach agent adapter".to_owned());
            }
            let mut response = agent_response(&request, fingerprint)?;
            if mode == "unlisted-evidence" {
                let workspace = required_string(&request, "workspace_root")?;
                let bytes = line_range(&Path::new(workspace).join("unlisted.txt"), 1, 1)?;
                response["evidence"] = json!([{
                    "path": "unlisted.txt",
                    "line_start": 1,
                    "line_end": 1,
                    "sha256": hash_bytes(&bytes),
                }]);
            }
            response
        }
        _ => return Err("mode must be packet or agent".to_owned()),
    };
    let bytes =
        serde_json::to_vec(&response).map_err(|error| format!("encode response: {error}"))?;
    std::io::stdout()
        .write_all(&bytes)
        .map_err(|error| format!("write response: {error}"))
}

fn agent_response(request: &Value, fingerprint: &str) -> Result<Value, String> {
    let task_id = required_string(request, "task_id")?;
    let workspace = required_string(request, "workspace_root")?;
    let arm = required_string(request, "arm")?;
    let (correct_answer, path, line_start, line_end) = match task_id {
        "typescript" => ("hello-typescript", "typescript.ts", 1, 1),
        "python" => ("hello-python", "python.py", 1, 1),
        "go" => ("hello-go", "go.go", 2, 2),
        "rust" => ("hello-rust", "rust.rs", 1, 1),
        "strict-json" => ("hello-json", "strict.json", 1, 1),
        _ => return Err(format!("unknown fixture task {task_id:?}")),
    };
    let evidence_bytes = line_range(&Path::new(workspace).join(path), line_start, line_end)?;
    let treatment = arm == "treatment";
    let answer = if treatment {
        correct_answer
    } else {
        "intentionally incorrect cold baseline"
    };
    let evidence = if treatment {
        json!([{
            "path": path,
            "line_start": line_start,
            "line_end": line_end,
            "sha256": hash_bytes(&evidence_bytes),
        }])
    } else {
        json!([])
    };
    let rendered_context = if treatment {
        let bytes = b"deterministic model context v1";
        json!({
            "renderer_identifier": required_string(request, "model_context_renderer_identifier")?,
            "renderer_version": required_string(request, "model_context_renderer_version")?,
            "bytes": bytes.len(),
            "sha256": hash_bytes(bytes),
            "evidence_count": 1,
        })
    } else {
        Value::Null
    };
    Ok(json!({
        "answer": answer,
        "usage": if treatment { usage(40, 10, 1, 1) } else { usage(80, 20, 3, 2) },
        "source_fingerprint_sha256": fingerprint,
        "evidence": evidence,
        "rendered_context": rendered_context,
    }))
}

fn usage(input: u64, output: u64, tool_calls: u64, reads: u64) -> Value {
    json!({
        "input_tokens": input,
        "output_tokens": output,
        "total_tokens": input + output,
        "estimated_cost_usd": 0.0,
        "tool_calls": tool_calls,
        "repository_file_reads": reads,
        "repeated_repository_file_reads": 0,
    })
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("request field {key:?} must be a string"))
}

fn line_range(path: &Path, first: usize, last: usize) -> Result<Vec<u8>, String> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let lines = text.lines().collect::<Vec<_>>();
    if first == 0 || last < first || last > lines.len() {
        return Err(format!("invalid line range for {}", path.display()));
    }
    Ok(lines[first - 1..last].join("\n").into_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::from("sha256:");
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
