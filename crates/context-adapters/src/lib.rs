// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin consumer translation and governed native-read fallback policy."]

use context_core::{ContextPacket, PolicySubject, PublicErrorCode, ResourceBudget};
use context_engine::{ContextPlan, ContextPlanStep, EngineError, LocalEngine, RequestContext};
use serde::{Deserialize, Serialize};

/// Adapter contract major supported by this release.
pub const ADAPTER_CONTRACT_VERSION: &str = "1.0.0";

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
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::IncompatibleContract => "incompatible adapter contract",
            Self::Engine(_) => "context engine request failed",
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
