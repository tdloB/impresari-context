// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Local metadata projection and narrowing-only budget policy contracts."]

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use context_core::{
    AuditEvent, AuditOutcome, Capability, ResourceBudget, validate_audit_event,
    validate_utc_timestamp,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const VERSION: &str = "1.0.0";
const MAX_RULES: usize = 128;

/// Stable local-dashboard failure categories without source-bearing detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DashboardErrorCode {
    /// A contract field or policy ordering rule is invalid.
    InvalidInput,
    /// A value exceeds the frozen local resource profile.
    ResourceLimit,
    /// A supplied identity does not match canonical content.
    IntegrityFailure,
    /// Stored metadata is incompatible or malformed.
    IncompatibleData,
}

/// Safe dashboard error that never includes policy bytes, paths, or audit data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DashboardError {
    code: DashboardErrorCode,
}

impl DashboardError {
    const fn new(code: DashboardErrorCode) -> Self {
        Self { code }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> DashboardErrorCode {
        self.code
    }
}

impl fmt::Display for DashboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            DashboardErrorCode::InvalidInput => "invalid dashboard contract input",
            DashboardErrorCode::ResourceLimit => "dashboard resource limit exceeded",
            DashboardErrorCode::IntegrityFailure => "dashboard identity verification failed",
            DashboardErrorCode::IncompatibleData => "incompatible dashboard metadata",
        })
    }
}

impl Error for DashboardError {}

/// Enumerated planner purposes that a local budget policy may narrow.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPurpose {
    /// Repository orientation.
    Orientation,
    /// Implementation work.
    Implementation,
    /// Bug investigation.
    BugInvestigation,
    /// Change review.
    ChangeReview,
    /// Security review.
    SecurityReview,
    /// Test selection.
    TestSelection,
    /// Configuration change.
    ConfigurationChange,
}

/// Optional narrowing ceilings for existing `ResourceBudget` fields.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetCeilings {
    /// Complete canonical response ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested: Option<String>,
    /// Evidence-item ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_evidence_items: Option<String>,
    /// File ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_files: Option<String>,
    /// Per-item excerpt ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_excerpt_bytes_per_item: Option<String>,
    /// Search-match ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_matches: Option<String>,
    /// Traversal-depth ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_traversal_depth: Option<String>,
    /// Elapsed-time ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_elapsed_ms: Option<String>,
    /// Accounted-memory ceiling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_bytes: Option<String>,
}

impl BudgetCeilings {
    fn is_empty(&self) -> bool {
        self.requested.is_none()
            && self.max_evidence_items.is_none()
            && self.max_files.is_none()
            && self.max_excerpt_bytes_per_item.is_none()
            && self.max_matches.is_none()
            && self.max_traversal_depth.is_none()
            && self.max_elapsed_ms.is_none()
            && self.max_memory_bytes.is_none()
    }

    fn values(&self) -> Result<[Option<u64>; 8], DashboardError> {
        Ok([
            optional_decimal(self.requested.as_deref())?,
            optional_decimal(self.max_evidence_items.as_deref())?,
            optional_decimal(self.max_files.as_deref())?,
            optional_decimal(self.max_excerpt_bytes_per_item.as_deref())?,
            optional_decimal(self.max_matches.as_deref())?,
            optional_decimal(self.max_traversal_depth.as_deref())?,
            optional_decimal(self.max_elapsed_ms.as_deref())?,
            optional_decimal(self.max_memory_bytes.as_deref())?,
        ])
    }

    fn validate(&self) -> Result<(), DashboardError> {
        let values = self.values()?;
        let ranges = [
            (1_024, 4_194_304),
            (1, 10_000),
            (1, 1_000_000),
            (1, 65_536),
            (1, 10_000),
            (1, 256),
            (1, 300_000),
            (1_048_576, 2_147_483_648),
        ];
        if values
            .iter()
            .zip(ranges)
            .any(|(value, (minimum, maximum))| {
                value.is_some_and(|value| !(minimum..=maximum).contains(&value))
            })
        {
            return Err(DashboardError::new(DashboardErrorCode::ResourceLimit));
        }
        Ok(())
    }
}

/// Enumerated exact-match selector; absence of both fields is the global scope.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetSelector {
    /// Optional planner purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<DashboardPurpose>,
    /// Optional engine capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<Capability>,
}

impl BudgetSelector {
    fn specificity(&self) -> u8 {
        u8::from(self.purpose.is_some()) + u8::from(self.capability.is_some())
    }

    fn matches(&self, purpose: DashboardPurpose, capability: Capability) -> bool {
        self.purpose.is_none_or(|candidate| candidate == purpose)
            && self
                .capability
                .is_none_or(|candidate| candidate == capability)
    }
}

/// One deterministic deny or ceiling rule.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalBudgetRule {
    /// Stable operator-owned rule identifier.
    pub rule_id: String,
    /// Exact enumerated selector.
    pub selector: BudgetSelector,
    /// A matching deny rule produces no effective budget.
    pub deny: bool,
    /// Narrowing ceilings for a non-deny rule.
    pub ceilings: BudgetCeilings,
}

/// Policy input before its canonical identity is calculated.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalBudgetPolicyDraft {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Monotonic operator-visible revision.
    pub revision: String,
    /// Explicit normalized UTC creation time.
    pub created_at: String,
    /// Optional normalized UTC expiry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Deterministically ordered policy rules.
    pub rules: Vec<LocalBudgetRule>,
}

/// Canonical narrowing-only local policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalBudgetPolicy {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated canonical policy identity.
    pub policy_id: String,
    /// Monotonic operator-visible revision.
    pub revision: String,
    /// Explicit normalized UTC creation time.
    pub created_at: String,
    /// Optional normalized UTC expiry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Canonically ordered policy rules.
    pub rules: Vec<LocalBudgetRule>,
}

/// Effective local-budget result recomputed by the engine boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectiveBudgetDecision {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated canonical decision identity.
    pub decision_id: String,
    /// Local policy identity, when present and active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_policy_id: Option<String>,
    /// Caller-requested budget.
    pub requested_budget: ResourceBudget,
    /// Field-wise minimum, absent only on denial.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_budget: Option<ResourceBudget>,
    /// Stable policy result.
    pub outcome: EffectiveBudgetOutcome,
    /// Selected rule, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_rule_id: Option<String>,
    /// Stable reduction, expiry, or denial reasons.
    pub reason_codes: Vec<String>,
    /// Explicit normalized UTC evaluation time.
    pub evaluated_at: String,
}

/// Stable narrowing-only policy outcomes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveBudgetOutcome {
    /// No governing layer reduced the request.
    Allow,
    /// At least one field was reduced.
    Limit,
    /// An exact matching rule denied the request.
    Deny,
}

/// Metadata-only event safe for dashboard serialization.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardRecord {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Opaque event identity.
    pub event_id: String,
    /// UTC event time.
    pub occurred_at: String,
    /// Attempted engine capability.
    pub capability: Capability,
    /// Metadata-only outcome.
    pub outcome: AuditOutcome,
    /// Opaque policy-decision identity.
    pub policy_decision: String,
    /// Effective request limits.
    pub limits: ResourceBudget,
    /// Duration in milliseconds.
    pub duration_ms: String,
    /// Engine semantic version.
    pub engine_version: String,
    /// Whether an opaque workspace identity existed.
    pub workspace_identity_present: bool,
    /// Whether an opaque snapshot identity existed.
    pub snapshot_identity_present: bool,
    /// One-way pseudonymous label derived from the workspace identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_label: Option<String>,
}

/// One deterministic aggregate bucket.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardAggregate {
    /// Capability in this bucket.
    pub capability: Capability,
    /// Outcome in this bucket.
    pub outcome: AuditOutcome,
    /// Decimal event count.
    pub events: String,
    /// Decimal accumulated duration.
    pub duration_ms: String,
}

/// Bounded recovery snapshot for a dashboard client.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardSnapshot {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Monotonic stream sequence selected by the foreground process.
    pub stream_sequence: String,
    /// Valid projected records, newest first.
    pub records: Vec<DashboardRecord>,
    /// Deterministic aggregate buckets.
    pub aggregates: Vec<DashboardAggregate>,
    /// Rows rejected before projection.
    pub unavailable_rows: String,
}

/// Validates and canonicalizes a policy draft.
///
/// # Errors
///
/// Fails for unknown contract versions, invalid times/revisions, duplicate
/// selectors, ambiguous rules, resource-limit expansion, or identity failure.
pub fn compile_policy(
    mut draft: LocalBudgetPolicyDraft,
) -> Result<LocalBudgetPolicy, DashboardError> {
    validate_policy_header(
        &draft.schema_name,
        &draft.schema_version,
        &draft.revision,
        &draft.created_at,
        draft.expires_at.as_deref(),
    )?;
    if draft.rules.is_empty() || draft.rules.len() > MAX_RULES {
        return Err(DashboardError::new(DashboardErrorCode::ResourceLimit));
    }
    let mut selectors = BTreeSet::new();
    let mut rule_ids = BTreeSet::new();
    for rule in &draft.rules {
        validate_rule_id(&rule.rule_id)?;
        rule.ceilings.validate()?;
        if (rule.deny && !rule.ceilings.is_empty()) || (!rule.deny && rule.ceilings.is_empty()) {
            return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
        }
        if !selectors.insert(rule.selector.clone()) || !rule_ids.insert(rule.rule_id.clone()) {
            return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
        }
    }
    draft.rules.sort_by(|left, right| {
        left.selector
            .cmp(&right.selector)
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
    let mut policy = LocalBudgetPolicy {
        schema_name: draft.schema_name,
        schema_version: draft.schema_version,
        policy_id: zero_hash(),
        revision: draft.revision,
        created_at: draft.created_at,
        expires_at: draft.expires_at,
        rules: draft.rules,
    };
    policy.policy_id = identity_for("local-budget-policy", &policy, "policy_id")?;
    Ok(policy)
}

/// Revalidates canonical policy bytes and their identity.
///
/// # Errors
///
/// Fails when any field or identity is not canonical.
pub fn validate_policy(policy: &LocalBudgetPolicy) -> Result<(), DashboardError> {
    let expected = compile_policy(LocalBudgetPolicyDraft {
        schema_name: policy.schema_name.clone(),
        schema_version: policy.schema_version.clone(),
        revision: policy.revision.clone(),
        created_at: policy.created_at.clone(),
        expires_at: policy.expires_at.clone(),
        rules: policy.rules.clone(),
    })?;
    if expected == *policy {
        Ok(())
    } else {
        Err(DashboardError::new(DashboardErrorCode::IntegrityFailure))
    }
}

/// Computes the field-wise minimum across all governing layers.
///
/// # Errors
///
/// Fails closed for invalid budgets, policies, timestamps, or identities.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_budget(
    engine_maximum: &ResourceBudget,
    authorized_budget: &ResourceBudget,
    local_policy: Option<&LocalBudgetPolicy>,
    caller_budget: &ResourceBudget,
    purpose: DashboardPurpose,
    capability: Capability,
    evaluated_at: &str,
) -> Result<EffectiveBudgetDecision, DashboardError> {
    validate_utc_timestamp(evaluated_at)
        .map_err(|_| DashboardError::new(DashboardErrorCode::InvalidInput))?;
    let engine_values = budget_values(engine_maximum)?;
    let authorized_values = budget_values(authorized_budget)?;
    let caller_values = budget_values(caller_budget)?;

    let mut selected_rule = None;
    let mut policy_id = None;
    let mut reasons = Vec::new();
    let mut local_ceilings = [None; 8];
    if let Some(policy) = local_policy {
        validate_policy(policy)?;
        if policy
            .expires_at
            .as_deref()
            .is_some_and(|expiry| timestamp_key(evaluated_at) >= timestamp_key(expiry))
        {
            reasons.push("local_policy_expired".into());
        } else {
            policy_id = Some(policy.policy_id.clone());
            selected_rule = policy
                .rules
                .iter()
                .filter(|rule| rule.selector.matches(purpose, capability))
                .min_by_key(|rule| {
                    (
                        !rule.deny,
                        Reverse(rule.selector.specificity()),
                        rule.selector.clone(),
                        rule.rule_id.clone(),
                    )
                });
            if let Some(rule) = selected_rule {
                if rule.deny {
                    reasons.push("local_policy_denied".into());
                    return decision(
                        policy_id,
                        caller_budget.clone(),
                        None,
                        EffectiveBudgetOutcome::Deny,
                        Some(rule.rule_id.clone()),
                        reasons,
                        evaluated_at,
                    );
                }
                local_ceilings = rule.ceilings.values()?;
            }
        }
    }

    let effective_values = std::array::from_fn(|index| {
        [
            engine_values[index],
            authorized_values[index],
            caller_values[index],
            local_ceilings[index].unwrap_or(u64::MAX),
        ]
        .into_iter()
        .min()
        .unwrap_or(caller_values[index])
    });
    let effective = budget_from_values(effective_values)?;
    let outcome = if effective == *caller_budget {
        reasons.push("caller_budget_unchanged".into());
        EffectiveBudgetOutcome::Allow
    } else {
        reasons.push("governing_budget_reduced".into());
        EffectiveBudgetOutcome::Limit
    };
    decision(
        policy_id,
        caller_budget.clone(),
        Some(effective),
        outcome,
        selected_rule.map(|rule| rule.rule_id.clone()),
        reasons,
        evaluated_at,
    )
}

/// Projects one validated audit event without source/query/path fields.
///
/// # Errors
///
/// Fails when a deserialized event is not canonical audit metadata.
pub fn project_event(event: &AuditEvent) -> Result<DashboardRecord, DashboardError> {
    validate_audit_event(event)
        .map_err(|_| DashboardError::new(DashboardErrorCode::IncompatibleData))?;
    budget_values(&event.limits)?;
    Ok(DashboardRecord {
        schema_name: "dashboard-record".into(),
        schema_version: VERSION.into(),
        event_id: event.event_id.clone(),
        occurred_at: event.occurred_at.clone(),
        capability: event.capability,
        outcome: event.outcome,
        policy_decision: event.policy_decision.clone(),
        limits: event.limits.clone(),
        duration_ms: event.duration_ms.clone(),
        engine_version: event.engine_version.clone(),
        workspace_identity_present: event.workspace_identity.is_some(),
        snapshot_identity_present: event.snapshot_id.is_some(),
        workspace_label: event
            .workspace_identity
            .as_deref()
            .map(pseudonymous_workspace_label),
    })
}

/// Builds a bounded deterministic snapshot from already projected records.
///
/// # Errors
///
/// Fails for more than 10,000 records or invalid decimal counters.
pub fn build_snapshot(
    stream_sequence: u64,
    records: Vec<DashboardRecord>,
    unavailable_rows: u64,
) -> Result<DashboardSnapshot, DashboardError> {
    if records.len() > 10_000 {
        return Err(DashboardError::new(DashboardErrorCode::ResourceLimit));
    }
    let mut buckets: BTreeMap<(String, String), (Capability, AuditOutcome, u64, u64)> =
        BTreeMap::new();
    for record in &records {
        let capability = serde_json::to_string(&record.capability)
            .map_err(|_| DashboardError::new(DashboardErrorCode::IncompatibleData))?;
        let outcome = serde_json::to_string(&record.outcome)
            .map_err(|_| DashboardError::new(DashboardErrorCode::IncompatibleData))?;
        let duration = decimal(&record.duration_ms)?;
        let bucket = buckets.entry((capability, outcome)).or_insert((
            record.capability,
            record.outcome,
            0,
            0,
        ));
        bucket.2 = bucket
            .2
            .checked_add(1)
            .ok_or_else(|| DashboardError::new(DashboardErrorCode::ResourceLimit))?;
        bucket.3 = bucket
            .3
            .checked_add(duration)
            .ok_or_else(|| DashboardError::new(DashboardErrorCode::ResourceLimit))?;
    }
    let aggregates = buckets
        .into_values()
        .map(
            |(capability, outcome, events, duration_ms)| DashboardAggregate {
                capability,
                outcome,
                events: events.to_string(),
                duration_ms: duration_ms.to_string(),
            },
        )
        .collect();
    Ok(DashboardSnapshot {
        schema_name: "dashboard-snapshot".into(),
        schema_version: VERSION.into(),
        stream_sequence: stream_sequence.to_string(),
        records,
        aggregates,
        unavailable_rows: unavailable_rows.to_string(),
    })
}

fn decision(
    local_policy_id: Option<String>,
    requested_budget: ResourceBudget,
    effective_budget: Option<ResourceBudget>,
    outcome: EffectiveBudgetOutcome,
    matched_rule_id: Option<String>,
    reason_codes: Vec<String>,
    evaluated_at: &str,
) -> Result<EffectiveBudgetDecision, DashboardError> {
    let mut value = EffectiveBudgetDecision {
        schema_name: "effective-budget-decision".into(),
        schema_version: VERSION.into(),
        decision_id: zero_hash(),
        local_policy_id,
        requested_budget,
        effective_budget,
        outcome,
        matched_rule_id,
        reason_codes,
        evaluated_at: evaluated_at.into(),
    };
    value.decision_id = identity_for("effective-budget-decision", &value, "decision_id")?;
    Ok(value)
}

fn validate_policy_header(
    schema_name: &str,
    schema_version: &str,
    revision: &str,
    created_at: &str,
    expires_at: Option<&str>,
) -> Result<(), DashboardError> {
    if schema_name != "local-budget-policy" || schema_version != VERSION || decimal(revision)? == 0
    {
        return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
    }
    validate_utc_timestamp(created_at)
        .map_err(|_| DashboardError::new(DashboardErrorCode::InvalidInput))?;
    if let Some(expiry) = expires_at {
        validate_utc_timestamp(expiry)
            .map_err(|_| DashboardError::new(DashboardErrorCode::InvalidInput))?;
        if timestamp_key(expiry) <= timestamp_key(created_at) {
            return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
        }
    }
    Ok(())
}

fn validate_rule_id(value: &str) -> Result<(), DashboardError> {
    if value.is_empty()
        || value.len() > 64
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || (index > 0 && (byte.is_ascii_digit() || matches!(byte, b'_' | b'-')))
        })
    {
        return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
    }
    Ok(())
}

fn budget_values(budget: &ResourceBudget) -> Result<[u64; 8], DashboardError> {
    let values = [
        decimal(&budget.requested)?,
        decimal(&budget.max_evidence_items)?,
        decimal(&budget.max_files)?,
        decimal(&budget.max_excerpt_bytes_per_item)?,
        decimal(&budget.max_matches)?,
        decimal(&budget.max_traversal_depth)?,
        decimal(&budget.max_elapsed_ms)?,
        decimal(&budget.max_memory_bytes)?,
    ];
    let canonical = budget_from_values(values)?;
    if canonical != *budget {
        return Err(DashboardError::new(DashboardErrorCode::InvalidInput));
    }
    Ok(values)
}

fn budget_from_values(values: [u64; 8]) -> Result<ResourceBudget, DashboardError> {
    ResourceBudget::conservative(
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
    )
    .map_err(|_| DashboardError::new(DashboardErrorCode::ResourceLimit))
}

fn decimal(value: &str) -> Result<u64, DashboardError> {
    if value == "0" || (!value.starts_with('0') && value.bytes().all(|byte| byte.is_ascii_digit()))
    {
        value
            .parse()
            .map_err(|_| DashboardError::new(DashboardErrorCode::ResourceLimit))
    } else {
        Err(DashboardError::new(DashboardErrorCode::InvalidInput))
    }
}

fn optional_decimal(value: Option<&str>) -> Result<Option<u64>, DashboardError> {
    value.map(decimal).transpose()
}

fn timestamp_key(value: &str) -> String {
    use fmt::Write as _;

    let mut key = value.trim_end_matches('Z').replace(['-', ':', 'T'], "");
    if let Some((whole, fraction)) = key
        .split_once('.')
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
    {
        key = whole;
        write!(key, "{fraction:0<10}").expect("writing to a string cannot fail");
    } else {
        key.push_str("0000000000");
    }
    key
}

fn identity_for<T: Serialize>(
    kind: &str,
    value: &T,
    omitted: &str,
) -> Result<String, DashboardError> {
    let mut projected = serde_json::to_value(value)
        .map_err(|_| DashboardError::new(DashboardErrorCode::IntegrityFailure))?;
    projected
        .as_object_mut()
        .ok_or_else(|| DashboardError::new(DashboardErrorCode::InvalidInput))?
        .remove(omitted);
    let payload = serde_json_canonicalizer::to_vec(&projected)
        .map_err(|_| DashboardError::new(DashboardErrorCode::IntegrityFailure))?;
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(VERSION.as_bytes());
    hasher.update(b"\0");
    hasher.update(payload);
    Ok(hash_label(hasher.finalize()))
}

fn pseudonymous_workspace_label(identity: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0dashboard-workspace-label\0");
    hasher.update(VERSION.as_bytes());
    hasher.update(b"\0");
    hasher.update(identity.as_bytes());
    hash_label(hasher.finalize())
}

fn hash_label(bytes: impl AsRef<[u8]>) -> String {
    use fmt::Write as _;
    bytes
        .as_ref()
        .iter()
        .fold(String::from("sha256:"), |mut value, byte| {
            write!(value, "{byte:02x}").expect("writing to a string cannot fail");
            value
        })
}

fn zero_hash() -> String {
    format!("sha256:{}", "0".repeat(64))
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::audit_event;

    fn budget(values: [u64; 8]) -> ResourceBudget {
        budget_from_values(values).expect("budget")
    }

    fn draft(rules: Vec<LocalBudgetRule>) -> LocalBudgetPolicyDraft {
        LocalBudgetPolicyDraft {
            schema_name: "local-budget-policy".into(),
            schema_version: VERSION.into(),
            revision: "1".into(),
            created_at: "2026-08-30T00:00:00Z".into(),
            expires_at: None,
            rules,
        }
    }

    fn ceiling(rule_id: &str, selector: BudgetSelector, requested: u64) -> LocalBudgetRule {
        LocalBudgetRule {
            rule_id: rule_id.into(),
            selector,
            deny: false,
            ceilings: BudgetCeilings {
                requested: Some(requested.to_string()),
                ..BudgetCeilings::default()
            },
        }
    }

    #[test]
    fn canonical_policy_identity_ignores_rule_input_order() {
        let global = ceiling(
            "global",
            BudgetSelector {
                purpose: None,
                capability: None,
            },
            16_384,
        );
        let exact = ceiling(
            "security_build",
            BudgetSelector {
                purpose: Some(DashboardPurpose::SecurityReview),
                capability: Some(Capability::ContextBuild),
            },
            8_192,
        );
        let left = compile_policy(draft(vec![global.clone(), exact.clone()])).expect("left");
        let right = compile_policy(draft(vec![exact, global])).expect("right");
        assert_eq!(left, right);
    }

    #[test]
    fn duplicate_selectors_and_expanding_or_ambiguous_rules_fail() {
        let selector = BudgetSelector {
            purpose: None,
            capability: None,
        };
        let duplicate = compile_policy(draft(vec![
            ceiling("one", selector.clone(), 8_192),
            ceiling("two", selector.clone(), 4_096),
        ]));
        assert_eq!(
            duplicate.expect_err("duplicate").code(),
            DashboardErrorCode::InvalidInput
        );
        let expanding = compile_policy(draft(vec![ceiling(
            "too_large",
            selector.clone(),
            4_194_305,
        )]));
        assert_eq!(
            expanding.expect_err("expanding").code(),
            DashboardErrorCode::ResourceLimit
        );
        let ambiguous = compile_policy(draft(vec![LocalBudgetRule {
            rule_id: "empty".into(),
            selector,
            deny: false,
            ceilings: BudgetCeilings::default(),
        }]));
        assert_eq!(
            ambiguous.expect_err("empty").code(),
            DashboardErrorCode::InvalidInput
        );
    }

    #[test]
    fn every_effective_field_is_at_most_every_governing_layer() {
        let maximum = budget([65_536, 100, 1_000, 4_096, 1_000, 32, 30_000, 536_870_912]);
        let authorized = budget([32_768, 80, 800, 2_048, 800, 24, 20_000, 268_435_456]);
        let caller = budget([48_000, 60, 900, 3_000, 600, 28, 25_000, 300_000_000]);
        let policy = compile_policy(draft(vec![LocalBudgetRule {
            rule_id: "narrow".into(),
            selector: BudgetSelector {
                purpose: Some(DashboardPurpose::Implementation),
                capability: Some(Capability::ContextBuild),
            },
            deny: false,
            ceilings: BudgetCeilings {
                requested: Some("12000".into()),
                max_files: Some("40".into()),
                max_elapsed_ms: Some("5000".into()),
                ..BudgetCeilings::default()
            },
        }]))
        .expect("policy");
        let decision = evaluate_budget(
            &maximum,
            &authorized,
            Some(&policy),
            &caller,
            DashboardPurpose::Implementation,
            Capability::ContextBuild,
            "2026-08-30T01:00:00Z",
        )
        .expect("decision");
        assert_eq!(decision.outcome, EffectiveBudgetOutcome::Limit);
        let effective =
            budget_values(decision.effective_budget.as_ref().expect("effective")).expect("values");
        for (index, value) in effective.into_iter().enumerate() {
            assert!(value <= budget_values(&maximum).expect("maximum")[index]);
            assert!(value <= budget_values(&authorized).expect("authorized")[index]);
            assert!(value <= budget_values(&caller).expect("caller")[index]);
        }
        assert_eq!(effective[0], 12_000);
        assert_eq!(effective[2], 40);
        assert_eq!(effective[6], 5_000);
    }

    #[test]
    fn deny_precedes_ceiling_and_expired_policy_withdraws() {
        let base = budget([65_536, 100, 100, 4_096, 1_000, 32, 30_000, 536_870_912]);
        let mut policy = compile_policy(draft(vec![
            ceiling(
                "exact_ceiling",
                BudgetSelector {
                    purpose: Some(DashboardPurpose::Implementation),
                    capability: Some(Capability::ContextBuild),
                },
                8_192,
            ),
            LocalBudgetRule {
                rule_id: "global_deny".into(),
                selector: BudgetSelector {
                    purpose: None,
                    capability: None,
                },
                deny: true,
                ceilings: BudgetCeilings::default(),
            },
        ]))
        .expect("policy");
        let denied = evaluate_budget(
            &base,
            &base,
            Some(&policy),
            &base,
            DashboardPurpose::Implementation,
            Capability::ContextBuild,
            "2026-08-30T01:00:00Z",
        )
        .expect("denied");
        assert_eq!(denied.outcome, EffectiveBudgetOutcome::Deny);

        let mut next = LocalBudgetPolicyDraft {
            schema_name: policy.schema_name.clone(),
            schema_version: policy.schema_version.clone(),
            revision: "2".into(),
            created_at: policy.created_at.clone(),
            expires_at: Some("2026-08-30T02:00:00Z".into()),
            rules: policy.rules.clone(),
        };
        next.rules.retain(|rule| !rule.deny);
        policy = compile_policy(next).expect("expiring policy");
        let expired = evaluate_budget(
            &base,
            &base,
            Some(&policy),
            &base,
            DashboardPurpose::Implementation,
            Capability::ContextBuild,
            "2026-08-30T03:00:00Z",
        )
        .expect("expired");
        assert_eq!(expired.outcome, EffectiveBudgetOutcome::Allow);
        assert_eq!(expired.local_policy_id, None);
        assert_eq!(
            expired.reason_codes,
            vec!["local_policy_expired", "caller_budget_unchanged"]
        );
    }

    #[test]
    fn projection_exposes_only_metadata_and_one_way_workspace_label() {
        let hash = format!("sha256:{}", "a".repeat(64));
        let event = audit_event(
            "evt_dashboard01",
            "req_dashboard01",
            "2026-08-30T01:00:00Z",
            Some(&hash),
            Some(&hash),
            Capability::ContextBuild,
            AuditOutcome::Limited,
            &hash,
            budget([8_192, 16, 32, 512, 64, 8, 5_000, 8_388_608]),
            17,
            "0.1.0",
        )
        .expect("event");
        let record = project_event(&event).expect("record");
        assert!(record.workspace_identity_present);
        assert!(record.snapshot_identity_present);
        assert_ne!(record.workspace_label.as_deref(), Some(hash.as_str()));
        let serialized = serde_json::to_string(&record).expect("serialize");
        for forbidden in [
            "\"display_path\":",
            "\"query\":",
            "\"excerpt\":",
            "\"prompt\":",
            "\"credential\":",
        ] {
            assert!(!serialized.contains(forbidden));
        }
    }

    #[test]
    fn snapshot_is_bounded_and_aggregates_deterministically() {
        let hash = format!("sha256:{}", "b".repeat(64));
        let event = audit_event(
            "evt_dashboard02",
            "req_dashboard02",
            "2026-08-30T01:00:00Z",
            None,
            None,
            Capability::CodeSearch,
            AuditOutcome::Allowed,
            &hash,
            budget([8_192, 16, 32, 512, 64, 8, 5_000, 8_388_608]),
            7,
            "0.1.0",
        )
        .expect("event");
        let record = project_event(&event).expect("record");
        let snapshot = build_snapshot(4, vec![record.clone(), record], 2).expect("snapshot");
        assert_eq!(snapshot.stream_sequence, "4");
        assert_eq!(snapshot.unavailable_rows, "2");
        assert_eq!(snapshot.aggregates.len(), 1);
        assert_eq!(snapshot.aggregates[0].events, "2");
        assert_eq!(snapshot.aggregates[0].duration_ms, "14");
    }
}
