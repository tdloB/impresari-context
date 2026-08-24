// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin command-line adapter over the shared Impresari Context engine."]

use std::{
    fmt::Write as _,
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
    RequestContext, SnapshotStatus, StructuralImpactRequest, TaskProfile,
};
use context_mcp::{MCP_PROTOCOL_VERSION, McpServer, ServerConfig};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_structural::{StructuralGraph, WorkerLauncher};
use context_workspace::DiscoveryPolicy;
use serde::Serialize;
use serde_json::json;

const DOCTOR_SCHEMA_VERSION: &str = "1.0.0";

const HELP: &str = "\
Impresari Context (working name)\n\
Usage:\n\
  impresari-context [global-options] workspace open <root> <cache-root>\n\
  impresari-context [global-options] snapshot build <root> <cache-root>\n\
  impresari-context [global-options] snapshot status <root> <cache-root> <expected-snapshot>\n\
  impresari-context [global-options] search <root> <cache-root> <exact_path|filename|literal|lexical> <query>\n\
  impresari-context [global-options] context build <root> <cache-root> <kind> <query> <purpose>\n\
  impresari-context [global-options] context profile-build <root> <cache-root> <profile> <query>\n\
  impresari-context [global-options] context profile-structure-build <root> <cache-root> <profile> <query> <graph-json> <start-node> <edge-kinds|all>\n\
  impresari-context [global-options] structure build <root> <cache-root> <worker> <worker-sha256> <empty-dir>\n\
  impresari-context [global-options] structure query <root> <cache-root> <graph-json> <start-node> <edge-kinds|all>\n\
  impresari-context [global-options] evidence expand <root> <cache-root> <evidence-json> <before> <after> <max>\n\
  impresari-context [global-options] packet validate <root> <cache-root> <packet-json>\n\
  impresari-context [global-options] handoff export <root> <cache-root> <packet-json> <export-root> <filename>\n\
  impresari-context [global-options] client kit render <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root>\n\
  impresari-context [global-options] client kit inspect <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit validate <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit install <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit remove <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] doctor inspect <root> <cache-root>\n\
  impresari-context [global-options] doctor mcp <root> <cache-root>\n\
  impresari-context [global-options] doctor codex-config <root> <cache-root> <config-toml>\n\
  impresari-context [global-options] doctor cursor-config <root> <cache-root> <mcp-json>\n\
  impresari-context [global-options] doctor claude-config <root> <cache-root> <mcp-json>\n\
  impresari-context [global-options] doctor gemini-config <root> <cache-root> <settings-json>\n\
  impresari-context [global-options] doctor copilot-config <root> <cache-root> <mcp-json>\n\
  impresari-context [global-options] doctor vscode-config <root> <cache-root> <mcp-json>\n\
Global options:\n\
  --human                 Add a concise diagnostic to stderr.\n\
  --at <UTC>              Deterministic RFC3339 operation time.\n\
  --cutoff <UTC>          Explicit audit retention cutoff.\n\
  --id-seed <8-64 chars>  Deterministic request/event identifier seed.\n\
  --apply                 Permit the explicit client-kit install or remove write.\n\
  --help                  Show this help.\n";

#[derive(Debug)]
struct GlobalOptions {
    human: bool,
    at: String,
    cutoff: String,
    id_seed: String,
    apply: bool,
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

#[derive(Serialize)]
struct ManagedConnectionKit {
    schema_name: &'static str,
    schema_version: &'static str,
    client: &'static str,
    level: &'static str,
    operation: &'static str,
    target_scope: &'static str,
    ownership: &'static str,
    external_write_performed: bool,
    configuration: serde_json::Value,
    limitations: Vec<&'static str>,
}

#[derive(Serialize)]
struct ManagedConnectionOperation {
    schema_name: &'static str,
    schema_version: &'static str,
    client: &'static str,
    level: &'static str,
    operation: &'static str,
    target_config: String,
    ownership: &'static str,
    external_write_performed: bool,
    state: &'static str,
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
        ["context", "profile-build", root, cache, profile, query] => {
            let profile = parse_task_profile(profile)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_context(
                &contexts.next(profile_purpose(profile)),
                profile,
                query,
                default_budget(),
            )?;
            Output::new("context profile-build", &result)
        }
        [
            "context",
            "profile-structure-build",
            root,
            cache,
            profile,
            query,
            graph_path,
            start_node,
            edge_kinds,
        ] => {
            let profile = parse_task_profile(profile)?;
            let graph: StructuralGraph =
                read_json(Path::new(graph_path), Capability::StructureQuery)?;
            let edge_kinds = parse_edge_kinds(edge_kinds)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_structural_context(
                &contexts.next(profile_purpose(profile)),
                profile,
                query,
                &StructuralImpactRequest {
                    graph,
                    start_node: start_node.to_string(),
                    edge_kinds,
                },
                default_budget(),
            )?;
            Output::new("context profile-structure-build", &result)
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
        ["client", "kit", "render", client, binary, root, cache] => {
            let kit = managed_connection_kit(
                client,
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
            )?;
            Output::new("client kit render", &kit)
        }
        [
            "client",
            "kit",
            "inspect",
            client,
            binary,
            root,
            cache,
            target,
        ] => {
            let operation = inspect_managed_connection(
                client,
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
                Path::new(target),
            )?;
            Output::new("client kit inspect", &operation)
        }
        [
            "client",
            "kit",
            "validate",
            client,
            binary,
            root,
            cache,
            target,
        ] => {
            let operation = validate_managed_connection(
                client,
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
                Path::new(target),
            )?;
            Output::new("client kit validate", &operation)
        }
        [
            "client",
            "kit",
            "install",
            client,
            binary,
            root,
            cache,
            target,
        ] => {
            let operation = install_managed_connection(
                client,
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
                Path::new(target),
                options.apply,
            )?;
            Output::new("client kit install", &operation)
        }
        [
            "client",
            "kit",
            "remove",
            client,
            binary,
            root,
            cache,
            target,
        ] => {
            let operation = remove_managed_connection(
                client,
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
                Path::new(target),
                options.apply,
            )?;
            Output::new("client kit remove", &operation)
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
        ["doctor", "gemini-config", root, cache, config_path] => {
            let report = doctor_client_config(root, cache, Path::new(config_path), "gemini")?;
            Output::new("doctor gemini-config", &report)
        }
        ["doctor", "copilot-config", root, cache, config_path] => {
            let report = doctor_client_config(root, cache, Path::new(config_path), "copilot")?;
            Output::new("doctor copilot-config", &report)
        }
        ["doctor", "vscode-config", root, cache, config_path] => {
            let report = doctor_client_config(root, cache, Path::new(config_path), "vscode")?;
            Output::new("doctor vscode-config", &report)
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
    let passed = client_config_is_safe(&config, client);
    report.checks.push(DoctorCheck {
        id: match client {
            "cursor" => "cursor_mcp_configuration",
            "claude" => "claude_mcp_configuration",
            "gemini" => "gemini_mcp_configuration",
            "copilot" => "copilot_mcp_configuration",
            "vscode" => "vscode_mcp_configuration",
            _ => unreachable!("only fixed named clients are dispatched"),
        },
        status: if passed { "passed" } else { "failed" },
        remediation: "use the documented fixed local-stdio argument shape and a separate cache",
    });
    if !passed {
        report.status = "failed";
    }
    report.limitations.push(match client {
        "cursor" => "Cursor configuration syntax is checked without launching Cursor or modifying its configuration.",
        "claude" => "Claude Code configuration syntax is checked without launching Claude Code or modifying its configuration.",
        "gemini" => "Gemini CLI configuration syntax is checked without launching Gemini CLI or modifying its configuration.",
        "copilot" => "GitHub Copilot CLI configuration syntax is checked without launching Copilot CLI or modifying its configuration.",
        "vscode" => "VS Code configuration syntax is checked without launching VS Code or modifying its configuration.",
        _ => unreachable!("only fixed named clients are dispatched"),
    });
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

fn client_config_is_safe(config: &serde_json::Value, client: &str) -> bool {
    let top_level_servers = if client == "vscode" {
        "servers"
    } else {
        "mcpServers"
    };
    let Some(entry) = config
        .get(top_level_servers)
        .and_then(serde_json::Value::as_object)
        .and_then(|servers| servers.get("impresari-context"))
        .and_then(serde_json::Value::as_object)
    else {
        return false;
    };
    let allowed_keys = match client {
        "cursor" | "claude" => &["type", "command", "args"][..],
        "gemini" => &["command", "args", "trust", "includeTools"][..],
        "copilot" => &["type", "command", "args", "tools"][..],
        "vscode" => &["command", "args"][..],
        _ => return false,
    };
    if !entry.keys().all(|key| allowed_keys.contains(&key.as_str()))
        || entry
            .get("command")
            .and_then(serde_json::Value::as_str)
            .is_none_or(|command| command.is_empty() || !Path::new(command).is_absolute())
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
    let client_options_are_safe = match client {
        "cursor" | "claude" => entry
            .get("type")
            .is_none_or(|value| value.as_str() == Some("stdio")),
        "gemini" => {
            entry.get("trust").and_then(serde_json::Value::as_bool) == Some(false)
                && exact_mcp_tool_allowlist(entry.get("includeTools"))
        }
        "copilot" => {
            matches!(
                entry.get("type").and_then(serde_json::Value::as_str),
                Some("local" | "stdio")
            ) && exact_mcp_tool_allowlist(entry.get("tools"))
        }
        "vscode" => true,
        _ => false,
    };
    fixed_stdio_args_are_safe(&args) && client_options_are_safe
}

fn exact_mcp_tool_allowlist(value: Option<&serde_json::Value>) -> bool {
    value
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tools| {
            tools
                .iter()
                .map(serde_json::Value::as_str)
                .collect::<Option<Vec<_>>>()
                == Some(vec![
                    "context_session_open",
                    "context_build",
                    "context_packet_resolve",
                    "context_session_close",
                ])
        })
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

fn managed_connection_kit(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
) -> Result<ManagedConnectionKit, EngineError> {
    let binary = canonical_regular_file(binary)?;
    let workspace = canonical_directory(workspace)?;
    let cache = canonical_directory(cache)?;
    if cache == workspace || cache.starts_with(&workspace) || workspace.starts_with(&cache) {
        return Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "managed connection requires a separate cache directory",
        ));
    }
    let arguments = vec![
        "--workspace".to_owned(),
        workspace.display().to_string(),
        "--cache".to_owned(),
        cache.display().to_string(),
        "--consumer-id".to_owned(),
        format!("consumer_{client}_managed"),
        "--role".to_owned(),
        "local_user".to_owned(),
    ];
    let (client, target_scope, configuration) = match client {
        "codex" => (
            "codex",
            "project",
            json!({
                "format": "toml",
                "entry": managed_toml_block(&binary, &arguments),
            }),
        ),
        "claude" | "cursor" | "copilot" => (
            match client {
                "claude" => "claude",
                "cursor" => "cursor",
                _ => "copilot",
            },
            "project_or_user",
            json!({"format": "json", "entry": {"mcpServers": {"impresari-context": {
                "command": binary, "args": arguments
            }}}}),
        ),
        "vscode" => (
            "vscode",
            "workspace_or_user",
            json!({"format": "json", "entry": {"servers": {"impresari-context": {
                "command": binary, "args": arguments
            }}}}),
        ),
        _ => {
            return Err(synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InvalidInput,
                "unsupported managed connection client",
            ));
        }
    };
    Ok(ManagedConnectionKit {
        schema_name: "managed-connection-kit",
        schema_version: "1.0.0",
        client,
        level: "l1",
        operation: "render",
        target_scope,
        ownership: "exact_fixed_entry:impresari-context",
        external_write_performed: false,
        configuration,
        limitations: vec![
            "Rendering does not write, trust, sign in, enable, or approve a client connection.",
            "Install and owned-entry removal require separate explicit operations.",
        ],
    })
}

fn managed_connection_contract(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
) -> Result<(&'static str, PathBuf, Vec<String>, &'static str), EngineError> {
    let kit = managed_connection_kit(client, binary, workspace, cache)?;
    let binary = canonical_regular_file(binary)?;
    let workspace = canonical_directory(workspace)?;
    let cache = canonical_directory(cache)?;
    let arguments = vec![
        "--workspace".to_owned(),
        workspace.display().to_string(),
        "--cache".to_owned(),
        cache.display().to_string(),
        "--consumer-id".to_owned(),
        format!("consumer_{}_managed", kit.client),
        "--role".to_owned(),
        "local_user".to_owned(),
    ];
    let format = if kit.client == "codex" {
        "toml"
    } else {
        "json"
    };
    Ok((kit.client, binary, arguments, format))
}

fn managed_operation(
    client: &'static str,
    operation: &'static str,
    target: &Path,
    external_write_performed: bool,
    state: &'static str,
) -> ManagedConnectionOperation {
    ManagedConnectionOperation {
        schema_name: "managed-connection-operation",
        schema_version: "1.0.0",
        client,
        level: "l1",
        operation,
        target_config: target.display().to_string(),
        ownership: "exact_fixed_entry:impresari-context",
        external_write_performed,
        state,
        limitations: vec![
            "This operation does not trust, sign in, enable, or approve a client connection.",
            "Only an explicit --apply install or remove can write the named configuration file.",
        ],
    }
}

fn inspect_managed_connection(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
) -> Result<ManagedConnectionOperation, EngineError> {
    let (client, binary, arguments, format) =
        managed_connection_contract(client, binary, workspace, cache)?;
    let target = managed_config_target(target)?;
    let state = match read_managed_config(&target)? {
        None => "absent",
        Some(text)
            if managed_entry_state(format, &text, &binary, &arguments)?
                == ManagedEntryState::Owned =>
        {
            "owned"
        }
        Some(_) => "unowned_or_conflicting",
    };
    Ok(managed_operation(client, "inspect", &target, false, state))
}

fn validate_managed_connection(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
) -> Result<ManagedConnectionOperation, EngineError> {
    let (client, binary, arguments, format) =
        managed_connection_contract(client, binary, workspace, cache)?;
    let target = managed_config_target(target)?;
    let text = read_managed_config(&target)?
        .ok_or_else(|| managed_config_error("managed connection configuration is absent"))?;
    if managed_entry_state(format, &text, &binary, &arguments)? != ManagedEntryState::Owned {
        return Err(managed_config_error(
            "managed connection configuration is not the exact owned entry",
        ));
    }
    Ok(managed_operation(
        client, "validate", &target, false, "owned",
    ))
}

fn install_managed_connection(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
    apply: bool,
) -> Result<ManagedConnectionOperation, EngineError> {
    let (client, binary, arguments, format) =
        managed_connection_contract(client, binary, workspace, cache)?;
    let target = managed_config_target(target)?;
    let current = read_managed_config(&target)?;
    let next = install_managed_entry(format, current.as_deref(), &binary, &arguments)?;
    if !apply {
        return Ok(managed_operation(
            client,
            "install",
            &target,
            false,
            "preview_ready",
        ));
    }
    atomic_write_managed_config(&target, next.as_bytes())?;
    Ok(managed_operation(client, "install", &target, true, "owned"))
}

fn remove_managed_connection(
    client: &str,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
    apply: bool,
) -> Result<ManagedConnectionOperation, EngineError> {
    let (client, binary, arguments, format) =
        managed_connection_contract(client, binary, workspace, cache)?;
    let target = managed_config_target(target)?;
    let current = read_managed_config(&target)?
        .ok_or_else(|| managed_config_error("managed connection configuration is absent"))?;
    let next = remove_managed_entry(format, &current, &binary, &arguments)?;
    if !apply {
        return Ok(managed_operation(
            client,
            "remove",
            &target,
            false,
            "preview_ready",
        ));
    }
    atomic_write_managed_config(&target, next.as_bytes())?;
    Ok(managed_operation(
        client, "remove", &target, true, "removed",
    ))
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ManagedEntryState {
    Owned,
    Absent,
    Conflicting,
}

fn managed_entry_state(
    format: &str,
    text: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<ManagedEntryState, EngineError> {
    match format {
        "toml" => toml_managed_entry_state(text, binary, arguments),
        "json" => json_managed_entry_state(text, binary, arguments),
        _ => Err(managed_config_error(
            "unsupported managed configuration format",
        )),
    }
}

fn install_managed_entry(
    format: &str,
    current: Option<&str>,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    match format {
        "toml" => install_toml_managed_entry(current, binary, arguments),
        "json" => install_json_managed_entry(current, binary, arguments),
        _ => Err(managed_config_error(
            "unsupported managed configuration format",
        )),
    }
}

fn remove_managed_entry(
    format: &str,
    current: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    match format {
        "toml" => remove_toml_managed_entry(current, binary, arguments),
        "json" => remove_json_managed_entry(current, binary, arguments),
        _ => Err(managed_config_error(
            "unsupported managed configuration format",
        )),
    }
}

fn managed_config_error(message: &str) -> EngineError {
    synthetic_error(
        Capability::WorkspaceOpen,
        PublicErrorCode::InvalidInput,
        message,
    )
}

fn canonical_regular_file(path: &Path) -> Result<PathBuf, EngineError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| {
        synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::PathNotFound,
            "managed connection binary not found",
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "managed connection binary must be a regular file",
        ));
    }
    fs::canonicalize(path).map_err(|_| {
        synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InternalFailure,
            "managed connection binary could not be resolved",
        )
    })
}

fn managed_config_target(path: &Path) -> Result<PathBuf, EngineError> {
    let file_name = path
        .file_name()
        .ok_or_else(|| managed_config_error("managed configuration requires a file target"))?;
    let parent = path.parent().ok_or_else(|| {
        managed_config_error("managed configuration requires an existing parent directory")
    })?;
    let metadata = fs::symlink_metadata(parent)
        .map_err(|_| managed_config_error("managed configuration parent directory not found"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(managed_config_error(
            "managed configuration parent must be a non-symlink directory",
        ));
    }
    let parent = fs::canonicalize(parent)
        .map_err(|_| managed_config_error("managed configuration parent could not be resolved"))?;
    Ok(parent.join(file_name))
}

fn read_managed_config(path: &Path) -> Result<Option<String>, EngineError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(managed_config_error(
                "managed configuration could not be inspected",
            ));
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 1_048_576 {
        return Err(managed_config_error(
            "managed configuration must be a bounded regular non-symlink file",
        ));
    }
    fs::read_to_string(path)
        .map(Some)
        .map_err(|_| managed_config_error("managed configuration is not valid UTF-8"))
}

fn atomic_write_managed_config(path: &Path, contents: &[u8]) -> Result<(), EngineError> {
    if contents.len() > 1_048_576 {
        return Err(managed_config_error(
            "managed configuration would exceed its size limit",
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| managed_config_error("managed configuration parent is unavailable"))?;
    let temp = parent.join(format!(
        ".impcx-{}.tmp",
        unique_seed()
            .map_err(|_| managed_config_error("managed configuration temporary name failed"))?
    ));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = options.open(&temp).map_err(|_| {
        managed_config_error("managed configuration temporary file could not be created")
    })?;
    file.write_all(contents)
        .and_then(|()| file.sync_all())
        .map_err(|_| {
            let _ = fs::remove_file(&temp);
            managed_config_error("managed configuration could not be written atomically")
        })?;
    fs::rename(&temp, path).map_err(|_| {
        let _ = fs::remove_file(&temp);
        managed_config_error("managed configuration atomic replacement failed")
    })
}

fn managed_toml_block(binary: &Path, arguments: &[String]) -> String {
    format!(
        "# Impresari Context managed connection kit v1; ownership=exact_fixed_entry:impresari-context\n[mcp_servers.\"impresari-context\"]\ncommand = {}\nargs = [{}]\ndefault_tools_approval_mode = \"prompt\"",
        toml_string_literal(&binary.display().to_string()),
        arguments
            .iter()
            .map(|value| toml_string_literal(value))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn toml_string_literal(value: &str) -> String {
    let mut encoded = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '"' => encoded.push_str("\\\""),
            '\u{08}' => encoded.push_str("\\b"),
            '\t' => encoded.push_str("\\t"),
            '\n' => encoded.push_str("\\n"),
            '\u{0C}' => encoded.push_str("\\f"),
            '\r' => encoded.push_str("\\r"),
            character if character.is_control() => {
                let _ = write!(encoded, "\\u{:04X}", character as u32);
            }
            character => encoded.push(character),
        }
    }
    encoded.push('"');
    encoded
}

fn toml_managed_entry_state(
    text: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<ManagedEntryState, EngineError> {
    let value: toml::Value = toml::from_str(text)
        .map_err(|_| managed_config_error("managed TOML configuration is malformed"))?;
    let Some(entry) = value
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .and_then(|servers| servers.get("impresari-context"))
    else {
        return Ok(ManagedEntryState::Absent);
    };
    let expected: toml::Value = toml::from_str(&managed_toml_block(binary, arguments))
        .map_err(|_| managed_config_error("managed TOML template is invalid"))?;
    let expected_entry = expected
        .get("mcp_servers")
        .and_then(toml::Value::as_table)
        .and_then(|servers| servers.get("impresari-context"));
    Ok(if Some(entry) == expected_entry {
        ManagedEntryState::Owned
    } else {
        ManagedEntryState::Conflicting
    })
}

fn install_toml_managed_entry(
    current: Option<&str>,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    if let Some(text) = current {
        if toml_managed_entry_state(text, binary, arguments)? != ManagedEntryState::Absent {
            return Err(managed_config_error(
                "managed TOML configuration already has an Impresari entry",
            ));
        }
        let separator = if text.is_empty() {
            ""
        } else if text.ends_with('\n') {
            "\n"
        } else {
            "\n\n"
        };
        Ok(format!(
            "{text}{separator}{}\n",
            managed_toml_block(binary, arguments)
        ))
    } else {
        Ok(format!("{}\n", managed_toml_block(binary, arguments)))
    }
}

fn remove_toml_managed_entry(
    current: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    if toml_managed_entry_state(current, binary, arguments)? != ManagedEntryState::Owned {
        return Err(managed_config_error(
            "managed TOML configuration does not contain the exact owned entry",
        ));
    }
    let block = managed_toml_block(binary, arguments);
    let start = current
        .find(&block)
        .ok_or_else(|| managed_config_error("managed TOML ownership marker is absent"))?;
    if current[start + block.len()..].contains(&block) {
        return Err(managed_config_error(
            "managed TOML ownership marker is ambiguous",
        ));
    }
    let mut remove_start = start;
    if remove_start > 0 && current.as_bytes()[remove_start - 1] == b'\n' {
        remove_start -= 1;
    }
    let mut remove_end = start + block.len();
    if current.as_bytes().get(remove_end) == Some(&b'\n') {
        remove_end += 1;
    }
    Ok(format!(
        "{}{}",
        &current[..remove_start],
        &current[remove_end..]
    ))
}

#[derive(Debug)]
struct JsonMember {
    key: String,
    start: usize,
    end: usize,
    value_start: usize,
    value_end: usize,
}

#[derive(Debug)]
struct JsonObject {
    closing: usize,
    members: Vec<JsonMember>,
}

fn json_managed_entry(binary: &Path, arguments: &[String]) -> serde_json::Value {
    json!({"command": binary, "args": arguments})
}

fn json_managed_entry_state(
    text: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<ManagedEntryState, EngineError> {
    let root = json_root_object(text)?;
    let Some(servers) = root
        .members
        .iter()
        .find(|member| member.key == "mcpServers")
    else {
        return Ok(ManagedEntryState::Absent);
    };
    let server_object = json_object_at(text, servers.value_start)?;
    let entries = server_object
        .members
        .iter()
        .filter(|member| member.key == "impresari-context")
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return Ok(ManagedEntryState::Absent);
    }
    if entries.len() != 1 {
        return Err(managed_config_error(
            "managed JSON configuration has duplicate Impresari entries",
        ));
    }
    let actual: serde_json::Value =
        serde_json::from_str(&text[entries[0].value_start..entries[0].value_end])
            .map_err(|_| managed_config_error("managed JSON Impresari entry is malformed"))?;
    Ok(if actual == json_managed_entry(binary, arguments) {
        ManagedEntryState::Owned
    } else {
        ManagedEntryState::Conflicting
    })
}

fn install_json_managed_entry(
    current: Option<&str>,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    let entry = serde_json::to_string(&json_managed_entry(binary, arguments))
        .map_err(|_| managed_config_error("managed JSON template could not be serialized"))?;
    let Some(text) = current else {
        return Ok(format!(
            "{{\n  \"mcpServers\": {{\n    \"impresari-context\": {entry}\n  }}\n}}\n"
        ));
    };
    let root = json_root_object(text)?;
    if let Some(servers) = root
        .members
        .iter()
        .find(|member| member.key == "mcpServers")
    {
        let server_object = json_object_at(text, servers.value_start)?;
        if server_object
            .members
            .iter()
            .any(|member| member.key == "impresari-context")
        {
            return Err(managed_config_error(
                "managed JSON configuration already has an Impresari entry",
            ));
        }
        Ok(insert_json_member(
            text,
            &server_object,
            "impresari-context",
            &entry,
        ))
    } else {
        let value = format!("{{\n    \"impresari-context\": {entry}\n  }}");
        Ok(insert_json_member(text, &root, "mcpServers", &value))
    }
}

fn remove_json_managed_entry(
    current: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    if json_managed_entry_state(current, binary, arguments)? != ManagedEntryState::Owned {
        return Err(managed_config_error(
            "managed JSON configuration does not contain the exact owned entry",
        ));
    }
    let root = json_root_object(current)?;
    let servers = root
        .members
        .iter()
        .find(|member| member.key == "mcpServers")
        .ok_or_else(|| managed_config_error("managed JSON server container is absent"))?;
    let server_object = json_object_at(current, servers.value_start)?;
    let index = server_object
        .members
        .iter()
        .position(|member| member.key == "impresari-context")
        .ok_or_else(|| managed_config_error("managed JSON entry is absent"))?;
    Ok(remove_json_member(current, &server_object, index))
}

fn json_root_object(text: &str) -> Result<JsonObject, EngineError> {
    serde_json::from_str::<serde_json::Value>(text)
        .map_err(|_| managed_config_error("managed JSON configuration is malformed"))?;
    let start = skip_json_whitespace(text, 0);
    let root = json_object_at(text, start)?;
    if skip_json_whitespace(text, root.closing + 1) != text.len() {
        return Err(managed_config_error(
            "managed JSON configuration has trailing data",
        ));
    }
    Ok(root)
}

fn json_object_at(text: &str, start: usize) -> Result<JsonObject, EngineError> {
    if text.as_bytes().get(start) != Some(&b'{') {
        return Err(managed_config_error(
            "managed JSON configuration root must be an object",
        ));
    }
    let mut index = skip_json_whitespace(text, start + 1);
    let mut members = Vec::new();
    if text.as_bytes().get(index) == Some(&b'}') {
        return Ok(JsonObject {
            closing: index,
            members,
        });
    }
    loop {
        let member_start = index;
        let key_end = json_string_end(text, index)?;
        let key: String = serde_json::from_str(&text[index..key_end])
            .map_err(|_| managed_config_error("managed JSON key is malformed"))?;
        index = skip_json_whitespace(text, key_end);
        if text.as_bytes().get(index) != Some(&b':') {
            return Err(managed_config_error("managed JSON key lacks a value"));
        }
        let value_start = skip_json_whitespace(text, index + 1);
        let value_end = json_value_end(text, value_start)?;
        members.push(JsonMember {
            key,
            start: member_start,
            end: value_end,
            value_start,
            value_end,
        });
        index = skip_json_whitespace(text, value_end);
        match text.as_bytes().get(index) {
            Some(b',') => index = skip_json_whitespace(text, index + 1),
            Some(b'}') => {
                return Ok(JsonObject {
                    closing: index,
                    members,
                });
            }
            _ => {
                return Err(managed_config_error(
                    "managed JSON object separator is invalid",
                ));
            }
        }
    }
}

fn json_string_end(text: &str, start: usize) -> Result<usize, EngineError> {
    if text.as_bytes().get(start) != Some(&b'\"') {
        return Err(managed_config_error("managed JSON string is malformed"));
    }
    let bytes = text.as_bytes();
    let mut index = start + 1;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index += 2,
            b'\"' => return Ok(index + 1),
            _ => index += 1,
        }
    }
    Err(managed_config_error("managed JSON string is unterminated"))
}

fn json_value_end(text: &str, start: usize) -> Result<usize, EngineError> {
    let bytes = text.as_bytes();
    match bytes.get(start) {
        Some(b'\"') => json_string_end(text, start),
        Some(b'{' | b'[') => {
            let mut stack = vec![bytes[start]];
            let mut index = start + 1;
            while index < bytes.len() {
                match bytes[index] {
                    b'\"' => index = json_string_end(text, index)?,
                    b'{' | b'[' => {
                        stack.push(bytes[index]);
                        index += 1;
                    }
                    b'}' => {
                        if stack.pop() != Some(b'{') {
                            return Err(managed_config_error("managed JSON nesting is invalid"));
                        }
                        index += 1;
                        if stack.is_empty() {
                            return Ok(index);
                        }
                    }
                    b']' => {
                        if stack.pop() != Some(b'[') {
                            return Err(managed_config_error("managed JSON nesting is invalid"));
                        }
                        index += 1;
                        if stack.is_empty() {
                            return Ok(index);
                        }
                    }
                    _ => index += 1,
                }
            }
            Err(managed_config_error("managed JSON value is unterminated"))
        }
        Some(_) => {
            let mut index = start;
            while index < bytes.len()
                && !matches!(
                    bytes[index],
                    b',' | b'}' | b']' | b' ' | b'\n' | b'\r' | b'\t'
                )
            {
                index += 1;
            }
            if index == start {
                Err(managed_config_error("managed JSON value is malformed"))
            } else {
                Ok(index)
            }
        }
        None => Err(managed_config_error("managed JSON value is absent")),
    }
}

fn skip_json_whitespace(text: &str, mut index: usize) -> usize {
    while matches!(
        text.as_bytes().get(index),
        Some(b' ' | b'\n' | b'\r' | b'\t')
    ) {
        index += 1;
    }
    index
}

fn insert_json_member(text: &str, object: &JsonObject, key: &str, value: &str) -> String {
    let prefix = if object.members.is_empty() {
        "\n  "
    } else {
        ",\n  "
    };
    format!(
        "{}{}\"{}\": {}\n{}",
        &text[..object.closing],
        prefix,
        key,
        value,
        &text[object.closing..]
    )
}

fn remove_json_member(text: &str, object: &JsonObject, index: usize) -> String {
    let member = &object.members[index];
    let (start, end) = if object.members.len() == 1 {
        (member.start, member.end)
    } else if index + 1 < object.members.len() {
        (member.start, object.members[index + 1].start)
    } else {
        (object.members[index - 1].end, member.end)
    };
    format!("{}{}", &text[..start], &text[end..])
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

fn parse_task_profile(value: &str) -> Result<TaskProfile, EngineError> {
    match value {
        "orientation" => Ok(TaskProfile::Orientation),
        "implementation" => Ok(TaskProfile::Implementation),
        "bug_investigation" => Ok(TaskProfile::BugInvestigation),
        "change_review" => Ok(TaskProfile::ChangeReview),
        "security_review" => Ok(TaskProfile::SecurityReview),
        "test_selection" => Ok(TaskProfile::TestSelection),
        "configuration_change" => Ok(TaskProfile::ConfigurationChange),
        _ => Err(synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid deterministic context profile",
        )),
    }
}

const fn profile_purpose(profile: TaskProfile) -> &'static str {
    match profile {
        TaskProfile::Orientation => "orientation",
        TaskProfile::Implementation => "implementation",
        TaskProfile::BugInvestigation => "bug_investigation",
        TaskProfile::ChangeReview => "change_review",
        TaskProfile::SecurityReview => "security_review",
        TaskProfile::TestSelection => "test_selection",
        TaskProfile::ConfigurationChange => "configuration_change",
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
        apply: false,
        command: Vec::new(),
    };
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--human" => options.human = true,
            "--apply" => options.apply = true,
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

    #[test]
    fn managed_connection_kit_render_is_fixed_and_source_free_for_every_client() {
        let root = TestRoot::new("managed-kit-workspace");
        let cache = TestRoot::new("managed-kit-cache");
        let binary = TestRoot::new("managed-kit-binary");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        fs::write(root.0.join("source.ts"), b"export const stable = true;\n")
            .expect("source fixture");
        let source_before = fs::read(root.0.join("source.ts")).expect("source before");
        for client in ["codex", "claude", "cursor", "copilot", "vscode"] {
            let command = vec![
                "client".into(),
                "kit".into(),
                "render".into(),
                client.into(),
                binary_path.display().to_string(),
                root.0.display().to_string(),
                cache.0.display().to_string(),
            ];
            let (code, value) = invoke(&command, "managedkit");
            assert_eq!(code, 0, "{client}");
            assert_eq!(value["schema_name"], "managed-connection-kit");
            assert_eq!(value["client"], client);
            assert_eq!(value["level"], "l1");
            assert_eq!(value["operation"], "render");
            assert_eq!(value["external_write_performed"], false);
            assert_eq!(value["ownership"], "exact_fixed_entry:impresari-context");
            assert!(value["configuration"].to_string().contains("--workspace"));
            assert!(value["configuration"].to_string().contains("--cache"));
            assert!(!value["configuration"].to_string().contains("env"));
        }
        assert_eq!(
            fs::read(root.0.join("source.ts")).expect("source after"),
            source_before
        );
        let invalid = vec![
            "client".into(),
            "kit".into(),
            "render".into(),
            "unknown".into(),
            binary_path.display().to_string(),
            root.0.display().to_string(),
            cache.0.display().to_string(),
        ];
        let (code, value) = invoke(&invalid, "managedkit");
        assert_eq!(code, 1);
        assert_eq!(value["code"], "invalid_input");
    }

    #[test]
    fn managed_connection_lifecycle_is_explicit_owned_and_preserves_unrelated_configuration() {
        let root = TestRoot::new("managed-lifecycle-workspace");
        let cache = TestRoot::new("managed-lifecycle-cache");
        let binary = TestRoot::new("managed-lifecycle-binary");
        let config_root = TestRoot::new("managed-lifecycle-config");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        fs::write(root.0.join("source.ts"), b"export const stable = true;\n")
            .expect("source fixture");
        let source_before = fs::read(root.0.join("source.ts")).expect("source before");

        for client in ["codex", "claude", "cursor", "copilot", "vscode"] {
            let target = config_root.0.join(format!("{client}.config"));
            let original = if client == "codex" {
                "[other]\nname = \"stable\"\n".to_owned()
            } else {
                "{\n  \"mcpServers\": {\"other\": {\"command\": \"other\"}},\n  \"unrelated\": true\n}\n".to_owned()
            };
            fs::write(&target, &original).expect("configuration fixture");
            let base = vec![
                "client".into(),
                "kit".into(),
                "install".into(),
                client.into(),
                binary_path.display().to_string(),
                root.0.display().to_string(),
                cache.0.display().to_string(),
                target.display().to_string(),
            ];
            let (code, preview) = invoke(&base, "managedlife");
            assert_eq!(code, 0, "{client} preview");
            assert_eq!(preview["external_write_performed"], false);
            assert_eq!(
                fs::read_to_string(&target).expect("preview target"),
                original
            );

            let mut install = base.clone();
            install.push("--apply".into());
            let (code, installed) = invoke(&install, "managedlife");
            assert_eq!(code, 0, "{client} install");
            assert_eq!(installed["external_write_performed"], true);

            let validate = vec![
                "client".into(),
                "kit".into(),
                "validate".into(),
                client.into(),
                binary_path.display().to_string(),
                root.0.display().to_string(),
                cache.0.display().to_string(),
                target.display().to_string(),
            ];
            assert_eq!(invoke(&validate, "managedlife").0, 0, "{client} validate");

            let mut remove = vec![
                "client".into(),
                "kit".into(),
                "remove".into(),
                client.into(),
                binary_path.display().to_string(),
                root.0.display().to_string(),
                cache.0.display().to_string(),
                target.display().to_string(),
            ];
            assert_eq!(
                invoke(&remove, "managedlife").0,
                0,
                "{client} remove preview"
            );
            remove.push("--apply".into());
            let (code, removed) = invoke(&remove, "managedlife");
            assert_eq!(code, 0, "{client} remove");
            assert_eq!(removed["external_write_performed"], true);
            let after = fs::read_to_string(&target).expect("removed target");
            assert!(!after.contains("impresari-context"));
            if client == "codex" {
                assert_eq!(after, original);
            } else {
                let after: serde_json::Value =
                    serde_json::from_str(&after).expect("valid JSON after removal");
                assert_eq!(after["unrelated"], true);
                assert_eq!(after["mcpServers"]["other"]["command"], "other");
            }
        }
        assert_eq!(
            fs::read(root.0.join("source.ts")).expect("source after"),
            source_before
        );
    }

    #[test]
    fn managed_connection_rejects_malformed_conflicting_and_non_explicit_writes() {
        let root = TestRoot::new("managed-reject-workspace");
        let cache = TestRoot::new("managed-reject-cache");
        let binary = TestRoot::new("managed-reject-binary");
        let config_root = TestRoot::new("managed-reject-config");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        let target = config_root.0.join("claude.json");
        fs::write(&target, "{not json").expect("malformed fixture");
        let command = vec![
            "client".into(),
            "kit".into(),
            "install".into(),
            "claude".into(),
            binary_path.display().to_string(),
            root.0.display().to_string(),
            cache.0.display().to_string(),
            target.display().to_string(),
            "--apply".into(),
        ];
        let (code, value) = invoke(&command, "managedreject");
        assert_eq!(code, 1);
        assert_eq!(value["code"], "invalid_input");
        assert_eq!(
            fs::read_to_string(&target).expect("malformed preserved"),
            "{not json"
        );

        fs::write(
            &target,
            "{\"mcpServers\": {\"impresari-context\": {\"command\": \"unexpected\"}}}",
        )
        .expect("conflicting fixture");
        let (code, _) = invoke(&command, "managedreject");
        assert_eq!(code, 1);
        assert!(
            fs::read_to_string(&target)
                .expect("conflicting preserved")
                .contains("unexpected")
        );
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
    fn cli_profile_build_returns_an_auditable_deterministic_plan() {
        let source = TestRoot::new("profile-source");
        let cache = TestRoot::new("profile-cache");
        fs::write(
            source.0.join("settings.toml"),
            b"feature_name = \"planner\"\n",
        )
        .expect("source");
        let before = fs::read(source.0.join("settings.toml")).expect("before");
        let (code, result) = invoke(
            &[
                "context".into(),
                "profile-build".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                "configuration_change".into(),
                "feature_name".into(),
            ],
            "profilecli",
        );
        assert_eq!(code, 0);
        assert_eq!(result["schema_name"], "profiled-context-packet");
        assert_eq!(result["plan"]["schema_name"], "deterministic-context-plan");
        assert_eq!(result["plan"]["task_profile"], "configuration_change");
        assert!(result["packet"]["packet_id"].as_str().is_some());
        assert_eq!(
            fs::read(source.0.join("settings.toml")).expect("after"),
            before
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
                ".cjs", ".cs", ".go", ".java", ".js", ".json", ".jsonc", ".jsx", ".kt", ".kts",
                ".mjs", ".py", ".rs", ".ts", ".toml", ".tsx", ".yaml", ".yml",
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
        let binary_path = config.0.join("impresari-context-mcp");
        let binary = binary_path.to_string_lossy();
        fs::write(
            &config_path,
            serde_json::to_vec(&serde_json::json!({
                "mcpServers": { "impresari-context": {
                    "type": "stdio", "command": binary.as_ref(),
                    "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_cursor_local", "--role", "local_user"]
                }}
            }))
            .expect("Cursor config JSON"),
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

        let claude_config_path = config.0.join("claude-mcp.json");
        fs::write(
            &claude_config_path,
            serde_json::to_vec(&serde_json::json!({
                "mcpServers": { "impresari-context": {
                    "command": binary.as_ref(),
                    "args": ["--workspace", "/work/source", "--cache", "/var/cache/impresari-context", "--consumer-id", "consumer_claude_local", "--role", "local_user"]
                }}
            }))
            .expect("Claude config JSON"),
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
    fn client_config_validator_accepts_documented_cursor_shape_and_rejects_unsafe_variants() {
        let binary = std::env::temp_dir()
            .join("impresari-context-mcp")
            .to_string_lossy()
            .into_owned();
        let unsafe_cache = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "type": "stdio", "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${workspaceFolder}/.cache", "--consumer-id", "consumer_cursor_local", "--role", "local_user"]
            }}
        });
        assert!(!client_config_is_safe(&unsafe_cache, "cursor"));

        let cursor_documented_shape = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_cursor_local", "--role", "local_user"]
            }}
        });
        assert!(client_config_is_safe(&cursor_documented_shape, "cursor"));

        let environment_forwarding = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_cursor_local", "--role", "local_user"],
                "env": {"UNSAFE": "1"}
            }}
        });
        assert!(!client_config_is_safe(&environment_forwarding, "cursor"));

        let relative_command = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "command": "impresari-context-mcp",
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_cursor_local", "--role", "local_user"]
            }}
        });
        assert!(!client_config_is_safe(&relative_command, "cursor"));

        let gemini_safe_shape = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "command": binary.as_str(),
                "args": ["--workspace", "/work/source", "--cache", "/var/cache/impresari-context", "--consumer-id", "consumer_gemini_local", "--role", "local_user"],
                "trust": false,
                "includeTools": ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"]
            }}
        });
        assert!(client_config_is_safe(&gemini_safe_shape, "gemini"));

        let copilot_safe_shape = serde_json::json!({
            "mcpServers": { "impresari-context": {
                "type": "local",
                "command": binary.as_str(),
                "args": ["--workspace", "/work/source", "--cache", "/var/cache/impresari-context", "--consumer-id", "consumer_copilot_local", "--role", "local_user"],
                "tools": ["context_session_open", "context_build", "context_packet_resolve", "context_session_close"]
            }}
        });
        assert!(client_config_is_safe(&copilot_safe_shape, "copilot"));

        let vscode_safe_shape = serde_json::json!({
            "servers": { "impresari-context": {
                "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
            }}
        });
        assert!(client_config_is_safe(&vscode_safe_shape, "vscode"));
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
