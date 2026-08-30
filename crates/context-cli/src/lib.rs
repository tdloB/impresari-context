// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin command-line adapter over the shared Impresari Context engine."]

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    fmt::Write as _,
    fs,
    io::{self, Cursor, Write},
    path::Path,
    path::PathBuf,
    time::Duration,
    time::{SystemTime, UNIX_EPOCH},
};

use context_adapters::{AdapterError, GuidedDeliveryIntent};
use context_claude_code::{
    ClaudeDeliveryError, ClaudeDeliveryPreparation, StdioClaudeCliTransport,
    deliver_claude_preview, prepare_claude_delivery, rehydrate_claude_delivery_preview,
};
use context_codex_app_server::{
    CodexDeliveryError, CodexDeliveryPreparation, StdioCodexAppServerTransport,
    deliver_codex_preview, prepare_codex_delivery, rehydrate_codex_delivery_preview,
};
use context_copilot_cli::{
    CopilotDeliveryError, CopilotDeliveryPreparation, StdioCopilotCliTransport,
    deliver_copilot_preview, prepare_copilot_delivery, rehydrate_copilot_delivery_preview,
};
use context_core::{
    Capability, ContextPacket, ErrorEnvelope, EvidenceRecord, PolicySubject, PublicErrorCode,
    RecoveryAction, ResourceBudget, error_envelope,
};
use context_cursor_agent::{
    CursorDeliveryError, CursorDeliveryPreparation, StdioCursorCliTransport,
    deliver_cursor_preview, prepare_cursor_delivery, rehydrate_cursor_delivery_preview,
};
use context_dashboard::{DashboardError, DashboardErrorCode, LocalBudgetPolicy, PolicyStore};
use context_dashboard_server::{
    DashboardServer, DashboardServerConfig, DashboardServerError, DashboardServerErrorCode,
};
use context_engine::{
    ContextPlan, ContextPlanStep, DeclaredAssociatedTests, DeclaredChangeSet,
    DeclaredConventionExemplars, EngineConfig, EngineError, IncrementalStructuralUpdate,
    LocalEngine, QueryKind, RepositoryOrientationRequest, RequestContext, SnapshotStatus,
    StructuralImpactRequest, TaskProfile,
};
use context_mcp::{MCP_PROTOCOL_VERSION, McpServer, ServerConfig};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_structural::{StructuralGraph, WorkerLauncher};
use context_vscode_copilot::{
    StdioVscodeChatTransport, VscodeDeliveryError, VscodeDeliveryPreparation,
    VscodeDeliveryReceipt, confirm_vscode_delivery, deliver_vscode_preview,
    prepare_vscode_delivery, rehydrate_vscode_delivery_preview,
};
use context_workspace::DiscoveryPolicy;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};

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
  impresari-context [global-options] context profile-change-set-build <root> <cache-root> <query> <declared-change-set-json>\n\
  impresari-context [global-options] context profile-associated-test-build <root> <cache-root> <query> <declared-associated-tests-json>\n\
  impresari-context [global-options] context profile-convention-exemplar-build <root> <cache-root> <query> <declared-convention-exemplars-json>\n\
  impresari-context [global-options] context profile-orientation-build <root> <cache-root> <query> <graph-json> <max-entries>\n\
  impresari-context [global-options] structure incremental-update <root> <cache-root> <incremental-update-json>\n\
  impresari-context [global-options] structure build <root> <cache-root> <worker> <worker-sha256> <empty-dir>\n\
  impresari-context [global-options] structure query <root> <cache-root> <graph-json> <start-node> <edge-kinds|all>\n\
  impresari-context [global-options] evidence expand <root> <cache-root> <evidence-json> <before> <after> <max>\n\
  impresari-context [global-options] packet validate <root> <cache-root> <packet-json>\n\
  impresari-context [global-options] handoff export <root> <cache-root> <packet-json> <export-root> <filename>\n\
  impresari-context [global-options] budget policy inspect <state-root>\n\
  impresari-context [global-options] budget policy apply <state-root> <policy-json> <expected-policy-id|absent> <expected-revision|absent>\n\
  impresari-context [global-options] budget policy remove <state-root> <expected-policy-id> <expected-revision>\n\
  impresari-context [global-options] budget policy rollback <state-root> <expected-policy-id|absent> <expected-revision|absent>\n\
  impresari-context [global-options] dashboard serve <audit-cache-root> <policy-state-root>\n\
  impresari-context [global-options] quickstart <codex|claude|cursor|copilot|vscode> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit render <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root>\n\
  impresari-context [global-options] client kit inspect <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit validate <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit install <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit update <codex|claude|cursor|copilot|vscode> <old-mcp-binary> <old-workspace> <old-cache-root> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client kit remove <codex|claude|cursor|copilot|vscode> <mcp-binary> <workspace> <cache-root> <config-file>\n\
  impresari-context [global-options] client delivery codex preview <workspace> <cache-root> <delivery-intent-json>\n\
  impresari-context [global-options] client delivery codex apply <delivery-preview-json> <runtime-parent> <codex-binary> <authenticated-codex-home> <expected-packet-id>\n\
  impresari-context [global-options] client delivery copilot preview <workspace> <cache-root> <delivery-intent-json>\n\
  impresari-context [global-options] client delivery copilot apply <delivery-preview-json> <runtime-parent> <copilot-binary> <authenticated-copilot-home> <github-auth-config> <expected-packet-id>\n\
  impresari-context [global-options] client delivery claude preview <workspace> <cache-root> <delivery-intent-json>\n\
  impresari-context [global-options] client delivery claude apply <delivery-preview-json> <runtime-parent> <claude-binary> <authenticated-user-home> <expected-packet-id>\n\
  impresari-context [global-options] client delivery cursor preview <workspace> <cache-root> <delivery-intent-json>\n\
  impresari-context [global-options] client delivery cursor apply <delivery-preview-json> <runtime-parent> <cursor-binary> <authenticated-user-home> <expected-packet-id>\n\
  impresari-context [global-options] client delivery vscode preview <workspace> <cache-root> <delivery-intent-json>\n\
  impresari-context [global-options] client delivery vscode apply <delivery-preview-json> <runtime-parent> <code-binary> <user-home> <expected-packet-id>\n\
  impresari-context [global-options] client delivery vscode confirm <launch-receipt-json> <expected-packet-id> <observed-packet-id>\n\
  impresari-context [global-options] client guidance render <codex|claude|cursor|copilot>\n\
  impresari-context [global-options] client guidance inspect <codex|claude|cursor|copilot> <project-root>\n\
  impresari-context [global-options] client guidance validate <codex|claude|cursor|copilot> <project-root>\n\
  impresari-context [global-options] client guidance install <codex|claude|cursor|copilot> <project-root>\n\
  impresari-context [global-options] client guidance remove <codex|claude|cursor|copilot> <project-root>\n\
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
  --budget-policy-root <path>  Apply an existing exact-owned local budget policy store at every engine admission.\n\
  --apply                 Permit an explicit preview-first owned configuration, policy, or delivery mutation.\n\
  --help                  Show this help.\n";

#[derive(Debug)]
struct GlobalOptions {
    human: bool,
    at: String,
    cutoff: String,
    id_seed: String,
    apply: bool,
    budget_policy_root: Option<PathBuf>,
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
    if options.command.len() == 4
        && options.command[0] == "dashboard"
        && options.command[1] == "serve"
    {
        return serve_dashboard(
            &options.command[2],
            &options.command[3],
            options.human,
            stdout,
            stderr,
        );
    }
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

fn serve_dashboard(
    audit_root: &str,
    policy_root: &str,
    human: bool,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let (server, ready) = match DashboardServer::bind(DashboardServerConfig::local(
        PathBuf::from(audit_root),
        PathBuf::from(policy_root),
    )) {
        Ok(bound) => bound,
        Err(error) => return emit_dashboard_server_error(error, human, stdout, stderr),
    };
    if write_json(stdout, &ready)
        .and_then(|()| stdout.flush())
        .is_err()
    {
        return 74;
    }
    if human {
        let _ = writeln!(
            stderr,
            "local metadata dashboard ready; open the bootstrap_url in this terminal's browser session"
        );
    }
    match server.run() {
        Ok(()) => 0,
        Err(error) => {
            let error = dashboard_server_cli_error(error);
            if write_json(stdout, error.envelope()).is_err() {
                return 74;
            }
            if human {
                let _ = writeln!(stderr, "{}", error.envelope().message);
            }
            1
        }
    }
}

fn emit_dashboard_server_error(
    error: DashboardServerError,
    human: bool,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let error = dashboard_server_cli_error(error);
    if write_json(stdout, error.envelope()).is_err() {
        return 74;
    }
    if human {
        let _ = writeln!(stderr, "{}", error.envelope().message);
    }
    1
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
    owned_entry: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous_owned_entry: Option<serde_json::Value>,
    planned_effect: &'static str,
    external_write_performed: bool,
    state: &'static str,
    limitations: Vec<&'static str>,
}

#[derive(Serialize)]
struct QuickstartReceipt {
    schema_name: &'static str,
    schema_version: &'static str,
    client: &'static str,
    state: &'static str,
    mcp_binary: String,
    prerequisites: DoctorReport,
    connection: ManagedConnectionOperation,
    external_write_performed: bool,
    next_steps: Vec<&'static str>,
    limitations: Vec<&'static str>,
}

struct ManagedEntryDetails<'a> {
    format: &'a str,
    binary: &'a Path,
    arguments: &'a [String],
}

#[derive(Serialize)]
struct GuidanceOperation {
    schema_name: &'static str,
    schema_version: &'static str,
    client: &'static str,
    level: &'static str,
    operation: &'static str,
    target_scope: &'static str,
    relative_target: &'static str,
    target_file: String,
    ownership: &'static str,
    content_sha256: String,
    artifact: &'static str,
    planned_effect: &'static str,
    external_write_performed: bool,
    state: &'static str,
    limitations: Vec<&'static str>,
}

struct GuidanceTemplate {
    client: &'static str,
    relative_target: &'static str,
    contents: &'static str,
    legacy_contents: &'static [&'static str],
}

#[derive(Serialize)]
struct CodexDeliveryApplyPreview {
    schema_name: &'static str,
    schema_version: &'static str,
    state: &'static str,
    expected_packet_id: String,
    client_io_performed: bool,
    apply_required: bool,
    preview: context_codex_app_server::CodexDeliveryPreview,
}

#[derive(Serialize)]
struct CopilotDeliveryApplyPreview {
    schema_name: &'static str,
    schema_version: &'static str,
    state: &'static str,
    expected_packet_id: String,
    client_io_performed: bool,
    apply_required: bool,
    preview: context_copilot_cli::CopilotDeliveryPreview,
}

#[derive(Serialize)]
struct ClaudeDeliveryApplyPreview {
    schema_name: &'static str,
    schema_version: &'static str,
    state: &'static str,
    expected_packet_id: String,
    client_io_performed: bool,
    apply_required: bool,
    preview: context_claude_code::ClaudeDeliveryPreview,
}

#[derive(Serialize)]
struct CursorDeliveryApplyPreview {
    schema_name: &'static str,
    schema_version: &'static str,
    state: &'static str,
    expected_packet_id: String,
    client_io_performed: bool,
    apply_required: bool,
    preview: context_cursor_agent::CursorDeliveryPreview,
}

#[derive(Serialize)]
struct VscodeDeliveryApplyPreview {
    schema_name: &'static str,
    schema_version: &'static str,
    state: &'static str,
    expected_packet_id: String,
    client_io_performed: bool,
    apply_required: bool,
    operator_confirmation_required_after_launch: bool,
    preview: context_vscode_copilot::VscodeDeliveryPreview,
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
            "context",
            "profile-change-set-build",
            root,
            cache,
            query,
            declaration_path,
        ] => {
            let declaration: DeclaredChangeSet =
                read_json(Path::new(declaration_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_declared_change_set_context(
                &contexts.next("change_review"),
                query,
                &declaration,
                default_budget(),
            )?;
            Output::new("context profile-change-set-build", &result)
        }
        [
            "context",
            "profile-associated-test-build",
            root,
            cache,
            query,
            declaration_path,
        ] => {
            let declaration: DeclaredAssociatedTests =
                read_json(Path::new(declaration_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_declared_associated_test_context(
                &contexts.next("test_selection"),
                query,
                &declaration,
                default_budget(),
            )?;
            Output::new("context profile-associated-test-build", &result)
        }
        [
            "context",
            "profile-convention-exemplar-build",
            root,
            cache,
            query,
            declaration_path,
        ] => {
            let declaration: DeclaredConventionExemplars =
                read_json(Path::new(declaration_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_declared_convention_exemplar_context(
                &contexts.next("implementation"),
                query,
                &declaration,
                default_budget(),
            )?;
            Output::new("context profile-convention-exemplar-build", &result)
        }
        [
            "context",
            "profile-orientation-build",
            root,
            cache,
            query,
            graph_path,
            max_entries,
        ] => {
            let graph: StructuralGraph =
                read_json(Path::new(graph_path), Capability::StructureQuery)?;
            let max_entries = max_entries.parse::<u32>().map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid repository map entry limit",
                )
            })?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_profiled_repository_orientation_context(
                &contexts.next("orientation"),
                query,
                &RepositoryOrientationRequest { graph, max_entries },
                default_budget(),
            )?;
            Output::new("context profile-orientation-build", &result)
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
        ["structure", "incremental-update", root, cache, update_path] => {
            let update: IncrementalStructuralUpdate =
                read_json(Path::new(update_path), Capability::StructureBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.apply_incremental_structural_update(
                &contexts.next("structure_incremental_update"),
                &update,
                &default_budget(),
            )?;
            Output::new("structure incremental-update", &result)
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
        ["budget", "policy", "inspect", state_root] => {
            let state = PolicyStore::open(Path::new(state_root))
                .and_then(|store| store.state())
                .map_err(dashboard_cli_error)?;
            Output::new("budget policy inspect", &state)
        }
        [
            "budget",
            "policy",
            "apply",
            state_root,
            policy_path,
            expected_policy_id,
            expected_revision,
        ] => {
            let policy: LocalBudgetPolicy =
                read_json(Path::new(policy_path), Capability::ContextBuild)?;
            let expected_policy_id = optional_expected(expected_policy_id);
            let expected_revision = optional_expected(expected_revision);
            let receipt = if options.apply {
                PolicyStore::apply(
                    Path::new(state_root),
                    policy,
                    expected_policy_id,
                    expected_revision,
                )
            } else {
                PolicyStore::preview_apply(
                    Path::new(state_root),
                    &policy,
                    expected_policy_id,
                    expected_revision,
                )
            }
            .map_err(dashboard_cli_error)?;
            Output::new("budget policy apply", &receipt)
        }
        [
            "budget",
            "policy",
            "remove",
            state_root,
            expected_policy_id,
            expected_revision,
        ] => {
            let receipt = if options.apply {
                PolicyStore::remove(Path::new(state_root), expected_policy_id, expected_revision)
            } else {
                PolicyStore::preview_remove(
                    Path::new(state_root),
                    expected_policy_id,
                    expected_revision,
                )
            }
            .map_err(dashboard_cli_error)?;
            Output::new("budget policy remove", &receipt)
        }
        [
            "budget",
            "policy",
            "rollback",
            state_root,
            expected_policy_id,
            expected_revision,
        ] => {
            let expected_policy_id = optional_expected(expected_policy_id);
            let expected_revision = optional_expected(expected_revision);
            let receipt = if options.apply {
                PolicyStore::rollback(Path::new(state_root), expected_policy_id, expected_revision)
            } else {
                PolicyStore::preview_rollback(
                    Path::new(state_root),
                    expected_policy_id,
                    expected_revision,
                )
            }
            .map_err(dashboard_cli_error)?;
            Output::new("budget policy rollback", &receipt)
        }
        ["quickstart", client, root, cache, target] => {
            let mcp_binary = sibling_mcp_binary()?;
            let receipt = quickstart_with_binary(
                client,
                &mcp_binary,
                Path::new(root),
                Path::new(cache),
                Path::new(target),
                options.apply,
            )?;
            Output::new("quickstart", &receipt)
        }
        [
            "client",
            "delivery",
            "codex",
            "preview",
            root,
            cache,
            intent_path,
        ] => {
            let intent: GuidedDeliveryIntent =
                read_json(Path::new(intent_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result =
                prepare_codex_delivery(&mut engine, intent).map_err(codex_delivery_error)?;
            Output::new("client delivery codex preview", &result)
        }
        [
            "client",
            "delivery",
            "codex",
            "apply",
            preview_path,
            runtime_parent,
            codex_binary,
            authenticated_codex_home,
            expected_packet_id,
        ] => {
            let result: CodexDeliveryPreparation =
                read_json(Path::new(preview_path), Capability::ContextBuild)?;
            let CodexDeliveryPreparation::Prepared(preview) = result else {
                return Output::new("client delivery codex apply", &result);
            };
            let preview =
                rehydrate_codex_delivery_preview(*preview).map_err(codex_delivery_error)?;
            if !options.apply {
                return Output::new(
                    "client delivery codex apply preview",
                    &CodexDeliveryApplyPreview {
                        schema_name: "codex-app-server-delivery-apply-preview",
                        schema_version: "1.0.0",
                        state: "apply_required",
                        expected_packet_id: expected_packet_id.to_string(),
                        client_io_performed: false,
                        apply_required: true,
                        preview,
                    },
                );
            }
            let runtime_parent = fs::canonicalize(runtime_parent).map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "Codex delivery cache root is unavailable",
                )
            })?;
            let transport = StdioCodexAppServerTransport::new(
                PathBuf::from(codex_binary),
                runtime_parent,
                PathBuf::from(authenticated_codex_home),
            )
            .map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid Codex App Server configuration",
                )
            })?;
            let receipt = deliver_codex_preview(&preview, expected_packet_id, &transport);
            Output::new("client delivery codex apply", &receipt)
        }
        [
            "client",
            "delivery",
            "copilot",
            "preview",
            root,
            cache,
            intent_path,
        ] => {
            let intent: GuidedDeliveryIntent =
                read_json(Path::new(intent_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result =
                prepare_copilot_delivery(&mut engine, intent).map_err(copilot_delivery_error)?;
            Output::new("client delivery copilot preview", &result)
        }
        [
            "client",
            "delivery",
            "copilot",
            "apply",
            preview_path,
            runtime_parent,
            copilot_binary,
            authenticated_copilot_home,
            github_auth_config,
            expected_packet_id,
        ] => {
            let result: CopilotDeliveryPreparation =
                read_json(Path::new(preview_path), Capability::ContextBuild)?;
            let CopilotDeliveryPreparation::Prepared(preview) = result else {
                return Output::new("client delivery copilot apply", &result);
            };
            let preview =
                rehydrate_copilot_delivery_preview(*preview).map_err(copilot_delivery_error)?;
            if !options.apply {
                return Output::new(
                    "client delivery copilot apply preview",
                    &CopilotDeliveryApplyPreview {
                        schema_name: "copilot-cli-delivery-apply-preview",
                        schema_version: "1.0.0",
                        state: "apply_required",
                        expected_packet_id: expected_packet_id.to_string(),
                        client_io_performed: false,
                        apply_required: true,
                        preview,
                    },
                );
            }
            let runtime_parent = fs::canonicalize(runtime_parent).map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "Copilot delivery runtime parent is unavailable",
                )
            })?;
            let transport = StdioCopilotCliTransport::new(
                PathBuf::from(copilot_binary),
                runtime_parent,
                PathBuf::from(authenticated_copilot_home),
                PathBuf::from(github_auth_config),
            )
            .map_err(copilot_delivery_error)?;
            let receipt = deliver_copilot_preview(&preview, expected_packet_id, &transport);
            Output::new("client delivery copilot apply", &receipt)
        }
        [
            "client",
            "delivery",
            "claude",
            "preview",
            root,
            cache,
            intent_path,
        ] => {
            let intent: GuidedDeliveryIntent =
                read_json(Path::new(intent_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result =
                prepare_claude_delivery(&mut engine, intent).map_err(claude_delivery_error)?;
            Output::new("client delivery claude preview", &result)
        }
        [
            "client",
            "delivery",
            "claude",
            "apply",
            preview_path,
            runtime_parent,
            claude_binary,
            authenticated_user_home,
            expected_packet_id,
        ] => {
            let result: ClaudeDeliveryPreparation =
                read_json(Path::new(preview_path), Capability::ContextBuild)?;
            let ClaudeDeliveryPreparation::Prepared(preview) = result else {
                return Output::new("client delivery claude apply", &result);
            };
            let preview =
                rehydrate_claude_delivery_preview(*preview).map_err(claude_delivery_error)?;
            if !options.apply {
                return Output::new(
                    "client delivery claude apply preview",
                    &ClaudeDeliveryApplyPreview {
                        schema_name: "claude-code-delivery-apply-preview",
                        schema_version: "1.0.0",
                        state: "apply_required",
                        expected_packet_id: expected_packet_id.to_string(),
                        client_io_performed: false,
                        apply_required: true,
                        preview,
                    },
                );
            }
            let runtime_parent = fs::canonicalize(runtime_parent).map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "Claude delivery runtime parent is unavailable",
                )
            })?;
            let transport = StdioClaudeCliTransport::new(
                PathBuf::from(claude_binary),
                runtime_parent,
                PathBuf::from(authenticated_user_home),
            )
            .map_err(claude_delivery_error)?;
            let receipt = deliver_claude_preview(&preview, expected_packet_id, &transport);
            Output::new("client delivery claude apply", &receipt)
        }
        [
            "client",
            "delivery",
            "cursor",
            "preview",
            root,
            cache,
            intent_path,
        ] => {
            let intent: GuidedDeliveryIntent =
                read_json(Path::new(intent_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result =
                prepare_cursor_delivery(&mut engine, intent).map_err(cursor_delivery_error)?;
            Output::new("client delivery cursor preview", &result)
        }
        [
            "client",
            "delivery",
            "cursor",
            "apply",
            preview_path,
            runtime_parent,
            cursor_binary,
            authenticated_user_home,
            expected_packet_id,
        ] => {
            let result: CursorDeliveryPreparation =
                read_json(Path::new(preview_path), Capability::ContextBuild)?;
            let CursorDeliveryPreparation::Prepared(preview) = result else {
                return Output::new("client delivery cursor apply", &result);
            };
            let preview =
                rehydrate_cursor_delivery_preview(*preview).map_err(cursor_delivery_error)?;
            if !options.apply {
                return Output::new(
                    "client delivery cursor apply preview",
                    &CursorDeliveryApplyPreview {
                        schema_name: "cursor-agent-delivery-apply-preview",
                        schema_version: "1.0.0",
                        state: "apply_required",
                        expected_packet_id: expected_packet_id.to_string(),
                        client_io_performed: false,
                        apply_required: true,
                        preview,
                    },
                );
            }
            let runtime_parent = fs::canonicalize(runtime_parent).map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "Cursor delivery runtime parent is unavailable",
                )
            })?;
            let transport = StdioCursorCliTransport::new(
                PathBuf::from(cursor_binary),
                runtime_parent,
                PathBuf::from(authenticated_user_home),
            )
            .map_err(cursor_delivery_error)?;
            let receipt = deliver_cursor_preview(&preview, expected_packet_id, &transport);
            Output::new("client delivery cursor apply", &receipt)
        }
        [
            "client",
            "delivery",
            "vscode",
            "preview",
            root,
            cache,
            intent_path,
        ] => {
            let intent: GuidedDeliveryIntent =
                read_json(Path::new(intent_path), Capability::ContextBuild)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result =
                prepare_vscode_delivery(&mut engine, intent).map_err(vscode_delivery_error)?;
            Output::new("client delivery vscode preview", &result)
        }
        [
            "client",
            "delivery",
            "vscode",
            "apply",
            preview_path,
            runtime_parent,
            code_binary,
            user_home,
            expected_packet_id,
        ] => {
            let result: VscodeDeliveryPreparation =
                read_json(Path::new(preview_path), Capability::ContextBuild)?;
            let VscodeDeliveryPreparation::Prepared(preview) = result else {
                return Output::new("client delivery vscode apply", &result);
            };
            let preview =
                rehydrate_vscode_delivery_preview(*preview).map_err(vscode_delivery_error)?;
            if !options.apply {
                return Output::new(
                    "client delivery vscode apply preview",
                    &VscodeDeliveryApplyPreview {
                        schema_name: "vscode-copilot-delivery-apply-preview",
                        schema_version: "1.0.0",
                        state: "apply_required",
                        expected_packet_id: expected_packet_id.to_string(),
                        client_io_performed: false,
                        apply_required: true,
                        operator_confirmation_required_after_launch: true,
                        preview,
                    },
                );
            }
            let runtime_parent = fs::canonicalize(runtime_parent).map_err(|_| {
                synthetic_error(
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "VS Code delivery runtime parent is unavailable",
                )
            })?;
            let transport = StdioVscodeChatTransport::new(
                PathBuf::from(code_binary),
                runtime_parent,
                PathBuf::from(user_home),
            )
            .map_err(vscode_delivery_error)?;
            let receipt = deliver_vscode_preview(&preview, expected_packet_id, &transport);
            Output::new("client delivery vscode apply", &receipt)
        }
        [
            "client",
            "delivery",
            "vscode",
            "confirm",
            receipt_path,
            expected_packet_id,
            observed_packet_id,
        ] => {
            let receipt: VscodeDeliveryReceipt =
                read_json(Path::new(receipt_path), Capability::ContextBuild)?;
            let receipt = confirm_vscode_delivery(&receipt, expected_packet_id, observed_packet_id)
                .map_err(vscode_delivery_error)?;
            Output::new("client delivery vscode confirm", &receipt)
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
            "update",
            client,
            old_binary,
            old_root,
            old_cache,
            binary,
            root,
            cache,
            target,
        ] => {
            let operation = update_managed_connection(
                client,
                Path::new(old_binary),
                Path::new(old_root),
                Path::new(old_cache),
                Path::new(binary),
                Path::new(root),
                Path::new(cache),
                Path::new(target),
                options.apply,
            )?;
            Output::new("client kit update", &operation)
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
        ["client", "guidance", "render", client] => {
            Output::new("client guidance render", &guidance_render(client)?)
        }
        ["client", "guidance", "inspect", client, root] => Output::new(
            "client guidance inspect",
            &inspect_guidance(client, Path::new(root))?,
        ),
        ["client", "guidance", "validate", client, root] => Output::new(
            "client guidance validate",
            &validate_guidance(client, Path::new(root))?,
        ),
        ["client", "guidance", "install", client, root] => Output::new(
            "client guidance install",
            &install_guidance(client, Path::new(root), options.apply)?,
        ),
        ["client", "guidance", "remove", client, root] => Output::new(
            "client guidance remove",
            &remove_guidance(client, Path::new(root), options.apply)?,
        ),
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

fn sibling_mcp_binary() -> Result<PathBuf, EngineError> {
    let executable = std::env::current_exe()
        .map_err(|_| managed_config_error("quickstart could not locate the current executable"))?;
    let parent = executable.parent().ok_or_else(|| {
        managed_config_error("quickstart could not locate the release binary directory")
    })?;
    let name = if cfg!(windows) {
        "impresari-context-mcp.exe"
    } else {
        "impresari-context-mcp"
    };
    canonical_regular_file(&parent.join(name)).map_err(|_| {
        managed_config_error("quickstart requires impresari-context-mcp beside the CLI executable")
    })
}

fn quickstart_with_binary(
    client: &str,
    mcp_binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
    apply: bool,
) -> Result<QuickstartReceipt, EngineError> {
    let prerequisites = doctor_inspect(workspace, cache)?;
    let connection =
        install_managed_connection(client, mcp_binary, workspace, cache, target, apply)?;
    Ok(QuickstartReceipt {
        schema_name: "quickstart-receipt",
        schema_version: "1.0.0",
        client: connection.client,
        state: if apply {
            "connection_installed"
        } else {
            "preview_ready"
        },
        mcp_binary: canonical_regular_file(mcp_binary)?.display().to_string(),
        prerequisites,
        external_write_performed: connection.external_write_performed,
        connection,
        next_steps: if apply {
            vec![
                "Open the named client and review its exact impresari-context server entry.",
                "Complete any client-controlled trust, start, and tool-approval steps.",
                "Use a bounded session open/build/resolve/close request to verify the live connection.",
            ]
        } else {
            vec![
                "Review the reported MCP binary, workspace, cache, target configuration, and owned entry.",
                "Rerun the same command with --apply to install only that exact owned entry.",
            ]
        },
        limitations: vec![
            "Quickstart does not discover a workspace, cache, or client configuration path.",
            "Quickstart does not trust, start, sign in to, enable, approve, or invoke a client.",
            "Native guidance remains a separate opt-in client guidance operation.",
        ],
    })
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
            "context_convention_exemplar_build",
            "structure_incremental_update",
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
        remediation: "use the documented user-scoped fixed local-stdio TOML entry with prompt approvals and no environment forwarding",
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
            "command" | "args" | "enabled" | "required" | "default_tools_approval_mode"
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
            .get("required")
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
    if client == "vscode" && (config.get("inputs").is_some() || config.get("sandbox").is_some()) {
        return false;
    }
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
        "cursor" | "claude" | "vscode" => &["type", "command", "args"][..],
        "gemini" => &["command", "args", "trust", "includeTools"][..],
        "copilot" => &["type", "command", "args", "tools"][..],
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
        "vscode" => entry.get("type").and_then(serde_json::Value::as_str) == Some("stdio"),
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
            "user",
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
            "workspace_extension_host",
            json!({"format": "json", "entry": {"servers": {"impresari-context": {
                "type": "stdio", "command": binary, "args": arguments
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
    let format = match kit.client {
        "codex" => "toml",
        "vscode" => "vscode-json",
        _ => "json",
    };
    Ok((kit.client, binary, arguments, format))
}

fn managed_operation(
    client: &'static str,
    operation: &'static str,
    target: &Path,
    entry: &ManagedEntryDetails<'_>,
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
        owned_entry: managed_entry_preview(entry.format, entry.binary, entry.arguments),
        previous_owned_entry: None,
        planned_effect: match operation {
            "install" => "add_exact_owned_entry",
            "remove" => "remove_exact_owned_entry",
            _ => "inspect_exact_owned_entry",
        },
        external_write_performed,
        state,
        limitations: vec![
            "This operation does not trust, sign in, enable, or approve a client connection.",
            "Only an explicit --apply install, update, or remove can write the named configuration file.",
        ],
    }
}

fn managed_entry_preview(format: &str, binary: &Path, arguments: &[String]) -> serde_json::Value {
    match format {
        "toml" => json!({"format": "toml", "entry": managed_toml_block(binary, arguments)}),
        "json" | "vscode-json" => {
            let container = json_server_container(format);
            json!({"format": "json", "entry": {container: {"impresari-context": json_managed_entry(format, binary, arguments)}}})
        }
        _ => json!({"format": "unsupported"}),
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
    let entry = ManagedEntryDetails {
        format,
        binary: &binary,
        arguments: &arguments,
    };
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
    Ok(managed_operation(
        client, "inspect", &target, &entry, false, state,
    ))
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
    let entry = ManagedEntryDetails {
        format,
        binary: &binary,
        arguments: &arguments,
    };
    let target = managed_config_target(target)?;
    let text = read_managed_config(&target)?
        .ok_or_else(|| managed_config_error("managed connection configuration is absent"))?;
    if managed_entry_state(format, &text, &binary, &arguments)? != ManagedEntryState::Owned {
        return Err(managed_config_error(
            "managed connection configuration is not the exact owned entry",
        ));
    }
    Ok(managed_operation(
        client, "validate", &target, &entry, false, "owned",
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
    let entry = ManagedEntryDetails {
        format,
        binary: &binary,
        arguments: &arguments,
    };
    let target = managed_config_target(target)?;
    let current = read_managed_config(&target)?;
    let next = install_managed_entry(format, current.as_deref(), &binary, &arguments)?;
    if !apply {
        return Ok(managed_operation(
            client,
            "install",
            &target,
            &entry,
            false,
            "preview_ready",
        ));
    }
    atomic_write_managed_config(&target, next.as_bytes())?;
    Ok(managed_operation(
        client, "install", &target, &entry, true, "owned",
    ))
}

#[allow(clippy::too_many_arguments)]
fn update_managed_connection(
    client: &str,
    old_binary: &Path,
    old_workspace: &Path,
    old_cache: &Path,
    binary: &Path,
    workspace: &Path,
    cache: &Path,
    target: &Path,
    apply: bool,
) -> Result<ManagedConnectionOperation, EngineError> {
    let (old_client, old_binary, old_arguments, old_format) =
        managed_connection_contract(client, old_binary, old_workspace, old_cache)?;
    let (new_client, binary, arguments, format) =
        managed_connection_contract(client, binary, workspace, cache)?;
    if old_client != new_client || old_format != format {
        return Err(managed_config_error(
            "managed connection update client contract changed unexpectedly",
        ));
    }
    let old_entry = ManagedEntryDetails {
        format: old_format,
        binary: &old_binary,
        arguments: &old_arguments,
    };
    let entry = ManagedEntryDetails {
        format,
        binary: &binary,
        arguments: &arguments,
    };
    let target = managed_config_target(target)?;
    let current = read_managed_config(&target)?
        .ok_or_else(|| managed_config_error("managed connection configuration is absent"))?;
    if managed_entry_state(format, &current, &old_binary, &old_arguments)?
        != ManagedEntryState::Owned
    {
        return Err(managed_config_error(
            "managed connection update requires the exact declared prior owned entry",
        ));
    }
    let without_old = remove_managed_entry(format, &current, &old_binary, &old_arguments)?;
    let next = install_managed_entry(format, Some(&without_old), &binary, &arguments)?;
    let mut operation = managed_operation(
        new_client,
        "update",
        &target,
        &entry,
        apply,
        if apply { "owned" } else { "preview_ready" },
    );
    operation.previous_owned_entry = Some(managed_entry_preview(
        old_entry.format,
        old_entry.binary,
        old_entry.arguments,
    ));
    operation.planned_effect = "replace_exact_owned_entry";
    if apply {
        atomic_write_managed_config(&target, next.as_bytes())?;
    }
    Ok(operation)
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
    let entry = ManagedEntryDetails {
        format,
        binary: &binary,
        arguments: &arguments,
    };
    let target = managed_config_target(target)?;
    let current = read_managed_config(&target)?
        .ok_or_else(|| managed_config_error("managed connection configuration is absent"))?;
    let next = remove_managed_entry(format, &current, &binary, &arguments)?;
    if !apply {
        return Ok(managed_operation(
            client,
            "remove",
            &target,
            &entry,
            false,
            "preview_ready",
        ));
    }
    if managed_document_is_empty(format, &next)? {
        remove_managed_config(&target)?;
    } else {
        atomic_write_managed_config(&target, next.as_bytes())?;
    }
    Ok(managed_operation(
        client, "remove", &target, &entry, true, "removed",
    ))
}

const GUIDANCE_MAX_BYTES: u64 = 16 * 1024;
const GUIDANCE_OWNERSHIP: &str = "exact_fixed_artifact:impresari-context";
const LEGACY_CODEX_GUIDANCE_V1: &str = r"<!-- Impresari Context native guidance v1; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when the
user requests repository context or an evidence-backed implementation,
investigation, review, test-selection, orientation, or configuration-change
task.

- Ask for or state one explicit supported profile and a bounded evidence budget.
- Treat every packet as snapshot-bound evidence. Show its packet ID, plan ID,
  reason codes, coverage, and omissions when relying on it.
- Do not infer unsupported runtime behavior, alter MCP configuration, bypass
  client approvals, execute repository code, or expand the requested budget.
- If the MCP server or packet is unavailable, say so briefly and continue with
  ordinary repository analysis; do not fabricate evidence.
";
const LEGACY_CLAUDE_GUIDANCE_V1: &str = r"---
name: impresari-context
description: Request bounded, source-grounded Impresari Context evidence when a user asks for repository context, implementation, investigation, review, testing, orientation, or configuration analysis.
---

<!-- Impresari Context native guidance v1; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use the already configured local `impresari-context` MCP server only for an
explicit supported task profile and bounded evidence budget. Treat returned
packets as snapshot-bound evidence: surface packet ID, plan ID, reason codes,
coverage, and omissions before relying on them.

Never alter MCP configuration, client approvals, budgets, source files, or
repository execution authority. If the server or packet is unavailable, state
that limitation and continue with ordinary analysis without fabricating
evidence.
";
const LEGACY_CURSOR_GUIDANCE_V1: &str = r"---
description: Use bounded, snapshot-grounded Impresari Context evidence for explicit repository-context tasks.
alwaysApply: false
---

<!-- Impresari Context native guidance v1; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only for an
explicit supported task profile and hard evidence budget. Show packet ID, plan
ID, reason codes, coverage, and omissions when using a returned packet.

Do not change MCP configuration, trust, approvals, source files, or execution
authority. Do not infer unsupported runtime behavior. If evidence is unavailable,
continue with normal analysis and state the limitation.
";
const LEGACY_COPILOT_GUIDANCE_V1: &str = r#"---
applyTo: "**"
---

<!-- Impresari Context native guidance v1; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when a task
explicitly calls for bounded repository evidence. Select a supported task
profile and hard budget; when a packet is returned, surface packet ID, plan ID,
reason codes, coverage, and omissions.

Do not alter MCP configuration, trust, approvals, source files, or execution
authority. If the server or packet is unavailable, state that and continue with
ordinary analysis without claiming unsupported evidence.
"#;
const LEGACY_COPILOT_GUIDANCE_V2: &str = r#"---
applyTo: "**"
---

<!-- Impresari Context native guidance v2; ownership=exact_fixed_artifact:impresari-context -->

# Impresari Context evidence guidance

Use an already configured local `impresari-context` MCP server only when a task
explicitly calls for bounded repository evidence. Select a supported task
profile and hard budget; when a packet is returned, surface packet ID, plan ID,
reason codes, coverage, and omissions.

For a session-scoped packet, use `context_session_open`, then
`context_build`, `context_packet_resolve`, and `context_session_close` in that
order. The build uses one explicit profile and a hard budget that validates
against the live tool schema; resolve only the returned packet ID in the same
session.

Do not alter MCP configuration, trust, approvals, source files, or execution
authority. If the server or packet is unavailable, state that and continue with
ordinary analysis without claiming unsupported evidence.
"#;

fn guidance_template(client: &str) -> Result<GuidanceTemplate, EngineError> {
    let template = match client {
        "codex" => GuidanceTemplate {
            client: "codex",
            relative_target: "AGENTS.md",
            contents: include_str!("../../../templates/client-guidance/codex/AGENTS.md"),
            legacy_contents: &[LEGACY_CODEX_GUIDANCE_V1],
        },
        "claude" => GuidanceTemplate {
            client: "claude",
            relative_target: ".claude/skills/impresari-context/SKILL.md",
            contents: include_str!("../../../templates/client-guidance/claude/SKILL.md"),
            legacy_contents: &[LEGACY_CLAUDE_GUIDANCE_V1],
        },
        "cursor" => GuidanceTemplate {
            client: "cursor",
            relative_target: ".cursor/rules/impresari-context.mdc",
            contents: include_str!(
                "../../../templates/client-guidance/cursor/impresari-context.mdc"
            ),
            legacy_contents: &[LEGACY_CURSOR_GUIDANCE_V1],
        },
        "copilot" => GuidanceTemplate {
            client: "copilot",
            relative_target: ".github/instructions/impresari-context.instructions.md",
            contents: include_str!(
                "../../../templates/client-guidance/copilot/impresari-context.instructions.md"
            ),
            legacy_contents: &[LEGACY_COPILOT_GUIDANCE_V1, LEGACY_COPILOT_GUIDANCE_V2],
        },
        _ => return Err(guidance_error("unsupported native guidance client")),
    };
    if template.contents.len() > usize::try_from(GUIDANCE_MAX_BYTES).unwrap_or(usize::MAX)
        || !template
            .contents
            .contains("ownership=exact_fixed_artifact:impresari-context")
    {
        return Err(guidance_error(
            "released native guidance template is invalid",
        ));
    }
    Ok(template)
}

fn guidance_digest(contents: &str) -> String {
    let digest = Sha256::digest(contents.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn guidance_operation(
    template: &GuidanceTemplate,
    contents: &'static str,
    operation: &'static str,
    target: &Path,
    external_write_performed: bool,
    state: &'static str,
) -> GuidanceOperation {
    GuidanceOperation {
        schema_name: "native-guidance-operation",
        schema_version: "1.0.0",
        client: template.client,
        level: "l2",
        operation,
        target_scope: "project",
        relative_target: template.relative_target,
        target_file: target.display().to_string(),
        ownership: GUIDANCE_OWNERSHIP,
        content_sha256: guidance_digest(contents),
        artifact: contents,
        planned_effect: match operation {
            "install" => "create_exact_owned_artifact",
            "remove" => "remove_exact_owned_artifact",
            _ => "inspect_exact_owned_artifact",
        },
        external_write_performed,
        state,
        limitations: vec![
            "This operation does not trust, sign in, enable, approve, configure, or invoke a client.",
            "Only an explicit --apply install or remove can write the fixed owned guidance artifact.",
        ],
    }
}

fn guidance_target(root: &Path, template: &GuidanceTemplate) -> Result<PathBuf, EngineError> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|_| guidance_error("native guidance project root not found"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(guidance_error(
            "native guidance project root must be a non-symlink directory",
        ));
    }
    let root = fs::canonicalize(root)
        .map_err(|_| guidance_error("native guidance project root could not be resolved"))?;
    let relative = Path::new(template.relative_target);
    let file_name = relative
        .file_name()
        .ok_or_else(|| guidance_error("native guidance target is invalid"))?;
    let parent_relative = relative.parent().unwrap_or_else(|| Path::new(""));
    let requested_parent = root.join(parent_relative);
    let parent_metadata = fs::symlink_metadata(&requested_parent)
        .map_err(|_| guidance_error("native guidance target parent directory not found"))?;
    if parent_metadata.file_type().is_symlink() || !parent_metadata.is_dir() {
        return Err(guidance_error(
            "native guidance target parent must be a non-symlink directory",
        ));
    }
    let parent = fs::canonicalize(&requested_parent)
        .map_err(|_| guidance_error("native guidance target parent could not be resolved"))?;
    if !parent.starts_with(&root) {
        return Err(guidance_error(
            "native guidance target parent escapes the project root",
        ));
    }
    Ok(parent.join(file_name))
}

fn read_guidance_target(path: &Path) -> Result<Option<String>, EngineError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(guidance_error(
                "native guidance target could not be inspected",
            ));
        }
    };
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > GUIDANCE_MAX_BYTES
    {
        return Err(guidance_error(
            "native guidance target must be a bounded regular non-symlink file",
        ));
    }
    fs::read_to_string(path)
        .map(Some)
        .map_err(|_| guidance_error("native guidance target is not valid UTF-8"))
}

fn recognized_guidance_contents(
    contents: Option<&str>,
    template: &GuidanceTemplate,
) -> Option<&'static str> {
    match contents {
        Some(contents) if contents == template.contents => Some(template.contents),
        Some(contents) => template
            .legacy_contents
            .iter()
            .copied()
            .find(|legacy| contents == *legacy),
        None => None,
    }
}

fn guidance_state(contents: Option<&str>, template: &GuidanceTemplate) -> &'static str {
    match contents {
        None => "absent",
        Some(contents) if contents == template.contents => "owned",
        Some(contents) if template.legacy_contents.contains(&contents) => "owned_legacy",
        Some(_) => "unowned_or_conflicting",
    }
}

fn guidance_render(client: &str) -> Result<GuidanceOperation, EngineError> {
    let template = guidance_template(client)?;
    Ok(guidance_operation(
        &template,
        template.contents,
        "render",
        Path::new(template.relative_target),
        false,
        "rendered",
    ))
}

fn inspect_guidance(client: &str, root: &Path) -> Result<GuidanceOperation, EngineError> {
    let template = guidance_template(client)?;
    let target = guidance_target(root, &template)?;
    let contents = read_guidance_target(&target)?;
    let state = guidance_state(contents.as_deref(), &template);
    Ok(guidance_operation(
        &template,
        recognized_guidance_contents(contents.as_deref(), &template).unwrap_or(template.contents),
        "inspect",
        &target,
        false,
        state,
    ))
}

fn validate_guidance(client: &str, root: &Path) -> Result<GuidanceOperation, EngineError> {
    let template = guidance_template(client)?;
    let target = guidance_target(root, &template)?;
    if guidance_state(read_guidance_target(&target)?.as_deref(), &template) != "owned" {
        return Err(guidance_error(
            "native guidance target is not the exact owned artifact",
        ));
    }
    Ok(guidance_operation(
        &template,
        template.contents,
        "validate",
        &target,
        false,
        "owned",
    ))
}

fn install_guidance(
    client: &str,
    root: &Path,
    apply: bool,
) -> Result<GuidanceOperation, EngineError> {
    let template = guidance_template(client)?;
    let target = guidance_target(root, &template)?;
    if read_guidance_target(&target)?.is_some() {
        return Err(guidance_error(
            "native guidance target already exists and will not be overwritten",
        ));
    }
    if !apply {
        return Ok(guidance_operation(
            &template,
            template.contents,
            "install",
            &target,
            false,
            "preview_ready",
        ));
    }
    atomic_write_guidance_target(&target, template.contents.as_bytes())?;
    Ok(guidance_operation(
        &template,
        template.contents,
        "install",
        &target,
        true,
        "owned",
    ))
}

fn remove_guidance(
    client: &str,
    root: &Path,
    apply: bool,
) -> Result<GuidanceOperation, EngineError> {
    let template = guidance_template(client)?;
    let target = guidance_target(root, &template)?;
    let contents = read_guidance_target(&target)?;
    let Some(recognized_contents) = recognized_guidance_contents(contents.as_deref(), &template)
    else {
        return Err(guidance_error(
            "native guidance removal requires the exact owned artifact",
        ));
    };
    if !apply {
        return Ok(guidance_operation(
            &template,
            recognized_contents,
            "remove",
            &target,
            false,
            "preview_ready",
        ));
    }
    remove_guidance_target(&target)?;
    Ok(guidance_operation(
        &template,
        recognized_contents,
        "remove",
        &target,
        true,
        "removed",
    ))
}

fn atomic_write_guidance_target(path: &Path, contents: &[u8]) -> Result<(), EngineError> {
    if contents.len() > usize::try_from(GUIDANCE_MAX_BYTES).unwrap_or(usize::MAX) {
        return Err(guidance_error(
            "native guidance artifact would exceed its size limit",
        ));
    }
    atomic_write_managed_config(path, contents)
        .map_err(|_| guidance_error("native guidance artifact could not be written atomically"))
}

fn remove_guidance_target(path: &Path) -> Result<(), EngineError> {
    remove_managed_config(path)
        .map_err(|_| guidance_error("native guidance artifact could not be removed"))
}

fn guidance_error(message: &str) -> EngineError {
    synthetic_error(
        Capability::WorkspaceOpen,
        PublicErrorCode::InvalidInput,
        message,
    )
}

fn managed_document_is_empty(format: &str, contents: &str) -> Result<bool, EngineError> {
    match format {
        "toml" => Ok(contents.trim().is_empty()),
        "json" | "vscode-json" => {
            let value: serde_json::Value = serde_json::from_str(contents)
                .map_err(|_| managed_config_error("managed JSON removal output is malformed"))?;
            Ok(value == json!({json_server_container(format): {}}))
        }
        _ => Err(managed_config_error(
            "unsupported managed configuration format",
        )),
    }
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
        "json" | "vscode-json" => json_managed_entry_state(format, text, binary, arguments),
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
        "json" | "vscode-json" => install_json_managed_entry(format, current, binary, arguments),
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
        "json" | "vscode-json" => remove_json_managed_entry(format, current, binary, arguments),
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
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "managed connection binary must be executable",
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

fn remove_managed_config(path: &Path) -> Result<(), EngineError> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|_| managed_config_error("managed configuration disappeared before removal"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(managed_config_error(
            "managed configuration is no longer a regular non-symlink file",
        ));
    }
    fs::remove_file(path)
        .map_err(|_| managed_config_error("managed configuration could not be removed"))
}

fn managed_toml_block(binary: &Path, arguments: &[String]) -> String {
    format!(
        "# Impresari Context managed connection kit v1; ownership=exact_fixed_entry:impresari-context\n[mcp_servers.\"impresari-context\"]\ncommand = {}\nargs = [{}]\nenabled = true\nrequired = true\ndefault_tools_approval_mode = \"prompt\"",
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

fn json_server_container(format: &str) -> &'static str {
    match format {
        "json" => "mcpServers",
        "vscode-json" => "servers",
        _ => unreachable!("only JSON managed-connection formats use a server container"),
    }
}

fn json_managed_entry(format: &str, binary: &Path, arguments: &[String]) -> serde_json::Value {
    if format == "vscode-json" {
        json!({"type": "stdio", "command": binary, "args": arguments})
    } else {
        json!({"command": binary, "args": arguments})
    }
}

fn json_managed_entry_state(
    format: &str,
    text: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<ManagedEntryState, EngineError> {
    let container = json_server_container(format);
    let root = json_root_object(text)?;
    if format == "vscode-json" && vscode_json_has_disallowed_globals(&root) {
        return Err(managed_config_error(
            "managed VS Code configuration has disallowed global input or sandbox settings",
        ));
    }
    let Some(servers) = root.members.iter().find(|member| member.key == container) else {
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
    Ok(if actual == json_managed_entry(format, binary, arguments) {
        ManagedEntryState::Owned
    } else {
        ManagedEntryState::Conflicting
    })
}

fn install_json_managed_entry(
    format: &str,
    current: Option<&str>,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    let container = json_server_container(format);
    let entry = serde_json::to_string(&json_managed_entry(format, binary, arguments))
        .map_err(|_| managed_config_error("managed JSON template could not be serialized"))?;
    let Some(text) = current else {
        return Ok(format!(
            "{{\n  \"{container}\": {{\n    \"impresari-context\": {entry}\n  }}\n}}\n"
        ));
    };
    let root = json_root_object(text)?;
    if format == "vscode-json" && vscode_json_has_disallowed_globals(&root) {
        return Err(managed_config_error(
            "managed VS Code configuration has disallowed global input or sandbox settings",
        ));
    }
    if let Some(servers) = root.members.iter().find(|member| member.key == container) {
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
        Ok(insert_json_member(text, &root, container, &value))
    }
}

fn vscode_json_has_disallowed_globals(root: &JsonObject) -> bool {
    root.members
        .iter()
        .any(|member| matches!(member.key.as_str(), "inputs" | "sandbox"))
}

fn remove_json_managed_entry(
    format: &str,
    current: &str,
    binary: &Path,
    arguments: &[String],
) -> Result<String, EngineError> {
    if json_managed_entry_state(format, current, binary, arguments)? != ManagedEntryState::Owned {
        return Err(managed_config_error(
            "managed JSON configuration does not contain the exact owned entry",
        ));
    }
    let container = json_server_container(format);
    let root = json_root_object(current)?;
    let servers = root
        .members
        .iter()
        .find(|member| member.key == container)
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
    let config = config(Path::new(cache), &options.cutoff)?;
    let context = contexts.next("workspace_open");
    if let Some(policy_root) = options.budget_policy_root.as_deref() {
        LocalEngine::open_with_budget_policy_store(config, &context, Path::new(root), policy_root)
    } else {
        LocalEngine::open(config, &context, Path::new(root))
    }
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
        budget_policy_root: None,
        command: Vec::new(),
    };
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--human" => options.human = true,
            "--apply" => options.apply = true,
            "--at" | "--cutoff" | "--id-seed" | "--budget-policy-root" => {
                let flag = arguments[index].as_str();
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                match flag {
                    "--at" => options.at.clone_from(value),
                    "--cutoff" => options.cutoff.clone_from(value),
                    "--id-seed" => options.id_seed.clone_from(value),
                    "--budget-policy-root" => {
                        options.budget_policy_root = Some(PathBuf::from(value));
                    }
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

fn optional_expected(value: &str) -> Option<&str> {
    (value != "absent").then_some(value)
}

fn dashboard_cli_error(error: DashboardError) -> EngineError {
    let (code, message) = match error.code() {
        DashboardErrorCode::InvalidInput => (
            PublicErrorCode::InvalidInput,
            "invalid local budget policy input",
        ),
        DashboardErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "local budget policy resource limit exceeded",
        ),
        DashboardErrorCode::IntegrityFailure => (
            PublicErrorCode::IntegrityFailure,
            "local budget policy identity verification failed",
        ),
        DashboardErrorCode::IncompatibleData => (
            PublicErrorCode::IncompatibleCache,
            "local budget policy state is incompatible",
        ),
        DashboardErrorCode::StaleState => (
            PublicErrorCode::StaleState,
            "local budget policy state changed",
        ),
        DashboardErrorCode::StorageFailure => (
            PublicErrorCode::InternalFailure,
            "local budget policy storage failed",
        ),
    };
    synthetic_error(Capability::ContextBuild, code, message)
}

fn dashboard_server_cli_error(error: DashboardServerError) -> EngineError {
    let (code, message) = match error.code() {
        DashboardServerErrorCode::InvalidConfiguration | DashboardServerErrorCode::Protocol => (
            PublicErrorCode::InvalidInput,
            "invalid local dashboard server request",
        ),
        DashboardServerErrorCode::BindFailure => (
            PublicErrorCode::InternalFailure,
            "local dashboard loopback bind failed",
        ),
        DashboardServerErrorCode::AuditUnavailable => (
            PublicErrorCode::IncompatibleCache,
            "local dashboard audit metadata is unavailable",
        ),
        DashboardServerErrorCode::PolicyUnavailable => (
            PublicErrorCode::StaleState,
            "local dashboard policy state is unavailable",
        ),
        DashboardServerErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "local dashboard resource limit exceeded",
        ),
        DashboardServerErrorCode::InternalFailure => (
            PublicErrorCode::InternalFailure,
            "local dashboard operation failed",
        ),
    };
    synthetic_error(Capability::ContextBuild, code, message)
}

fn codex_delivery_error(error: CodexDeliveryError) -> EngineError {
    match error {
        CodexDeliveryError::Adapter(AdapterError::Engine(error)) => error,
        CodexDeliveryError::Adapter(AdapterError::IncompatibleContract)
        | CodexDeliveryError::InvalidConfiguration
        | CodexDeliveryError::InvalidPreview => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid Codex delivery configuration",
        ),
        CodexDeliveryError::Adapter(AdapterError::Serialization)
        | CodexDeliveryError::Serialization => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InternalFailure,
            "Codex delivery packet serialization failed",
        ),
    }
}

fn copilot_delivery_error(error: CopilotDeliveryError) -> EngineError {
    match error {
        CopilotDeliveryError::Adapter(AdapterError::Engine(error)) => error,
        CopilotDeliveryError::Adapter(AdapterError::IncompatibleContract)
        | CopilotDeliveryError::InvalidConfiguration
        | CopilotDeliveryError::InvalidPreview => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid Copilot delivery configuration",
        ),
        CopilotDeliveryError::Adapter(AdapterError::Serialization)
        | CopilotDeliveryError::Serialization => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InternalFailure,
            "Copilot delivery packet serialization failed",
        ),
    }
}

fn claude_delivery_error(error: ClaudeDeliveryError) -> EngineError {
    match error {
        ClaudeDeliveryError::Adapter(AdapterError::Engine(error)) => error,
        ClaudeDeliveryError::Adapter(AdapterError::IncompatibleContract)
        | ClaudeDeliveryError::InvalidConfiguration
        | ClaudeDeliveryError::InvalidPreview => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid Claude delivery configuration",
        ),
        ClaudeDeliveryError::Adapter(AdapterError::Serialization)
        | ClaudeDeliveryError::Serialization => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InternalFailure,
            "Claude delivery packet serialization failed",
        ),
    }
}

fn cursor_delivery_error(error: CursorDeliveryError) -> EngineError {
    match error {
        CursorDeliveryError::Adapter(AdapterError::Engine(error)) => error,
        CursorDeliveryError::Adapter(AdapterError::IncompatibleContract)
        | CursorDeliveryError::InvalidConfiguration
        | CursorDeliveryError::InvalidPreview => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid Cursor delivery configuration",
        ),
        CursorDeliveryError::Adapter(AdapterError::Serialization)
        | CursorDeliveryError::Serialization => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InternalFailure,
            "Cursor delivery packet serialization failed",
        ),
    }
}

fn vscode_delivery_error(error: VscodeDeliveryError) -> EngineError {
    match error {
        VscodeDeliveryError::Adapter(AdapterError::Engine(error)) => error,
        VscodeDeliveryError::Adapter(AdapterError::IncompatibleContract)
        | VscodeDeliveryError::InvalidConfiguration
        | VscodeDeliveryError::InvalidPreview
        | VscodeDeliveryError::InvalidConfirmation => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InvalidInput,
            "invalid VS Code Copilot delivery configuration",
        ),
        VscodeDeliveryError::Adapter(AdapterError::Serialization)
        | VscodeDeliveryError::Serialization => synthetic_error(
            Capability::ContextBuild,
            PublicErrorCode::InternalFailure,
            "VS Code Copilot delivery packet serialization failed",
        ),
    }
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

    #[cfg(unix)]
    fn mark_executable(path: &Path) {
        let mut permissions = fs::metadata(path).expect("binary metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(path, permissions).expect("binary permissions");
    }

    #[cfg(not(unix))]
    fn mark_executable(_path: &Path) {}

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
    fn quickstart_previews_then_installs_only_the_explicit_owned_entry() {
        let workspace = TestRoot::new("quickstart-workspace");
        let cache = TestRoot::new("quickstart-cache");
        let binaries = TestRoot::new("quickstart-binaries");
        let binary = binaries.0.join("impresari-context-mcp");
        fs::write(&binary, b"fixture binary").expect("binary fixture");
        mark_executable(&binary);
        let source = workspace.0.join("source.ts");
        fs::write(&source, b"export const stable = true;\n").expect("source fixture");
        let source_before = fs::read(&source).expect("source before");
        let config_parent = workspace.0.join(".cursor");
        fs::create_dir(&config_parent).expect("config parent");
        let target = config_parent.join("mcp.json");

        let preview =
            quickstart_with_binary("cursor", &binary, &workspace.0, &cache.0, &target, false)
                .expect("quickstart preview");
        assert_eq!(preview.schema_name, "quickstart-receipt");
        assert_eq!(preview.state, "preview_ready");
        assert!(!preview.external_write_performed);
        assert!(!target.exists());

        let applied =
            quickstart_with_binary("cursor", &binary, &workspace.0, &cache.0, &target, true)
                .expect("quickstart apply");
        assert_eq!(applied.state, "connection_installed");
        assert!(applied.external_write_performed);
        let config = fs::read_to_string(&target).expect("installed configuration");
        assert!(config.contains("impresari-context"));
        assert!(config.contains("--workspace"));
        assert_eq!(fs::read(&source).expect("source after"), source_before);
    }

    #[test]
    fn managed_connection_kit_render_is_fixed_and_source_free_for_every_client() {
        let root = TestRoot::new("managed-kit-workspace");
        let cache = TestRoot::new("managed-kit-cache");
        let binary = TestRoot::new("managed-kit-binary");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        mark_executable(&binary_path);
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
            if client == "codex" {
                assert_eq!(value["target_scope"], "user");
                assert!(
                    value["configuration"]["entry"]
                        .as_str()
                        .is_some_and(|entry| entry.contains("enabled = true"))
                );
                assert!(
                    value["configuration"]["entry"]
                        .as_str()
                        .is_some_and(|entry| entry.contains("required = true"))
                );
            } else if client == "vscode" {
                assert_eq!(value["target_scope"], "workspace_extension_host");
                assert_eq!(
                    value["configuration"]["entry"]["servers"]["impresari-context"]["type"],
                    "stdio"
                );
            }
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
    fn codex_delivery_preview_and_apply_preview_never_start_a_client_process() {
        let root = TestRoot::new("codex-delivery-workspace");
        let cache = TestRoot::new("codex-delivery-cache");
        let input = TestRoot::new("codex-delivery-input");
        let source = root.0.join("authentication.rs");
        fs::write(&source, b"pub fn authenticate() {}\n").expect("source fixture");
        let source_before = fs::read(&source).expect("source before");
        let root_arg = root.0.display().to_string();
        let cache_arg = cache.0.display().to_string();
        let (code, snapshot) = invoke(
            &[
                "snapshot".into(),
                "build".into(),
                root_arg.clone(),
                cache_arg.clone(),
            ],
            "codexsnapshot",
        );
        assert_eq!(code, 0);
        let intent_path = input.0.join("intent.json");
        let intent = serde_json::json!({
            "adapter_contract_version": context_adapters::GUIDED_DELIVERY_CONTRACT_VERSION,
            "client": context_adapters::CODEX_APP_SERVER_CLIENT,
            "scope": context_adapters::CODEX_APP_SERVER_SCOPE,
            "client_version": context_adapters::CODEX_APP_SERVER_VERSION,
            "lifecycle_point": context_adapters::CODEX_APP_SERVER_LIFECYCLE_POINT,
            "consent": true,
            "request_id": "req_codexdelivery01",
            "event_id": "evt_codexdelivery01",
            "consumer_id": "consumer_codexdelivery",
            "role": "local_user",
            "purpose": "implementation",
            "occurred_at": "2026-08-21T12:00:00Z",
            "workspace_identity": snapshot["workspace_identity"],
            "workspace_snapshot": snapshot["snapshot_id"],
            "task_profile": "implementation",
            "query": "authenticate",
            "budget": ResourceBudget::conservative(8192, 16, 128, 1024, 64, 8, 30_000, 8_388_608).expect("budget")
        });
        fs::write(
            &intent_path,
            serde_json::to_vec(&intent).expect("intent JSON"),
        )
        .expect("intent fixture");
        let intent_arg = intent_path.display().to_string();
        let (code, preview) = invoke(
            &[
                "client".into(),
                "delivery".into(),
                "codex".into(),
                "preview".into(),
                root_arg.clone(),
                cache_arg.clone(),
                intent_arg.clone(),
            ],
            "codexpreview",
        );
        assert_eq!(code, 0, "{preview}");
        assert_eq!(preview["state"], "prepared");
        let expected_packet_id = preview["value"]["delivery_envelope"]["packet_id"]
            .as_str()
            .expect("packet identity")
            .to_owned();
        let preview_path = input.0.join("delivery-preview.json");
        fs::write(
            &preview_path,
            serde_json::to_vec(&preview).expect("delivery preview JSON"),
        )
        .expect("delivery preview fixture");
        let nonexistent_binary = input.0.join("not-a-codex-binary");
        let (code, apply_preview) = invoke(
            &[
                "client".into(),
                "delivery".into(),
                "codex".into(),
                "apply".into(),
                preview_path.display().to_string(),
                input.0.display().to_string(),
                nonexistent_binary.display().to_string(),
                input
                    .0
                    .join("authenticated-codex-home")
                    .display()
                    .to_string(),
                expected_packet_id,
            ],
            "codexapply",
        );
        assert_eq!(code, 0, "{apply_preview}");
        assert_eq!(apply_preview["state"], "apply_required");
        assert_eq!(apply_preview["client_io_performed"], false);
        assert_eq!(fs::read(&source).expect("source after"), source_before);
    }

    #[test]
    fn managed_connection_lifecycle_is_explicit_owned_and_preserves_unrelated_configuration() {
        let root = TestRoot::new("managed-lifecycle-workspace");
        let cache = TestRoot::new("managed-lifecycle-cache");
        let binary = TestRoot::new("managed-lifecycle-binary");
        let config_root = TestRoot::new("managed-lifecycle-config");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        mark_executable(&binary_path);
        fs::write(root.0.join("source.ts"), b"export const stable = true;\n")
            .expect("source fixture");
        let source_before = fs::read(root.0.join("source.ts")).expect("source before");

        for client in ["codex", "claude", "cursor", "copilot", "vscode"] {
            let target = config_root.0.join(format!("{client}.config"));
            let original = if client == "codex" {
                "[other]\nname = \"stable\"\n".to_owned()
            } else {
                let container = managed_json_container_for_test(client);
                format!(
                    "{{\n  \"{container}\": {{\"other\": {{\"command\": \"other\"}}}},\n  \"unrelated\": true\n}}\n"
                )
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
            assert_eq!(preview["planned_effect"], "add_exact_owned_entry");
            assert!(
                preview["owned_entry"]
                    .to_string()
                    .contains("impresari-context")
            );
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
                let container = managed_json_container_for_test(client);
                assert_eq!(after[container]["other"]["command"], "other");
            }
        }
        assert_eq!(
            fs::read(root.0.join("source.ts")).expect("source after"),
            source_before
        );
    }

    #[test]
    fn managed_connection_removal_restores_an_absent_target() {
        let root = TestRoot::new("managed-empty-workspace");
        let cache = TestRoot::new("managed-empty-cache");
        let binary = TestRoot::new("managed-empty-binary");
        let config_root = TestRoot::new("managed-empty-config");
        let binary_path = binary.0.join("impresari-context-mcp");
        fs::write(&binary_path, b"fixture binary").expect("binary fixture");
        mark_executable(&binary_path);
        for client in ["codex", "claude", "vscode"] {
            let target = config_root.0.join(format!("{client}.config"));
            let mut command = vec![
                "client".into(),
                "kit".into(),
                "install".into(),
                client.into(),
                binary_path.display().to_string(),
                root.0.display().to_string(),
                cache.0.display().to_string(),
                target.display().to_string(),
                "--apply".into(),
            ];
            assert_eq!(invoke(&command, "managedempty").0, 0, "{client} install");
            command[2] = "remove".into();
            assert_eq!(invoke(&command, "managedempty").0, 0, "{client} remove");
            assert!(!target.exists(), "{client} empty target was not removed");
        }
    }

    #[test]
    fn native_guidance_lifecycle_is_explicit_exact_and_source_free_for_every_client() {
        let clients = ["codex", "claude", "cursor", "copilot"];
        for client in clients {
            let root = TestRoot::new(&format!("guidance-{client}"));
            let source = root.0.join("source.ts");
            fs::write(&source, b"export const untouched = true;\n").expect("source fixture");
            let source_before = fs::read(&source).expect("source before");
            let template = guidance_template(client).expect("template");
            let target = root.0.join(template.relative_target);
            fs::create_dir_all(target.parent().expect("target parent")).expect("target parent");

            let render = vec![
                "client".into(),
                "guidance".into(),
                "render".into(),
                client.into(),
            ];
            let (code, rendered) = invoke(&render, "guidancerender");
            assert_eq!(code, 0, "{client} render");
            assert_eq!(rendered["level"], "l2");
            assert_eq!(rendered["relative_target"], template.relative_target);
            assert_eq!(rendered["artifact"], template.contents);

            let install = vec![
                "client".into(),
                "guidance".into(),
                "install".into(),
                client.into(),
                root.0.display().to_string(),
            ];
            let (code, preview) = invoke(&install, "guidancelife");
            assert_eq!(code, 0, "{client} install preview");
            assert_eq!(preview["external_write_performed"], false);
            assert!(!target.exists(), "{client} preview wrote target");

            let mut apply = install.clone();
            apply.push("--apply".into());
            let (code, applied) = invoke(&apply, "guidancelife");
            assert_eq!(code, 0, "{client} install apply");
            assert_eq!(applied["state"], "owned");
            assert_eq!(
                fs::read_to_string(&target).expect("artifact"),
                template.contents
            );
            assert_eq!(
                fs::read(&source).expect("source after install"),
                source_before
            );

            let inspect = vec![
                "client".into(),
                "guidance".into(),
                "inspect".into(),
                client.into(),
                root.0.display().to_string(),
            ];
            assert_eq!(invoke(&inspect, "guidancelife").1["state"], "owned");
            let validate = vec![
                "client".into(),
                "guidance".into(),
                "validate".into(),
                client.into(),
                root.0.display().to_string(),
            ];
            assert_eq!(invoke(&validate, "guidancelife").0, 0, "{client} validate");

            let remove = vec![
                "client".into(),
                "guidance".into(),
                "remove".into(),
                client.into(),
                root.0.display().to_string(),
            ];
            let (code, preview) = invoke(&remove, "guidancelife");
            assert_eq!(code, 0, "{client} remove preview");
            assert_eq!(preview["external_write_performed"], false);
            assert!(target.exists(), "{client} preview removed target");
            let mut apply = remove;
            apply.push("--apply".into());
            let (code, removed) = invoke(&apply, "guidancelife");
            assert_eq!(code, 0, "{client} remove apply");
            assert_eq!(removed["state"], "removed");
            assert!(!target.exists(), "{client} target not removed");
            assert_eq!(
                fs::read(&source).expect("source after remove"),
                source_before
            );
        }
    }

    #[test]
    fn native_guidance_exactly_removes_a_recognized_v1_artifact() {
        for client in ["codex", "claude", "cursor", "copilot"] {
            let root = TestRoot::new(&format!("guidance-legacy-{client}"));
            let template = guidance_template(client).expect("template");
            let target = root.0.join(template.relative_target);
            fs::create_dir_all(target.parent().expect("target parent")).expect("target parent");
            fs::write(
                &target,
                template.legacy_contents.first().expect("v1 artifact"),
            )
            .expect("legacy artifact");

            let inspect = vec![
                "client".into(),
                "guidance".into(),
                "inspect".into(),
                client.into(),
                root.0.display().to_string(),
            ];
            let (code, inspected) = invoke(&inspect, "guidancelegacy");
            assert_eq!(code, 0, "{client} inspect");
            assert_eq!(inspected["state"], "owned_legacy");
            assert_eq!(
                inspected["content_sha256"],
                guidance_digest(template.legacy_contents[0])
            );

            let remove = vec![
                "client".into(),
                "guidance".into(),
                "remove".into(),
                client.into(),
                root.0.display().to_string(),
                "--apply".into(),
            ];
            let (code, removed) = invoke(&remove, "guidancelegacy");
            assert_eq!(code, 0, "{client} removal");
            assert_eq!(removed["state"], "removed");
            assert_eq!(
                removed["content_sha256"],
                guidance_digest(template.legacy_contents[0])
            );
            assert!(!target.exists(), "{client} legacy artifact remains");
        }
    }

    #[test]
    fn copilot_v2_guidance_is_removal_only_after_the_v3_upgrade() {
        let root = TestRoot::new("guidance-copilot-v2");
        let template = guidance_template("copilot").expect("copilot template");
        let target = root.0.join(template.relative_target);
        fs::create_dir_all(target.parent().expect("target parent")).expect("target parent");
        fs::write(&target, LEGACY_COPILOT_GUIDANCE_V2).expect("v2 artifact");

        let inspect = vec![
            "client".into(),
            "guidance".into(),
            "inspect".into(),
            "copilot".into(),
            root.0.display().to_string(),
        ];
        let (code, inspected) = invoke(&inspect, "guidancecopilotv2");
        assert_eq!(code, 0, "inspect v2 guidance");
        assert_eq!(inspected["state"], "owned_legacy");
        assert_eq!(
            inspected["content_sha256"],
            guidance_digest(LEGACY_COPILOT_GUIDANCE_V2)
        );

        let validate = vec![
            "client".into(),
            "guidance".into(),
            "validate".into(),
            "copilot".into(),
            root.0.display().to_string(),
        ];
        assert_ne!(invoke(&validate, "guidancecopilotv2").0, 0);

        let remove = vec![
            "client".into(),
            "guidance".into(),
            "remove".into(),
            "copilot".into(),
            root.0.display().to_string(),
            "--apply".into(),
        ];
        let (code, removed) = invoke(&remove, "guidancecopilotv2");
        assert_eq!(code, 0, "remove v2 guidance");
        assert_eq!(removed["state"], "removed");
        assert!(!target.exists(), "v2 artifact remains");
    }

    #[test]
    fn native_guidance_refuses_existing_missing_parent_and_oversized_targets_without_writes() {
        let root = TestRoot::new("guidance-reject");
        let source = root.0.join("source.ts");
        fs::write(&source, b"export const untouched = true;\n").expect("source fixture");
        let source_before = fs::read(&source).expect("source before");

        let codex_install = vec![
            "client".into(),
            "guidance".into(),
            "install".into(),
            "codex".into(),
            root.0.display().to_string(),
            "--apply".into(),
        ];
        fs::write(root.0.join("AGENTS.md"), b"unowned instructions\n").expect("conflict");
        assert_eq!(invoke(&codex_install, "guidancereject").0, 1);
        assert_eq!(
            fs::read(root.0.join("AGENTS.md")).expect("conflict unchanged"),
            b"unowned instructions\n"
        );

        let cursor_install = vec![
            "client".into(),
            "guidance".into(),
            "install".into(),
            "cursor".into(),
            root.0.display().to_string(),
            "--apply".into(),
        ];
        assert_eq!(invoke(&cursor_install, "guidancereject").0, 1);
        assert!(!root.0.join(".cursor").exists());

        let copilot_parent = root.0.join(".github/instructions");
        fs::create_dir_all(&copilot_parent).expect("copilot parent");
        let oversized = copilot_parent.join("impresari-context.instructions.md");
        fs::write(
            &oversized,
            vec![b'x'; usize::try_from(GUIDANCE_MAX_BYTES).unwrap() + 1],
        )
        .expect("oversized fixture");
        let copilot_inspect = vec![
            "client".into(),
            "guidance".into(),
            "inspect".into(),
            "copilot".into(),
            root.0.display().to_string(),
        ];
        assert_eq!(invoke(&copilot_inspect, "guidancereject").0, 1);
        assert_eq!(
            fs::metadata(&oversized).expect("oversized unchanged").len(),
            GUIDANCE_MAX_BYTES + 1
        );
        assert_eq!(
            fs::read(&source).expect("source after rejection"),
            source_before
        );
    }

    #[test]
    fn managed_connection_update_requires_exact_prior_contract_and_preserves_unrelated_configuration()
     {
        let root = TestRoot::new("managed-update-workspace");
        let old_cache = TestRoot::new("managed-update-old-cache");
        let new_cache = TestRoot::new("managed-update-new-cache");
        let binary = TestRoot::new("managed-update-binary");
        let config_root = TestRoot::new("managed-update-config");
        let old_binary = binary.0.join("old-mcp");
        let new_binary = binary.0.join("new-mcp");
        fs::write(&old_binary, b"old fixture").expect("old binary fixture");
        fs::write(&new_binary, b"new fixture").expect("new binary fixture");
        mark_executable(&old_binary);
        mark_executable(&new_binary);
        fs::write(root.0.join("source.ts"), b"export const stable = true;\n")
            .expect("source fixture");
        let source_before = fs::read(root.0.join("source.ts")).expect("source before");

        for client in ["codex", "claude", "cursor", "copilot", "vscode"] {
            let target = config_root.0.join(format!("{client}.config"));
            let unrelated = if client == "codex" {
                "[other]\nname = \"stable\"\n".to_owned()
            } else {
                let container = managed_json_container_for_test(client);
                format!(
                    "{{\n  \"{container}\": {{\"other\": {{\"command\": \"other\"}}}},\n  \"unrelated\": true\n}}\n"
                )
            };
            fs::write(&target, unrelated).expect("configuration fixture");
            let install = vec![
                "client".into(),
                "kit".into(),
                "install".into(),
                client.into(),
                old_binary.display().to_string(),
                root.0.display().to_string(),
                old_cache.0.display().to_string(),
                target.display().to_string(),
                "--apply".into(),
            ];
            assert_eq!(invoke(&install, "managedupdate").0, 0, "{client} install");
            let update = vec![
                "client".into(),
                "kit".into(),
                "update".into(),
                client.into(),
                old_binary.display().to_string(),
                root.0.display().to_string(),
                old_cache.0.display().to_string(),
                new_binary.display().to_string(),
                root.0.display().to_string(),
                new_cache.0.display().to_string(),
                target.display().to_string(),
            ];
            let before_preview = fs::read_to_string(&target).expect("target before preview");
            let (code, preview) = invoke(&update, "managedupdate");
            assert_eq!(code, 0, "{client} update preview");
            assert_eq!(preview["planned_effect"], "replace_exact_owned_entry");
            assert_eq!(preview["external_write_performed"], false);
            assert!(
                preview["previous_owned_entry"]
                    .to_string()
                    .contains("old-mcp")
            );
            assert_eq!(
                fs::read_to_string(&target).expect("target after preview"),
                before_preview
            );

            let mut apply = update.clone();
            apply.push("--apply".into());
            let (code, applied) = invoke(&apply, "managedupdate");
            assert_eq!(code, 0, "{client} update apply");
            assert_eq!(applied["state"], "owned");
            let validate = vec![
                "client".into(),
                "kit".into(),
                "validate".into(),
                client.into(),
                new_binary.display().to_string(),
                root.0.display().to_string(),
                new_cache.0.display().to_string(),
                target.display().to_string(),
            ];
            assert_eq!(
                invoke(&validate, "managedupdate").0,
                0,
                "{client} new contract validates"
            );
            assert_ne!(
                invoke(&update, "managedupdate").0,
                0,
                "{client} stale prior is rejected"
            );
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
        mark_executable(&binary_path);
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

        let vscode_target = config_root.0.join("vscode.json");
        let vscode_command = vec![
            "client".into(),
            "kit".into(),
            "install".into(),
            "vscode".into(),
            binary_path.display().to_string(),
            root.0.display().to_string(),
            cache.0.display().to_string(),
            vscode_target.display().to_string(),
            "--apply".into(),
        ];
        let unsafe_vscode_config = "{\"sandbox\": {}, \"servers\": {}}";
        fs::write(&vscode_target, unsafe_vscode_config).expect("unsafe VS Code config");
        assert_eq!(invoke(&vscode_command, "managedreject").0, 1);
        assert_eq!(
            fs::read_to_string(&vscode_target).expect("unsafe VS Code config preserved"),
            unsafe_vscode_config
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

    fn managed_json_container_for_test(client: &str) -> &'static str {
        if client == "vscode" {
            "servers"
        } else {
            "mcpServers"
        }
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
        assert_eq!(code, 0, "{result}");
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
    fn cli_declared_change_set_build_uses_the_shared_verified_contract() {
        let source = TestRoot::new("declared-change-set-cli-source");
        let cache = TestRoot::new("declared-change-set-cli-cache");
        let declaration = TestRoot::new("declared-change-set-cli-input");
        fs::write(source.0.join("review.rs"), b"pub fn review() {}\n").expect("source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-14T12:00:00Z", 10_000, 10_485_760)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &direct_context(1, "workspace_open"), &source.0)
                .expect("open");
        engine
            .build_snapshot(&direct_context(2, "snapshot_build"), default_budget())
            .expect("snapshot");
        let snapshot = engine
            .snapshot_status(&direct_context(3, "snapshot_status"), default_budget())
            .expect("snapshot status");
        let evidence = engine
            .search(
                &direct_context(4, "search"),
                QueryKind::ExactPath,
                "review.rs",
                &default_budget(),
            )
            .expect("exact evidence");
        let artifact = &evidence.matches[0].artifact;
        let manifest = json!({
            "schema_name":"declared-change-set", "schema_version":"1.0.0",
            "workspace_snapshot":snapshot.snapshot_id,
            "entries":[{"path":{
                "platform_family":artifact.path.platform_family,
                "unit_encoding":artifact.path.unit_encoding,
                "relative_units_base64url":artifact.path.relative_units_base64url
            }, "content_hash":artifact.content_hash}]
        });
        let manifest_path = declaration.0.join("declared-change-set.json");
        fs::write(
            &manifest_path,
            serde_json::to_vec(&manifest).expect("manifest JSON"),
        )
        .expect("manifest");
        drop(engine);
        let (code, result) = invoke(
            &[
                "context".into(),
                "profile-change-set-build".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                "review".into(),
                manifest_path.to_string_lossy().into_owned(),
            ],
            "declaredchangesetcli",
        );
        assert_eq!(code, 0, "{result}");
        assert_eq!(result["plan"]["task_profile"], "change_review");
        assert_eq!(
            result["plan"]["declared_change_set"]["workspace_snapshot"],
            manifest["workspace_snapshot"]
        );
        assert_eq!(
            result["plan"]["coverage"]
                .as_array()
                .expect("coverage")
                .iter()
                .find(|coverage| coverage["evidence_class"] == "change_set")
                .expect("change-set coverage")["status"],
            "available"
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
    fn dashboard_startup_failure_is_source_free_and_does_not_create_roots() {
        let root = TestRoot::new("dashboard-startup");
        let audit = root.0.join("missing-audit");
        let policy = root.0.join("missing-policy");
        let (code, envelope) = invoke(
            &[
                "dashboard".into(),
                "serve".into(),
                audit.to_string_lossy().into_owned(),
                policy.to_string_lossy().into_owned(),
            ],
            "dasherror",
        );
        assert_eq!(code, 1);
        assert_eq!(envelope["code"], "incompatible_cache");
        let encoded = serde_json::to_string(&envelope).expect("error JSON");
        assert!(!encoded.contains(root.0.to_string_lossy().as_ref()));
        assert!(!audit.exists());
        assert!(!policy.exists());
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
                ".c", ".cc", ".cjs", ".clj", ".cljc", ".cljs", ".cpp", ".cs", ".cxx", ".ex",
                ".exs", ".go", ".h", ".hh", ".hpp", ".hs", ".hxx", ".java", ".js", ".json",
                ".jsonc", ".jsx", ".kt", ".kts", ".lhs", ".mjs", ".php", ".py", ".rb", ".rs",
                ".scala", ".swift", ".toml", ".ts", ".tsx", ".yaml", ".yml",
            ]),
            "the public manifest must match the shipped structural worker inventory"
        );
        assert_eq!(
            manifest["first_class_clients"],
            serde_json::json!([
                "Codex",
                "Claude Code",
                "Cursor",
                "GitHub Copilot CLI",
                "VS Code Copilot"
            ])
        );
        let first_class = manifest["client_support"]
            .as_array()
            .expect("client support array")
            .iter()
            .filter(|entry| entry["first_class"] == true)
            .collect::<Vec<_>>();
        assert_eq!(first_class.len(), 5, "client promotion must be explicit");
        let first_class_contracts = first_class
            .iter()
            .map(|entry| {
                (
                    entry["client"].as_str().expect("first-class client"),
                    entry["classification"].as_str().expect("classification"),
                    entry["conformance"].as_str().expect("conformance"),
                )
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            first_class_contracts,
            std::collections::BTreeSet::from([
                (
                    "Claude Code",
                    "first_class",
                    "managed_local_scope_configuration_and_bounded_model_directed_lifecycle",
                ),
                (
                    "Codex",
                    "first_class",
                    "managed_user_home_configuration_and_app_server_direct_tool_conformance",
                ),
                (
                    "Cursor",
                    "first_class",
                    "managed_project_configuration_and_guarded_agent_mode_packet_equivalence",
                ),
                (
                    "GitHub Copilot CLI",
                    "first_class",
                    "managed_trusted_project_configuration_and_bounded_model_directed_packet_equivalence",
                ),
                (
                    "VS Code Copilot",
                    "first_class",
                    "managed_extension_host_workspace_configuration_bounded_session_tool_lifecycle_and_operator_confirmed_guided_delivery",
                ),
            ])
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
                "type": "stdio", "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
            }}
        });
        assert!(client_config_is_safe(&vscode_safe_shape, "vscode"));

        let vscode_missing_type = serde_json::json!({
            "servers": { "impresari-context": {
                "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
            }}
        });
        assert!(!client_config_is_safe(&vscode_missing_type, "vscode"));

        let vscode_wrong_type = serde_json::json!({
            "servers": { "impresari-context": {
                "type": "http", "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
            }}
        });
        assert!(!client_config_is_safe(&vscode_wrong_type, "vscode"));

        let vscode_sandboxed = serde_json::json!({
            "servers": { "impresari-context": {
                "type": "stdio", "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"],
                "sandboxEnabled": true
            }}
        });
        assert!(!client_config_is_safe(&vscode_sandboxed, "vscode"));

        let vscode_global_sandbox = serde_json::json!({
            "servers": { "impresari-context": {
                "type": "stdio", "command": binary.as_str(),
                "args": ["--workspace", "${workspaceFolder}", "--cache", "${env:IMPRESARI_CONTEXT_CACHE}", "--consumer-id", "consumer_vscode_local", "--role", "local_user"]
            }},
            "sandbox": {"network": {"allowedDomains": ["example.invalid"]}}
        });
        assert!(!client_config_is_safe(&vscode_global_sandbox, "vscode"));
    }

    #[test]
    fn doctor_codex_config_validates_a_user_scoped_fixed_stdio_entry() {
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

    #[test]
    fn budget_policy_cli_is_preview_first_exact_and_reversible() {
        let state = TestRoot::new("budget-policy-state");
        let input = TestRoot::new("budget-policy-input");
        let store = state.0.join("policy");
        let policy = context_dashboard::compile_policy(context_dashboard::LocalBudgetPolicyDraft {
            schema_name: "local-budget-policy".into(),
            schema_version: "1.0.0".into(),
            revision: "1".into(),
            created_at: "2026-08-21T00:00:00Z".into(),
            expires_at: None,
            rules: vec![context_dashboard::LocalBudgetRule {
                rule_id: "cli_limit".into(),
                selector: context_dashboard::BudgetSelector {
                    purpose: Some(context_dashboard::DashboardPurpose::Implementation),
                    capability: Some(Capability::ContextBuild),
                },
                deny: false,
                ceilings: context_dashboard::BudgetCeilings {
                    requested: Some("8192".into()),
                    ..context_dashboard::BudgetCeilings::default()
                },
            }],
        })
        .expect("policy");
        let policy_path = input.0.join("policy.json");
        fs::write(
            &policy_path,
            serde_json::to_vec(&policy).expect("policy JSON"),
        )
        .expect("policy input");
        let base = vec![
            "budget".into(),
            "policy".into(),
            "apply".into(),
            store.to_string_lossy().into_owned(),
            policy_path.to_string_lossy().into_owned(),
            "absent".into(),
            "absent".into(),
        ];
        let (code, preview) = invoke(&base, "budgetpre");
        assert_eq!(code, 0);
        assert_eq!(preview["state"], "preview");
        assert_eq!(preview["external_write_performed"], false);
        assert!(!store.exists());
        let mut apply = base;
        apply.push("--apply".into());
        let (code, applied) = invoke(&apply, "budgetapp");
        assert_eq!(code, 0, "apply response: {applied}");
        assert_eq!(applied["state"], "applied");
        assert_eq!(applied["after"]["current_policy_id"], policy.policy_id);

        let (code, inspected) = invoke(
            &[
                "budget".into(),
                "policy".into(),
                "inspect".into(),
                store.to_string_lossy().into_owned(),
            ],
            "budgetins",
        );
        assert_eq!(code, 0);
        assert_eq!(inspected["current_policy_id"], policy.policy_id);

        let remove = vec![
            "budget".into(),
            "policy".into(),
            "remove".into(),
            store.to_string_lossy().into_owned(),
            policy.policy_id.clone(),
            policy.revision.clone(),
            "--apply".into(),
        ];
        let (code, removed) = invoke(&remove, "budgetrem");
        assert_eq!(code, 0);
        assert_eq!(removed["state"], "removed");
        let rollback = vec![
            "budget".into(),
            "policy".into(),
            "rollback".into(),
            store.to_string_lossy().into_owned(),
            "absent".into(),
            "absent".into(),
            "--apply".into(),
        ];
        let (code, restored) = invoke(&rollback, "budgetrol");
        assert_eq!(code, 0);
        assert_eq!(restored["state"], "rolled_back");
        assert_eq!(restored["after"]["current_policy_id"], policy.policy_id);

        assert_runtime_uses_budget_policy(&store);
    }

    fn assert_runtime_uses_budget_policy(store: &Path) {
        let source = TestRoot::new("budget-policy-source");
        let cache = TestRoot::new("budget-policy-cache");
        fs::write(source.0.join("lib.rs"), b"pub fn bounded() {}\n").expect("source");
        let (code, packet) = invoke(
            &[
                "--budget-policy-root".into(),
                store.to_string_lossy().into_owned(),
                "context".into(),
                "build".into(),
                source.0.to_string_lossy().into_owned(),
                cache.0.to_string_lossy().into_owned(),
                "literal".into(),
                "bounded".into(),
                "implementation".into(),
            ],
            "budgetrun",
        );
        assert_eq!(code, 0);
        assert_eq!(packet["budget"]["requested"], "8192");
    }
}
