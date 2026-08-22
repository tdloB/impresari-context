// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Adapter-neutral capability service shared by the CLI and libraries."]

use std::{
    fmt,
    fs::{self, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{
    AuditOutcome, Capability, ContextPacket, ErrorEnvelope, EvidenceRecord, PacketDraft,
    PacketValidationResult, PolicyDecision, PolicyOutcome, PolicySubject, PublicErrorCode,
    RecoveryAction, ResourceBudget, audit_event, build_packet, decide, error_envelope,
    packet_bytes, packet_validation_result, validate_packet,
};
use context_retrieval::{
    RetrievalErrorCode, SearchBudget, build_lexical_generation_bounded, evidence_record,
    expand_evidence_record, lookup_exact_path, search_filename, search_lexical, search_literal,
};
use context_store::{AuditRetention, AuditStore, CacheErrorCode, WorkspaceCache};
use context_structural::{
    FactClass, GRAPH_VERSION, GraphFileInput, PROTOCOL_VERSION, RESOLVER_VERSION, StructuralError,
    StructuralGraph, StructuralLanguage, StructuralQueryResult, WorkerLauncher, WorkerPath,
    WorkerRequest, build_graph_with_unknowns, query_graph,
};
use context_workspace::{
    AuthorizedWorkspace, DiscoveryPolicy, PathIdentity, SkipReason, WorkspaceErrorCode,
    WorkspaceSnapshot,
};
use serde::{Deserialize, Serialize};

const CONTRACT_VERSION: &str = "1.0.0";
const ENGINE_VERSION: &str = "0.0.0";

/// Caller-controlled data for one capability invocation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestContext {
    /// Unique opaque request identifier.
    pub request_id: String,
    /// Unique opaque audit-event identifier.
    pub event_id: String,
    /// Policy subject data.
    pub subject: PolicySubject,
    /// Trusted normalized UTC operation time.
    pub occurred_at: String,
}

/// Explicit local engine configuration.
#[derive(Clone, Debug)]
pub struct EngineConfig {
    /// Cache/audit root, always separate from source.
    pub cache_root: PathBuf,
    /// Bounded discovery policy.
    pub discovery: DiscoveryPolicy,
    /// Explicit audit retention policy.
    pub audit_retention: AuditRetention,
}

/// Public workspace-open response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkspaceHandle {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Opaque session-local handle.
    pub workspace_handle: String,
    /// Opaque workspace identity.
    pub workspace_identity: String,
    /// Policy decision identity.
    pub policy_decision: String,
    /// Current handle state.
    pub state: String,
}

/// One aggregated snapshot omission category.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SkippedSummary {
    /// Stable omission reason.
    pub reason: String,
    /// Omitted object count.
    pub count: String,
    /// Whether this makes the snapshot partial.
    pub affects_completeness: bool,
}

/// Public snapshot status response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SnapshotStatus {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Workspace identity.
    pub workspace_identity: String,
    /// Snapshot identity.
    pub snapshot_id: String,
    /// Snapshot state.
    pub state: String,
    /// Freshness state.
    pub freshness: String,
    /// Completeness state.
    pub completeness: String,
    /// Discovery-policy identity.
    pub discovery_policy: String,
    /// Engine version.
    pub engine_version: String,
    /// Eligible file count.
    pub eligible_files: String,
    /// Eligible byte count.
    pub eligible_bytes: String,
    /// Optional bounded Git HEAD revision.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_revision: Option<String>,
    /// `unknown` for Git roots or `not_applicable` otherwise.
    pub working_tree: String,
    /// Explicit omission categories.
    pub skipped: Vec<SkippedSummary>,
}

/// Supported MVP query strategies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryKind {
    /// Resolve one exact relative path.
    ExactPath,
    /// Match a filename/display path.
    Filename,
    /// Match exact source bytes.
    Literal,
    /// Use lexical candidates followed by exact source verification.
    Lexical,
}

/// Public bounded search result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SearchResponse {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Request identity.
    pub request_id: String,
    /// Snapshot identity.
    pub snapshot_id: String,
    /// Freshness state.
    pub freshness: String,
    /// Completeness state.
    pub completeness: String,
    /// Exact evidence matches.
    pub matches: Vec<context_core::EvidenceRecord>,
    /// Whether results were limited.
    pub truncated: bool,
    /// Stable limit reasons.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub truncation_reasons: Vec<String>,
    /// Explicit unavailable or unsupported semantics.
    pub unknowns: Vec<String>,
}

/// Versioned receipt for a no-overwrite local packet handoff.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HandoffReceipt {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Preserved packet identity.
    pub packet_id: String,
    /// Safe destination filename, never an ambient absolute path.
    pub destination_name: String,
    /// Exact canonical bytes written.
    pub exported_bytes: String,
    /// Always false: export cannot add capability or evidence authority.
    pub authority_added: bool,
}

/// Safe service failure. Private causes are intentionally not serialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EngineError {
    envelope: Box<ErrorEnvelope>,
}

impl EngineError {
    /// Returns the adapter-neutral structured error.
    #[must_use]
    pub const fn envelope(&self) -> &ErrorEnvelope {
        &self.envelope
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.envelope.message)
    }
}

impl std::error::Error for EngineError {}

/// Wraps a validated core error envelope for a thin local adapter.
///
/// Adapters should construct the input through [`context_core::error_envelope`]
/// and must not include local paths, queries, source, or diagnostic causes.
#[must_use]
pub fn adapter_error(envelope: ErrorEnvelope) -> EngineError {
    EngineError {
        envelope: Box::new(envelope),
    }
}

/// Stateful local session. All public operations pass through one policy path.
pub struct LocalEngine {
    config: EngineConfig,
    workspace: AuthorizedWorkspace,
    snapshot: Option<WorkspaceSnapshot>,
    cache: Option<WorkspaceCache>,
    audit: AuditStore,
    handle: String,
}

impl LocalEngine {
    /// Opens an explicit workspace and records the policy decision and audit.
    ///
    /// # Errors
    ///
    /// Returns a safe structured failure for policy, workspace, or audit errors.
    pub fn open(
        config: EngineConfig,
        context: &RequestContext,
        root: &Path,
    ) -> Result<(Self, WorkspaceHandle), EngineError> {
        let started = Instant::now();
        let decision = authorize(context, None, Capability::WorkspaceOpen, None)?;
        let mut audit = AuditStore::open(&config.cache_root)
            .map_err(|error| cache_error(context, Capability::WorkspaceOpen, error.code(), None))?;
        let workspace = match AuthorizedWorkspace::open(root) {
            Ok(workspace) => workspace,
            Err(error) => {
                let public =
                    workspace_error(context, Capability::WorkspaceOpen, error.code(), None, None);
                record_event(
                    &mut audit,
                    &config.audit_retention,
                    context,
                    &decision,
                    Capability::WorkspaceOpen,
                    AuditOutcome::Failed,
                    None,
                    None,
                    elapsed_ms(started),
                )?;
                return Err(public);
            }
        };
        let identity = workspace.identity().to_owned();
        let handle = format!("wsp_{}", &identity[7..23]);
        let mut engine = Self {
            config,
            workspace,
            snapshot: None,
            cache: None,
            audit,
            handle: handle.clone(),
        };
        engine.record(
            context,
            &decision,
            Capability::WorkspaceOpen,
            AuditOutcome::Allowed,
            elapsed_ms(started),
        )?;
        let response = WorkspaceHandle {
            schema_name: "workspace-handle".into(),
            schema_version: CONTRACT_VERSION.into(),
            workspace_handle: handle,
            workspace_identity: identity,
            policy_decision: decision.decision_id,
            state: "ready".into(),
        };
        Ok((engine, response))
    }

    /// Builds a fresh deterministic snapshot under the gateway.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, workspace, or audit failure.
    pub fn build_snapshot(
        &mut self,
        context: &RequestContext,
        budget: ResourceBudget,
    ) -> Result<SnapshotStatus, EngineError> {
        let started = Instant::now();
        let (discovery, max_elapsed) = bounded_discovery(self.config.discovery, &budget)
            .map_err(|code| core_error(context, Capability::SnapshotBuild, code, self.ids()))?;
        let decision = self.authorize(context, Capability::SnapshotBuild, Some(budget))?;
        let result = self
            .workspace
            .snapshot_bounded(discovery, max_elapsed)
            .map_err(|error| {
                self.workspace_failure(context, Capability::SnapshotBuild, error.code())
            })
            .map(|snapshot| {
                let status = snapshot_status(&snapshot);
                self.snapshot = Some(snapshot);
                status
            });
        self.finalize(
            context,
            &decision,
            Capability::SnapshotBuild,
            AuditOutcome::Allowed,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds one snapshot-bound structural graph through fresh isolated workers.
    ///
    /// Exact source remains under the workspace capability. Each supported file
    /// is read and reverified by the control process, then only its bounded bytes
    /// and lossless relative identity are sent to the pinned worker. Unsupported
    /// artifacts remain explicit graph unknowns.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, stale-state, workspace, worker, resource, or
    /// graph-validation failure. Partial worker output is never returned.
    pub fn build_structure(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
    ) -> Result<StructuralGraph, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::StructureBuild, Some(budget.clone()))?;
        let result = self.build_structure_internal(context, budget, launcher, started);
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::StructureBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    fn build_structure_internal(
        &self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
        started: Instant,
    ) -> Result<StructuralGraph, EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::StaleState,
                "workspace snapshot is unavailable",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        let limits = structural_limits(context, budget, self.ids())?;
        let mut files = Vec::new();
        let mut unknowns = Vec::new();
        for artifact in snapshot
            .artifacts
            .iter()
            .take(usize::try_from(limits.files).unwrap_or(usize::MAX))
        {
            if u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX) >= limits.elapsed_ms
            {
                return Err(failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::ResourceLimit,
                    "structural elapsed-time limit exceeded",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    None,
                ));
            }
            let Some(language) = structural_language(&artifact.path.display_path) else {
                unknowns.push("unsupported_structural_language".into());
                continue;
            };
            let exact = self
                .workspace
                .read_exact(&artifact.path, artifact.size_bytes)
                .map_err(|error| {
                    self.workspace_failure(context, Capability::StructureBuild, error.code())
                })?;
            if exact.content_hash != artifact.content_hash {
                return Err(failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "workspace changed during structural analysis",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RefreshSnapshot),
                ));
            }
            let request = structural_request(context, language, exact, limits);
            let response = launcher.execute(&request).map_err(|error| {
                structural_failure(
                    context,
                    error,
                    self.workspace.identity(),
                    &snapshot.snapshot_id,
                )
            })?;
            files.push(GraphFileInput {
                path: request.path,
                response,
            });
        }
        if u64::try_from(snapshot.artifacts.len()).unwrap_or(u64::MAX) > limits.files {
            unknowns.push("structural_file_limit_reached".into());
        }
        build_graph_with_unknowns(&snapshot.snapshot_id, files, unknowns).map_err(|error| {
            structural_failure(
                context,
                error,
                self.workspace.identity(),
                &snapshot.snapshot_id,
            )
        })
    }

    /// Traverses a current snapshot-bound structural graph through the audited gateway.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, stale-state, input, integrity, or resource failure.
    pub fn query_structure(
        &mut self,
        context: &RequestContext,
        graph: &StructuralGraph,
        start_node: &str,
        edge_kinds: &[String],
        budget: &ResourceBudget,
    ) -> Result<StructuralQueryResult, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::StructureQuery, Some(budget.clone()))?;
        let result = self.query_structure_internal(context, graph, start_node, edge_kinds, budget);
        let outcome = result.as_ref().map_or(AuditOutcome::Failed, |value| {
            if value.truncated {
                AuditOutcome::Limited
            } else {
                AuditOutcome::Allowed
            }
        });
        self.finalize(
            context,
            &decision,
            Capability::StructureQuery,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    fn query_structure_internal(
        &self,
        context: &RequestContext,
        graph: &StructuralGraph,
        start_node: &str,
        edge_kinds: &[String],
        budget: &ResourceBudget,
    ) -> Result<StructuralQueryResult, EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::StaleState,
                "workspace snapshot is unavailable",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        if graph.workspace_snapshot != snapshot.snapshot_id {
            return Err(failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::StaleState,
                "structural graph is stale",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RebuildIndex),
            ));
        }
        let max_depth = u32::try_from(budget.max_traversal_depth_u64().map_err(|error| {
            core_error(
                context,
                Capability::StructureQuery,
                error.code(),
                self.ids(),
            )
        })?)
        .map_err(|_| {
            failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::InvalidInput,
                "invalid structural query budget",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                None,
            )
        })?;
        let maximum = budget.max_matches.parse::<u32>().map_err(|_| {
            failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::InvalidInput,
                "invalid structural query budget",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                None,
            )
        })?;
        let result = query_graph(graph, start_node, edge_kinds, max_depth, maximum, maximum)
            .map_err(|error| {
                structural_query_failure(
                    context,
                    error,
                    self.workspace.identity(),
                    &snapshot.snapshot_id,
                )
            })?;
        let output_limit = budget.requested.parse::<usize>().map_err(|_| {
            failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::InvalidInput,
                "invalid structural query budget",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                None,
            )
        })?;
        if serde_json::to_vec(&result).map_or(true, |bytes| bytes.len() > output_limit) {
            return Err(failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::BudgetExceeded,
                "structural query output budget exceeded",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::IncreaseBudget),
            ));
        }
        Ok(result)
    }

    /// Reports the current in-session snapshot through the gateway.
    ///
    /// # Errors
    ///
    /// Returns stale-state when no snapshot has been built.
    pub fn snapshot_status(
        &mut self,
        context: &RequestContext,
        budget: ResourceBudget,
    ) -> Result<SnapshotStatus, EngineError> {
        self.snapshot_status_against(context, budget, None)
    }

    /// Reports current or stale status relative to an expected snapshot.
    ///
    /// # Errors
    ///
    /// Returns stale-state when no snapshot has been built.
    pub fn snapshot_status_against(
        &mut self,
        context: &RequestContext,
        budget: ResourceBudget,
        expected_snapshot: Option<&str>,
    ) -> Result<SnapshotStatus, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::SnapshotStatus, Some(budget))?;
        let result = if expected_snapshot.is_some_and(|expected| !valid_sha256(expected)) {
            Err(failure(
                context,
                Capability::SnapshotStatus,
                PublicErrorCode::InvalidInput,
                "invalid expected snapshot identity",
                Some(self.workspace.identity()),
                None,
                None,
            ))
        } else {
            self.snapshot
                .as_ref()
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::SnapshotStatus,
                        PublicErrorCode::StaleState,
                        "workspace snapshot is unavailable",
                        Some(self.workspace.identity()),
                        None,
                        Some(RecoveryAction::RefreshSnapshot),
                    )
                })
                .map(|snapshot| {
                    let mut status = snapshot_status(snapshot);
                    if expected_snapshot.is_some_and(|expected| expected != snapshot.snapshot_id) {
                        status.state = "stale".into();
                        status.freshness = "stale".into();
                    }
                    status
                })
        };
        self.finalize(
            context,
            &decision,
            Capability::SnapshotStatus,
            AuditOutcome::Allowed,
            result,
            elapsed_ms(started),
        )
    }

    /// Executes a bounded filename, literal, or lexical query.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, retrieval, cache, stale-state, or audit failure.
    pub fn search(
        &mut self,
        context: &RequestContext,
        kind: QueryKind,
        query: &str,
        budget: &ResourceBudget,
    ) -> Result<SearchResponse, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::CodeSearch, Some(budget.clone()))?;
        let result = self.search_internal(context, Capability::CodeSearch, kind, query, budget);
        let outcome = result.as_ref().map_or(AuditOutcome::Failed, audit_outcome);
        self.finalize(
            context,
            &decision,
            Capability::CodeSearch,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds a bounded immutable context packet using the same retrieval path.
    ///
    /// # Errors
    ///
    /// Returns a structured gateway, retrieval, packet, or audit failure.
    pub fn build_context(
        &mut self,
        context: &RequestContext,
        kind: QueryKind,
        query: &str,
        budget: ResourceBudget,
    ) -> Result<ContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget.clone()))?;
        let result = self
            .search_internal(context, Capability::ContextBuild, kind, query, &budget)
            .and_then(|search| {
                build_packet(PacketDraft {
                    workspace_identity: self.workspace.identity().to_owned(),
                    workspace_snapshot: search.snapshot_id,
                    request_id: context.request_id.clone(),
                    purpose: context.subject.purpose.clone(),
                    created_at: context.occurred_at.clone(),
                    policy_decision: decision.decision_id.clone(),
                    budget,
                    evidence: search.matches,
                    assumptions: Vec::new(),
                    conflicts: Vec::new(),
                    unknowns: search.unknowns,
                    redactions: Vec::new(),
                })
                .map_err(|error| {
                    core_error(context, Capability::ContextBuild, error.code(), self.ids())
                })
            });
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            AuditOutcome::Allowed,
            result,
            elapsed_ms(started),
        )
    }

    /// Reauthorizes and expands exact evidence from current source.
    ///
    /// # Errors
    ///
    /// Returns a structured failure for stale, forged, malformed, over-budget,
    /// unauthorized, or unavailable evidence.
    pub fn expand_evidence(
        &mut self,
        context: &RequestContext,
        evidence: &EvidenceRecord,
        before_bytes: u64,
        after_bytes: u64,
        max_bytes: u64,
        budget: ResourceBudget,
    ) -> Result<EvidenceRecord, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::EvidenceExpand, Some(budget))?;
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::EvidenceExpand,
                    PublicErrorCode::StaleState,
                    "workspace snapshot is unavailable",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })
            .and_then(|snapshot| {
                expand_evidence_record(
                    &self.workspace,
                    snapshot,
                    evidence,
                    before_bytes,
                    after_bytes,
                    max_bytes,
                )
                .map_err(|error| {
                    retrieval_error(
                        context,
                        Capability::EvidenceExpand,
                        error.code(),
                        self.ids(),
                    )
                })
            });
        self.finalize(
            context,
            &decision,
            Capability::EvidenceExpand,
            AuditOutcome::Allowed,
            result,
            elapsed_ms(started),
        )
    }

    /// Validates packet integrity, authorization, freshness, and every exact
    /// evidence excerpt against current source.
    ///
    /// # Errors
    ///
    /// Returns a structured service failure if policy or audit cannot complete.
    pub fn validate_context_packet(
        &mut self,
        context: &RequestContext,
        packet: &ContextPacket,
        budget: ResourceBudget,
    ) -> Result<PacketValidationResult, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextValidate, Some(budget))?;
        let snapshot = self.snapshot.as_ref();
        let authorized = packet.workspace_identity == self.workspace.identity();
        let evidence_available = authorized
            && snapshot.is_some_and(|snapshot| {
                packet.observed_evidence.iter().all(|evidence| {
                    expand_evidence_record(&self.workspace, snapshot, evidence, 0, 0, 65_536)
                        .is_ok()
                })
            });
        let result = packet_validation_result(
            packet,
            authorized,
            snapshot.map(|value| value.snapshot_id.as_str()),
            evidence_available,
            &context.occurred_at,
        )
        .map_err(|error| {
            core_error(
                context,
                Capability::ContextValidate,
                error.code(),
                self.ids(),
            )
        });
        let outcome = result.as_ref().map_or(AuditOutcome::Failed, |validation| {
            if matches!(
                validation.status,
                context_core::PacketValidationStatus::ValidCurrent
            ) {
                AuditOutcome::Allowed
            } else {
                AuditOutcome::Limited
            }
        });
        self.finalize(
            context,
            &decision,
            Capability::ContextValidate,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Writes the exact canonical packet to an explicit non-workspace export
    /// root using atomic no-overwrite publication.
    ///
    /// # Errors
    ///
    /// Returns a structured failure for invalid roots/names, stale or corrupt
    /// packets, budget excess, existing destinations, I/O, policy, or audit.
    pub fn export_handoff(
        &mut self,
        context: &RequestContext,
        packet: &ContextPacket,
        budget: &ResourceBudget,
        export_root: &Path,
        destination_name: &str,
    ) -> Result<HandoffReceipt, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::HandoffExport, Some(budget.clone()))?;
        let result =
            self.export_handoff_inner(context, packet, budget, export_root, destination_name);
        self.finalize(
            context,
            &decision,
            Capability::HandoffExport,
            AuditOutcome::Allowed,
            result,
            elapsed_ms(started),
        )
    }

    #[allow(clippy::too_many_lines)]
    fn export_handoff_inner(
        &self,
        context: &RequestContext,
        packet: &ContextPacket,
        budget: &ResourceBudget,
        export_root: &Path,
        destination_name: &str,
    ) -> Result<HandoffReceipt, EngineError> {
        validate_packet(packet).map_err(|error| {
            core_error(context, Capability::HandoffExport, error.code(), self.ids())
        })?;
        let current = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::StaleState,
                "workspace snapshot is unavailable",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        if packet.workspace_identity != self.workspace.identity() {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::PolicyDenied,
                "packet belongs to another workspace",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                None,
            ));
        }
        if packet.workspace_snapshot != current.snapshot_id {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::StaleState,
                "packet snapshot is stale",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                Some(RecoveryAction::RefreshSnapshot),
            ));
        }
        let bytes = packet_bytes(packet).map_err(|error| {
            core_error(context, Capability::HandoffExport, error.code(), self.ids())
        })?;
        let output_limit = budget.requested.parse::<u64>().map_err(|_| {
            core_error(
                context,
                Capability::HandoffExport,
                context_core::CoreErrorCode::InvalidInput,
                self.ids(),
            )
        })?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > output_limit {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::BudgetExceeded,
                "export budget exceeded",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                Some(RecoveryAction::IncreaseBudget),
            ));
        }
        prepare_export_root(&self.workspace, export_root).map_err(|code| {
            failure(
                context,
                Capability::HandoffExport,
                code,
                "export destination is not allowed",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                Some(RecoveryAction::ReduceScope),
            )
        })?;
        if destination_name.is_empty()
            || destination_name.len() > 255
            || destination_name.contains(['/', '\\', '\0'])
            || matches!(destination_name, "." | "..")
        {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::InvalidInput,
                "invalid export filename",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                None,
            ));
        }
        let target = export_root.join(destination_name);
        if target.try_exists().unwrap_or(true) {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::InvalidInput,
                "export destination already exists",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                None,
            ));
        }
        let temporary = export_root.join(format!(
            ".impresari-{}-{}.tmp",
            &packet.packet_id[7..19],
            context.request_id.replace('_', "-")
        ));
        write_no_overwrite(&temporary, &target, &bytes).map_err(|_| {
            failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::InternalFailure,
                "export write failed",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                Some(RecoveryAction::Retry),
            )
        })?;
        let written = fs::read(&target).map_err(|_| {
            failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::InternalFailure,
                "export verification failed",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                None,
            )
        })?;
        if written != bytes {
            return Err(failure(
                context,
                Capability::HandoffExport,
                PublicErrorCode::IntegrityFailure,
                "export verification failed",
                Some(self.workspace.identity()),
                Some(&current.snapshot_id),
                None,
            ));
        }
        Ok(HandoffReceipt {
            schema_name: "handoff-export".into(),
            schema_version: CONTRACT_VERSION.into(),
            packet_id: packet.packet_id.clone(),
            destination_name: destination_name.into(),
            exported_bytes: bytes.len().to_string(),
            authority_added: false,
        })
    }

    fn search_internal(
        &mut self,
        context: &RequestContext,
        capability: Capability,
        kind: QueryKind,
        query: &str,
        budget: &ResourceBudget,
    ) -> Result<SearchResponse, EngineError> {
        let search_budget = search_budget(budget)
            .map_err(|code| core_error(context, capability, code, self.ids()))?;
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                capability,
                PublicErrorCode::StaleState,
                "workspace snapshot is unavailable",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        let result = match kind {
            QueryKind::ExactPath => {
                let path = PathIdentity::from_portable_relative_path(query).map_err(|error| {
                    workspace_error(
                        context,
                        capability,
                        error.code(),
                        Some(self.workspace.identity()),
                        Some(&snapshot.snapshot_id),
                    )
                })?;
                lookup_exact_path(&self.workspace, snapshot, &path, search_budget)
            }
            QueryKind::Filename => search_filename(&self.workspace, snapshot, query, search_budget),
            QueryKind::Literal => {
                search_literal(&self.workspace, snapshot, query.as_bytes(), search_budget)
            }
            QueryKind::Lexical => {
                if self.cache.is_none() {
                    let mut cache =
                        WorkspaceCache::open(&self.config.cache_root, self.workspace.identity())
                            .map_err(|error| {
                                cache_error(context, capability, error.code(), Some(self.ids()))
                            })?;
                    let max_memory = budget.max_memory_bytes_u64().map_err(|error| {
                        core_error(context, capability, error.code(), self.ids())
                    })?;
                    build_lexical_generation_bounded(
                        &self.workspace,
                        snapshot,
                        &mut cache,
                        max_memory,
                    )
                    .map_err(|error| {
                        retrieval_error(context, capability, error.code(), self.ids())
                    })?;
                    self.cache = Some(cache);
                }
                search_lexical(
                    &self.workspace,
                    snapshot,
                    self.cache.as_ref().expect("cache initialized"),
                    query,
                    search_budget,
                )
            }
        }
        .map_err(|error| retrieval_error(context, capability, error.code(), self.ids()))?;
        let response = SearchResponse {
            schema_name: "search-result".into(),
            schema_version: CONTRACT_VERSION.into(),
            request_id: context.request_id.clone(),
            snapshot_id: snapshot.snapshot_id.clone(),
            freshness: "current".into(),
            completeness: if snapshot.complete && !result.truncated {
                "complete".into()
            } else {
                "partial".into()
            },
            matches: result.matches.iter().map(evidence_record).collect(),
            truncated: result.truncated,
            truncation_reasons: result
                .truncation_reasons
                .iter()
                .map(ToString::to_string)
                .collect(),
            unknowns: if snapshot.complete {
                Vec::new()
            } else {
                vec!["snapshot_partial".into()]
            },
        };
        bound_search_response(response, budget)
            .map_err(|code| core_error(context, capability, code, self.ids()))
    }

    fn authorize(
        &self,
        context: &RequestContext,
        capability: Capability,
        budget: Option<ResourceBudget>,
    ) -> Result<PolicyDecision, EngineError> {
        authorize(context, Some(self.workspace.identity()), capability, budget)
    }

    fn record(
        &mut self,
        context: &RequestContext,
        decision: &PolicyDecision,
        capability: Capability,
        outcome: AuditOutcome,
        duration_ms: u64,
    ) -> Result<(), EngineError> {
        record_event(
            &mut self.audit,
            &self.config.audit_retention,
            context,
            decision,
            capability,
            outcome,
            Some(self.workspace.identity()),
            self.snapshot
                .as_ref()
                .map(|snapshot| snapshot.snapshot_id.as_str()),
            duration_ms,
        )
    }

    fn finalize<T>(
        &mut self,
        context: &RequestContext,
        decision: &PolicyDecision,
        capability: Capability,
        success_outcome: AuditOutcome,
        result: Result<T, EngineError>,
        duration_ms: u64,
    ) -> Result<T, EngineError> {
        let outcome = if result.is_ok() {
            success_outcome
        } else {
            AuditOutcome::Failed
        };
        self.record(context, decision, capability, outcome, duration_ms)?;
        result
    }

    fn workspace_failure(
        &self,
        context: &RequestContext,
        capability: Capability,
        code: WorkspaceErrorCode,
    ) -> EngineError {
        workspace_error(
            context,
            capability,
            code,
            Some(self.workspace.identity()),
            self.snapshot
                .as_ref()
                .map(|snapshot| snapshot.snapshot_id.as_str()),
        )
    }

    fn ids(&self) -> (Option<&str>, Option<&str>) {
        (
            Some(self.workspace.identity()),
            self.snapshot
                .as_ref()
                .map(|snapshot| snapshot.snapshot_id.as_str()),
        )
    }

    /// Returns the session-local opaque workspace handle.
    #[must_use]
    pub fn workspace_handle(&self) -> &str {
        &self.handle
    }
}

fn authorize(
    context: &RequestContext,
    workspace: Option<&str>,
    capability: Capability,
    budget: Option<ResourceBudget>,
) -> Result<PolicyDecision, EngineError> {
    let decision = decide(
        &context.request_id,
        &context.subject,
        workspace,
        capability,
        budget,
        &context.occurred_at,
    )
    .map_err(|error| core_error(context, capability, error.code(), (workspace, None)))?;
    if decision.outcome == PolicyOutcome::Deny {
        Err(failure(
            context,
            capability,
            PublicErrorCode::PolicyDenied,
            "capability denied by local policy",
            workspace,
            None,
            Some(RecoveryAction::RequestAuthorization),
        ))
    } else {
        Ok(decision)
    }
}

#[allow(clippy::too_many_arguments)]
fn record_event(
    audit: &mut AuditStore,
    retention: &AuditRetention,
    context: &RequestContext,
    decision: &PolicyDecision,
    capability: Capability,
    outcome: AuditOutcome,
    workspace: Option<&str>,
    snapshot: Option<&str>,
    duration_ms: u64,
) -> Result<(), EngineError> {
    let limits = decision
        .effective_budget
        .clone()
        .unwrap_or_else(default_budget);
    let event = audit_event(
        &context.event_id,
        &context.request_id,
        &context.occurred_at,
        workspace,
        snapshot,
        capability,
        outcome,
        &decision.decision_id,
        limits,
        duration_ms,
        ENGINE_VERSION,
    )
    .map_err(|error| core_error(context, capability, error.code(), (workspace, snapshot)))?;
    audit.append(&event, retention).map_err(|error| {
        cache_error(
            context,
            capability,
            error.code(),
            Some((workspace, snapshot)),
        )
    })?;
    Ok(())
}

fn snapshot_status(snapshot: &WorkspaceSnapshot) -> SnapshotStatus {
    SnapshotStatus {
        schema_name: "snapshot-status".into(),
        schema_version: CONTRACT_VERSION.into(),
        workspace_identity: snapshot.workspace_identity.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        state: if snapshot.complete {
            "current"
        } else {
            "partial"
        }
        .into(),
        freshness: "current".into(),
        completeness: if snapshot.complete {
            "complete"
        } else {
            "partial"
        }
        .into(),
        discovery_policy: snapshot.discovery_policy.clone(),
        engine_version: ENGINE_VERSION.into(),
        eligible_files: snapshot.artifacts.len().to_string(),
        eligible_bytes: snapshot.eligible_bytes.to_string(),
        repository_revision: snapshot.repository_revision.clone(),
        working_tree: snapshot.working_tree.into(),
        skipped: snapshot
            .skipped
            .iter()
            .map(|(reason, count)| SkippedSummary {
                reason: skip_reason(*reason).into(),
                count: count.to_string(),
                affects_completeness: true,
            })
            .collect(),
    }
}

fn skip_reason(reason: SkipReason) -> &'static str {
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

fn search_budget(budget: &ResourceBudget) -> Result<SearchBudget, context_core::CoreErrorCode> {
    let parse = |value: &str| {
        value
            .parse::<u64>()
            .map_err(|_| context_core::CoreErrorCode::InvalidInput)
    };
    SearchBudget::with_memory_limit(
        parse(&budget.max_files)?,
        parse(&budget.max_matches)?,
        parse(&budget.max_excerpt_bytes_per_item)?,
        Duration::from_millis(parse(&budget.max_elapsed_ms)?),
        parse(&budget.max_memory_bytes)?,
    )
    .map_err(|_| context_core::CoreErrorCode::ResourceLimit)
}

fn bound_search_response(
    mut response: SearchResponse,
    budget: &ResourceBudget,
) -> Result<SearchResponse, context_core::CoreErrorCode> {
    let maximum = budget
        .requested
        .parse::<usize>()
        .map_err(|_| context_core::CoreErrorCode::InvalidInput)?;
    loop {
        let bytes = serde_json::to_vec(&response)
            .map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
        if bytes.len() <= maximum {
            return Ok(response);
        }
        if response.matches.pop().is_none() {
            return Err(context_core::CoreErrorCode::BudgetTooSmall);
        }
        response.truncated = true;
        response.completeness = "partial".into();
        response.truncation_reasons.push("output_budget".into());
        response.truncation_reasons.sort();
        response.truncation_reasons.dedup();
    }
}

fn bounded_discovery(
    configured: DiscoveryPolicy,
    budget: &ResourceBudget,
) -> Result<(DiscoveryPolicy, Duration), context_core::CoreErrorCode> {
    let map = |_| context_core::CoreErrorCode::InvalidInput;
    let max_files = budget.max_files_u64().map_err(map)?;
    let max_depth = budget.max_traversal_depth_u64().map_err(map)?;
    let max_memory = budget.max_memory_bytes_u64().map_err(map)?;
    let max_elapsed = budget.max_elapsed_ms_u64().map_err(map)?;
    let max_total_bytes = configured.max_total_bytes.min(max_memory);
    let policy = DiscoveryPolicy::new(
        configured.max_files.min(max_files),
        max_total_bytes,
        configured.max_file_bytes.min(max_total_bytes),
        configured.max_depth.min(max_depth),
    )
    .map_err(|_| context_core::CoreErrorCode::ResourceLimit)?;
    Ok((policy, Duration::from_millis(max_elapsed)))
}

fn default_budget() -> ResourceBudget {
    ResourceBudget::conservative(1024, 1, 1, 1, 1, 1, 1, 1_048_576)
        .expect("constant minimum budget")
}

fn elapsed_ms(started: Instant) -> u64 {
    let nanoseconds = started.elapsed().as_nanos();
    let rounded_up = nanoseconds.saturating_add(999_999) / 1_000_000;
    u64::try_from(rounded_up).unwrap_or(u64::MAX)
}

fn audit_outcome(result: &SearchResponse) -> AuditOutcome {
    if result.truncated || result.completeness == "partial" {
        AuditOutcome::Limited
    } else {
        AuditOutcome::Allowed
    }
}

fn prepare_export_root(
    workspace: &AuthorizedWorkspace,
    root: &Path,
) -> Result<(), PublicErrorCode> {
    if root.as_os_str().is_empty() || root.parent().is_none() {
        return Err(PublicErrorCode::RootNotAllowed);
    }
    if root
        .try_exists()
        .map_err(|_| PublicErrorCode::RootNotAllowed)?
    {
        if fs::symlink_metadata(root)
            .map_err(|_| PublicErrorCode::RootNotAllowed)?
            .file_type()
            .is_symlink()
        {
            return Err(PublicErrorCode::SymlinkEscape);
        }
    } else {
        fs::create_dir_all(root).map_err(|_| PublicErrorCode::RootNotAllowed)?;
    }
    let resolved = root
        .canonicalize()
        .map_err(|_| PublicErrorCode::RootNotAllowed)?;
    if resolved.parent().is_none()
        || std::env::var_os("HOME").is_some_and(|home| resolved == Path::new(&home))
        || workspace
            .is_same_root(&resolved)
            .map_err(|_| PublicErrorCode::RootNotAllowed)?
    {
        return Err(PublicErrorCode::RootNotAllowed);
    }
    set_private_directory(&resolved).map_err(|_| PublicErrorCode::RootNotAllowed)
}

fn write_no_overwrite(temporary: &Path, target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(temporary)?;
    set_private_file(temporary)?;
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = fs::remove_file(temporary);
        return Err(error);
    }
    if let Err(error) = fs::hard_link(temporary, target) {
        let _ = fs::remove_file(temporary);
        return Err(error);
    }
    fs::remove_file(temporary)
}

#[cfg(unix)]
fn set_private_directory(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}

#[cfg(windows)]
#[allow(clippy::unnecessary_wraps)] // Keep the cross-platform fallible helper contract uniform.
fn set_private_directory(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_private_file(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(windows)]
#[allow(clippy::unnecessary_wraps)] // Keep the cross-platform fallible helper contract uniform.
fn set_private_file(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[derive(Clone, Copy)]
struct StructuralLimits {
    files: u64,
    depth: u32,
    elapsed_ms: u64,
    facts: u32,
    response_bytes: u32,
}

fn structural_limits(
    context: &RequestContext,
    budget: &ResourceBudget,
    ids: (Option<&str>, Option<&str>),
) -> Result<StructuralLimits, EngineError> {
    let map_core = |error: context_core::CoreError| {
        core_error(context, Capability::StructureBuild, error.code(), ids)
    };
    let parse_u32 = |value: &str| {
        value
            .parse::<u64>()
            .ok()
            .and_then(|parsed| u32::try_from(parsed).ok())
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid structural resource budget",
                    ids.0,
                    ids.1,
                    None,
                )
            })
    };
    Ok(StructuralLimits {
        files: budget.max_files_u64().map_err(map_core)?,
        depth: u32::try_from(budget.max_traversal_depth_u64().map_err(map_core)?).map_err(
            |_| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid structural resource budget",
                    ids.0,
                    ids.1,
                    None,
                )
            },
        )?,
        elapsed_ms: budget.max_elapsed_ms_u64().map_err(map_core)?,
        facts: parse_u32(&budget.max_matches)?,
        response_bytes: parse_u32(&budget.requested)?,
    })
}

fn structural_request(
    context: &RequestContext,
    language: StructuralLanguage,
    exact: context_workspace::ExactRead,
    limits: StructuralLimits,
) -> WorkerRequest {
    WorkerRequest {
        schema_name: "structural-worker-request".into(),
        schema_version: PROTOCOL_VERSION.into(),
        request_id: context.request_id.clone(),
        language,
        path: WorkerPath {
            display_path: exact.path.display_path.clone(),
            platform_family: exact.path.platform_family.to_owned(),
            unit_encoding: exact.path.unit_encoding.to_owned(),
            relative_units_base64url: exact.path.relative_units_base64url.clone(),
        },
        content_hash: exact.content_hash,
        source_base64url: URL_SAFE_NO_PAD.encode(exact.bytes),
        fact_classes: vec![
            FactClass::Declaration,
            FactClass::Contains,
            FactClass::Import,
            FactClass::Export,
            FactClass::Call,
        ],
        max_facts: limits.facts,
        max_nesting_depth: limits.depth,
        max_response_bytes: limits.response_bytes,
        parser_version: "tree-sitter-0.26.12".into(),
        grammar_version: grammar_version(language).into(),
        resolver_version: RESOLVER_VERSION.into(),
        graph_version: GRAPH_VERSION.into(),
    }
}

fn structural_language(path: &str) -> Option<StructuralLanguage> {
    match Path::new(path).extension().and_then(|value| value.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("tsx") => Some(StructuralLanguage::Tsx),
        Some(extension) if extension.eq_ignore_ascii_case("ts") => {
            Some(StructuralLanguage::TypeScript)
        }
        Some(extension) if extension.eq_ignore_ascii_case("jsx") => Some(StructuralLanguage::Jsx),
        Some(extension)
            if extension.eq_ignore_ascii_case("js")
                || extension.eq_ignore_ascii_case("mjs")
                || extension.eq_ignore_ascii_case("cjs") =>
        {
            Some(StructuralLanguage::JavaScript)
        }
        _ => None,
    }
}

const fn grammar_version(language: StructuralLanguage) -> &'static str {
    match language {
        StructuralLanguage::TypeScript | StructuralLanguage::Tsx => "tree-sitter-typescript-0.23.2",
        StructuralLanguage::JavaScript | StructuralLanguage::Jsx => "tree-sitter-javascript-0.25.0",
    }
}

fn structural_failure(
    context: &RequestContext,
    error: StructuralError,
    workspace: &str,
    snapshot: &str,
) -> EngineError {
    let (code, message, recovery) = match error {
        StructuralError::InvalidRequest | StructuralError::ContractMismatch => (
            PublicErrorCode::IntegrityFailure,
            "structural response validation failed",
            RecoveryAction::RebuildIndex,
        ),
        StructuralError::ResourceLimit | StructuralError::Timeout => (
            PublicErrorCode::ResourceLimit,
            "structural resource limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        StructuralError::WorkerIdentity => (
            PublicErrorCode::IntegrityFailure,
            "structural worker identity mismatch",
            RecoveryAction::None,
        ),
        StructuralError::ParserFailure | StructuralError::Io | StructuralError::WorkerFailure => (
            PublicErrorCode::InternalFailure,
            "structural worker failed",
            RecoveryAction::Retry,
        ),
    };
    failure(
        context,
        Capability::StructureBuild,
        code,
        message,
        Some(workspace),
        Some(snapshot),
        Some(recovery),
    )
}

fn structural_query_failure(
    context: &RequestContext,
    error: StructuralError,
    workspace: &str,
    snapshot: &str,
) -> EngineError {
    let (code, message, recovery) = match error {
        StructuralError::InvalidRequest => (
            PublicErrorCode::InvalidInput,
            "invalid structural query",
            RecoveryAction::None,
        ),
        StructuralError::ContractMismatch => (
            PublicErrorCode::IntegrityFailure,
            "structural graph validation failed",
            RecoveryAction::RebuildIndex,
        ),
        StructuralError::ResourceLimit | StructuralError::Timeout => (
            PublicErrorCode::ResourceLimit,
            "structural query resource limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        StructuralError::WorkerIdentity
        | StructuralError::ParserFailure
        | StructuralError::Io
        | StructuralError::WorkerFailure => (
            PublicErrorCode::InternalFailure,
            "structural query failed",
            RecoveryAction::Retry,
        ),
    };
    failure(
        context,
        Capability::StructureQuery,
        code,
        message,
        Some(workspace),
        Some(snapshot),
        Some(recovery),
    )
}

fn retrieval_error(
    context: &RequestContext,
    capability: Capability,
    code: RetrievalErrorCode,
    ids: (Option<&str>, Option<&str>),
) -> EngineError {
    let (public, message, recovery) = match code {
        RetrievalErrorCode::InvalidInput => (
            PublicErrorCode::InvalidInput,
            "invalid search request",
            RecoveryAction::None,
        ),
        RetrievalErrorCode::StaleState => (
            PublicErrorCode::StaleState,
            "workspace snapshot is stale",
            RecoveryAction::RefreshSnapshot,
        ),
        RetrievalErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "search resource limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        RetrievalErrorCode::EvidenceUnavailable => (
            PublicErrorCode::EvidenceUnavailable,
            "verified evidence is unavailable",
            RecoveryAction::RefreshSnapshot,
        ),
        RetrievalErrorCode::CacheFailure => (
            PublicErrorCode::CorruptCache,
            "derived search cache failed",
            RecoveryAction::RebuildIndex,
        ),
    };
    failure(
        context,
        capability,
        public,
        message,
        ids.0,
        ids.1,
        Some(recovery),
    )
}

fn workspace_error(
    context: &RequestContext,
    capability: Capability,
    code: WorkspaceErrorCode,
    workspace: Option<&str>,
    snapshot: Option<&str>,
) -> EngineError {
    let (public, message, recovery) = match code {
        WorkspaceErrorCode::PathNotFound => (
            PublicErrorCode::PathNotFound,
            "path not found",
            RecoveryAction::None,
        ),
        WorkspaceErrorCode::RootNotDirectory | WorkspaceErrorCode::PathOutsideRoot => (
            PublicErrorCode::RootNotAllowed,
            "workspace root is not allowed",
            RecoveryAction::RequestAuthorization,
        ),
        WorkspaceErrorCode::InvalidPathIdentity => (
            PublicErrorCode::InvalidInput,
            "invalid path identity",
            RecoveryAction::None,
        ),
        WorkspaceErrorCode::SymlinkRejected => (
            PublicErrorCode::SymlinkEscape,
            "symbolic link is not allowed",
            RecoveryAction::ReduceScope,
        ),
        WorkspaceErrorCode::UnsupportedObject => (
            PublicErrorCode::UnsupportedFilesystemObject,
            "unsupported filesystem object",
            RecoveryAction::ReduceScope,
        ),
        WorkspaceErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "workspace resource limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        WorkspaceErrorCode::ChangedDuringRead => (
            PublicErrorCode::StaleState,
            "workspace changed during read",
            RecoveryAction::RefreshSnapshot,
        ),
        WorkspaceErrorCode::IoFailure => (
            PublicErrorCode::InternalFailure,
            "workspace operation failed",
            RecoveryAction::Retry,
        ),
    };
    failure(
        context,
        capability,
        public,
        message,
        workspace,
        snapshot,
        Some(recovery),
    )
}

fn cache_error(
    context: &RequestContext,
    capability: Capability,
    code: CacheErrorCode,
    ids: Option<(Option<&str>, Option<&str>)>,
) -> EngineError {
    let (workspace, snapshot) = ids.unwrap_or((None, None));
    let (public, message, recovery) = match code {
        CacheErrorCode::InvalidCacheRoot => (
            PublicErrorCode::RootNotAllowed,
            "cache root is not allowed",
            RecoveryAction::ReduceScope,
        ),
        CacheErrorCode::WriterBusy => (
            PublicErrorCode::InternalFailure,
            "local store is busy",
            RecoveryAction::Retry,
        ),
        CacheErrorCode::IncompatibleCache => (
            PublicErrorCode::IncompatibleCache,
            "local store is incompatible",
            RecoveryAction::RebuildIndex,
        ),
        CacheErrorCode::CorruptCache => (
            PublicErrorCode::CorruptCache,
            "local store is corrupt",
            RecoveryAction::RebuildIndex,
        ),
        CacheErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "local store limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        CacheErrorCode::StorageFailure => (
            PublicErrorCode::InternalFailure,
            "local store operation failed",
            RecoveryAction::Retry,
        ),
    };
    failure(
        context,
        capability,
        public,
        message,
        workspace,
        snapshot,
        Some(recovery),
    )
}

fn core_error(
    context: &RequestContext,
    capability: Capability,
    code: context_core::CoreErrorCode,
    ids: (Option<&str>, Option<&str>),
) -> EngineError {
    let (public, message, recovery) = match code {
        context_core::CoreErrorCode::InvalidInput => (
            PublicErrorCode::InvalidInput,
            "invalid request",
            RecoveryAction::None,
        ),
        context_core::CoreErrorCode::BudgetTooSmall => (
            PublicErrorCode::BudgetTooSmall,
            "packet budget is too small",
            RecoveryAction::IncreaseBudget,
        ),
        context_core::CoreErrorCode::ResourceLimit => (
            PublicErrorCode::ResourceLimit,
            "resource limit exceeded",
            RecoveryAction::ReduceScope,
        ),
        context_core::CoreErrorCode::CanonicalizationFailure => (
            PublicErrorCode::InternalFailure,
            "response serialization failed",
            RecoveryAction::None,
        ),
        context_core::CoreErrorCode::IntegrityFailure => (
            PublicErrorCode::IntegrityFailure,
            "integrity verification failed",
            RecoveryAction::RefreshSnapshot,
        ),
    };
    failure(
        context,
        capability,
        public,
        message,
        ids.0,
        ids.1,
        Some(recovery),
    )
}

fn failure(
    context: &RequestContext,
    capability: Capability,
    code: PublicErrorCode,
    message: &str,
    workspace: Option<&str>,
    snapshot: Option<&str>,
    recovery: Option<RecoveryAction>,
) -> EngineError {
    let request_id = if valid_identifier(&context.request_id) {
        context.request_id.as_str()
    } else {
        "req_invalid00"
    };
    let envelope = error_envelope(
        code,
        message,
        matches!(
            recovery,
            Some(RecoveryAction::Retry | RecoveryAction::RefreshSnapshot)
        ),
        capability,
        request_id,
        workspace,
        snapshot,
        false,
        recovery,
    )
    .expect("safe constant error envelope");
    EngineError {
        envelope: Box::new(envelope),
    }
}

fn valid_identifier(value: &str) -> bool {
    let Some((prefix, suffix)) = value.split_once('_') else {
        return false;
    };
    !prefix.is_empty()
        && suffix.len() >= 8
        && suffix.len() <= 128
        && prefix.bytes().all(|byte| byte.is_ascii_lowercase())
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::validate_packet;
    use jsonschema::Registry;
    use serde::Serialize;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);
    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-engine-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create test root");
            Self(path)
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn request(sequence: u64, purpose: &str) -> RequestContext {
        RequestContext {
            request_id: format!("req_{sequence:08}"),
            event_id: format!("evt_{sequence:08}"),
            subject: PolicySubject {
                caller_id: "caller_12345678".into(),
                role: "local_user".into(),
                purpose: purpose.into(),
            },
            occurred_at: format!("2026-08-21T00:00:{sequence:02}Z"),
        }
    }

    fn budget() -> ResourceBudget {
        ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 536_870_912)
            .expect("budget")
    }

    fn assert_schema<T: Serialize>(name: &str, value: &T) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../schemas/v1");
        let (schema_name, fragment) = name.split_once('#').unwrap_or((name, ""));
        let registry_document: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("registry.json")).expect("schema registry"))
                .expect("registry JSON");
        let mut registry = Registry::new();
        for entry in registry_document["schemas"].as_array().expect("schemas") {
            let path = entry["path"].as_str().expect("schema path");
            if path.ends_with(".schema.json") {
                let schema: serde_json::Value =
                    serde_json::from_slice(&fs::read(root.join(path)).expect("schema file"))
                        .expect("schema JSON");
                let id = schema["$id"].as_str().expect("schema id").to_owned();
                registry = registry.add(id, schema).expect("register schema");
            }
        }
        let registry = registry.prepare().expect("prepare registry");
        let schema: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(schema_name)).expect("target schema"))
                .expect("target schema JSON");
        let schema = if fragment.is_empty() {
            schema
        } else {
            let id = schema["$id"].as_str().expect("target schema id");
            serde_json::json!({"$ref": format!("{id}#{fragment}")})
        };
        let validator = jsonschema::draft202012::options()
            .with_registry(&registry)
            .should_validate_formats(true)
            .build(&schema)
            .expect("compile schema");
        validator
            .validate(&serde_json::to_value(value).expect("response JSON"))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
    }

    fn assert_recovery_validation_and_export(
        engine: &mut LocalEngine,
        packet: &ContextPacket,
        export_root: &Path,
    ) {
        let expanded = engine
            .expand_evidence(
                &request(9, "evidence_recovery"),
                &packet.observed_evidence[0],
                2,
                2,
                16,
                budget(),
            )
            .expect("expand");
        assert_eq!(
            expanded.evidence_id,
            packet.observed_evidence[0].evidence_id
        );
        assert_eq!(
            engine
                .validate_context_packet(&request(10, "validation"), packet, budget())
                .expect("validate")
                .status,
            context_core::PacketValidationStatus::ValidCurrent
        );
        let mut corrupt = packet.clone();
        corrupt.purpose = "tampered".into();
        assert_eq!(
            engine
                .validate_context_packet(&request(11, "validation"), &corrupt, budget())
                .expect("corrupt status")
                .status,
            context_core::PacketValidationStatus::Corrupt
        );
        let receipt = engine
            .export_handoff(
                &request(12, "handoff"),
                packet,
                &budget(),
                export_root,
                "packet.json",
            )
            .expect("export");
        assert_schema("handoff-export.schema.json", &receipt);
        assert_eq!(receipt.packet_id, packet.packet_id);
        assert!(!receipt.authority_added);
        assert_eq!(
            fs::read(export_root.join("packet.json")).expect("export bytes"),
            packet_bytes(packet).expect("canonical packet")
        );
        assert_eq!(
            engine
                .export_handoff(
                    &request(13, "handoff"),
                    packet,
                    &budget(),
                    export_root,
                    "packet.json",
                )
                .expect_err("no overwrite")
                .envelope()
                .code,
            PublicErrorCode::InvalidInput
        );
    }

    #[test]
    fn one_gateway_drives_snapshot_search_index_and_packet_building() {
        let source = TestRoot::new("source");
        let cache = TestRoot::new("cache");
        let export = TestRoot::new("export");
        fs::write(source.0.join("sample.rs"), b"pub fn alpha() { beta(); }\n")
            .expect("source file");
        let original = fs::read(source.0.join("sample.rs")).expect("original");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(100, 1_048_576, 65_536, 16).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, handle) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        assert_schema("workspace-handle.schema.json", &handle);
        assert_eq!(engine.workspace_handle(), handle.workspace_handle);
        let snapshot = engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("snapshot");
        assert_schema("snapshot-status.schema.json", &snapshot);
        assert_eq!(snapshot.state, "current");
        assert_eq!(
            engine
                .snapshot_status(&request(3, "status"), budget())
                .expect("status")
                .snapshot_id,
            snapshot.snapshot_id
        );
        let literal = engine
            .search(&request(4, "search"), QueryKind::Literal, "beta", &budget())
            .expect("literal");
        assert_schema("search.schema.json#/$defs/result", &literal);
        assert_eq!(literal.matches.len(), 1);
        assert_eq!(literal.matches[0].span.start_byte, "17");
        let lexical = engine
            .search(
                &request(5, "search"),
                QueryKind::Lexical,
                "alpha",
                &budget(),
            )
            .expect("lexical");
        assert_eq!(lexical.matches.len(), 1);
        let filename = engine
            .search(
                &request(6, "search"),
                QueryKind::Filename,
                "SAMPLE",
                &budget(),
            )
            .expect("filename");
        assert_eq!(filename.matches.len(), 1);
        let exact_path = engine
            .search(
                &request(7, "search"),
                QueryKind::ExactPath,
                "sample.rs",
                &budget(),
            )
            .expect("exact path");
        assert_eq!(exact_path.matches.len(), 1);
        let packet = engine
            .build_context(
                &request(8, "implementation_review"),
                QueryKind::Literal,
                "alpha",
                budget(),
            )
            .expect("packet");
        validate_packet(&packet).expect("valid packet");
        assert_eq!(packet.purpose, "implementation_review");
        assert_recovery_validation_and_export(&mut engine, &packet, &export.0);
        assert_eq!(
            fs::read(source.0.join("sample.rs")).expect("after"),
            original
        );
        assert_eq!(
            fs::read_dir(&source.0).expect("source entries").count(),
            1,
            "engine must not add source-workspace files"
        );
        assert!(cache.0.join("audit/audit.sqlite3").is_file());
        assert!(cache.0.join("workspaces").is_dir());
        drop(engine);
        let audit = AuditStore::open(&cache.0).expect("reopen audit");
        let events = audit.recent(100).expect("audit events");
        assert!(events.len() >= 13);
        assert!(events.iter().all(|event| {
            event
                .duration_ms
                .parse::<u64>()
                .is_ok_and(|duration| duration > 0)
        }));
    }

    #[test]
    fn errors_are_structured_safe_and_do_not_echo_query_or_path() {
        let source = TestRoot::new("source-errors");
        let cache = TestRoot::new("cache-errors");
        fs::write(source.0.join("sample.txt"), b"safe").expect("source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 1024, 1024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 10, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        let error = engine
            .search(
                &request(2, "search"),
                QueryKind::Literal,
                "secret-query",
                &budget(),
            )
            .expect_err("snapshot required");
        assert_eq!(error.envelope().code, PublicErrorCode::StaleState);
        assert_schema("error-envelope.schema.json", error.envelope());
        let serialized = format!("{error:?}");
        assert!(!serialized.contains("secret-query"));
        assert!(!serialized.contains(source.0.to_string_lossy().as_ref()));
        drop(engine);
        let audit = AuditStore::open(&cache.0).expect("reopen audit");
        let events = audit.recent(10).expect("audit events");
        assert!(events.iter().any(|event| {
            event.event_id == "evt_00000002" && event.outcome == AuditOutcome::Failed
        }));
    }

    #[test]
    fn request_memory_and_traversal_limits_narrow_configured_scope() {
        let source = TestRoot::new("request-limits-source");
        let cache = TestRoot::new("request-limits-cache");
        fs::create_dir_all(source.0.join("one/two")).expect("nested directories");
        fs::write(source.0.join("one/two/deep.txt"), b"deep-marker").expect("deep file");
        fs::write(source.0.join("large.txt"), vec![b'x'; 1_100_000]).expect("large file");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(100, 4_194_304, 2_097_152, 16).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(20, "open"), &source.0).expect("open");
        let shallow = ResourceBudget::conservative(8192, 20, 100, 128, 100, 1, 30_000, 1_048_576)
            .expect("shallow budget");
        let status = engine
            .build_snapshot(&request(21, "limited_snapshot"), shallow)
            .expect("partial snapshot");
        assert_eq!(status.completeness, "partial");
        assert!(
            status
                .skipped
                .iter()
                .any(|item| item.reason == "limit_reached")
        );
        assert!(status.skipped.iter().any(|item| item.reason == "oversized"));

        let broad = ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 4_194_304)
            .expect("broad budget");
        engine
            .build_snapshot(&request(22, "broad_snapshot"), broad)
            .expect("broad snapshot");
        let memory_limited =
            ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 1_048_576)
                .expect("memory budget");
        let result = engine
            .search(
                &request(23, "memory_search"),
                QueryKind::Filename,
                "large.txt",
                &memory_limited,
            )
            .expect("limited search");
        assert!(result.truncated);
        assert_eq!(result.completeness, "partial");
        assert_eq!(result.truncation_reasons, vec!["memory_limit"]);
        assert!(result.matches.is_empty());

        let output_limited =
            ResourceBudget::conservative(1024, 20, 100, 1024, 100, 16, 30_000, 4_194_304)
                .expect("output budget");
        let result = engine
            .search(
                &request(24, "output_search"),
                QueryKind::Filename,
                "large.txt",
                &output_limited,
            )
            .expect("output-limited search");
        assert!(serde_json::to_vec(&result).expect("response JSON").len() <= 1024);
        assert!(result.truncated);
        assert!(result.truncation_reasons.contains(&"output_budget".into()));
    }
}
