// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Frozen provider-free ordinary/eager/progressive MCP mechanics gate."]

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{POLICY_PROFILE, PolicySubject, ResourceBudget};
use context_engine::{EngineConfig, LocalEngine, RequestContext};
use context_mcp::{
    DeliveryMode, MCP_PROTOCOL_VERSION, McpServer, ServerConfig, StructuralLifecycleReceipt,
    StructuralRuntime,
};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_structural::{
    FactClass, GRAPH_VERSION, GraphFileInput, PROTOCOL_VERSION, RESOLVER_VERSION, StructuralGraph,
    StructuralLanguage, WorkerPath, WorkerRequest, build_graph, process_request,
};
use context_workspace::{DiscoveryPolicy, PathIdentity};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const CORPUS: &[u8] = include_bytes!("../../../evaluation/v1/structural-utility-manifest.json");
const GATE: &[u8] = include_bytes!("../../../evaluation/v1/progressive-structural-manifest.json");

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    schema_version: String,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    id: String,
    split: String,
    language: StructuralLanguage,
    path: String,
    task: String,
    source: String,
    selected_symbol: String,
    edge_kinds: Vec<String>,
    minimum_structural_evidence: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GateManifest {
    schema_version: String,
    fixture_manifest: String,
    fixture_manifest_sha256: String,
    delivery_modes: Vec<String>,
    scripted_expansion_policy: String,
    requirements: Requirements,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)] // Frozen manifest booleans make every required invariant explicit and independently reviewable.
struct Requirements {
    identical_tool_definitions: bool,
    warm_cache_arm: bool,
    progressive_initial_smaller_than_eager: bool,
    preserve_ordinary_anchors: bool,
    full_expansion_equals_eager_structural_evidence: bool,
    source_immutable: bool,
    provider_calls: u64,
}

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct Root(PathBuf);

impl Root {
    fn new(label: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "impresari-progressive-gate-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create root");
        Self(path)
    }
}

impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Arm {
    server: McpServer,
    _cache: Root,
    tools: Value,
    build: Value,
    initial_tool_result_bytes: usize,
}

fn hash(bytes: &[u8]) -> String {
    let mut value = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(value, "{byte:02x}").expect("hash formatting");
    }
    value
}

fn request(sequence: u64) -> RequestContext {
    RequestContext {
        request_id: format!("req_progressive{sequence:08}"),
        event_id: format!("evt_progressive{sequence:08}"),
        subject: PolicySubject {
            caller_id: "provider_free_progressive_gate".into(),
            role: "local_evaluator".into(),
            purpose: "progressive_structural_mechanics".into(),
        },
        occurred_at: format!("2026-09-01T00:00:{sequence:02}Z"),
    }
}

fn budget() -> ResourceBudget {
    ResourceBudget::conservative(16_384, 32, 100, 512, 100, 16, 30_000, 536_870_912)
        .expect("budget")
}

fn grammar_version(language: StructuralLanguage) -> &'static str {
    match language {
        StructuralLanguage::TypeScript => "tree-sitter-typescript-0.23.2",
        StructuralLanguage::Rust => "tree-sitter-rust-0.24.2",
        StructuralLanguage::Ruby => "tree-sitter-ruby-0.23.1",
        _ => unreachable!("closed progressive corpus language"),
    }
}

fn graph(fixture: &Fixture, snapshot_id: &str) -> StructuralGraph {
    let source = fixture.source.as_bytes();
    let identity = PathIdentity::from_portable_relative_path(&fixture.path).expect("portable path");
    let request = WorkerRequest {
        schema_name: "structural-worker-request".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: format!("req_progressive_{}", fixture.id),
        language: fixture.language,
        path: WorkerPath {
            display_path: identity.display_path,
            platform_family: identity.platform_family.into(),
            unit_encoding: identity.unit_encoding.into(),
            relative_units_base64url: identity.relative_units_base64url,
        },
        content_hash: hash(source),
        source_base64url: URL_SAFE_NO_PAD.encode(source),
        fact_classes: vec![
            FactClass::Declaration,
            FactClass::Contains,
            FactClass::Import,
            FactClass::Export,
            FactClass::Call,
            FactClass::Reference,
        ],
        max_facts: 100,
        max_nesting_depth: 16,
        max_response_bytes: 1_048_576,
        parser_version: "tree-sitter-0.26.13".into(),
        grammar_version: grammar_version(fixture.language).into(),
        resolver_version: RESOLVER_VERSION.into(),
        graph_version: GRAPH_VERSION.into(),
    };
    let response = process_request(&request).expect("worker response");
    build_graph(
        snapshot_id,
        vec![GraphFileInput {
            path: request.path,
            response,
        }],
    )
    .expect("graph")
}

fn exchange(server: &mut McpServer, requests: &[Value]) -> Vec<Value> {
    let input = requests
        .iter()
        .map(|value| serde_json::to_string(value).expect("request JSON"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let mut output = Vec::new();
    server
        .serve(Cursor::new(input), &mut output)
        .expect("MCP exchange");
    output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("response JSON"))
        .collect()
}

fn build_arguments(fixture: &Fixture, progressive: bool, run: &str) -> Value {
    let mut value = json!({
        "request_id":format!("req_progressive_{run}"),
        "event_id":format!("evt_progressive_{run}"),
        "purpose":"bug_investigation",
        "occurred_at":"2026-09-01T00:00:03Z",
        "profile":"bug_investigation",
        "query":fixture.task,
        "budget":{
            "unit_kind":"utf8_bytes", "requested":"16384", "hard":true,
            "max_evidence_items":"32", "max_files":"100",
            "max_excerpt_bytes_per_item":"512", "max_matches":"100",
            "max_traversal_depth":"16", "max_elapsed_ms":"30000",
            "max_memory_bytes":"536870912", "policy_profile":POLICY_PROFILE
        }
    });
    if progressive {
        value.as_object_mut().expect("arguments").insert(
            "session_id".into(),
            Value::String("session_progressive01".into()),
        );
    }
    value
}

fn run_arm(fixture: &Fixture, source: &Root, mode: DeliveryMode) -> Arm {
    let cache = Root::new("cache");
    run_arm_with_cache(fixture, source, mode, cache, "fresh01", 0)
}

fn run_arm_with_cache(
    fixture: &Fixture,
    source: &Root,
    mode: DeliveryMode,
    cache: Root,
    run: &str,
    request_offset: u64,
) -> Arm {
    let config = EngineConfig {
        cache_root: cache.0.clone(),
        discovery: DiscoveryPolicy::new(10, 16_384, 16_384, 8).expect("discovery"),
        audit_retention: AuditRetention::new("2026-09-01T00:00:00Z", 100, 1_048_576)
            .expect("retention"),
    };
    let (mut engine, _) =
        LocalEngine::open(config, &request(request_offset + 1), &source.0).expect("engine");
    let snapshot = engine
        .build_snapshot(&request(request_offset + 2), budget())
        .expect("snapshot");
    let structural_runtime = (mode != DeliveryMode::Ordinary).then(|| {
        let graph = graph(fixture, &snapshot.snapshot_id);
        StructuralRuntime {
            receipt: StructuralLifecycleReceipt {
                schema_name: "impresari_context_structural_lifecycle".into(),
                schema_version: "1.0".into(),
                enabled: true,
                state: "prepared".into(),
                graph_id: Some(graph.graph_id.clone()),
                snapshot_id: Some(graph.workspace_snapshot.clone()),
                worker_sha256: Some(format!("sha256:{}", "a".repeat(64))),
                graph_completeness: Some(graph.completeness.clone()),
                preparation_elapsed_ms: 0,
            },
            graph,
            edge_kinds: fixture.edge_kinds.clone(),
        }
    });
    let mut server = McpServer::new(
        engine,
        ServerConfig {
            consumer_id: "provider_free_progressive_gate".into(),
            role: "local_evaluator".into(),
            session_policy: SessionPolicy::new(2, 8, 1_048_576).expect("sessions"),
            delivery_mode: mode,
            structural_runtime,
        },
    );
    let mut requests = vec![
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":MCP_PROTOCOL_VERSION,"capabilities":{},"clientInfo":{"name":"provider-free-gate","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
    ];
    if mode == DeliveryMode::ProgressiveStructural {
        requests.push(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"context_session_open","arguments":{"session_id":"session_progressive01"}}}));
    }
    requests.push(json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"context_build","arguments":build_arguments(fixture, mode == DeliveryMode::ProgressiveStructural, run)}}));
    let responses = exchange(&mut server, &requests);
    let tools = responses[1]["result"]["tools"].clone();
    let build_response = responses.last().expect("build response");
    assert_eq!(
        build_response["result"]["isError"], false,
        "{build_response:?}"
    );
    let initial_tool_result_bytes = serde_json::to_vec(&build_response["result"])
        .expect("tool result bytes")
        .len();
    Arm {
        server,
        _cache: cache,
        tools,
        build: build_response["result"]["structuredContent"].clone(),
        initial_tool_result_bytes,
    }
}

fn evidence_by_id(values: &[Value]) -> BTreeMap<String, Value> {
    values
        .iter()
        .map(|value| {
            (
                value["evidence_id"]
                    .as_str()
                    .expect("evidence id")
                    .to_owned(),
                value.clone(),
            )
        })
        .collect()
}

#[test]
#[allow(clippy::too_many_lines)]
fn frozen_provider_free_progressive_structural_gate_passes() {
    let corpus: Corpus = serde_json::from_slice(CORPUS).expect("corpus");
    let gate: GateManifest = serde_json::from_slice(GATE).expect("gate manifest");
    assert_eq!(corpus.schema_version, "1.0.0");
    assert_eq!(gate.schema_version, "1.0.0");
    assert_eq!(
        gate.fixture_manifest,
        "evaluation/v1/structural-utility-manifest.json"
    );
    assert_eq!(gate.fixture_manifest_sha256, hash(CORPUS));
    assert_eq!(
        gate.delivery_modes,
        ["ordinary", "eager_structural", "progressive_structural"]
    );
    assert_eq!(
        gate.scripted_expansion_policy,
        "lookup_all_admitted_then_expand_every_returned_handle"
    );
    assert!(gate.requirements.identical_tool_definitions);
    assert!(gate.requirements.warm_cache_arm);
    assert!(gate.requirements.progressive_initial_smaller_than_eager);
    assert!(gate.requirements.preserve_ordinary_anchors);
    assert!(
        gate.requirements
            .full_expansion_equals_eager_structural_evidence
    );
    assert!(gate.requirements.source_immutable);
    assert_eq!(gate.requirements.provider_calls, 0);
    assert!(corpus.fixtures.len() >= 6);

    for fixture in &corpus.fixtures {
        assert!(matches!(
            fixture.split.as_str(),
            "development" | "validation" | "heldout"
        ));
        assert!(!fixture.selected_symbol.is_empty());
        assert!(fixture.minimum_structural_evidence >= 1);
        let source = Root::new(&fixture.id);
        let path = source.0.join(&fixture.path);
        fs::create_dir_all(path.parent().expect("source parent")).expect("source parent");
        fs::write(&path, fixture.source.as_bytes()).expect("source");
        let source_before = hash(&fs::read(&path).expect("source before"));

        let ordinary = run_arm(fixture, &source, DeliveryMode::Ordinary);
        let eager = run_arm(fixture, &source, DeliveryMode::EagerStructural);
        let mut progressive = run_arm(fixture, &source, DeliveryMode::ProgressiveStructural);
        let repeated_progressive = run_arm(fixture, &source, DeliveryMode::ProgressiveStructural);
        let warm_cache = Root::new("warm-cache");
        let warm_seed = run_arm_with_cache(
            fixture,
            &source,
            DeliveryMode::EagerStructural,
            warm_cache,
            "warmseed",
            10,
        );
        let Arm {
            server: warm_seed_server,
            _cache: warm_cache,
            tools: warm_seed_tools,
            build: warm_seed_build,
            initial_tool_result_bytes: _,
        } = warm_seed;
        drop(warm_seed_server);
        drop(warm_seed_tools);
        drop(warm_seed_build);
        let warm_reuse = run_arm_with_cache(
            fixture,
            &source,
            DeliveryMode::EagerStructural,
            warm_cache,
            "warmreuse",
            20,
        );
        assert_eq!(
            evidence_by_id(
                warm_reuse.build["packet"]["observed_evidence"]
                    .as_array()
                    .expect("warm evidence")
            ),
            evidence_by_id(
                eager.build["packet"]["observed_evidence"]
                    .as_array()
                    .expect("fresh eager evidence")
            ),
            "{} warm-cache evidence",
            fixture.id
        );
        assert_eq!(ordinary.tools, eager.tools, "{} eager tools", fixture.id);
        assert_eq!(
            ordinary.tools, progressive.tools,
            "{} progressive tools",
            fixture.id
        );
        assert_eq!(
            progressive.build["disclosure_map"], repeated_progressive.build["disclosure_map"],
            "{} deterministic disclosure map",
            fixture.id
        );
        assert_eq!(
            progressive.build["receipt"]["receipt_id"],
            repeated_progressive.build["receipt"]["receipt_id"],
            "{} deterministic map receipt",
            fixture.id
        );
        assert!(
            progressive.initial_tool_result_bytes < eager.initial_tool_result_bytes,
            "{} progressive={} eager={}",
            fixture.id,
            progressive.initial_tool_result_bytes,
            eager.initial_tool_result_bytes
        );

        let ordinary_evidence = ordinary.build["packet"]["observed_evidence"]
            .as_array()
            .expect("ordinary evidence");
        let progressive_anchors = progressive.build["initial_packet"]["observed_evidence"]
            .as_array()
            .expect("progressive anchors");
        assert_eq!(
            progressive_anchors, ordinary_evidence,
            "{} anchors",
            fixture.id
        );
        let eager_structural = eager.build["packet"]["observed_evidence"]
            .as_array()
            .expect("eager evidence")
            .iter()
            .filter(|value| value["extraction"]["method"] == "structural_graph_edge")
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            eager_structural.len() >= fixture.minimum_structural_evidence,
            "{} eager structural evidence",
            fixture.id
        );

        let lookup_response = exchange(
            &mut progressive.server,
            &[
                json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"context_disclosure_lookup","arguments":{
                    "session_id":"session_progressive01",
                    "handle":progressive.build["disclosure_map"]["map_id"],
                    "relation_kinds":["all_admitted"],
                    "max_items":"100", "max_depth":"1", "max_bytes":"131072"
                }}}),
            ],
        );
        let lookup = &lookup_response[0]["result"]["structuredContent"];
        assert_eq!(
            lookup_response[0]["result"]["isError"], false,
            "{lookup_response:?}"
        );
        let lookup_bytes = serde_json::to_vec(&lookup_response[0]["result"])
            .expect("lookup bytes")
            .len();
        assert_eq!(
            lookup["receipt"]["per_call"]["serialized_response_bytes"],
            lookup_bytes
        );

        let handles = lookup["items"]
            .as_array()
            .expect("lookup items")
            .iter()
            .map(|item| item["evidence_handle"].as_str().expect("handle").to_owned())
            .collect::<Vec<_>>();
        let mut expanded = Vec::new();
        let mut final_receipt = None;
        for (index, handle) in handles.iter().enumerate() {
            let sequence = 10 + index;
            let responses = exchange(
                &mut progressive.server,
                &[
                    json!({"jsonrpc":"2.0","id":sequence,"method":"tools/call","params":{"name":"context_evidence_expand","arguments":{
                        "request_id":format!("req_expand{sequence:08}"),
                        "event_id":format!("evt_expand{sequence:08}"),
                        "purpose":"bug_investigation",
                        "occurred_at":format!("2026-09-01T00:01:{index:02}Z"),
                        "session_id":"session_progressive01", "evidence_handle":handle,
                        "before_bytes":"0", "after_bytes":"0", "max_bytes":"512"
                    }}}),
                ],
            );
            assert_eq!(
                responses[0]["result"]["isError"], false,
                "{} {responses:?}",
                fixture.id
            );
            let result = &responses[0]["result"]["structuredContent"];
            assert_eq!(result["state"], "ready", "{result:?}");
            expanded.push(result["evidence"].clone());
            final_receipt = Some(result["receipt"].clone());
        }
        assert_eq!(
            evidence_by_id(&expanded),
            evidence_by_id(&eager_structural),
            "{} eager/progressive evidence",
            fixture.id
        );
        let final_receipt = final_receipt.expect("at least one expansion");
        assert_eq!(
            final_receipt["cumulative"]["expansions"],
            u64::try_from(handles.len()).expect("handle count")
        );
        assert_eq!(
            source_before,
            hash(&fs::read(&path).expect("source after")),
            "{} source mutation",
            fixture.id
        );
    }
}
