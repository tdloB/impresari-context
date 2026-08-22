// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Bounded local stdio MCP transport over the public context engine."]

use std::{
    collections::BTreeSet,
    io::{BufRead, Write},
};

use context_core::{PolicySubject, ResourceBudget};
use context_engine::{ContextPlan, ContextPlanStep, LocalEngine, RequestContext};
use context_session::{SessionPolicy, SessionStore};
use serde::Deserialize;
use serde_json::{Value, json};

/// MCP revision implemented by this transport.
pub const MCP_PROTOCOL_VERSION: &str = "2025-11-25";
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
        if request.protocol_version != MCP_PROTOCOL_VERSION
            || !request.capabilities.is_object()
            || !request.client_info.is_object()
        {
            return error(id, -32602, "unsupported protocol version or capabilities");
        }
        self.initialized_response_sent = true;
        success(
            id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
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
        }
        let Ok(call) = serde_json::from_value::<Call>(params) else {
            return error(id, -32602, "invalid tool call parameters");
        };
        let result = match call.name.as_str() {
            "context_session_open" => self.session_open(call.arguments),
            "context_build" => self.context_build(call.arguments),
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

    fn context_build(&mut self, value: Value) -> Result<Value, &'static str> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Args {
            request_id: String,
            event_id: String,
            purpose: String,
            occurred_at: String,
            steps: Vec<ContextPlanStep>,
            budget: ResourceBudget,
            session_id: Option<String>,
        }
        let args: Args = serde_json::from_value(value).map_err(|_| "invalid context input")?;
        let packet = self
            .engine
            .build_planned_context(
                &RequestContext {
                    request_id: args.request_id,
                    event_id: args.event_id,
                    subject: PolicySubject {
                        caller_id: self.consumer_id.clone(),
                        role: self.role.clone(),
                        purpose: args.purpose,
                    },
                    occurred_at: args.occurred_at,
                },
                &ContextPlan { steps: args.steps },
                args.budget,
            )
            .map_err(|_| "context build failed")?;
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
            json!({"packet": packet, "reference": reference, "orchestration_authority_added": false, "filesystem_authority_added": false}),
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
        {"name":"context_build","title":"Build verified context","description":"Build a bounded verified packet from the fixed authorized workspace.","inputSchema":{"type":"object","additionalProperties":false,"properties":{"request_id":{"type":"string"},"event_id":{"type":"string"},"purpose":{"type":"string"},"occurred_at":{"type":"string"},"steps":{"type":"array","minItems":1,"maxItems":8,"items":{"type":"object","additionalProperties":false,"properties":{"kind":{"enum":["exact_path","filename","literal","lexical"]},"query":{"type":"string"}},"required":["kind","query"]}},"budget":budget,"session_id":{"type":"string"}},"required":["request_id","event_id","purpose","occurred_at","steps","budget"]}},
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

    fn server() -> (McpServer, Root) {
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
        )
    }

    #[test]
    fn lifecycle_tools_and_sessions_are_newline_clean() {
        let (mut server, _source) = server();
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
            Some(4)
        );
        assert_eq!(values[3]["result"]["isError"], false);
        assert_eq!(values[4]["result"]["isError"], false);
    }

    #[test]
    fn malformed_batch_duplicate_and_oversized_messages_fail_closed() {
        let (mut server, _source) = server();
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
