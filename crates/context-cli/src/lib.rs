// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin command-line adapter over the shared Impresari Context engine."]

use std::{
    fs,
    io::{self, Cursor, Write},
    path::Path,
    path::PathBuf,
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

use context_core::{
    Capability, ContextPacket, ErrorEnvelope, EvidenceRecord, PolicySubject, PublicErrorCode,
    RecoveryAction, ResourceBudget, error_envelope,
};
use context_engine::{
    ContextPlan, ContextPlanStep, EngineConfig, EngineError, LocalEngine, QueryKind,
    RequestContext, SnapshotStatus,
};
use context_mcp::{MCP_PROTOCOL_VERSION, McpServer, ServerConfig};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_structural::{StructuralGraph, WorkerLauncher};
use context_workspace::DiscoveryPolicy;
use serde::Serialize;

const DOCTOR_SCHEMA_VERSION: &str = "1.0.0";

const HELP: &str = "\
Impresari Context (working name)\n\
Usage:\n\
  impresari-context [global-options] workspace open <root> <cache-root>\n\
  impresari-context [global-options] snapshot build <root> <cache-root>\n\
  impresari-context [global-options] snapshot status <root> <cache-root> <expected-snapshot>\n\
  impresari-context [global-options] search <root> <cache-root> <exact_path|filename|literal|lexical> <query>\n\
  impresari-context [global-options] context build <root> <cache-root> <kind> <query> <purpose>\n\
  impresari-context [global-options] structure build <root> <cache-root> <worker> <worker-sha256> <empty-dir>\n\
  impresari-context [global-options] structure query <root> <cache-root> <graph-json> <start-node> <edge-kinds|all>\n\
  impresari-context [global-options] evidence expand <root> <cache-root> <evidence-json> <before> <after> <max>\n\
  impresari-context [global-options] packet validate <root> <cache-root> <packet-json>\n\
  impresari-context [global-options] handoff export <root> <cache-root> <packet-json> <export-root> <filename>\n\
  impresari-context [global-options] doctor inspect <root> <cache-root>\n\
  impresari-context [global-options] doctor mcp <root> <cache-root>\n\
  impresari-context [global-options] doctor codex-config <root> <cache-root> <config-toml>\n\
  impresari-context [global-options] doctor cursor-config <root> <cache-root> <mcp-json>\n\
  impresari-context [global-options] doctor claude-config <root> <cache-root> <mcp-json>\n\
Global options:\n\
  --human                 Add a concise diagnostic to stderr.\n\
  --at <UTC>              Deterministic RFC3339 operation time.\n\
  --cutoff <UTC>          Explicit audit retention cutoff.\n\
  --id-seed <8-64 chars>  Deterministic request/event identifier seed.\n\
  --help                  Show this help.\n";

#[derive(Debug)]
struct GlobalOptions {
    human: bool,
    at: String,
    cutoff: String,
    id_seed: String,
    command: Vec<String>,
}

struct ContextSequence {
    next: u64,
    seed: String,
    at: String,
}

impl ContextSequence {
    fn next(&mut self, purpose: &str) -> RequestContext {
        self.next += 1;
        let suffix = format!("{}{:02}", self.seed, self.next);
        RequestContext {
            request_id: format!("req_{suffix}"),
            event_id: format!("evt_{suffix}"),
            subject: PolicySubject {
                caller_id: "caller_local_cli".into(),
                role: "local_user".into(),
                purpose: purpose.into(),
            },
            occurred_at: self.at.clone(),
        }
    }
}

/// Executes one CLI invocation with injectable output streams.
///
/// Machine-readable success or error JSON is written to stdout. Optional human
/// diagnostics are written only to stderr. The return value is a process code.
pub fn execute(arguments: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    if arguments.iter().any(|argument| argument == "--help") {
        let _ = stderr.write_all(HELP.as_bytes());
        return 0;
    }
    let options = match parse_globals(arguments) {
        Ok(options) => options,
        Err(message) => {
            return emit_parse_error(stdout, stderr, &message);
        }
    };
    let mut contexts = ContextSequence {
        next: 0,
        seed: options.id_seed.clone(),
        at: options.at.clone(),
    };
    let result = dispatch(&options, &mut contexts);
    match result {
        Ok(output) => {
            if write_json(stdout, &output.value).is_err() {
                return 74;
            }
            if options.human {
                let _ = writeln!(stderr, "{} completed", output.label);
            }
            0
        }
        Err(error) => {
            if write_json(stdout, error.envelope()).is_err() {
                return 74;
            }
            if options.human {
                let _ = writeln!(stderr, "{}", error.envelope().message);
            }
            1
        }
    }
}

struct Output {
    label: &'static str,
    value: serde_json::Value,
}

impl Output {
    fn new(label: &'static str, value: &impl Serialize) -> Result<Self, EngineError> {
        let value = serde_json::to_value(value).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InternalFailure,
                "response serialization failed",
            )
        })?;
        Ok(Self { label, value })
    }
}

#[derive(Serialize)]
struct DoctorCheck {
    id: &'static str,
    status: &'static str,
    remediation: &'static str,
}

#[derive(Serialize)]
struct DoctorReport {
    schema_name: &'static str,
    schema_version: &'static str,
    status: &'static str,
    checks: Vec<DoctorCheck>,
    limitations: Vec<&'static str>,
}

#[allow(clippy::too_many_lines)]
fn dispatch(
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<Output, EngineError> {
    match options
        .command
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["workspace", "open", root, cache] => {
            let (_, handle) = open_engine(root, cache, options, contexts)?;
            Output::new("workspace open", &handle)
        }
        ["snapshot", "build", root, cache] => {
            let (mut engine, _) = open_engine(root, cache, options, contexts)?;
            let status =
                engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
            Output::new("snapshot build", &status)
        }
        ["snapshot", "status", root, cache, expected] => {
            let (mut engine, _) = open_engine(root, cache, options, contexts)?;
            let _ = engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
            let status = engine.snapshot_status_against(
                &contexts.next("snapshot_status"),
                default_budget(),
                Some(expected),
            )?;
            Output::new("snapshot status", &status)
        }
        ["search", root, cache, kind, query] => {
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.search(
                &contexts.next("search"),
                parse_kind(kind)?,
                query,
                &default_budget(),
            )?;
            Output::new("search", &result)
        }
        ["context", "build", root, cache, kind, query, purpose] => {
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_context(
                &contexts.next(purpose),
                parse_kind(kind)?,
                query,
                default_budget(),
            )?;
            Output::new("context build", &result)
        }
        [
            "structure",
            "build",
            root,
            cache,
            worker,
            expected_sha256,
            empty_directory,
        ] => {
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let launcher = WorkerLauncher {
                executable: PathBuf::from(worker),
                expected_sha256: expected_sha256.to_string(),
                empty_working_directory: PathBuf::from(empty_directory),
                timeout: Duration::from_secs(5),
            };
            let result = engine.build_structure(
                &contexts.next("structure_build"),
                &default_budget(),
                &launcher,
            )?;
            Output::new("structure build", &result)
        }
        [
            "structure",
            "query",
            root,
            cache,
            graph_path,
            start_node,
            edge_kinds,
        ] => {
            let graph: StructuralGraph =
                read_json(Path::new(graph_path), Capability::StructureQuery)?;
            let edge_kinds = parse_edge_kinds(edge_kinds)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.query_structure(
                &contexts.next("structure_query"),
                &graph,
                start_node,
                &edge_kinds,
                &default_budget(),
            )?;
            Output::new("structure query", &result)
        }
        [
            "evidence",
            "expand",
            root,
            cache,
            evidence_path,
            before,
            after,
            maximum,
        ] => {
            let evidence: EvidenceRecord =
                read_json(Path::new(evidence_path), Capability::EvidenceExpand)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.expand_evidence(
                &contexts.next("evidence_recovery"),
                &evidence,
                parse_u64(before, Capability::EvidenceExpand)?,
                parse_u64(after, Capability::EvidenceExpand)?,
                parse_u64(maximum, Capability::EvidenceExpand)?,
                default_budget(),
            )?;
            Output::new("evidence expand", &result)
        }
        ["packet", "validate", root, cache, packet_path] => {
            let packet: ContextPacket =
                read_json(Path::new(packet_path), Capability::ContextValidate)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.validate_context_packet(
                &contexts.next("packet_validation"),
                &packet,
                default_budget(),
            )?;
            Output::new("packet validate", &result)
        }
        [
            "handoff",
            "export",
            root,
            cache,
            packet_path,
            export_root,
            filename,
        ] => {
            let packet: ContextPacket =
                read_json(Path::new(packet_path), Capability::HandoffExport)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.export_handoff(
                &contexts.next("handoff"),
                &packet,
                &default_budget(),
                Path::new(export_root),
                filename,
            )?;
            Output::new("handoff export", &result)
        }
        ["doctor", "inspect", root, cache] => {
            let report = doctor_inspect(Path::new(root), Path::new(cache))?;
            Output::new("doctor inspect", &report)
        }
        ["doctor", "mcp", root, cache] => {
            let report = doctor_mcp(root, cache, options)?;
            Output::new("doctor mcp", &report)
        }
        ["doctor", "codex-config", root, cache, config_path] => {
            let report = doctor_codex_config(root, cache, Path::new(config_path))?;
            Output::new("doctor codex-config", &report)
        }
        ["doctor", "cursor-config", root, cache, config_path] => {
            let report = doctor_client_config(root, cache, Path::new(config_path), "cursor")?;
            Output::new("doctor cursor-config", &report)
        }
        ["doctor", "claude-config", root, cache, config_path] => {
            let report = doctor_client_config(root, cache, Path::new(config_path), "claude")?;
            Output::new("doctor claude-config", &report)
        }
        _ => Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "invalid command shape; use --help",
        )),
    }
}

fn doctor_inspect(workspace: &Path, cache: &Path) -> Result<DoctorReport, EngineError> {
    let workspace = canonical_directory(workspace)?;
    let cache = canonical_directory(cache)?;
    let separated =
        workspace != cache && !cache.starts_with(&workspace) && !workspace.starts_with(&cache);
    let platform_supported = matches!(
        (std::env::consts::OS, std::env::consts::ARCH),
        ("macos", "aarch64") | ("linux" | "windows", "x86_64")
    );
    let status = if separated && platform_supported {
        "partial"
    } else {
        "failed"
    };
    Ok(DoctorReport {
        schema_name: "doctor-report",
        schema_version: DOCTOR_SCHEMA_VERSION,
        status,
        checks: vec![
            DoctorCheck {
                id: "workspace_root",
                status: "passed",
                remediation: "none",
            },
            DoctorCheck {
                id: "cache_root",
                status: "passed",
                remediation: "none",
            },
            DoctorCheck {
                id: "workspace_cache_separation",
                status: if separated { "passed" } else { "failed" },
                remediation: "use a separate existing cache directory outside the workspace",
            },
            DoctorCheck {
                id: "tier_a_platform",
                status: if platform_supported {
                    "passed"
                } else {
                    "failed"
                },
                remediation: "use a published Tier A platform",
            },
        ],
        limitations: vec![
            "MCP lifecycle is not checked by doctor inspect.",
            "Structural worker identity is not checked by doctor inspect.",
            "Client configuration syntax is not checked by doctor inspect.",
        ],
    })
}

#[allow(clippy::too_many_lines)]
fn doctor_mcp(
    root: &str,
    cache: &str,
    options: &GlobalOptions,
) -> Result<DoctorReport, EngineError> {
    let mut report = doctor_inspect(Path::new(root), Path::new(cache))?;
    if report.status == "failed" {
        report
            .limitations
            .push("MCP lifecycle is not checked when prerequisite checks fail.");
        return Ok(report);
    }
    let plan = ContextPlan {
        steps: vec![ContextPlanStep {
            kind: QueryKind::Literal,
            query: "__impresari_doctor_probe__".into(),
        }],
    };
    let request = RequestContext {
        request_id: "req_doctor_mcp_build".into(),
        event_id: "evt_doctor_mcp_build".into(),
        subject: PolicySubject {
            caller_id: "consumer_doctor_local".into(),
            role: "local_user".into(),
            purpose: "doctor_mcp_equivalence".into(),
        },
        occurred_at: options.at.clone(),
    };
    let budget = default_budget();
    let direct_cache = Path::new(cache).join("doctor-direct");
    let mcp_cache = Path::new(cache).join("doctor-mcp");
    let (mut direct_engine, _) = LocalEngine::open(
        config(&direct_cache, &options.cutoff)?,
        &RequestContext {
            request_id: "req_doctor_mcp_direct_open".into(),
            event_id: "evt_doctor_mcp_direct_open".into(),
            subject: request.subject.clone(),
            occurred_at: options.at.clone(),
        },
        Path::new(root),
    )?;
    direct_engine.build_snapshot(
        &RequestContext {
            request_id: "req_doctor_mcp_direct_snapshot".into(),
            event_id: "evt_doctor_mcp_direct_snapshot".into(),
            subject: request.subject.clone(),
            occurred_at: options.at.clone(),
        },
        budget.clone(),
    )?;
    let expected_packet = direct_engine.build_planned_context(&request, &plan, budget.clone())?;
    drop(direct_engine);
    let (mut mcp_engine, _) = LocalEngine::open(
        config(&mcp_cache, &options.cutoff)?,
        &RequestContext {
            request_id: "req_doctor_mcp_transport_open".into(),
            event_id: "evt_doctor_mcp_transport_open".into(),
            subject: request.subject.clone(),
            occurred_at: options.at.clone(),
        },
        Path::new(root),
    )?;
    mcp_engine.build_snapshot(
        &RequestContext {
            request_id: "req_doctor_mcp_transport_snapshot".into(),
            event_id: "evt_doctor_mcp_transport_snapshot".into(),
            subject: request.subject.clone(),
            occurred_at: options.at.clone(),
        },
        budget.clone(),
    )?;
    let server = McpServer::new(
        mcp_engine,
        ServerConfig {
            consumer_id: request.subject.caller_id.clone(),
            role: request.subject.role.clone(),
            session_policy: SessionPolicy::new(1, 1, 65_536).map_err(|_| {
                synthetic_error(
                    Capability::WorkspaceOpen,
                    PublicErrorCode::InternalFailure,
                    "doctor session policy is invalid",
                )
            })?,
        },
    );
    let exchange = doctor_mcp_exchange(server, &request, &plan, &budget);
    report.checks.push(DoctorCheck {
        id: "mcp_initialize_and_tool_discovery",
        status: if exchange.lifecycle_passed {
            "passed"
        } else {
            "failed"
        },
        remediation: "rebuild the local MCP package and rerun doctor mcp",
    });
    report.checks.push(DoctorCheck {
        id: "direct_engine_mcp_packet_equivalence",
        status: if exchange.packet.as_ref() == Some(&expected_packet) {
            "passed"
        } else {
            "failed"
        },
        remediation: "rebuild the local engine and MCP packages from the same revision",
    });
    if !exchange.lifecycle_passed || exchange.packet.as_ref() != Some(&expected_packet) {
        report.status = "failed";
    }
    report
        .limitations
        .retain(|item| *item != "MCP lifecycle is not checked by doctor inspect.");
    report
        .limitations
        .push("doctor mcp validates in-process JSON-RPC framing, not an external client or child-binary launch.");
    Ok(report)
}

struct DoctorMcpExchange {
    lifecycle_passed: bool,
    packet: Option<ContextPacket>,
}

fn doctor_mcp_exchange(
    mut server: McpServer,
    request: &RequestContext,
    plan: &ContextPlan,
    budget: &ResourceBudget,
) -> DoctorMcpExchange {
    let requests = [
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {"name": "doctor", "version": "1"},
            },
        }),
        serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "context_build", "arguments": {
                "request_id": request.request_id,
                "event_id": request.event_id,
                "purpose": request.subject.purpose,
                "occurred_at": request.occurred_at,
                "steps": plan.steps,
                "budget": budget,
            }},
        }),
    ];
    let Ok(mut input) = requests
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map(|frames| frames.join("\n"))
    else {
        return DoctorMcpExchange {
            lifecycle_passed: false,
            packet: None,
        };
    };
    input.push('\n');
    let mut output = Vec::new();
    if server
        .serve(Cursor::new(input.into_bytes()), &mut output)
        .is_err()
        || output.len() > 1_048_576
    {
        return DoctorMcpExchange {
            lifecycle_passed: false,
            packet: None,
        };
    }
    let values = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(serde_json::from_slice::<serde_json::Value>)
        .collect::<Result<Vec<_>, _>>();
    let Ok(values) = values else {
        return DoctorMcpExchange {
            lifecycle_passed: false,
            packet: None,
        };
    };
    if values.len() != 3 || values[0]["result"]["protocolVersion"] != MCP_PROTOCOL_VERSION {
        return DoctorMcpExchange {
            lifecycle_passed: false,
            packet: None,
        };
    }
    let names = values[1]["result"]["tools"].as_array().map(|tools| {
        tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect::<Vec<_>>()
    });
    let lifecycle_passed = names
        == Some(vec![
            "context_session_open",
            "context_build",
            "context_packet_resolve",
            "context_session_close",
        ]);
    let packet =
        serde_json::from_value(values[2]["result"]["structuredContent"]["packet"].clone()).ok();
    DoctorMcpExchange {
        lifecycle_passed,
        packet,
    }
}

fn doctor_client_config(
    root: &str,
    cache: &str,
    config_path: &Path,
    client: &'static str,
) -> Result<DoctorReport, EngineError> {
    let mut report = doctor_inspect(Path::new(root), Path::new(cache))?;
    let config: serde_json::Value = read_json(config_path, Capability::WorkspaceOpen)?;
    let passed = client_config_is_safe(&config, client == "cursor");
    report.checks.push(DoctorCheck {
        id: if client == "cursor" {
            "cursor_mcp_configuration"
        } else {
            "claude_mcp_configuration"
        },
        status: if passed { "passed" } else { "failed" },
        remediation: "use the documented fixed local-stdio argument shape and a separate cache",
    });
    if !passed {
        report.status = "failed";
    }
    report.limitations.push(
        if client == "cursor" {
            "Cursor configuration syntax is checked without launching Cursor or modifying its configuration."
        } else {
            "Claude Code configuration syntax is checked without launching Claude Code or modifying its configuration."
        },
    );
    Ok(report)
}

fn doctor_codex_config(
    root: &str,
    cache: &str,
    config_path: &Path,
) -> Result<DoctorReport, EngineError> {
    let mut report = doctor_inspect(Path::new(root), Path::new(cache))?;
    let workspace = canonical_directory(Path::new(root))?;
    let cache = canonical_directory(Path::new(cache))?;
    let config = read_toml(config_path, Capability::WorkspaceOpen)?;
    let passed = codex_config_is_safe(&config, &workspace, &cache);
    report.checks.push(DoctorCheck {
        id: "codex_mcp_configuration",
        status: if passed { "passed" } else { "failed" },
        remediation: "use the documented project-scoped fixed local-stdio TOML entry with prompt approvals and no environment forwarding",
    });
    if !passed {
        report.status = "failed";
    }
    report.limitations.push(
        "Codex configuration syntax is checked without launching Codex or modifying its configuration.",
    );
    Ok(report)
}

fn codex_config_is_safe(config: &toml::Value, workspace: &Path, cache: &Path) -> bool {
    let Some(entry) = config
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .and_then(|servers| servers.get("impresari-context"))
        .and_then(toml::Value::as_table)
    else {
        return false;
    };
    let Some(command) = entry.get("command").and_then(toml::Value::as_str) else {
        return false;
    };
    let Some(args) = entry
        .get("args")
        .and_then(toml::Value::as_array)
        .and_then(|args| {
            args.iter()
                .map(toml::Value::as_str)
                .collect::<Option<Vec<_>>>()
        })
    else {
        return false;
    };
    if !entry.keys().all(|key| {
        matches!(
            key.as_str(),
            "command" | "args" | "enabled" | "default_tools_approval_mode"
        )
    }) {
        return false;
    }
    Path::new(command).is_absolute()
        && fs::metadata(command).is_ok_and(|metadata| metadata.is_file())
        && fixed_stdio_args_are_safe(&args)
        && configured_directory_matches(args[1], workspace)
        && configured_directory_matches(args[3], cache)
        && entry
            .get("enabled")
            .is_none_or(|value| value.as_bool() == Some(true))
        && entry
            .get("default_tools_approval_mode")
            .and_then(toml::Value::as_str)
            == Some("prompt")
}

fn configured_directory_matches(configured: &str, expected: &Path) -> bool {
    fs::canonicalize(configured).is_ok_and(|path| path == expected)
}

fn client_config_is_safe(config: &serde_json::Value, requires_stdio_type: bool) -> bool {
    let Some(entry) = config
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .and_then(|servers| servers.get("impresari-context"))
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    if (requires_stdio_type
        && entry.get("type").and_then(serde_json::Value::as_str) != Some("stdio"))
        || (!requires_stdio_type
            && entry
                .get("type")
                .is_some_and(|value| value.as_str() != Some("stdio")))
        || entry
            .get("command")
            .and_then(serde_json::Value::as_str)
            .is_none_or(str::is_empty)
    {
        return false;
    }
    let Some(args) = entry
        .get("args")
        .and_then(serde_json::Value::as_array)
        .and_then(|args| {
            args.iter()
                .map(serde_json::Value::as_str)
                .collect::<Option<Vec<_>>>()
        })
    else {
        return false;
    };
    fixed_stdio_args_are_safe(&args)
}

fn fixed_stdio_args_are_safe(args: &[&str]) -> bool {
    args.len() == 8
        && args[0] == "--workspace"
        && !args[1].is_empty()
        && args[2] == "--cache"
        && cache_argument_is_separate(args[3])
        && args[4] == "--consumer-id"
        && identifier_like(args[5])
        && args[6] == "--role"
        && identifier_like(args[7])
}

fn cache_argument_is_separate(value: &str) -> bool {
    !value.is_empty()
        && value != "${workspaceFolder}"
        && !value.starts_with("${workspaceFolder}/")
        && !value.starts_with("${workspaceFolder}\\")
}

fn identifier_like(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn canonical_directory(path: &Path) -> Result<PathBuf, EngineError> {
    let metadata = fs::metadata(path).map_err(|_| {
        synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::PathNotFound,
            "doctor input directory not found",
        )
    })?;
    if !metadata.is_dir() {
        return Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "doctor input is not a directory",
        ));
    }
    fs::canonicalize(path).map_err(|_| {
        synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InternalFailure,
            "doctor input directory could not be resolved",
        )
    })
}

fn open_engine(
    root: &str,
    cache: &str,
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<(LocalEngine, context_engine::WorkspaceHandle), EngineError> {
    LocalEngine::open(
        config(Path::new(cache), &options.cutoff)?,
        &contexts.next("workspace_open"),
        Path::new(root),
    )
}

fn prepared_engine(
    root: &str,
    cache: &str,
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<(LocalEngine, SnapshotStatus), EngineError> {
    let (mut engine, _) = open_engine(root, cache, options, contexts)?;
    let status = engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
    Ok((engine, status))
}

fn config(cache: &Path, cutoff: &str) -> Result<EngineConfig, EngineError> {
    Ok(EngineConfig {
        cache_root: cache.to_owned(),
        discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InternalFailure,
                "default discovery policy is invalid",
            )
        })?,
        audit_retention: AuditRetention::new(cutoff, 10_000, 10_485_760).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InvalidInput,
                "invalid audit retention cutoff",
            )
        })?,
    })
}

fn default_budget() -> ResourceBudget {
    ResourceBudget::conservative(65_536, 100, 10_000, 4096, 1000, 32, 30_000, 536_870_912)
        .expect("versioned default budget")
}

fn parse_kind(value: &str) -> Result<QueryKind, EngineError> {
    match value {
        "exact_path" => Ok(QueryKind::ExactPath),
        "filename" => Ok(QueryKind::Filename),
        "literal" => Ok(QueryKind::Literal),
        "lexical" => Ok(QueryKind::Lexical),
        _ => Err(synthetic_error(
            Capability::CodeSearch,
            PublicErrorCode::InvalidInput,
            "unsupported query kind",
        )),
    }
}

fn parse_edge_kinds(value: &str) -> Result<Vec<String>, EngineError> {
    if value == "all" {
        return Ok(Vec::new());
    }
    let kinds = value.split(',').map(str::to_owned).collect::<Vec<_>>();
    if kinds.is_empty()
        || kinds.iter().any(|kind| {
            !matches!(
                kind.as_str(),
                "declares" | "contains" | "imports" | "exports" | "calls"
            )
        })
    {
        return Err(synthetic_error(
            Capability::StructureQuery,
            PublicErrorCode::InvalidInput,
            "unsupported structural edge kind",
        ));
    }
    Ok(kinds)
}

fn parse_u64(value: &str, capability: Capability) -> Result<u64, EngineError> {
    value.parse().map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "invalid numeric argument",
        )
    })
}

fn read_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    capability: Capability,
) -> Result<T, EngineError> {
    let metadata = fs::metadata(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::PathNotFound,
            "input file not found",
        )
    })?;
    if !metadata.is_file() || metadata.len() > 4_194_304 {
        return Err(synthetic_error(
            capability,
            PublicErrorCode::ResourceLimit,
            "input file is not a bounded regular file",
        ));
    }
    let bytes = fs::read(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InternalFailure,
            "input read failed",
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "input JSON is invalid",
        )
    })
}

fn read_toml(path: &Path, capability: Capability) -> Result<toml::Value, EngineError> {
    let metadata = fs::metadata(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::PathNotFound,
            "input file not found",
        )
    })?;
    if !metadata.is_file() || metadata.len() > 4_194_304 {
        return Err(synthetic_error(
            capability,
            PublicErrorCode::ResourceLimit,
            "input file is not a bounded regular file",
        ));
    }
    let text = fs::read_to_string(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "input TOML is not valid UTF-8",
        )
    })?;
    toml::from_str(&text).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "input TOML is invalid",
        )
    })
}

fn parse_globals(arguments: &[String]) -> Result<GlobalOptions, String> {
    let now = unix_seconds()?;
    let mut options = GlobalOptions {
        human: false,
        at: timestamp(now),
        cutoff: timestamp(now.saturating_sub(7 * 24 * 60 * 60)),
        id_seed: unique_seed()?,
        command: Vec::new(),
    };
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--human" => options.human = true,
            "--at" | "--cutoff" | "--id-seed" => {
                let flag = arguments[index].as_str();
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                match flag {
                    "--at" => options.at.clone_from(value),
                    "--cutoff" => options.cutoff.clone_from(value),
                    "--id-seed" => options.id_seed.clone_from(value),
                    _ => unreachable!(),
                }
            }
            value if value.starts_with('-') => return Err("unknown global option".into()),
            _ => options.command.push(arguments[index].clone()),
        }
        index += 1;
    }
    if options.id_seed.len() < 8
        || options.id_seed.len() > 64
        || !options
            .id_seed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("invalid --id-seed".into());
    }
    context_core::validate_utc_timestamp(&options.at).map_err(|_| "invalid --at".to_owned())?;
    context_core::validate_utc_timestamp(&options.cutoff)
        .map_err(|_| "invalid --cutoff".to_owned())?;
    Ok(options)
}

fn synthetic_error(capability: Capability, code: PublicErrorCode, message: &str) -> EngineError {
    let envelope = error_envelope(
        code,
        message,
        false,
        capability,
        "req_clierror0",
        None,
        None,
        false,
        Some(RecoveryAction::None),
    )
    .expect("constant CLI error");
    engine_error(envelope)
}

fn engine_error(envelope: ErrorEnvelope) -> EngineError {
    // The engine intentionally owns construction. Round-trip through a tiny
    // local operation is avoided by exposing this crate-private adapter below.
    context_engine::adapter_error(envelope)
}

fn emit_parse_error(stdout: &mut dyn Write, stderr: &mut dyn Write, message: &str) -> i32 {
    let error = synthetic_error(
        Capability::WorkspaceOpen,
        PublicErrorCode::InvalidInput,
        "invalid command-line arguments",
    );
    let _ = write_json(stdout, error.envelope());
    let _ = writeln!(stderr, "{message}; use --help");
    2
}

fn write_json(output: &mut dyn Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")
}

fn unix_seconds() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system clock precedes Unix epoch".into())
}

fn unique_seed() -> Result<String, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{}{}", std::process::id(), duration.as_nanos()))
        .map_err(|_| "system clock precedes Unix epoch".into())
}

fn timestamp(seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);
    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-cli-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create root");
            Self(path)
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn direct_context(sequence: u64, purpose: &str) -> RequestContext {
        RequestContext {
            request_id: format!("req_abcdefgh{sequence:02}"),
            event_id: format!("evt_abcdefgh{sequence:02}"),
            subject: PolicySubject {
                caller_id: "caller_local_cli".into(),
                role: "local_user".into(),
                purpose: purpose.into(),
            },
            occurred_at: "2026-08-21T12:00:00Z".into(),
        }
    }

    fn invoke(command: &[String], seed: &str) -> (i32, serde_json::Value) {
        let mut arguments = vec![
            "--at".into(),
            "2026-08-21T12:00:00Z".into(),
            "--cutoff".into(),
            "2026-08-14T12:00:00Z".into(),
            "--id-seed".into(),
            seed.into(),
        ];
        arguments.extend_from_slice(command);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = execute(&arguments, &mut stdout, &mut stderr);
        assert!(stderr.is_empty());
        let value = serde_json::from_slice(&stdout).expect("machine JSON");
        (code, value)
    }

    fn toml_basic_string(value: &str) -> String {
        let mut encoded = String::from("\"");
        for character in value.chars() {
            match character {
                '\\' => encoded.push_str("\\\\"),
                '"' => encoded.push_str("\\\""),
                '\n' => encoded.push_str("\\n"),
                '\r' => encoded.push_str("\\r"),
                '\t' => encoded.push_str("\\t"),
                _ => encoded.push(character),
            }
        }
        encoded.push('"');
        encoded
    }

    #[test]
    fn cli_search_is_semantically_identical_to_direct_library_use() {
        let source = TestRoot::new("source");
        let cli_cache = TestRoot::new("cli-cache");
        let library_cache = TestRoot::new("library-cache");
        fs::write(source.0.join("sample.rs"), b"fn alpha() { beta(); }\n").expect("source");
        let arguments = vec![
            "--at".into(),
            "2026-08-21T12:00:00Z".into(),
            "--cutoff".into(),
            "2026-08-14T12:00:00Z".into(),
            "--id-seed".into(),
            "abcdefgh".into(),
            "--human".into(),
            "search".into(),
            source.0.to_string_lossy().into_owned(),
            cli_cache.0.to_string_lossy().into_owned(),
            "literal".into(),
            "beta".into(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&arguments, &mut stdout, &mut stderr), 0);
        let cli_value: serde_json::Value = serde_json::from_slice(&stdout).expect("CLI JSON");
        assert_eq!(
            String::from_utf8(stderr).expect("stderr"),
            "search completed\n"
        );

        let direct_config = EngineConfig {
            cache_root: library_cache.0.clone(),
            discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-14T12:00:00Z", 10_000, 10_485_760)
                .expect("retention"),
        };
        let (mut engine, _) = LocalEngine::open(
            direct_config,
            &direct_context(1, "workspace_open"),
            &source.0,
        )
        .expect("direct open");
        engine
            .build_snapshot(&direct_context(2, "snapshot_build"), default_budget())
            .expect("direct snapshot");
        let direct = engine
            .search(
                &direct_context(3, "search"),
                QueryKind::Literal,
                "beta",
                &default_budget(),
            )
            .expect("direct search");
        assert_eq!(
            cli_value,
            serde_json::to_value(direct).expect("direct JSON")
        );
    }

    #[test]
    fn parse_errors_are_machine_readable_and_clock_conversion_is_stable() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&["unknown".into()], &mut stdout, &mut stderr), 1);
        let envelope: ErrorEnvelope = serde_json::from_slice(&stdout).expect("error envelope");
        assert_eq!(envelope.code, PublicErrorCode::InvalidInput);
        assert_eq!(timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(timestamp(1_704_067_200), "2024-01-01T00:00:00Z");
    }

    #[test]
    fn doctor_inspect_is_source_free_non_mutating_and_detects_cache_overlap() {
        let source = TestRoot::new("doctor-source");
        let cache = TestRoot::new("doctor-cache");
        fs::write(
            source.0.join("secret.rs"),
            b"const TOKEN: &str = \"secret-value\";\n",
        )
        .expect("source");
        let before = fs::read(source.0.join("secret.rs")).expect("before");
        let (code, report) = invoke(
            &[
                "doctor".into(),
                "inspect".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
            ],
            "doctorone",
        );
        assert_eq!(code, 0);
        assert_eq!(report["schema_name"], "doctor-report");
        assert_eq!(report["schema_version"], DOCTOR_SCHEMA_VERSION);
        assert_eq!(report["status"], "partial");
        let encoded = serde_json::to_string(&report).expect("report JSON");
        assert!(!encoded.contains("secret-value"));
        assert!(!encoded.contains(source.0.to_string_lossy().as_ref()));
        assert!(!encoded.contains(cache.0.to_string_lossy().as_ref()));
        assert_eq!(fs::read(source.0.join("secret.rs")).expect("after"), before);

        let (code, overlap) = invoke(
            &[
                "doctor".into(),
                "inspect".into(),
                source.0.to_string_lossy().into_owned(),
                source.0.to_string_lossy().into_owned(),
            ],
            "doctortwo",
        );
        assert_eq!(code, 0);
        assert_eq!(overlap["status"], "failed");
        assert_eq!(
            overlap["checks"][2]["status"], "failed",
            "cache overlap must fail closed"
        );
    }

    #[test]
    fn doctor_mcp_exercises_the_real_lifecycle_without_mutating_source() {
        let source = TestRoot::new("doctor-mcp-source");
        let cache = TestRoot::new("doctor-mcp-cache");
        fs::write(
            source.0.join("module.rs"),
            b"const PRIVATE: &str = \"do-not-leak\";\n",
        )
        .expect("source");
        let before = fs::read(source.0.join("module.rs")).expect("before");
        let (code, report) = invoke(
            &[
                "doctor".into(),
                "mcp".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
            ],
            "doctormcp",
        );
        assert_eq!(code, 0);
        assert_eq!(report["status"], "partial");
        assert_eq!(
            report["checks"][4],
            serde_json::json!({
                "id": "mcp_initialize_and_tool_discovery",
                "status": "passed",
                "remediation": "rebuild the local MCP package and rerun doctor mcp",
            })
        );
        assert_eq!(
            report["checks"][5],
            serde_json::json!({
                "id": "direct_engine_mcp_packet_equivalence",
                "status": "passed",
                "remediation": "rebuild the local engine and MCP packages from the same revision",
            })
        );
        let encoded = serde_json::to_string(&report).expect("report JSON");
        assert!(!encoded.contains("do-not-leak"));
        assert!(!encoded.contains(source.0.to_string_lossy().as_ref()));
        assert!(!encoded.contains(cache.0.to_string_lossy().as_ref()));
        assert_eq!(fs::read(source.0.join("module.rs")).expect("after"), before);
    }

    #[test]
    fn compatibility_manifest_cannot_overclaim_shipped_languages_or_clients() {
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../../docs/reference/compatibility-manifest-v1.json"
        ))
        .expect("compatibility manifest JSON");
        assert_eq!(
            manifest["schema_name"],
            "impresari-context-compatibility-manifest"
        );
        assert_eq!(manifest["schema_version"], "1.0.0");
        let structural_extensions = manifest["language_support"]
            .as_array()
            .expect("language support array")
            .iter()
            .filter(|entry| entry["structural_evidence"] == "supported")
            .flat_map(|entry| entry["extensions"].as_array().into_iter().flatten())
            .filter_map(serde_json::Value::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            structural_extensions,
            std::collections::BTreeSet::from([
                ".cjs", ".go", ".js", ".json", ".jsonc", ".jsx", ".mjs", ".py", ".rs", ".ts",
                ".toml", ".tsx",
            ]),
            "the public manifest must match the shipped structural worker inventory"
        );
        assert_eq!(manifest["first_class_clients"], serde_json::json!([]));
        assert!(
            manifest["client_support"]
                .as_array()
                .expect("client support array")
                .iter()
                .all(|entry| entry["first_class"] == false)
        );
    }

    #[test]
    fn doctor_cursor_config_validates_only_the_fixed_stdio_contract() {
        let source = TestRoot::new("cursor-source");
        let cache = TestRoot::new("cursor-cache");
        let config = TestRoot::new("cursor-config");
        fs::write(
            source.0.join("source.rs"),
            b"const SECRET: &str = \"keep-private\";\n",
        )
        .expect("source");
        let source_before = fs::read(source.0.join("source.rs")).expect("source before");
        let config_path = config.0.join("mcp.json");
        fs::write(
            &config_path,
            br#"{
              "mcpServers": {
                "impresari-context": {
                  "type": "stdio",
                  "command": "/opt/impresari-context-mcp",
                  "args": [
                    "--workspace", "${workspaceFolder}",
                    "--cache", "${env:IMPRESARI_CONTEXT_CACHE}",
                    "--consumer-id", "consumer_cursor_local",
                    "--role", "local_user"
                  ]
                }
              }
            }"#,
        )
        .expect("config");
        let (code, report) = invoke(
            &[
                "doctor".into(),
                "cursor-config".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                config_path.to_string_lossy().into_owned(),
            ],
            "cursorcfg",
        );
        assert_eq!(code, 0);
        assert_eq!(report["status"], "partial");
        assert_eq!(report["checks"][4]["status"], "passed");
        let encoded = serde_json::to_string(&report).expect("report JSON");
        assert!(!encoded.contains("keep-private"));
        assert!(!encoded.contains(config_path.to_string_lossy().as_ref()));
        assert_eq!(
            fs::read(source.0.join("source.rs")).expect("source after"),
            source_before
        );

        let unsafe_config = serde_json::json!({
            "mcpServers": {
                "impresari-context": {
                    "type": "stdio",
                    "command": "/opt/impresari-context-mcp",
                    "args": [
                        "--workspace", "${workspaceFolder}",
                        "--cache", "${workspaceFolder}/.cache",
                        "--consumer-id", "consumer_cursor_local",
                        "--role", "local_user"
                    ]
                }
            }
        });
        assert!(!client_config_is_safe(&unsafe_config, true));

        let claude_config_path = config.0.join("claude-mcp.json");
        fs::write(
            &claude_config_path,
            br#"{
              "mcpServers": {
                "impresari-context": {
                  "command": "/opt/impresari-context-mcp",
                  "args": [
                    "--workspace", "/work/source",
                    "--cache", "/var/cache/impresari-context",
                    "--consumer-id", "consumer_claude_local",
                    "--role", "local_user"
                  ]
                }
              }
            }"#,
        )
        .expect("Claude config");
        let (code, claude_report) = invoke(
            &[
                "doctor".into(),
                "claude-config".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                claude_config_path.to_string_lossy().into_owned(),
            ],
            "claudecfg",
        );
        assert_eq!(code, 0);
        assert_eq!(claude_report["status"], "partial");
        assert_eq!(claude_report["checks"][4]["id"], "claude_mcp_configuration");
        assert_eq!(claude_report["checks"][4]["status"], "passed");
    }

    #[test]
    fn doctor_codex_config_validates_a_project_scoped_fixed_stdio_entry() {
        let source = TestRoot::new("codex-source");
        let cache = TestRoot::new("codex-cache");
        let config = TestRoot::new("codex-config");
        fs::write(
            source.0.join("source.rs"),
            b"const SECRET: &str = \"keep-private\";\n",
        )
        .expect("source");
        let source_before = fs::read(source.0.join("source.rs")).expect("source before");
        let binary_path = config.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"test binary placeholder").expect("binary");
        let config_path = config.0.join("config.toml");
        let binary_toml = toml_basic_string(binary_path.to_string_lossy().as_ref());
        let source_toml = toml_basic_string(source.0.to_string_lossy().as_ref());
        let cache_toml = toml_basic_string(cache.0.to_string_lossy().as_ref());
        fs::write(
            &config_path,
            format!(
                r#"[mcp_servers."impresari-context"]
command = {binary_toml}
args = [
  "--workspace", {source_toml},
  "--cache", {cache_toml},
  "--consumer-id", "consumer_codex_local",
  "--role", "local_user"
]
enabled = true
default_tools_approval_mode = "prompt"
"#,
            ),
        )
        .expect("config");
        let (code, report) = invoke(
            &[
                "doctor".into(),
                "codex-config".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                config_path.to_string_lossy().into_owned(),
            ],
            "codexcfg",
        );
        assert_eq!(code, 0, "{report}");
        assert_eq!(report["status"], "partial", "{report}");
        assert_eq!(report["checks"][4]["id"], "codex_mcp_configuration");
        assert_eq!(report["checks"][4]["status"], "passed");
        let encoded = serde_json::to_string(&report).expect("report JSON");
        assert!(!encoded.contains("keep-private"));
        assert!(!encoded.contains(source.0.to_string_lossy().as_ref()));
        assert!(!encoded.contains(cache.0.to_string_lossy().as_ref()));
        assert!(!encoded.contains(config_path.to_string_lossy().as_ref()));
        assert_eq!(
            fs::read(source.0.join("source.rs")).expect("source after"),
            source_before
        );

        let unsafe_config = toml::from_str::<toml::Value>(&format!(
            r#"[mcp_servers."impresari-context"]
command = {binary_toml}
args = [
  "--workspace", {source_toml},
  "--cache", {cache_toml},
  "--consumer-id", "consumer_codex_local",
  "--role", "local_user"
]
default_tools_approval_mode = "prompt"
env_vars = ["HOME"]
"#,
        ))
        .expect("unsafe TOML parses");
        assert!(!codex_config_is_safe(&unsafe_config, &source.0, &cache.0));

        fs::write(&config_path, "[mcp_servers").expect("malformed config");
        let (code, malformed) = invoke(
            &[
                "doctor".into(),
                "codex-config".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                config_path.to_string_lossy().into_owned(),
            ],
            "codexbad",
        );
        assert_eq!(code, 1);
        let envelope: ErrorEnvelope = serde_json::from_value(malformed).expect("error envelope");
        assert_eq!(envelope.code, PublicErrorCode::InvalidInput);
    }

    #[test]
    fn cli_packet_recovery_validation_and_handoff_lifecycle_is_complete() {
        let source = TestRoot::new("lifecycle-source");
        let cache = TestRoot::new("lifecycle-cache");
        let export = TestRoot::new("lifecycle-export");
        fs::write(source.0.join("lib.rs"), b"pub fn verified() {}\n").expect("source");
        let source_arg = source.0.to_string_lossy().into_owned();
        let cache_arg = cache.0.to_string_lossy().into_owned();
        let (code, packet_value) = invoke(
            &[
                "context".into(),
                "build".into(),
                source_arg.clone(),
                cache_arg.clone(),
                "literal".into(),
                "verified".into(),
                "review".into(),
            ],
            "lifecyclea",
        );
        assert_eq!(code, 0);
        let packet: ContextPacket = serde_json::from_value(packet_value).expect("packet");
        let packet_path = cache.0.join("packet-input.json");
        fs::write(
            &packet_path,
            serde_json::to_vec(&packet).expect("packet JSON"),
        )
        .expect("packet input");
        let evidence_path = cache.0.join("evidence-input.json");
        fs::write(
            &evidence_path,
            serde_json::to_vec(&packet.observed_evidence[0]).expect("evidence JSON"),
        )
        .expect("evidence input");

        let (code, validation) = invoke(
            &[
                "packet".into(),
                "validate".into(),
                source_arg.clone(),
                cache_arg.clone(),
                packet_path.to_string_lossy().into_owned(),
            ],
            "lifecycleb",
        );
        assert_eq!(code, 0);
        assert_eq!(validation["status"], "valid_current");
        let (code, expanded) = invoke(
            &[
                "evidence".into(),
                "expand".into(),
                source_arg.clone(),
                cache_arg.clone(),
                evidence_path.to_string_lossy().into_owned(),
                "2".into(),
                "2".into(),
                "32".into(),
            ],
            "lifecyclec",
        );
        assert_eq!(code, 0);
        assert_eq!(
            expanded["evidence_id"],
            packet.observed_evidence[0].evidence_id
        );
        let (code, receipt) = invoke(
            &[
                "handoff".into(),
                "export".into(),
                source_arg,
                cache_arg,
                packet_path.to_string_lossy().into_owned(),
                export.0.to_string_lossy().into_owned(),
                "handoff.json".into(),
            ],
            "lifecycled",
        );
        assert_eq!(code, 0);
        assert_eq!(receipt["packet_id"], packet.packet_id);
        assert!(export.0.join("handoff.json").is_file());
    }
}
