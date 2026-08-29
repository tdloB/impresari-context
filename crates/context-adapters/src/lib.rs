// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin consumer translation and governed native-read fallback policy."]

use std::fmt::Write as _;

use context_core::{ContextPacket, PolicySubject, PublicErrorCode, ResourceBudget};
use context_engine::{
    ContextPlan, ContextPlanStep, EngineError, LocalEngine, ProfiledContextPacket, RequestContext,
    TaskProfile,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Adapter contract major supported by this release.
pub const ADAPTER_CONTRACT_VERSION: &str = "1.0.0";

/// Client-neutral contract version for one explicitly prepared delivery packet.
pub const GUIDED_DELIVERY_CONTRACT_VERSION: &str = "1.0.0";

/// Exact Codex App Server client identity admitted for the CI-3b adapter.
pub const CODEX_APP_SERVER_CLIENT: &str = "codex";
/// Process-local, ephemeral App Server scope; this is not a desktop-thread attachment.
pub const CODEX_APP_SERVER_SCOPE: &str = "app_server_ephemeral";
/// Codex App Server version admitted by this release.
pub const CODEX_APP_SERVER_VERSION: &str = "0.150.0-alpha.8";
/// The explicit App Server lifecycle point used for one packet handoff.
pub const CODEX_APP_SERVER_LIFECYCLE_POINT: &str = "turn_start";

/// Exact GitHub Copilot CLI client identity admitted for the CI-3c adapter.
pub const COPILOT_CLI_CLIENT: &str = "github_copilot_cli";
/// Non-interactive prompt scope; this is not an interactive or VS Code session.
pub const COPILOT_CLI_SCOPE: &str = "programmatic_prompt";
/// GitHub Copilot CLI version admitted by this release.
pub const COPILOT_CLI_VERSION: &str = "1.0.80";
/// The explicit programmatic lifecycle point used for one packet handoff.
pub const COPILOT_CLI_LIFECYCLE_POINT: &str = "prompt_start";

/// Exact Claude Code client identity admitted for the CI-3d adapter.
pub const CLAUDE_CODE_CLIENT: &str = "claude_code";
/// Safe-mode non-interactive scope; this is not an interactive Claude session.
pub const CLAUDE_CODE_SCOPE: &str = "safe_mode_print";
/// Claude Code version admitted by this release.
pub const CLAUDE_CODE_VERSION: &str = "2.1.241";
/// The explicit programmatic lifecycle point used for one packet handoff.
pub const CLAUDE_CODE_LIFECYCLE_POINT: &str = "prompt_start";
/// Exact Cursor Agent client identity admitted for guided delivery.
pub const CURSOR_AGENT_CLIENT: &str = "cursor_agent";
/// Exact Cursor Agent lifecycle scope admitted for guided delivery.
pub const CURSOR_AGENT_SCOPE: &str = "ask_mode_print";
/// Exact Cursor Agent build admitted for guided delivery.
pub const CURSOR_AGENT_VERSION: &str = "2026.08.25-3e8eec8";
/// Exact Cursor Agent lifecycle point admitted for guided delivery.
pub const CURSOR_AGENT_LIFECYCLE_POINT: &str = "prompt_start";

/// Consumer policy for acquiring repository context.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRequirement {
    /// A valid Impresari Context packet is mandatory; fallback is prohibited.
    Required,
    /// Prefer a packet but permit a separately authorized native read only when
    /// the engine is genuinely unavailable or does not support the capability.
    Preferred,
    /// The consumer deliberately disabled the integration and owns all native-read policy.
    Disabled,
}

/// Bounded reason a consumer may present to fallback policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackReason {
    /// The engine process/library could not be reached or initialized.
    EngineUnavailable,
    /// The requested capability is explicitly unsupported.
    UnsupportedCapability,
    /// Engine evidence or packet integrity validation failed.
    IntegrityFailure,
    /// Policy denied the engine request.
    PolicyDenied,
    /// The workspace or packet is stale.
    StaleState,
    /// A resource or context budget was exceeded.
    ResourceLimit,
    /// A safe internal error did not match another category.
    InternalFailure,
}

/// Governed decision; this never performs or authorizes a filesystem read.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FallbackDecision {
    /// Schema discriminator.
    pub schema_name: String,
    /// Adapter contract version.
    pub schema_version: String,
    /// Whether the consumer may consider its own native-read path.
    pub native_read_may_be_considered: bool,
    /// Always true when consideration is allowed: consumer policy must authorize it.
    pub consumer_authorization_required: bool,
    /// Stable reason for allow/deny.
    pub decision_reason: String,
    /// Always false: an adapter decision adds no filesystem authority.
    pub authority_added: bool,
}

/// Returns a fail-closed fallback decision.
#[must_use]
pub fn decide_native_read_fallback(
    requirement: ContextRequirement,
    reason: FallbackReason,
) -> FallbackDecision {
    let allowed = match requirement {
        ContextRequirement::Required => false,
        ContextRequirement::Preferred => matches!(
            reason,
            FallbackReason::EngineUnavailable | FallbackReason::UnsupportedCapability
        ),
        ContextRequirement::Disabled => true,
    };
    FallbackDecision {
        schema_name: "native-read-fallback-decision".into(),
        schema_version: ADAPTER_CONTRACT_VERSION.into(),
        native_read_may_be_considered: allowed,
        consumer_authorization_required: allowed,
        decision_reason: if allowed {
            "consumer_policy_evaluation_required"
        } else {
            "fallback_prohibited"
        }
        .into(),
        authority_added: false,
    }
}

/// Maps a safe engine error category to fallback policy without inspecting private causes.
#[must_use]
pub const fn fallback_reason(error: &EngineError) -> FallbackReason {
    match error.envelope().code {
        PublicErrorCode::UnsupportedCapability | PublicErrorCode::UnsupportedArtifact => {
            FallbackReason::UnsupportedCapability
        }
        PublicErrorCode::IntegrityFailure
        | PublicErrorCode::CorruptCache
        | PublicErrorCode::IncompatibleCache => FallbackReason::IntegrityFailure,
        PublicErrorCode::PolicyDenied
        | PublicErrorCode::RootNotAllowed
        | PublicErrorCode::SymlinkEscape => FallbackReason::PolicyDenied,
        PublicErrorCode::StaleState | PublicErrorCode::EvidenceUnavailable => {
            FallbackReason::StaleState
        }
        PublicErrorCode::BudgetTooSmall
        | PublicErrorCode::BudgetExceeded
        | PublicErrorCode::ResourceLimit
        | PublicErrorCode::PartialResult => FallbackReason::ResourceLimit,
        PublicErrorCode::PathNotFound
        | PublicErrorCode::UnsupportedFilesystemObject
        | PublicErrorCode::InvalidInput
        | PublicErrorCode::InternalFailure => FallbackReason::InternalFailure,
    }
}

/// Minimal OS-shaped request translated into the public engine contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsContextRequest {
    /// Version the consumer expects.
    pub adapter_contract_version: String,
    /// Trusted opaque request identifier supplied by the orchestrator.
    pub request_id: String,
    /// Trusted opaque audit event identifier supplied by the orchestrator.
    pub event_id: String,
    /// Opaque OS consumer identity.
    pub consumer_id: String,
    /// Opaque OS role; the core still owns mechanical authorization.
    pub role: String,
    /// Task-specific purpose, never an instruction authority.
    pub purpose: String,
    /// Trusted normalized UTC operation time.
    pub occurred_at: String,
    /// Ordered exact retrieval plan.
    pub steps: Vec<ContextPlanStep>,
    /// Hard model-neutral budget.
    pub budget: ResourceBudget,
}

/// Successful OS adapter result. It conveys context but no routing or approval authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OsContextResponse {
    /// Adapter contract version actually used.
    pub adapter_contract_version: String,
    /// Immutable verified packet.
    pub packet: ContextPacket,
    /// Always false: the response does not direct agents or approve work.
    pub orchestration_authority_added: bool,
}

/// Explicit caller-declared request for one reference delivery preparation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GuidedDeliveryIntent {
    /// Version expected by the caller.
    pub adapter_contract_version: String,
    /// Fixed client identity from the narrow guided-delivery allowlist.
    pub client: String,
    /// Fixed client scope from the narrow guided-delivery allowlist.
    pub scope: String,
    /// Client adapter version.
    pub client_version: String,
    /// Documented lifecycle point from the narrow guided-delivery allowlist.
    pub lifecycle_point: String,
    /// Explicit one-delivery consent. False means no packet is prepared.
    pub consent: bool,
    /// Opaque validated request identifier.
    pub request_id: String,
    /// Opaque validated event identifier.
    pub event_id: String,
    /// Opaque local consumer identity.
    pub consumer_id: String,
    /// Local policy role.
    pub role: String,
    /// Bounded policy purpose.
    pub purpose: String,
    /// Normalized UTC operation time.
    pub occurred_at: String,
    /// Caller-declared immutable workspace identity.
    pub workspace_identity: String,
    /// Caller-declared immutable workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Explicit deterministic planning profile.
    pub task_profile: TaskProfile,
    /// Bounded user-declared query.
    pub query: String,
    /// Hard evidence budget.
    pub budget: ResourceBudget,
}

/// Source-free delivery receipt; no client delivery happens in this reference adapter.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GuidedDeliveryReceipt {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// `prepared` or `no_delivery`.
    pub outcome: String,
    /// Stable reason for the outcome.
    pub reason_code: String,
    /// Declared client identity.
    pub client: String,
    /// Declared client scope.
    pub scope: String,
    /// Declared client adapter version.
    pub client_version: String,
    /// Declared lifecycle point.
    pub lifecycle_point: String,
    /// Opaque request identity bound to the result.
    pub request_id: String,
    /// Opaque event identity bound to the result.
    pub event_id: String,
    /// Workspace identity when it was verified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_identity: Option<String>,
    /// Packet identity when prepared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_id: Option<String>,
    /// Planner identity when prepared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    /// Snapshot identity when prepared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_snapshot: Option<String>,
    /// Policy decision identity when prepared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_decision: Option<String>,
    /// Always false: this reference adapter does not contact a client.
    pub client_io_performed: bool,
    /// Always false: preparing a packet adds no authority.
    pub authority_added: bool,
}

/// Result of one explicit delivery preparation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GuidedDeliveryResult {
    /// Immutable planner output when preparation succeeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepared: Option<ProfiledContextPacket>,
    /// Exact canonical bytes of the shared planner packet when preparation succeeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet_bytes: Option<Vec<u8>>,
    /// Visible prepared/no-delivery receipt.
    pub receipt: GuidedDeliveryReceipt,
}

/// Prepares one exact planner packet without contacting a client lifecycle surface.
///
/// # Errors
///
/// Returns an engine error only after the explicit intent passes local contract
/// validation. Disabled or unsupported reference identities return a visible
/// no-delivery result without invoking the engine.
pub fn prepare_guided_delivery(
    engine: &mut LocalEngine,
    intent: GuidedDeliveryIntent,
) -> Result<GuidedDeliveryResult, AdapterError> {
    if intent.adapter_contract_version != GUIDED_DELIVERY_CONTRACT_VERSION {
        return Ok(no_delivery(&intent, "incompatible_contract"));
    }
    if !intent.consent {
        return Ok(no_delivery(&intent, "explicit_consent_required"));
    }
    if !guided_delivery_identity_is_supported(&intent) {
        return Ok(no_delivery(&intent, "unsupported_client_lifecycle"));
    }
    if !delivery_intent_text_is_valid(&intent) {
        return Ok(no_delivery(&intent, "invalid_declared_intent"));
    }
    let context = RequestContext {
        request_id: intent.request_id.clone(),
        event_id: intent.event_id.clone(),
        subject: PolicySubject {
            caller_id: intent.consumer_id.clone(),
            role: intent.role.clone(),
            purpose: intent.purpose.clone(),
        },
        occurred_at: intent.occurred_at.clone(),
    };
    let snapshot_context = derived_snapshot_status_context(&context);
    let snapshot = match engine.snapshot_status_against(
        &snapshot_context,
        intent.budget.clone(),
        Some(&intent.workspace_snapshot),
    ) {
        Ok(value) => value,
        Err(error)
            if matches!(
                error.envelope().code,
                PublicErrorCode::StaleState | PublicErrorCode::EvidenceUnavailable
            ) =>
        {
            return Ok(no_delivery(&intent, "snapshot_unavailable"));
        }
        Err(error) => return Err(AdapterError::Engine(error)),
    };
    if snapshot.workspace_identity != intent.workspace_identity {
        return Ok(no_delivery(&intent, "workspace_identity_mismatch"));
    }
    if snapshot.state != "current" || snapshot.freshness != "current" {
        return Ok(no_delivery(&intent, "snapshot_stale"));
    }
    let prepared_reason_code = prepared_reason_code(&intent);
    let client = intent.client.clone();
    let scope = intent.scope.clone();
    let client_version = intent.client_version.clone();
    let lifecycle_point = intent.lifecycle_point.clone();
    let prepared = engine
        .build_profiled_context(&context, intent.task_profile, &intent.query, intent.budget)
        .map_err(AdapterError::Engine)?;
    let packet_bytes =
        context_core::packet_bytes(&prepared.packet).map_err(|_| AdapterError::Serialization)?;
    let receipt = GuidedDeliveryReceipt {
        schema_name: "guided-delivery-receipt".into(),
        schema_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
        outcome: "prepared".into(),
        reason_code: prepared_reason_code.into(),
        client,
        scope,
        client_version,
        lifecycle_point,
        request_id: context.request_id,
        event_id: context.event_id,
        workspace_identity: Some(snapshot.workspace_identity),
        packet_id: Some(prepared.packet.packet_id.clone()),
        plan_id: Some(prepared.plan.plan_id.clone()),
        workspace_snapshot: Some(prepared.packet.workspace_snapshot.clone()),
        policy_decision: Some(prepared.packet.policy_decision.clone()),
        client_io_performed: false,
        authority_added: false,
    };
    Ok(GuidedDeliveryResult {
        prepared: Some(prepared),
        packet_bytes: Some(packet_bytes),
        receipt,
    })
}

fn guided_delivery_identity_is_supported(intent: &GuidedDeliveryIntent) -> bool {
    (intent.client == "reference"
        && intent.scope == "process_local"
        && intent.client_version == GUIDED_DELIVERY_CONTRACT_VERSION
        && intent.lifecycle_point == "prepare")
        || (intent.client == CODEX_APP_SERVER_CLIENT
            && intent.scope == CODEX_APP_SERVER_SCOPE
            && intent.client_version == CODEX_APP_SERVER_VERSION
            && intent.lifecycle_point == CODEX_APP_SERVER_LIFECYCLE_POINT)
        || (intent.client == COPILOT_CLI_CLIENT
            && intent.scope == COPILOT_CLI_SCOPE
            && intent.client_version == COPILOT_CLI_VERSION
            && intent.lifecycle_point == COPILOT_CLI_LIFECYCLE_POINT)
        || (intent.client == CLAUDE_CODE_CLIENT
            && intent.scope == CLAUDE_CODE_SCOPE
            && intent.client_version == CLAUDE_CODE_VERSION
            && intent.lifecycle_point == CLAUDE_CODE_LIFECYCLE_POINT)
        || (intent.client == CURSOR_AGENT_CLIENT
            && intent.scope == CURSOR_AGENT_SCOPE
            && intent.client_version == CURSOR_AGENT_VERSION
            && intent.lifecycle_point == CURSOR_AGENT_LIFECYCLE_POINT)
}

fn prepared_reason_code(intent: &GuidedDeliveryIntent) -> &'static str {
    match intent.client.as_str() {
        CODEX_APP_SERVER_CLIENT => "codex_app_server_packet_prepared",
        COPILOT_CLI_CLIENT => "copilot_cli_packet_prepared",
        CLAUDE_CODE_CLIENT => "claude_code_packet_prepared",
        CURSOR_AGENT_CLIENT => "cursor_agent_packet_prepared",
        _ => "reference_packet_prepared",
    }
}

fn delivery_intent_text_is_valid(intent: &GuidedDeliveryIntent) -> bool {
    [&intent.request_id, &intent.event_id, &intent.consumer_id]
        .iter()
        .all(|value| identifier_is_valid(value))
        && role_is_valid(&intent.role)
        && !intent.purpose.is_empty()
        && intent.purpose.len() <= 256
        && !intent.purpose.contains('\0')
        && !intent.query.is_empty()
        && intent.query.len() <= 4096
        && !intent.query.contains('\0')
        && context_core::validate_utc_timestamp(&intent.occurred_at).is_ok()
        && sha256_identity_is_valid(&intent.workspace_identity)
        && sha256_identity_is_valid(&intent.workspace_snapshot)
        && budget_is_valid(&intent.budget)
}

fn no_delivery(intent: &GuidedDeliveryIntent, reason_code: &str) -> GuidedDeliveryResult {
    GuidedDeliveryResult {
        prepared: None,
        packet_bytes: None,
        receipt: GuidedDeliveryReceipt {
            schema_name: "guided-delivery-receipt".into(),
            schema_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
            outcome: "no_delivery".into(),
            reason_code: reason_code.into(),
            client: intent.client.clone(),
            scope: intent.scope.clone(),
            client_version: intent.client_version.clone(),
            lifecycle_point: intent.lifecycle_point.clone(),
            request_id: intent.request_id.clone(),
            event_id: intent.event_id.clone(),
            workspace_identity: None,
            packet_id: None,
            plan_id: None,
            workspace_snapshot: None,
            policy_decision: None,
            client_io_performed: false,
            authority_added: false,
        },
    }
}

fn identifier_is_valid(value: &str) -> bool {
    let Some((prefix, suffix)) = value.split_once('_') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.len() <= 32
        && prefix.as_bytes()[0].is_ascii_lowercase()
        && prefix.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
        && (8..=128).contains(&suffix.len())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn role_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_lowercase()
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || (index > 0 && (byte.is_ascii_digit() || byte == b'_' || byte == b'-'))
        })
}

fn sha256_identity_is_valid(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn budget_is_valid(budget: &ResourceBudget) -> bool {
    let values = [
        &budget.requested,
        &budget.max_evidence_items,
        &budget.max_files,
        &budget.max_excerpt_bytes_per_item,
        &budget.max_matches,
        &budget.max_traversal_depth,
        &budget.max_elapsed_ms,
        &budget.max_memory_bytes,
    ];
    let mut parsed = [0_u64; 8];
    for (index, value) in values.into_iter().enumerate() {
        let Ok(number) = value.parse::<u64>() else {
            return false;
        };
        if number.to_string() != *value {
            return false;
        }
        parsed[index] = number;
    }
    budget.unit_kind == "utf8_bytes"
        && budget.hard
        && budget.policy_profile == context_core::POLICY_PROFILE
        && ResourceBudget::conservative(
            parsed[0], parsed[1], parsed[2], parsed[3], parsed[4], parsed[5], parsed[6], parsed[7],
        )
        .is_ok()
}

fn derived_snapshot_status_context(context: &RequestContext) -> RequestContext {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0guided-delivery-snapshot-status-event\0");
    hasher.update(context.event_id.as_bytes());
    let mut event_id = String::from("evt_");
    for byte in hasher.finalize() {
        write!(event_id, "{byte:02x}").expect("string write");
    }
    RequestContext {
        request_id: context.request_id.clone(),
        event_id,
        subject: context.subject.clone(),
        occurred_at: context.occurred_at.clone(),
    }
}

/// Builds context through the same public engine method used by non-OS consumers.
///
/// # Errors
///
/// Fails closed for incompatible adapter versions or any engine failure.
pub fn acquire_for_os(
    engine: &mut LocalEngine,
    request: OsContextRequest,
) -> Result<OsContextResponse, AdapterError> {
    if request.adapter_contract_version != ADAPTER_CONTRACT_VERSION {
        return Err(AdapterError::IncompatibleContract);
    }
    let context = RequestContext {
        request_id: request.request_id,
        event_id: request.event_id,
        subject: PolicySubject {
            caller_id: request.consumer_id,
            role: request.role,
            purpose: request.purpose,
        },
        occurred_at: request.occurred_at,
    };
    let packet = engine
        .build_planned_context(
            &context,
            &ContextPlan {
                steps: request.steps,
            },
            request.budget,
        )
        .map_err(AdapterError::Engine)?;
    Ok(OsContextResponse {
        adapter_contract_version: ADAPTER_CONTRACT_VERSION.into(),
        packet,
        orchestration_authority_added: false,
    })
}

/// Adapter-layer failure retaining the public engine envelope but no private cause.
#[derive(Debug)]
pub enum AdapterError {
    /// Consumer and adapter major contract do not match.
    IncompatibleContract,
    /// The public engine operation failed.
    Engine(EngineError),
    /// Canonical packet serialization failed before any client delivery.
    Serialization,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::IncompatibleContract => "incompatible adapter contract",
            Self::Engine(_) => "context engine request failed",
            Self::Serialization => "context packet serialization failed",
        })
    }
}

impl std::error::Error for AdapterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_is_narrow_and_never_adds_authority() {
        for requirement in [ContextRequirement::Required, ContextRequirement::Preferred] {
            for reason in [
                FallbackReason::IntegrityFailure,
                FallbackReason::PolicyDenied,
                FallbackReason::StaleState,
                FallbackReason::ResourceLimit,
                FallbackReason::InternalFailure,
            ] {
                let decision = decide_native_read_fallback(requirement, reason);
                assert!(!decision.native_read_may_be_considered);
                assert!(!decision.authority_added);
            }
        }
        let unavailable = decide_native_read_fallback(
            ContextRequirement::Preferred,
            FallbackReason::EngineUnavailable,
        );
        assert!(unavailable.native_read_may_be_considered);
        assert!(unavailable.consumer_authorization_required);
        assert!(!unavailable.authority_added);
    }
}
