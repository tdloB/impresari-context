// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Bounded local stdio MCP transport over the public context engine."]

use std::{
    collections::BTreeSet,
    io::{BufRead, Write},
};

use context_core::{PolicySubject, ResourceBudget};
use context_engine::{
    ContextPlan, ContextPlanStep, DeclaredAssociatedTests, DeclaredChangeSet,
    DeclaredConventionExemplars, IncrementalStructuralUpdate, LocalEngine,
    RepositoryOrientationRequest, RequestContext, StructuralImpactRequest, TaskProfile,
};
use context_session::{SessionPolicy, SessionStore};
use context_structural::StructuralGraph;
use serde::Deserialize;
use serde_json::{Value, json};

/// Preferred MCP revision implemented by this transport.
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
/// Older MCP revision accepted for clients that have not yet adopted the preferred revision.
pub const MCP_COMPATIBLE_PROTOCOL_VERSION: &str = "2025-06-18";
/// Maximum encoded JSON-RPC line accepted from a client.
pub const MAX_MESSAGE_BYTES: usize = 1_048_576;
/// Maximum request identifiers retained for replay rejection in one process.
pub const MAX_REQUESTS: usize = 10_000;

/// Trusted launch configuration. The client cannot change these values via MCP.
pub struct ServerConfig {
    /// Fixed consumer identity.
    pub consumer_id: String,
    /// Fixed policy role.
    pub role: String,
    /// Bounded process-local session policy.
    pub session_policy: SessionPolicy,
}

/// Stateful single-client stdio MCP service.
pub struct McpServer {
    engine: LocalEngine,
    consumer_id: String,
    role: String,
    sessions: SessionStore,
    initialized_response_sent: bool,
    operation_ready: bool,
    request_ids: BTreeSet<String>,
}

impl McpServer {
    /// Creates a server around an already-authorized and snapshotted engine.
    #[must_use]
    pub fn new(engine: LocalEngine, config: ServerConfig) -> Self {
        Self {
            engine,
            consumer_id: config.consumer_id,
            role: config.role,
            sessions: SessionStore::new(config.session_policy),
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

    fn context_convention_exemplar_build(&mut self, value: Value) -> Result<Value, &'static str> {
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

    fn structure_incremental_update(&mut self, value: Value) -> Result<Value, &'static str> {
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
    fn context_build(&mut self, value: Value) -> Result<Value, &'static str> {
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
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid context input")?;
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
                let profiled = self
                    .engine
                    .build_profiled_context(&context, profile, &query, args.budget)
                    .map_err(|_| "profiled context build failed")?;
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
        Ok(
            json!({"packet": packet, "plan": plan, "reference": reference, "orchestration_authority_added": false, "filesystem_authority_added": false}),
        )
    }
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
        "description":"Build a bounded verified packet from explicit steps, a deterministic declared profile, a verified caller-declared current change set or associated-test set, or a profile plus an already validated snapshot-bound structural graph. Adds no authority.",
        "inputSchema":{
            "type":"object",
            "additionalProperties":false,
            "properties":{
                "request_id":{"type":"string"}, "event_id":{"type":"string"},
                "purpose":{"type":"string"}, "occurred_at":{"type":"string"},
                "steps":{"type":"array","minItems":1,"maxItems":8,"items":{"type":"object","additionalProperties":false,"properties":{"kind":{"enum":["exact_path","filename","literal","lexical"]},"query":{"type":"string"}},"required":["kind","query"]}},
                "profile":{"enum":["orientation","implementation","bug_investigation","change_review","security_review","test_selection","configuration_change"]},
                "query":{"type":"string","minLength":1,"maxLength":4096},
                "structural_graph":{"type":"object"}, "start_node":{"type":"string"},
                "edge_kinds":{"type":"array","maxItems":8,"items":{"enum":["declares","contains","imports","exports","calls","references"]}},
                "declared_change_set":{"type":"object"}, "declared_associated_tests":{"type":"object"}, "orientation_graph":{"type":"object"}, "max_orientation_entries":{"type":"integer","minimum":1,"maximum":10000}, "budget":budget, "session_id":{"type":"string"}
            },
            "required":["request_id","event_id","purpose","occurred_at","budget"],
            "oneOf":[
                {"required":["steps"],"not":{"anyOf":[{"required":["profile"]},{"required":["query"]},{"required":["structural_graph"]},{"required":["start_node"]},{"required":["edge_kinds"]},{"required":["declared_change_set"]},{"required":["declared_associated_tests"]},{"required":["orientation_graph"]},{"required":["max_orientation_entries"]}]}},
                {"required":["profile","query"],"not":{"anyOf":[{"required":["steps"]},{"required":["structural_graph"]},{"required":["start_node"]},{"required":["edge_kinds"]},{"required":["declared_change_set"]},{"required":["declared_associated_tests"]},{"required":["orientation_graph"]},{"required":["max_orientation_entries"]}]}},
                {"required":["profile","query","structural_graph","start_node"],"not":{"anyOf":[{"required":["steps"]},{"required":["declared_change_set"]},{"required":["declared_associated_tests"]},{"required":["orientation_graph"]},{"required":["max_orientation_entries"]}]}},
                {"required":["profile","query","declared_change_set"],"not":{"anyOf":[{"required":["steps"]},{"required":["structural_graph"]},{"required":["start_node"]},{"required":["edge_kinds"]},{"required":["declared_associated_tests"]},{"required":["orientation_graph"]},{"required":["max_orientation_entries"]}]}},
                {"required":["profile","query","declared_associated_tests"],"not":{"anyOf":[{"required":["steps"]},{"required":["structural_graph"]},{"required":["start_node"]},{"required":["edge_kinds"]},{"required":["declared_change_set"]},{"required":["orientation_graph"]},{"required":["max_orientation_entries"]}]}},
                {"required":["profile","query","orientation_graph","max_orientation_entries"],"not":{"anyOf":[{"required":["steps"]},{"required":["structural_graph"]},{"required":["start_node"]},{"required":["edge_kinds"]},{"required":["declared_change_set"]},{"required":["declared_associated_tests"]}]}}
            ]
        }
    })
}

fn tool_definitions() -> Value {
    let budget = json!({
        "type":"object", "additionalProperties":false,
        "properties":{
            "unit_kind":{"const":"utf8_bytes"}, "requested":{"type":"string"},
            "hard":{"const":true}, "max_evidence_items":{"type":"string"},
            "max_files":{"type":"string"}, "max_excerpt_bytes_per_item":{"type":"string"},
            "max_matches":{"type":"string"}, "max_traversal_depth":{"type":"string"},
            "max_elapsed_ms":{"type":"string"}, "max_memory_bytes":{"type":"string"},
            "policy_profile":{"type":"string"}
        },
        "required":["unit_kind","requested","hard","max_evidence_items","max_files","max_excerpt_bytes_per_item","max_matches","max_traversal_depth","max_elapsed_ms","max_memory_bytes","policy_profile"]
    });
    json!([
        {"name":"context_session_open","title":"Open context session","description":"Open a bounded process-local session. Adds no authority.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"}},"required":["session_id"]}},
        context_build_definition(&budget),
        {"name":"context_convention_exemplar_build","title":"Build verified convention exemplar context","description":"Build exact current-source evidence from caller-declared opaque labels and verified artifacts. It does not infer conventions or rank examples.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string"},"event_id":{"type":"string"},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"query":{"type":"string","minLength":1,"maxLength":4096},"declaration":{"type":"object"},"budget":budget},"required":["request_id","event_id","purpose","occurred_at","query","declaration","budget"]}},
        {"name":"structure_incremental_update","title":"Apply verified incremental structural update","description":"Rebuild a current structural graph from exact cached unchanged results and caller-declared validated replacements. Does not watch, poll, or launch a parser.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string"},"event_id":{"type":"string"},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"update":{"type":"object"},"budget":budget},"required":["request_id","event_id","purpose","occurred_at","update","budget"]}},
        {"name":"context_packet_resolve","title":"Resolve context packet","description":"Resolve an immutable packet for the owning process-local session.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"},"packet_id":{"type":"string"}},"required":["session_id","packet_id"]}},
        {"name":"context_session_close","title":"Close context session","description":"Close a process-local session and invalidate its references.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"session_id":{"type":"string"}},"required":["session_id"]}}
    ])
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::Cursor,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    use context_engine::{EngineConfig, RequestContext};
    use context_store::AuditRetention;
    use context_workspace::DiscoveryPolicy;

    use super::*;

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

    fn server() -> (McpServer, Root, Root) {
        let source = Root::new("source");
        let cache = Root::new("cache");
        fs::write(source.0.join("auth.rs"), b"fn authenticate() {}\n").expect("source");
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
        engine
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
        (
            McpServer::new(
                engine,
                ServerConfig {
                    consumer_id: "consumer_mcptest01".into(),
                    role: "client".into(),
                    session_policy: SessionPolicy::new(2, 4, 65_536).expect("sessions"),
                },
            ),
            source,
            cache,
        )
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
            Some(6)
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
            Some(6)
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
