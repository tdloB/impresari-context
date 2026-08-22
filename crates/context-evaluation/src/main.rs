// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Frozen, reproducible local evaluation runner."]

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use context_core::{PolicySubject, ResourceBudget};
use context_engine::{EngineConfig, LocalEngine, QueryKind, RequestContext};
use context_store::AuditRetention;
use context_workspace::{DiscoveryPolicy, PathIdentity};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

const MANIFEST_BYTES: &[u8] = include_bytes!("../../../evaluation/v1/manifest.json");
static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

#[derive(Deserialize)]
struct Manifest {
    schema_version: String,
    fixtures: Vec<Fixture>,
}
#[derive(Deserialize)]
struct Fixture {
    id: String,
    split: String,
    kind: String,
    query: String,
    required_paths: Vec<String>,
    files: Vec<FixtureFile>,
}
#[derive(Deserialize)]
struct FixtureFile {
    path: String,
    marker: String,
    prefix_bytes: usize,
    suffix_bytes: usize,
}

#[derive(Serialize)]
struct Report {
    schema_name: &'static str,
    schema_version: String,
    manifest_sha256: String,
    fixture_count: usize,
    heldout_count: usize,
    required_evidence_recall: f64,
    native_baseline_recall: f64,
    evidence_precision: f64,
    context_reduction: f64,
    exact_recovery_rate: f64,
    stale_detection_rate: f64,
    budget_compliance_rate: f64,
    deterministic_fixture_rate: f64,
    false_exact_authority_count: u64,
    failures: Vec<String>,
}

struct TempRoot(PathBuf);
impl TempRoot {
    fn new(label: &str) -> Result<Self, String> {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "impresari-eval-{label}-{}-{sequence}",
            process::id()
        ));
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request(index: usize, phase: &str) -> RequestContext {
    RequestContext {
        request_id: format!("req_eval{index:02}{phase}"),
        event_id: format!("evt_eval{index:02}{phase}"),
        subject: PolicySubject {
            caller_id: "caller_evaluation".into(),
            role: "local_user".into(),
            purpose: "frozen_evaluation".into(),
        },
        occurred_at: "2026-08-21T12:00:00Z".into(),
    }
}

fn budget() -> ResourceBudget {
    ResourceBudget::conservative(
        65_536,
        10_000,
        10_000,
        65_536,
        10_000,
        32,
        30_000,
        536_870_912,
    )
    .expect("valid frozen budget")
}

fn hex_digest(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(71);
    output.push_str("sha256:");
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn query_kind(value: &str) -> Result<QueryKind, String> {
    match value {
        "exact_path" => Ok(QueryKind::ExactPath),
        "filename" => Ok(QueryKind::Filename),
        "literal" => Ok(QueryKind::Literal),
        "lexical" => Ok(QueryKind::Lexical),
        _ => Err(format!("unsupported query kind {value}")),
    }
}

fn generate_fixture(fixture: &Fixture, root: &std::path::Path) -> Result<u64, String> {
    let mut baseline = 0_u64;
    for file in &fixture.files {
        let path = root.join(&file.path);
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)
            .map_err(|error| error.to_string())?;
        let content = format!(
            "{}{}{}",
            "p ".repeat(file.prefix_bytes / 2),
            file.marker,
            " s".repeat(file.suffix_bytes / 2)
        );
        if fixture.required_paths.contains(&file.path) {
            baseline += u64::try_from(content.len()).map_err(|error| error.to_string())?;
        }
        fs::write(path, content).map_err(|error| error.to_string())?;
    }
    Ok(baseline)
}

fn native_baseline_paths(fixture: &Fixture) -> BTreeSet<&str> {
    let query_lower = fixture.query.to_ascii_lowercase();
    let lexical_terms = query_lower.split_ascii_whitespace().collect::<Vec<_>>();
    fixture
        .files
        .iter()
        .filter(|file| match fixture.kind.as_str() {
            "exact_path" => file.path == fixture.query,
            "filename" => file.path.to_ascii_lowercase().contains(&query_lower),
            "literal" => file
                .marker
                .as_bytes()
                .windows(fixture.query.len())
                .any(|window| window == fixture.query.as_bytes()),
            "lexical" => {
                let content = file.marker.to_ascii_lowercase();
                lexical_terms.iter().all(|term| content.contains(term))
            }
            _ => false,
        })
        .map(|file| file.path.as_str())
        .collect()
}

fn ratio(numerator: usize, denominator: usize) -> Result<f64, String> {
    if denominator == 0 {
        return Ok(0.0);
    }
    let numerator = u32::try_from(numerator).map_err(|error| error.to_string())?;
    let denominator = u32::try_from(denominator).map_err(|error| error.to_string())?;
    Ok(f64::from(numerator) / f64::from(denominator))
}

#[allow(clippy::too_many_lines)]
fn run() -> Result<Report, String> {
    let manifest: Manifest =
        serde_json::from_slice(MANIFEST_BYTES).map_err(|error| error.to_string())?;
    if manifest.fixtures.len() < 12 {
        return Err("evaluation requires at least 12 fixtures".into());
    }
    let heldout = manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.split == "heldout")
        .count();
    if heldout * 4 < manifest.fixtures.len() {
        return Err("held-out fixtures are below 25%".into());
    }
    let mut required = 0_usize;
    let mut relevant = 0_usize;
    let mut baseline_relevant = 0_usize;
    let mut retrieved = 0_usize;
    let mut baseline_bytes = 0_u64;
    let mut engine_bytes = 0_u64;
    let mut recovered = 0_usize;
    let mut stale = 0_usize;
    let mut bounded = 0_usize;
    let mut deterministic = 0_usize;
    let mut false_authority = 0_u64;
    let mut failures = Vec::new();
    for (index, fixture) in manifest.fixtures.iter().enumerate() {
        let source = TempRoot::new(&fixture.id)?;
        let cache = TempRoot::new("cache")?;
        baseline_bytes += generate_fixture(fixture, &source.0)?;
        let baseline_found = native_baseline_paths(fixture);
        baseline_relevant += fixture
            .required_paths
            .iter()
            .filter(|path| baseline_found.contains(path.as_str()))
            .count();
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10_000, 536_870_912, 65_536, 32)
                .map_err(|error| error.to_string())?,
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 10_000, 10_485_760)
                .map_err(|error| error.to_string())?,
        };
        let (mut engine, _) = LocalEngine::open(config, &request(index, "open"), &source.0)
            .map_err(|error| format!("{} open: {error}", fixture.id))?;
        engine
            .build_snapshot(&request(index, "snap"), budget())
            .map_err(|error| format!("{} snapshot: {error}", fixture.id))?;
        let first = engine
            .search(
                &request(index, "find"),
                query_kind(&fixture.kind)?,
                &fixture.query,
                &budget(),
            )
            .map_err(|error| format!("{} first search: {error:?}", fixture.id))?;
        let second = engine
            .search(
                &request(index, "redo"),
                query_kind(&fixture.kind)?,
                &fixture.query,
                &budget(),
            )
            .map_err(|error| format!("{} repeat search: {error}", fixture.id))?;
        let mut found = BTreeSet::new();
        for item in &first.matches {
            let identity = PathIdentity::from_encoded_native_units(
                &item.artifact.path.platform_family,
                &item.artifact.path.unit_encoding,
                &item.artifact.path.relative_units_base64url,
            )
            .map_err(|error| format!("{} returned invalid path identity: {error}", fixture.id))?;
            let native = identity.to_relative_path().map_err(|error| {
                format!("{} returned undecodable path identity: {error}", fixture.id)
            })?;
            let portable = native
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_str()
                        .ok_or_else(|| format!("{} returned a non-UTF-8 fixture path", fixture.id))
                })
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            found.insert(portable);
        }
        required += fixture.required_paths.len();
        retrieved += found.len();
        relevant += fixture
            .required_paths
            .iter()
            .filter(|path| found.contains(path.as_str()))
            .count();
        engine_bytes += first
            .matches
            .iter()
            .map(|item| {
                item.span
                    .end_byte
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_sub(item.span.start_byte.parse::<u64>().unwrap_or(0))
            })
            .sum::<u64>();
        if first
            .matches
            .iter()
            .all(|item| item.confidence == "confirmed" && item.kind == "exact_source")
        {
        } else {
            false_authority += 1;
            failures.push(format!("{} returned false exact authority", fixture.id));
        }
        if serde_json::to_value(&first).ok().map(|mut value| {
            value["request_id"] = serde_json::Value::Null;
            value
        }) == serde_json::to_value(&second).ok().map(|mut value| {
            value["request_id"] = serde_json::Value::Null;
            value
        }) {
            deterministic += 1;
        }
        let packet = engine
            .build_context(
                &request(index, "pack"),
                query_kind(&fixture.kind)?,
                &fixture.query,
                budget(),
            )
            .map_err(|error| format!("{} packet: {error}", fixture.id))?;
        if packet
            .accounting
            .delivered_bytes
            .parse::<u64>()
            .is_ok_and(|delivered| delivered <= 65_536)
        {
            bounded += 1;
        }
        if let Some(evidence) = first.matches.first() {
            if engine
                .expand_evidence(&request(index, "recv"), evidence, 0, 0, 65_536, budget())
                .is_ok()
            {
                recovered += 1;
            }
            fs::write(
                source.0.join(&fixture.files[0].path),
                b"controlled mutation",
            )
            .map_err(|error| error.to_string())?;
            if engine
                .expand_evidence(&request(index, "stale"), evidence, 0, 0, 65_536, budget())
                .is_err()
            {
                stale += 1;
            }
        }
        if relevant < required {
            failures.push(format!("{} missed required evidence", fixture.id));
        }
    }
    let recall = ratio(relevant, required)?;
    let baseline_recall = ratio(baseline_relevant, required)?;
    let precision = ratio(relevant, retrieved)?;
    let reduction = if baseline_bytes == 0 {
        0.0
    } else {
        let engine = u32::try_from(engine_bytes).map_err(|error| error.to_string())?;
        let baseline = u32::try_from(baseline_bytes).map_err(|error| error.to_string())?;
        1.0 - f64::from(engine) / f64::from(baseline)
    };
    if recall < 0.90 {
        failures.push("recall below 0.90".into());
    }
    if precision < 0.70 {
        failures.push("precision below 0.70".into());
    }
    if recall + 0.02 < baseline_recall {
        failures.push("recall is more than 0.02 below native baseline".into());
    }
    if reduction < 0.30 {
        failures.push("context reduction below 0.30".into());
    }
    Ok(Report {
        schema_name: "evaluation-report",
        schema_version: manifest.schema_version,
        manifest_sha256: hex_digest(MANIFEST_BYTES),
        fixture_count: manifest.fixtures.len(),
        heldout_count: heldout,
        required_evidence_recall: recall,
        native_baseline_recall: baseline_recall,
        evidence_precision: precision,
        context_reduction: reduction,
        exact_recovery_rate: ratio(recovered, manifest.fixtures.len())?,
        stale_detection_rate: ratio(stale, manifest.fixtures.len())?,
        budget_compliance_rate: ratio(bounded, manifest.fixtures.len())?,
        deterministic_fixture_rate: ratio(deterministic, manifest.fixtures.len())?,
        false_exact_authority_count: false_authority,
        failures,
    })
}

fn main() {
    match run() {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("report JSON")
            );
            if !report.failures.is_empty() {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("evaluation failed: {error}");
            process::exit(1);
        }
    }
}
