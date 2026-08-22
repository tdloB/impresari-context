// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Protocol-independent contracts, policy, packet, and audit types."]

use std::{error::Error, fmt};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const VERSION: &str = "1.0.0";
/// Fingerprint of `conservative-local-v1.json`.
pub const POLICY_PROFILE: &str =
    "sha256:aba86621046ccc86cff7aabb81f4eab1020ab6db53ae1b649ea3977dec9649e8";

/// Stable core failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoreErrorCode {
    /// A contract value is invalid.
    InvalidInput,
    /// Required packet metadata cannot fit.
    BudgetTooSmall,
    /// A hard policy ceiling was exceeded.
    ResourceLimit,
    /// Canonical serialization failed.
    CanonicalizationFailure,
    /// Packet integrity or accounting is invalid.
    IntegrityFailure,
}

/// Safe contract error.
#[derive(Debug)]
pub struct CoreError {
    code: CoreErrorCode,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CoreError {
    fn new(code: CoreErrorCode) -> Self {
        Self { code, source: None }
    }
    fn canonical(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            code: CoreErrorCode::CanonicalizationFailure,
            source: Some(Box::new(source)),
        }
    }
    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> CoreErrorCode {
        self.code
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            CoreErrorCode::InvalidInput => "invalid contract input",
            CoreErrorCode::BudgetTooSmall => "packet budget is too small",
            CoreErrorCode::ResourceLimit => "resource policy limit exceeded",
            CoreErrorCode::CanonicalizationFailure => "canonical serialization failed",
            CoreErrorCode::IntegrityFailure => "packet integrity check failed",
        })
    }
}
impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|value| value as &dyn Error)
    }
}

/// Public engine capability.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Capability {
    /// Open one explicitly approved workspace.
    #[serde(rename = "workspace.open")]
    WorkspaceOpen,
    /// Query snapshot status.
    #[serde(rename = "snapshot.status")]
    SnapshotStatus,
    /// Build a derived index.
    #[serde(rename = "index.build")]
    IndexBuild,
    /// Search source evidence.
    #[serde(rename = "code.search")]
    CodeSearch,
    /// Describe code using source evidence.
    #[serde(rename = "code.describe")]
    CodeDescribe,
    /// Build a bounded packet.
    #[serde(rename = "context.build")]
    ContextBuild,
    /// Expand exact evidence.
    #[serde(rename = "evidence.expand")]
    EvidenceExpand,
    /// Validate a packet.
    #[serde(rename = "context.validate")]
    ContextValidate,
    /// Export an explicit handoff.
    #[serde(rename = "handoff.export")]
    HandoffExport,
}

/// Hard model-neutral and operational request budget.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ResourceBudget {
    /// Always `utf8_bytes` in v1.
    pub unit_kind: String,
    /// Complete canonical response ceiling.
    pub requested: String,
    /// Always true in v1.
    pub hard: bool,
    /// Independent evidence-item ceiling.
    pub max_evidence_items: String,
    /// Independent file ceiling.
    pub max_files: String,
    /// Independent excerpt ceiling.
    pub max_excerpt_bytes_per_item: String,
    /// Independent match ceiling.
    pub max_matches: String,
    /// Independent traversal ceiling.
    pub max_traversal_depth: String,
    /// Independent elapsed-time ceiling.
    pub max_elapsed_ms: String,
    /// Independent memory ceiling.
    pub max_memory_bytes: String,
    /// Exact resource-policy fingerprint.
    pub policy_profile: String,
}

impl ResourceBudget {
    /// Creates and validates the conservative v1 budget.
    ///
    /// # Errors
    ///
    /// Fails when any value is outside the accepted resource profile.
    #[allow(clippy::too_many_arguments)]
    pub fn conservative(
        requested: u64,
        max_evidence_items: u64,
        max_files: u64,
        max_excerpt_bytes_per_item: u64,
        max_matches: u64,
        max_traversal_depth: u64,
        max_elapsed_ms: u64,
        max_memory_bytes: u64,
    ) -> Result<Self, CoreError> {
        let ranges = [
            (requested, 1024, 4_194_304),
            (max_evidence_items, 1, 10_000),
            (max_files, 1, 1_000_000),
            (max_excerpt_bytes_per_item, 1, 65_536),
            (max_matches, 1, 10_000),
            (max_traversal_depth, 1, 256),
            (max_elapsed_ms, 1, 300_000),
            (max_memory_bytes, 1_048_576, 2_147_483_648),
        ];
        if ranges
            .iter()
            .any(|(value, min, max)| value < min || value > max)
        {
            return Err(CoreError::new(CoreErrorCode::ResourceLimit));
        }
        Ok(Self {
            unit_kind: "utf8_bytes".into(),
            requested: requested.to_string(),
            hard: true,
            max_evidence_items: max_evidence_items.to_string(),
            max_files: max_files.to_string(),
            max_excerpt_bytes_per_item: max_excerpt_bytes_per_item.to_string(),
            max_matches: max_matches.to_string(),
            max_traversal_depth: max_traversal_depth.to_string(),
            max_elapsed_ms: max_elapsed_ms.to_string(),
            max_memory_bytes: max_memory_bytes.to_string(),
            policy_profile: POLICY_PROFILE.into(),
        })
    }

    fn requested_u64(&self) -> Result<u64, CoreError> {
        decimal(&self.requested)
    }
}

/// Policy outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOutcome {
    /// Request is permitted unchanged.
    Allow,
    /// Request is denied.
    Deny,
    /// Request is permitted only with a reduced effective budget.
    Limit,
}

/// Immutable capability decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyDecision {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated decision identity.
    pub decision_id: String,
    /// Caller request identifier.
    pub request_id: String,
    /// Optional authorized workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_identity: Option<String>,
    /// Requested capability.
    pub capability: Capability,
    /// Policy outcome.
    pub outcome: PolicyOutcome,
    /// Stable policy reasons.
    pub reason_codes: Vec<String>,
    /// Active resource profile.
    pub policy_profile: String,
    /// Effective budget when allowed/limited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_budget: Option<ResourceBudget>,
    /// Caller-supplied UTC timestamp.
    pub evaluated_at: String,
}

/// Evaluates one local capability request deterministically.
///
/// # Errors
///
/// Fails for malformed identifiers/timestamps or canonicalization errors.
pub fn decide(
    request_id: &str,
    workspace_identity: Option<&str>,
    capability: Capability,
    budget: Option<ResourceBudget>,
    evaluated_at: &str,
) -> Result<PolicyDecision, CoreError> {
    validate_identifier(request_id)?;
    if let Some(identity) = workspace_identity {
        validate_sha256(identity)?;
    }
    validate_timestamp(evaluated_at)?;
    let requires_workspace = !matches!(capability, Capability::WorkspaceOpen);
    let outcome = if requires_workspace && workspace_identity.is_none() {
        PolicyOutcome::Deny
    } else {
        PolicyOutcome::Allow
    };
    let reasons = if outcome == PolicyOutcome::Deny {
        vec!["workspace_required".into()]
    } else {
        vec!["local_policy_allowed".into()]
    };
    let mut decision = PolicyDecision {
        schema_name: "policy-decision".into(),
        schema_version: VERSION.into(),
        decision_id: zero_hash(),
        request_id: request_id.into(),
        workspace_identity: workspace_identity.map(str::to_owned),
        capability,
        outcome,
        reason_codes: reasons,
        policy_profile: POLICY_PROFILE.into(),
        effective_budget: if outcome == PolicyOutcome::Deny {
            None
        } else {
            budget
        },
        evaluated_at: evaluated_at.into(),
    };
    decision.decision_id = identity_for("policy-decision", &decision, "decision_id")?;
    Ok(decision)
}

/// Metadata-only local audit outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Operation was allowed and completed.
    Allowed,
    /// Operation was denied by policy.
    Denied,
    /// Operation completed under a limiting condition.
    Limited,
    /// Operation failed.
    Failed,
    /// Operation was cancelled.
    Cancelled,
}

/// Metadata-first event that intentionally has no query, path, or excerpt field.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AuditEvent {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Caller-provided opaque event identifier.
    pub event_id: String,
    /// Opaque request identifier.
    pub request_id: String,
    /// UTC event time.
    pub occurred_at: String,
    /// Optional opaque workspace identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_identity: Option<String>,
    /// Optional snapshot identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    /// Capability attempted.
    pub capability: Capability,
    /// Operation outcome.
    pub outcome: AuditOutcome,
    /// Policy decision identity.
    pub policy_decision: String,
    /// Effective resource limits.
    pub limits: ResourceBudget,
    /// Duration in milliseconds.
    pub duration_ms: String,
    /// Engine semantic version.
    pub engine_version: String,
}

/// Constructs a validated metadata-only audit event.
///
/// # Errors
///
/// Fails for malformed identifiers, hashes, timestamps, duration, or version.
#[allow(clippy::too_many_arguments)]
pub fn audit_event(
    event_id: &str,
    request_id: &str,
    occurred_at: &str,
    workspace_identity: Option<&str>,
    snapshot_id: Option<&str>,
    capability: Capability,
    outcome: AuditOutcome,
    policy_decision: &str,
    limits: ResourceBudget,
    duration_ms: u64,
    engine_version: &str,
) -> Result<AuditEvent, CoreError> {
    validate_identifier(event_id)?;
    validate_identifier(request_id)?;
    validate_timestamp(occurred_at)?;
    validate_sha256(policy_decision)?;
    if let Some(value) = workspace_identity {
        validate_sha256(value)?;
    }
    if let Some(value) = snapshot_id {
        validate_sha256(value)?;
    }
    if engine_version.split('.').count() != 3
        || !engine_version
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'.')
    {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    Ok(AuditEvent {
        schema_name: "audit-event".into(),
        schema_version: VERSION.into(),
        event_id: event_id.into(),
        request_id: request_id.into(),
        occurred_at: occurred_at.into(),
        workspace_identity: workspace_identity.map(str::to_owned),
        snapshot_id: snapshot_id.map(str::to_owned),
        capability,
        outcome,
        policy_decision: policy_decision.into(),
        limits,
        duration_ms: duration_ms.to_string(),
        engine_version: engine_version.into(),
    })
}

/// Serialized path identity embedded in evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidencePath {
    /// Safe display path.
    pub display_path: String,
    /// Platform family.
    pub platform_family: String,
    /// Unit encoding.
    pub unit_encoding: String,
    /// Canonical native units.
    pub relative_units_base64url: String,
}

/// Packet evidence with exact source span and bounded bytes.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceRecord {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Evidence identity.
    pub evidence_id: String,
    /// Snapshot identity.
    pub workspace_snapshot: String,
    /// Artifact metadata.
    pub artifact: EvidenceArtifact,
    /// Exact source span.
    pub span: EvidenceSpan,
    /// Bounded exact excerpt.
    pub excerpt: EvidenceExcerpt,
    /// Evidence kind.
    pub kind: String,
    /// Extraction metadata.
    pub extraction: EvidenceExtraction,
    /// Confidence classification.
    pub confidence: String,
    /// Trust classification.
    pub trust: String,
    /// Freshness classification.
    pub freshness: String,
    /// Optional sensitivity classification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sensitivity: Option<String>,
}

/// Artifact portion of evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct EvidenceArtifact {
    pub path: EvidencePath,
    pub content_hash: String,
    pub file_kind: String,
    pub decoding: String,
}
/// Byte span portion of evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct EvidenceSpan {
    pub start_byte: String,
    pub end_byte: String,
}
/// Byte-safe excerpt portion of evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct EvidenceExcerpt {
    pub encoding: String,
    pub bytes_base64url: String,
    pub match_start_byte: String,
    pub match_end_byte: String,
}
/// Extraction provenance.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct EvidenceExtraction {
    pub method: String,
    pub version: String,
}

/// Exact packet accounting included within the byte budget.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PacketAccounting {
    /// Requested complete bytes.
    pub requested_bytes: String,
    /// Required-only serialized bytes.
    pub reserved_bytes: String,
    /// Actual canonical serialized bytes.
    pub delivered_bytes: String,
    /// Evidence items omitted.
    pub omitted_items: String,
    /// Accounting algorithm version.
    pub accounting_version: String,
}

/// Immutable bounded context packet.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(missing_docs)]
pub struct ContextPacket {
    pub schema_name: String,
    pub schema_version: String,
    pub packet_id: String,
    pub workspace_snapshot: String,
    pub request_id: String,
    pub purpose: String,
    pub created_at: String,
    pub freshness: String,
    pub completeness: String,
    pub policy_decision: String,
    pub budget: ResourceBudget,
    pub accounting: PacketAccounting,
    pub observed_evidence: Vec<EvidenceRecord>,
    pub assumptions: Vec<String>,
    pub conflicts: Vec<String>,
    pub unknowns: Vec<String>,
    pub redactions: Vec<String>,
    pub truncations: Vec<String>,
    pub packager_version: String,
}

/// Inputs for deterministic packet construction.
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub struct PacketDraft {
    pub workspace_snapshot: String,
    pub request_id: String,
    pub purpose: String,
    pub created_at: String,
    pub policy_decision: String,
    pub budget: ResourceBudget,
    pub evidence: Vec<EvidenceRecord>,
    pub assumptions: Vec<String>,
    pub conflicts: Vec<String>,
    pub unknowns: Vec<String>,
    pub redactions: Vec<String>,
}

/// Builds a packet through monotonic evidence removal.
///
/// # Errors
///
/// Fails for invalid metadata, required content larger than the budget, or
/// canonicalization/integrity errors.
pub fn build_packet(mut draft: PacketDraft) -> Result<ContextPacket, CoreError> {
    validate_sha256(&draft.workspace_snapshot)?;
    validate_sha256(&draft.policy_decision)?;
    validate_identifier(&draft.request_id)?;
    validate_timestamp(&draft.created_at)?;
    if draft.purpose.is_empty() || draft.purpose.len() > 256 || draft.purpose.contains('\0') {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    let requested = draft.budget.requested_u64()?;
    let max_items = decimal(&draft.budget.max_evidence_items)?;
    draft
        .evidence
        .sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));
    for evidence in &draft.evidence {
        validate_evidence(evidence, &draft.workspace_snapshot)?;
    }
    let initial = draft.evidence.len();
    draft
        .evidence
        .truncate(usize::try_from(max_items).unwrap_or(usize::MAX));
    let mut required = packet_from(&draft, Vec::new(), initial, "0".into());
    stabilize(&mut required)?;
    let reserved = required.accounting.delivered_bytes.clone();
    if decimal(&reserved)? > requested {
        return Err(CoreError::new(CoreErrorCode::BudgetTooSmall));
    }
    let mut selected = draft.evidence.clone();
    loop {
        let omitted = initial.saturating_sub(selected.len());
        let mut packet = packet_from(&draft, selected.clone(), omitted, reserved.clone());
        stabilize(&mut packet)?;
        if decimal(&packet.accounting.delivered_bytes)? <= requested {
            return Ok(packet);
        }
        if selected.pop().is_none() {
            return Err(CoreError::new(CoreErrorCode::BudgetTooSmall));
        }
    }
}

/// Recomputes packet identity and exact byte accounting.
///
/// # Errors
///
/// Fails for a digest mismatch, invalid accounting ordering, or budget excess.
pub fn validate_packet(packet: &ContextPacket) -> Result<(), CoreError> {
    let expected = identity_for("context-packet", packet, "packet_id")?;
    let bytes = canonical_bytes(packet)?;
    let requested = decimal(&packet.accounting.requested_bytes)?;
    let reserved = decimal(&packet.accounting.reserved_bytes)?;
    let delivered = decimal(&packet.accounting.delivered_bytes)?;
    if expected != packet.packet_id
        || delivered != bytes.len() as u64
        || reserved > delivered
        || delivered > requested
    {
        return Err(CoreError::new(CoreErrorCode::IntegrityFailure));
    }
    Ok(())
}

/// Returns exact RFC 8785 packet bytes.
///
/// # Errors
///
/// Fails when canonical serialization fails.
pub fn packet_bytes(packet: &ContextPacket) -> Result<Vec<u8>, CoreError> {
    canonical_bytes(packet)
}

/// Stable packet-validation status taxonomy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PacketValidationStatus {
    /// Packet is valid and bound to the current snapshot.
    ValidCurrent,
    /// Packet is internally valid but refers to an older snapshot.
    ValidStale,
    /// Packet identity or accounting is corrupt.
    Corrupt,
    /// Packet contract version is unsupported.
    Incompatible,
    /// Caller is not authorized to validate against the workspace.
    Denied,
    /// Packet is valid but one or more evidence checks are unavailable.
    PartiallyUnavailable,
}

/// One stable validation check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PacketValidationCheck {
    /// Contract-defined check name.
    pub name: String,
    /// `pass`, `fail`, or `unavailable`.
    pub outcome: String,
    /// Optional machine-readable reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
}

/// Versioned packet-validation result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PacketValidationResult {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Claimed packet identity.
    pub packet_id: String,
    /// Overall validation status.
    pub status: PacketValidationStatus,
    /// Ordered validation checks.
    pub checks: Vec<PacketValidationCheck>,
    /// UTC validation time supplied by the trusted caller.
    pub validated_at: String,
}

/// Produces the complete validation taxonomy without substituting evidence.
///
/// `current_snapshot` must come from an authorized current workspace snapshot.
/// `evidence_available` reports whether every evidence reference was rechecked
/// successfully by the workspace layer.
///
/// # Errors
///
/// Fails only when the trusted validation timestamp is malformed.
pub fn packet_validation_result(
    packet: &ContextPacket,
    authorized: bool,
    current_snapshot: Option<&str>,
    evidence_available: bool,
    validated_at: &str,
) -> Result<PacketValidationResult, CoreError> {
    validate_timestamp(validated_at)?;
    let mut checks = Vec::with_capacity(6);
    let compatible = packet.schema_name == "context-packet" && packet.schema_version == VERSION;
    checks.push(validation_check(
        "schema",
        compatible,
        "unsupported_packet_version",
    ));
    if !compatible {
        return Ok(validation_result(
            packet,
            PacketValidationStatus::Incompatible,
            checks,
            validated_at,
        ));
    }
    let intact = validate_packet(packet).is_ok();
    checks.push(validation_check("integrity", intact, "packet_integrity"));
    if !intact {
        return Ok(validation_result(
            packet,
            PacketValidationStatus::Corrupt,
            checks,
            validated_at,
        ));
    }
    checks.push(validation_check("authorization", authorized, "denied"));
    if !authorized {
        return Ok(validation_result(
            packet,
            PacketValidationStatus::Denied,
            checks,
            validated_at,
        ));
    }
    let Some(snapshot) = current_snapshot else {
        checks.push(unavailable_check(
            "snapshot",
            "current_snapshot_unavailable",
        ));
        return Ok(validation_result(
            packet,
            PacketValidationStatus::PartiallyUnavailable,
            checks,
            validated_at,
        ));
    };
    let current = packet.workspace_snapshot == snapshot;
    checks.push(validation_check("snapshot", current, "snapshot_stale"));
    if !current {
        checks.push(validation_check("freshness", false, "snapshot_stale"));
        return Ok(validation_result(
            packet,
            PacketValidationStatus::ValidStale,
            checks,
            validated_at,
        ));
    }
    if evidence_available {
        checks.push(validation_check("evidence", true, "evidence_unavailable"));
        checks.push(validation_check("freshness", true, "snapshot_stale"));
        Ok(validation_result(
            packet,
            PacketValidationStatus::ValidCurrent,
            checks,
            validated_at,
        ))
    } else {
        checks.push(unavailable_check("evidence", "evidence_unavailable"));
        checks.push(unavailable_check("freshness", "evidence_unavailable"));
        Ok(validation_result(
            packet,
            PacketValidationStatus::PartiallyUnavailable,
            checks,
            validated_at,
        ))
    }
}

fn validation_check(name: &str, pass: bool, reason: &str) -> PacketValidationCheck {
    PacketValidationCheck {
        name: name.into(),
        outcome: if pass { "pass" } else { "fail" }.into(),
        reason_code: (!pass).then(|| reason.into()),
    }
}
fn unavailable_check(name: &str, reason: &str) -> PacketValidationCheck {
    PacketValidationCheck {
        name: name.into(),
        outcome: "unavailable".into(),
        reason_code: Some(reason.into()),
    }
}
fn validation_result(
    packet: &ContextPacket,
    status: PacketValidationStatus,
    checks: Vec<PacketValidationCheck>,
    validated_at: &str,
) -> PacketValidationResult {
    PacketValidationResult {
        schema_name: "packet-validation".into(),
        schema_version: VERSION.into(),
        packet_id: packet.packet_id.clone(),
        status,
        checks,
        validated_at: validated_at.into(),
    }
}

fn packet_from(
    draft: &PacketDraft,
    evidence: Vec<EvidenceRecord>,
    omitted: usize,
    reserved: String,
) -> ContextPacket {
    ContextPacket {
        schema_name: "context-packet".into(),
        schema_version: VERSION.into(),
        packet_id: zero_hash(),
        workspace_snapshot: draft.workspace_snapshot.clone(),
        request_id: draft.request_id.clone(),
        purpose: draft.purpose.clone(),
        created_at: draft.created_at.clone(),
        freshness: "current".into(),
        completeness: if omitted == 0 {
            "complete".into()
        } else {
            "partial".into()
        },
        policy_decision: draft.policy_decision.clone(),
        budget: draft.budget.clone(),
        accounting: PacketAccounting {
            requested_bytes: draft.budget.requested.clone(),
            reserved_bytes: reserved,
            delivered_bytes: "0".into(),
            omitted_items: omitted.to_string(),
            accounting_version: VERSION.into(),
        },
        observed_evidence: evidence,
        assumptions: draft.assumptions.clone(),
        conflicts: draft.conflicts.clone(),
        unknowns: draft.unknowns.clone(),
        redactions: draft.redactions.clone(),
        truncations: if omitted == 0 {
            Vec::new()
        } else {
            vec!["evidence_budget".into()]
        },
        packager_version: VERSION.into(),
    }
}

fn stabilize(packet: &mut ContextPacket) -> Result<(), CoreError> {
    for _ in 0..8 {
        packet.packet_id = identity_for("context-packet", packet, "packet_id")?;
        let length = canonical_bytes(packet)?.len().to_string();
        if packet.accounting.delivered_bytes == length {
            return Ok(());
        }
        packet.accounting.delivered_bytes = length;
    }
    Err(CoreError::new(CoreErrorCode::IntegrityFailure))
}

fn identity_for<T: Serialize>(kind: &str, value: &T, omitted: &str) -> Result<String, CoreError> {
    let mut projected = serde_json::to_value(value).map_err(CoreError::canonical)?;
    projected
        .as_object_mut()
        .ok_or_else(|| CoreError::new(CoreErrorCode::InvalidInput))?
        .remove(omitted);
    let payload = canonical_bytes(&projected)?;
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(VERSION.as_bytes());
    hasher.update(b"\0");
    hasher.update(payload);
    Ok(label(hasher.finalize()))
}

fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
    serde_json_canonicalizer::to_vec(value).map_err(CoreError::canonical)
}
fn label(bytes: impl AsRef<[u8]>) -> String {
    let mut value = String::from("sha256:");
    for byte in bytes.as_ref() {
        use fmt::Write as _;
        write!(value, "{byte:02x}").expect("string write");
    }
    value
}
fn zero_hash() -> String {
    format!("sha256:{}", "0".repeat(64))
}
fn decimal(value: &str) -> Result<u64, CoreError> {
    if value == "0" || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        value
            .parse()
            .map_err(|_| CoreError::new(CoreErrorCode::ResourceLimit))
    } else {
        Err(CoreError::new(CoreErrorCode::InvalidInput))
    }
}
fn validate_sha256(value: &str) -> Result<(), CoreError> {
    if value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CoreError::new(CoreErrorCode::InvalidInput))
    }
}
fn validate_identifier(value: &str) -> Result<(), CoreError> {
    let (prefix, suffix) = value
        .split_once('_')
        .ok_or_else(|| CoreError::new(CoreErrorCode::InvalidInput))?;
    if prefix.is_empty()
        || suffix.len() < 8
        || suffix.len() > 128
        || !prefix.bytes().all(|b| b.is_ascii_lowercase())
        || !suffix
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    Ok(())
}
fn validate_timestamp(value: &str) -> Result<(), CoreError> {
    if value.len() >= 20
        && value.ends_with('Z')
        && value.as_bytes().get(4) == Some(&b'-')
        && value.as_bytes().get(10) == Some(&b'T')
    {
        Ok(())
    } else {
        Err(CoreError::new(CoreErrorCode::InvalidInput))
    }
}

fn validate_evidence(evidence: &EvidenceRecord, snapshot: &str) -> Result<(), CoreError> {
    validate_sha256(&evidence.evidence_id)?;
    validate_sha256(&evidence.workspace_snapshot)?;
    validate_sha256(&evidence.artifact.content_hash)?;
    if evidence.workspace_snapshot != snapshot || evidence.excerpt.encoding != "base64url" {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    let start = decimal(&evidence.span.start_byte)?;
    let end = decimal(&evidence.span.end_byte)?;
    let match_start = decimal(&evidence.excerpt.match_start_byte)?;
    let match_end = decimal(&evidence.excerpt.match_end_byte)?;
    if start > end || match_start > match_end || end - start != match_end - match_start {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    let excerpt = URL_SAFE_NO_PAD
        .decode(&evidence.excerpt.bytes_base64url)
        .map_err(|_| CoreError::new(CoreErrorCode::InvalidInput))?;
    if URL_SAFE_NO_PAD.encode(&excerpt) != evidence.excerpt.bytes_base64url
        || match_end > u64::try_from(excerpt.len()).unwrap_or(u64::MAX)
    {
        return Err(CoreError::new(CoreErrorCode::InvalidInput));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn budget(bytes: u64) -> ResourceBudget {
        ResourceBudget::conservative(bytes, 100, 20, 2000, 1000, 32, 30_000, 536_870_912)
            .expect("budget")
    }
    fn evidence(id: char, excerpt: usize) -> EvidenceRecord {
        EvidenceRecord {
            schema_name: "evidence".into(),
            schema_version: VERSION.into(),
            evidence_id: format!("sha256:{}", id.to_string().repeat(64)),
            workspace_snapshot: A.into(),
            artifact: EvidenceArtifact {
                path: EvidencePath {
                    display_path: format!("{id}.rs"),
                    platform_family: "unix".into(),
                    unit_encoding: "unix_bytes".into(),
                    relative_units_base64url: "YQ".into(),
                },
                content_hash: B.into(),
                file_kind: "regular_file".into(),
                decoding: "utf8".into(),
            },
            span: EvidenceSpan {
                start_byte: "0".into(),
                end_byte: "1".into(),
            },
            excerpt: EvidenceExcerpt {
                encoding: "base64url".into(),
                bytes_base64url: URL_SAFE_NO_PAD.encode(vec![b'x'; excerpt]),
                match_start_byte: "0".into(),
                match_end_byte: "1".into(),
            },
            kind: "exact_source".into(),
            extraction: EvidenceExtraction {
                method: "literal_search".into(),
                version: VERSION.into(),
            },
            confidence: "confirmed".into(),
            trust: "untrusted_workspace_content".into(),
            freshness: "current".into(),
            sensitivity: Some("normal".into()),
        }
    }
    fn draft(bytes: u64, evidence: Vec<EvidenceRecord>) -> PacketDraft {
        PacketDraft {
            workspace_snapshot: A.into(),
            request_id: "req_12345678".into(),
            purpose: "test".into(),
            created_at: "2026-08-21T00:00:00Z".into(),
            policy_decision: B.into(),
            budget: budget(bytes),
            evidence,
            assumptions: Vec::new(),
            conflicts: Vec::new(),
            unknowns: Vec::new(),
            redactions: Vec::new(),
        }
    }

    #[test]
    fn policy_decision_is_deterministic_and_workspace_gated() {
        let first = decide(
            "req_12345678",
            Some(A),
            Capability::CodeSearch,
            Some(budget(4096)),
            "2026-08-21T00:00:00Z",
        )
        .expect("decision");
        let second = decide(
            "req_12345678",
            Some(A),
            Capability::CodeSearch,
            Some(budget(4096)),
            "2026-08-21T00:00:00Z",
        )
        .expect("decision");
        assert_eq!(first, second);
        assert_eq!(
            decide(
                "req_12345678",
                None,
                Capability::CodeSearch,
                None,
                "2026-08-21T00:00:00Z"
            )
            .expect("deny")
            .outcome,
            PolicyOutcome::Deny
        );
    }

    #[test]
    fn packet_never_exceeds_budget_and_removal_is_deterministic() {
        let packet = build_packet(draft(
            4096,
            vec![
                evidence('b', 1200),
                evidence('a', 1200),
                evidence('c', 1200),
            ],
        ))
        .expect("packet");
        validate_packet(&packet).expect("valid");
        assert!(packet_bytes(&packet).expect("bytes").len() <= 4096);
        assert_eq!(
            packet.observed_evidence.first().expect("one").evidence_id,
            format!("sha256:{}", "a".repeat(64))
        );
        assert!(!packet.observed_evidence.is_empty());
    }

    #[test]
    fn tampering_and_too_small_budgets_fail_closed() {
        let mut packet = build_packet(draft(4096, vec![evidence('a', 32)])).expect("packet");
        packet.purpose = "tampered".into();
        assert_eq!(
            validate_packet(&packet).expect_err("tamper").code(),
            CoreErrorCode::IntegrityFailure
        );
        assert_eq!(
            build_packet(draft(1024, Vec::new()))
                .expect_err("too small")
                .code(),
            CoreErrorCode::BudgetTooSmall
        );
    }

    #[test]
    fn evidence_match_length_must_equal_the_authoritative_span() {
        let mut item = evidence('a', 32);
        item.span.end_byte = "2".into();
        assert_eq!(
            validate_evidence(&item, A)
                .expect_err("mismatched span")
                .code(),
            CoreErrorCode::InvalidInput
        );
    }

    #[test]
    fn packet_validation_distinguishes_every_runtime_state() {
        let packet = build_packet(draft(4096, vec![evidence('a', 32)])).expect("packet");
        let at = "2026-08-21T00:00:00Z";
        let status = |packet: &ContextPacket, authorized, snapshot, evidence_available| {
            packet_validation_result(packet, authorized, snapshot, evidence_available, at)
                .expect("validation")
                .status
        };
        assert_eq!(
            status(&packet, true, Some(A), true),
            PacketValidationStatus::ValidCurrent
        );
        assert_eq!(
            status(&packet, true, Some(B), true),
            PacketValidationStatus::ValidStale
        );
        assert_eq!(
            status(&packet, false, Some(A), true),
            PacketValidationStatus::Denied
        );
        assert_eq!(
            status(&packet, true, Some(A), false),
            PacketValidationStatus::PartiallyUnavailable
        );

        let mut corrupt = packet.clone();
        corrupt.purpose = "tampered".into();
        assert_eq!(
            status(&corrupt, true, Some(A), true),
            PacketValidationStatus::Corrupt
        );
        let mut incompatible = packet;
        incompatible.schema_version = "2.0.0".into();
        assert_eq!(
            status(&incompatible, true, Some(A), true),
            PacketValidationStatus::Incompatible
        );
    }
}
