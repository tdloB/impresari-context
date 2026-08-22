// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Structural worker protocol and deterministic TypeScript/JavaScript extraction."]

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fmt, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tree_sitter::{Language, Node, Parser};

/// Worker protocol version.
pub const PROTOCOL_VERSION: &str = "1.0.0";
/// Graph contract version.
pub const GRAPH_VERSION: &str = "1.0.0";
/// Resolver version.
pub const RESOLVER_VERSION: &str = "0.1.0";
/// Maximum accepted request frame size.
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
/// Maximum emitted response frame size.
pub const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;

/// Supported structural language.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralLanguage {
    /// TypeScript source.
    TypeScript,
    /// TypeScript with JSX.
    Tsx,
    /// JavaScript source.
    JavaScript,
    /// JavaScript with JSX.
    Jsx,
}

/// Requested structural fact classes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FactClass {
    /// A named declaration.
    Declaration,
    /// A declaration nested in another declaration.
    Contains,
    /// An ES module import.
    Import,
    /// An ES module export.
    Export,
    /// A syntax-confirmed call expression.
    Call,
}

/// Lossless capability-relative source path supplied by the control process.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerPath {
    /// Safe display path.
    pub display_path: String,
    /// Platform family used to interpret native units.
    pub platform_family: String,
    /// Encoding of the lossless relative units.
    pub unit_encoding: String,
    /// Base64url native relative path units.
    pub relative_units_base64url: String,
}

/// One bounded parser request.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    /// Schema discriminator.
    pub schema_name: String,
    /// Protocol version.
    pub schema_version: String,
    /// Opaque request identity.
    pub request_id: String,
    /// Source language.
    pub language: StructuralLanguage,
    /// Lossless relative source path.
    pub path: WorkerPath,
    /// SHA-256 identity of the raw source bytes.
    pub content_hash: String,
    /// Base64url raw source bytes.
    pub source_base64url: String,
    /// Fact classes requested by the control process.
    pub fact_classes: Vec<FactClass>,
    /// Maximum returned facts.
    pub max_facts: u32,
    /// Maximum syntax traversal depth.
    pub max_nesting_depth: u32,
    /// Maximum serialized response bytes.
    pub max_response_bytes: u32,
    /// Expected parser runtime version.
    pub parser_version: String,
    /// Expected grammar version.
    pub grammar_version: String,
    /// Expected resolver version.
    pub resolver_version: String,
    /// Expected graph contract version.
    pub graph_version: String,
}

/// Provenance attached to each structural fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FactProvenance {
    /// Extraction mechanism.
    pub method: String,
    /// Parser runtime version.
    pub parser_version: String,
    /// Grammar identifier and version.
    pub grammar_version: String,
    /// Project-owned resolver version.
    pub resolver_version: String,
    /// Graph contract version.
    pub graph_version: String,
}

/// One deterministic source-derived structural fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralFact {
    /// Fact class.
    pub class: FactClass,
    /// Stable key local to this source artifact.
    pub local_key: String,
    /// Syntax kind that produced the fact.
    pub syntax_kind: String,
    /// Optional declared or imported/exported name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional module specifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Start byte in raw source.
    pub start_byte: u64,
    /// Exclusive end byte in raw source.
    pub end_byte: u64,
    /// Optional containing fact key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_key: Option<String>,
    /// `confirmed` or `heuristic`.
    pub confidence: String,
    /// Exact extraction provenance.
    pub provenance: FactProvenance,
}

/// Successful complete worker result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerSuccess {
    /// Schema discriminator.
    pub schema_name: String,
    /// Protocol version.
    pub schema_version: String,
    /// Request identity.
    pub request_id: String,
    /// Echoed content identity.
    pub content_hash: String,
    /// Whether the syntax tree contains recovery/error nodes.
    pub syntax_errors: bool,
    /// Complete deterministically ordered fact collection.
    pub facts: Vec<StructuralFact>,
    /// Explicit bounded warnings.
    pub warnings: Vec<String>,
}

/// Canonical structural graph node.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNode {
    /// Content-derived node identity.
    pub node_id: String,
    /// `file` or `symbol`.
    pub kind: String,
    /// Lossless source path.
    pub path: WorkerPath,
    /// Optional source-derived name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional source span for symbol nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<GraphSpan>,
    /// Extraction confidence.
    pub confidence: String,
    /// Resolver provenance.
    pub provenance: FactProvenance,
}

/// Byte-authoritative graph source span.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphSpan {
    /// Inclusive starting byte.
    pub start_byte: u64,
    /// Exclusive ending byte.
    pub end_byte: u64,
}

/// Canonical structural graph edge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEdge {
    /// Content-derived edge identity.
    pub edge_id: String,
    /// `declares`, `contains`, `imports`, or `exports`.
    pub kind: String,
    /// Source node identity.
    pub source_node: String,
    /// Target node identity when locally resolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_node: Option<String>,
    /// Module specifier for import/export relationships.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// `confirmed`, `heuristic`, `unresolved`, or `unsupported`.
    pub resolution: String,
    /// Exact source evidence span.
    pub span: GraphSpan,
    /// Resolver provenance.
    pub provenance: FactProvenance,
}

/// One validated file result accepted for graph promotion.
#[derive(Clone, Debug)]
pub struct GraphFileInput {
    /// Lossless path used for the worker request.
    pub path: WorkerPath,
    /// Complete validated worker response.
    pub response: WorkerSuccess,
}

/// Immutable canonical structural graph bound to one workspace snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralGraph {
    /// Schema discriminator.
    pub schema_name: String,
    /// Graph contract version.
    pub schema_version: String,
    /// Content-derived graph identity.
    pub graph_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Completeness relative to supported eligible artifacts.
    pub completeness: String,
    /// Deterministically ordered nodes.
    pub nodes: Vec<GraphNode>,
    /// Deterministically ordered edges.
    pub edges: Vec<GraphEdge>,
    /// Explicit unresolved/unsupported conditions.
    pub unknowns: Vec<String>,
}

/// One bounded, deterministic traversal of a structural graph.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralQueryResult {
    /// Schema discriminator.
    pub schema_name: String,
    /// Graph contract version.
    pub schema_version: String,
    /// Exact graph identity queried.
    pub graph_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact starting node identity.
    pub start_node: String,
    /// Traversed nodes in deterministic discovery order.
    pub nodes: Vec<GraphNode>,
    /// Traversed edges in deterministic discovery order.
    pub edges: Vec<GraphEdge>,
    /// Whether a declared traversal limit stopped expansion.
    pub truncated: bool,
    /// Explicit unavailable or unresolved states observed during traversal.
    pub unknowns: Vec<String>,
}

/// Traverses resolved outbound graph relationships within hard node, edge, and depth limits.
///
/// Empty `edge_kinds` permits every relationship kind. Unresolved relationships are
/// reported as unknowns but never invented as graph targets.
///
/// # Errors
///
/// Returns an error when graph identity/state is malformed, the start node does not
/// exist, a requested edge kind is invalid, or a hard limit is zero.
pub fn query_graph(
    graph: &StructuralGraph,
    start_node: &str,
    edge_kinds: &[String],
    max_depth: u32,
    max_nodes: u32,
    max_edges: u32,
) -> Result<StructuralQueryResult, StructuralError> {
    validate_graph_for_query(graph)?;
    if !valid_sha256(start_node) || max_nodes == 0 || max_edges == 0 {
        return Err(StructuralError::InvalidRequest);
    }
    let allowed = edge_kinds
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if allowed.iter().any(|kind| !valid_edge_kind(kind)) {
        return Err(StructuralError::InvalidRequest);
    }
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.node_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    if !nodes_by_id.contains_key(start_node) {
        return Err(StructuralError::InvalidRequest);
    }
    let mut outgoing: BTreeMap<&str, Vec<&GraphEdge>> = BTreeMap::new();
    for edge in &graph.edges {
        if allowed.is_empty() || allowed.contains(edge.kind.as_str()) {
            outgoing
                .entry(edge.source_node.as_str())
                .or_default()
                .push(edge);
        }
    }
    for edges in outgoing.values_mut() {
        edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    }

    let mut queue = VecDeque::from([(start_node.to_owned(), 0_u32)]);
    let mut visited = BTreeSet::from([start_node.to_owned()]);
    let mut selected_nodes = vec![nodes_by_id[start_node].clone()];
    let mut selected_edges = Vec::new();
    let mut unknowns = Vec::new();
    let mut truncated = false;
    while let Some((node_id, depth)) = queue.pop_front() {
        let Some(edges) = outgoing.get(node_id.as_str()) else {
            continue;
        };
        if depth >= max_depth {
            if !edges.is_empty() {
                truncated = true;
                unknowns.push("traversal_depth_limit_reached".into());
            }
            continue;
        }
        for edge in edges {
            if selected_edges.len()
                >= usize::try_from(max_edges).map_err(|_| StructuralError::ResourceLimit)?
            {
                truncated = true;
                unknowns.push("traversal_edge_limit_reached".into());
                break;
            }
            let Some(target) = edge.target_node.as_deref() else {
                selected_edges.push((*edge).clone());
                unknowns.push("unresolved_traversal_target".into());
                continue;
            };
            let Some(target_node) = nodes_by_id.get(target) else {
                return Err(StructuralError::ContractMismatch);
            };
            if !visited.contains(target) {
                if selected_nodes.len()
                    >= usize::try_from(max_nodes).map_err(|_| StructuralError::ResourceLimit)?
                {
                    truncated = true;
                    unknowns.push("traversal_node_limit_reached".into());
                    continue;
                }
                visited.insert(target.to_owned());
                selected_nodes.push((*target_node).clone());
                queue.push_back((target.to_owned(), depth.saturating_add(1)));
            }
            selected_edges.push((*edge).clone());
        }
    }
    unknowns.sort();
    unknowns.dedup();
    Ok(StructuralQueryResult {
        schema_name: "structural-query-result".into(),
        schema_version: GRAPH_VERSION.into(),
        graph_id: graph.graph_id.clone(),
        workspace_snapshot: graph.workspace_snapshot.clone(),
        start_node: start_node.into(),
        nodes: selected_nodes,
        edges: selected_edges,
        truncated,
        unknowns,
    })
}

fn validate_graph_for_query(graph: &StructuralGraph) -> Result<(), StructuralError> {
    if graph.schema_name != "structural-graph"
        || graph.schema_version != GRAPH_VERSION
        || !valid_sha256(&graph.graph_id)
        || !valid_sha256(&graph.workspace_snapshot)
        || graph.nodes.iter().any(|node| !valid_sha256(&node.node_id))
        || graph.edges.iter().any(|edge| {
            !valid_sha256(&edge.edge_id)
                || !valid_sha256(&edge.source_node)
                || edge
                    .target_node
                    .as_ref()
                    .is_some_and(|target| !valid_sha256(target))
        })
    {
        return Err(StructuralError::ContractMismatch);
    }
    Ok(())
}

fn valid_edge_kind(kind: &str) -> bool {
    matches!(
        kind,
        "declares" | "contains" | "imports" | "exports" | "calls"
    )
}

/// Builds one deterministic graph from fully validated worker results.
///
/// # Errors
///
/// Returns an error for malformed snapshot identity, duplicate paths/facts,
/// inconsistent provenance, invalid spans, or canonicalization failure.
pub fn build_graph(
    workspace_snapshot: &str,
    files: Vec<GraphFileInput>,
) -> Result<StructuralGraph, StructuralError> {
    build_graph_with_unknowns(workspace_snapshot, files, Vec::new())
}

/// Builds a graph while preserving caller-observed unsupported or excluded states.
///
/// # Errors
///
/// Returns the same validation failures as [`build_graph`].
pub fn build_graph_with_unknowns(
    workspace_snapshot: &str,
    mut files: Vec<GraphFileInput>,
    mut unknowns: Vec<String>,
) -> Result<StructuralGraph, StructuralError> {
    if !valid_sha256(workspace_snapshot) {
        return Err(StructuralError::ContractMismatch);
    }
    files.sort_by(|left, right| {
        left.path
            .relative_units_base64url
            .cmp(&right.path.relative_units_base64url)
    });
    if files
        .windows(2)
        .any(|pair| pair[0].path.relative_units_base64url == pair[1].path.relative_units_base64url)
    {
        return Err(StructuralError::ContractMismatch);
    }
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut file_nodes = BTreeMap::new();
    for file in &files {
        let file_id = add_file_node(workspace_snapshot, file, &mut nodes)?;
        if file_nodes
            .insert(file.path.display_path.clone(), file_id)
            .is_some()
        {
            return Err(StructuralError::ContractMismatch);
        }
    }
    for file in files {
        promote_file(
            workspace_snapshot,
            &file,
            &file_nodes,
            &mut nodes,
            &mut edges,
            &mut unknowns,
        )?;
    }
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    unknowns.sort();
    unknowns.dedup();
    let completeness = if unknowns.is_empty() {
        "complete"
    } else {
        "partial"
    };
    let graph_id = graph_identity(
        "structural-graph",
        &serde_json::json!({
            "workspace_snapshot": workspace_snapshot,
            "completeness": completeness,
            "nodes": &nodes,
            "edges": &edges,
            "unknowns": &unknowns,
        }),
    )?;
    Ok(StructuralGraph {
        schema_name: "structural-graph".into(),
        schema_version: GRAPH_VERSION.into(),
        graph_id,
        workspace_snapshot: workspace_snapshot.into(),
        completeness: completeness.into(),
        nodes,
        edges,
        unknowns,
    })
}

fn promote_file(
    workspace_snapshot: &str,
    file: &GraphFileInput,
    file_nodes: &BTreeMap<String, String>,
    nodes: &mut Vec<GraphNode>,
    edges: &mut Vec<GraphEdge>,
    unknowns: &mut Vec<String>,
) -> Result<(), StructuralError> {
    let file_id = file_nodes
        .get(&file.path.display_path)
        .ok_or(StructuralError::ContractMismatch)?;
    let local_nodes = add_declarations(workspace_snapshot, file, file_id, nodes, edges)?;
    add_relationships(
        workspace_snapshot,
        file,
        file_id,
        file_nodes,
        &local_nodes,
        edges,
        unknowns,
    )?;
    if file.response.syntax_errors {
        unknowns.push("syntax_recovery_present".into());
    }
    Ok(())
}

fn add_file_node(
    workspace_snapshot: &str,
    file: &GraphFileInput,
    nodes: &mut Vec<GraphNode>,
) -> Result<String, StructuralError> {
    let file_id = graph_identity(
        "graph-node",
        &serde_json::json!({
            "workspace_snapshot": workspace_snapshot,
            "path": &file.path,
            "kind": "file"
        }),
    )?;
    let provenance = file
        .response
        .facts
        .first()
        .map_or_else(default_provenance, |fact| fact.provenance.clone());
    nodes.push(GraphNode {
        node_id: file_id.clone(),
        kind: "file".into(),
        path: file.path.clone(),
        name: None,
        span: None,
        confidence: "confirmed".into(),
        provenance,
    });
    Ok(file_id)
}

fn add_declarations(
    workspace_snapshot: &str,
    file: &GraphFileInput,
    file_id: &str,
    nodes: &mut Vec<GraphNode>,
    edges: &mut Vec<GraphEdge>,
) -> Result<std::collections::BTreeMap<String, String>, StructuralError> {
    let mut local_nodes = std::collections::BTreeMap::new();
    for fact in &file.response.facts {
        if fact.provenance.graph_version != GRAPH_VERSION || fact.end_byte < fact.start_byte {
            return Err(StructuralError::ContractMismatch);
        }
        if fact.class != FactClass::Declaration {
            continue;
        }
        let node_id = graph_identity(
            "graph-node",
            &serde_json::json!({
                "workspace_snapshot": workspace_snapshot, "path": &file.path,
                "local_key": &fact.local_key, "kind": "symbol"
            }),
        )?;
        if local_nodes
            .insert(fact.local_key.clone(), node_id.clone())
            .is_some()
        {
            return Err(StructuralError::ContractMismatch);
        }
        nodes.push(GraphNode {
            node_id: node_id.clone(),
            kind: "symbol".into(),
            path: file.path.clone(),
            name: fact.name.clone(),
            span: Some(GraphSpan {
                start_byte: fact.start_byte,
                end_byte: fact.end_byte,
            }),
            confidence: fact.confidence.clone(),
            provenance: fact.provenance.clone(),
        });
        edges.push(graph_edge(
            workspace_snapshot,
            "declares",
            file_id,
            Some(&node_id),
            None,
            "confirmed",
            fact,
        )?);
    }
    Ok(local_nodes)
}

fn add_relationships(
    workspace_snapshot: &str,
    file: &GraphFileInput,
    file_id: &str,
    file_nodes: &BTreeMap<String, String>,
    local_nodes: &std::collections::BTreeMap<String, String>,
    edges: &mut Vec<GraphEdge>,
    unknowns: &mut Vec<String>,
) -> Result<(), StructuralError> {
    for fact in &file.response.facts {
        match fact.class {
            FactClass::Contains => {
                add_containment(workspace_snapshot, fact, local_nodes, edges, unknowns)?;
            }
            FactClass::Import => {
                let target = fact.module.as_deref().and_then(|module| {
                    resolve_relative_module(&file.path.display_path, module, file_nodes)
                });
                edges.push(graph_edge(
                    workspace_snapshot,
                    "imports",
                    file_id,
                    target,
                    fact.module.as_deref(),
                    if target.is_some() {
                        "confirmed"
                    } else {
                        "unresolved"
                    },
                    fact,
                )?);
                if target.is_none() {
                    unknowns.push("unresolved_module_import".into());
                }
            }
            FactClass::Export => {
                let target = fact.name.as_ref().and_then(|name| {
                    file.response
                        .facts
                        .iter()
                        .find(|candidate| {
                            candidate.class == FactClass::Declaration
                                && candidate.name.as_ref() == Some(name)
                        })
                        .and_then(|candidate| local_nodes.get(&candidate.local_key))
                });
                edges.push(graph_edge(
                    workspace_snapshot,
                    "exports",
                    file_id,
                    target,
                    fact.module.as_deref(),
                    if target.is_some() {
                        "confirmed"
                    } else {
                        "unresolved"
                    },
                    fact,
                )?);
                if target.is_none() {
                    unknowns.push("unresolved_export_target".into());
                }
            }
            FactClass::Call => {
                let source = fact
                    .parent_key
                    .as_ref()
                    .and_then(|key| local_nodes.get(key))
                    .map_or(file_id, String::as_str);
                let target = fact.name.as_ref().and_then(|name| {
                    file.response
                        .facts
                        .iter()
                        .find(|candidate| {
                            candidate.class == FactClass::Declaration
                                && candidate.name.as_ref() == Some(name)
                        })
                        .and_then(|candidate| local_nodes.get(&candidate.local_key))
                });
                edges.push(graph_edge(
                    workspace_snapshot,
                    "calls",
                    source,
                    target,
                    None,
                    if target.is_some() {
                        "heuristic"
                    } else {
                        "unresolved"
                    },
                    fact,
                )?);
                if target.is_none() {
                    unknowns.push("unresolved_call_target".into());
                }
            }
            FactClass::Declaration => {}
        }
    }
    Ok(())
}

fn resolve_relative_module<'a>(
    source_path: &str,
    module: &str,
    file_nodes: &'a BTreeMap<String, String>,
) -> Option<&'a String> {
    if !(module.starts_with("./") || module.starts_with("../"))
        || source_path.starts_with('/')
        || source_path.contains('\\')
        || module.contains('\\')
    {
        return None;
    }
    let mut components = source_path
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    components.pop()?;
    for component in module.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop()?;
            }
            value if value != "." && value != ".." => components.push(value.into()),
            _ => return None,
        }
    }
    let base = components.join("/");
    let has_supported_extension = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"]
        .iter()
        .any(|extension| base.ends_with(extension));
    let candidates = if has_supported_extension {
        vec![base]
    } else {
        [
            ".ts",
            ".tsx",
            ".js",
            ".jsx",
            ".mjs",
            ".cjs",
            "/index.ts",
            "/index.tsx",
            "/index.js",
            "/index.jsx",
            "/index.mjs",
            "/index.cjs",
        ]
        .iter()
        .map(|suffix| format!("{base}{suffix}"))
        .collect()
    };
    candidates
        .iter()
        .find_map(|candidate| file_nodes.get(candidate))
}

fn add_containment(
    workspace_snapshot: &str,
    fact: &StructuralFact,
    local_nodes: &std::collections::BTreeMap<String, String>,
    edges: &mut Vec<GraphEdge>,
    unknowns: &mut Vec<String>,
) -> Result<(), StructuralError> {
    let target = fact
        .parent_key
        .as_ref()
        .and_then(|parent| fact.local_key.strip_prefix(&format!("contains:{parent}:")))
        .and_then(|key| local_nodes.get(key));
    let source = fact
        .parent_key
        .as_ref()
        .and_then(|key| local_nodes.get(key));
    if let (Some(source), Some(target)) = (source, target) {
        edges.push(graph_edge(
            workspace_snapshot,
            "contains",
            source,
            Some(target),
            None,
            "confirmed",
            fact,
        )?);
    } else {
        unknowns.push("unresolved_local_containment".into());
    }
    Ok(())
}

fn graph_edge(
    workspace_snapshot: &str,
    kind: &str,
    source_node: &str,
    target_node: Option<&String>,
    module: Option<&str>,
    resolution: &str,
    fact: &StructuralFact,
) -> Result<GraphEdge, StructuralError> {
    let span = GraphSpan {
        start_byte: fact.start_byte,
        end_byte: fact.end_byte,
    };
    let edge_id = graph_identity(
        "graph-edge",
        &serde_json::json!({
            "workspace_snapshot": workspace_snapshot,
            "kind": kind,
            "source_node": source_node,
            "target_node": target_node,
            "module": module,
            "resolution": resolution,
            "span": &span,
            "provenance": &fact.provenance,
        }),
    )?;
    Ok(GraphEdge {
        edge_id,
        kind: kind.into(),
        source_node: source_node.into(),
        target_node: target_node.cloned(),
        module: module.map(str::to_owned),
        resolution: resolution.into(),
        span,
        provenance: fact.provenance.clone(),
    })
}

fn default_provenance() -> FactProvenance {
    FactProvenance {
        method: "tree_sitter_syntax".into(),
        parser_version: "tree-sitter-0.26.12".into(),
        grammar_version: "mixed-pinned-grammars".into(),
        resolver_version: RESOLVER_VERSION.into(),
        graph_version: GRAPH_VERSION.into(),
    }
}

fn graph_identity(kind: &str, value: &serde_json::Value) -> Result<String, StructuralError> {
    let canonical =
        serde_json_canonicalizer::to_vec(value).map_err(|_| StructuralError::InvalidRequest)?;
    let mut preimage = Vec::new();
    preimage.extend_from_slice(b"impresari-context\0");
    preimage.extend_from_slice(kind.as_bytes());
    preimage.extend_from_slice(b"\0");
    preimage.extend_from_slice(GRAPH_VERSION.as_bytes());
    preimage.extend_from_slice(b"\0");
    preimage.extend_from_slice(&canonical);
    Ok(sha256(&preimage))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// Safe worker failure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerFailure {
    /// Schema discriminator.
    pub schema_name: String,
    /// Protocol version.
    pub schema_version: String,
    /// Opaque request identity when it was safely decoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Stable error code.
    pub code: String,
    /// Whether a fresh bounded worker retry may succeed.
    pub retryable: bool,
    /// Bounded path/source-free message.
    pub message: String,
}

/// Worker response union.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum WorkerResponse {
    /// Complete success.
    Success(WorkerSuccess),
    /// Safe failure.
    Error(WorkerFailure),
}

/// Stable local structural failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuralError {
    /// Frame or request is malformed.
    InvalidRequest,
    /// Contract or identity does not match.
    ContractMismatch,
    /// Declared limit was exceeded.
    ResourceLimit,
    /// Parser failed without safe detail.
    ParserFailure,
    /// Input/output operation failed.
    Io,
    /// Worker executable identity was not the configured pinned artifact.
    WorkerIdentity,
    /// Worker exceeded its wall-time ceiling.
    Timeout,
    /// Worker exited abnormally or returned a safe error response.
    WorkerFailure,
}

impl fmt::Display for StructuralError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidRequest => "invalid structural request",
            Self::ContractMismatch => "structural contract mismatch",
            Self::ResourceLimit => "structural resource limit exceeded",
            Self::ParserFailure => "structural parser failed",
            Self::Io => "structural worker I/O failed",
            Self::WorkerIdentity => "structural worker identity mismatch",
            Self::Timeout => "structural worker timed out",
            Self::WorkerFailure => "structural worker failed",
        })
    }
}

/// Explicit capability-reduced worker launcher configuration.
#[derive(Clone, Debug)]
pub struct WorkerLauncher {
    /// Pinned worker executable path selected by the embedding distribution.
    pub executable: PathBuf,
    /// Expected SHA-256 of the exact executable bytes.
    pub expected_sha256: String,
    /// Existing empty, non-workspace directory used as the worker current directory.
    pub empty_working_directory: PathBuf,
    /// Hard wall-time ceiling.
    pub timeout: Duration,
}

impl WorkerLauncher {
    /// Starts a fresh worker, sends one request, and validates one complete response.
    ///
    /// # Errors
    ///
    /// Returns an error for executable identity mismatch, authority-reduction setup
    /// failure, timeout, abnormal exit, invalid framing, unsafe response, or an
    /// identity/span/limit mismatch. No partial facts are returned.
    pub fn execute(&self, request: &WorkerRequest) -> Result<WorkerSuccess, StructuralError> {
        validate_request(request)?;
        validate_launcher(self)?;
        let request_bytes =
            serde_json::to_vec(request).map_err(|_| StructuralError::InvalidRequest)?;
        if request_bytes.len() > MAX_REQUEST_BYTES {
            return Err(StructuralError::ResourceLimit);
        }
        let mut child = Command::new(&self.executable)
            .current_dir(&self.empty_working_directory)
            .env_clear()
            .env("LC_ALL", "C")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|_| StructuralError::Io)?;

        let mut stdin = child.stdin.take().ok_or(StructuralError::Io)?;
        write_frame(&mut stdin, &request_bytes, MAX_REQUEST_BYTES)?;
        drop(stdin);

        let stdout = child.stdout.take().ok_or(StructuralError::Io)?;
        let stderr = child.stderr.take().ok_or(StructuralError::Io)?;
        let stdout_limit = usize::try_from(request.max_response_bytes)
            .map_err(|_| StructuralError::ResourceLimit)?
            .min(MAX_RESPONSE_BYTES)
            .saturating_add(5);
        let stdout_reader = thread::spawn(move || read_capped(stdout, stdout_limit));
        let stderr_reader = thread::spawn(move || read_capped(stderr, 16 * 1024));

        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|_| StructuralError::Io)? {
                break status;
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(StructuralError::Timeout);
            }
            thread::sleep(Duration::from_millis(2));
        };
        let stdout = stdout_reader.join().map_err(|_| StructuralError::Io)??;
        let _stderr = stderr_reader.join().map_err(|_| StructuralError::Io)??;
        if !status.success() {
            return Err(StructuralError::WorkerFailure);
        }
        let maximum = usize::try_from(request.max_response_bytes)
            .map_err(|_| StructuralError::ResourceLimit)?
            .min(MAX_RESPONSE_BYTES);
        let payload = read_single_frame(&mut stdout.as_slice(), maximum)?;
        let response: WorkerResponse =
            serde_json::from_slice(&payload).map_err(|_| StructuralError::InvalidRequest)?;
        match response {
            WorkerResponse::Success(success) => {
                validate_success(&success, request)?;
                Ok(success)
            }
            WorkerResponse::Error(_) => Err(StructuralError::WorkerFailure),
        }
    }
}

fn validate_launcher(launcher: &WorkerLauncher) -> Result<(), StructuralError> {
    if launcher.timeout.is_zero() || launcher.timeout > Duration::from_secs(300) {
        return Err(StructuralError::ResourceLimit);
    }
    let executable = fs::metadata(&launcher.executable).map_err(|_| StructuralError::Io)?;
    if !executable.is_file() {
        return Err(StructuralError::WorkerIdentity);
    }
    let bytes = fs::read(&launcher.executable).map_err(|_| StructuralError::Io)?;
    if sha256(&bytes) != launcher.expected_sha256 {
        return Err(StructuralError::WorkerIdentity);
    }
    validate_empty_directory(&launcher.empty_working_directory)
}

fn validate_empty_directory(path: &Path) -> Result<(), StructuralError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| StructuralError::Io)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(StructuralError::InvalidRequest);
    }
    let mut entries = fs::read_dir(path).map_err(|_| StructuralError::Io)?;
    if entries
        .next()
        .transpose()
        .map_err(|_| StructuralError::Io)?
        .is_some()
    {
        return Err(StructuralError::InvalidRequest);
    }
    Ok(())
}

fn read_capped(mut reader: impl Read, maximum: usize) -> Result<Vec<u8>, StructuralError> {
    let maximum = u64::try_from(maximum).map_err(|_| StructuralError::ResourceLimit)?;
    let mut output = Vec::new();
    reader
        .by_ref()
        .take(maximum.saturating_add(1))
        .read_to_end(&mut output)
        .map_err(|_| StructuralError::Io)?;
    if u64::try_from(output.len()).map_err(|_| StructuralError::ResourceLimit)? > maximum {
        return Err(StructuralError::ResourceLimit);
    }
    Ok(output)
}

fn validate_success(
    success: &WorkerSuccess,
    request: &WorkerRequest,
) -> Result<(), StructuralError> {
    let source = URL_SAFE_NO_PAD
        .decode(&request.source_base64url)
        .map_err(|_| StructuralError::InvalidRequest)?;
    if success.schema_name != "structural-worker-response"
        || success.schema_version != PROTOCOL_VERSION
        || success.request_id != request.request_id
        || success.content_hash != request.content_hash
        || success.facts.len()
            > usize::try_from(request.max_facts).map_err(|_| StructuralError::ResourceLimit)?
    {
        return Err(StructuralError::ContractMismatch);
    }
    let mut previous = None;
    for fact in &success.facts {
        if !request.fact_classes.contains(&fact.class)
            || fact.provenance.parser_version != request.parser_version
            || fact.provenance.grammar_version != request.grammar_version
            || fact.provenance.resolver_version != request.resolver_version
            || fact.provenance.graph_version != request.graph_version
            || fact.end_byte < fact.start_byte
            || fact.end_byte
                > u64::try_from(source.len()).map_err(|_| StructuralError::ResourceLimit)?
            || !matches!(fact.confidence.as_str(), "confirmed" | "heuristic")
        {
            return Err(StructuralError::ContractMismatch);
        }
        let ordering = (
            fact.start_byte,
            fact.end_byte,
            fact.class,
            fact.local_key.as_str(),
        );
        if previous.is_some_and(|prior| prior > ordering) {
            return Err(StructuralError::ContractMismatch);
        }
        previous = Some(ordering);
    }
    Ok(())
}

impl std::error::Error for StructuralError {}

/// Reads exactly one length-prefixed frame and requires EOF afterward.
///
/// # Errors
///
/// Returns an error for malformed length, truncation, trailing bytes, I/O, or
/// a frame exceeding `maximum`.
pub fn read_single_frame(
    reader: &mut impl Read,
    maximum: usize,
) -> Result<Vec<u8>, StructuralError> {
    let mut length = [0_u8; 4];
    reader
        .read_exact(&mut length)
        .map_err(|_| StructuralError::Io)?;
    let size =
        usize::try_from(u32::from_be_bytes(length)).map_err(|_| StructuralError::ResourceLimit)?;
    if size == 0 || size > maximum {
        return Err(StructuralError::ResourceLimit);
    }
    let mut payload = vec![0_u8; size];
    reader
        .read_exact(&mut payload)
        .map_err(|_| StructuralError::Io)?;
    let mut trailing = [0_u8; 1];
    match reader.read(&mut trailing) {
        Ok(0) => Ok(payload),
        Ok(_) => Err(StructuralError::InvalidRequest),
        Err(_) => Err(StructuralError::Io),
    }
}

/// Writes one length-prefixed frame.
///
/// # Errors
///
/// Returns an error for output above `maximum`, conversion failure, or I/O.
pub fn write_frame(
    writer: &mut impl Write,
    payload: &[u8],
    maximum: usize,
) -> Result<(), StructuralError> {
    if payload.is_empty() || payload.len() > maximum {
        return Err(StructuralError::ResourceLimit);
    }
    let length = u32::try_from(payload.len()).map_err(|_| StructuralError::ResourceLimit)?;
    writer
        .write_all(&length.to_be_bytes())
        .map_err(|_| StructuralError::Io)?;
    writer.write_all(payload).map_err(|_| StructuralError::Io)?;
    writer.flush().map_err(|_| StructuralError::Io)
}

/// Validates and processes one request without filesystem or network access.
///
/// # Errors
///
/// Returns a stable error when the request is invalid, exceeds limits, or the
/// parser cannot be configured.
pub fn process_request(request: &WorkerRequest) -> Result<WorkerSuccess, StructuralError> {
    validate_request(request)?;
    let source = URL_SAFE_NO_PAD
        .decode(&request.source_base64url)
        .map_err(|_| StructuralError::InvalidRequest)?;
    if sha256(&source) != request.content_hash {
        return Err(StructuralError::ContractMismatch);
    }
    let language = language(request.language);
    let mut parser = Parser::new();
    parser
        .set_language(&language)
        .map_err(|_| StructuralError::ParserFailure)?;
    let tree = parser
        .parse(&source, None)
        .ok_or(StructuralError::ParserFailure)?;
    let provenance = provenance(request);
    let mut facts = Vec::new();
    let mut ancestors = Vec::new();
    visit(
        tree.root_node(),
        &source,
        request,
        &provenance,
        0,
        &mut ancestors,
        &mut facts,
    )?;
    facts.sort_by(|left, right| {
        (left.start_byte, left.end_byte, left.class, &left.local_key).cmp(&(
            right.start_byte,
            right.end_byte,
            right.class,
            &right.local_key,
        ))
    });
    Ok(WorkerSuccess {
        schema_name: "structural-worker-response".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: request.request_id.clone(),
        content_hash: request.content_hash.clone(),
        syntax_errors: tree.root_node().has_error(),
        facts,
        warnings: if tree.root_node().has_error() {
            vec!["syntax_recovery_present".into()]
        } else {
            Vec::new()
        },
    })
}

fn validate_request(request: &WorkerRequest) -> Result<(), StructuralError> {
    if request.schema_name != "structural-worker-request"
        || request.schema_version != PROTOCOL_VERSION
        || request.graph_version != GRAPH_VERSION
        || request.resolver_version != RESOLVER_VERSION
        || request.parser_version != "tree-sitter-0.26.12"
        || request.max_facts == 0
        || request.max_facts > 100_000
        || request.max_nesting_depth == 0
        || request.max_nesting_depth > 1024
        || request.max_response_bytes == 0
        || usize::try_from(request.max_response_bytes)
            .map_err(|_| StructuralError::ResourceLimit)?
            > MAX_RESPONSE_BYTES
        || request.request_id.len() < 8
        || request.request_id.len() > 128
        || request.path.display_path.is_empty()
        || request.path.display_path.len() > 4096
        || request.fact_classes.is_empty()
    {
        return Err(StructuralError::ContractMismatch);
    }
    let expected_grammar = match request.language {
        StructuralLanguage::TypeScript | StructuralLanguage::Tsx => "tree-sitter-typescript-0.23.2",
        StructuralLanguage::JavaScript | StructuralLanguage::Jsx => "tree-sitter-javascript-0.25.0",
    };
    if request.grammar_version != expected_grammar {
        return Err(StructuralError::ContractMismatch);
    }
    Ok(())
}

fn language(language: StructuralLanguage) -> Language {
    match language {
        StructuralLanguage::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        StructuralLanguage::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        StructuralLanguage::JavaScript | StructuralLanguage::Jsx => {
            tree_sitter_javascript::LANGUAGE.into()
        }
    }
}

fn provenance(request: &WorkerRequest) -> FactProvenance {
    FactProvenance {
        method: "tree_sitter_syntax".into(),
        parser_version: request.parser_version.clone(),
        grammar_version: request.grammar_version.clone(),
        resolver_version: request.resolver_version.clone(),
        graph_version: request.graph_version.clone(),
    }
}

fn visit(
    node: Node<'_>,
    source: &[u8],
    request: &WorkerRequest,
    provenance: &FactProvenance,
    depth: u32,
    ancestors: &mut Vec<String>,
    facts: &mut Vec<StructuralFact>,
) -> Result<(), StructuralError> {
    if depth > request.max_nesting_depth {
        return Err(StructuralError::ResourceLimit);
    }
    let parent = ancestors.last().cloned();
    let produced = fact_for_node(node, source, request, provenance, parent.as_deref());
    let mut pushed = false;
    if let Some(fact) = produced {
        if facts.len()
            >= usize::try_from(request.max_facts).map_err(|_| StructuralError::ResourceLimit)?
        {
            return Err(StructuralError::ResourceLimit);
        }
        let key = fact.local_key.clone();
        if parent.is_some()
            && request.fact_classes.contains(&FactClass::Contains)
            && fact.class == FactClass::Declaration
        {
            let contains = StructuralFact {
                class: FactClass::Contains,
                local_key: format!("contains:{}:{key}", parent.as_deref().unwrap_or_default()),
                syntax_kind: "containment".into(),
                name: fact.name.clone(),
                module: None,
                start_byte: fact.start_byte,
                end_byte: fact.end_byte,
                parent_key: parent.clone(),
                confidence: "confirmed".into(),
                provenance: provenance.clone(),
            };
            facts.push(contains);
        }
        if fact.class == FactClass::Declaration {
            ancestors.push(key);
            pushed = true;
        }
        facts.push(fact);
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        visit(
            child,
            source,
            request,
            provenance,
            depth + 1,
            ancestors,
            facts,
        )?;
    }
    if pushed {
        ancestors.pop();
    }
    Ok(())
}

fn fact_for_node(
    node: Node<'_>,
    source: &[u8],
    request: &WorkerRequest,
    provenance: &FactProvenance,
    parent: Option<&str>,
) -> Option<StructuralFact> {
    let kind = node.kind();
    let (class, name, module) = match kind {
        "function_declaration"
        | "class_declaration"
        | "interface_declaration"
        | "type_alias_declaration"
        | "enum_declaration"
        | "method_definition"
        | "abstract_method_signature" => {
            let name = node
                .child_by_field_name("name")
                .and_then(|value| text(value, source));
            (FactClass::Declaration, name, None)
        }
        "lexical_declaration" | "variable_declaration" => {
            let declarator = named_descendant(node, "variable_declarator")?;
            let name = declarator
                .child_by_field_name("name")
                .and_then(|value| text(value, source));
            (FactClass::Declaration, name, None)
        }
        "import_statement" => {
            let module = node
                .child_by_field_name("source")
                .and_then(|value| string_text(value, source));
            (FactClass::Import, None, module)
        }
        "export_statement" => {
            let module = node
                .child_by_field_name("source")
                .and_then(|value| string_text(value, source));
            let name = node
                .child_by_field_name("declaration")
                .and_then(|value| value.child_by_field_name("name"))
                .and_then(|value| text(value, source));
            (FactClass::Export, name, module)
        }
        "call_expression" => {
            let function = node.child_by_field_name("function")?;
            let name = if function.kind() == "identifier" {
                text(function, source)
            } else {
                None
            };
            (FactClass::Call, name, None)
        }
        _ => return None,
    };
    if !request.fact_classes.contains(&class) {
        return None;
    }
    let start = node.start_byte();
    let end = node.end_byte();
    Some(StructuralFact {
        class,
        local_key: format!("{}:{start}:{end}", class_name(class)),
        syntax_kind: kind.into(),
        name,
        module,
        start_byte: u64::try_from(start).ok()?,
        end_byte: u64::try_from(end).ok()?,
        parent_key: parent.map(str::to_owned),
        confidence: "confirmed".into(),
        provenance: provenance.clone(),
    })
}

fn named_descendant<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    node.named_children(&mut cursor)
        .find(|child| child.kind() == kind)
}

fn text(node: Node<'_>, source: &[u8]) -> Option<String> {
    std::str::from_utf8(source.get(node.byte_range())?)
        .ok()
        .map(str::to_owned)
}

fn string_text(node: Node<'_>, source: &[u8]) -> Option<String> {
    text(node, source).map(|value| value.trim_matches(['\'', '"']).to_owned())
}

const fn class_name(class: FactClass) -> &'static str {
    match class {
        FactClass::Declaration => "declaration",
        FactClass::Contains => "contains",
        FactClass::Import => "import",
        FactClass::Export => "export",
        FactClass::Call => "call",
    }
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let hex = digest
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            use fmt::Write as _;
            write!(output, "{byte:02x}").expect("writing to a string cannot fail");
            output
        });
    format!("sha256:{hex}")
}

/// Runs one worker transaction over standard I/O.
///
/// # Errors
///
/// Returns only for transport failures; request failures are encoded safely.
pub fn run_stdio() -> Result<(), StructuralError> {
    let payload = read_single_frame(&mut io::stdin().lock(), MAX_REQUEST_BYTES)?;
    let request: Result<WorkerRequest, _> = serde_json::from_slice(&payload);
    let (response, maximum) = match request {
        Ok(request) => {
            let maximum = usize::try_from(request.max_response_bytes)
                .unwrap_or(MAX_RESPONSE_BYTES)
                .min(MAX_RESPONSE_BYTES);
            let response = match process_request(&request) {
                Ok(success) => WorkerResponse::Success(success),
                Err(error) => WorkerResponse::Error(failure(Some(request.request_id), error)),
            };
            (response, maximum)
        }
        Err(_) => (
            WorkerResponse::Error(failure(None, StructuralError::InvalidRequest)),
            MAX_RESPONSE_BYTES,
        ),
    };
    let bytes = serde_json::to_vec(&response).map_err(|_| StructuralError::InvalidRequest)?;
    write_frame(&mut io::stdout().lock(), &bytes, maximum)
}

fn failure(request_id: Option<String>, error: StructuralError) -> WorkerFailure {
    let (code, retryable) = match error {
        StructuralError::InvalidRequest => ("invalid_request", false),
        StructuralError::ContractMismatch => ("contract_mismatch", false),
        StructuralError::ResourceLimit => ("resource_limit", false),
        StructuralError::ParserFailure => ("parser_failure", true),
        StructuralError::Io => ("io_failure", true),
        StructuralError::WorkerIdentity => ("worker_identity", false),
        StructuralError::Timeout => ("timeout", true),
        StructuralError::WorkerFailure => ("worker_failure", true),
    };
    WorkerFailure {
        schema_name: "structural-worker-error".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id,
        code: code.into(),
        retryable,
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source: &[u8], language: StructuralLanguage) -> WorkerRequest {
        let grammar = match language {
            StructuralLanguage::TypeScript | StructuralLanguage::Tsx => {
                "tree-sitter-typescript-0.23.2"
            }
            StructuralLanguage::JavaScript | StructuralLanguage::Jsx => {
                "tree-sitter-javascript-0.25.0"
            }
        };
        WorkerRequest {
            schema_name: "structural-worker-request".into(),
            schema_version: PROTOCOL_VERSION.into(),
            request_id: "req_structural_01".into(),
            language,
            path: WorkerPath {
                display_path: "src/example.ts".into(),
                platform_family: "unix".into(),
                unit_encoding: "unix_bytes".into(),
                relative_units_base64url: "c3JjL2V4YW1wbGUudHM".into(),
            },
            content_hash: sha256(source),
            source_base64url: URL_SAFE_NO_PAD.encode(source),
            fact_classes: vec![
                FactClass::Declaration,
                FactClass::Contains,
                FactClass::Import,
                FactClass::Export,
                FactClass::Call,
            ],
            max_facts: 100,
            max_nesting_depth: 128,
            max_response_bytes: 1_048_576,
            parser_version: "tree-sitter-0.26.12".into(),
            grammar_version: grammar.into(),
            resolver_version: RESOLVER_VERSION.into(),
            graph_version: GRAPH_VERSION.into(),
        }
    }

    #[test]
    fn extracts_typescript_declarations_imports_exports_and_containment() {
        let source = br#"import { value } from "./dep";
export function outer() { const nested = value(); }
class Example { method() {} }
"#;
        let output =
            process_request(&request(source, StructuralLanguage::TypeScript)).expect("parse");
        assert!(!output.syntax_errors);
        assert!(
            output
                .facts
                .iter()
                .any(|fact| fact.class == FactClass::Import
                    && fact.module.as_deref() == Some("./dep"))
        );
        assert!(
            output.facts.iter().any(
                |fact| fact.class == FactClass::Export && fact.name.as_deref() == Some("outer")
            )
        );
        assert!(
            output
                .facts
                .iter()
                .any(|fact| fact.class == FactClass::Declaration
                    && fact.name.as_deref() == Some("Example"))
        );
        assert!(output.facts.iter().any(
            |fact| fact.class == FactClass::Contains && fact.name.as_deref() == Some("nested")
        ));
        assert!(
            output
                .facts
                .iter()
                .any(|fact| fact.class == FactClass::Call && fact.name.as_deref() == Some("value"))
        );
    }

    #[test]
    fn rejects_hash_mismatch_and_fact_limit() {
        let mut invalid = request(b"const value = 1;", StructuralLanguage::TypeScript);
        invalid.content_hash = sha256(b"other");
        assert_eq!(
            process_request(&invalid),
            Err(StructuralError::ContractMismatch)
        );

        let mut limited = request(b"const a = 1; const b = 2;", StructuralLanguage::TypeScript);
        limited.max_facts = 1;
        assert_eq!(
            process_request(&limited),
            Err(StructuralError::ResourceLimit)
        );
    }

    #[test]
    fn framing_rejects_trailing_and_oversized_data() {
        let mut framed = Vec::new();
        write_frame(&mut framed, b"{}", 10).expect("frame");
        assert_eq!(
            read_single_frame(&mut framed.as_slice(), 10).expect("read"),
            b"{}"
        );
        framed.push(1);
        assert_eq!(
            read_single_frame(&mut framed.as_slice(), 10),
            Err(StructuralError::InvalidRequest)
        );
        assert_eq!(
            write_frame(&mut Vec::new(), b"too long", 2),
            Err(StructuralError::ResourceLimit)
        );
    }

    #[test]
    fn serde_rejects_unknown_and_duplicate_fields() {
        let request = request(b"const value = 1;", StructuralLanguage::TypeScript);
        let mut value = serde_json::to_value(&request).expect("serialize");
        value
            .as_object_mut()
            .expect("object")
            .insert("unknown".into(), serde_json::Value::Bool(true));
        assert!(serde_json::from_value::<WorkerRequest>(value).is_err());

        let json = serde_json::to_string(&request).expect("json");
        let duplicate = json.replacen('{', "{\"request_id\":\"req_duplicate\",", 1);
        assert!(serde_json::from_str::<WorkerRequest>(&duplicate).is_err());
    }

    #[test]
    fn graph_is_snapshot_bound_deterministic_and_explicitly_partial() {
        let source = br#"import { value } from "./dep";
function helper() { return 1; }
export function outer() { const nested = value(); helper(); }
"#;
        let request = request(source, StructuralLanguage::TypeScript);
        let response = process_request(&request).expect("parse");
        let input = GraphFileInput {
            path: request.path,
            response,
        };
        let snapshot = sha256(b"snapshot");
        let first = build_graph(&snapshot, vec![input.clone()]).expect("graph");
        let second = build_graph(&snapshot, vec![input]).expect("graph");
        assert_eq!(first, second);
        assert_eq!(first.completeness, "partial");
        assert!(
            first
                .edges
                .iter()
                .any(|edge| edge.kind == "imports" && edge.resolution == "unresolved")
        );
        assert!(
            first
                .edges
                .iter()
                .any(|edge| edge.kind == "contains" && edge.resolution == "confirmed")
        );
        let helper = first
            .nodes
            .iter()
            .find(|node| node.name.as_deref() == Some("helper"))
            .expect("helper node");
        assert!(first.edges.iter().any(|edge| {
            edge.kind == "calls"
                && edge.resolution == "heuristic"
                && edge.target_node.as_deref() == Some(helper.node_id.as_str())
        }));
        assert!(
            first
                .nodes
                .iter()
                .all(|node| node.node_id.starts_with("sha256:"))
        );

        let file = first
            .nodes
            .iter()
            .find(|node| node.kind == "file")
            .expect("file node");
        let traversal = query_graph(
            &first,
            &file.node_id,
            &["declares".into(), "imports".into(), "exports".into()],
            2,
            100,
            100,
        )
        .expect("bounded traversal");
        assert_eq!(traversal.graph_id, first.graph_id);
        assert!(traversal.nodes.len() > 1);
        assert!(traversal.edges.iter().any(|edge| edge.kind == "declares"));
        assert!(
            traversal
                .unknowns
                .contains(&"unresolved_traversal_target".into())
        );

        let limited = query_graph(&first, &file.node_id, &[], 1, 1, 1).expect("limited traversal");
        assert!(limited.truncated);
        assert_eq!(limited.nodes.len(), 1);
        assert!(
            limited
                .unknowns
                .contains(&"traversal_node_limit_reached".into())
        );
    }

    #[test]
    fn relative_imports_resolve_only_to_snapshot_files() {
        let entry_source = br#"import { value } from "./dep";
export function outer() { return value(); }
"#;
        let dep_source = b"export const value = 1;\n";
        let entry_request = request(entry_source, StructuralLanguage::TypeScript);
        let mut dep_request = request(dep_source, StructuralLanguage::TypeScript);
        dep_request.path = WorkerPath {
            display_path: "src/dep.ts".into(),
            platform_family: "unix".into(),
            unit_encoding: "unix_bytes".into(),
            relative_units_base64url: "c3JjL2RlcC50cw".into(),
        };
        let graph = build_graph(
            &sha256(b"resolved-snapshot"),
            vec![
                GraphFileInput {
                    path: entry_request.path.clone(),
                    response: process_request(&entry_request).expect("entry parse"),
                },
                GraphFileInput {
                    path: dep_request.path.clone(),
                    response: process_request(&dep_request).expect("dependency parse"),
                },
            ],
        )
        .expect("resolved graph");
        let dependency = graph
            .nodes
            .iter()
            .find(|node| node.kind == "file" && node.path.display_path == "src/dep.ts")
            .expect("dependency file node");
        let import = graph
            .edges
            .iter()
            .find(|edge| edge.kind == "imports")
            .expect("import edge");
        assert_eq!(import.resolution, "confirmed");
        assert_eq!(
            import.target_node.as_deref(),
            Some(dependency.node_id.as_str())
        );
        assert!(!graph.unknowns.contains(&"unresolved_module_import".into()));
    }
}
