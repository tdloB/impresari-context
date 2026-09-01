// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Frozen structural-intelligence release evaluation."]

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_structural::{
    FactClass, GRAPH_VERSION, GraphFileInput, PROTOCOL_VERSION, RESOLVER_VERSION,
    StructuralLanguage, WorkerPath, WorkerRequest, build_graph, process_request, repository_map,
    validate_graph,
};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

const MANIFEST: &[u8] = include_bytes!("../../../evaluation/v1/structural-manifest.json");

#[derive(Deserialize)]
struct Manifest {
    fixtures: Vec<Fixture>,
}
#[derive(Deserialize)]
struct Fixture {
    id: String,
    split: String,
    source: String,
    required_symbols: Vec<String>,
    required_edges: Vec<String>,
}

fn hash(bytes: &[u8]) -> String {
    let mut output = String::from("sha256:");
    for byte in Sha256::digest(bytes) {
        use std::fmt::Write as _;
        write!(output, "{byte:02x}").expect("string write");
    }
    output
}

fn request(fixture: &Fixture) -> WorkerRequest {
    let path = format!("src/{}.ts", fixture.id);
    WorkerRequest {
        schema_name: "structural-worker-request".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: format!("req_struct_{}", fixture.id),
        language: StructuralLanguage::TypeScript,
        path: WorkerPath {
            display_path: path.clone(),
            platform_family: "unix".into(),
            unit_encoding: "unix_bytes".into(),
            relative_units_base64url: URL_SAFE_NO_PAD.encode(path),
        },
        content_hash: hash(fixture.source.as_bytes()),
        source_base64url: URL_SAFE_NO_PAD.encode(fixture.source.as_bytes()),
        fact_classes: vec![
            FactClass::Declaration,
            FactClass::Contains,
            FactClass::Import,
            FactClass::Export,
            FactClass::Call,
            FactClass::Reference,
        ],
        max_facts: 1000,
        max_nesting_depth: 128,
        max_response_bytes: 1_048_576,
        parser_version: "tree-sitter-0.26.13".into(),
        grammar_version: "tree-sitter-typescript-0.23.2".into(),
        resolver_version: RESOLVER_VERSION.into(),
        graph_version: GRAPH_VERSION.into(),
    }
}

#[test]
fn frozen_structural_corpus_meets_release_thresholds() {
    let manifest: Manifest = serde_json::from_slice(MANIFEST).expect("manifest");
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
    let (mut required, mut found, mut deterministic, mut valid) =
        (0_usize, 0_usize, 0_usize, 0_usize);
    for fixture in &manifest.fixtures {
        let request = request(fixture);
        let response = process_request(&request).expect("parse");
        let input = GraphFileInput {
            path: request.path,
            response,
        };
        let snapshot = hash(format!("snapshot:{}", fixture.id).as_bytes());
        let first = build_graph(&snapshot, vec![input.clone()]).expect("graph");
        let second = build_graph(&snapshot, vec![input]).expect("repeat graph");
        deterministic += usize::from(first == second);
        valid += usize::from(validate_graph(&first).is_ok());
        let symbols = first
            .nodes
            .iter()
            .filter_map(|node| node.name.as_deref())
            .collect::<Vec<_>>();
        let edges = first
            .edges
            .iter()
            .map(|edge| edge.kind.as_str())
            .collect::<Vec<_>>();
        required += fixture.required_symbols.len() + fixture.required_edges.len();
        found += fixture
            .required_symbols
            .iter()
            .filter(|name| symbols.contains(&name.as_str()))
            .count();
        found += fixture
            .required_edges
            .iter()
            .filter(|kind| edges.contains(&kind.as_str()))
            .count();
        assert!(
            !repository_map(&first, 100)
                .expect("map")
                .directories
                .is_empty()
        );
        assert!(
            first
                .edges
                .iter()
                .filter(|edge| edge.resolution == "confirmed")
                .all(|edge| edge.target_node.is_some())
        );
    }
    assert_eq!(found, required, "frozen structural recall");
    assert_eq!(deterministic, manifest.fixtures.len());
    assert_eq!(valid, manifest.fixtures.len());
}
