// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Provider-free structural-seeding utility gate."]

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{PolicySubject, ResourceBudget, validate_packet};
use context_engine::{
    EngineConfig, LocalEngine, ProfiledContextPacket, RepositoryReadTelemetry, RequestContext,
    StructuralSeedRequest, TaskProfile,
};
use context_store::AuditRetention;
use context_structural::{
    FactClass, GRAPH_VERSION, GraphFileInput, PROTOCOL_VERSION, RESOLVER_VERSION, StructuralGraph,
    StructuralLanguage, WorkerPath, WorkerRequest, build_graph, process_request, validate_graph,
};
use context_workspace::{DiscoveryPolicy, PathIdentity};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const MANIFEST: &[u8] = include_bytes!("../../../evaluation/v1/structural-utility-manifest.json");
const MAX_PACKET_GROWTH_BYTES: usize = 8_192;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
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

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "impresari-structural-utility-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create test root");
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ArmResult {
    packet: ProfiledContextPacket,
    telemetry: RepositoryReadTelemetry,
    graph: Option<StructuralGraph>,
}

fn hash(bytes: &[u8]) -> String {
    let mut value = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(value, "{byte:02x}").expect("string write");
    }
    value
}

fn request(sequence: u64, purpose: &str) -> RequestContext {
    RequestContext {
        request_id: format!("req_{sequence:08}"),
        event_id: format!("evt_{sequence:08}"),
        subject: PolicySubject {
            caller_id: "provider_free_utility_gate".into(),
            role: "local_evaluator".into(),
            purpose: purpose.into(),
        },
        occurred_at: format!("2026-09-01T00:00:{sequence:02}Z"),
    }
}

fn budget() -> ResourceBudget {
    ResourceBudget::conservative(8_192, 20, 100, 128, 100, 16, 30_000, 536_870_912).expect("budget")
}

fn grammar_version(language: StructuralLanguage) -> &'static str {
    match language {
        StructuralLanguage::TypeScript => "tree-sitter-typescript-0.23.2",
        StructuralLanguage::Rust => "tree-sitter-rust-0.24.2",
        StructuralLanguage::Ruby => "tree-sitter-ruby-0.23.1",
        _ => unreachable!("closed utility manifest language"),
    }
}

fn graph(fixture: &Fixture, snapshot_id: &str) -> StructuralGraph {
    let source = fixture.source.as_bytes();
    let path = PathIdentity::from_portable_relative_path(&fixture.path)
        .expect("portable structural fixture path");
    let request = WorkerRequest {
        schema_name: "structural-worker-request".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: format!("req_structural_{}", fixture.id),
        language: fixture.language,
        path: WorkerPath {
            display_path: path.display_path,
            platform_family: path.platform_family.into(),
            unit_encoding: path.unit_encoding.into(),
            relative_units_base64url: path.relative_units_base64url,
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
    let response = process_request(&request).expect("structural worker");
    let graph = build_graph(
        snapshot_id,
        vec![GraphFileInput {
            path: request.path,
            response,
        }],
    )
    .expect("graph");
    validate_graph(&graph).expect("valid graph");
    graph
}

fn run_arm(
    fixture: &Fixture,
    source_root: &TestRoot,
    cache_root: &TestRoot,
    seeded: bool,
) -> ArmResult {
    let config = EngineConfig {
        cache_root: cache_root.0.clone(),
        discovery: DiscoveryPolicy::new(10, 8_192, 8_192, 8).expect("discovery"),
        audit_retention: AuditRetention::new("2026-09-01T00:00:00Z", 20, 1_048_576)
            .expect("retention"),
    };
    let (mut engine, _) =
        LocalEngine::open(config, &request(1, "structural_utility"), &source_root.0).expect("open");
    let snapshot = engine
        .build_snapshot(&request(2, "structural_utility"), budget())
        .expect("snapshot");
    assert_eq!(snapshot.completeness, "complete");
    let graph = seeded.then(|| graph(fixture, &snapshot.snapshot_id));
    let packet = match &graph {
        Some(graph) => engine
            .build_profiled_seeded_structural_context(
                &request(3, "structural_utility"),
                TaskProfile::BugInvestigation,
                &fixture.task,
                &StructuralSeedRequest {
                    graph: graph.clone(),
                    edge_kinds: fixture.edge_kinds.clone(),
                },
                budget(),
            )
            .expect("seeded packet"),
        None => engine
            .build_profiled_context(
                &request(3, "structural_utility"),
                TaskProfile::BugInvestigation,
                &fixture.task,
                budget(),
            )
            .expect("baseline packet"),
    };
    validate_packet(&packet.packet).expect("valid packet");
    let telemetry = engine.repository_read_telemetry();
    assert!(telemetry.complete, "complete product read telemetry");
    ArmResult {
        packet,
        telemetry,
        graph,
    }
}

#[test]
#[allow(clippy::too_many_lines)]
fn frozen_provider_free_structural_utility_gate_passes() {
    let manifest: Manifest = serde_json::from_slice(MANIFEST).expect("manifest");
    assert_eq!(manifest.schema_version, "1.0.0");
    assert!(manifest.fixtures.len() >= 6);
    assert!(
        manifest
            .fixtures
            .iter()
            .filter(|fixture| fixture.split == "heldout")
            .count()
            * 4
            >= manifest.fixtures.len()
    );
    let mut languages = manifest
        .fixtures
        .iter()
        .map(|fixture| fixture.language)
        .collect::<Vec<_>>();
    languages.sort_by_key(|language| format!("{language:?}"));
    languages.dedup();
    assert!(languages.len() >= 3);
    let mut total_structural_evidence = 0_usize;
    let mut total_added_reads = 0_u64;
    let mut total_added_repeats = 0_u64;
    let mut total_added_source_bytes = 0_u64;
    let mut maximum_packet_growth = 0_usize;

    for fixture in &manifest.fixtures {
        let source_root = TestRoot::new(&fixture.id);
        let source_path = source_root.0.join(&fixture.path);
        fs::create_dir_all(source_path.parent().expect("source parent")).expect("source parent");
        fs::write(&source_path, fixture.source.as_bytes()).expect("source");
        let source_before = hash(&fs::read(&source_path).expect("source before"));
        let baseline_cache = TestRoot::new("baseline-cache");
        let seeded_cache = TestRoot::new("seeded-cache");
        let repeated_cache = TestRoot::new("repeated-cache");

        let baseline = run_arm(fixture, &source_root, &baseline_cache, false);
        let seeded = run_arm(fixture, &source_root, &seeded_cache, true);
        let repeated = run_arm(fixture, &source_root, &repeated_cache, true);

        assert_eq!(
            baseline.telemetry.source_fingerprint_sha256,
            seeded.telemetry.source_fingerprint_sha256
        );
        let baseline_ids = baseline
            .packet
            .packet
            .observed_evidence
            .iter()
            .map(|evidence| evidence.evidence_id.as_str())
            .collect::<Vec<_>>();
        assert!(!baseline_ids.is_empty(), "{} exact anchors", fixture.id);
        let seeded_ids = seeded
            .packet
            .packet
            .observed_evidence
            .iter()
            .map(|evidence| evidence.evidence_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            &seeded_ids[..baseline_ids.len()],
            baseline_ids,
            "{} anchor order",
            fixture.id
        );
        let structural = &seeded.packet.packet.observed_evidence[baseline_ids.len()..];
        assert!(
            structural.len() >= fixture.minimum_structural_evidence,
            "{} structural novelty; traversal={:?}; evidence={:?}",
            fixture.id,
            seeded.packet.plan.structural_query,
            seeded.packet.packet.observed_evidence
        );
        assert!(structural.iter().all(|evidence| {
            evidence.extraction.method == "structural_graph_edge"
                && !baseline_ids.contains(&evidence.evidence_id.as_str())
        }));
        total_structural_evidence = total_structural_evidence
            .checked_add(structural.len())
            .expect("structural evidence total");
        let traversal = seeded
            .packet
            .plan
            .structural_query
            .as_ref()
            .expect("structural query");
        assert!(traversal.result.edges.len() >= structural.len());
        let selected = traversal
            .result
            .nodes
            .iter()
            .find(|node| node.node_id == traversal.result.start_node)
            .expect("selected node");
        assert_eq!(
            selected.name.as_deref(),
            Some(fixture.selected_symbol.as_str())
        );

        let baseline_bytes = serde_json::to_vec(&baseline.packet)
            .expect("baseline bytes")
            .len();
        let seeded_bytes = serde_json::to_vec(&seeded.packet)
            .expect("seeded bytes")
            .len();
        assert!(seeded_bytes >= baseline_bytes);
        let packet_growth = seeded_bytes - baseline_bytes;
        assert!(packet_growth <= MAX_PACKET_GROWTH_BYTES);
        maximum_packet_growth = maximum_packet_growth.max(packet_growth);

        let added_reads = seeded
            .telemetry
            .repository_file_reads
            .checked_sub(baseline.telemetry.repository_file_reads)
            .expect("added reads");
        let added_repeats = seeded
            .telemetry
            .repeated_repository_file_reads
            .checked_sub(baseline.telemetry.repeated_repository_file_reads)
            .expect("added repeated reads");
        let added_bytes = seeded
            .telemetry
            .source_bytes_read
            .checked_sub(baseline.telemetry.source_bytes_read)
            .expect("added source bytes");
        let traversed_edges = u64::try_from(traversal.result.edges.len()).expect("edge count");
        assert!(added_reads <= traversed_edges, "{} added reads", fixture.id);
        assert!(
            added_repeats <= traversed_edges,
            "{} added repeats",
            fixture.id
        );
        assert!(
            added_bytes
                <= traversed_edges
                    .checked_mul(u64::try_from(fixture.source.len()).expect("source size"))
                    .expect("byte ceiling"),
            "{} added bytes",
            fixture.id
        );
        total_added_reads = total_added_reads
            .checked_add(added_reads)
            .expect("added reads total");
        total_added_repeats = total_added_repeats
            .checked_add(added_repeats)
            .expect("added repeats total");
        total_added_source_bytes = total_added_source_bytes
            .checked_add(added_bytes)
            .expect("added source bytes total");

        assert_eq!(
            seeded.packet, repeated.packet,
            "{} deterministic packet",
            fixture.id
        );
        assert_eq!(
            seeded.telemetry, repeated.telemetry,
            "{} deterministic telemetry",
            fixture.id
        );
        assert_eq!(
            seeded.graph, repeated.graph,
            "{} deterministic graph",
            fixture.id
        );
        assert_eq!(
            source_before,
            hash(&fs::read(&source_path).expect("source after")),
            "{} source mutation",
            fixture.id
        );
    }
    eprintln!(
        "provider-free structural utility: fixtures={} languages={} new_structural_evidence={} added_reads={} added_repeats={} added_source_bytes={} max_profiled_packet_growth={}",
        manifest.fixtures.len(),
        languages.len(),
        total_structural_evidence,
        total_added_reads,
        total_added_repeats,
        total_added_source_bytes,
        maximum_packet_growth
    );
}
