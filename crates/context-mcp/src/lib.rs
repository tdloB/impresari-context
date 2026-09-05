// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Bounded local stdio MCP transport over the public context engine."]

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, Write},
    time::Instant,
};

use context_core::{POLICY_PROFILE, PolicySubject, ResourceBudget, json_contract_identity};
use context_engine::{
    ContextPlan, ContextPlanStep, DeclaredAssociatedTests, DeclaredChangeSet,
    DeclaredConventionExemplars, IncrementalStructuralUpdate, LocalEngine, ProfiledContextPacket,
    RepositoryOrientationRequest, RepositoryReadTelemetry, RequestContext,
    StructuralEvidenceExpansion, StructuralImpactRequest, StructuralPlannerQuery,
    StructuralSeedRequest, TaskProfile,
};
use context_session::{SessionPolicy, SessionStore};
use context_structural::{GraphEdge, StructuralGraph, WorkerLauncher};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Preferred MCP revision implemented by this transport.
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
/// Older MCP revision accepted for clients that have not yet adopted the preferred revision.
pub const MCP_COMPATIBLE_PROTOCOL_VERSION: &str = "2025-06-18";
/// Maximum encoded JSON-RPC line accepted from a client.
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;
/// Maximum request identifiers retained for replay rejection in one process.
pub const MAX_REQUESTS: usize = 10_000;
/// Public v1 request and event identifier grammar.
const IDENTIFIER_PATTERN: &str = "^[a-z][a-z0-9_-]{0,31}_[A-Za-z0-9_-]{8,128}$";
/// Public v1 canonical decimal grammar for resource-budget fields.
const DECIMAL_PATTERN: &str = "^(?:0|[1-9][0-9]*)$";

/// Trusted launch configuration. The client cannot change these values via MCP.
pub struct ServerConfig {
    /// Fixed consumer identity.
    pub consumer_id: String,
    /// Fixed policy role.
    pub role: String,
    /// Bounded process-local session policy.
    pub session_policy: SessionPolicy,
    /// Fixed trusted delivery mode selected before MCP initialization.
    pub delivery_mode: DeliveryMode,
    /// Optional graph prepared solely from trusted process-startup authority.
    pub structural_runtime: Option<StructuralRuntime>,
}

/// Trusted startup delivery mode. Repository or tool input cannot change it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMode {
    /// Existing packet delivery with no trusted structural runtime.
    Ordinary,
    /// Existing eager packet delivery with exact structural excerpts.
    EagerStructural,
    /// Compact structural map with session-owned deferred exact evidence.
    ProgressiveStructural,
}

impl DeliveryMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::EagerStructural => "eager_structural",
            Self::ProgressiveStructural => "progressive_structural",
        }
    }
}

/// Trusted structural state prepared before MCP initialization.
#[derive(Clone, Debug)]
pub struct StructuralRuntime {
    /// Exact snapshot-bound graph retained only for this process.
    ///
    /// Thin but complete. Used when no task-scoped build is configured, and as
    /// the fallback when one is configured but nominates nothing.
    pub graph: StructuralGraph,
    /// Closed traversal kinds; empty means every admitted graph edge kind.
    pub edge_kinds: Vec<String>,
    /// Non-authoritative lifecycle receipt returned beside every packet.
    pub receipt: StructuralLifecycleReceipt,
    /// Present when the server may build a dense task-scoped graph per request.
    pub task_scoped: Option<TaskScopedStructure>,
}

/// Inputs a server needs to build a task-scoped structural graph per request.
///
/// A whole-repository graph divides one fact allowance across every file, which
/// on a large repository leaves roughly one fact each. Building per request over
/// the files a task nominates gives each of them a large share of the same
/// allowance.
#[derive(Clone, Debug)]
pub struct TaskScopedStructure {
    /// Pinned, hash-attested structural worker boundary.
    pub launcher: WorkerLauncher,
    /// Admitted structural budget for a scoped build.
    pub budget: ResourceBudget,
}

/// Closed metadata proving which structural lifecycle an MCP result used.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StructuralLifecycleReceipt {
    /// Schema discriminator.
    pub schema_name: String,
    /// Receipt contract version.
    pub schema_version: String,
    /// Whether trusted structural startup was enabled.
    pub enabled: bool,
    /// `disabled` or `prepared`.
    pub state: String,
    /// Exact prepared graph identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_id: Option<String>,
    /// Exact startup snapshot identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    /// Exact worker executable identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_sha256: Option<String>,
    /// Graph completeness reported by the validated graph.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_completeness: Option<String>,
    /// Wall time spent preparing structural state before MCP readiness.
    pub preparation_elapsed_ms: u64,
}

impl StructuralLifecycleReceipt {
    fn disabled() -> Self {
        Self {
            schema_name: "impresari_context_structural_lifecycle".into(),
            schema_version: "1.0".into(),
            enabled: false,
            state: "disabled".into(),
            graph_id: None,
            snapshot_id: None,
            worker_sha256: None,
            graph_completeness: None,
            preparation_elapsed_ms: 0,
        }
    }
}

const PROGRESSIVE_CONTRACT_VERSION: &str = "1.0.0";
const MAX_PROGRESSIVE_MAPS: u64 = 1;
const MAX_PROGRESSIVE_LOOKUPS: u64 = 64;
const MAX_PROGRESSIVE_EXPANSIONS: u64 = 64;
const MAX_PROGRESSIVE_ITEMS: u64 = 256;
const MAX_PROGRESSIVE_RESPONSE_BYTES: u64 = 4_194_304;

#[derive(Clone, Debug, Serialize)]
struct DisclosureMapItem {
    item_handle: String,
    display_path: String,
    relationship_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_label: Option<String>,
    confidence: String,
    freshness: String,
    unknowns: Vec<String>,
}

#[derive(Clone, Debug)]
struct StoredDisclosureItem {
    public: DisclosureMapItem,
    evidence_handle: String,
    edge_id: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
struct DisclosureConsumption {
    maps: u64,
    lookups: u64,
    expansions: u64,
    returned_items: u64,
    exact_source_bytes: u64,
    serialized_response_bytes: u64,
    repository_reads: u64,
    repeated_repository_reads: u64,
    elapsed_ms: u64,
}

impl DisclosureConsumption {
    fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            maps: self.maps.checked_add(other.maps)?,
            lookups: self.lookups.checked_add(other.lookups)?,
            expansions: self.expansions.checked_add(other.expansions)?,
            returned_items: self.returned_items.checked_add(other.returned_items)?,
            exact_source_bytes: self
                .exact_source_bytes
                .checked_add(other.exact_source_bytes)?,
            serialized_response_bytes: self
                .serialized_response_bytes
                .checked_add(other.serialized_response_bytes)?,
            repository_reads: self.repository_reads.checked_add(other.repository_reads)?,
            repeated_repository_reads: self
                .repeated_repository_reads
                .checked_add(other.repeated_repository_reads)?,
            elapsed_ms: self.elapsed_ms.checked_add(other.elapsed_ms)?,
        })
    }

    fn within(self, ceiling: Self) -> bool {
        self.maps <= ceiling.maps
            && self.lookups <= ceiling.lookups
            && self.expansions <= ceiling.expansions
            && self.returned_items <= ceiling.returned_items
            && self.exact_source_bytes <= ceiling.exact_source_bytes
            && self.serialized_response_bytes <= ceiling.serialized_response_bytes
            && self.repository_reads <= ceiling.repository_reads
            && self.repeated_repository_reads <= ceiling.repeated_repository_reads
            && self.elapsed_ms <= ceiling.elapsed_ms
    }

    fn remaining(self, ceiling: Self) -> Self {
        Self {
            maps: ceiling.maps.saturating_sub(self.maps),
            lookups: ceiling.lookups.saturating_sub(self.lookups),
            expansions: ceiling.expansions.saturating_sub(self.expansions),
            returned_items: ceiling.returned_items.saturating_sub(self.returned_items),
            exact_source_bytes: ceiling
                .exact_source_bytes
                .saturating_sub(self.exact_source_bytes),
            serialized_response_bytes: ceiling
                .serialized_response_bytes
                .saturating_sub(self.serialized_response_bytes),
            repository_reads: ceiling
                .repository_reads
                .saturating_sub(self.repository_reads),
            repeated_repository_reads: ceiling
                .repeated_repository_reads
                .saturating_sub(self.repeated_repository_reads),
            elapsed_ms: ceiling.elapsed_ms.saturating_sub(self.elapsed_ms),
        }
    }
}

#[derive(Debug)]
struct ProgressiveSession {
    map_id: String,
    workspace_identity: String,
    workspace_snapshot: String,
    graph_id: String,
    plan_id: String,
    policy_decision: String,
    budget: ResourceBudget,
    structural_query: Option<StructuralPlannerQuery>,
    items: Vec<StoredDisclosureItem>,
    consumed: DisclosureConsumption,
    ceiling: DisclosureConsumption,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DisclosureLookupArgs {
    session_id: String,
    handle: String,
    relation_kinds: Vec<String>,
    max_items: String,
    max_depth: String,
    max_bytes: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceExpandArgs {
    request_id: String,
    event_id: String,
    purpose: String,
    occurred_at: String,
    session_id: String,
    evidence_handle: String,
    before_bytes: String,
    after_bytes: String,
    max_bytes: String,
}

/// Stateful single-client stdio MCP service.
pub struct McpServer {
    engine: LocalEngine,
    consumer_id: String,
    role: String,
    sessions: SessionStore,
    delivery_mode: DeliveryMode,
    structural_runtime: Option<StructuralRuntime>,
    disclosures: BTreeMap<String, ProgressiveSession>,
    initialized_response_sent: bool,
    operation_ready: bool,
    request_ids: BTreeSet<String>,
}

impl McpServer {
    /// Creates a server around an already-authorized and snapshotted engine.
    #[must_use]
    pub fn new(engine: LocalEngine, config: ServerConfig) -> Self {
        let structural_runtime = if config.delivery_mode == DeliveryMode::Ordinary {
            None
        } else {
            config.structural_runtime
        };
        Self {
            engine,
            consumer_id: config.consumer_id,
            role: config.role,
            sessions: SessionStore::new(config.session_policy),
            delivery_mode: config.delivery_mode,
            structural_runtime,
            disclosures: BTreeMap::new(),
            initialized_response_sent: false,
            operation_ready: false,
            request_ids: BTreeSet::new(),
        }
    }

    /// Serves bounded newline-delimited JSON-RPC until EOF.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when stdin cannot be read or stdout cannot be written.
    pub fn serve<R: BufRead, W: Write>(
        &mut self,
        mut input: R,
        output: &mut W,
    ) -> std::io::Result<()> {
        loop {
            let Some(line) = read_bounded_line(&mut input)? else {
                return Ok(());
            };
            let response = if line.overflowed {
                Some(error(
                    Value::Null,
                    -32600,
                    "request exceeds transport limit",
                ))
            } else {
                self.handle_bytes(&line.bytes)
            };
            if let Some(response) = response {
                serde_json::to_writer(&mut *output, &response)?;
                output.write_all(b"\n")?;
                output.flush()?;
            }
        }
    }

    fn handle_bytes(&mut self, bytes: &[u8]) -> Option<Value> {
        let Ok(value) = serde_json::from_slice::<Value>(bytes) else {
            return Some(error(Value::Null, -32700, "parse error"));
        };
        if value.is_array() {
            return Some(error(Value::Null, -32600, "batching is not supported"));
        }
        let Some(object) = value.as_object() else {
            return Some(error(Value::Null, -32600, "invalid request"));
        };
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params"))
        {
            return Some(error(
                object.get("id").cloned().unwrap_or(Value::Null),
                -32600,
                "invalid request fields",
            ));
        }
        if object.get("jsonrpc") != Some(&Value::String("2.0".into())) {
            return Some(error(
                object.get("id").cloned().unwrap_or(Value::Null),
                -32600,
                "invalid request",
            ));
        }
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return Some(error(
                object.get("id").cloned().unwrap_or(Value::Null),
                -32600,
                "invalid request method",
            ));
        };
        let id = object.get("id").cloned();
        if id.is_none() {
            if method == "notifications/initialized" && self.initialized_response_sent {
                self.operation_ready = true;
            }
            return None;
        }
        let id = id.unwrap_or(Value::Null);
        if !matches!(id, Value::String(_) | Value::Number(_)) || id.is_null() {
            return Some(error(Value::Null, -32600, "invalid request id"));
        }
        if self.request_ids.len() >= MAX_REQUESTS {
            return Some(error(id, -32001, "request limit reached"));
        }
        let id_key = serde_json::to_string(&id).unwrap_or_default();
        if !self.request_ids.insert(id_key) {
            return Some(error(id, -32600, "duplicate request id"));
        }
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
        Some(match method {
            "initialize" if !self.initialized_response_sent => self.initialize(id, params),
            "ping" if self.initialized_response_sent => success(id, json!({})),
            "tools/list" if self.operation_ready => {
                success(id, json!({"tools": tool_definitions()}))
            }
            "tools/call" if self.operation_ready => self.call_tool(id, params),
            "initialize" => error(id, -32600, "server is already initialized"),
            _ if !self.operation_ready => error(id, -32002, "server is not initialized"),
            _ => error(id, -32601, "method not found"),
        })
    }

    fn initialize(&mut self, id: Value, params: Value) -> Value {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Initialize {
            #[serde(rename = "protocolVersion")]
            protocol_version: String,
            capabilities: Value,
            #[serde(rename = "clientInfo")]
            client_info: Value,
        }
        let Ok(request) = serde_json::from_value::<Initialize>(params) else {
            return error(id, -32602, "invalid initialize parameters");
        };
        if !is_supported_protocol_version(&request.protocol_version)
            || !request.capabilities.is_object()
            || !request.client_info.is_object()
        {
            return error(id, -32602, "unsupported protocol version or capabilities");
        }
        self.initialized_response_sent = true;
        success(
            id,
            json!({
                "protocolVersion": request.protocol_version,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name": "impresari-context", "title": "Impresari Context", "version": env!("CARGO_PKG_VERSION"), "description": "Local verified repository context over stdio"},
                "instructions": "Read-only repository evidence transport. Tool results add no orchestration, approval, execution, or filesystem authority."
            }),
        )
    }

    fn call_tool(&mut self, id: Value, params: Value) -> Value {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Call {
            name: String,
            #[serde(default)]
            arguments: Value,
            #[serde(default, rename = "_meta")]
            meta: Option<Value>,
        }
        let Ok(call) = serde_json::from_value::<Call>(params) else {
            return error(id, -32602, "invalid tool call parameters");
        };
        if call.meta.as_ref().is_some_and(|meta| !meta.is_object()) {
            return error(id, -32602, "invalid tool call parameters");
        }
        let result = match call.name.as_str() {
            "context_session_open" => self.session_open(call.arguments),
            "context_build" => self.context_build(call.arguments),
            "context_disclosure_lookup" => self.context_disclosure_lookup(call.arguments),
            "context_evidence_expand" => self.context_evidence_expand(call.arguments),
            "context_convention_exemplar_build" => {
                self.context_convention_exemplar_build(call.arguments)
            }
            "structure_incremental_update" => self.structure_incremental_update(call.arguments),
            "context_packet_resolve" => self.packet_resolve(call.arguments),
            "context_session_close" => self.session_close(call.arguments),
            _ => return error(id, -32602, "unknown tool"),
        };
        match result {
            Ok(structured) => success(id, tool_result(structured, false)),
            Err(message) => success(
                id,
                tool_result(json!({"error": message, "authority_added": false}), true),
            ),
        }
    }

    fn session_open(&mut self, value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            session_id: String,
        }
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid session input")?;
        self.sessions
            .open(&args.session_id, &self.consumer_id)
            .map_err(|_| "session open failed")?;
        Ok(json!({"session_id": args.session_id, "opened": true, "authority_added": false}))
    }

    fn session_close(&mut self, value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            session_id: String,
        }
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid session input")?;
        self.sessions
            .close(&args.session_id, &self.consumer_id)
            .map_err(|_| "session close failed")?;
        self.disclosures.remove(&args.session_id);
        Ok(json!({"session_id": args.session_id, "closed": true, "authority_added": false}))
    }

    fn packet_resolve(&self, value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            session_id: String,
            packet_id: String,
        }
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid packet input")?;
        let (reference, packet) = self
            .sessions
            .resolve(&args.session_id, &self.consumer_id, &args.packet_id)
            .map_err(|_| "packet resolve failed")?;
        Ok(json!({"reference": reference, "packet": packet, "authority_added": false}))
    }

    fn context_convention_exemplar_build(
        &mut self,
        mut value: Value,
    ) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            request_id: String,
            event_id: String,
            purpose: String,
            occurred_at: String,
            query: String,
            declaration: DeclaredConventionExemplars,
            budget: ResourceBudget,
        }
        normalize_wire_budget(&mut value)?;
        let args: Args =
            serde_json::from_value(value).map_err(|_| "invalid convention exemplar input")?;
        let context = RequestContext {
            request_id: args.request_id,
            event_id: args.event_id,
            subject: PolicySubject {
                caller_id: self.consumer_id.clone(),
                role: self.role.clone(),
                purpose: args.purpose,
            },
            occurred_at: args.occurred_at,
        };
        let profiled = self
            .engine
            .build_profiled_declared_convention_exemplar_context(
                &context,
                &args.query,
                &args.declaration,
                args.budget,
            )
            .map_err(|_| "convention exemplar context build failed")?;
        Ok(json!({"packet": profiled.packet, "plan": profiled.plan, "authority_added": false}))
    }

    fn structure_incremental_update(&mut self, mut value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            request_id: String,
            event_id: String,
            purpose: String,
            occurred_at: String,
            update: IncrementalStructuralUpdate,
            budget: ResourceBudget,
        }
        normalize_wire_budget(&mut value)?;
        let args: Args =
            serde_json::from_value(value).map_err(|_| "invalid incremental update input")?;
        let context = RequestContext {
            request_id: args.request_id,
            event_id: args.event_id,
            subject: PolicySubject {
                caller_id: self.consumer_id.clone(),
                role: self.role.clone(),
                purpose: args.purpose,
            },
            occurred_at: args.occurred_at,
        };
        let graph = self
            .engine
            .apply_incremental_structural_update(&context, &args.update, &args.budget)
            .map_err(|_| "incremental structural update failed")?;
        Ok(json!({"graph": graph, "authority_added": false}))
    }

    #[allow(clippy::too_many_lines)] // One request grammar is kept co-located with its exclusive dispatch.
    fn context_build(&mut self, mut value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            request_id: String,
            event_id: String,
            purpose: String,
            occurred_at: String,
            steps: Option<Vec<ContextPlanStep>>,
            profile: Option<TaskProfile>,
            query: Option<String>,
            structural_graph: Option<StructuralGraph>,
            start_node: Option<String>,
            edge_kinds: Option<Vec<String>>,
            declared_change_set: Option<DeclaredChangeSet>,
            declared_associated_tests: Option<DeclaredAssociatedTests>,
            orientation_graph: Option<StructuralGraph>,
            max_orientation_entries: Option<u32>,
            budget: ResourceBudget,
            session_id: Option<String>,
        }
        normalize_wire_budget(&mut value)?;
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid context input")?;
        if self.delivery_mode != DeliveryMode::Ordinary && self.structural_runtime.is_none() {
            return Err("structural delivery runtime unavailable");
        }
        if self.delivery_mode == DeliveryMode::ProgressiveStructural {
            let session_id = args
                .session_id
                .as_deref()
                .ok_or("progressive context build requires an open session")?;
            self.sessions
                .authorize(session_id, &self.consumer_id)
                .map_err(|_| "progressive session unavailable")?;
            if let Some(session) = self.disclosures.get(session_id) {
                return progressive_exhausted(
                    session,
                    "map",
                    DisclosureConsumption {
                        maps: 1,
                        ..DisclosureConsumption::default()
                    },
                    "map_count_exhausted",
                );
            }
            if args.steps.is_some()
                || args.profile.is_none()
                || args.query.is_none()
                || args.structural_graph.is_some()
                || args.start_node.is_some()
                || args.edge_kinds.is_some()
                || args.declared_change_set.is_some()
                || args.declared_associated_tests.is_some()
                || args.orientation_graph.is_some()
                || args.max_orientation_entries.is_some()
            {
                return Err("progressive context build requires only profile and query");
            }
        }
        let progressive_session_id = args.session_id.clone();
        let progressive_budget = args.budget.clone();
        let progressive_started = Instant::now();
        let reads_before = self.engine.repository_read_telemetry();
        let context = RequestContext {
            request_id: args.request_id,
            event_id: args.event_id,
            subject: PolicySubject {
                caller_id: self.consumer_id.clone(),
                role: self.role.clone(),
                purpose: args.purpose,
            },
            occurred_at: args.occurred_at,
        };
        let (packet, plan) = match (
            args.steps,
            args.profile,
            args.query,
            args.structural_graph,
            args.start_node,
            args.edge_kinds,
            args.declared_change_set,
            args.declared_associated_tests,
            args.orientation_graph,
            args.max_orientation_entries,
        ) {
            (Some(steps), None, None, None, None, None, None, None, None, None) => (
                self.engine
                    .build_planned_context(&context, &ContextPlan { steps }, args.budget)
                    .map_err(|_| "context build failed")?,
                None,
            ),
            (None, Some(profile), Some(query), None, None, None, None, None, None, None) => {
                let profiled = if let Some(runtime) = &self.structural_runtime {
                    // Prefer a dense graph over the files this task nominated.
                    // Fall back to the thin whole-repository graph when nothing
                    // is nominated, so a task naming no code still works.
                    let scoped = runtime.task_scoped.as_ref().and_then(|scoped| {
                        self.engine
                            .build_task_scoped_structure(
                                &context,
                                &query,
                                &scoped.budget,
                                &scoped.launcher,
                            )
                            .ok()
                            .filter(|(_, nomination)| !nomination.files.is_empty())
                            .map(|(graph, nomination)| {
                                let order = nomination
                                    .files
                                    .iter()
                                    .map(|file| file.display_path.clone())
                                    .collect::<Vec<_>>();
                                (graph, order, nomination.admitted_identifiers)
                            })
                    });
                    // Carry the nomination order so a name shared by several
                    // files resolves to the file the task is about rather than
                    // to whichever path sorts first.
                    let (graph, nominated_order, admitted_identifiers) =
                        scoped.unwrap_or_else(|| (runtime.graph.clone(), Vec::new(), Vec::new()));
                    if self.delivery_mode == DeliveryMode::ProgressiveStructural {
                        self.engine
                            .build_profiled_seeded_progressive_context(
                                &context,
                                profile,
                                &query,
                                &StructuralSeedRequest {
                                    nominated_order: nominated_order.clone(),
                                    admitted_identifiers: admitted_identifiers.clone(),
                                    graph,
                                    edge_kinds: runtime.edge_kinds.clone(),
                                },
                                args.budget,
                            )
                            .map_err(|_| "profiled progressive context build failed")?
                    } else {
                        self.engine
                            .build_profiled_seeded_structural_context(
                                &context,
                                profile,
                                &query,
                                &StructuralSeedRequest {
                                    nominated_order: nominated_order.clone(),
                                    admitted_identifiers: admitted_identifiers.clone(),
                                    graph,
                                    edge_kinds: runtime.edge_kinds.clone(),
                                },
                                args.budget,
                            )
                            .map_err(|_| "profiled structural context build failed")?
                    }
                } else {
                    self.engine
                        .build_profiled_context(&context, profile, &query, args.budget)
                        .map_err(|_| "profiled context build failed")?
                };
                if self.delivery_mode == DeliveryMode::ProgressiveStructural {
                    let session_id = progressive_session_id
                        .as_deref()
                        .ok_or("progressive context build requires an open session")?;
                    return self.progressive_context_build(
                        session_id,
                        &profiled,
                        progressive_budget,
                        &reads_before,
                        progressive_started,
                    );
                }
                (profiled.packet, Some(profiled.plan))
            }
            (
                None,
                Some(profile),
                Some(query),
                Some(graph),
                Some(start_node),
                edge_kinds,
                None,
                None,
                None,
                None,
            ) => {
                let profiled = self
                    .engine
                    .build_profiled_structural_context(
                        &context,
                        profile,
                        &query,
                        &StructuralImpactRequest {
                            graph,
                            start_node,
                            edge_kinds: edge_kinds.unwrap_or_default(),
                        },
                        args.budget,
                    )
                    .map_err(|_| "profiled structural context build failed")?;
                (profiled.packet, Some(profiled.plan))
            }
            (
                None,
                Some(TaskProfile::ChangeReview),
                Some(query),
                None,
                None,
                None,
                Some(declaration),
                None,
                None,
                None,
            ) => {
                let profiled = self
                    .engine
                    .build_profiled_declared_change_set_context(
                        &context,
                        &query,
                        &declaration,
                        args.budget,
                    )
                    .map_err(|_| "declared change-set context build failed")?;
                (profiled.packet, Some(profiled.plan))
            }
            (
                None,
                Some(TaskProfile::TestSelection),
                Some(query),
                None,
                None,
                None,
                None,
                Some(declaration),
                None,
                None,
            ) => {
                let profiled = self
                    .engine
                    .build_profiled_declared_associated_test_context(
                        &context,
                        &query,
                        &declaration,
                        args.budget,
                    )
                    .map_err(|_| "declared associated-test context build failed")?;
                (profiled.packet, Some(profiled.plan))
            }
            (
                None,
                Some(TaskProfile::Orientation),
                Some(query),
                None,
                None,
                None,
                None,
                None,
                Some(graph),
                Some(max_entries),
            ) => {
                let profiled = self
                    .engine
                    .build_profiled_repository_orientation_context(
                        &context,
                        &query,
                        &RepositoryOrientationRequest { graph, max_entries },
                        args.budget,
                    )
                    .map_err(|_| "repository orientation context build failed")?;
                (profiled.packet, Some(profiled.plan))
            }
            _ => return Err("context build requires either steps or profile and query"),
        };
        let reference = if let Some(session_id) = args.session_id {
            Some(
                self.sessions
                    .attach(&session_id, &self.consumer_id, &packet)
                    .map_err(|_| "packet attach failed")?,
            )
        } else {
            None
        };
        let read_telemetry = self.engine.repository_read_telemetry();
        let structural_lifecycle = self
            .structural_runtime
            .as_ref()
            .map_or_else(StructuralLifecycleReceipt::disabled, |runtime| {
                runtime.receipt.clone()
            });
        Ok(
            json!({"delivery_mode": self.delivery_mode.as_str(), "packet": packet, "plan": plan, "reference": reference, "read_telemetry": read_telemetry, "structural_lifecycle": structural_lifecycle, "orchestration_authority_added": false, "filesystem_authority_added": false}),
        )
    }

    #[allow(clippy::too_many_lines)]
    fn progressive_context_build(
        &mut self,
        session_id: &str,
        profiled: &ProfiledContextPacket,
        budget: ResourceBudget,
        reads_before: &RepositoryReadTelemetry,
        started: Instant,
    ) -> Result<Value, &'static str> {
        self.sessions
            .authorize(session_id, &self.consumer_id)
            .map_err(|_| "progressive session unavailable")?;
        if self.disclosures.contains_key(session_id) {
            let session = self
                .disclosures
                .get(session_id)
                .ok_or("progressive session unavailable")?;
            return progressive_exhausted(
                session,
                "map",
                DisclosureConsumption {
                    maps: 1,
                    ..DisclosureConsumption::default()
                },
                "map_count_exhausted",
            );
        }
        let runtime = self
            .structural_runtime
            .as_ref()
            .ok_or("progressive structural runtime unavailable")?;
        let packet = &profiled.packet;
        let initial_packet = packet.clone();
        let mut items = profiled.plan.structural_query.as_ref().map_or_else(
            || Ok(Vec::new()),
            |query| {
                query
                    .result
                    .edges
                    .iter()
                    .map(|edge| {
                        disclosure_item(
                            query,
                            &runtime.graph.graph_id,
                            &packet.workspace_identity,
                            &profiled.plan.plan_id,
                            &packet.policy_decision,
                            &budget,
                            edge,
                        )
                    })
                    .collect()
            },
        )?;
        let ceiling = progressive_ceiling(&budget)?;
        // A map larger than the session's item ceiling used to be discarded
        // whole: the traversal ran, the items were built, and the consumer got
        // nothing. The reads are already spent by this point and returning
        // nothing does not refund them, so the ceiling is honoured by
        // disclosing what fits and saying so.
        //
        // Truncating here, before the map identity is computed, keeps the
        // identity, the disclosed items, the session's lookup targets and the
        // consumption accounting describing the same set.
        let item_ceiling_reached = truncate_to_item_ceiling(&mut items, ceiling.returned_items);
        let public_items = items
            .iter()
            .map(|item| item.public.clone())
            .collect::<Vec<_>>();
        let map_id = sha256_identity(
            "progressive-disclosure-map",
            &json!({
                "workspace_identity": packet.workspace_identity,
                "workspace_snapshot": packet.workspace_snapshot,
                "graph_id": runtime.graph.graph_id,
                "plan_id": profiled.plan.plan_id,
                "policy_decision": packet.policy_decision,
                "budget": budget,
                "items": public_items,
                "contract_version": PROGRESSIVE_CONTRACT_VERSION
            }),
        )?;
        let reads_after = self.engine.repository_read_telemetry();
        let per_call = DisclosureConsumption {
            maps: 1,
            returned_items: u64::try_from(items.len()).unwrap_or(u64::MAX),
            repository_reads: reads_after
                .repository_file_reads
                .saturating_sub(reads_before.repository_file_reads),
            repeated_repository_reads: reads_after
                .repeated_repository_file_reads
                .saturating_sub(reads_before.repeated_repository_file_reads),
            elapsed_ms: elapsed_ms(started),
            ..DisclosureConsumption::default()
        };
        let mut progressive = ProgressiveSession {
            map_id: map_id.clone(),
            workspace_identity: packet.workspace_identity.clone(),
            workspace_snapshot: packet.workspace_snapshot.clone(),
            graph_id: runtime.graph.graph_id.clone(),
            plan_id: profiled.plan.plan_id.clone(),
            policy_decision: packet.policy_decision.clone(),
            budget,
            structural_query: profiled.plan.structural_query.clone(),
            items,
            consumed: DisclosureConsumption::default(),
            ceiling,
        };
        let state = if runtime.graph.completeness == "complete"
            && !item_ceiling_reached
            && !profiled
                .plan
                .structural_query
                .as_ref()
                .is_some_and(|query| query.result.truncated)
        {
            "ready"
        } else {
            "partial"
        };
        let omissions = profiled.plan.structural_query.as_ref().map_or_else(
            || vec!["structural_seed_unavailable".to_owned()],
            |query| {
                let mut values = query.result.unknowns.clone();
                if query.result.truncated {
                    values.push("structural_query_limited".into());
                }
                values.sort();
                values.dedup();
                values
            },
        );
        let mut omissions = omissions;
        if item_ceiling_reached {
            omissions.push("progressive_item_ceiling_reached".to_owned());
            omissions.sort();
            omissions.dedup();
        }
        let base = json!({
            "schema_name":"progressive-context-build-result",
            "schema_version":PROGRESSIVE_CONTRACT_VERSION,
            "delivery_mode":self.delivery_mode.as_str(),
            "initial_packet":initial_packet,
            "plan":{
                "schema_name":profiled.plan.schema_name,
                "schema_version":profiled.plan.schema_version,
                "plan_id":profiled.plan.plan_id,
                "task_profile":profiled.plan.task_profile,
                "workspace_snapshot":profiled.plan.workspace_snapshot,
                "policy_decision":profiled.plan.policy_decision,
                "coverage":profiled.plan.coverage,
                "omitted_candidates":profiled.plan.omitted_candidates
            },
            "disclosure_map":{
                "schema_name":"progressive-disclosure-map",
                "schema_version":PROGRESSIVE_CONTRACT_VERSION,
                "map_id":map_id,
                "workspace_snapshot":progressive.workspace_snapshot,
                "graph_id":progressive.graph_id,
                "state":state,
                "items":public_items,
                "omissions":omissions
            },
            "read_telemetry":reads_after,
            "structural_lifecycle":runtime.receipt,
            "orchestration_authority_added":false,
            "filesystem_authority_added":false
        });
        let result = finalize_progressive_output(
            &mut progressive,
            "map",
            state,
            &map_id,
            per_call,
            base,
            None,
        )?;
        self.disclosures.insert(session_id.to_owned(), progressive);
        Ok(result)
    }

    #[allow(clippy::too_many_lines)] // Validation, ownership, filtering, and cumulative receipt accounting remain one closed operation.
    fn context_disclosure_lookup(&mut self, value: Value) -> Result<Value, &'static str> {
        if self.delivery_mode != DeliveryMode::ProgressiveStructural {
            return Ok(progressive_unavailable(self.delivery_mode, "lookup"));
        }
        let args: DisclosureLookupArgs =
            serde_json::from_value(value).map_err(|_| "invalid disclosure lookup")?;
        self.sessions
            .authorize(&args.session_id, &self.consumer_id)
            .map_err(|_| "disclosure lookup failed")?;
        let max_items = canonical_decimal(&args.max_items)?;
        let max_depth = canonical_decimal(&args.max_depth)?;
        let max_bytes = canonical_decimal(&args.max_bytes)?;
        if max_depth > 1 || max_items == 0 || max_bytes == 0 {
            return Err("invalid disclosure lookup limits");
        }
        let allowed = [
            "declares",
            "contains",
            "imports",
            "exports",
            "calls",
            "references",
            "all_admitted",
        ];
        if args.relation_kinds.is_empty()
            || args.relation_kinds.len() > 7
            || args
                .relation_kinds
                .iter()
                .any(|kind| !allowed.contains(&kind.as_str()))
        {
            return Err("invalid disclosure relation kinds");
        }
        let started = Instant::now();
        let session = self
            .disclosures
            .get_mut(&args.session_id)
            .ok_or("disclosure lookup failed")?;
        if max_items > session.ceiling.returned_items
            || max_bytes > session.ceiling.serialized_response_bytes
        {
            return Err("disclosure lookup exceeds session ceilings");
        }
        let all = args
            .relation_kinds
            .iter()
            .any(|kind| kind == "all_admitted");
        let selected = session
            .items
            .iter()
            .filter(|item| {
                (args.handle == session.map_id || args.handle == item.public.item_handle)
                    && (all
                        || args
                            .relation_kinds
                            .contains(&item.public.relationship_class))
            })
            .take(usize::try_from(max_items).unwrap_or(usize::MAX))
            .map(|item| {
                json!({
                    "item":item.public,
                    "evidence_handle":item.evidence_handle,
                    "exact_evidence_available":true
                })
            })
            .collect::<Vec<_>>();
        if args.handle != session.map_id
            && !session
                .items
                .iter()
                .any(|item| item.public.item_handle == args.handle)
        {
            return Err("disclosure lookup failed");
        }
        let result_id = sha256_identity(
            "progressive-disclosure-lookup",
            &json!({"map_id":session.map_id,"handle":args.handle,"relation_kinds":args.relation_kinds,"max_items":args.max_items,"max_depth":args.max_depth,"items":selected}),
        )?;
        let per_call = DisclosureConsumption {
            lookups: 1,
            returned_items: u64::try_from(selected.len()).unwrap_or(u64::MAX),
            elapsed_ms: elapsed_ms(started),
            ..DisclosureConsumption::default()
        };
        let state = if selected.len() == session.items.len() {
            "ready"
        } else {
            "partial"
        };
        let base = json!({
            "schema_name":"progressive-disclosure-lookup-result",
            "schema_version":PROGRESSIVE_CONTRACT_VERSION,
            "delivery_mode":self.delivery_mode.as_str(),
            "map_id":session.map_id,
            "result_id":result_id,
            "state":state,
            "items":selected,
            "truncated":state == "partial",
            "authority_added":false
        });
        finalize_progressive_output(
            session,
            "lookup",
            state,
            &result_id,
            per_call,
            base,
            Some(max_bytes),
        )
    }

    #[allow(clippy::too_many_lines)] // Reservation, recovery, telemetry, and cumulative receipt accounting remain one closed operation.
    fn context_evidence_expand(&mut self, value: Value) -> Result<Value, &'static str> {
        if self.delivery_mode != DeliveryMode::ProgressiveStructural {
            return Ok(progressive_unavailable(self.delivery_mode, "expand"));
        }
        let args: EvidenceExpandArgs =
            serde_json::from_value(value).map_err(|_| "invalid evidence expansion")?;
        self.sessions
            .authorize(&args.session_id, &self.consumer_id)
            .map_err(|_| "evidence expansion failed")?;
        let before = canonical_decimal(&args.before_bytes)?;
        let after = canonical_decimal(&args.after_bytes)?;
        let maximum = canonical_decimal(&args.max_bytes)?;
        if maximum == 0 {
            return Err("invalid evidence expansion limits");
        }
        let (structural_query, edge_id, budget, ceiling, consumed) = {
            let session = self
                .disclosures
                .get(&args.session_id)
                .ok_or("evidence expansion failed")?;
            let item = session
                .items
                .iter()
                .find(|item| item.evidence_handle == args.evidence_handle)
                .ok_or("evidence expansion failed")?;
            (
                session
                    .structural_query
                    .clone()
                    .ok_or("evidence expansion failed")?,
                item.edge_id.clone(),
                session.budget.clone(),
                session.ceiling,
                session.consumed,
            )
        };
        if maximum > ceiling.exact_source_bytes {
            return Err("evidence expansion exceeds session ceilings");
        }
        let reservation = DisclosureConsumption {
            expansions: 1,
            exact_source_bytes: maximum,
            repository_reads: 2,
            repeated_repository_reads: 2,
            ..DisclosureConsumption::default()
        };
        if !consumed
            .checked_add(reservation)
            .is_some_and(|candidate| candidate.within(ceiling))
        {
            let session = self
                .disclosures
                .get(&args.session_id)
                .ok_or("evidence expansion failed")?;
            return progressive_exhausted(
                session,
                "expand",
                reservation,
                "cumulative_ceiling_exhausted",
            );
        }
        let context = RequestContext {
            request_id: args.request_id,
            event_id: args.event_id,
            subject: PolicySubject {
                caller_id: self.consumer_id.clone(),
                role: self.role.clone(),
                purpose: args.purpose,
            },
            occurred_at: args.occurred_at,
        };
        let started = Instant::now();
        let reads_before = self.engine.repository_read_telemetry();
        let expanded = self
            .engine
            .expand_structural_edge_evidence(
                &context,
                &structural_query,
                &edge_id,
                StructuralEvidenceExpansion {
                    before_bytes: before,
                    after_bytes: after,
                    max_bytes: maximum,
                },
                budget,
            )
            .map_err(|_| "evidence expansion failed")?;
        let reads_after = self.engine.repository_read_telemetry();
        let exact_bytes = encoded_len(&expanded.excerpt.bytes_base64url)?;
        let per_call = DisclosureConsumption {
            expansions: 1,
            exact_source_bytes: exact_bytes,
            repository_reads: reads_after
                .repository_file_reads
                .saturating_sub(reads_before.repository_file_reads),
            repeated_repository_reads: reads_after
                .repeated_repository_file_reads
                .saturating_sub(reads_before.repeated_repository_file_reads),
            elapsed_ms: elapsed_ms(started),
            ..DisclosureConsumption::default()
        };
        let result_id = sha256_identity(
            "progressive-evidence-expansion",
            &json!({"evidence_handle":args.evidence_handle,"evidence_id":expanded.evidence_id,"before_bytes":args.before_bytes,"after_bytes":args.after_bytes,"max_bytes":args.max_bytes}),
        )?;
        let base = json!({
            "schema_name":"progressive-evidence-expansion-result",
            "schema_version":PROGRESSIVE_CONTRACT_VERSION,
            "delivery_mode":self.delivery_mode.as_str(),
            "result_id":result_id,
            "state":"ready",
            "evidence":expanded,
            "read_telemetry":reads_after,
            "authority_added":false
        });
        let session = self
            .disclosures
            .get_mut(&args.session_id)
            .ok_or("evidence expansion failed")?;
        finalize_progressive_output(session, "expand", "ready", &result_id, per_call, base, None)
    }
}

fn disclosure_item(
    query: &StructuralPlannerQuery,
    graph_id: &str,
    workspace_identity: &str,
    plan_id: &str,
    policy_decision: &str,
    budget: &ResourceBudget,
    edge: &GraphEdge,
) -> Result<StoredDisclosureItem, &'static str> {
    let source_node = query
        .result
        .nodes
        .iter()
        .find(|node| node.node_id == edge.source_node);
    let display_path = source_node
        .map(|node| node.path.display_path.clone())
        .unwrap_or_default();
    let path_identity = source_node
        .map(|node| node.path.relative_units_base64url.clone())
        .unwrap_or_default();
    let symbol_label = edge
        .target_node
        .as_deref()
        .and_then(|target| {
            query
                .result
                .nodes
                .iter()
                .find(|node| node.node_id == target)
        })
        .and_then(|node| node.name.clone())
        .or_else(|| source_node.and_then(|node| node.name.clone()))
        .map(|label| label.chars().take(128).collect());
    let mut unknowns = Vec::new();
    if source_node.is_none() {
        unknowns.push("relationship_source_unavailable".into());
    }
    if matches!(edge.resolution.as_str(), "unresolved" | "unsupported") {
        unknowns.push(format!("relationship_{}", edge.resolution));
    }
    unknowns.sort();
    unknowns.dedup();
    let item_handle = sha256_identity(
        "progressive-disclosure-item",
        &json!({
            "workspace_identity":workspace_identity,
            "workspace_snapshot":query.result.workspace_snapshot,
            "graph_id":graph_id,
            "plan_id":plan_id,
            "policy_decision":policy_decision,
            "budget":budget,
            "query_id":query.query_id,
            "edge":edge,
            "display_path":display_path,
            "contract_version":PROGRESSIVE_CONTRACT_VERSION
        }),
    )?;
    let evidence_handle = sha256_identity(
        "progressive-evidence-handle",
        &json!({
            "workspace_identity":workspace_identity,
            "workspace_snapshot":query.result.workspace_snapshot,
            "graph_id":graph_id,
            "plan_id":plan_id,
            "policy_decision":policy_decision,
            "budget":budget,
            "query_id":query.query_id,
            "edge_id":edge.edge_id,
            "path_identity":path_identity,
            "span":edge.span,
            "contract_version":PROGRESSIVE_CONTRACT_VERSION
        }),
    )?;
    Ok(StoredDisclosureItem {
        public: DisclosureMapItem {
            item_handle,
            display_path,
            relationship_class: edge.kind.clone(),
            symbol_label,
            confidence: edge.resolution.clone(),
            freshness: "current_snapshot".into(),
            unknowns,
        },
        evidence_handle,
        edge_id: edge.edge_id.clone(),
    })
}

/// Narrow a map to the items its ceiling admits, reporting whether it bit.
///
/// A disclosure that exceeds its ceiling used to be discarded whole, after the
/// reads that produced it were already spent (ADR-0134). Truncation honours the
/// same bound and returns what it allows.
fn truncate_to_item_ceiling<T>(items: &mut Vec<T>, ceiling: u64) -> bool {
    let ceiling = usize::try_from(ceiling).unwrap_or(usize::MAX);
    let reached = items.len() > ceiling;
    if reached {
        items.truncate(ceiling);
    }
    reached
}

fn progressive_ceiling(budget: &ResourceBudget) -> Result<DisclosureConsumption, &'static str> {
    Ok(DisclosureConsumption {
        maps: MAX_PROGRESSIVE_MAPS,
        lookups: MAX_PROGRESSIVE_LOOKUPS,
        expansions: MAX_PROGRESSIVE_EXPANSIONS,
        returned_items: MAX_PROGRESSIVE_ITEMS.min(canonical_decimal(&budget.max_matches)?),
        exact_source_bytes: canonical_decimal(&budget.requested)?,
        serialized_response_bytes: MAX_PROGRESSIVE_RESPONSE_BYTES,
        repository_reads: canonical_decimal(&budget.max_files)?,
        repeated_repository_reads: canonical_decimal(&budget.max_files)?,
        elapsed_ms: canonical_decimal(&budget.max_elapsed_ms)?,
    })
}

fn finalize_progressive_output(
    session: &mut ProgressiveSession,
    operation: &str,
    state: &str,
    result_identity: &str,
    mut per_call: DisclosureConsumption,
    mut result: Value,
    call_byte_ceiling: Option<u64>,
) -> Result<Value, &'static str> {
    for _ in 0..8 {
        let cumulative = session
            .consumed
            .checked_add(per_call)
            .ok_or("progressive disclosure accounting overflow")?;
        if !cumulative.within(session.ceiling) {
            return progressive_exhausted(
                session,
                operation,
                per_call,
                "cumulative_ceiling_exhausted",
            );
        }
        let mut identity_consumption = cumulative;
        identity_consumption.elapsed_ms = 0;
        identity_consumption.serialized_response_bytes = 0;
        let receipt_id = sha256_identity(
            "progressive-disclosure-receipt",
            &json!({
                "operation":operation,
                "state":state,
                "result_identity":result_identity,
                "workspace_identity":session.workspace_identity,
                "workspace_snapshot":session.workspace_snapshot,
                "graph_id":session.graph_id,
                "plan_id":session.plan_id,
                "policy_decision":session.policy_decision,
                "cumulative":identity_consumption,
                "contract_version":PROGRESSIVE_CONTRACT_VERSION
            }),
        )?;
        let receipt = json!({
            "schema_name":"progressive-disclosure-receipt",
            "schema_version":PROGRESSIVE_CONTRACT_VERSION,
            "receipt_id":receipt_id,
            "mode":"progressive_structural",
            "operation":operation,
            "state":state,
            "result_identity":result_identity,
            "workspace_identity":session.workspace_identity,
            "workspace_snapshot":session.workspace_snapshot,
            "graph_id":session.graph_id,
            "plan_id":session.plan_id,
            "policy_decision":session.policy_decision,
            "session_bound":true,
            "per_call":per_call,
            "cumulative":cumulative,
            "remaining":cumulative.remaining(session.ceiling),
            "truncated":state == "partial",
            "exhausted":state == "exhausted",
            "authority_added":false
        });
        result
            .as_object_mut()
            .ok_or("invalid progressive result")?
            .insert("receipt".into(), receipt);
        let bytes = u64::try_from(
            serde_json::to_vec(&tool_result(result.clone(), false))
                .map_err(|_| "progressive result serialization failed")?
                .len(),
        )
        .unwrap_or(u64::MAX);
        if call_byte_ceiling.is_some_and(|ceiling| bytes > ceiling) {
            return progressive_exhausted(
                session,
                operation,
                per_call,
                "call_response_ceiling_exhausted",
            );
        }
        if per_call.serialized_response_bytes == bytes {
            session.consumed = cumulative;
            return Ok(result);
        }
        per_call.serialized_response_bytes = bytes;
    }
    Err("progressive disclosure accounting did not stabilize")
}

fn progressive_exhausted(
    session: &ProgressiveSession,
    operation: &str,
    attempted: DisclosureConsumption,
    reason_code: &str,
) -> Result<Value, &'static str> {
    let mut identity_consumption = session.consumed;
    identity_consumption.elapsed_ms = 0;
    identity_consumption.serialized_response_bytes = 0;
    let result_identity = sha256_identity(
        "progressive-disclosure-exhausted",
        &json!({
            "operation":operation,
            "reason_code":reason_code,
            "workspace_snapshot":session.workspace_snapshot,
            "graph_id":session.graph_id,
            "plan_id":session.plan_id,
            "policy_decision":session.policy_decision,
            "cumulative":identity_consumption,
            "attempted":attempted,
            "contract_version":PROGRESSIVE_CONTRACT_VERSION
        }),
    )?;
    let receipt_id = sha256_identity(
        "progressive-disclosure-receipt",
        &json!({
            "operation":operation,
            "state":"exhausted",
            "result_identity":result_identity,
            "workspace_identity":session.workspace_identity,
            "workspace_snapshot":session.workspace_snapshot,
            "graph_id":session.graph_id,
            "plan_id":session.plan_id,
            "policy_decision":session.policy_decision,
            "cumulative":identity_consumption,
            "contract_version":PROGRESSIVE_CONTRACT_VERSION
        }),
    )?;
    Ok(json!({
        "schema_name":"progressive-disclosure-exhausted",
        "schema_version":PROGRESSIVE_CONTRACT_VERSION,
        "delivery_mode":"progressive_structural",
        "operation":operation,
        "state":"exhausted",
        "reason_code":reason_code,
        "result_id":result_identity,
        "attempted":attempted,
        "receipt":{
            "schema_name":"progressive-disclosure-receipt",
            "schema_version":PROGRESSIVE_CONTRACT_VERSION,
            "receipt_id":receipt_id,
            "mode":"progressive_structural",
            "operation":operation,
            "state":"exhausted",
            "result_identity":result_identity,
            "workspace_identity":session.workspace_identity,
            "workspace_snapshot":session.workspace_snapshot,
            "graph_id":session.graph_id,
            "plan_id":session.plan_id,
            "policy_decision":session.policy_decision,
            "session_bound":true,
            "per_call":DisclosureConsumption::default(),
            "cumulative":session.consumed,
            "remaining":session.consumed.remaining(session.ceiling),
            "truncated":false,
            "exhausted":true,
            "authority_added":false
        },
        "authority_added":false
    }))
}

fn progressive_unavailable(mode: DeliveryMode, operation: &str) -> Value {
    json!({
        "schema_name":"progressive-disclosure-unavailable",
        "schema_version":PROGRESSIVE_CONTRACT_VERSION,
        "delivery_mode":mode.as_str(),
        "operation":operation,
        "state":"unavailable",
        "reason_code":"unavailable_in_delivery_mode",
        "authority_added":false
    })
}

fn sha256_identity(domain: &str, value: &Value) -> Result<String, &'static str> {
    json_contract_identity(domain, value).map_err(|_| "identity serialization failed")
}

fn canonical_decimal(value: &str) -> Result<u64, &'static str> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("invalid canonical decimal");
    }
    value.parse().map_err(|_| "invalid canonical decimal")
}

fn encoded_len(value: &str) -> Result<u64, &'static str> {
    if value
        .bytes()
        .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')))
    {
        return Err("invalid evidence encoding");
    }
    let length = u64::try_from(value.len()).map_err(|_| "evidence length overflow")?;
    if length % 4 == 1 {
        return Err("invalid evidence encoding");
    }
    length
        .checked_mul(6)
        .map(|bits| bits / 8)
        .ok_or("evidence length overflow")
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Normalize MCP's JSON-integer transport aliases to the canonical decimal
/// strings owned by `ResourceBudget`. The core remains the sole policy
/// validator; fractions, signed values, and all other JSON types fail closed.
fn normalize_wire_budget(value: &mut Value) -> Result<(), &'static str> {
    const FIELDS: [&str; 8] = [
        "requested",
        "max_evidence_items",
        "max_files",
        "max_excerpt_bytes_per_item",
        "max_matches",
        "max_traversal_depth",
        "max_elapsed_ms",
        "max_memory_bytes",
    ];
    let Some(budget) = value.get_mut("budget") else {
        return Ok(());
    };
    let object = budget.as_object_mut().ok_or("invalid budget input")?;
    for field in FIELDS {
        let Some(value) = object.get_mut(field) else {
            continue;
        };
        if let Some(number) = value.as_u64() {
            *value = Value::String(number.to_string());
        } else if !value.is_string() {
            return Err("invalid budget input");
        }
    }
    Ok(())
}

struct BoundedLine {
    bytes: Vec<u8>,
    overflowed: bool,
}

fn read_bounded_line<R: BufRead>(input: &mut R) -> std::io::Result<Option<BoundedLine>> {
    let mut bytes = Vec::new();
    let mut overflowed = false;
    let mut observed = false;
    loop {
        let available = input.fill_buf()?;
        if available.is_empty() {
            if !observed {
                return Ok(None);
            }
            break;
        }
        observed = true;
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |position| position + 1);
        let content_end = newline.unwrap_or(take);
        if !overflowed {
            let remaining = MAX_MESSAGE_BYTES.saturating_sub(bytes.len());
            if content_end > remaining {
                bytes.extend_from_slice(&available[..remaining]);
                overflowed = true;
            } else {
                bytes.extend_from_slice(&available[..content_end]);
            }
        }
        input.consume(take);
        if newline.is_some() {
            break;
        }
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    Ok(Some(BoundedLine { bytes, overflowed }))
}

fn is_supported_protocol_version(version: &str) -> bool {
    matches!(
        version,
        MCP_PROTOCOL_VERSION | MCP_COMPATIBLE_PROTOCOL_VERSION
    )
}

#[allow(clippy::needless_pass_by_value)]
fn success(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}
#[allow(clippy::needless_pass_by_value)]
fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}
#[allow(clippy::needless_pass_by_value)]
fn tool_result(structured: Value, is_error: bool) -> Value {
    let text = serde_json::to_string(&structured).unwrap_or_else(|_| "{}".into());
    json!({"content": [{"type": "text", "text": text}], "structuredContent": structured, "isError": is_error})
}

fn context_build_definition(budget: &Value) -> Value {
    json!({
        "name":"context_build",
        "title":"Build verified context",
        "description":"Build a bounded verified packet. Canonical direct evidence uses one-to-eight explicit steps and no profile/query; canonical planner evidence uses exactly one profile plus query and no steps. Every form requires client-generated request/event identifiers, RFC 3339 occurred_at, and the complete current hard budget from this schema. session_id is optional but, when present, must name an already-open same-process session for packet resolution. Adds no authority.",
        "inputSchema":{
            "type":"object",
            "additionalProperties":false,
            "properties":{
                "request_id":{"type":"string","pattern":IDENTIFIER_PATTERN,"description":"Client-generated opaque request identifier matching this pattern."}, "event_id":{"type":"string","pattern":IDENTIFIER_PATTERN,"description":"Client-generated opaque event identifier matching this pattern."},
                "purpose":{"type":"string","description":"Caller-declared bounded evidence purpose."}, "occurred_at":{"type":"string","description":"Caller-declared RFC 3339 operation time."},
                "steps":{"type":"array","minItems":1,"maxItems":8,"description":"Canonical direct-evidence form. Do not combine with profile, query, or structural declarations.","items":{"type":"object","additionalProperties":false,"properties":{"kind":{"enum":["exact_path","filename","literal","lexical"]},"query":{"type":"string"}},"required":["kind","query"]}},
                "profile":{"enum":["orientation","implementation","bug_investigation","change_review","security_review","test_selection","configuration_change"],"description":"Canonical planner form. Requires query and cannot be combined with steps."},
                "query":{"type":"string","minLength":1,"maxLength":4096,"description":"Bounded query required with profile unless explicit steps are used."},
                "structural_graph":{"type":"object"}, "start_node":{"type":"string"},
                "edge_kinds":{"type":"array","maxItems":8,"items":{"enum":["declares","contains","imports","exports","calls","references"]}},
                "declared_change_set":{"type":"object"}, "declared_associated_tests":{"type":"object"}, "orientation_graph":{"type":"object"}, "max_orientation_entries":{"type":"integer","minimum":1,"maximum":10000}, "budget":budget, "session_id":{"type":"string","description":"Optional already-open same-process session that owns the returned packet reference."}
            },
            "required":["request_id","event_id","purpose","occurred_at","budget"],
            "examples":[{
                "request_id":"req_guidepacket0001", "event_id":"evt_guidepacket0001",
                "purpose":"direct_evidence_example", "occurred_at":"2026-08-27T00:00:00Z",
                "steps":[{"kind":"filename","query":"README.md"}],
                "budget":{"unit_kind":"utf8_bytes", "requested":"4096", "hard":true,
                    "max_evidence_items":"20", "max_files":"100",
                    "max_excerpt_bytes_per_item":"256", "max_matches":"100",
                    "max_traversal_depth":"8", "max_elapsed_ms":"30000",
                    "max_memory_bytes":"1048576", "policy_profile":POLICY_PROFILE}
            }]
        }
    })
}

fn tool_definitions() -> Value {
    let budget = json!({
        "type":"object", "additionalProperties":false,
        "properties":{
            "unit_kind":{"const":"utf8_bytes"}, "requested":decimal_schema(),
            "hard":{"const":true}, "max_evidence_items":decimal_schema(),
            "max_files":decimal_schema(), "max_excerpt_bytes_per_item":decimal_schema(),
            "max_matches":decimal_schema(), "max_traversal_depth":decimal_schema(),
            "max_elapsed_ms":decimal_schema(), "max_memory_bytes":decimal_schema(),
            "policy_profile":{"const":POLICY_PROFILE}
        },
        "required":["unit_kind","requested","hard","max_evidence_items","max_files","max_excerpt_bytes_per_item","max_matches","max_traversal_depth","max_elapsed_ms","max_memory_bytes","policy_profile"]
    });
    json!([
        {"name":"context_session_open","title":"Open context session","description":"Open a bounded process-local session. Adds no authority.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"}},"required":["session_id"]}},
        context_build_definition(&budget),
        {"name":"context_disclosure_lookup","title":"Look up progressive structural context","description":"Resolve bounded structural relationships from a session-owned progressive disclosure map. The tool is always advertised and returns a closed unavailable result outside progressive_structural mode.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"},"handle":{"type":"string"},"relation_kinds":{"type":"array","minItems":1,"maxItems":7,"items":{"enum":["all_admitted","declares","contains","imports","exports","calls","references"]}},"max_items":decimal_schema(),"max_depth":decimal_schema(),"max_bytes":decimal_schema()},"required":["session_id","handle","relation_kinds","max_items","max_depth","max_bytes"]}},
        {"name":"context_evidence_expand","title":"Expand progressive exact evidence","description":"Expand a session-owned evidence handle through the existing exact-evidence gateway with current-source revalidation. The tool is always advertised and returns a closed unavailable result outside progressive_structural mode.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string","pattern":IDENTIFIER_PATTERN},"event_id":{"type":"string","pattern":IDENTIFIER_PATTERN},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"session_id":{"type":"string"},"evidence_handle":{"type":"string"},"before_bytes":decimal_schema(),"after_bytes":decimal_schema(),"max_bytes":decimal_schema()},"required":["request_id","event_id","purpose","occurred_at","session_id","evidence_handle","before_bytes","after_bytes","max_bytes"]}},
        {"name":"context_convention_exemplar_build","title":"Build verified convention exemplar context","description":"Build exact current-source evidence from caller-declared opaque labels and verified artifacts. It does not infer conventions or rank examples.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string"},"event_id":{"type":"string"},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"query":{"type":"string","minLength":1,"maxLength":4096},"declaration":{"type":"object"},"budget":budget},"required":["request_id","event_id","purpose","occurred_at","query","declaration","budget"]}},
        {"name":"structure_incremental_update","title":"Apply verified incremental structural update","description":"Rebuild a current structural graph from exact cached unchanged results and caller-declared validated replacements. Does not watch, poll, or launch a parser.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string"},"event_id":{"type":"string"},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"update":{"type":"object"},"budget":budget},"required":["request_id","event_id","purpose","occurred_at","update","budget"]}},
        {"name":"context_packet_resolve","title":"Resolve context packet","description":"Resolve an immutable packet for the owning process-local session.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"},"packet_id":{"type":"string"}},"required":["session_id","packet_id"]}},
        {"name":"context_session_close","title":"Close context session","description":"Close a process-local session and invalidate its references.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"}},"required":["session_id"]}}
    ])
}

fn decimal_schema() -> Value {
    json!({
        "type":"string",
        "pattern":DECIMAL_PATTERN,
        "description":"Canonical decimal-string wire form. The server continues to normalize non-negative integer values from previously compatible clients."
    })
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_map_over_its_item_ceiling_is_truncated_not_discarded() {
        // The reads are already spent when the ceiling is tested, so returning
        // nothing spends the cost and delivers no value (ADR-0134). Measured,
        // discarding cost six of twenty-two tasks their entire map.
        let mut items: Vec<u64> = (0..400).collect();
        assert!(truncate_to_item_ceiling(&mut items, 256));
        assert_eq!(items.len(), 256);
        // The traversal's own order survives: the prefix nearest the seeds.
        assert_eq!(items.first(), Some(&0));
        assert_eq!(items.last(), Some(&255));
    }

    #[test]
    fn a_map_within_its_item_ceiling_is_untouched() {
        let mut items: Vec<u64> = (0..10).collect();
        assert!(!truncate_to_item_ceiling(&mut items, 256));
        assert_eq!(items, (0..10).collect::<Vec<_>>());
    }

    #[test]
    fn truncation_only_ever_narrows() {
        // A ceiling of zero admits nothing; it must never admit more.
        let mut empty: Vec<u64> = Vec::new();
        assert!(!truncate_to_item_ceiling(&mut empty, 0));
        assert!(empty.is_empty());
        let mut one: Vec<u64> = vec![7];
        assert!(truncate_to_item_ceiling(&mut one, 0));
        assert!(one.is_empty());
    }
    use std::{
        fs,
        io::Cursor,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use context_engine::{EngineConfig, RequestContext};
    use context_store::AuditRetention;
    use context_structural::{
        FactClass, FactProvenance, GRAPH_VERSION, GraphFileInput, PROTOCOL_VERSION,
        RESOLVER_VERSION, StructuralFact, WorkerPath, WorkerSuccess, build_graph,
    };
    use context_workspace::{DiscoveryPolicy, PathIdentity};

    use super::*;

    fn assert_copilot_schema_subset(value: &Value) {
        match value {
            Value::Object(object) => {
                for unsupported in ["oneOf", "anyOf", "allOf", "not"] {
                    assert!(
                        !object.contains_key(unsupported),
                        "VS Code Copilot rejects schema keyword {unsupported}"
                    );
                }
                object.values().for_each(assert_copilot_schema_subset);
            }
            Value::Array(values) => values.iter().for_each(assert_copilot_schema_subset),
            _ => {}
        }
    }

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct Root(PathBuf);
    impl Root {
        fn new(label: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-mcp-{label}-{}-{sequence}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir(&path).expect("root");
            Self(path)
        }
    }
    impl Drop for Root {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn structural_graph(content_hash: &str, snapshot_id: &str) -> StructuralGraph {
        let identity = PathIdentity::from_portable_relative_path("auth.rs").expect("portable path");
        let path = WorkerPath {
            display_path: identity.display_path,
            platform_family: identity.platform_family.into(),
            unit_encoding: identity.unit_encoding.into(),
            relative_units_base64url: identity.relative_units_base64url,
        };
        let provenance = FactProvenance {
            method: "tree_sitter".into(),
            parser_version: "tree-sitter-0.26.13".into(),
            grammar_version: "tree-sitter-rust-0.24.2".into(),
            resolver_version: RESOLVER_VERSION.into(),
            graph_version: GRAPH_VERSION.into(),
        };
        let declaration = |local_key: &str, name: &str, start_byte, end_byte| StructuralFact {
            class: FactClass::Declaration,
            local_key: local_key.into(),
            syntax_kind: "function_item".into(),
            name: Some(name.into()),
            module: None,
            start_byte,
            end_byte,
            parent_key: None,
            confidence: "confirmed".into(),
            provenance: provenance.clone(),
        };
        let response = WorkerSuccess {
            schema_name: "structural-worker-response".into(),
            schema_version: PROTOCOL_VERSION.into(),
            request_id: "req_mcpstructural01".into(),
            content_hash: content_hash.into(),
            syntax_errors: false,
            facts: vec![
                declaration("declaration:0:30", "authenticate", 0, 30),
                declaration("declaration:31:44", "audit", 31, 44),
            ],
            warnings: Vec::new(),
        };
        build_graph(snapshot_id, vec![GraphFileInput { path, response }]).expect("graph")
    }

    fn server_with_structural_runtime(structural: bool) -> (McpServer, Root, Root) {
        let source = Root::new("source");
        let cache = Root::new("cache");
        let source_bytes = b"fn authenticate() { audit(); }\nfn audit() {}\n";
        fs::write(source.0.join("auth.rs"), source_bytes).expect("source");
        let request = RequestContext {
            request_id: "req_mcptestopen".into(),
            event_id: "evt_mcptestopen".into(),
            subject: PolicySubject {
                caller_id: "consumer_mcptest01".into(),
                role: "client".into(),
                purpose: "test".into(),
            },
            occurred_at: "2026-08-22T00:00:00Z".into(),
        };
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(20, 4096, 4096, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576)
                .expect("audit"),
        };
        let (mut engine, _) = LocalEngine::open(config, &request, &source.0).expect("open");
        let snapshot = engine
            .build_snapshot(
                &RequestContext {
                    request_id: "req_mcptestsnap".into(),
                    event_id: "evt_mcptestsnap".into(),
                    subject: request.subject,
                    occurred_at: request.occurred_at,
                },
                ResourceBudget::conservative(4096, 20, 100, 256, 100, 8, 30_000, 1_048_576)
                    .expect("budget"),
            )
            .expect("snapshot");
        let structural_runtime = structural.then(|| {
            let graph = structural_graph(
                "sha256:6737e54394e323b0b40a3aae590984b4015a2f6f0c9432e82e7ec56da1b1e38c",
                &snapshot.snapshot_id,
            );
            StructuralRuntime {
                task_scoped: None,
                receipt: StructuralLifecycleReceipt {
                    schema_name: "impresari_context_structural_lifecycle".into(),
                    schema_version: "1.0".into(),
                    enabled: true,
                    state: "prepared".into(),
                    graph_id: Some(graph.graph_id.clone()),
                    snapshot_id: Some(graph.workspace_snapshot.clone()),
                    worker_sha256: Some(
                        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                            .into(),
                    ),
                    graph_completeness: Some(graph.completeness.clone()),
                    preparation_elapsed_ms: 1,
                },
                graph,
                edge_kinds: Vec::new(),
            }
        });
        (
            McpServer::new(
                engine,
                ServerConfig {
                    consumer_id: "consumer_mcptest01".into(),
                    role: "client".into(),
                    session_policy: SessionPolicy::new(2, 4, 65_536).expect("sessions"),
                    structural_runtime,
                    delivery_mode: if structural {
                        DeliveryMode::EagerStructural
                    } else {
                        DeliveryMode::Ordinary
                    },
                },
            ),
            source,
            cache,
        )
    }

    fn server() -> (McpServer, Root, Root) {
        server_with_structural_runtime(false)
    }

    fn progressive_server() -> (McpServer, Root, Root) {
        let (mut server, source, cache) = server_with_structural_runtime(true);
        server.delivery_mode = DeliveryMode::ProgressiveStructural;
        (server, source, cache)
    }

    fn profiled_arguments(session_id: Option<&str>) -> Value {
        let mut value = json!({
            "request_id":"req_mcpstructuralbuild01",
            "event_id":"evt_mcpstructuralbuild01",
            "purpose":"bug_investigation",
            "occurred_at":"2026-08-22T00:00:00Z",
            "profile":"bug_investigation",
            "query":"Fix authenticate in auth.rs",
            "budget":{
                "unit_kind":"utf8_bytes", "requested":"4096", "hard":true,
                "max_evidence_items":"20", "max_files":"100",
                "max_excerpt_bytes_per_item":"256", "max_matches":"100",
                "max_traversal_depth":"8", "max_elapsed_ms":"30000",
                "max_memory_bytes":"1048576", "policy_profile":POLICY_PROFILE
            }
        });
        if let Some(session_id) = session_id {
            value
                .as_object_mut()
                .expect("arguments")
                .insert("session_id".into(), Value::String(session_id.into()));
        }
        value
    }

    fn open_progressive_map(server: &mut McpServer, session_id: &str) -> (Value, String) {
        server
            .session_open(json!({"session_id":session_id}))
            .expect("open progressive session");
        let built = server
            .context_build(profiled_arguments(Some(session_id)))
            .expect("progressive build");
        let lookup = server
            .context_disclosure_lookup(json!({
                "session_id":session_id,
                "handle":built["disclosure_map"]["map_id"],
                "relation_kinds":["all_admitted"],
                "max_items":"20",
                "max_depth":"1",
                "max_bytes":"65536"
            }))
            .expect("lookup");
        let evidence_handle = lookup["items"][0]["evidence_handle"]
            .as_str()
            .expect("evidence handle")
            .to_owned();
        (built, evidence_handle)
    }

    fn profiled_build(mut server: McpServer) -> Value {
        let request = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                "name":"context_build", "arguments": profiled_arguments(None)
            }
        });
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"1\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{request}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .nth(1)
            .expect("tool response")
    }

    fn listed_tools(mut server: McpServer) -> Value {
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .nth(1)
            .expect("tools response")["result"]
            .clone()
    }

    #[test]
    fn lifecycle_tools_and_sessions_are_newline_clean() {
        let (mut server, _source, _cache) = server();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"context_session_open\",\"arguments\":{\"session_id\":\"session_test01\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"context_session_close\",\"arguments\":{\"session_id\":\"session_test01\"}}}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let lines = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 5);
        let values = lines
            .iter()
            .map(|line| serde_json::from_slice::<Value>(line).expect("json only"))
            .collect::<Vec<_>>();
        assert_eq!(values[0]["error"]["code"], -32002);
        assert_eq!(values[1]["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
        assert_eq!(
            values[2]["result"]["tools"].as_array().map(Vec::len),
            Some(8)
        );
        assert_eq!(values[3]["result"]["isError"], false);
        assert_eq!(values[4]["result"]["isError"], false);
    }

    #[test]
    fn initialize_negotiates_supported_legacy_revision() {
        let (mut server, _source, _cache) = server();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\"}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(
            values[0]["result"]["protocolVersion"],
            MCP_COMPATIBLE_PROTOCOL_VERSION
        );
        assert_eq!(
            values[1]["result"]["tools"].as_array().map(Vec::len),
            Some(8)
        );
    }

    #[test]
    fn tool_calls_accept_standard_metadata_without_authority_effect() {
        let (mut server, _source, _cache) = server();
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"context_session_open\",\"arguments\":{\"session_id\":\"session_test01\"},\"_meta\":{\"progressToken\":\"client-owned\"}}}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values[1]["result"]["isError"], false);
        assert_eq!(values[1]["result"]["structuredContent"]["opened"], true);
    }

    #[test]
    fn context_build_schema_exposes_the_fixed_policy_profile() {
        let tools = tool_definitions();
        let build = tools
            .as_array()
            .expect("tool definitions are an array")
            .iter()
            .find(|tool| tool["name"] == "context_build")
            .expect("context_build definition");
        assert_eq!(
            build["inputSchema"]["properties"]["budget"]["properties"]["policy_profile"]["const"],
            POLICY_PROFILE
        );
        assert_eq!(
            build["inputSchema"]["properties"]["request_id"]["pattern"],
            IDENTIFIER_PATTERN
        );
        assert_eq!(
            build["inputSchema"]["properties"]["event_id"]["pattern"],
            IDENTIFIER_PATTERN
        );
        assert_eq!(
            build["inputSchema"]["properties"]["budget"]["properties"]["requested"]["pattern"],
            DECIMAL_PATTERN
        );
        assert_eq!(
            build["inputSchema"]["properties"]["budget"]["properties"]["requested"]["type"],
            "string"
        );
        assert!(
            build["description"]
                .as_str()
                .expect("context_build description")
                .contains("Canonical direct evidence")
        );
        assert!(
            build["inputSchema"]["properties"]["steps"]["description"]
                .as_str()
                .expect("steps description")
                .contains("Do not combine with profile")
        );
        assert_eq!(
            build["inputSchema"]["examples"][0]["steps"][0]["kind"],
            "filename"
        );
        assert_eq!(
            build["inputSchema"]["examples"][0]["budget"]["policy_profile"],
            POLICY_PROFILE
        );
        assert_copilot_schema_subset(&build["inputSchema"]);
    }

    #[test]
    fn profiled_context_build_accepts_a_fixed_policy_budget() {
        let (mut server, _source, _cache) = server();
        let budget = json!({
            "unit_kind":"utf8_bytes", "requested":"4096", "hard":true,
            "max_evidence_items":"20", "max_files":"100",
            "max_excerpt_bytes_per_item":"256", "max_matches":"100",
            "max_traversal_depth":"8", "max_elapsed_ms":"30000",
            "max_memory_bytes":"1048576", "policy_profile":POLICY_PROFILE
        });
        let request = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                "name":"context_build", "arguments": {
                    "request_id":"req_mcpprofilebudget01", "event_id":"evt_mcpprofilebudget01",
                    "purpose":"orientation", "occurred_at":"2026-08-22T00:00:00Z",
                    "profile":"orientation", "query":"auth", "budget":budget
                }
            }
        });
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"1\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{request}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values[1]["result"]["isError"], false, "{values:?}");
        assert_eq!(
            values[1]["result"]["structuredContent"]["plan"]["task_profile"],
            "orientation"
        );
    }

    #[test]
    fn trusted_structural_runtime_preserves_the_tool_request_and_adds_trailing_evidence() {
        let (baseline_tools, _baseline_source, _baseline_cache) =
            server_with_structural_runtime(false);
        let (structural_tools, _structural_source, _structural_cache) =
            server_with_structural_runtime(true);
        assert_eq!(listed_tools(baseline_tools), listed_tools(structural_tools));

        let (baseline, _baseline_source, _baseline_cache) = server_with_structural_runtime(false);
        let (structural, _structural_source, _structural_cache) =
            server_with_structural_runtime(true);
        let baseline = profiled_build(baseline);
        let structural = profiled_build(structural);
        assert_eq!(baseline["result"]["isError"], false, "{baseline:?}");
        assert_eq!(structural["result"]["isError"], false, "{structural:?}");
        let baseline_content = &baseline["result"]["structuredContent"];
        let structural_content = &structural["result"]["structuredContent"];
        assert_eq!(baseline_content["structural_lifecycle"]["enabled"], false);
        assert_eq!(
            baseline_content["structural_lifecycle"]["state"],
            "disabled"
        );
        assert_eq!(structural_content["structural_lifecycle"]["enabled"], true);
        assert_eq!(
            structural_content["structural_lifecycle"]["state"],
            "prepared"
        );
        assert!(
            structural_content["plan"]["structural_query"].is_object(),
            "{structural_content:?}"
        );
        let baseline_evidence = baseline_content["packet"]["observed_evidence"]
            .as_array()
            .expect("baseline evidence");
        let structural_evidence = structural_content["packet"]["observed_evidence"]
            .as_array()
            .expect("structural evidence");
        assert!(structural_evidence.len() > baseline_evidence.len());
        for (ordinary, seeded) in baseline_evidence.iter().zip(structural_evidence) {
            assert_eq!(ordinary["artifact"], seeded["artifact"]);
            assert_eq!(ordinary["range"], seeded["range"]);
            assert_eq!(ordinary["extraction"], seeded["extraction"]);
        }
        assert!(
            structural_evidence[baseline_evidence.len()..]
                .iter()
                .all(|evidence| evidence["extraction"]["method"] == "structural_graph_edge")
        );
        assert_eq!(
            baseline_content["read_telemetry"]["complete"],
            structural_content["read_telemetry"]["complete"]
        );
    }

    #[test]
    fn progressive_delivery_defers_and_recovers_exact_eager_evidence() {
        let (eager, _eager_source, _eager_cache) = server_with_structural_runtime(true);
        let eager = profiled_build(eager);
        let eager_evidence = eager["result"]["structuredContent"]["packet"]["observed_evidence"]
            .as_array()
            .expect("eager evidence")
            .iter()
            .filter(|value| value["extraction"]["method"] == "structural_graph_edge")
            .cloned()
            .collect::<Vec<_>>();
        assert!(!eager_evidence.is_empty());

        let (mut progressive, _source, _cache) = progressive_server();
        progressive
            .session_open(json!({"session_id":"session_progressive01"}))
            .expect("open progressive session");
        let built = progressive
            .context_build(profiled_arguments(Some("session_progressive01")))
            .expect("progressive build");
        let eager_bytes = serde_json::to_vec(&eager["result"]["structuredContent"])
            .expect("eager serialization")
            .len();
        let progressive_bytes = serde_json::to_vec(&built)
            .expect("progressive serialization")
            .len();
        assert!(
            progressive_bytes < eager_bytes,
            "progressive initial response must be smaller: progressive={progressive_bytes} eager={eager_bytes}"
        );
        assert_eq!(built["delivery_mode"], "progressive_structural");
        assert!(
            built["initial_packet"]["observed_evidence"]
                .as_array()
                .expect("initial anchors")
                .iter()
                .all(|value| value["extraction"]["method"] != "structural_graph_edge")
        );
        let map_id = built["disclosure_map"]["map_id"]
            .as_str()
            .expect("map id")
            .to_owned();
        let lookup = progressive
            .context_disclosure_lookup(json!({
                "session_id":"session_progressive01",
                "handle":map_id,
                "relation_kinds":["all_admitted"],
                "max_items":"20",
                "max_depth":"1",
                "max_bytes":"65536"
            }))
            .expect("lookup");
        let expected_label = if eager_evidence[0]["span"]["start_byte"] == "0" {
            "authenticate"
        } else {
            "audit"
        };
        let evidence_handle = lookup["items"]
            .as_array()
            .expect("lookup items")
            .iter()
            .find(|item| item["item"]["symbol_label"] == expected_label)
            .and_then(|item| item["evidence_handle"].as_str())
            .expect("matching evidence handle")
            .to_owned();
        let expanded = progressive
            .context_evidence_expand(json!({
                "request_id":"req_progressiveexpand01",
                "event_id":"evt_progressiveexpand01",
                "purpose":"bug_investigation",
                "occurred_at":"2026-08-22T00:00:01Z",
                "session_id":"session_progressive01",
                "evidence_handle":evidence_handle,
                "before_bytes":"0",
                "after_bytes":"0",
                "max_bytes":"256"
            }))
            .expect("expand");
        let mut recovered = expanded["evidence"].clone();
        recovered
            .as_object_mut()
            .expect("evidence object")
            .remove("evidence_id");
        recovered
            .as_object_mut()
            .expect("evidence object")
            .remove("workspace_snapshot");
        assert!(eager_evidence.iter().any(|candidate| {
            let mut candidate = candidate.clone();
            candidate
                .as_object_mut()
                .expect("evidence object")
                .remove("evidence_id");
            candidate
                .as_object_mut()
                .expect("evidence object")
                .remove("workspace_snapshot");
            candidate == recovered
        }));
        assert_eq!(expanded["receipt"]["operation"], "expand");
        assert_eq!(expanded["receipt"]["cumulative"]["maps"], 1);
        assert_eq!(expanded["receipt"]["cumulative"]["lookups"], 1);
        assert_eq!(expanded["receipt"]["cumulative"]["expansions"], 1);
    }

    #[test]
    fn progressive_handles_are_session_owned_and_close_fail_closed() {
        let (mut progressive, _source, _cache) = progressive_server();
        progressive
            .session_open(json!({"session_id":"session_progressive01"}))
            .expect("open owner");
        progressive
            .session_open(json!({"session_id":"session_progressive02"}))
            .expect("open foreign");
        let built = progressive
            .context_build(profiled_arguments(Some("session_progressive01")))
            .expect("build");
        let map_id = built["disclosure_map"]["map_id"].clone();
        let foreign = progressive.context_disclosure_lookup(json!({
            "session_id":"session_progressive02", "handle":map_id,
            "relation_kinds":["all_admitted"], "max_items":"1",
            "max_depth":"1", "max_bytes":"4096"
        }));
        assert_eq!(foreign.err(), Some("disclosure lookup failed"));
        assert!(
            progressive
                .context_disclosure_lookup(json!({
                    "session_id":"session_progressive01",
                    "handle":format!("sha256:{}", "f".repeat(64)),
                    "relation_kinds":["all_admitted"], "max_items":"1",
                    "max_depth":"1", "max_bytes":"4096"
                }))
                .is_err()
        );
        progressive
            .session_close(json!({"session_id":"session_progressive01"}))
            .expect("close");
        assert!(
            progressive
                .context_disclosure_lookup(json!({
                    "session_id":"session_progressive01", "handle":map_id,
                    "relation_kinds":["all_admitted"], "max_items":"1",
                    "max_depth":"1", "max_bytes":"4096"
                }))
                .is_err()
        );
    }

    #[test]
    fn progressive_tools_are_closed_without_progressive_mode() {
        let (mut ordinary, _source, _cache) = server();
        let unavailable = ordinary
            .context_disclosure_lookup(json!({"repository_text":"ignored"}))
            .expect("closed unavailable");
        assert_eq!(unavailable["state"], "unavailable");
        assert_eq!(unavailable["delivery_mode"], "ordinary");
    }

    #[test]
    fn progressive_build_shape_and_runtime_fail_before_repository_work() {
        let (mut progressive, _source, _cache) = progressive_server();
        let reads = progressive.engine.repository_read_telemetry();
        assert_eq!(
            progressive.context_build(profiled_arguments(None)).err(),
            Some("progressive context build requires an open session")
        );
        assert_eq!(progressive.engine.repository_read_telemetry(), reads);

        progressive
            .session_open(json!({"session_id":"session_progressive01"}))
            .expect("open session");
        let mut direct = profiled_arguments(Some("session_progressive01"));
        let object = direct.as_object_mut().expect("arguments");
        object.remove("profile");
        object.remove("query");
        object.insert(
            "steps".into(),
            json!([{"kind":"filename","query":"auth.rs"}]),
        );
        assert_eq!(
            progressive.context_build(direct).err(),
            Some("progressive context build requires only profile and query")
        );
        assert_eq!(progressive.engine.repository_read_telemetry(), reads);

        let (mut missing_runtime, _source, _cache) = server();
        missing_runtime.delivery_mode = DeliveryMode::ProgressiveStructural;
        assert_eq!(
            missing_runtime
                .context_build(profiled_arguments(Some("session_progressive01")))
                .err(),
            Some("structural delivery runtime unavailable")
        );
    }

    #[test]
    fn progressive_expansion_rejects_source_mutation_without_evidence() {
        let (mut progressive, source, _cache) = progressive_server();
        let (_, evidence_handle) = open_progressive_map(&mut progressive, "session_progressive01");
        fs::write(
            source.0.join("auth.rs"),
            b"fn authenticate() { panic!(\"changed\"); }\nfn audit() {}\n",
        )
        .expect("mutate source");
        let result = progressive.context_evidence_expand(json!({
            "request_id":"req_progressivestale01",
            "event_id":"evt_progressivestale01",
            "purpose":"bug_investigation",
            "occurred_at":"2026-08-22T00:00:01Z",
            "session_id":"session_progressive01",
            "evidence_handle":evidence_handle,
            "before_bytes":"0", "after_bytes":"0", "max_bytes":"256"
        }));
        assert_eq!(result.err(), Some("evidence expansion failed"));
    }

    #[test]
    fn progressive_cumulative_exhaustion_precedes_an_additional_read() {
        let (mut progressive, _source, _cache) = progressive_server();
        let (built, evidence_handle) =
            open_progressive_map(&mut progressive, "session_progressive01");
        let consumed_before_duplicate = progressive
            .disclosures
            .get("session_progressive01")
            .expect("session")
            .consumed;
        let reads_before_duplicate = progressive.engine.repository_read_telemetry();
        let duplicate_map = progressive
            .context_build(profiled_arguments(Some("session_progressive01")))
            .expect("closed exhausted map result");
        assert_eq!(duplicate_map["state"], "exhausted");
        assert_eq!(duplicate_map["receipt"]["cumulative"]["maps"], 1);
        assert_eq!(
            duplicate_map["receipt"]["cumulative"]["serialized_response_bytes"],
            consumed_before_duplicate.serialized_response_bytes
        );
        assert_eq!(
            progressive.engine.repository_read_telemetry(),
            reads_before_duplicate
        );
        assert_eq!(built["receipt"]["cumulative"]["maps"], 1);

        let first = progressive
            .context_evidence_expand(json!({
                "request_id":"req_progressiveexpand01",
                "event_id":"evt_progressiveexpand01",
                "purpose":"bug_investigation",
                "occurred_at":"2026-08-22T00:00:01Z",
                "session_id":"session_progressive01",
                "evidence_handle":evidence_handle,
                "before_bytes":"0", "after_bytes":"0", "max_bytes":"256"
            }))
            .expect("first expansion");
        assert_eq!(first["state"], "ready");
        let reads = progressive.engine.repository_read_telemetry();
        let exhausted = progressive
            .context_evidence_expand(json!({
                "request_id":"req_progressiveexpand02",
                "event_id":"evt_progressiveexpand02",
                "purpose":"bug_investigation",
                "occurred_at":"2026-08-22T00:00:02Z",
                "session_id":"session_progressive01",
                "evidence_handle":evidence_handle,
                "before_bytes":"0", "after_bytes":"0", "max_bytes":"4096"
            }))
            .expect("closed exhausted expansion");
        assert_eq!(exhausted["state"], "exhausted");
        assert_eq!(exhausted["receipt"]["cumulative"]["expansions"], 1);
        assert_eq!(progressive.engine.repository_read_telemetry(), reads);
    }

    #[test]
    fn profiled_context_build_normalizes_integer_budget_transport_values() {
        let (mut server, _source, _cache) = server();
        let budget = json!({
            "unit_kind":"utf8_bytes", "requested":4096, "hard":true,
            "max_evidence_items":20, "max_files":100,
            "max_excerpt_bytes_per_item":256, "max_matches":100,
            "max_traversal_depth":8, "max_elapsed_ms":30000,
            "max_memory_bytes":1_048_576, "policy_profile":POLICY_PROFILE
        });
        let request = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                "name":"context_build", "arguments": {
                    "request_id":"req_mcpintegerbudget01", "event_id":"evt_mcpintegerbudget01",
                    "purpose":"orientation", "occurred_at":"2026-08-22T00:00:00Z",
                    "profile":"orientation", "query":"auth", "budget":budget
                }
            }
        });
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"1\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{request}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values[1]["result"]["isError"], false, "{values:?}");
        assert_eq!(
            values[1]["result"]["structuredContent"]["packet"]["budget"]["requested"],
            "4096"
        );
    }

    #[test]
    fn profiled_context_build_returns_plan_and_packet_without_new_authority() {
        let (mut server, _source, _cache) = server();
        let budget = json!({
            "unit_kind":"utf8_bytes", "requested":"4096", "hard":true,
            "max_evidence_items":"20", "max_files":"100",
            "max_excerpt_bytes_per_item":"256", "max_matches":"100",
            "max_traversal_depth":"8", "max_elapsed_ms":"30000",
            "max_memory_bytes":"1048576",
            "policy_profile":"sha256:aba86621046ccc86cff7aabb81f4eab1020ab6db53ae1b649ea3977dec9649e8"
        });
        let request = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                "name":"context_build", "arguments": {
                    "request_id":"req_mcpprofile01", "event_id":"evt_mcpprofile01",
                    "purpose":"configuration_change", "occurred_at":"2026-08-22T00:00:00Z",
                    "profile":"configuration_change", "query":"auth", "budget":budget
                }
            }
        });
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"1\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{request}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values[1]["result"]["isError"], false, "{values:?}");
        let content = &values[1]["result"]["structuredContent"];
        assert_eq!(content["plan"]["schema_name"], "deterministic-context-plan");
        assert_eq!(content["packet"]["purpose"], "configuration_change");
        let telemetry = &content["read_telemetry"];
        assert_eq!(
            telemetry["schema_name"],
            "impresari_context_repository_read_telemetry"
        );
        assert_eq!(telemetry["schema_version"], "1.0");
        assert_eq!(
            telemetry["source_fingerprint_sha256"]
                .as_str()
                .expect("fingerprint")
                .len(),
            71
        );
        let reads = telemetry["repository_file_reads"]
            .as_u64()
            .expect("read count");
        let repeats = telemetry["repeated_repository_file_reads"]
            .as_u64()
            .expect("repeat count");
        assert!(reads >= 1);
        assert!(repeats <= reads);
        assert!(telemetry["source_bytes_read"].as_u64().expect("bytes") >= 21);
        assert_eq!(telemetry["complete"], true);
        assert_eq!(content["orchestration_authority_added"], false);
        assert_eq!(content["filesystem_authority_added"], false);
    }

    #[test]
    fn declared_change_set_context_build_uses_the_shared_verified_contract() {
        let (mut server, _source, _cache) = server();
        let exact = server
            .engine
            .search(
                &RequestContext {
                    request_id: "req_mcpdeclaredlookup".into(),
                    event_id: "evt_mcpdeclaredlookup".into(),
                    subject: PolicySubject {
                        caller_id: "consumer_mcptest01".into(),
                        role: "client".into(),
                        purpose: "change_review".into(),
                    },
                    occurred_at: "2026-08-22T00:00:00Z".into(),
                },
                context_engine::QueryKind::ExactPath,
                "auth.rs",
                &ResourceBudget::conservative(4096, 20, 100, 256, 100, 8, 30_000, 1_048_576)
                    .expect("budget"),
            )
            .expect("exact evidence");
        let artifact = &exact.matches[0].artifact;
        let budget = json!({
            "unit_kind":"utf8_bytes", "requested":"4096", "hard":true,
            "max_evidence_items":"20", "max_files":"100",
            "max_excerpt_bytes_per_item":"256", "max_matches":"100",
            "max_traversal_depth":"8", "max_elapsed_ms":"30000",
            "max_memory_bytes":"1048576",
            "policy_profile":"sha256:aba86621046ccc86cff7aabb81f4eab1020ab6db53ae1b649ea3977dec9649e8"
        });
        let request = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call", "params": {
                "name":"context_build", "arguments": {
                    "request_id":"req_mcpdeclared01", "event_id":"evt_mcpdeclared01",
                    "purpose":"change_review", "occurred_at":"2026-08-22T00:00:00Z",
                    "profile":"change_review", "query":"authenticate", "budget":budget,
                    "declared_change_set": {
                        "schema_name":"declared-change-set", "schema_version":"1.0.0",
                        "workspace_snapshot":exact.snapshot_id,
                        "entries":[{"path": {
                            "platform_family":artifact.path.platform_family,
                            "unit_encoding":artifact.path.unit_encoding,
                            "relative_units_base64url":artifact.path.relative_units_base64url
                        }, "content_hash":artifact.content_hash}]
                    }
                }
            }
        });
        let input = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test\",\"version\":\"1\"}}}}}}\n{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}}\n{request}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values[1]["result"]["isError"], false, "{values:?}");
        let content = &values[1]["result"]["structuredContent"];
        assert_eq!(content["plan"]["task_profile"], "change_review");
        assert_eq!(
            content["plan"]["declared_change_set"]["workspace_snapshot"],
            request["params"]["arguments"]["declared_change_set"]["workspace_snapshot"]
        );
    }

    #[test]
    fn malformed_batch_duplicate_and_oversized_messages_fail_closed() {
        let (mut server, _source, _cache) = server();
        let oversized = "x".repeat(MAX_MESSAGE_BYTES + 1);
        let input = format!(
            "not-json\n[]\n{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}}\n{{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}}\n{oversized}\n{{\"jsonrpc\":\"2.0\",\"id\":2}}\n{{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"ping\",\"unexpected\":true}}\n"
        );
        let mut output = Vec::new();
        server
            .serve(Cursor::new(input), &mut output)
            .expect("serve");
        let values = output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_slice::<Value>(line).expect("json"))
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 7);
        assert_eq!(values[0]["error"]["code"], -32700);
        assert_eq!(values[1]["error"]["code"], -32600);
        assert_eq!(values[3]["error"]["message"], "duplicate request id");
        assert_eq!(
            values[4]["error"]["message"],
            "request exceeds transport limit"
        );
        assert_eq!(values[5]["error"]["message"], "invalid request method");
        assert_eq!(values[6]["error"]["message"], "invalid request fields");
    }
}
