// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Evidence-only hostile-repository admission contracts and inventory."]

use std::{
    collections::BTreeMap,
    error::Error,
    fmt::{self, Write as _},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{
    EvidenceArtifact, EvidenceExcerpt, EvidenceExtraction, EvidencePath, EvidenceRecord,
    EvidenceSpan, validate_utc_timestamp,
};
use context_workspace::{
    ArtifactRecord, AuthorizedWorkspace, PathIdentity, SkipReason, WorkspaceErrorCode,
    WorkspaceSnapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CONTRACT_VERSION: &str = "1.0.0";
const PROFILE_DIGEST: &str =
    "sha256:1fa9f737b8452d86bffa00ff0a76539c8312f2342b2a14568e270b05b3170c83";
const MAX_FILES: usize = 10_000;
const MAX_WORKSPACE_BYTES: u64 = 536_870_912;
const MAX_FILE_BYTES: u64 = 16_777_216;
const CLASSIFICATION_PREFIX_BYTES: usize = 4096;
const MAX_OUTPUT_BYTES: usize = 4_194_304;
const MAX_FINDINGS: usize = 1000;
const MAX_ELAPSED: Duration = Duration::from_secs(30);

/// Stable HRA-1 inventory failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionErrorCode {
    /// The supplied timestamp is not canonical UTC.
    InvalidTimestamp,
    /// The workspace capability and snapshot do not identify the same root.
    WorkspaceMismatch,
    /// A snapshot artifact changed after snapshot creation.
    StaleSnapshot,
    /// The frozen profile could not represent a valid bounded result.
    ResourceLimit,
    /// Canonical serialization failed.
    Serialization,
}

/// Safe HRA-1 failure that carries no path, source, or repository-controlled text.
#[derive(Debug)]
pub struct AdmissionError {
    code: AdmissionErrorCode,
}

impl AdmissionError {
    const fn new(code: AdmissionErrorCode) -> Self {
        Self { code }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> AdmissionErrorCode {
        self.code
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            AdmissionErrorCode::InvalidTimestamp => "invalid inventory timestamp",
            AdmissionErrorCode::WorkspaceMismatch => "workspace does not match snapshot",
            AdmissionErrorCode::StaleSnapshot => "workspace snapshot is stale",
            AdmissionErrorCode::ResourceLimit => "inventory resource limit exceeded",
            AdmissionErrorCode::Serialization => "inventory serialization failed",
        })
    }
}

impl Error for AdmissionError {}

/// Lossless path identity copied from the authoritative workspace snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryPath {
    /// Diagnostic-only escaped display path.
    pub display_path: String,
    /// Native path family.
    pub platform_family: String,
    /// Native unit encoding.
    pub unit_encoding: String,
    /// Canonical unpadded base64url native units.
    pub relative_units_base64url: String,
}

impl From<&PathIdentity> for InventoryPath {
    fn from(path: &PathIdentity) -> Self {
        Self {
            display_path: path.display_path.clone(),
            platform_family: path.platform_family.to_owned(),
            unit_encoding: path.unit_encoding.to_owned(),
            relative_units_base64url: path.relative_units_base64url.clone(),
        }
    }
}

/// One static, snapshot-bound artifact classification.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityArtifact {
    /// Domain-separated identity of the classification record.
    pub artifact_id: String,
    /// Lossless relative path identity.
    pub path: InventoryPath,
    /// SHA-256 of exact current bytes.
    pub content_hash: String,
    /// Exact file length as a canonical unsigned decimal string.
    pub byte_size: String,
    /// Conservative bounded format classification.
    pub format: String,
    /// Closed artifact classes.
    pub classes: Vec<String>,
    /// Closed target-platform classes.
    pub target_platforms: Vec<String>,
    /// `observed` or `unknown`.
    pub classification: String,
    /// Future analyzer capabilities implied by the static class; none are run.
    pub analyzer_requirements: Vec<String>,
    /// Stable, content-free limitations.
    pub limitations: Vec<String>,
}

/// One explicit exclusion reason and count.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryExclusion {
    /// Stable exclusion category.
    pub reason: String,
    /// Number of excluded objects as a canonical unsigned decimal string.
    pub count: String,
}

/// HRA-1 evidence-only inventory output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityArtifactInventory {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated inventory identity.
    pub inventory_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Frozen HRA resource-profile digest.
    pub profile_digest: String,
    /// Caller-supplied canonical UTC creation time.
    pub generated_at: String,
    /// `complete` or `partial`; HRA-1 never claims availability beyond input.
    pub completeness: String,
    /// Deterministically ordered artifact records.
    pub artifacts: Vec<SecurityArtifact>,
    /// Deterministically ordered exclusion summaries.
    pub exclusions: Vec<InventoryExclusion>,
    /// Sum of all exclusion counts.
    pub excluded_count: String,
    /// Constant proof that the inventory adds no authority.
    pub authority_added: bool,
}

/// One schema-valid, authority-free HRA-2 observed finding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SecurityFinding {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated finding identity.
    pub finding_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact artifact content digest.
    pub artifact_hash: String,
    /// Exact evidence identity.
    pub evidence_id: String,
    /// Always `observed` in HRA-2.
    pub classification: String,
    /// Closed security-finding category.
    pub category: String,
    /// Bounded severity; an observation is not a risk decision.
    pub severity: String,
    /// Exact syntactic declaration confidence.
    pub confidence: String,
    /// Stable rule identifier.
    pub method: String,
    /// Always untrusted workspace content.
    pub trust: String,
    /// Stable limitations that prohibit intent or safety inference.
    pub limitations: Vec<String>,
    /// Constant proof that the observation adds no authority.
    pub authority_added: bool,
}

/// Explicit reason why a candidate HRA-2 file was not interpreted.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSurfaceExclusion {
    /// Lossless path identity from the inventory.
    pub path: InventoryPath,
    /// Stable, content-free exclusion reason.
    pub reason: String,
}

/// Bounded HRA-2 observations and their exact evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSurfaceObservations {
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Deterministically ordered observed findings.
    pub findings: Vec<SecurityFinding>,
    /// Exact evidence records referenced by the findings.
    pub evidence: Vec<EvidenceRecord>,
    /// Explicit unsupported or omitted candidates.
    pub exclusions: Vec<ExecutionSurfaceExclusion>,
    /// Whether the frozen finding limit omitted additional observations.
    pub truncated: bool,
    /// Constant proof that observations add no authority.
    pub authority_added: bool,
}

#[derive(Serialize)]
struct FindingIdentity<'finding> {
    workspace_snapshot: &'finding str,
    artifact_hash: &'finding str,
    evidence_id: &'finding str,
    category: &'finding str,
    method: &'finding str,
}

#[derive(Serialize)]
struct EvidenceIdentity<'evidence> {
    snapshot: &'evidence str,
    path_units: &'evidence str,
    content_hash: &'evidence str,
    start_byte: String,
    end_byte: String,
}

#[derive(Serialize)]
struct ArtifactIdentity<'artifact> {
    path: &'artifact InventoryPath,
    content_hash: &'artifact str,
    byte_size: &'artifact str,
    format: &'artifact str,
    classes: &'artifact [String],
    target_platforms: &'artifact [String],
    classification: &'artifact str,
    analyzer_requirements: &'artifact [String],
    limitations: &'artifact [String],
}

#[derive(Serialize)]
struct InventoryIdentity<'inventory> {
    workspace_snapshot: &'inventory str,
    profile_digest: &'inventory str,
    generated_at: &'inventory str,
    completeness: &'inventory str,
    artifacts: &'inventory [SecurityArtifact],
    exclusions: &'inventory [InventoryExclusion],
    excluded_count: &'inventory str,
    authority_added: bool,
}

#[derive(Serialize)]
struct InventoryWire<'inventory> {
    schema_name: &'static str,
    schema_version: &'static str,
    inventory_id: &'inventory str,
    workspace_snapshot: &'inventory str,
    profile_digest: &'static str,
    generated_at: &'inventory str,
    completeness: &'inventory str,
    artifacts: &'inventory [SecurityArtifact],
    exclusions: &'inventory [InventoryExclusion],
    excluded_count: &'inventory str,
    authority_added: bool,
}

/// Builds a bounded HRA-1 inventory from an existing exact workspace snapshot.
///
/// The operation performs only capability-relative guarded reads. It does not
/// emit findings or decisions, parse hostile formats deeply, invoke analyzers,
/// start processes, access the network, upload content, or mutate the workspace.
///
/// # Errors
///
/// Returns a safe error for an invalid timestamp, mismatched workspace,
/// snapshot mutation, serialization failure, or an unrepresentable output.
pub fn build_security_artifact_inventory(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    generated_at: &str,
) -> Result<SecurityArtifactInventory, AdmissionError> {
    validate_utc_timestamp(generated_at)
        .map_err(|_| AdmissionError::new(AdmissionErrorCode::InvalidTimestamp))?;
    if workspace.identity() != snapshot.workspace_identity {
        return Err(AdmissionError::new(AdmissionErrorCode::WorkspaceMismatch));
    }

    let (artifacts, exclusions) = collect_artifacts(workspace, snapshot)?;
    finalize_inventory(snapshot, generated_at, artifacts, exclusions)
}

/// Emits narrow HRA-2 package lifecycle observations from an HRA-1 inventory.
///
/// Only a strict `package.json` top-level `scripts` object is inspected. The
/// implementation recognizes a closed set of lifecycle keys, records only the
/// exact key token as evidence, and never interprets or executes its value.
/// Unsupported syntax is explicit. No process, network, analyzer, upload,
/// policy, deep-parser, or repository-execution capability is used.
///
/// # Errors
///
/// Returns a safe error for mismatched workspace/snapshot/inventory identity,
/// stale source, serialization failure, or an unrepresentable resource state.
pub fn observe_execution_surfaces(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    inventory: &SecurityArtifactInventory,
) -> Result<ExecutionSurfaceObservations, AdmissionError> {
    if workspace.identity() != snapshot.workspace_identity
        || inventory.workspace_snapshot != snapshot.snapshot_id
        || inventory.profile_digest != PROFILE_DIGEST
        || inventory.authority_added
    {
        return Err(AdmissionError::new(AdmissionErrorCode::WorkspaceMismatch));
    }

    let mut findings = Vec::new();
    let mut evidence = Vec::new();
    let mut exclusions = Vec::new();
    let mut truncated = false;
    for artifact in &inventory.artifacts {
        if !is_package_json(&artifact.path) {
            continue;
        }
        let snapshot_artifact = snapshot
            .artifacts
            .iter()
            .find(|candidate| {
                candidate.path.relative_units_base64url == artifact.path.relative_units_base64url
            })
            .ok_or_else(|| AdmissionError::new(AdmissionErrorCode::WorkspaceMismatch))?;
        if snapshot_artifact.content_hash != artifact.content_hash
            || snapshot_artifact.size_bytes.to_string() != artifact.byte_size
        {
            return Err(AdmissionError::new(AdmissionErrorCode::WorkspaceMismatch));
        }
        let exact = workspace
            .read_exact(&snapshot_artifact.path, MAX_FILE_BYTES)
            .map_err(|_| AdmissionError::new(AdmissionErrorCode::StaleSnapshot))?;
        if exact.content_hash != artifact.content_hash
            || exact.bytes.len().to_string() != artifact.byte_size
        {
            return Err(AdmissionError::new(AdmissionErrorCode::StaleSnapshot));
        }

        let spans = match package_lifecycle_spans(&exact.bytes) {
            Ok(spans) => spans,
            Err(reason) => {
                exclusions.push(ExecutionSurfaceExclusion {
                    path: artifact.path.clone(),
                    reason: reason.to_owned(),
                });
                continue;
            }
        };
        for (rule, start, end) in spans {
            if findings.len() >= MAX_FINDINGS {
                truncated = true;
                break;
            }
            let record =
                make_observed_evidence(snapshot, snapshot_artifact, &exact.bytes, start, end)?;
            let finding = make_lifecycle_finding(snapshot, snapshot_artifact, &record, rule)?;
            evidence.push(record);
            findings.push(finding);
        }
        if truncated {
            exclusions.push(ExecutionSurfaceExclusion {
                path: artifact.path.clone(),
                reason: "finding_limit".to_owned(),
            });
            break;
        }
    }
    findings.sort_by(|left, right| left.finding_id.cmp(&right.finding_id));
    evidence.sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));
    exclusions.sort_by(|left, right| {
        left.path
            .relative_units_base64url
            .cmp(&right.path.relative_units_base64url)
            .then(left.reason.cmp(&right.reason))
    });
    Ok(ExecutionSurfaceObservations {
        workspace_snapshot: snapshot.snapshot_id.clone(),
        findings,
        evidence,
        exclusions,
        truncated,
        authority_added: false,
    })
}

const LIFECYCLE_RULES: [(&str, &str); 8] = [
    ("preinstall", "npm-preinstall-v1"),
    ("install", "npm-install-v1"),
    ("postinstall", "npm-postinstall-v1"),
    ("prepare", "npm-prepare-v1"),
    ("prepublish", "npm-prepublish-v1"),
    ("prepublishOnly", "npm-prepublish-only-v1"),
    ("publish", "npm-publish-v1"),
    ("postpublish", "npm-postpublish-v1"),
];

fn is_package_json(path: &InventoryPath) -> bool {
    path.display_path.replace('\\', "/").rsplit('/').next() == Some("package.json")
}

fn package_lifecycle_spans(
    bytes: &[u8],
) -> Result<Vec<(&'static str, usize, usize)>, &'static str> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| "invalid_json")?;
    let root = value.as_object().ok_or("root_not_object")?;
    let Some(scripts) = root.get("scripts") else {
        return Ok(Vec::new());
    };
    let scripts = scripts.as_object().ok_or("scripts_not_object")?;
    let Some((object_start, object_end)) = top_level_json_object_field(bytes, b"scripts") else {
        return Err("scripts_key_encoding_unsupported");
    };
    let object = &bytes[object_start..object_end];
    let mut spans = Vec::new();
    for (key, rule) in LIFECYCLE_RULES {
        let Some(value) = scripts.get(key) else {
            continue;
        };
        if !value.is_string() {
            return Err("lifecycle_value_not_string");
        }
        let matches = direct_json_key_spans(object, key.as_bytes());
        if matches.len() != 1 {
            return Err("lifecycle_key_ambiguous");
        }
        let (start, end) = matches[0];
        spans.push((rule, object_start + start, object_start + end));
    }
    spans.sort_by_key(|(_, start, _)| *start);
    Ok(spans)
}

fn top_level_json_object_field(bytes: &[u8], key: &[u8]) -> Option<(usize, usize)> {
    let spans = direct_json_key_spans(bytes, key);
    let (key_start, key_end) = *spans.first()?;
    if spans.len() != 1 || key_start == 0 {
        return None;
    }
    let mut cursor = key_end;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b':') {
        return None;
    }
    cursor += 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'{') {
        return None;
    }
    matching_json_object_end(bytes, cursor).map(|end| (cursor, end))
}

fn direct_json_key_spans(bytes: &[u8], key: &[u8]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut index = 0;
    let mut depth = 0_u32;
    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                depth = depth.saturating_add(1);
                index += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            b'"' => {
                let start = index;
                let Some(end) = json_string_end(bytes, index) else {
                    break;
                };
                if depth == 1 && bytes.get(start + 1..end) == Some(key) {
                    let mut cursor = end + 1;
                    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                        cursor += 1;
                    }
                    if bytes.get(cursor) == Some(&b':') {
                        spans.push((start, end + 1));
                    }
                }
                index = end + 1;
            }
            _ => index += 1,
        }
    }
    spans
}

fn json_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    let mut escaped = false;
    while cursor < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[cursor] == b'\\' {
            escaped = true;
        } else if bytes[cursor] == b'"' {
            return Some(cursor);
        }
        cursor += 1;
    }
    None
}

fn matching_json_object_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0_u32;
    let mut cursor = start;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'"' => cursor = json_string_end(bytes, cursor)? + 1,
            b'{' => {
                depth = depth.saturating_add(1);
                cursor += 1;
            }
            b'}' => {
                depth = depth.checked_sub(1)?;
                cursor += 1;
                if depth == 0 {
                    return Some(cursor);
                }
            }
            _ => cursor += 1,
        }
    }
    None
}

fn make_observed_evidence(
    snapshot: &WorkspaceSnapshot,
    artifact: &ArtifactRecord,
    bytes: &[u8],
    start: usize,
    end: usize,
) -> Result<EvidenceRecord, AdmissionError> {
    let payload = EvidenceIdentity {
        snapshot: &snapshot.snapshot_id,
        path_units: &artifact.path.relative_units_base64url,
        content_hash: &artifact.content_hash,
        start_byte: start.to_string(),
        end_byte: end.to_string(),
    };
    let evidence_id = structured_identity("evidence", &payload)?;
    Ok(EvidenceRecord {
        schema_name: "evidence".to_owned(),
        schema_version: CONTRACT_VERSION.to_owned(),
        evidence_id,
        workspace_snapshot: snapshot.snapshot_id.clone(),
        artifact: EvidenceArtifact {
            path: EvidencePath {
                display_path: artifact.path.display_path.clone(),
                platform_family: artifact.path.platform_family.to_owned(),
                unit_encoding: artifact.path.unit_encoding.to_owned(),
                relative_units_base64url: artifact.path.relative_units_base64url.clone(),
            },
            content_hash: artifact.content_hash.clone(),
            file_kind: "regular_file".to_owned(),
            decoding: "utf8".to_owned(),
        },
        span: EvidenceSpan {
            start_byte: start.to_string(),
            end_byte: end.to_string(),
        },
        excerpt: EvidenceExcerpt {
            encoding: "base64url".to_owned(),
            bytes_base64url: URL_SAFE_NO_PAD.encode(&bytes[start..end]),
            match_start_byte: "0".to_owned(),
            match_end_byte: (end - start).to_string(),
        },
        kind: "exact_source".to_owned(),
        extraction: EvidenceExtraction {
            method: "permitted_pattern".to_owned(),
            version: CONTRACT_VERSION.to_owned(),
        },
        confidence: "confirmed".to_owned(),
        trust: "untrusted_workspace_content".to_owned(),
        freshness: "current".to_owned(),
        sensitivity: None,
    })
}

fn make_lifecycle_finding(
    snapshot: &WorkspaceSnapshot,
    artifact: &ArtifactRecord,
    evidence: &EvidenceRecord,
    rule: &str,
) -> Result<SecurityFinding, AdmissionError> {
    let payload = FindingIdentity {
        workspace_snapshot: &snapshot.snapshot_id,
        artifact_hash: &artifact.content_hash,
        evidence_id: &evidence.evidence_id,
        category: "lifecycle_hook",
        method: rule,
    };
    Ok(SecurityFinding {
        schema_name: "security-finding".to_owned(),
        schema_version: CONTRACT_VERSION.to_owned(),
        finding_id: structured_identity("security-finding", &payload)?,
        workspace_snapshot: snapshot.snapshot_id.clone(),
        artifact_hash: artifact.content_hash.clone(),
        evidence_id: evidence.evidence_id.clone(),
        classification: "observed".to_owned(),
        category: "lifecycle_hook".to_owned(),
        severity: "informational".to_owned(),
        confidence: "confirmed".to_owned(),
        method: rule.to_owned(),
        trust: "untrusted_workspace_content".to_owned(),
        limitations: vec![
            "declaration_value_not_interpreted_or_executed".to_owned(),
            "observed_configuration_does_not_establish_intent_or_safety".to_owned(),
        ],
        authority_added: false,
    })
}

fn collect_artifacts(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
) -> Result<(Vec<SecurityArtifact>, BTreeMap<String, u64>), AdmissionError> {
    let started = Instant::now();
    let mut exclusions = snapshot
        .skipped
        .iter()
        .map(|(reason, count)| (skip_reason_name(*reason).to_owned(), *count))
        .collect::<BTreeMap<_, _>>();
    let mut artifacts = Vec::with_capacity(snapshot.artifacts.len().min(MAX_FILES));
    let mut admitted_bytes = 0_u64;

    for (index, artifact) in snapshot.artifacts.iter().enumerate() {
        if index >= MAX_FILES
            || admitted_bytes.saturating_add(artifact.size_bytes) > MAX_WORKSPACE_BYTES
            || started.elapsed() >= MAX_ELAPSED
        {
            add_exclusion(
                &mut exclusions,
                "limit_reached",
                u64::try_from(snapshot.artifacts.len() - index).unwrap_or(u64::MAX),
            );
            break;
        }
        if artifact.size_bytes > MAX_FILE_BYTES {
            add_exclusion(&mut exclusions, "profile_file_limit", 1);
            continue;
        }

        let exact = match workspace.read_exact(&artifact.path, MAX_FILE_BYTES) {
            Ok(exact) => exact,
            Err(error) if error.code() == WorkspaceErrorCode::ChangedDuringRead => {
                return Err(AdmissionError::new(AdmissionErrorCode::StaleSnapshot));
            }
            Err(_) => {
                add_exclusion(&mut exclusions, "read_failed", 1);
                continue;
            }
        };
        let exact_len = u64::try_from(exact.bytes.len()).unwrap_or(u64::MAX);
        if exact.content_hash != artifact.content_hash || exact_len != artifact.size_bytes {
            return Err(AdmissionError::new(AdmissionErrorCode::StaleSnapshot));
        }

        artifacts.push(classify_artifact(artifact, &exact.bytes)?);
        admitted_bytes = admitted_bytes.saturating_add(artifact.size_bytes);
    }
    Ok((artifacts, exclusions))
}

fn finalize_inventory(
    snapshot: &WorkspaceSnapshot,
    generated_at: &str,
    mut artifacts: Vec<SecurityArtifact>,
    mut exclusions: BTreeMap<String, u64>,
) -> Result<SecurityArtifactInventory, AdmissionError> {
    loop {
        let runtime_incomplete = exclusions.keys().any(|reason| {
            matches!(
                reason.as_str(),
                "oversized"
                    | "limit_reached"
                    | "read_failed"
                    | "changed_during_read"
                    | "profile_file_limit"
                    | "snapshot_mismatch"
            )
        });
        let completeness = if snapshot.complete && !runtime_incomplete {
            "complete"
        } else {
            "partial"
        };
        let exclusion_summaries = exclusions
            .iter()
            .map(|(reason, count)| InventoryExclusion {
                reason: reason.clone(),
                count: count.to_string(),
            })
            .collect::<Vec<_>>();
        let excluded_count = exclusions
            .values()
            .copied()
            .fold(0_u64, u64::saturating_add)
            .to_string();
        let identity_payload = InventoryIdentity {
            workspace_snapshot: &snapshot.snapshot_id,
            profile_digest: PROFILE_DIGEST,
            generated_at,
            completeness,
            artifacts: &artifacts,
            exclusions: &exclusion_summaries,
            excluded_count: &excluded_count,
            authority_added: false,
        };
        let inventory_id = structured_identity("security-artifact-inventory", &identity_payload)?;
        let wire = InventoryWire {
            schema_name: "security-artifact-inventory",
            schema_version: CONTRACT_VERSION,
            inventory_id: &inventory_id,
            workspace_snapshot: &snapshot.snapshot_id,
            profile_digest: PROFILE_DIGEST,
            generated_at,
            completeness,
            artifacts: &artifacts,
            exclusions: &exclusion_summaries,
            excluded_count: &excluded_count,
            authority_added: false,
        };
        let bytes = serde_json_canonicalizer::to_vec(&wire)
            .map_err(|_| AdmissionError::new(AdmissionErrorCode::Serialization))?;
        if bytes.len() <= MAX_OUTPUT_BYTES {
            return Ok(SecurityArtifactInventory {
                schema_name: "security-artifact-inventory".to_owned(),
                schema_version: CONTRACT_VERSION.to_owned(),
                inventory_id,
                workspace_snapshot: snapshot.snapshot_id.clone(),
                profile_digest: PROFILE_DIGEST.to_owned(),
                generated_at: generated_at.to_owned(),
                completeness: completeness.to_owned(),
                artifacts,
                exclusions: exclusion_summaries,
                excluded_count,
                authority_added: false,
            });
        }
        if artifacts.pop().is_none() {
            return Err(AdmissionError::new(AdmissionErrorCode::ResourceLimit));
        }
        add_exclusion(&mut exclusions, "limit_reached", 1);
    }
}

fn classify_artifact(
    artifact: &ArtifactRecord,
    bytes: &[u8],
) -> Result<SecurityArtifact, AdmissionError> {
    let prefix = &bytes[..bytes.len().min(CLASSIFICATION_PREFIX_BYTES)];
    let extension = artifact
        .path
        .to_relative_path()
        .ok()
        .and_then(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(str::to_owned)
        })
        .map(|value| value.to_ascii_lowercase());
    let extension = extension.as_deref().unwrap_or("");
    let (format, classes, targets, requirements, mut limitations) =
        classify_prefix(prefix, extension);
    if extension_implies_pe(extension) && format != "pe_candidate" {
        limitations.push("extension_magic_disagreement".to_owned());
    }
    if format == "pe_candidate" && !extension_implies_pe(extension) {
        limitations.push("extension_magic_disagreement".to_owned());
    }

    let path = InventoryPath::from(&artifact.path);
    let byte_size = artifact.size_bytes.to_string();
    let classification = if format == "unknown" {
        "unknown"
    } else {
        "observed"
    };
    limitations.sort();
    limitations.dedup();
    let identity_payload = ArtifactIdentity {
        path: &path,
        content_hash: &artifact.content_hash,
        byte_size: &byte_size,
        format,
        classes: &classes,
        target_platforms: &targets,
        classification,
        analyzer_requirements: &requirements,
        limitations: &limitations,
    };
    let artifact_id = structured_identity("security-artifact", &identity_payload)?;
    Ok(SecurityArtifact {
        artifact_id,
        path,
        content_hash: artifact.content_hash.clone(),
        byte_size,
        format: format.to_owned(),
        classes,
        target_platforms: targets,
        classification: classification.to_owned(),
        analyzer_requirements: requirements,
        limitations,
    })
}

type Classification = (
    &'static str,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
);

fn classify_prefix(prefix: &[u8], extension: &str) -> Classification {
    classify_magic(prefix, extension).unwrap_or_else(|| classify_text_or_binary(prefix, extension))
}

fn classify_magic(prefix: &[u8], extension: &str) -> Option<Classification> {
    if prefix.starts_with(b"MZ") {
        let class = if matches!(extension, "dll" | "ocx") {
            "library"
        } else {
            "executable"
        };
        return Some(classification(
            "pe_candidate",
            &[class],
            &["windows"],
            &["windows.pe.metadata"],
            &["bounded_magic_only", "deep_format_parsing_not_performed"],
        ));
    }
    if extension == "msi" && prefix.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
        return Some(classification(
            "msi_candidate",
            &["installer"],
            &["windows"],
            &["windows.msi.tables"],
            &["bounded_magic_only", "deep_format_parsing_not_performed"],
        ));
    }
    if is_archive_magic(prefix) {
        return Some(classification(
            "archive_candidate",
            &["archive"],
            &["cross_platform"],
            &["archive.static"],
            &["archive_not_traversed", "bounded_magic_only"],
        ));
    }
    None
}

fn classify_text_or_binary(prefix: &[u8], extension: &str) -> Classification {
    if prefix.contains(&0) || std::str::from_utf8(prefix).is_err() {
        return classification(
            "binary",
            &["unknown"],
            &["cross_platform"],
            &["binary.static"],
            &["binary_content_not_parsed"],
        );
    }

    match extension {
        "ps1" | "psd1" | "psm1" => classification(
            "text",
            &["script"],
            &["windows"],
            &["windows.powershell.static"],
            &["content_not_executed"],
        ),
        "bat" | "cmd" => classification(
            "text",
            &["script"],
            &["windows"],
            &["windows.batch.static"],
            &["content_not_executed"],
        ),
        "reg" => classification(
            "text",
            &["configuration"],
            &["windows"],
            &["windows.persistence.static"],
            &["content_not_executed"],
        ),
        "sln" | "csproj" | "vcxproj" | "props" | "targets" => classification(
            "text",
            &["configuration"],
            &["windows"],
            &["windows.build.execution-surface"],
            &["configuration_not_evaluated"],
        ),
        "sh" | "bash" | "zsh" | "fish" => classification(
            "text",
            &["script"],
            &["cross_platform"],
            &["script.static"],
            &["content_not_executed"],
        ),
        "json" | "jsonc" | "toml" | "yaml" | "yml" | "xml" | "ini" | "conf" => classification(
            "text",
            &["configuration"],
            &["cross_platform"],
            &[],
            &["configuration_not_evaluated"],
        ),
        "md" | "markdown" | "rst" | "txt" => classification(
            "text",
            &["document"],
            &["cross_platform"],
            &[],
            &["text_treated_as_untrusted_data"],
        ),
        "rs" | "go" | "py" | "js" | "jsx" | "ts" | "tsx" | "java" | "kt" | "kts" | "cs"
        | "scala" | "ex" | "exs" | "clj" | "cljs" | "cljc" | "hs" | "lhs" | "c" | "h" | "cc"
        | "cpp" | "cxx" | "hpp" | "hh" | "hxx" | "rb" | "php" | "swift" => classification(
            "text",
            &["source"],
            &["cross_platform"],
            &[],
            &["source_not_executed"],
        ),
        _ if prefix.is_empty() => classification(
            "unknown",
            &["unknown"],
            &["cross_platform"],
            &[],
            &["empty_artifact"],
        ),
        _ => classification(
            "text",
            &["unknown"],
            &["cross_platform"],
            &[],
            &["unrecognized_text_artifact"],
        ),
    }
}

fn classification(
    format: &'static str,
    classes: &[&str],
    targets: &[&str],
    requirements: &[&str],
    limitations: &[&str],
) -> Classification {
    (
        format,
        classes.iter().map(|value| (*value).to_owned()).collect(),
        targets.iter().map(|value| (*value).to_owned()).collect(),
        requirements
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
        limitations
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    )
}

fn is_archive_magic(prefix: &[u8]) -> bool {
    prefix.starts_with(b"PK\x03\x04")
        || prefix.starts_with(&[0x1f, 0x8b])
        || prefix.starts_with(&[0x37, 0x7a, 0xbc, 0xaf, 0x27, 0x1c])
        || prefix.starts_with(b"Rar!\x1a\x07")
        || prefix.get(257..262) == Some(b"ustar")
}

fn extension_implies_pe(extension: &str) -> bool {
    matches!(
        extension,
        "exe" | "dll" | "sys" | "scr" | "com" | "cpl" | "ocx"
    )
}

fn add_exclusion(exclusions: &mut BTreeMap<String, u64>, reason: &str, count: u64) {
    let entry = exclusions.entry(reason.to_owned()).or_default();
    *entry = entry.saturating_add(count);
}

const fn skip_reason_name(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::PolicyExcluded => "policy_excluded",
        SkipReason::Oversized => "oversized",
        SkipReason::Symlink => "symlink",
        SkipReason::SpecialFile => "special_file",
        SkipReason::LimitReached => "limit_reached",
        SkipReason::ReadFailed => "read_failed",
        SkipReason::ChangedDuringRead => "changed_during_read",
    }
}

fn structured_identity<T: Serialize>(kind: &str, value: &T) -> Result<String, AdmissionError> {
    let payload = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| AdmissionError::new(AdmissionErrorCode::Serialization))?;
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context");
    hasher.update([0]);
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(CONTRACT_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    let mut digest = String::with_capacity(71);
    digest.push_str("sha256:");
    for byte in hasher.finalize() {
        write!(digest, "{byte:02x}").expect("writing to a string cannot fail");
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use context_workspace::DiscoveryPolicy;
    use jsonschema::Registry;
    use serde_json::Value;

    use super::*;

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestWorkspace {
        root: PathBuf,
    }

    impl TestWorkspace {
        fn new() -> Self {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "impresari-hra-inventory-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&root).expect("create test workspace");
            Self { root }
        }

        fn write(&self, relative: &str, bytes: &[u8]) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create fixture parent");
            }
            fs::write(path, bytes).expect("write fixture");
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).expect("remove isolated test workspace");
        }
    }

    fn policy() -> DiscoveryPolicy {
        DiscoveryPolicy::new(
            u64::try_from(MAX_FILES).expect("files fit u64"),
            MAX_WORKSPACE_BYTES,
            MAX_FILE_BYTES,
            64,
        )
        .expect("frozen discovery policy")
    }

    fn artifact<'inventory>(
        inventory: &'inventory SecurityArtifactInventory,
        display_path: &str,
    ) -> &'inventory SecurityArtifact {
        inventory
            .artifacts
            .iter()
            .find(|artifact| artifact.path.display_path.replace('\\', "/") == display_path)
            .unwrap_or_else(|| panic!("missing artifact {display_path}"))
    }

    fn repository_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repository root")
    }

    fn read_json(path: &Path) -> Value {
        serde_json::from_slice(&fs::read(path).expect("read JSON")).expect("valid JSON")
    }

    #[test]
    fn inventory_classifies_bounded_cross_platform_artifacts_without_retaining_bytes() {
        let fixture = TestWorkspace::new();
        fixture.write("bin/tool.exe", b"MZDO_NOT_RETAIN");
        fixture.write(
            "setup/setup.msi",
            &[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1, 0, 1],
        );
        fixture.write("scripts/install.ps1", b"Write-Output 'synthetic'");
        fixture.write("scripts/install.cmd", b"@echo off\r\n");
        fixture.write(
            "config/service.reg",
            b"Windows Registry Editor Version 5.00\r\n",
        );
        fixture.write(
            "build/example.sln",
            b"Microsoft Visual Studio Solution File",
        );
        fixture.write("archive/payload.zip", b"PK\x03\x04synthetic");
        fixture.write("src/main.rs", b"fn main() {}\n");
        fixture.write("notes/readme.md", b"repository text is untrusted\n");
        fixture.write("unknown/blob.bin", &[0, 1, 2, 3]);

        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let before = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &before, "2026-08-29T13:00:00Z")
                .expect("inventory");
        let repeated =
            build_security_artifact_inventory(&workspace, &before, "2026-08-29T13:00:00Z")
                .expect("repeat inventory");
        assert_eq!(inventory, repeated, "identical input must be byte-stable");
        assert_eq!(inventory.completeness, "complete");
        assert!(!inventory.authority_added);
        assert_eq!(inventory.profile_digest, PROFILE_DIGEST);

        assert_eq!(artifact(&inventory, "bin/tool.exe").format, "pe_candidate");
        assert_eq!(artifact(&inventory, "bin/tool.exe").classes, ["executable"]);
        assert_eq!(
            artifact(&inventory, "setup/setup.msi").format,
            "msi_candidate"
        );
        assert_eq!(
            artifact(&inventory, "scripts/install.ps1").classes,
            ["script"]
        );
        assert_eq!(
            artifact(&inventory, "scripts/install.ps1").analyzer_requirements,
            ["windows.powershell.static"]
        );
        assert_eq!(
            artifact(&inventory, "archive/payload.zip").format,
            "archive_candidate"
        );
        assert_eq!(artifact(&inventory, "src/main.rs").classes, ["source"]);
        assert_eq!(artifact(&inventory, "unknown/blob.bin").format, "binary");

        let serialized = serde_json::to_vec(&inventory).expect("serialize inventory");
        let text = std::str::from_utf8(&serialized).expect("inventory JSON");
        assert!(!text.contains("DO_NOT_RETAIN"));
        assert!(!text.contains("finding_id"));
        assert!(!text.contains("decision"));
        assert!(serialized.len() <= MAX_OUTPUT_BYTES);

        let after = workspace
            .snapshot(policy())
            .expect("snapshot after inventory");
        assert_eq!(
            before.snapshot_id, after.snapshot_id,
            "source must not mutate"
        );
    }

    #[test]
    fn inventory_is_valid_under_the_frozen_published_schema() {
        let fixture = TestWorkspace::new();
        fixture.write("src/lib.rs", b"pub fn bounded() {}\n");
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let snapshot = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect("inventory");

        let root = repository_root();
        let schema_root = root.join("schemas/v1");
        let common = read_json(&schema_root.join("common.schema.json"));
        let path = read_json(&schema_root.join("path-identity.schema.json"));
        let schema = read_json(&schema_root.join("security-artifact-inventory.schema.json"));
        let registry = Registry::new()
            .add(common["$id"].as_str().expect("common id"), common.clone())
            .expect("register common")
            .add(path["$id"].as_str().expect("path id"), path.clone())
            .expect("register path")
            .prepare()
            .expect("prepare registry");
        let validator = jsonschema::draft202012::options()
            .with_registry(&registry)
            .build(&schema)
            .expect("compile inventory schema");
        let value = serde_json::to_value(inventory).expect("inventory value");
        assert!(validator.is_valid(&value));
    }

    #[test]
    fn stale_snapshot_and_invalid_control_input_fail_closed() {
        let fixture = TestWorkspace::new();
        fixture.write("src/lib.rs", b"original\n");
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let snapshot = workspace.snapshot(policy()).expect("snapshot");
        assert_eq!(
            build_security_artifact_inventory(&workspace, &snapshot, "not-a-time")
                .expect_err("timestamp should fail")
                .code(),
            AdmissionErrorCode::InvalidTimestamp
        );
        fixture.write("src/lib.rs", b"changed\n");
        assert_eq!(
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect_err("stale snapshot should fail")
                .code(),
            AdmissionErrorCode::StaleSnapshot
        );
    }

    #[test]
    fn snapshot_omissions_are_explicit_and_make_incomplete_inventory_visible() {
        let fixture = TestWorkspace::new();
        fixture.write("small.txt", b"ok");
        fixture.write("oversized.bin", b"12345");
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let narrow = DiscoveryPolicy::new(10, 100, 4, 16).expect("narrow policy");
        let snapshot = workspace.snapshot(narrow).expect("partial snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect("inventory");
        assert_eq!(inventory.completeness, "partial");
        assert_eq!(inventory.excluded_count, "1");
        assert_eq!(
            inventory.exclusions,
            [InventoryExclusion {
                reason: "oversized".to_owned(),
                count: "1".to_owned(),
            }]
        );
    }

    #[test]
    fn execution_surfaces_emit_only_exact_npm_lifecycle_declarations() {
        let fixture = TestWorkspace::new();
        fixture.write(
            "package.json",
            br#"{
  "scripts": {
    "test": "ignored",
    "preinstall": "repository command remains untrusted",
    "postinstall": "another untrusted value"
  },
  "metadata": {"install": "not a lifecycle declaration"}
}"#,
        );
        fixture.write(
            "nested/package.json",
            br#"{"scripts":{"build":"ignored"},"preinstall":"not under scripts"}"#,
        );
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let before = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &before, "2026-08-29T13:00:00Z")
                .expect("inventory");
        let observed = observe_execution_surfaces(&workspace, &before, &inventory)
            .expect("execution-surface observations");

        assert_eq!(observed.findings.len(), 2);
        assert_eq!(observed.evidence.len(), 2);
        assert!(observed.exclusions.is_empty());
        assert!(!observed.truncated);
        assert!(!observed.authority_added);
        assert!(
            observed
                .findings
                .iter()
                .all(|finding| finding.category == "lifecycle_hook"
                    && finding.classification == "observed"
                    && finding.severity == "informational"
                    && !finding.authority_added)
        );
        let excerpts = observed
            .evidence
            .iter()
            .map(|record| {
                URL_SAFE_NO_PAD
                    .decode(&record.excerpt.bytes_base64url)
                    .expect("decode exact evidence")
            })
            .collect::<Vec<_>>();
        assert!(excerpts.contains(&br#""preinstall""#.to_vec()));
        assert!(excerpts.contains(&br#""postinstall""#.to_vec()));
        let serialized = serde_json::to_string(&observed).expect("serialize observations");
        assert!(!serialized.contains("repository command remains untrusted"));
        assert!(!serialized.contains("another untrusted value"));

        let after = workspace
            .snapshot(policy())
            .expect("snapshot after observations");
        assert_eq!(
            before.snapshot_id, after.snapshot_id,
            "source must not mutate"
        );
    }

    #[test]
    fn execution_surface_records_validate_under_frozen_schemas() {
        let fixture = TestWorkspace::new();
        fixture.write(
            "package.json",
            br#"{"scripts":{"prepare":"untrusted value"}}"#,
        );
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let snapshot = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect("inventory");
        let observed = observe_execution_surfaces(&workspace, &snapshot, &inventory)
            .expect("execution-surface observations");

        let root = repository_root();
        let schema_root = root.join("schemas/v1");
        let common = read_json(&schema_root.join("common.schema.json"));
        let path = read_json(&schema_root.join("path-identity.schema.json"));
        let registry = Registry::new()
            .add(common["$id"].as_str().expect("common id"), common.clone())
            .expect("register common")
            .add(path["$id"].as_str().expect("path id"), path.clone())
            .expect("register path")
            .prepare()
            .expect("prepare registry");
        for (schema_name, value) in [
            (
                "security-finding.schema.json",
                serde_json::to_value(&observed.findings[0]).expect("finding value"),
            ),
            (
                "evidence.schema.json",
                serde_json::to_value(&observed.evidence[0]).expect("evidence value"),
            ),
        ] {
            let schema = read_json(&schema_root.join(schema_name));
            let validator = jsonschema::draft202012::options()
                .with_registry(&registry)
                .build(&schema)
                .expect("compile frozen schema");
            assert!(validator.is_valid(&value), "{schema_name} should validate");
        }
    }

    #[test]
    fn unsupported_and_stale_execution_surface_inputs_fail_closed() {
        let fixture = TestWorkspace::new();
        fixture.write("package.json", br#"{"scripts":["unsupported"]}"#);
        fixture.write("invalid/package.json", b"not json");
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let snapshot = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect("inventory");
        let observed = observe_execution_surfaces(&workspace, &snapshot, &inventory)
            .expect("explicit unsupported observations");
        let reasons = observed
            .exclusions
            .iter()
            .map(|exclusion| exclusion.reason.as_str())
            .collect::<Vec<_>>();
        assert!(reasons.contains(&"scripts_not_object"));
        assert!(reasons.contains(&"invalid_json"));
        assert!(observed.findings.is_empty());

        fixture.write("package.json", br#"{"scripts":{"install":"changed"}}"#);
        assert_eq!(
            observe_execution_surfaces(&workspace, &snapshot, &inventory)
                .expect_err("stale source should fail")
                .code(),
            AdmissionErrorCode::StaleSnapshot
        );
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_links_are_reported_but_never_followed() {
        use std::os::unix::fs::symlink;

        let fixture = TestWorkspace::new();
        fixture.write("real.txt", b"bounded");
        symlink("real.txt", fixture.root.join("link.txt")).expect("create symlink");
        let workspace = AuthorizedWorkspace::open(&fixture.root).expect("open workspace");
        let snapshot = workspace.snapshot(policy()).expect("snapshot");
        let inventory =
            build_security_artifact_inventory(&workspace, &snapshot, "2026-08-29T13:00:00Z")
                .expect("inventory");
        assert_eq!(inventory.artifacts.len(), 1);
        assert!(
            inventory
                .exclusions
                .iter()
                .any(|exclusion| exclusion.reason == "symlink" && exclusion.count == "1")
        );
    }
}
