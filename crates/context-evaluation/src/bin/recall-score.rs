// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Offline task-relative recall scorer (IC-TRFC-125)."]
//!
//! Scores delivered context against a reference change. Performs no model
//! call, opens no network socket, and never hands reference data to the
//! product: it reads the product's already-written output as opaque JSON.

use std::{collections::BTreeSet, fs, path::Path, process};

use serde::{Deserialize, Serialize};

const USAGE: &str = "usage: impresari-context-recall-score <corpus.json>";
const CORPUS_SCHEMA_NAME: &str = "impresari_context_recall_corpus";
const CORPUS_SCHEMA_VERSION: &str = "1.0";
const REPORT_SCHEMA_NAME: &str = "impresari_context_recall_report";
const REPORT_SCHEMA_VERSION: &str = "1.0";
const MAX_CORPUS_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Corpus {
    schema_name: String,
    schema_version: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    instance_id: String,
    /// Unified diff of the accepted change. Never reaches the product.
    reference_patch: String,
    /// Path to the product's already-written build result.
    delivered_context: String,
}

/// What an accepted change actually touched.
#[derive(Debug, Default, Eq, PartialEq)]
struct Reference {
    files: BTreeSet<String>,
    symbols: BTreeSet<String>,
}

/// What the product actually delivered.
#[derive(Debug, Default, Eq, PartialEq)]
struct Delivered {
    map_files: BTreeSet<String>,
    map_symbols: BTreeSet<String>,
    evidence_files: BTreeSet<String>,
    bytes: u64,
}

#[derive(Serialize)]
struct CaseScore {
    instance_id: String,
    reference_files: usize,
    reference_symbols: usize,
    /// Reference files named by the disclosure map. This is the number that
    /// decides whether an agent is pointed at the right place.
    map_file_recall_numerator: usize,
    map_symbol_recall_numerator: usize,
    /// Reference files present anywhere in the packet, including evidence the
    /// map never pointed at. A high evidence recall with a low map recall means
    /// retrieval worked and selection did not.
    evidence_file_recall_numerator: usize,
    delivered_bytes: u64,
    missing_files: Vec<String>,
    missing_symbols: Vec<String>,
}

#[derive(Serialize)]
struct Report {
    schema_name: String,
    schema_version: String,
    cases: Vec<CaseScore>,
    total_reference_files: usize,
    total_reference_symbols: usize,
    total_map_files_recalled: usize,
    total_map_symbols_recalled: usize,
    total_evidence_files_recalled: usize,
    total_delivered_bytes: u64,
    /// Percentages are integer basis points of one hundred, floored.
    map_file_recall_percent: u64,
    map_symbol_recall_percent: u64,
    evidence_file_recall_percent: u64,
    model_calls: u64,
    network_requests: u64,
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let [corpus] = arguments.as_slice() else {
        eprintln!("{USAGE}");
        process::exit(2);
    };
    match run(Path::new(corpus)) {
        Ok(report) => {
            let Ok(text) = serde_json::to_string(&report) else {
                eprintln!("impresari-context-recall-score: serialization failed");
                process::exit(1);
            };
            println!("{text}");
        }
        Err(error) => {
            eprintln!("impresari-context-recall-score: {error}");
            process::exit(1);
        }
    }
}

fn run(corpus_path: &Path) -> Result<Report, String> {
    let corpus = load_corpus(corpus_path)?;
    let mut cases = Vec::with_capacity(corpus.cases.len());
    for case in &corpus.cases {
        cases.push(score_case(case)?);
    }
    Ok(summarize(cases))
}

fn load_corpus(path: &Path) -> Result<Corpus, String> {
    let metadata = fs::metadata(path).map_err(|_| "corpus is unreadable".to_owned())?;
    if !metadata.is_file() || metadata.len() > MAX_CORPUS_BYTES {
        return Err("corpus is not a bounded regular file".to_owned());
    }
    let bytes = fs::read(path).map_err(|_| "corpus is unreadable".to_owned())?;
    let corpus: Corpus =
        serde_json::from_slice(&bytes).map_err(|_| "corpus is malformed".to_owned())?;
    if corpus.schema_name != CORPUS_SCHEMA_NAME || corpus.schema_version != CORPUS_SCHEMA_VERSION {
        return Err("corpus schema is unsupported".to_owned());
    }
    if corpus.cases.is_empty() {
        return Err("corpus is empty".to_owned());
    }
    Ok(corpus)
}

fn score_case(case: &Case) -> Result<CaseScore, String> {
    let reference = parse_reference_patch(&case.reference_patch);
    if reference.files.is_empty() {
        return Err(format!(
            "reference patch for {} names no file",
            case.instance_id
        ));
    }
    let delivered = load_delivered(Path::new(&case.delivered_context))?;

    let map_files = reference.files.intersection(&delivered.map_files).count();
    let map_symbols = reference
        .symbols
        .intersection(&delivered.map_symbols)
        .count();
    let evidence_files = reference
        .files
        .intersection(&delivered.evidence_files)
        .count();

    Ok(CaseScore {
        instance_id: case.instance_id.clone(),
        reference_files: reference.files.len(),
        reference_symbols: reference.symbols.len(),
        map_file_recall_numerator: map_files,
        map_symbol_recall_numerator: map_symbols,
        evidence_file_recall_numerator: evidence_files,
        delivered_bytes: delivered.bytes,
        missing_files: reference
            .files
            .difference(&delivered.map_files)
            .cloned()
            .collect(),
        missing_symbols: reference
            .symbols
            .difference(&delivered.map_symbols)
            .cloned()
            .collect(),
    })
}

/// Extract the files and enclosing symbols an accepted change touched.
///
/// File names come from the `+++ b/<path>` header. Symbols come from the hunk
/// section heading, which the diff format already reserves for the enclosing
/// declaration.
fn parse_reference_patch(patch: &str) -> Reference {
    let mut reference = Reference::default();
    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("+++ ") {
            let touched = rest.split('\t').next().unwrap_or(rest).trim();
            if touched == "/dev/null" {
                continue;
            }
            let touched = touched.strip_prefix("b/").unwrap_or(touched);
            if !touched.is_empty() {
                reference.files.insert(touched.to_owned());
            }
        } else if let Some(rest) = line.strip_prefix("@@ ")
            && let Some((_, heading)) = rest.split_once("@@")
            && let Some(symbol) = declaration_name(heading.trim())
        {
            reference.symbols.insert(symbol);
        }
    }
    reference
}

/// Name declared by a hunk section heading such as `def foo(self):`.
fn declaration_name(heading: &str) -> Option<String> {
    let rest = heading
        .strip_prefix("def ")
        .or_else(|| heading.strip_prefix("class "))
        .or_else(|| heading.strip_prefix("fn "))
        .or_else(|| heading.strip_prefix("function "))
        .or_else(|| heading.strip_prefix("struct "))
        .or_else(|| heading.strip_prefix("impl "))?;
    let name = rest
        .trim_start()
        .split(|character: char| {
            !(character.is_ascii_alphanumeric() || character == '_' || character == '$')
        })
        .next()?;
    (!name.is_empty()).then(|| name.to_owned())
}

/// Read the product's output as opaque JSON.
///
/// The scorer deliberately does not link the engine. It observes only what the
/// product already wrote, so no reference data can reach selection.
fn load_delivered(path: &Path) -> Result<Delivered, String> {
    let metadata = fs::metadata(path).map_err(|_| "delivered context is unreadable".to_owned())?;
    if !metadata.is_file() || metadata.len() > MAX_CORPUS_BYTES {
        return Err("delivered context is not a bounded regular file".to_owned());
    }
    let bytes = fs::read(path).map_err(|_| "delivered context is unreadable".to_owned())?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "delivered context is malformed".to_owned())?;
    let root = structured_content(&value);
    let mut delivered = Delivered {
        bytes: metadata.len(),
        ..Delivered::default()
    };
    if let Some(items) = root
        .get("disclosure_map")
        .and_then(|map| map.get("items"))
        .and_then(serde_json::Value::as_array)
    {
        for item in items {
            if let Some(path) = item.get("display_path").and_then(serde_json::Value::as_str) {
                delivered.map_files.insert(path.to_owned());
            }
            if let Some(symbol) = item.get("symbol_label").and_then(serde_json::Value::as_str) {
                delivered.map_symbols.insert(symbol.to_owned());
            }
        }
    }
    if let Some(evidence) = root
        .get("initial_packet")
        .and_then(|packet| packet.get("observed_evidence"))
        .and_then(serde_json::Value::as_array)
    {
        for item in evidence {
            if let Some(path) = item
                .get("artifact")
                .and_then(|artifact| artifact.get("path"))
                .and_then(|path| path.get("display_path"))
                .and_then(serde_json::Value::as_str)
            {
                delivered.evidence_files.insert(path.to_owned());
            }
        }
    }
    Ok(delivered)
}

/// Accept either a bare build result or one wrapped in an MCP tool response.
fn structured_content(value: &serde_json::Value) -> &serde_json::Value {
    value
        .get("result")
        .and_then(|result| result.get("structuredContent"))
        .or_else(|| value.get("structuredContent"))
        .unwrap_or(value)
}

fn summarize(cases: Vec<CaseScore>) -> Report {
    let total_reference_files = cases.iter().map(|case| case.reference_files).sum();
    let total_reference_symbols = cases.iter().map(|case| case.reference_symbols).sum();
    let total_map_files_recalled = cases
        .iter()
        .map(|case| case.map_file_recall_numerator)
        .sum();
    let total_map_symbols_recalled = cases
        .iter()
        .map(|case| case.map_symbol_recall_numerator)
        .sum();
    let total_evidence_files_recalled = cases
        .iter()
        .map(|case| case.evidence_file_recall_numerator)
        .sum();
    let total_delivered_bytes = cases
        .iter()
        .map(|case| case.delivered_bytes)
        .fold(0u64, u64::saturating_add);
    Report {
        schema_name: REPORT_SCHEMA_NAME.to_owned(),
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        map_file_recall_percent: percent(total_map_files_recalled, total_reference_files),
        map_symbol_recall_percent: percent(total_map_symbols_recalled, total_reference_symbols),
        evidence_file_recall_percent: percent(total_evidence_files_recalled, total_reference_files),
        cases,
        total_reference_files,
        total_reference_symbols,
        total_map_files_recalled,
        total_map_symbols_recalled,
        total_evidence_files_recalled,
        total_delivered_bytes,
        // Stated, not inferred: this tool calls no model and opens no socket.
        model_calls: 0,
        network_requests: 0,
    }
}

fn percent(numerator: usize, denominator: usize) -> u64 {
    if denominator == 0 {
        return 0;
    }
    let numerator = u64::try_from(numerator).unwrap_or(u64::MAX);
    let denominator = u64::try_from(denominator).unwrap_or(u64::MAX);
    numerator.saturating_mul(100) / denominator
}

#[cfg(test)]
mod tests {
    use super::*;

    const ASTROPY_PATCH: &str = "diff --git a/astropy/timeseries/core.py b/astropy/timeseries/core.py\n\
--- a/astropy/timeseries/core.py\n\
+++ b/astropy/timeseries/core.py\n\
@@ -55,6 +55,13 @@ class BaseTimeSeries(QTable):\n\
     _required_columns_relax = False\n\
 \n\
     def _check_required_columns(self):\n\
+        def as_scalar_or_list_str(obj):\n\
@@ -76,9 +83,10 @@ def _check_required_columns(self):\n\
-                raise ValueError(\"bad\")\n";

    #[test]
    fn reference_patch_yields_touched_files_and_enclosing_symbols() {
        let reference = parse_reference_patch(ASTROPY_PATCH);
        assert!(reference.files.contains("astropy/timeseries/core.py"));
        assert_eq!(reference.files.len(), 1);
        assert!(reference.symbols.contains("BaseTimeSeries"));
        assert!(reference.symbols.contains("_check_required_columns"));
    }

    #[test]
    fn deleted_files_and_absent_headings_are_not_counted() {
        let patch = "--- a/gone.py\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-x = 1\n";
        let reference = parse_reference_patch(patch);
        assert!(reference.files.is_empty());
        assert!(reference.symbols.is_empty());
    }

    #[test]
    fn declaration_names_are_extracted_across_languages_and_prose_is_not() {
        assert_eq!(
            declaration_name("def _check_required_columns(self):").as_deref(),
            Some("_check_required_columns")
        );
        assert_eq!(
            declaration_name("class BaseTimeSeries(QTable):").as_deref(),
            Some("BaseTimeSeries")
        );
        assert_eq!(
            declaration_name("fn structural_seed_decision(").as_deref(),
            Some("structural_seed_decision")
        );
        assert_eq!(declaration_name("some prose about a class"), None);
        assert_eq!(declaration_name(""), None);
    }

    #[test]
    fn a_map_pointing_at_the_wrong_file_scores_zero_while_evidence_can_still_hit() {
        // This is the measured astropy failure: sixteen items, all in the
        // sibling file, while the evidence packet did contain the target.
        let delivered = serde_json::json!({
            "structuredContent": {
                "disclosure_map": {"items": [
                    {"display_path": "astropy/timeseries/sampled.py", "symbol_label": "TimeSeries"},
                    {"display_path": "astropy/timeseries/sampled.py", "symbol_label": "add_column"}
                ]},
                "initial_packet": {"observed_evidence": [
                    {"artifact": {"path": {"display_path": "astropy/timeseries/core.py"}}}
                ]}
            }
        });
        let root = structured_content(&delivered);
        assert!(root.get("disclosure_map").is_some());

        let reference = parse_reference_patch(ASTROPY_PATCH);
        let map_files: BTreeSet<String> = ["astropy/timeseries/sampled.py".to_owned()]
            .into_iter()
            .collect();
        let evidence_files: BTreeSet<String> = ["astropy/timeseries/core.py".to_owned()]
            .into_iter()
            .collect();
        assert_eq!(reference.files.intersection(&map_files).count(), 0);
        assert_eq!(reference.files.intersection(&evidence_files).count(), 1);
    }

    #[test]
    fn percentages_floor_and_tolerate_an_empty_denominator() {
        assert_eq!(percent(0, 0), 0);
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(2, 2), 100);
        assert_eq!(percent(0, 7), 0);
    }

    #[test]
    fn scorer_never_hands_reference_data_to_the_product() {
        // Oracle isolation is structural: this binary links no engine crate and
        // names no product entry point. It reads output the product already
        // wrote.
        let source = include_str!("recall-score.rs");
        // Scan only the shipped code; this list would otherwise match itself.
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        // Network absence is enforced repository-wide by
        // scripts/check-security-boundaries.sh; this test covers the isolation
        // that scan cannot see — that the scorer never links the product.
        for forbidden in [
            "context_engine",
            "context_store",
            "context_workspace",
            "LocalEngine",
        ] {
            assert!(
                !shipped.contains(forbidden),
                "scorer must not reach {forbidden}"
            );
        }
    }
}
