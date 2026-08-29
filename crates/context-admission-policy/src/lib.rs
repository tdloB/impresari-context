// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Pure, authority-neutral repository admission policy evaluation."]

use std::{
    collections::BTreeSet,
    error::Error,
    fmt::{self, Write as _},
};

use context_admission::{
    AnalyzerCoverage, RepositorySecurityAssessment, SecurityFinding, validate_analyzer_coverage,
    validate_repository_security_assessment, validate_security_finding,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CONTRACT_VERSION: &str = "1.0.0";
const EVALUATOR_VERSION: &str = "1.0.0";
const MAX_RULES: usize = 256;

/// Stable failures from closed, pure policy evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyErrorCode {
    /// A supplied immutable record is malformed or internally inconsistent.
    InvalidInput,
    /// The supplied policy digest does not identify its exact contents.
    PolicyDigestMismatch,
    /// Policy rules are ambiguous, duplicated, or non-canonical.
    InvalidPolicy,
    /// Canonical serialization failed.
    Serialization,
}

/// Content-free policy evaluation failure.
#[derive(Debug)]
pub struct PolicyError(PolicyErrorCode);

impl PolicyError {
    const fn new(code: PolicyErrorCode) -> Self {
        Self(code)
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> PolicyErrorCode {
        self.0
    }
}

impl fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            PolicyErrorCode::InvalidInput => "invalid immutable admission input",
            PolicyErrorCode::PolicyDigestMismatch => "admission policy digest mismatch",
            PolicyErrorCode::InvalidPolicy => "invalid deterministic admission policy",
            PolicyErrorCode::Serialization => "admission policy serialization failed",
        })
    }
}

impl Error for PolicyError {}

/// Closed deterministic policy contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryAdmissionPolicy {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Stable policy identifier.
    pub policy_id: String,
    /// Exact policy version.
    pub policy_version: String,
    /// Domain-separated digest of every authority-bearing policy field.
    pub policy_digest: String,
    /// Canonically ordered rules.
    pub rules: Vec<AdmissionRule>,
    /// Fixed external exception owner; the evaluator accepts no exception input.
    pub exception_authority: String,
    /// Always false.
    pub authority_added: bool,
}

/// One closed deterministic matching rule.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionRule {
    /// Stable rule identifier.
    pub rule_id: String,
    /// Canonical unsigned decimal priority; lower values evaluate first.
    pub priority: String,
    /// Closed effect.
    pub effect: String,
    /// Closed immutable input field.
    pub match_field: String,
    /// Sorted unique closed values.
    pub match_values: Vec<String>,
    /// Stable decision reason.
    pub reason_code: String,
    /// Required only for isolated eligibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarantine_profile: Option<String>,
}

/// Authority-neutral four-state decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryAdmissionDecision {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated decision identity.
    pub decision_id: String,
    /// Exact assessment identity.
    pub assessment_id: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact policy identifier.
    pub policy_id: String,
    /// Exact policy version.
    pub policy_version: String,
    /// Exact policy digest.
    pub policy_digest: String,
    /// Pure evaluator contract version.
    pub evaluator_version: String,
    /// One of the four frozen decisions.
    pub decision: String,
    /// Every matched rule, ordered by priority and identifier.
    pub matched_rule_ids: Vec<String>,
    /// Sorted unique missing mandatory analyzer capabilities.
    pub missing_prerequisites: Vec<String>,
    /// Present only for isolated eligibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quarantine_profile: Option<String>,
    /// Always false.
    pub safety_claimed: bool,
    /// Always false.
    pub ordinary_host_execution_authorized: bool,
    /// Always false.
    pub authority_added: bool,
}

#[derive(Serialize)]
struct PolicyIdentity<'a> {
    policy_id: &'a str,
    policy_version: &'a str,
    rules: &'a [AdmissionRule],
    exception_authority: &'a str,
    authority_added: bool,
}

#[derive(Serialize)]
struct DecisionIdentity<'a> {
    assessment_id: &'a str,
    workspace_snapshot: &'a str,
    policy_id: &'a str,
    policy_version: &'a str,
    policy_digest: &'a str,
    evaluator_version: &'static str,
    decision: &'a str,
    matched_rule_ids: &'a [String],
    missing_prerequisites: &'a [String],
    quarantine_profile: &'a Option<String>,
    safety_claimed: bool,
    ordinary_host_execution_authorized: bool,
    authority_added: bool,
}

/// Computes the exact domain-separated digest for a policy's contents.
///
/// # Errors
///
/// Returns a serialization failure if canonical JSON cannot be produced.
pub fn policy_digest(policy: &RepositoryAdmissionPolicy) -> Result<String, PolicyError> {
    structured_identity(
        "repository-admission-policy",
        &PolicyIdentity {
            policy_id: &policy.policy_id,
            policy_version: &policy.policy_version,
            rules: &policy.rules,
            exception_authority: &policy.exception_authority,
            authority_added: false,
        },
    )
}

/// Evaluates immutable admission evidence through a closed deterministic policy.
///
/// This function performs no filesystem, process, network, model, credential,
/// approval, or exception operation. It can make eligibility more restrictive,
/// but it can never authorize ordinary-host execution.
///
/// # Errors
///
/// Returns a content-free error for malformed, mismatched, non-canonical, or
/// authority-claiming inputs.
pub fn evaluate_repository_admission(
    assessment: &RepositorySecurityAssessment,
    coverage: &AnalyzerCoverage,
    findings: &[SecurityFinding],
    policy: &RepositoryAdmissionPolicy,
) -> Result<RepositoryAdmissionDecision, PolicyError> {
    validate_inputs(assessment, coverage, findings)?;
    validate_policy(policy)?;

    let mut matched = Vec::new();
    let mut strongest = 0_u8;
    let mut eligibility_profiles = BTreeSet::new();
    for rule in &policy.rules {
        if rule_matches(rule, assessment, coverage, findings) {
            matched.push(rule.rule_id.clone());
            strongest = strongest.max(effect_rank(&rule.effect));
            if rule.effect == "allow_isolated_eligibility" {
                eligibility_profiles.insert(rule.quarantine_profile.clone().unwrap_or_default());
            }
        }
    }

    let mut missing_prerequisites = coverage
        .requirements
        .iter()
        .filter(|requirement| requirement.mandatory && requirement.state != "completed")
        .map(|requirement| requirement.capability_id.clone())
        .collect::<Vec<_>>();
    missing_prerequisites.sort();
    missing_prerequisites.dedup();

    let complete = assessment.completeness == "complete"
        && assessment.conflicts.is_empty()
        && assessment.unknowns.is_empty()
        && missing_prerequisites.is_empty();
    let (decision, quarantine_profile) = if strongest == 4 {
        ("blocked", None)
    } else if strongest == 3 {
        ("manual_review_required", None)
    } else if strongest == 2 || !complete {
        ("analysis_incomplete", None)
    } else if strongest == 1 && eligibility_profiles.len() == 1 {
        (
            "isolated_execution_eligible",
            eligibility_profiles.into_iter().next(),
        )
    } else {
        ("manual_review_required", None)
    };

    let mut output = RepositoryAdmissionDecision {
        schema_name: "repository-admission-decision".to_owned(),
        schema_version: CONTRACT_VERSION.to_owned(),
        decision_id: String::new(),
        assessment_id: assessment.assessment_id.clone(),
        workspace_snapshot: assessment.workspace_snapshot.clone(),
        policy_id: policy.policy_id.clone(),
        policy_version: policy.policy_version.clone(),
        policy_digest: policy.policy_digest.clone(),
        evaluator_version: EVALUATOR_VERSION.to_owned(),
        decision: decision.to_owned(),
        matched_rule_ids: matched,
        missing_prerequisites,
        quarantine_profile,
        safety_claimed: false,
        ordinary_host_execution_authorized: false,
        authority_added: false,
    };
    output.decision_id = decision_identity(&output)?;
    Ok(output)
}

fn validate_inputs(
    assessment: &RepositorySecurityAssessment,
    coverage: &AnalyzerCoverage,
    findings: &[SecurityFinding],
) -> Result<(), PolicyError> {
    validate_repository_security_assessment(assessment)
        .map_err(|_| PolicyError::new(PolicyErrorCode::InvalidInput))?;
    validate_analyzer_coverage(coverage)
        .map_err(|_| PolicyError::new(PolicyErrorCode::InvalidInput))?;
    if assessment.workspace_snapshot != coverage.workspace_snapshot
        || assessment.coverage_id != coverage.coverage_id
        || findings.len() > 1000
    {
        return Err(PolicyError::new(PolicyErrorCode::InvalidInput));
    }
    let mut supplied_ids = findings
        .iter()
        .map(|finding| finding.finding_id.clone())
        .collect::<Vec<_>>();
    supplied_ids.sort();
    supplied_ids.dedup();
    if supplied_ids != assessment.finding_ids
        || findings.iter().any(|finding| {
            validate_security_finding(finding).is_err()
                || finding.workspace_snapshot != assessment.workspace_snapshot
        })
    {
        return Err(PolicyError::new(PolicyErrorCode::InvalidInput));
    }
    Ok(())
}

fn validate_policy(policy: &RepositoryAdmissionPolicy) -> Result<(), PolicyError> {
    if policy.schema_name != "repository-admission-policy"
        || policy.schema_version != CONTRACT_VERSION
        || !valid_identifier(&policy.policy_id)
        || !valid_schema_version(&policy.policy_version)
        || policy.exception_authority != "authorized_human_outside_context_core"
        || policy.authority_added
        || policy.rules.is_empty()
        || policy.rules.len() > MAX_RULES
        || policy_digest(policy)? != policy.policy_digest
    {
        return Err(PolicyError::new(PolicyErrorCode::PolicyDigestMismatch));
    }
    let mut previous: Option<(u64, &str)> = None;
    let mut ids = BTreeSet::new();
    for rule in &policy.rules {
        let priority = canonical_unsigned(&rule.priority)
            .ok_or_else(|| PolicyError::new(PolicyErrorCode::InvalidPolicy))?;
        let key = (priority, rule.rule_id.as_str());
        if previous.is_some_and(|value| value >= key)
            || !ids.insert(rule.rule_id.as_str())
            || rule.match_values.is_empty()
            || rule.match_values.len() > 16
            || !strictly_sorted(&rule.match_values)
            || !valid_schema_name(&rule.rule_id)
            || !valid_schema_name(&rule.reason_code)
            || effect_rank(&rule.effect) == 0
            || !matches!(
                rule.match_field.as_str(),
                "finding_category"
                    | "finding_severity"
                    | "finding_classification"
                    | "coverage_state"
                    | "assessment_completeness"
            )
            || !valid_match_values(&rule.match_field, &rule.match_values)
            || rule
                .quarantine_profile
                .as_deref()
                .is_some_and(|profile| !valid_schema_name(profile))
            || (rule.effect == "allow_isolated_eligibility") != rule.quarantine_profile.is_some()
        {
            return Err(PolicyError::new(PolicyErrorCode::InvalidPolicy));
        }
        previous = Some(key);
    }
    Ok(())
}

fn rule_matches(
    rule: &AdmissionRule,
    assessment: &RepositorySecurityAssessment,
    coverage: &AnalyzerCoverage,
    findings: &[SecurityFinding],
) -> bool {
    let values = rule
        .match_values
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    match rule.match_field.as_str() {
        "finding_category" => findings
            .iter()
            .any(|finding| values.contains(finding.category.as_str())),
        "finding_severity" => findings
            .iter()
            .any(|finding| values.contains(finding.severity.as_str())),
        "finding_classification" => findings
            .iter()
            .any(|finding| values.contains(finding.classification.as_str())),
        "coverage_state" => coverage
            .requirements
            .iter()
            .any(|requirement| values.contains(requirement.state.as_str())),
        "assessment_completeness" => values.contains(assessment.completeness.as_str()),
        _ => false,
    }
}

const fn effect_rank(effect: &str) -> u8 {
    match effect.as_bytes() {
        b"block" => 4,
        b"manual_review" => 3,
        b"require_analysis" => 2,
        b"allow_isolated_eligibility" => 1,
        _ => 0,
    }
}

fn canonical_unsigned(value: &str) -> Option<u64> {
    if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
        return None;
    }
    value.parse().ok()
}

fn strictly_sorted(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

fn valid_schema_name(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

fn valid_schema_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_identifier(value: &str) -> bool {
    let Some(separator) = value.find('_') else {
        return false;
    };
    (1..=32).contains(&separator)
        && (8..=128).contains(&value.len().saturating_sub(separator + 1))
        && value.as_bytes()[0].is_ascii_lowercase()
        && value[..separator]
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value[separator + 1..]
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn valid_match_values(field: &str, values: &[String]) -> bool {
    values.iter().all(|value| match field {
        "finding_category" => matches!(
            value.as_str(),
            "execution_surface"
                | "lifecycle_hook"
                | "code_download"
                | "credential_path"
                | "persistence"
                | "privilege"
                | "host_mount"
                | "network_reference"
                | "format_mismatch"
                | "analyzer_detection"
                | "unknown"
        ),
        "finding_severity" => matches!(
            value.as_str(),
            "informational" | "low" | "medium" | "high" | "critical"
        ),
        "finding_classification" => matches!(
            value.as_str(),
            "observed" | "derived" | "reputation_match" | "heuristically_suspicious" | "unknown"
        ),
        "coverage_state" => matches!(
            value.as_str(),
            "not_requested"
                | "pending"
                | "completed"
                | "partial"
                | "failed"
                | "unavailable"
                | "stale"
        ),
        "assessment_completeness" => matches!(value.as_str(), "complete" | "partial"),
        _ => false,
    })
}

fn decision_identity(decision: &RepositoryAdmissionDecision) -> Result<String, PolicyError> {
    structured_identity(
        "repository-admission-decision",
        &DecisionIdentity {
            assessment_id: &decision.assessment_id,
            workspace_snapshot: &decision.workspace_snapshot,
            policy_id: &decision.policy_id,
            policy_version: &decision.policy_version,
            policy_digest: &decision.policy_digest,
            evaluator_version: EVALUATOR_VERSION,
            decision: &decision.decision,
            matched_rule_ids: &decision.matched_rule_ids,
            missing_prerequisites: &decision.missing_prerequisites,
            quarantine_profile: &decision.quarantine_profile,
            safety_claimed: false,
            ordinary_host_execution_authorized: false,
            authority_added: false,
        },
    )
}

fn structured_identity<T: Serialize>(kind: &str, value: &T) -> Result<String, PolicyError> {
    let payload = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| PolicyError::new(PolicyErrorCode::Serialization))?;
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
    use super::*;
    use context_admission::AnalyzerRequirement;
    use jsonschema::Registry;

    const HASH_A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn assessment(
        completeness: &str,
        finding_ids: Vec<String>,
        coverage_id: &str,
    ) -> RepositorySecurityAssessment {
        let unknowns = if completeness == "complete" {
            Vec::new()
        } else {
            vec!["mandatory-analysis-incomplete".to_owned()]
        };
        let mut assessment = RepositorySecurityAssessment {
            schema_name: "repository-security-assessment".to_owned(),
            schema_version: CONTRACT_VERSION.to_owned(),
            assessment_id: String::new(),
            workspace_snapshot: HASH_B.to_owned(),
            profile_digest:
                "sha256:1fa9f737b8452d86bffa00ff0a76539c8312f2342b2a14568e270b05b3170c83".to_owned(),
            inventory_id: HASH_A.to_owned(),
            finding_ids,
            coverage_id: coverage_id.to_owned(),
            completeness: completeness.to_owned(),
            conflicts: Vec::new(),
            unknowns,
            exclusions: Vec::new(),
            safety_claimed: false,
            ordinary_host_execution_authorized: false,
            authority_added: false,
        };
        assessment.assessment_id = structured_identity(
            "repository-security-assessment",
            &serde_json::json!({
                "workspace_snapshot": assessment.workspace_snapshot,
                "profile_digest": assessment.profile_digest,
                "inventory_id": assessment.inventory_id,
                "finding_ids": assessment.finding_ids,
                "coverage_id": assessment.coverage_id,
                "completeness": assessment.completeness,
                "conflicts": assessment.conflicts,
                "unknowns": assessment.unknowns,
                "exclusions": assessment.exclusions,
                "safety_claimed": false,
                "ordinary_host_execution_authorized": false,
                "authority_added": false
            }),
        )
        .expect("assessment identity");
        assessment
    }

    fn coverage(state: &str) -> AnalyzerCoverage {
        let capability_id = "static-malware-analysis".to_owned();
        let artifact_hashes = vec![HASH_A.to_owned()];
        let reason_rule_ids = vec!["inventory-artifact-requires-analysis-v1".to_owned()];
        let requirement_digest = structured_identity(
            "analyzer-requirement",
            &serde_json::json!({
                "capability_id": capability_id,
                "artifact_hashes": artifact_hashes,
                "reason_rule_ids": reason_rule_ids,
                "mandatory": true,
                "minimum_contract_version": CONTRACT_VERSION
            }),
        )
        .expect("requirement identity");
        let mut coverage = AnalyzerCoverage {
            schema_name: "analyzer-coverage".to_owned(),
            schema_version: CONTRACT_VERSION.to_owned(),
            coverage_id: String::new(),
            workspace_snapshot: HASH_B.to_owned(),
            generated_at: "2026-08-29T17:00:00Z".to_owned(),
            requirements: vec![AnalyzerRequirement {
                requirement_id: format!("req_{}", &requirement_digest[7..39]),
                capability_id,
                artifact_hashes,
                reason_rule_ids,
                mandatory: true,
                minimum_contract_version: CONTRACT_VERSION.to_owned(),
                state: state.to_owned(),
                state_reason: if state == "completed" {
                    "normalized-analyzer-result-current"
                } else {
                    "analyzer-execution-not-authorized"
                }
                .to_owned(),
                analyzer_identity: None,
                ruleset_digest: None,
                completed_at: None,
                fresh_until: None,
                result_digest: None,
            }],
            authority_added: false,
        };
        coverage.coverage_id = structured_identity(
            "analyzer-coverage",
            &serde_json::json!({
                "workspace_snapshot": coverage.workspace_snapshot,
                "generated_at": coverage.generated_at,
                "requirements": coverage.requirements,
                "authority_added": false
            }),
        )
        .expect("coverage identity");
        coverage
    }

    fn finding(severity: &str) -> SecurityFinding {
        let mut finding = SecurityFinding {
            schema_name: "security-finding".to_owned(),
            schema_version: CONTRACT_VERSION.to_owned(),
            finding_id: String::new(),
            workspace_snapshot: HASH_B.to_owned(),
            artifact_hash: HASH_A.to_owned(),
            evidence_id: Some(HASH_B.to_owned()),
            classification: "observed".to_owned(),
            category: "execution_surface".to_owned(),
            severity: severity.to_owned(),
            confidence: "confirmed".to_owned(),
            method: "synthetic-rule".to_owned(),
            analyzer_identity: None,
            ruleset_digest: None,
            trust: "untrusted_workspace_content".to_owned(),
            limitations: vec!["synthetic-only".to_owned()],
            authority_added: false,
        };
        finding.finding_id = structured_identity(
            "security-finding",
            &serde_json::json!({
                "workspace_snapshot": finding.workspace_snapshot,
                "artifact_hash": finding.artifact_hash,
                "evidence_id": finding.evidence_id,
                "category": finding.category,
                "method": finding.method
            }),
        )
        .expect("finding identity");
        finding
    }

    fn policy() -> RepositoryAdmissionPolicy {
        let mut policy = RepositoryAdmissionPolicy {
            schema_name: "repository-admission-policy".to_owned(),
            schema_version: CONTRACT_VERSION.to_owned(),
            policy_id: "policy_reference01".to_owned(),
            policy_version: CONTRACT_VERSION.to_owned(),
            policy_digest: String::new(),
            rules: vec![
                AdmissionRule {
                    rule_id: "critical-block".to_owned(),
                    priority: "10".to_owned(),
                    effect: "block".to_owned(),
                    match_field: "finding_severity".to_owned(),
                    match_values: vec!["critical".to_owned()],
                    reason_code: "critical-finding".to_owned(),
                    quarantine_profile: None,
                },
                AdmissionRule {
                    rule_id: "medium-review".to_owned(),
                    priority: "15".to_owned(),
                    effect: "manual_review".to_owned(),
                    match_field: "finding_severity".to_owned(),
                    match_values: vec!["medium".to_owned()],
                    reason_code: "medium-finding-review".to_owned(),
                    quarantine_profile: None,
                },
                AdmissionRule {
                    rule_id: "missing-analysis".to_owned(),
                    priority: "20".to_owned(),
                    effect: "require_analysis".to_owned(),
                    match_field: "coverage_state".to_owned(),
                    match_values: vec![
                        "failed".to_owned(),
                        "stale".to_owned(),
                        "unavailable".to_owned(),
                    ],
                    reason_code: "mandatory-analysis-incomplete".to_owned(),
                    quarantine_profile: None,
                },
                AdmissionRule {
                    rule_id: "eligible-complete".to_owned(),
                    priority: "30".to_owned(),
                    effect: "allow_isolated_eligibility".to_owned(),
                    match_field: "assessment_completeness".to_owned(),
                    match_values: vec!["complete".to_owned()],
                    reason_code: "complete-for-isolated-review".to_owned(),
                    quarantine_profile: Some("disposable-quarantine-v1".to_owned()),
                },
            ],
            exception_authority: "authorized_human_outside_context_core".to_owned(),
            authority_added: false,
        };
        policy.policy_digest = policy_digest(&policy).expect("policy digest");
        policy
    }

    #[test]
    fn truth_table_is_deterministic_and_authority_neutral() {
        let policy = policy();
        let mut completed = coverage("completed");
        completed.requirements[0].analyzer_identity = Some(HASH_A.to_owned());
        completed.requirements[0].ruleset_digest = Some(HASH_B.to_owned());
        completed.requirements[0].completed_at = Some("2026-08-29T16:00:00Z".to_owned());
        completed.requirements[0].fresh_until = Some("2026-08-30T16:00:00Z".to_owned());
        completed.requirements[0].result_digest = Some(HASH_A.to_owned());
        completed.coverage_id = structured_identity(
            "analyzer-coverage",
            &serde_json::json!({
                "workspace_snapshot": completed.workspace_snapshot,
                "generated_at": completed.generated_at,
                "requirements": completed.requirements,
                "authority_added": false
            }),
        )
        .expect("completed coverage identity");
        let complete = assessment("complete", Vec::new(), &completed.coverage_id);
        let first =
            evaluate_repository_admission(&complete, &completed, &[], &policy).expect("eligible");
        let second =
            evaluate_repository_admission(&complete, &completed, &[], &policy).expect("repeat");
        assert_eq!(first, second);
        assert_eq!(first.decision, "isolated_execution_eligible");
        assert_eq!(
            first.quarantine_profile.as_deref(),
            Some("disposable-quarantine-v1")
        );
        assert!(!first.ordinary_host_execution_authorized);
        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("repository root");
        let schema_root = repository_root.join("schemas/v1");
        let common: serde_json::Value = serde_json::from_slice(
            &std::fs::read(schema_root.join("common.schema.json")).expect("common schema"),
        )
        .expect("common JSON");
        let schema: serde_json::Value = serde_json::from_slice(
            &std::fs::read(schema_root.join("repository-admission-decision.schema.json"))
                .expect("decision schema"),
        )
        .expect("decision JSON");
        let common_id = common["$id"].as_str().expect("common id").to_owned();
        let registry = Registry::new()
            .add(&common_id, common)
            .expect("register common")
            .prepare()
            .expect("prepare registry");
        let validator = jsonschema::draft202012::options()
            .with_registry(&registry)
            .build(&schema)
            .expect("compile decision schema");
        assert!(validator.is_valid(&serde_json::to_value(&first).expect("decision JSON")));

        let unavailable = coverage("unavailable");
        let partial = assessment("partial", Vec::new(), &unavailable.coverage_id);
        let incomplete = evaluate_repository_admission(&partial, &unavailable, &[], &policy)
            .expect("incomplete");
        assert_eq!(incomplete.decision, "analysis_incomplete");
        assert_eq!(
            incomplete.missing_prerequisites,
            ["static-malware-analysis"]
        );

        let medium = finding("medium");
        let review_assessment = assessment(
            "complete",
            vec![medium.finding_id.clone()],
            &completed.coverage_id,
        );
        let review =
            evaluate_repository_admission(&review_assessment, &completed, &[medium], &policy)
                .expect("manual review");
        assert_eq!(review.decision, "manual_review_required");
    }

    #[test]
    fn adding_a_blocking_finding_is_monotonic_toward_denial() {
        let policy = policy();
        let critical = finding("critical");
        let mut completed = coverage("completed");
        completed.requirements[0].analyzer_identity = Some(HASH_A.to_owned());
        completed.requirements[0].ruleset_digest = Some(HASH_B.to_owned());
        completed.requirements[0].completed_at = Some("2026-08-29T16:00:00Z".to_owned());
        completed.requirements[0].fresh_until = Some("2026-08-30T16:00:00Z".to_owned());
        completed.requirements[0].result_digest = Some(HASH_A.to_owned());
        completed.coverage_id = structured_identity(
            "analyzer-coverage",
            &serde_json::json!({
                "workspace_snapshot": completed.workspace_snapshot,
                "generated_at": completed.generated_at,
                "requirements": completed.requirements,
                "authority_added": false
            }),
        )
        .expect("completed coverage identity");
        let assessment = assessment(
            "complete",
            vec![critical.finding_id.clone()],
            &completed.coverage_id,
        );
        let decision = evaluate_repository_admission(&assessment, &completed, &[critical], &policy)
            .expect("blocked");
        assert_eq!(decision.decision, "blocked");
        assert_eq!(decision.quarantine_profile, None);
    }

    #[test]
    fn conflicting_eligibility_profiles_fail_to_manual_review() {
        let mut policy = policy();
        policy.rules.push(AdmissionRule {
            rule_id: "eligible-other-profile".to_owned(),
            priority: "40".to_owned(),
            effect: "allow_isolated_eligibility".to_owned(),
            match_field: "assessment_completeness".to_owned(),
            match_values: vec!["complete".to_owned()],
            reason_code: "complete-for-other-isolated-review".to_owned(),
            quarantine_profile: Some("disposable-quarantine-v2".to_owned()),
        });
        policy.policy_digest = policy_digest(&policy).expect("policy digest");
        let mut completed = coverage("completed");
        completed.requirements[0].analyzer_identity = Some(HASH_A.to_owned());
        completed.requirements[0].ruleset_digest = Some(HASH_B.to_owned());
        completed.requirements[0].completed_at = Some("2026-08-29T16:00:00Z".to_owned());
        completed.requirements[0].fresh_until = Some("2026-08-30T16:00:00Z".to_owned());
        completed.requirements[0].result_digest = Some(HASH_A.to_owned());
        completed.coverage_id = structured_identity(
            "analyzer-coverage",
            &serde_json::json!({
                "workspace_snapshot": completed.workspace_snapshot,
                "generated_at": completed.generated_at,
                "requirements": completed.requirements,
                "authority_added": false
            }),
        )
        .expect("completed coverage identity");
        let assessment = assessment("complete", Vec::new(), &completed.coverage_id);
        let decision =
            evaluate_repository_admission(&assessment, &completed, &[], &policy).expect("decision");
        assert_eq!(decision.decision, "manual_review_required");
        assert_eq!(decision.quarantine_profile, None);
    }

    #[test]
    fn exception_and_authority_claims_are_not_inputs() {
        let mut value = serde_json_canonicalizer::to_vec(&policy()).expect("serialize");
        let mut json: serde_json::Value = serde_json::from_slice(&value).expect("json");
        json["exception_granted"] = serde_json::Value::Bool(true);
        value = serde_json::to_vec(&json).expect("serialize mutation");
        assert!(serde_json::from_slice::<RepositoryAdmissionPolicy>(&value).is_err());

        let mut authority = policy();
        authority.authority_added = true;
        let coverage = coverage("unavailable");
        assert!(
            evaluate_repository_admission(
                &assessment("partial", Vec::new(), &coverage.coverage_id),
                &coverage,
                &[],
                &authority,
            )
            .is_err()
        );
    }

    #[test]
    fn published_policy_fixture_has_its_exact_digest() {
        let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .expect("repository root");
        let bytes = std::fs::read(
            repository_root.join("tests/conformance/v1/valid/repository-admission-policy.json"),
        )
        .expect("policy fixture");
        let fixture: RepositoryAdmissionPolicy =
            serde_json::from_slice(&bytes).expect("closed policy fixture");
        assert_eq!(
            policy_digest(&fixture).expect("policy digest"),
            fixture.policy_digest
        );
        validate_policy(&fixture).expect("valid policy fixture");
    }
}
