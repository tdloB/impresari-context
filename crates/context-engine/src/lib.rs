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

pub mod cache_prefix;
pub mod file_nomination;
pub mod identifier_index;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{
    AuditOutcome, Capability, ContextPacket, ErrorEnvelope, EvidenceRecord, PacketDraft,
    PacketValidationResult, PolicyDecision, PolicyOutcome, PolicySubject, PublicErrorCode,
    RecoveryAction, ResourceBudget, audit_event, build_packet_with_evidence_order, decide,
    error_envelope, packet_bytes, packet_validation_result, validate_packet,
};
use context_dashboard::{
    DashboardErrorCode, EffectiveBudgetOutcome, PolicyStore, dashboard_purpose, evaluate_budget,
};
use context_retrieval::{
    RetrievalErrorCode, SearchBudget, build_lexical_generation_bounded, evidence_for_span,
    evidence_record, expand_evidence_record, lookup_exact_path, search_filename, search_lexical,
    search_literal,
};
use context_store::{
    AuditRetention, AuditStore, CacheErrorCode, CachedGraph, CachedStructuralFile, WorkspaceCache,
};
use context_structural::{
    FactClass, GRAPH_VERSION, GraphFileInput, GraphNode, PROTOCOL_VERSION, RESOLVER_VERSION,
    RepositoryMap, StructuralError, StructuralGraph, StructuralLanguage, StructuralQueryResult,
    WorkerLauncher, WorkerPath, WorkerRequest, WorkerSuccess, build_graph_with_unknowns,
    query_graph, repository_map, validate_graph, validate_worker_success, worker_cache_identity,
};
use context_workspace::{
    AuthorizedWorkspace, DiscoveryPolicy, PathIdentity, SkipReason, WorkspaceErrorCode,
    WorkspaceSnapshot,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

const CONTRACT_VERSION: &str = "1.0.0";
const ENGINE_VERSION: &str = "0.2.0";

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

/// Source-free process-lifecycle repository read attestation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RepositoryReadTelemetry {
    /// Closed schema discriminator consumed by the independent evaluator.
    pub schema_name: String,
    /// Closed schema version.
    pub schema_version: String,
    /// Contract-form SHA-256 over sorted portable paths and exact source bytes.
    pub source_fingerprint_sha256: String,
    /// File reads measured at the capability-relative byte boundary.
    pub repository_file_reads: u64,
    /// Reads of paths already observed during this process lifecycle.
    pub repeated_repository_file_reads: u64,
    /// Exact source bytes materialized by those reads.
    pub source_bytes_read: u64,
    /// True only when counters and source identity are exhaustive.
    pub complete: bool,
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
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
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

/// One deterministic retrieval step in a task-specific context plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPlanStep {
    /// Bounded retrieval strategy.
    pub kind: QueryKind,
    /// Strategy input. Repository text remains untrusted data.
    pub query: String,
}

/// Adapter-neutral multi-strategy context plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPlan {
    /// Ordered retrieval steps. V1 accepts between one and eight.
    pub steps: Vec<ContextPlanStep>,
}

/// Declared deterministic context-planning profile.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskProfile {
    /// Establish repository orientation from bounded filename and lexical evidence.
    Orientation,
    /// Gather implementation-relevant exact textual evidence.
    Implementation,
    /// Gather bounded literal and lexical evidence for an observed defect.
    BugInvestigation,
    /// Gather bounded review evidence; change-set semantics remain explicit when unavailable.
    ChangeReview,
    /// Gather bounded security-review evidence without claiming reachability.
    SecurityReview,
    /// Gather bounded test-selection evidence without claiming test association.
    TestSelection,
    /// Gather bounded configuration-change evidence without runtime inference.
    ConfigurationChange,
}

/// A named evidence class accounted for by the deterministic planner.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannerEvidenceClass {
    /// Exact relative-path lookup.
    ExactPath,
    /// Filename or display-path retrieval.
    Filename,
    /// Exact source-byte retrieval.
    Literal,
    /// Lexical candidates followed by source verification.
    Lexical,
    /// Structural relationship traversal.
    StructuralRelationship,
    /// Change-set-derived evidence.
    ChangeSet,
    /// Associated-test evidence.
    AssociatedTest,
    /// Exact configuration-to-code relationship evidence.
    ConfigurationToCodeReference,
    /// Caller-declared convention and exemplar assertion with exact current evidence.
    ConventionExemplar,
    /// Bounded repository directory and manifest map.
    RepositoryOrientation,
}

/// One selected deterministic retrieval step and its stable selection reason.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannedContextStep {
    /// Bounded retrieval operation.
    pub step: ContextPlanStep,
    /// Stable profile rule that selected this operation.
    pub reason_code: String,
}

/// Availability of one planner evidence class.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerCoverage {
    /// Evidence class being reported.
    pub evidence_class: PlannerEvidenceClass,
    /// `available` or `unavailable` for this planner implementation.
    pub status: String,
    /// Stable reason for availability or omission.
    pub reason_code: String,
}

/// A candidate deliberately omitted from a deterministic plan or packet.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerOmission {
    /// Stable candidate category; never source content.
    pub candidate: String,
    /// Stable omission reason.
    pub reason_code: String,
    /// Number of omitted candidates when a bounded packet reports a count.
    pub count: String,
}

/// Auditable deterministic retrieval plan bound to a snapshot and policy decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeterministicContextPlan {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Exact plan identity.
    pub plan_id: String,
    /// Declared profile used to select the plan.
    pub task_profile: TaskProfile,
    /// Snapshot to which this plan is bound.
    pub workspace_snapshot: String,
    /// Policy decision used by the eventual packet build.
    pub policy_decision: String,
    /// Selected retrieval operations in execution order.
    pub steps: Vec<PlannedContextStep>,
    /// Complete inventory of currently available and unavailable evidence classes.
    pub coverage: Vec<PlannerCoverage>,
    /// Profile candidates omitted before retrieval.
    pub omitted_candidates: Vec<PlannerOmission>,
    /// Exact structural traversal used by this plan, when the caller selected
    /// the separately admitted structural-impact adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structural_query: Option<StructuralPlannerQuery>,
    /// Current-snapshot-verified caller declaration used by this plan, when
    /// the separately admitted declared-change-set adapter is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_change_set: Option<VerifiedDeclaredChangeSet>,
    /// Current-snapshot-verified caller-declared source-to-test associations
    /// used by this plan, when the separately admitted adapter is active.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_associated_tests: Option<VerifiedDeclaredAssociatedTests>,
    /// Current-snapshot-verified caller convention/exemplar assertion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_convention_exemplars: Option<VerifiedDeclaredConventionExemplars>,
    /// Current-snapshot repository map used by the admitted orientation adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_orientation: Option<RepositoryOrientationMap>,
}

/// Snapshot-bound structural traversal that contributed exact planner evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralPlannerQuery {
    /// Content-derived identity of the complete declared traversal result.
    pub query_id: String,
    /// Requested graph relationship kinds; an empty list means every supported kind.
    pub edge_kinds: Vec<String>,
    /// Canonical graph traversal result that produced the structural evidence.
    pub result: StructuralQueryResult,
}

/// Explicit bounded structural traversal input for the impact-planner adapter.
#[derive(Clone, Debug)]
pub struct StructuralImpactRequest {
    /// Already validated canonical graph supplied by the caller.
    pub graph: StructuralGraph,
    /// Exact graph node from which bounded traversal begins.
    pub start_node: String,
    /// Relationship kinds requested by the caller; empty permits every kind.
    pub edge_kinds: Vec<String>,
}

/// Validated graph input whose start node is selected by the product.
#[derive(Clone, Debug)]
pub struct StructuralSeedRequest {
    /// Already validated canonical graph supplied by the caller.
    pub graph: StructuralGraph,
    /// Relationship kinds requested by the caller; empty permits every kind.
    pub edge_kinds: Vec<String>,
}

/// Narrowing-only exact-source bounds for one deferred structural expansion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StructuralEvidenceExpansion {
    /// Maximum bytes admitted before the exact structural span.
    pub before_bytes: u64,
    /// Maximum bytes admitted after the exact structural span.
    pub after_bytes: u64,
    /// Hard maximum bytes returned by the expansion.
    pub max_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructuralPlanAnnotation<'a> {
    Available(&'a str),
    Omitted(&'a str),
}

/// Explicit bounded repository-map input for the orientation adapter.
#[derive(Clone, Debug)]
pub struct RepositoryOrientationRequest {
    /// Already validated canonical graph supplied by the caller.
    pub graph: StructuralGraph,
    /// Maximum combined directory and package entries.
    pub max_entries: u32,
}

/// Snapshot-bound repository-map projection that contributed orientation metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryOrientationMap {
    /// Content-derived identity of this canonical map.
    pub map_id: String,
    /// Canonical bounded repository map.
    pub result: RepositoryMap,
}

/// One caller-supplied current parser result replacing an artifact in a prior graph.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncrementalStructuralReplacement {
    /// Exact lossless path identity of the current artifact.
    pub path: WorkerPath,
    /// Expected current source hash.
    pub content_hash: String,
    /// Complete untrusted parser result, revalidated by the engine.
    pub response: WorkerSuccess,
}

/// Explicit one-shot structural replacement manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IncrementalStructuralUpdate {
    /// Exact worker executable identity that owns cached parser results.
    pub worker_sha256: String,
    /// Canonical graph from which this update proceeds.
    pub prior_graph: StructuralGraph,
    /// Current parser results replacing changed artifacts.
    pub replacements: Vec<IncrementalStructuralReplacement>,
    /// Paths asserted removed from the current snapshot.
    pub removed_paths: Vec<WorkerPath>,
}

/// Lossless native path identity supplied in a declared change-set manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredChangePath {
    /// Native platform family for the encoded relative units.
    pub platform_family: String,
    /// Native unit encoding for the encoded relative units.
    pub unit_encoding: String,
    /// Canonical unpadded base64url native relative path units.
    pub relative_units_base64url: String,
}

/// One caller-declared current artifact expected to participate in review.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredChangeEntry {
    /// Lossless relative artifact identity.
    pub path: DeclaredChangePath,
    /// Expected SHA-256 hash of that artifact in the declared snapshot.
    pub content_hash: String,
}

/// Untrusted caller declaration to be verified against the current snapshot.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredChangeSet {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Snapshot the caller says the declared entries belong to.
    pub workspace_snapshot: String,
    /// Optional caller assertion about a base revision; never a computed diff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asserted_base_revision: Option<String>,
    /// Current artifact declarations to verify.
    pub entries: Vec<DeclaredChangeEntry>,
}

/// Canonical, current-snapshot-verified change declaration bound into a plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDeclaredChangeSet {
    /// Content-derived declaration identity.
    pub declaration_id: String,
    /// Exact verified workspace snapshot.
    pub workspace_snapshot: String,
    /// Caller assertion retained distinctly from observed source evidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asserted_base_revision: Option<String>,
    /// Whether the optional assertion matches bounded repository metadata.
    pub base_revision_status: String,
    /// Canonically ordered current-hash-verified entries.
    pub entries: Vec<DeclaredChangeEntry>,
}

/// One caller-declared association between a current source artifact and a
/// current test artifact. The association itself is untrusted caller input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredAssociatedTest {
    /// Caller-selected current source artifact.
    pub source: DeclaredChangeEntry,
    /// Caller-selected current test artifact.
    pub test: DeclaredChangeEntry,
}

/// Untrusted caller declaration of current source-to-test associations.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredAssociatedTests {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Snapshot the caller says every endpoint belongs to.
    pub workspace_snapshot: String,
    /// Source-to-test assertions to verify against that snapshot.
    pub associations: Vec<DeclaredAssociatedTest>,
}

/// Canonical current-snapshot-verified source-to-test association assertion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDeclaredAssociatedTests {
    /// Content-derived identity of the verified association set.
    pub association_id: String,
    /// Exact verified workspace snapshot.
    pub workspace_snapshot: String,
    /// Canonically ordered source-to-test assertions with verified current hashes.
    pub associations: Vec<DeclaredAssociatedTest>,
}

/// One caller assertion associating an opaque label with an exact current artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredConventionExemplar {
    /// Opaque bounded caller label; not observed evidence.
    pub label: String,
    /// Current artifact declaration to verify.
    pub artifact: DeclaredChangeEntry,
}

/// Untrusted caller declaration of convention/exemplar examples.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredConventionExemplars {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Snapshot the caller says every exemplar belongs to.
    pub workspace_snapshot: String,
    /// Opaque labels and current artifact assertions to verify.
    pub exemplars: Vec<DeclaredConventionExemplar>,
}

/// Canonical current-snapshot-verified convention/exemplar assertion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDeclaredConventionExemplars {
    /// Content-derived identity of this verified declaration.
    pub declaration_id: String,
    /// Exact verified workspace snapshot.
    pub workspace_snapshot: String,
    /// Canonically ordered opaque labels and verified current artifact assertions.
    pub exemplars: Vec<DeclaredConventionExemplar>,
}

/// One deterministic plan together with its exact, bounded context packet.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfiledContextPacket {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Retrieval plan used to build the packet.
    pub plan: DeterministicContextPlan,
    /// Immutable packet identity and evidence selected under the supplied budget.
    pub packet: ContextPacket,
    /// Additional omissions caused while packaging bounded evidence.
    pub omitted_candidates: Vec<PlannerOmission>,
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
    budget_policy_root: Option<PathBuf>,
    /// Snapshot-bound identifier index used to nominate candidate files.
    ///
    /// Present only after preparation builds it. Absent means nomination is
    /// unavailable, not that it silently falls back to scanning: searching per
    /// identifier costs thousands of repository reads and exhausts the request.
    identifier_index: Option<crate::identifier_index::TaskIdentifierIndex>,
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
        Self::open_internal(config, context, root, None)
    }

    /// Projects cumulative product read telemetry without adding authority.
    #[must_use]
    pub fn repository_read_telemetry(&self) -> RepositoryReadTelemetry {
        let counters = self.workspace.repository_read_counters();
        let (source_fingerprint_sha256, source_complete) = self.snapshot.as_ref().map_or_else(
            || (contract_sha256(&[]), false),
            |snapshot| {
                (
                    snapshot.source_fingerprint_sha256.clone(),
                    snapshot.complete
                        && snapshot.source_fingerprint_compatible
                        && snapshot.skipped.is_empty(),
                )
            },
        );
        RepositoryReadTelemetry {
            schema_name: "impresari_context_repository_read_telemetry".into(),
            schema_version: "1.0".into(),
            source_fingerprint_sha256,
            repository_file_reads: counters.repository_file_reads,
            repeated_repository_file_reads: counters.repeated_repository_file_reads,
            source_bytes_read: counters.source_bytes_read,
            complete: counters.complete && source_complete,
        }
    }

    /// Opens an explicit workspace with an exact-owned local budget policy store.
    ///
    /// The policy is reloaded and revalidated at every capability admission so
    /// an applied update is immediately effective for a long-lived engine.
    ///
    /// # Errors
    ///
    /// Returns a safe structured failure for policy-store, policy, workspace,
    /// cache, or audit errors.
    pub fn open_with_budget_policy_store(
        config: EngineConfig,
        context: &RequestContext,
        root: &Path,
        budget_policy_root: &Path,
    ) -> Result<(Self, WorkspaceHandle), EngineError> {
        Self::open_internal(
            config,
            context,
            root,
            Some(budget_policy_root.to_path_buf()),
        )
    }

    fn open_internal(
        config: EngineConfig,
        context: &RequestContext,
        root: &Path,
        budget_policy_root: Option<PathBuf>,
    ) -> Result<(Self, WorkspaceHandle), EngineError> {
        let started = Instant::now();
        if let Some(policy_root) = budget_policy_root.as_deref() {
            validate_policy_store_separation(policy_root, root, &config.cache_root)
                .map_err(|code| dashboard_error(context, Capability::WorkspaceOpen, code, None))?;
        }
        let mut audit = AuditStore::open(&config.cache_root)
            .map_err(|error| cache_error(context, Capability::WorkspaceOpen, error.code(), None))?;
        let decision = authorize(
            context,
            None,
            Capability::WorkspaceOpen,
            None,
            budget_policy_root.as_deref(),
        )?;
        if decision.outcome == PolicyOutcome::Deny {
            record_event(
                &mut audit,
                &config.audit_retention,
                context,
                &decision,
                Capability::WorkspaceOpen,
                AuditOutcome::Denied,
                None,
                None,
                elapsed_ms(started),
            )?;
            return Err(policy_denied(context, Capability::WorkspaceOpen, None));
        }
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
            identifier_index: None,
            config,
            workspace,
            snapshot: None,
            cache: None,
            audit,
            handle: handle.clone(),
            budget_policy_root,
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
        let decision = self.authorize(context, Capability::SnapshotBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::SnapshotBuild, &decision, self.ids())?;
        let (discovery, max_elapsed) = bounded_discovery(self.config.discovery, &budget)
            .map_err(|code| core_error(context, Capability::SnapshotBuild, code, self.ids()))?;
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
    /// Build the snapshot-bound identifier index used to nominate files.
    ///
    /// Reads each admitted file once, during preparation. Nomination is a
    /// planning step, so its cost belongs here rather than in a request's
    /// context read budget — searching per identifier instead cost roughly
    /// 3,900 repository reads each and exhausted every request.
    ///
    /// # Errors
    /// Returns a closed engine failure when no snapshot is prepared.
    pub fn build_identifier_index(&mut self, context: &RequestContext) -> Result<(), EngineError> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "workspace snapshot is unavailable",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?
            .clone();
        let mut builder =
            crate::identifier_index::TaskIdentifierIndexBuilder::new(&snapshot.snapshot_id);
        for artifact in snapshot
            .artifacts
            .iter()
            .take(crate::identifier_index::MAX_INDEXED_FILES)
        {
            // An unreadable file contributes nothing and must not fail
            // preparation for every other file.
            if let Ok(exact) = self
                .workspace
                .read_exact(&artifact.path, artifact.size_bytes)
            {
                builder.admit(&artifact.path.display_path, &exact.bytes);
            }
        }
        self.identifier_index = Some(builder.finish());
        Ok(())
    }

    /// Whether a usable identifier index is prepared for the current snapshot.
    #[must_use]
    pub fn has_identifier_index(&self) -> bool {
        self.snapshot.as_ref().is_some_and(|snapshot| {
            self.identifier_index
                .as_ref()
                .is_some_and(|index| index.workspace_snapshot == snapshot.snapshot_id)
        })
    }

    /// Nominate candidate files from the task, then build a dense structural
    /// graph over only those files.
    ///
    /// The whole seed-scoped pipeline in one call: admitted task signals, an
    /// index lookup that reads nothing, ranked nomination, and a scoped build.
    /// It exists as one method so `task_signals` stays private and no caller can
    /// supply its own nomination — a caller able to choose the files could steer
    /// selection, and steering is oracle authority.
    ///
    /// The returned graph is **partial by construction**. The nomination beside
    /// it carries the scope disclosure a consumer needs to read it safely.
    ///
    /// # Errors
    /// Returns a closed engine failure for an invalid task, or for
    /// authorization, snapshot, worker, resource, or store failures.
    pub fn build_task_scoped_structure(
        &mut self,
        context: &RequestContext,
        query: &str,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
    ) -> Result<(StructuralGraph, crate::file_nomination::FileNomination), EngineError> {
        use std::collections::BTreeSet;

        if !valid_task_query(query) {
            return Err(core_error(
                context,
                Capability::StructureBuild,
                context_core::CoreErrorCode::InvalidInput,
                self.ids(),
            ));
        }
        let signals = task_signals(query);
        let snapshot_id = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.snapshot_id.clone())
            .unwrap_or_default();
        let tracked: BTreeSet<String> = self
            .snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .artifacts
                    .iter()
                    .map(|artifact| artifact.path.display_path.clone())
                    .collect()
            })
            .unwrap_or_default();

        // Nomination reads nothing. The index answers from memory.
        let identifier_matches = self
            .identifier_index
            .as_ref()
            .and_then(|index| {
                index
                    .identifier_matches(&snapshot_id, &signals.identifiers)
                    .ok()
            })
            .unwrap_or_default();

        let nomination = crate::file_nomination::nominate_files(
            &signals.paths,
            &signals.identifiers,
            &tracked,
            &identifier_matches,
        );
        let scope: BTreeSet<String> = nomination
            .files
            .iter()
            .map(|file| file.display_path.clone())
            .collect();
        // A distinct audit identity: the caller's own context is still to be
        // used for the context build that follows, and the store rejects a
        // duplicate event identity.
        let build_context = derived_task_scope_context(context);
        let graph = self.build_structure_for_paths(&build_context, budget, launcher, &scope)?;
        Ok((graph, nomination))
    }

    /// Build a structural graph over an explicit, bounded set of files.
    ///
    /// A whole-repository graph divides one fact allowance across every file,
    /// which on a large repository leaves roughly one fact each — too thin to
    /// hold a module's declarations. Scoping to the files a task actually
    /// nominated gives each of them a large share of the same allowance, so the
    /// graph is dense where it matters and small enough to store.
    ///
    /// The resulting graph is **partial by construction**. A caller must treat
    /// it as covering only `paths`.
    ///
    /// # Errors
    /// Returns the same failures as a whole-repository build.
    pub fn build_structure_for_paths(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
        paths: &std::collections::BTreeSet<String>,
    ) -> Result<StructuralGraph, EngineError> {
        self.build_structure_scoped(context, budget, launcher, Some(paths))
    }

    /// Build a structural graph over every eligible file in the snapshot.
    ///
    /// The result is thin but complete. Prefer
    /// [`Self::build_structure_for_paths`] when a task has nominated files, and
    /// see [ADR-0128] for why density beats coverage on a large repository.
    ///
    /// [ADR-0128]: https://github.com/tdloB/impresari-context/blob/main/docs/decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md
    ///
    /// # Errors
    /// Returns a closed engine failure for authorization, snapshot, worker,
    /// resource, or store failures.
    pub fn build_structure(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
    ) -> Result<StructuralGraph, EngineError> {
        self.build_structure_scoped(context, budget, launcher, None)
    }

    fn build_structure_scoped(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
        scope: Option<&std::collections::BTreeSet<String>>,
    ) -> Result<StructuralGraph, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::StructureBuild, Some(budget.clone()))?;
        let budget = admitted_budget(context, Capability::StructureBuild, &decision, self.ids())?;
        if self.cache.is_none() {
            self.cache = Some(
                WorkspaceCache::open(&self.config.cache_root, self.workspace.identity()).map_err(
                    |error| {
                        cache_error(
                            context,
                            Capability::StructureBuild,
                            error.code(),
                            Some(self.ids()),
                        )
                    },
                )?,
            );
        }
        let result = self
            .build_structure_internal(context, &budget, launcher, started, scope)
            .and_then(|graph| {
                let payload = serde_json::to_vec(&graph).map_err(|_| {
                    failure(
                        context,
                        Capability::StructureBuild,
                        PublicErrorCode::InternalFailure,
                        "structural graph serialization failed",
                        Some(self.workspace.identity()),
                        self.snapshot
                            .as_ref()
                            .map(|value| value.snapshot_id.as_str()),
                        None,
                    )
                })?;
                self.cache
                    .as_mut()
                    .ok_or_else(|| {
                        failure(
                            context,
                            Capability::StructureBuild,
                            PublicErrorCode::InternalFailure,
                            "structural cache initialization failed",
                            Some(self.workspace.identity()),
                            self.snapshot
                                .as_ref()
                                .map(|value| value.snapshot_id.as_str()),
                            None,
                        )
                    })?
                    .promote_graph(&CachedGraph {
                        graph_id: graph.graph_id.clone(),
                        snapshot_id: graph.workspace_snapshot.clone(),
                        payload,
                    })
                    .map_err(|error| {
                        cache_error(
                            context,
                            Capability::StructureBuild,
                            error.code(),
                            Some(self.ids()),
                        )
                    })?;
                Ok(graph)
            });
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

    /// Applies one explicit current-snapshot structural replacement manifest.
    ///
    /// # Errors
    ///
    /// Fails closed for a stale or malformed prior graph, malformed replacement,
    /// changed source, missing exact cache entry, removal mismatch, or resource limit.
    pub fn apply_incremental_structural_update(
        &mut self,
        context: &RequestContext,
        update: &IncrementalStructuralUpdate,
        budget: &ResourceBudget,
    ) -> Result<StructuralGraph, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::StructureBuild, Some(budget.clone()))?;
        let budget = admitted_budget(context, Capability::StructureBuild, &decision, self.ids())?;
        let result =
            self.apply_incremental_structural_update_internal(context, update, &budget, started);
        let outcome = result.as_ref().map_or(AuditOutcome::Failed, |graph| {
            if graph.completeness == "partial" {
                AuditOutcome::Limited
            } else {
                AuditOutcome::Allowed
            }
        });
        self.finalize(
            context,
            &decision,
            Capability::StructureBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    #[allow(clippy::too_many_lines)] // The complete fail-closed manifest validation is intentionally co-located.
    fn apply_incremental_structural_update_internal(
        &mut self,
        context: &RequestContext,
        update: &IncrementalStructuralUpdate,
        budget: &ResourceBudget,
        started: Instant,
    ) -> Result<StructuralGraph, EngineError> {
        if !valid_sha256(&update.worker_sha256) {
            return Err(failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::InvalidInput,
                "invalid incremental worker identity",
                Some(self.workspace.identity()),
                self.snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.snapshot_id.as_str()),
                Some(RecoveryAction::ReduceScope),
            ));
        }
        validate_graph(&update.prior_graph).map_err(|error| {
            structural_failure(
                context,
                error,
                self.workspace.identity(),
                &update.prior_graph.workspace_snapshot,
            )
        })?;
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "workspace snapshot is unavailable",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?
            .clone();
        if update.prior_graph.workspace_snapshot == snapshot.snapshot_id {
            return Err(failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::InvalidInput,
                "incremental update requires a newer workspace snapshot",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RefreshSnapshot),
            ));
        }
        let limits = structural_limits(context, budget, self.ids())?;
        let mut replacements = std::collections::BTreeMap::new();
        for replacement in &update.replacements {
            if replacement.path.relative_units_base64url.is_empty()
                || replacements
                    .insert(
                        replacement.path.relative_units_base64url.as_str(),
                        replacement,
                    )
                    .is_some()
            {
                return Err(failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid incremental replacement manifest",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
        }
        let current_paths = snapshot
            .artifacts
            .iter()
            .map(|artifact| artifact.path.relative_units_base64url.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        for removed in &update.removed_paths {
            if current_paths.contains(removed.relative_units_base64url.as_str()) {
                return Err(failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "declared removed artifact remains in current snapshot",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RefreshSnapshot),
                ));
            }
        }
        if self.cache.is_none() {
            self.cache = Some(
                WorkspaceCache::open(&self.config.cache_root, self.workspace.identity()).map_err(
                    |error| {
                        cache_error(
                            context,
                            Capability::StructureBuild,
                            error.code(),
                            Some(self.ids()),
                        )
                    },
                )?,
            );
        }
        let mut files = Vec::new();
        let mut unknowns = Vec::new();
        for artifact in snapshot
            .artifacts
            .iter()
            .take(usize::try_from(limits.files).unwrap_or(usize::MAX))
        {
            if elapsed_ms(started) >= limits.elapsed_ms {
                return Err(failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::ResourceLimit,
                    "structural elapsed-time limit exceeded",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
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
                    "workspace changed during incremental structural update",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RefreshSnapshot),
                ));
            }
            let request = structural_request(context, language, exact, limits);
            let response = if let Some(replacement) =
                replacements.remove(request.path.relative_units_base64url.as_str())
            {
                if replacement.path != request.path
                    || replacement.content_hash != request.content_hash
                {
                    return Err(failure(
                        context,
                        Capability::StructureBuild,
                        PublicErrorCode::StaleState,
                        "incremental replacement does not match current artifact",
                        Some(self.workspace.identity()),
                        Some(&snapshot.snapshot_id),
                        Some(RecoveryAction::RefreshSnapshot),
                    ));
                }
                validate_worker_success(&replacement.response, &request).map_err(|error| {
                    structural_failure(
                        context,
                        error,
                        self.workspace.identity(),
                        &snapshot.snapshot_id,
                    )
                })?;
                replacement.response.clone()
            } else {
                self.load_cached_structural(
                    context,
                    &snapshot.snapshot_id,
                    &request,
                    &update.worker_sha256,
                )?
            };
            files.push(GraphFileInput {
                path: request.path,
                response,
            });
        }
        if !replacements.is_empty() {
            return Err(failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::StaleState,
                "incremental replacement is absent from current snapshot",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RefreshSnapshot),
            ));
        }
        if u64::try_from(snapshot.artifacts.len()).unwrap_or(u64::MAX) > limits.files {
            unknowns.push("structural_file_limit_reached".into());
        }
        let graph =
            build_graph_with_unknowns(&snapshot.snapshot_id, files, unknowns).map_err(|error| {
                structural_failure(
                    context,
                    error,
                    self.workspace.identity(),
                    &snapshot.snapshot_id,
                )
            })?;
        let payload = serde_json::to_vec(&graph).map_err(|_| {
            failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::InternalFailure,
                "structural graph serialization failed",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                None,
            )
        })?;
        self.cache
            .as_mut()
            .ok_or_else(|| {
                structural_cache_unavailable(
                    context,
                    self.workspace.identity(),
                    &snapshot.snapshot_id,
                )
            })?
            .promote_graph(&CachedGraph {
                graph_id: graph.graph_id.clone(),
                snapshot_id: graph.workspace_snapshot.clone(),
                payload,
            })
            .map_err(|error| {
                cache_error(
                    context,
                    Capability::StructureBuild,
                    error.code(),
                    Some(self.ids()),
                )
            })?;
        Ok(graph)
    }

    fn load_cached_structural(
        &self,
        context: &RequestContext,
        snapshot_id: &str,
        request: &WorkerRequest,
        worker_sha256: &str,
    ) -> Result<WorkerSuccess, EngineError> {
        let toolchain_identity =
            worker_cache_identity(request, worker_sha256).map_err(|error| {
                structural_failure(context, error, self.workspace.identity(), snapshot_id)
            })?;
        let cached = self
            .cache
            .as_ref()
            .ok_or_else(|| {
                structural_cache_unavailable(context, self.workspace.identity(), snapshot_id)
            })?
            .structural_file(
                &request.path.relative_units_base64url,
                &request.content_hash,
                &toolchain_identity,
            )
            .map_err(|error| {
                cache_error(
                    context,
                    Capability::StructureBuild,
                    error.code(),
                    Some(self.ids()),
                )
            })?;
        let response = cached
            .and_then(|entry| cached_worker_success(&entry.payload, request))
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "exact cached structural result is unavailable",
                    Some(self.workspace.identity()),
                    Some(snapshot_id),
                    Some(RecoveryAction::RebuildIndex),
                )
            })?;
        validate_worker_success(&response, request).map_err(|error| {
            structural_failure(context, error, self.workspace.identity(), snapshot_id)
        })?;
        Ok(response)
    }

    /// Loads and fully revalidates the graph cached for the current snapshot.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, cache, decoding, or integrity failure.
    pub fn cached_structure(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
    ) -> Result<Option<StructuralGraph>, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::StructureQuery, Some(budget.clone()))?;
        let result = self.cached_structure_internal(context);
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::StructureQuery,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    fn cached_structure_internal(
        &mut self,
        context: &RequestContext,
    ) -> Result<Option<StructuralGraph>, EngineError> {
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
        if self.cache.is_none() {
            self.cache = Some(
                WorkspaceCache::open(&self.config.cache_root, self.workspace.identity()).map_err(
                    |error| {
                        cache_error(
                            context,
                            Capability::StructureQuery,
                            error.code(),
                            Some(self.ids()),
                        )
                    },
                )?,
            );
        }
        let Some(cached) = self
            .cache
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureQuery,
                    PublicErrorCode::InternalFailure,
                    "structural cache initialization failed",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    None,
                )
            })?
            .graph_for_snapshot(&snapshot.snapshot_id)
            .map_err(|error| {
                cache_error(
                    context,
                    Capability::StructureQuery,
                    error.code(),
                    Some(self.ids()),
                )
            })?
        else {
            return Ok(None);
        };
        let graph: StructuralGraph = serde_json::from_slice(&cached.payload).map_err(|_| {
            failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::IntegrityFailure,
                "cached structural graph is invalid",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RebuildIndex),
            )
        })?;
        if graph.graph_id != cached.graph_id || validate_graph(&graph).is_err() {
            return Err(failure(
                context,
                Capability::StructureQuery,
                PublicErrorCode::IntegrityFailure,
                "cached structural graph failed integrity validation",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RebuildIndex),
            ));
        }
        Ok(Some(graph))
    }

    fn build_structure_internal(
        &mut self,
        context: &RequestContext,
        budget: &ResourceBudget,
        launcher: &WorkerLauncher,
        started: Instant,
        scope: Option<&std::collections::BTreeSet<String>>,
    ) -> Result<StructuralGraph, EngineError> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::StructureBuild,
                    PublicErrorCode::StaleState,
                    "workspace snapshot is unavailable",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?
            .clone();
        let limits = structural_limits(context, budget, self.ids())?;
        let mut files = Vec::new();
        let mut unknowns = Vec::new();
        let mut remaining_facts = limits.facts;
        // Scoping decides density. The same allowance divided across sixteen
        // nominated files gives each of them hundreds of facts; divided across
        // a whole repository it gives each of them about one.
        let in_scope = |artifact: &&context_workspace::ArtifactRecord| {
            artifact_in_scope(&artifact.path.display_path, scope)
        };
        let mut remaining_supported_files =
            supported_file_count(&snapshot.artifacts, limits.files, scope);
        if scope.is_some() {
            // A scoped graph is dense but partial and must never read as a
            // whole-repository one.
            unknowns.push("structural_scope_limited_to_nominated_files".into());
        }
        for artifact in snapshot
            .artifacts
            .iter()
            .take(usize::try_from(limits.files).unwrap_or(usize::MAX))
            .filter(in_scope)
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
            let Some(file_fact_quota) =
                structural_fact_quota(remaining_facts, remaining_supported_files)
            else {
                unknowns.push("structural_fact_limit_reached".into());
                break;
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
            let mut file_limits = limits;
            file_limits.facts = file_fact_quota;
            let request = structural_request(context, language, exact, file_limits);
            let response =
                self.load_or_parse_structural(context, &snapshot.snapshot_id, &request, launcher)?;
            remaining_facts = remaining_facts
                .saturating_sub(u32::try_from(response.facts.len()).unwrap_or(u32::MAX));
            remaining_supported_files = remaining_supported_files.saturating_sub(1);
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

    fn load_or_parse_structural(
        &mut self,
        context: &RequestContext,
        snapshot_id: &str,
        request: &WorkerRequest,
        launcher: &WorkerLauncher,
    ) -> Result<context_structural::WorkerSuccess, EngineError> {
        let toolchain_identity = worker_cache_identity(request, &launcher.expected_sha256)
            .map_err(|error| {
                structural_failure(context, error, self.workspace.identity(), snapshot_id)
            })?;
        let cached = self
            .cache
            .as_ref()
            .ok_or_else(|| {
                structural_cache_unavailable(context, self.workspace.identity(), snapshot_id)
            })?
            .structural_file(
                &request.path.relative_units_base64url,
                &request.content_hash,
                &toolchain_identity,
            )
            .map_err(|error| {
                cache_error(
                    context,
                    Capability::StructureBuild,
                    error.code(),
                    Some(self.ids()),
                )
            })?;
        let response = cached
            .and_then(|record| cached_worker_success(&record.payload, request))
            .map_or_else(|| launcher.execute(request), Ok)
            .map_err(|error| {
                structural_failure(context, error, self.workspace.identity(), snapshot_id)
            })?;
        let payload = serde_json::to_vec(&response).map_err(|_| {
            failure(
                context,
                Capability::StructureBuild,
                PublicErrorCode::InternalFailure,
                "structural result serialization failed",
                Some(self.workspace.identity()),
                Some(snapshot_id),
                None,
            )
        })?;
        self.cache
            .as_mut()
            .ok_or_else(|| {
                structural_cache_unavailable(context, self.workspace.identity(), snapshot_id)
            })?
            .store_structural_file(&CachedStructuralFile {
                path_units: request.path.relative_units_base64url.clone(),
                content_hash: request.content_hash.clone(),
                toolchain_identity,
                payload,
            })
            .map_err(|error| {
                cache_error(
                    context,
                    Capability::StructureBuild,
                    error.code(),
                    Some((Some(self.workspace.identity()), Some(snapshot_id))),
                )
            })?;
        Ok(response)
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
        let budget = admitted_budget(context, Capability::StructureQuery, &decision, self.ids())?;
        let result = self.query_structure_internal(context, graph, start_node, edge_kinds, &budget);
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
        let budget = admitted_budget(context, Capability::CodeSearch, &decision, self.ids())?;
        let result = self.search_internal(context, Capability::CodeSearch, kind, query, &budget);
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
        self.build_planned_context(
            context,
            &ContextPlan {
                steps: vec![ContextPlanStep {
                    kind,
                    query: query.to_owned(),
                }],
            },
            budget,
        )
    }

    /// Builds one bounded packet from a declared deterministic task profile.
    ///
    /// The profile selects only documented retrieval rules. It does not inspect
    /// repository text, call a model, execute code, or add authority. The
    /// returned plan is bound to the exact snapshot and policy decision that
    /// produced the returned packet.
    ///
    /// # Errors
    ///
    /// Returns a structured policy, stale-state, plan, retrieval, packet, or
    /// audit failure.
    pub fn build_profiled_context(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = self.build_profiled_context_internal(
            context,
            profile,
            query,
            budget,
            &decision.decision_id,
            started,
            None,
            None,
            None,
            None,
            None,
            None,
            true,
        );
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds one profiled packet with exact evidence recovered from one
    /// current-snapshot structural traversal.
    ///
    /// The graph query is authorized through the existing `StructureQuery`
    /// gateway before packet construction. This method adds no graph-building,
    /// process, source-write, or semantic-resolution authority.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when the graph is stale or malformed, the
    /// traversal or packet exceeds a declared bound, or source bytes can no
    /// longer be recovered exactly from the current snapshot.
    pub fn build_profiled_structural_context(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        structural_request: &StructuralImpactRequest,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let structure_context = derived_structure_query_context(context, 0);
        let traversal = self.query_structure(
            &structure_context,
            &structural_request.graph,
            &structural_request.start_node,
            &structural_request.edge_kinds,
            &budget,
        )?;
        let structural_query = structural_planner_query(&structural_request.edge_kinds, traversal)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = self.build_profiled_context_internal(
            context,
            profile,
            query,
            budget,
            &decision.decision_id,
            started,
            Some(&structural_query),
            Some(StructuralPlanAnnotation::Available(
                "validated_structural_relationship_available",
            )),
            None,
            None,
            None,
            None,
            true,
        );
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds one profiled packet whose structural start node is selected from
    /// admitted exact task signals by the product.
    ///
    /// Ambiguous or unavailable seeds retain ordinary profiled retrieval and
    /// record an explicit omission. The supplied graph must remain valid and
    /// bound to the current snapshot even when no seed is selected.
    ///
    /// # Errors
    ///
    /// Returns a structured failure for invalid or stale graph state, invalid
    /// task input, bounded traversal failure, retrieval failure, or packet
    /// construction failure.
    pub fn build_profiled_seeded_structural_context(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        structural_request: &StructuralSeedRequest,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        self.build_profiled_seeded_structural_context_internal(
            context,
            profile,
            query,
            structural_request,
            budget,
            true,
        )
    }

    /// Builds a profiled structural plan and ordinary anchor packet while
    /// deferring exact structural-source recovery to a later authorized call.
    ///
    /// # Errors
    ///
    /// Returns the same policy, graph, seed, traversal, retrieval, and packet
    /// failures as [`Self::build_profiled_seeded_structural_context`].
    pub fn build_profiled_seeded_progressive_context(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        structural_request: &StructuralSeedRequest,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        self.build_profiled_seeded_structural_context_internal(
            context,
            profile,
            query,
            structural_request,
            budget,
            false,
        )
    }

    /// Resolve a ranked seed set and traverse from every admitted seed.
    ///
    /// One anchor cannot describe a task whose answer spans a subclass and the
    /// parent it inherits from, so each seed contributes a traversal and the
    /// results are merged deterministically.
    fn seeded_structural_query(
        &mut self,
        context: &RequestContext,
        structural_request: &StructuralSeedRequest,
        query: &str,
        budget: &ResourceBudget,
    ) -> Result<
        (
            Option<StructuralPlannerQuery>,
            StructuralPlanAnnotation<'static>,
        ),
        EngineError,
    > {
        let selection = structural_seed_selection(&structural_request.graph, query)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        let Some(primary) = selection.seeds.first() else {
            let reason_code = selection
                .unknowns
                .first()
                .copied()
                .unwrap_or("structural_seed_unavailable");
            return Ok((None, StructuralPlanAnnotation::Omitted(reason_code)));
        };
        let reason_code = primary.reason_code;
        let seed_budget = narrow_structural_seed_budget(budget)
            .map_err(|code| core_error(context, Capability::StructureQuery, code, self.ids()))?;
        let mut traversals = Vec::with_capacity(selection.seeds.len());
        for (ordinal, seed) in selection.seeds.iter().enumerate() {
            let structure_context = derived_structure_query_context(context, ordinal);
            traversals.push(self.query_structure(
                &structure_context,
                &structural_request.graph,
                &seed.node_id,
                &structural_request.edge_kinds,
                &seed_budget,
            )?);
        }
        let mut traversal = merge_structural_traversals(traversals).ok_or_else(|| {
            core_error(
                context,
                Capability::ContextBuild,
                context_core::CoreErrorCode::IntegrityFailure,
                self.ids(),
            )
        })?;
        traversal.unknowns.extend(
            selection
                .unknowns
                .iter()
                .map(|unknown| (*unknown).to_owned()),
        );
        traversal.unknowns.sort();
        traversal.unknowns.dedup();
        let planner_query = structural_planner_query(&structural_request.edge_kinds, traversal)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        Ok((
            Some(planner_query),
            StructuralPlanAnnotation::Available(reason_code),
        ))
    }

    fn build_profiled_seeded_structural_context_internal(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        structural_request: &StructuralSeedRequest,
        budget: ResourceBudget,
        recover_structural_evidence: bool,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let snapshot_id = self
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.snapshot_id.clone())
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::StaleState,
                    "workspace snapshot required",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?;
        if structural_request.graph.workspace_snapshot != snapshot_id {
            return Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::StaleState,
                "structural graph is stale",
                Some(self.workspace.identity()),
                Some(&snapshot_id),
                Some(RecoveryAction::RebuildIndex),
            ));
        }
        validate_graph(&structural_request.graph).map_err(|error| {
            structural_query_failure(context, error, self.workspace.identity(), &snapshot_id)
        })?;
        let selected = self.seeded_structural_query(context, structural_request, query, &budget)?;
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = self.build_profiled_context_internal(
            context,
            profile,
            query,
            budget,
            &decision.decision_id,
            started,
            selected.0.as_ref(),
            Some(selected.1),
            None,
            None,
            None,
            None,
            recover_structural_evidence,
        );
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds a snapshot-bound orientation packet from a bounded repository map.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when the supplied graph is malformed or stale,
    /// the map exceeds a declared bound, or ordinary packet retrieval fails.
    #[allow(clippy::too_many_lines)]
    pub fn build_profiled_repository_orientation_context(
        &mut self,
        context: &RequestContext,
        query: &str,
        request: &RepositoryOrientationRequest,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let structure_context = derived_structure_query_context(context, 0);
        let started = Instant::now();
        let structure_decision = self.authorize(
            &structure_context,
            Capability::StructureQuery,
            Some(budget.clone()),
        )?;
        let structure_budget = admitted_budget(
            &structure_context,
            Capability::StructureQuery,
            &structure_decision,
            self.ids(),
        )?;
        let maximum_entries = u32::try_from(structure_budget.max_files_u64().map_err(|code| {
            core_error(context, Capability::StructureQuery, code.code(), self.ids())
        })?)
        .unwrap_or(u32::MAX);
        let orientation = (|| {
            let snapshot = self.snapshot.as_ref().ok_or_else(|| {
                failure(
                    &structure_context,
                    Capability::StructureQuery,
                    PublicErrorCode::StaleState,
                    "workspace snapshot is unavailable",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?;
            if request.graph.workspace_snapshot != snapshot.snapshot_id {
                return Err(failure(
                    &structure_context,
                    Capability::StructureQuery,
                    PublicErrorCode::StaleState,
                    "structural graph is stale",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RebuildIndex),
                ));
            }
            let result = repository_map(&request.graph, request.max_entries.min(maximum_entries))
                .map_err(|error| {
                structural_query_failure(
                    &structure_context,
                    error,
                    self.workspace.identity(),
                    &snapshot.snapshot_id,
                )
            })?;
            let mut value = RepositoryOrientationMap {
                map_id: format!("sha256:{}", "0".repeat(64)),
                result,
            };
            value.map_id = repository_orientation_identity(&value).map_err(|code| {
                core_error(
                    &structure_context,
                    Capability::StructureQuery,
                    code,
                    self.ids(),
                )
            })?;
            Ok(value)
        })();
        let structure_outcome = orientation.as_ref().map_or(AuditOutcome::Failed, |value| {
            if value.result.truncated {
                AuditOutcome::Limited
            } else {
                AuditOutcome::Allowed
            }
        });
        let orientation = self.finalize(
            &structure_context,
            &structure_decision,
            Capability::StructureQuery,
            structure_outcome,
            orientation,
            elapsed_ms(started),
        )?;
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = self.build_profiled_context_internal(
            context,
            TaskProfile::Orientation,
            query,
            budget,
            &decision.decision_id,
            Instant::now(),
            None,
            None,
            None,
            None,
            Some(&orientation),
            None,
            true,
        );
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds one change-review packet anchored to a caller-declared set of
    /// current artifacts.
    ///
    /// The declaration is not a Git diff and does not establish history. Every
    /// declared path and hash is verified against the current authorized
    /// snapshot before exact current-source evidence is recovered.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when the declaration is malformed, stale,
    /// exceeds the supplied budget, or does not exactly match the snapshot.
    pub fn build_profiled_declared_change_set_context(
        &mut self,
        context: &RequestContext,
        query: &str,
        declaration: &DeclaredChangeSet,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = (|| {
            let declared_change_set =
                self.verify_declared_change_set(context, declaration, &budget)?;
            self.build_profiled_context_internal(
                context,
                TaskProfile::ChangeReview,
                query,
                budget,
                &decision.decision_id,
                started,
                None,
                None,
                Some(&declared_change_set),
                None,
                None,
                None,
                true,
            )
        })();
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds one test-selection packet from caller-declared, current-snapshot
    /// source-to-test associations. It neither discovers nor executes tests.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when an asserted pair is malformed, stale,
    /// self-associated, duplicate, out of budget, or cannot be read exactly.
    pub fn build_profiled_declared_associated_test_context(
        &mut self,
        context: &RequestContext,
        query: &str,
        declaration: &DeclaredAssociatedTests,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = (|| {
            let associated =
                self.verify_declared_associated_tests(context, declaration, &budget)?;
            self.build_profiled_context_internal(
                context,
                TaskProfile::TestSelection,
                query,
                budget,
                &decision.decision_id,
                started,
                None,
                None,
                None,
                Some(&associated),
                None,
                None,
                true,
            )
        })();
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    /// Builds an implementation packet with caller-declared verified convention exemplars.
    ///
    /// # Errors
    ///
    /// Returns a structured failure when the declaration is malformed, stale,
    /// over budget, or exact current evidence cannot be recovered.
    pub fn build_profiled_declared_convention_exemplar_context(
        &mut self,
        context: &RequestContext,
        query: &str,
        declaration: &DeclaredConventionExemplars,
        budget: ResourceBudget,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = (|| {
            let conventions =
                self.verify_declared_convention_exemplars(context, declaration, &budget)?;
            self.build_profiled_context_internal(
                context,
                TaskProfile::Implementation,
                query,
                budget,
                &decision.decision_id,
                started,
                None,
                None,
                None,
                None,
                None,
                Some(&conventions),
                true,
            )
        })();
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Sequential plan assembly keeps policy, supplemental evidence, and packet ordering auditable together.
    fn build_profiled_context_internal(
        &mut self,
        context: &RequestContext,
        profile: TaskProfile,
        query: &str,
        budget: ResourceBudget,
        policy_decision: &str,
        started: Instant,
        structural_query: Option<&StructuralPlannerQuery>,
        structural_annotation: Option<StructuralPlanAnnotation<'_>>,
        declared_change_set: Option<&VerifiedDeclaredChangeSet>,
        declared_associated_tests: Option<&VerifiedDeclaredAssociatedTests>,
        repository_orientation: Option<&RepositoryOrientationMap>,
        declared_convention_exemplars: Option<&VerifiedDeclaredConventionExemplars>,
        recover_structural_evidence: bool,
    ) -> Result<ProfiledContextPacket, EngineError> {
        let snapshot = self
            .snapshot
            .as_ref()
            .ok_or_else(|| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::StaleState,
                    "workspace snapshot required",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?
            .snapshot_id
            .clone();
        let max_literal_bytes = resource_budget_max_literal_bytes(&budget)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        let mut plan = deterministic_plan(
            profile,
            query,
            &snapshot,
            policy_decision,
            max_literal_bytes,
        )
        .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        apply_structural_annotation_to_plan(&mut plan, structural_query, structural_annotation)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        if let Some(declared_change_set) = declared_change_set {
            apply_declared_change_set_to_plan(&mut plan, declared_change_set)
                .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        }
        if let Some(associated) = declared_associated_tests {
            apply_declared_associated_tests_to_plan(&mut plan, associated)
                .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        }
        if let Some(conventions) = declared_convention_exemplars {
            apply_declared_convention_exemplars_to_plan(&mut plan, conventions)
                .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        }
        if let Some(orientation) = repository_orientation {
            apply_repository_orientation_to_plan(&mut plan, orientation)
                .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        }
        let (structural_evidence, structural_unknowns) = if recover_structural_evidence {
            structural_query.map_or_else(
                || Ok((Vec::new(), Vec::new())),
                |value| {
                    self.structural_evidence(
                        context,
                        Capability::ContextBuild,
                        value,
                        &budget,
                        started,
                    )
                },
            )?
        } else {
            let mut unknowns = structural_query
                .map(|value| value.result.unknowns.clone())
                .unwrap_or_default();
            if structural_query.is_some_and(|value| value.result.truncated) {
                unknowns.push("structural_query_limited".into());
            }
            unknowns.sort();
            unknowns.dedup();
            (Vec::new(), unknowns)
        };
        let (declared_evidence, declared_unknowns) = declared_change_set.map_or_else(
            || Ok((Vec::new(), Vec::new())),
            |value| self.declared_change_set_evidence(context, value, &budget, started),
        )?;
        let (associated_evidence, associated_unknowns) = declared_associated_tests.map_or_else(
            || Ok((Vec::new(), Vec::new())),
            |value| self.declared_associated_test_evidence(context, value, &budget, started),
        )?;
        let (convention_evidence, convention_unknowns) = declared_convention_exemplars
            .map_or_else(
                || Ok((Vec::new(), Vec::new())),
                |value| {
                    self.declared_convention_exemplar_evidence(context, value, &budget, started)
                },
            )?;
        let mut leading_evidence = declared_evidence;
        leading_evidence.extend(associated_evidence);
        leading_evidence.extend(convention_evidence);
        let mut leading_unknowns = declared_unknowns;
        leading_unknowns.extend(associated_unknowns);
        leading_unknowns.extend(convention_unknowns);
        let packet = self.build_planned_context_with_supplemental_internal(
            context,
            &ContextPlan {
                steps: plan.steps.iter().map(|item| item.step.clone()).collect(),
            },
            budget,
            policy_decision,
            started,
            leading_evidence,
            leading_unknowns,
            structural_evidence,
            structural_unknowns,
            Some(&snapshot),
        )?;
        let mut omitted_candidates = Vec::new();
        if packet.accounting.omitted_items != "0" {
            omitted_candidates.push(PlannerOmission {
                candidate: "retrieved_evidence".into(),
                reason_code: "evidence_budget".into(),
                count: packet.accounting.omitted_items.clone(),
            });
        }
        Ok(ProfiledContextPacket {
            schema_name: "profiled-context-packet".into(),
            schema_version: CONTRACT_VERSION.into(),
            plan,
            packet,
            omitted_candidates,
        })
    }

    /// Builds one packet from an ordered, bounded set of deterministic retrieval
    /// strategies. Exact evidence is deduplicated by identity before packaging;
    /// empty and limited steps remain visible as unknowns and truncations.
    ///
    /// # Errors
    ///
    /// Returns a structured gateway, plan, retrieval, packet, or audit failure.
    pub fn build_planned_context(
        &mut self,
        context: &RequestContext,
        plan: &ContextPlan,
        budget: ResourceBudget,
    ) -> Result<ContextPacket, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::ContextBuild, Some(budget))?;
        let budget = admitted_budget(context, Capability::ContextBuild, &decision, self.ids())?;
        let result = self.build_planned_context_internal(
            context,
            plan,
            budget,
            &decision.decision_id,
            started,
        );
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::ContextBuild,
            outcome,
            result,
            elapsed_ms(started),
        )
    }

    fn build_planned_context_internal(
        &mut self,
        context: &RequestContext,
        plan: &ContextPlan,
        budget: ResourceBudget,
        policy_decision: &str,
        started: Instant,
    ) -> Result<ContextPacket, EngineError> {
        self.build_planned_context_with_supplemental_internal(
            context,
            plan,
            budget,
            policy_decision,
            started,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            None,
        )
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Sequential bounded retrieval and accounting remain auditable together.
    fn build_planned_context_with_supplemental_internal(
        &mut self,
        context: &RequestContext,
        plan: &ContextPlan,
        budget: ResourceBudget,
        policy_decision: &str,
        started: Instant,
        leading_evidence: Vec<EvidenceRecord>,
        leading_unknowns: Vec<String>,
        trailing_evidence: Vec<EvidenceRecord>,
        trailing_unknowns: Vec<String>,
        expected_snapshot: Option<&str>,
    ) -> Result<ContextPacket, EngineError> {
        if plan.steps.is_empty() || plan.steps.len() > 8 {
            Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::InvalidInput,
                "invalid context retrieval plan",
                Some(self.workspace.identity()),
                self.snapshot
                    .as_ref()
                    .map(|value| value.snapshot_id.as_str()),
                Some(RecoveryAction::ReduceScope),
            ))
        } else {
            let mut evidence = std::collections::BTreeMap::new();
            let mut evidence_order = Vec::new();
            for item in leading_evidence {
                Self::insert_ranked_evidence(&mut evidence, &mut evidence_order, item);
            }
            let mut unknowns = leading_unknowns;
            let mut snapshot_id = expected_snapshot.map(str::to_owned);
            let plan_elapsed_limit = budget.max_elapsed_ms_u64().map_err(|error| {
                core_error(context, Capability::ContextBuild, error.code(), self.ids())
            })?;
            for (index, step) in plan.steps.iter().enumerate() {
                if elapsed_ms(started) >= plan_elapsed_limit {
                    return Err(failure(
                        context,
                        Capability::ContextBuild,
                        PublicErrorCode::ResourceLimit,
                        "context planning resource limit exceeded",
                        Some(self.workspace.identity()),
                        snapshot_id.as_deref(),
                        Some(RecoveryAction::ReduceScope),
                    ));
                }
                let search = self.search_internal(
                    context,
                    Capability::ContextBuild,
                    step.kind,
                    &step.query,
                    &budget,
                )?;
                if snapshot_id
                    .as_ref()
                    .is_some_and(|expected| expected != &search.snapshot_id)
                {
                    return Err(failure(
                        context,
                        Capability::ContextBuild,
                        PublicErrorCode::StaleState,
                        "workspace changed during context planning",
                        Some(self.workspace.identity()),
                        Some(&search.snapshot_id),
                        Some(RecoveryAction::RefreshSnapshot),
                    ));
                }
                snapshot_id = Some(search.snapshot_id);
                if search.matches.is_empty() {
                    unknowns.push(format!("plan_step_{index}_no_evidence"));
                }
                if search.truncated {
                    unknowns.push(format!("plan_step_{index}_limited"));
                }
                unknowns.extend(search.unknowns);
                for item in search.matches {
                    Self::insert_ranked_evidence(&mut evidence, &mut evidence_order, item);
                }
            }
            for item in trailing_evidence {
                Self::insert_ranked_evidence(&mut evidence, &mut evidence_order, item);
            }
            unknowns.extend(trailing_unknowns);
            let snapshot_id = snapshot_id.ok_or_else(|| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid context retrieval plan",
                    Some(self.workspace.identity()),
                    None,
                    Some(RecoveryAction::ReduceScope),
                )
            })?;
            unknowns.sort();
            unknowns.dedup();
            build_packet_with_evidence_order(
                PacketDraft {
                    workspace_identity: self.workspace.identity().to_owned(),
                    workspace_snapshot: snapshot_id,
                    request_id: context.request_id.clone(),
                    purpose: context.subject.purpose.clone(),
                    created_at: context.occurred_at.clone(),
                    policy_decision: policy_decision.to_owned(),
                    budget,
                    evidence: evidence.into_values().collect(),
                    assumptions: Vec::new(),
                    conflicts: Vec::new(),
                    unknowns,
                    redactions: Vec::new(),
                },
                &evidence_order,
            )
            .map_err(|error| {
                core_error(context, Capability::ContextBuild, error.code(), self.ids())
            })
        }
    }

    fn insert_ranked_evidence(
        evidence: &mut std::collections::BTreeMap<String, EvidenceRecord>,
        evidence_order: &mut Vec<String>,
        item: EvidenceRecord,
    ) {
        if !evidence.contains_key(&item.evidence_id) {
            evidence_order.push(item.evidence_id.clone());
            evidence.insert(item.evidence_id.clone(), item);
        }
    }

    fn structural_evidence(
        &self,
        context: &RequestContext,
        capability: Capability,
        structural_query: &StructuralPlannerQuery,
        budget: &ResourceBudget,
        started: Instant,
    ) -> Result<(Vec<EvidenceRecord>, Vec<String>), EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                capability,
                PublicErrorCode::StaleState,
                "workspace snapshot required",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        let result = &structural_query.result;
        if result.workspace_snapshot != snapshot.snapshot_id {
            return Err(failure(
                context,
                capability,
                PublicErrorCode::StaleState,
                "structural query is stale",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RefreshSnapshot),
            ));
        }
        let paths = result
            .nodes
            .iter()
            .map(|node| (node.node_id.as_str(), &node.path))
            .collect::<std::collections::BTreeMap<_, _>>();
        let source_budget = search_budget(budget)
            .map_err(|code| core_error(context, capability, code, self.ids()))?;
        let elapsed_limit = budget
            .max_elapsed_ms_u64()
            .map_err(|error| core_error(context, capability, error.code(), self.ids()))?;
        let mut evidence = std::collections::BTreeMap::new();
        for edge in &result.edges {
            if elapsed_ms(started) >= elapsed_limit {
                return Err(failure(
                    context,
                    capability,
                    PublicErrorCode::ResourceLimit,
                    "context planning resource limit exceeded",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
            let worker_path = paths.get(edge.source_node.as_str()).ok_or_else(|| {
                failure(
                    context,
                    capability,
                    PublicErrorCode::IntegrityFailure,
                    "structural query has an invalid source node",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RebuildIndex),
                )
            })?;
            let path = PathIdentity::from_encoded_native_units(
                &worker_path.platform_family,
                &worker_path.unit_encoding,
                &worker_path.relative_units_base64url,
            )
            .map_err(|_| {
                failure(
                    context,
                    capability,
                    PublicErrorCode::IntegrityFailure,
                    "structural query has an invalid source path",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RebuildIndex),
                )
            })?;
            let recovered = evidence_for_span(
                &self.workspace,
                snapshot,
                &path,
                edge.span.start_byte,
                edge.span.end_byte,
                source_budget,
                "structural_graph_edge",
            )
            .map_err(|error| retrieval_error(context, capability, error.code(), self.ids()))?;
            let record = evidence_record(&recovered);
            evidence.entry(record.evidence_id.clone()).or_insert(record);
        }
        let mut unknowns = result.unknowns.clone();
        if result.truncated {
            unknowns.push("structural_query_limited".into());
        }
        unknowns.sort();
        unknowns.dedup();
        Ok((evidence.into_values().collect(), unknowns))
    }

    #[allow(clippy::too_many_lines)] // Each fail-closed manifest boundary retains its safe error mapping.
    fn verify_declared_change_set(
        &self,
        context: &RequestContext,
        declaration: &DeclaredChangeSet,
        budget: &ResourceBudget,
    ) -> Result<VerifiedDeclaredChangeSet, EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::StaleState,
                "workspace snapshot required",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        if declaration.schema_name != "declared-change-set"
            || declaration.schema_version != CONTRACT_VERSION
            || declaration.workspace_snapshot != snapshot.snapshot_id
            || !valid_sha256(&declaration.workspace_snapshot)
            || declaration.entries.is_empty()
            || declaration.entries.len() > 10_000
            || u64::try_from(declaration.entries.len()).unwrap_or(u64::MAX)
                > budget.max_files_u64().map_err(|error| {
                    core_error(context, Capability::ContextBuild, error.code(), self.ids())
                })?
            || declaration
                .asserted_base_revision
                .as_ref()
                .is_some_and(|value| value.is_empty() || value.len() > 256 || value.contains('\0'))
        {
            return Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::InvalidInput,
                "invalid declared change-set",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::ReduceScope),
            ));
        }
        let mut entries = std::collections::BTreeMap::new();
        for entry in &declaration.entries {
            if !valid_sha256(&entry.content_hash) {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared change-set",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
            let path = PathIdentity::from_encoded_native_units(
                &entry.path.platform_family,
                &entry.path.unit_encoding,
                &entry.path.relative_units_base64url,
            )
            .map_err(|_| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared change-set",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                )
            })?;
            let artifact = snapshot
                .artifacts
                .iter()
                .find(|artifact| {
                    artifact.path == path && artifact.content_hash == entry.content_hash
                })
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::ContextBuild,
                        PublicErrorCode::StaleState,
                        "declared change-set does not match current snapshot",
                        Some(self.workspace.identity()),
                        Some(&snapshot.snapshot_id),
                        Some(RecoveryAction::RefreshSnapshot),
                    )
                })?;
            let canonical_entry = DeclaredChangeEntry {
                path: DeclaredChangePath {
                    platform_family: artifact.path.platform_family.into(),
                    unit_encoding: artifact.path.unit_encoding.into(),
                    relative_units_base64url: artifact.path.relative_units_base64url.clone(),
                },
                content_hash: artifact.content_hash.clone(),
            };
            if entries
                .insert(
                    artifact.path.relative_units_base64url.clone(),
                    canonical_entry,
                )
                .is_some()
            {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared change-set",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
        }
        let metadata = self.workspace.repository_metadata();
        let base_revision_status = match declaration.asserted_base_revision.as_deref() {
            None => "not_asserted",
            Some(value) if metadata.revision.as_deref() == Some(value) => {
                "matched_repository_metadata"
            }
            Some(_) => "unavailable_or_mismatched",
        };
        let mut verified = VerifiedDeclaredChangeSet {
            declaration_id: format!("sha256:{}", "0".repeat(64)),
            workspace_snapshot: snapshot.snapshot_id.clone(),
            asserted_base_revision: declaration.asserted_base_revision.clone(),
            base_revision_status: base_revision_status.into(),
            entries: entries.into_values().collect(),
        };
        verified.declaration_id = declared_change_set_identity(&verified)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        Ok(verified)
    }

    fn verify_declared_convention_exemplars(
        &self,
        context: &RequestContext,
        declaration: &DeclaredConventionExemplars,
        budget: &ResourceBudget,
    ) -> Result<VerifiedDeclaredConventionExemplars, EngineError> {
        if declaration.schema_name != "declared-convention-exemplars"
            || declaration.schema_version != CONTRACT_VERSION
            || declaration.exemplars.is_empty()
            || declaration.exemplars.len() > 10_000
            || declaration.exemplars.iter().any(|item| {
                item.label.is_empty() || item.label.len() > 128 || item.label.contains('\0')
            })
        {
            return Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::InvalidInput,
                "invalid declared convention exemplars",
                Some(self.workspace.identity()),
                self.snapshot
                    .as_ref()
                    .map(|snapshot| snapshot.snapshot_id.as_str()),
                Some(RecoveryAction::ReduceScope),
            ));
        }
        let verified_entries = self.verify_declared_change_set(
            context,
            &DeclaredChangeSet {
                schema_name: "declared-change-set".into(),
                schema_version: CONTRACT_VERSION.into(),
                workspace_snapshot: declaration.workspace_snapshot.clone(),
                asserted_base_revision: None,
                entries: declaration
                    .exemplars
                    .iter()
                    .map(|item| item.artifact.clone())
                    .collect(),
            },
            budget,
        )?;
        let canonical_entries = verified_entries
            .entries
            .into_iter()
            .map(|item| (item.path.relative_units_base64url.clone(), item))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut exemplars = std::collections::BTreeMap::new();
        for item in &declaration.exemplars {
            let key = &item.artifact.path.relative_units_base64url;
            let artifact = canonical_entries
                .get(key)
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::ContextBuild,
                        PublicErrorCode::IntegrityFailure,
                        "declared convention exemplar verification failed",
                        Some(self.workspace.identity()),
                        Some(&declaration.workspace_snapshot),
                        None,
                    )
                })?
                .clone();
            let identity = format!("{}:{}", item.label, key);
            if exemplars
                .insert(
                    identity,
                    DeclaredConventionExemplar {
                        label: item.label.clone(),
                        artifact,
                    },
                )
                .is_some()
            {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared convention exemplars",
                    Some(self.workspace.identity()),
                    Some(&declaration.workspace_snapshot),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
        }
        let mut verified = VerifiedDeclaredConventionExemplars {
            declaration_id: format!("sha256:{}", "0".repeat(64)),
            workspace_snapshot: declaration.workspace_snapshot.clone(),
            exemplars: exemplars.into_values().collect(),
        };
        verified.declaration_id = declared_convention_exemplars_identity(&verified)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        Ok(verified)
    }

    #[allow(clippy::too_many_lines)] // Each manifest boundary retains its explicit fail-closed mapping.
    fn verify_declared_associated_tests(
        &self,
        context: &RequestContext,
        declaration: &DeclaredAssociatedTests,
        budget: &ResourceBudget,
    ) -> Result<VerifiedDeclaredAssociatedTests, EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::StaleState,
                "workspace snapshot required",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        if declaration.schema_name != "declared-associated-tests"
            || declaration.schema_version != CONTRACT_VERSION
            || declaration.workspace_snapshot != snapshot.snapshot_id
            || declaration.associations.is_empty()
            || declaration.associations.len() > 10_000
            || u64::try_from(declaration.associations.len()).unwrap_or(u64::MAX)
                > budget.max_files_u64().map_err(|error| {
                    core_error(context, Capability::ContextBuild, error.code(), self.ids())
                })?
        {
            return Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::InvalidInput,
                "invalid declared associated tests",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::ReduceScope),
            ));
        }
        let canonical = |entry: &DeclaredChangeEntry| -> Result<DeclaredChangeEntry, EngineError> {
            if !valid_sha256(&entry.content_hash) {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared associated tests",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
            let path = PathIdentity::from_encoded_native_units(
                &entry.path.platform_family,
                &entry.path.unit_encoding,
                &entry.path.relative_units_base64url,
            )
            .map_err(|_| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared associated tests",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                )
            })?;
            let artifact = snapshot
                .artifacts
                .iter()
                .find(|artifact| {
                    artifact.path == path && artifact.content_hash == entry.content_hash
                })
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::ContextBuild,
                        PublicErrorCode::StaleState,
                        "declared associated tests do not match current snapshot",
                        Some(self.workspace.identity()),
                        Some(&snapshot.snapshot_id),
                        Some(RecoveryAction::RefreshSnapshot),
                    )
                })?;
            Ok(DeclaredChangeEntry {
                path: DeclaredChangePath {
                    platform_family: artifact.path.platform_family.into(),
                    unit_encoding: artifact.path.unit_encoding.into(),
                    relative_units_base64url: artifact.path.relative_units_base64url.clone(),
                },
                content_hash: artifact.content_hash.clone(),
            })
        };
        let mut associations = std::collections::BTreeMap::new();
        for association in &declaration.associations {
            let source = canonical(&association.source)?;
            let test = canonical(&association.test)?;
            let source_key = &source.path.relative_units_base64url;
            let test_key = &test.path.relative_units_base64url;
            if source_key == test_key {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared associated tests",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
            let mut pair = [source_key.clone(), test_key.clone()];
            pair.sort();
            if associations
                .insert(
                    (pair[0].clone(), pair[1].clone()),
                    DeclaredAssociatedTest { source, test },
                )
                .is_some()
            {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::InvalidInput,
                    "invalid declared associated tests",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
        }
        let mut verified = VerifiedDeclaredAssociatedTests {
            association_id: format!("sha256:{}", "0".repeat(64)),
            workspace_snapshot: snapshot.snapshot_id.clone(),
            associations: associations.into_values().collect(),
        };
        verified.association_id = declared_associated_tests_identity(&verified)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        Ok(verified)
    }

    fn declared_change_set_evidence(
        &self,
        context: &RequestContext,
        declared_change_set: &VerifiedDeclaredChangeSet,
        budget: &ResourceBudget,
        started: Instant,
    ) -> Result<(Vec<EvidenceRecord>, Vec<String>), EngineError> {
        let snapshot = self.snapshot.as_ref().ok_or_else(|| {
            failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::StaleState,
                "workspace snapshot required",
                Some(self.workspace.identity()),
                None,
                Some(RecoveryAction::RefreshSnapshot),
            )
        })?;
        if declared_change_set.workspace_snapshot != snapshot.snapshot_id {
            return Err(failure(
                context,
                Capability::ContextBuild,
                PublicErrorCode::StaleState,
                "declared change-set is stale",
                Some(self.workspace.identity()),
                Some(&snapshot.snapshot_id),
                Some(RecoveryAction::RefreshSnapshot),
            ));
        }
        let source_budget = search_budget(budget)
            .map_err(|code| core_error(context, Capability::ContextBuild, code, self.ids()))?;
        let elapsed_limit = budget.max_elapsed_ms_u64().map_err(|error| {
            core_error(context, Capability::ContextBuild, error.code(), self.ids())
        })?;
        let mut evidence = Vec::with_capacity(declared_change_set.entries.len());
        for entry in &declared_change_set.entries {
            if elapsed_ms(started) >= elapsed_limit {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::ResourceLimit,
                    "context planning resource limit exceeded",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::ReduceScope),
                ));
            }
            let path = PathIdentity::from_encoded_native_units(
                &entry.path.platform_family,
                &entry.path.unit_encoding,
                &entry.path.relative_units_base64url,
            )
            .map_err(|_| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::IntegrityFailure,
                    "verified declared change-set has an invalid path",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RebuildIndex),
                )
            })?;
            let recovered = lookup_exact_path(&self.workspace, snapshot, &path, source_budget)
                .map_err(|error| {
                    retrieval_error(context, Capability::ContextBuild, error.code(), self.ids())
                })?;
            let record = recovered.matches.into_iter().next().ok_or_else(|| {
                failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::StaleState,
                    "declared change-set source is unavailable",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RefreshSnapshot),
                )
            })?;
            if record.content_hash != entry.content_hash {
                return Err(failure(
                    context,
                    Capability::ContextBuild,
                    PublicErrorCode::StaleState,
                    "declared change-set source changed",
                    Some(self.workspace.identity()),
                    Some(&snapshot.snapshot_id),
                    Some(RecoveryAction::RefreshSnapshot),
                ));
            }
            evidence.push(evidence_record(&record));
        }
        let unknowns = if declared_change_set.base_revision_status == "unavailable_or_mismatched" {
            vec!["asserted_base_revision_unavailable_or_mismatched".into()]
        } else {
            Vec::new()
        };
        Ok((evidence, unknowns))
    }

    fn declared_associated_test_evidence(
        &self,
        context: &RequestContext,
        associated: &VerifiedDeclaredAssociatedTests,
        budget: &ResourceBudget,
        started: Instant,
    ) -> Result<(Vec<EvidenceRecord>, Vec<String>), EngineError> {
        let mut entries = std::collections::BTreeMap::new();
        for pair in &associated.associations {
            for entry in [&pair.source, &pair.test] {
                entries
                    .entry(entry.path.relative_units_base64url.clone())
                    .or_insert_with(|| entry.clone());
            }
        }
        let declared = VerifiedDeclaredChangeSet {
            declaration_id: "associated-test-evidence".into(),
            workspace_snapshot: associated.workspace_snapshot.clone(),
            asserted_base_revision: None,
            base_revision_status: "not_asserted".into(),
            entries: entries.into_values().collect(),
        };
        self.declared_change_set_evidence(context, &declared, budget, started)
    }

    fn declared_convention_exemplar_evidence(
        &self,
        context: &RequestContext,
        conventions: &VerifiedDeclaredConventionExemplars,
        budget: &ResourceBudget,
        started: Instant,
    ) -> Result<(Vec<EvidenceRecord>, Vec<String>), EngineError> {
        let mut entries = conventions
            .exemplars
            .iter()
            .map(|item| item.artifact.clone())
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.path
                .relative_units_base64url
                .cmp(&right.path.relative_units_base64url)
        });
        entries.dedup_by(|left, right| {
            left.path.relative_units_base64url == right.path.relative_units_base64url
        });
        let declaration = VerifiedDeclaredChangeSet {
            declaration_id: conventions.declaration_id.clone(),
            workspace_snapshot: conventions.workspace_snapshot.clone(),
            asserted_base_revision: None,
            base_revision_status: "not_asserted".into(),
            entries,
        };
        self.declared_change_set_evidence(context, &declaration, budget, started)
    }

    /// Reauthorizes and recovers one exact structural edge from current source.
    ///
    /// This is the deferred counterpart to eager structural packet evidence.
    /// It accepts only an edge already present in the validated, snapshot-bound
    /// planner traversal and returns the same canonical evidence record.
    ///
    /// # Errors
    ///
    /// Returns a structured failure for an unavailable edge, stale traversal,
    /// changed source, authorization failure, or exhausted budget.
    #[allow(clippy::too_many_lines)] // Authorization, recovery, optional expansion, and audit finalization remain one gateway transaction.
    pub fn expand_structural_edge_evidence(
        &mut self,
        context: &RequestContext,
        structural_query: &StructuralPlannerQuery,
        edge_id: &str,
        expansion: StructuralEvidenceExpansion,
        budget: ResourceBudget,
    ) -> Result<EvidenceRecord, EngineError> {
        let started = Instant::now();
        let decision = self.authorize(context, Capability::EvidenceExpand, Some(budget))?;
        let budget = admitted_budget(context, Capability::EvidenceExpand, &decision, self.ids())?;
        let admitted_max = budget
            .requested
            .parse::<u64>()
            .ok()
            .zip(budget.max_excerpt_bytes_per_item.parse::<u64>().ok())
            .map_or(0, |(requested, excerpt)| requested.min(excerpt));
        let max_bytes = expansion.max_bytes.min(admitted_max);
        let result = (|| {
            let edge = structural_query
                .result
                .edges
                .iter()
                .find(|edge| edge.edge_id == edge_id)
                .cloned()
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::EvidenceExpand,
                        PublicErrorCode::InvalidInput,
                        "structural evidence handle is unavailable",
                        Some(self.workspace.identity()),
                        self.snapshot
                            .as_ref()
                            .map(|value| value.snapshot_id.as_str()),
                        Some(RecoveryAction::ReduceScope),
                    )
                })?;
            let mut selected = structural_query.clone();
            selected.result.edges = vec![edge];
            let recovered = self
                .structural_evidence(
                    context,
                    Capability::EvidenceExpand,
                    &selected,
                    &budget,
                    started,
                )?
                .0
                .into_iter()
                .next()
                .ok_or_else(|| {
                    failure(
                        context,
                        Capability::EvidenceExpand,
                        PublicErrorCode::StaleState,
                        "structural evidence is unavailable",
                        Some(self.workspace.identity()),
                        self.snapshot
                            .as_ref()
                            .map(|value| value.snapshot_id.as_str()),
                        Some(RecoveryAction::RefreshSnapshot),
                    )
                })?;
            if expansion.before_bytes == 0
                && expansion.after_bytes == 0
                && recovered
                    .span
                    .end_byte
                    .parse::<u64>()
                    .ok()
                    .zip(recovered.span.start_byte.parse::<u64>().ok())
                    .is_some_and(|(end, start)| end.saturating_sub(start) <= max_bytes)
            {
                Ok(recovered)
            } else {
                let snapshot = self.snapshot.as_ref().ok_or_else(|| {
                    failure(
                        context,
                        Capability::EvidenceExpand,
                        PublicErrorCode::StaleState,
                        "workspace snapshot is unavailable",
                        Some(self.workspace.identity()),
                        None,
                        Some(RecoveryAction::RefreshSnapshot),
                    )
                })?;
                expand_evidence_record(
                    &self.workspace,
                    snapshot,
                    &recovered,
                    expansion.before_bytes,
                    expansion.after_bytes,
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
            }
        })();
        let outcome = result
            .as_ref()
            .map_or(AuditOutcome::Failed, |_| AuditOutcome::Allowed);
        self.finalize(
            context,
            &decision,
            Capability::EvidenceExpand,
            outcome,
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
        let budget = admitted_budget(context, Capability::EvidenceExpand, &decision, self.ids())?;
        let admitted_max = budget
            .requested
            .parse::<u64>()
            .ok()
            .zip(budget.max_excerpt_bytes_per_item.parse::<u64>().ok())
            .map_or(0, |(requested, excerpt)| requested.min(excerpt));
        let max_bytes = max_bytes.min(admitted_max);
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
        let budget = admitted_budget(context, Capability::ContextValidate, &decision, self.ids())?;
        let evidence_ceiling = budget
            .max_excerpt_bytes_per_item
            .parse::<u64>()
            .unwrap_or(0);
        let snapshot = self.snapshot.as_ref();
        let authorized = packet.workspace_identity == self.workspace.identity();
        let evidence_available = authorized
            && snapshot.is_some_and(|snapshot| {
                packet.observed_evidence.iter().all(|evidence| {
                    expand_evidence_record(
                        &self.workspace,
                        snapshot,
                        evidence,
                        0,
                        0,
                        evidence_ceiling,
                    )
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
        let budget = admitted_budget(context, Capability::HandoffExport, &decision, self.ids())?;
        let result =
            self.export_handoff_inner(context, packet, &budget, export_root, destination_name);
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
                    self.cache = Some(
                        WorkspaceCache::open(&self.config.cache_root, self.workspace.identity())
                            .map_err(|error| {
                                cache_error(context, capability, error.code(), Some(self.ids()))
                            })?,
                    );
                }
                let current_generation = self
                    .cache
                    .as_ref()
                    .expect("cache initialized")
                    .current()
                    .map_err(|error| {
                        cache_error(context, capability, error.code(), Some(self.ids()))
                    })?;
                let generation_is_current = current_generation
                    .as_ref()
                    .is_some_and(|generation| generation.snapshot_id == snapshot.snapshot_id);
                if !generation_is_current {
                    let max_memory = budget.max_memory_bytes_u64().map_err(|error| {
                        core_error(context, capability, error.code(), self.ids())
                    })?;
                    build_lexical_generation_bounded(
                        &self.workspace,
                        snapshot,
                        self.cache.as_mut().expect("cache initialized"),
                        max_memory,
                    )
                    .map_err(|error| {
                        retrieval_error(context, capability, error.code(), self.ids())
                    })?;
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
        &mut self,
        context: &RequestContext,
        capability: Capability,
        budget: Option<ResourceBudget>,
    ) -> Result<PolicyDecision, EngineError> {
        let decision = authorize(
            context,
            Some(self.workspace.identity()),
            capability,
            budget,
            self.budget_policy_root.as_deref(),
        )?;
        if decision.outcome == PolicyOutcome::Deny {
            self.record(context, &decision, capability, AuditOutcome::Denied, 0)?;
            Err(policy_denied(
                context,
                capability,
                Some(self.workspace.identity()),
            ))
        } else {
            Ok(decision)
        }
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
            if decision.outcome == PolicyOutcome::Limit {
                AuditOutcome::Limited
            } else {
                success_outcome
            }
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
    budget_policy_root: Option<&Path>,
) -> Result<PolicyDecision, EngineError> {
    let mut decision = decide(
        &context.request_id,
        &context.subject,
        workspace,
        capability,
        budget.clone(),
        &context.occurred_at,
    )
    .map_err(|error| core_error(context, capability, error.code(), (workspace, None)))?;
    if let Some(root) = budget_policy_root {
        let policy = PolicyStore::open(root)
            .and_then(|store| store.current())
            .map_err(|error| dashboard_error(context, capability, error.code(), workspace))?;
        let engine_maximum = engine_budget_maximum();
        let caller_budget = budget.unwrap_or_else(|| engine_maximum.clone());
        let authorized_budget = decision
            .effective_budget
            .as_ref()
            .unwrap_or(&caller_budget)
            .clone();
        let effective = evaluate_budget(
            &engine_maximum,
            &authorized_budget,
            policy.as_ref(),
            &caller_budget,
            dashboard_purpose(&context.subject.purpose),
            capability,
            &context.occurred_at,
        )
        .map_err(|error| dashboard_error(context, capability, error.code(), workspace))?;
        decision.decision_id = effective.decision_id;
        decision.effective_budget = effective.effective_budget;
        decision.reason_codes.extend(effective.reason_codes);
        decision.policy_profile = format!("{}+local-budget-policy-v1", decision.policy_profile);
        decision.outcome = match effective.outcome {
            EffectiveBudgetOutcome::Allow => PolicyOutcome::Allow,
            EffectiveBudgetOutcome::Limit => PolicyOutcome::Limit,
            EffectiveBudgetOutcome::Deny => PolicyOutcome::Deny,
        };
    }
    Ok(decision)
}

fn policy_denied(
    context: &RequestContext,
    capability: Capability,
    workspace: Option<&str>,
) -> EngineError {
    failure(
        context,
        capability,
        PublicErrorCode::PolicyDenied,
        "capability denied by local policy",
        workspace,
        None,
        Some(RecoveryAction::RequestAuthorization),
    )
}

fn validate_policy_store_separation(
    policy_root: &Path,
    workspace_root: &Path,
    cache_root: &Path,
) -> Result<(), DashboardErrorCode> {
    PolicyStore::open(policy_root).map_err(|error| error.code())?;
    let policy = policy_root
        .canonicalize()
        .map_err(|_| DashboardErrorCode::StorageFailure)?;
    let workspace = workspace_root
        .canonicalize()
        .map_err(|_| DashboardErrorCode::StorageFailure)?;
    let cache = cache_root
        .canonicalize()
        .map_err(|_| DashboardErrorCode::StorageFailure)?;
    if paths_overlap(&policy, &workspace) || paths_overlap(&policy, &cache) {
        return Err(DashboardErrorCode::InvalidInput);
    }
    Ok(())
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
}

fn dashboard_error(
    context: &RequestContext,
    capability: Capability,
    code: DashboardErrorCode,
    workspace: Option<&str>,
) -> EngineError {
    let public = match code {
        DashboardErrorCode::InvalidInput => PublicErrorCode::InvalidInput,
        DashboardErrorCode::ResourceLimit => PublicErrorCode::ResourceLimit,
        DashboardErrorCode::IntegrityFailure => PublicErrorCode::IntegrityFailure,
        DashboardErrorCode::IncompatibleData => PublicErrorCode::IncompatibleCache,
        DashboardErrorCode::StaleState => PublicErrorCode::StaleState,
        DashboardErrorCode::StorageFailure => PublicErrorCode::InternalFailure,
    };
    failure(
        context,
        capability,
        public,
        "local budget policy admission failed",
        workspace,
        None,
        Some(RecoveryAction::None),
    )
}

fn engine_budget_maximum() -> ResourceBudget {
    ResourceBudget::conservative(
        4_194_304,
        10_000,
        1_000_000,
        65_536,
        10_000,
        256,
        300_000,
        2_147_483_648,
    )
    .expect("versioned engine maximum budget")
}

fn admitted_budget(
    context: &RequestContext,
    capability: Capability,
    decision: &PolicyDecision,
    ids: (Option<&str>, Option<&str>),
) -> Result<ResourceBudget, EngineError> {
    decision.effective_budget.clone().ok_or_else(|| {
        failure(
            context,
            capability,
            PublicErrorCode::PolicyDenied,
            "effective budget is unavailable",
            ids.0,
            ids.1,
            Some(RecoveryAction::RequestAuthorization),
        )
    })
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

fn cached_worker_success(payload: &[u8], request: &WorkerRequest) -> Option<WorkerSuccess> {
    let mut response: WorkerSuccess = serde_json::from_slice(payload).ok()?;
    // Request identity is transport correlation, not parser-result provenance.
    // Every cache hit is rebound to the current request before full validation.
    response.request_id.clone_from(&request.request_id);
    validate_worker_success(&response, request).ok()?;
    Some(response)
}

fn resource_budget_max_literal_bytes(
    budget: &ResourceBudget,
) -> Result<u64, context_core::CoreErrorCode> {
    budget
        .max_excerpt_bytes_per_item
        .parse::<u64>()
        .map_err(|_| context_core::CoreErrorCode::InvalidInput)
}

fn narrow_structural_seed_budget(
    budget: &ResourceBudget,
) -> Result<ResourceBudget, context_core::CoreErrorCode> {
    let depth = budget
        .max_traversal_depth
        .parse::<u64>()
        .map_err(|_| context_core::CoreErrorCode::InvalidInput)?;
    let matches = budget
        .max_matches
        .parse::<u64>()
        .map_err(|_| context_core::CoreErrorCode::InvalidInput)?;
    if depth == 0 || matches == 0 {
        return Err(context_core::CoreErrorCode::InvalidInput);
    }
    let mut narrowed = budget.clone();
    narrowed.max_traversal_depth = "1".into();
    narrowed.max_matches = matches.min(16).to_string();
    Ok(narrowed)
}

fn deterministic_plan(
    profile: TaskProfile,
    query: &str,
    snapshot_id: &str,
    policy_decision: &str,
    max_literal_bytes: u64,
) -> Result<DeterministicContextPlan, context_core::CoreErrorCode> {
    if !valid_task_query(query) {
        return Err(context_core::CoreErrorCode::InvalidInput);
    }
    let (base_steps, mut omitted_candidates) = profile_base_plan(profile, query);
    let (steps, original_query_omitted) =
        expand_profile_steps(query, base_steps, max_literal_bytes);
    if original_query_omitted {
        omitted_candidates.push(planner_omission(
            "original_query",
            "original_query_exceeds_retrieval_contract",
        ));
    }
    if steps.is_empty() {
        return Err(context_core::CoreErrorCode::InvalidInput);
    }
    let mut plan = DeterministicContextPlan {
        schema_name: "deterministic-context-plan".into(),
        schema_version: CONTRACT_VERSION.into(),
        plan_id: format!("sha256:{}", "0".repeat(64)),
        task_profile: profile,
        workspace_snapshot: snapshot_id.to_owned(),
        policy_decision: policy_decision.to_owned(),
        steps,
        coverage: planner_coverage(),
        omitted_candidates,
        structural_query: None,
        declared_change_set: None,
        declared_associated_tests: None,
        declared_convention_exemplars: None,
        repository_orientation: None,
    };
    plan.plan_id = deterministic_plan_identity(&plan)?;
    Ok(plan)
}

fn profile_base_plan(
    profile: TaskProfile,
    query: &str,
) -> (Vec<PlannedContextStep>, Vec<PlannerOmission>) {
    let mut omissions = Vec::new();
    let steps = match profile {
        TaskProfile::Orientation => vec![
            planned_step(QueryKind::Filename, query, "profile_orientation_filename"),
            planned_step(QueryKind::Lexical, query, "profile_orientation_lexical"),
        ],
        TaskProfile::Implementation => vec![
            planned_step(QueryKind::Lexical, query, "profile_implementation_lexical"),
            planned_step(QueryKind::Literal, query, "profile_implementation_literal"),
        ],
        TaskProfile::BugInvestigation => vec![
            planned_step(
                QueryKind::Literal,
                query,
                "profile_bug_investigation_literal",
            ),
            planned_step(
                QueryKind::Lexical,
                query,
                "profile_bug_investigation_lexical",
            ),
        ],
        TaskProfile::ChangeReview => {
            omissions.push(planner_omission(
                "change_set",
                "change_set_evidence_unavailable",
            ));
            vec![
                planned_step(QueryKind::Filename, query, "profile_change_review_filename"),
                planned_step(QueryKind::Lexical, query, "profile_change_review_lexical"),
            ]
        }
        TaskProfile::SecurityReview => {
            omissions.push(planner_omission(
                "structural_relationship",
                "structural_relationship_evidence_not_connected",
            ));
            vec![
                planned_step(QueryKind::Literal, query, "profile_security_review_literal"),
                planned_step(QueryKind::Lexical, query, "profile_security_review_lexical"),
            ]
        }
        TaskProfile::TestSelection => {
            omissions.push(planner_omission(
                "associated_test",
                "associated_test_evidence_unavailable",
            ));
            vec![
                planned_step(
                    QueryKind::Filename,
                    query,
                    "profile_test_selection_filename",
                ),
                planned_step(QueryKind::Lexical, query, "profile_test_selection_lexical"),
            ]
        }
        TaskProfile::ConfigurationChange => {
            omissions.push(planner_omission(
                "configuration_to_code_reference",
                "configuration_to_code_reference_unavailable",
            ));
            vec![
                planned_step(
                    QueryKind::Filename,
                    query,
                    "profile_configuration_change_filename",
                ),
                planned_step(
                    QueryKind::Literal,
                    query,
                    "profile_configuration_change_literal",
                ),
            ]
        }
    };
    (steps, omissions)
}

fn planned_step(kind: QueryKind, query: &str, reason_code: &str) -> PlannedContextStep {
    PlannedContextStep {
        step: ContextPlanStep {
            kind,
            query: query.to_owned(),
        },
        reason_code: reason_code.into(),
    }
}

const MAX_TASK_SIGNAL_TOKENS: usize = 16;
/// Raw tokens scanned before classification. Prose, issue-template boilerplate,
/// and markup punctuation routinely precede the first code signal, so the
/// per-list ceiling must be reached by classified signals rather than by the
/// first few words of a task description.
const MAX_TASK_SCANNED_TOKENS: usize = 4_096;
const MAX_TASK_SIGNAL_BYTES: usize = 256;
const MAX_PROFILE_STEPS: usize = 8;

#[derive(Debug, Default, Eq, PartialEq)]
struct TaskSignals {
    quoted: Vec<String>,
    paths: Vec<String>,
    identifiers: Vec<String>,
    lexical: Vec<String>,
}

fn valid_task_query(query: &str) -> bool {
    !query.is_empty()
        && query.len() <= 4_096
        && !query
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}

fn expand_profile_steps(
    query: &str,
    base_steps: Vec<PlannedContextStep>,
    max_literal_bytes: u64,
) -> (Vec<PlannedContextStep>, bool) {
    let mut base = base_steps.into_iter();
    let Some(first) = base.next() else {
        return (Vec::new(), false);
    };
    let fallback = base.next();
    let signals = task_signals(query);
    let original_query_omitted = !full_query_compatible(&first.step, max_literal_bytes);
    let mut steps = if original_query_omitted {
        Vec::new()
    } else {
        vec![first]
    };

    for quoted in &signals.quoted {
        push_unique_planned_step(
            &mut steps,
            QueryKind::Literal,
            quoted,
            "task_signal_quoted_literal",
            max_literal_bytes,
        );
    }
    for path in &signals.paths {
        push_unique_planned_step(
            &mut steps,
            QueryKind::Filename,
            path,
            "task_signal_portable_path",
            max_literal_bytes,
        );
    }
    for identifier in &signals.identifiers {
        push_unique_planned_step(
            &mut steps,
            QueryKind::Literal,
            identifier,
            "task_signal_code_identifier",
            max_literal_bytes,
        );
    }
    for lexical in &signals.lexical {
        push_unique_planned_step(
            &mut steps,
            QueryKind::Lexical,
            lexical,
            "task_signal_lexical_fallback",
            max_literal_bytes,
        );
    }
    if (steps.is_empty() || (steps.len() == 1 && !original_query_omitted))
        && let Some(fallback) = fallback
        && full_query_compatible(&fallback.step, max_literal_bytes)
    {
        push_unique_planned_step(
            &mut steps,
            fallback.step.kind,
            &fallback.step.query,
            &fallback.reason_code,
            max_literal_bytes,
        );
    }
    (steps, original_query_omitted)
}

fn full_query_compatible(step: &ContextPlanStep, max_literal_bytes: u64) -> bool {
    if step.query.len() > MAX_TASK_SIGNAL_BYTES {
        return false;
    }
    if step.kind == QueryKind::Literal
        && u64::try_from(step.query.len()).map_or(true, |length| length > max_literal_bytes)
    {
        return false;
    }
    if step.kind != QueryKind::Lexical {
        return true;
    }
    let terms = lexical_task_terms(&step.query);
    !terms.is_empty() && terms.len() <= 16
}

fn push_unique_planned_step(
    steps: &mut Vec<PlannedContextStep>,
    kind: QueryKind,
    query: &str,
    reason_code: &str,
    max_literal_bytes: u64,
) {
    if steps.len() >= MAX_PROFILE_STEPS
        || query.is_empty()
        || query.len() > MAX_TASK_SIGNAL_BYTES
        || (kind == QueryKind::Literal
            && u64::try_from(query.len()).map_or(true, |length| length > max_literal_bytes))
        || steps
            .iter()
            .any(|existing| existing.step.kind == kind && existing.step.query == query)
    {
        return;
    }
    steps.push(planned_step(kind, query, reason_code));
}

fn task_signals(query: &str) -> TaskSignals {
    let mut signals = TaskSignals::default();
    extract_quoted_signals(query, &mut signals.quoted);

    let tokens = query
        .split(|character: char| {
            !(character.is_ascii_alphanumeric()
                || matches!(character, '_' | '-' | '.' | '/' | ':' | '\\'))
        })
        .filter_map(normalize_signal_token)
        .take(MAX_TASK_SCANNED_TOKENS);
    for token in tokens {
        if signals.paths.len() >= MAX_TASK_SIGNAL_TOKENS
            && signals.identifiers.len() >= MAX_TASK_SIGNAL_TOKENS
        {
            break;
        }
        // A dotted token is usually an attribute or module access, so its final
        // component is the name a graph node actually carries. `ts.remove_column`
        // and `numpy.__version__` never match a symbol; `remove_column` and
        // `__version__` do.
        if let Some((_, member)) = token.rsplit_once('.')
            && is_code_identifier_signal(member)
            && signals.identifiers.len() < MAX_TASK_SIGNAL_TOKENS
        {
            push_unique_text(&mut signals.identifiers, member.to_owned());
        }
        if is_portable_path_signal(&token) {
            if signals.paths.len() < MAX_TASK_SIGNAL_TOKENS {
                push_unique_text(&mut signals.paths, token);
            }
        } else if is_code_identifier_signal(&token)
            && signals.identifiers.len() < MAX_TASK_SIGNAL_TOKENS
        {
            push_unique_text(&mut signals.identifiers, token);
        }
    }

    let unfiltered_lexical = lexical_task_terms(query);
    let mut lexical = unfiltered_lexical
        .iter()
        .filter(|term| term.len() >= 3 && !is_task_stop_word(term))
        .cloned()
        .collect::<Vec<_>>();
    lexical.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
    lexical.dedup();
    if lexical.is_empty()
        && let Some(fallback) = unfiltered_lexical.into_iter().find(|term| term.len() >= 3)
    {
        lexical.push(fallback);
    }
    signals.lexical = lexical;
    signals
}

/// Maximum structural seeds admitted for one task.
///
/// A closed constant, never caller-supplied: a caller able to widen selection
/// could steer it, and steering is oracle authority.
const MAX_STRUCTURAL_SEEDS: usize = 8;

/// Ranked classes of seed candidate, most specific first.
///
/// The order is total, so selection is deterministic for a given snapshot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum SeedRank {
    UniqueSymbolInExactPath,
    UniqueExactFilePath,
    GloballyUniqueSymbol,
    GloballyAmbiguousSymbol,
}

impl SeedRank {
    const fn reason_code(self) -> &'static str {
        match self {
            Self::UniqueSymbolInExactPath => "unique_symbol_in_exact_path",
            Self::UniqueExactFilePath => "unique_exact_file_path",
            Self::GloballyUniqueSymbol => "globally_unique_symbol",
            Self::GloballyAmbiguousSymbol => "globally_ambiguous_symbol",
        }
    }
}

/// One admitted seed and why it was admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
struct StructuralSeed {
    node_id: String,
    reason_code: &'static str,
}

/// Bounded ranked seed set plus anything the caller must disclose.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct StructuralSeedSelection {
    seeds: Vec<StructuralSeed>,
    unknowns: Vec<&'static str>,
}

/// Select a bounded, ranked set of structural seeds from admitted task signals.
///
/// Ambiguity is ranked, not abandoned: several nodes sharing a name is the most
/// common useful signal about where to look, and returning nothing is strictly
/// worse than returning a ranked list with the ambiguity disclosed. Selection
/// yields nothing only when no signal matches any node.
fn structural_seed_selection(
    graph: &StructuralGraph,
    query: &str,
) -> Result<StructuralSeedSelection, context_core::CoreErrorCode> {
    if !valid_task_query(query) {
        return Err(context_core::CoreErrorCode::InvalidInput);
    }
    let signals = task_signals(query);
    let nodes = graph
        .nodes
        .iter()
        .map(|node| {
            let path = graph_node_portable_path(node)?;
            Ok((node, path))
        })
        .collect::<Result<Vec<_>, context_core::CoreErrorCode>>()?;

    // (rank, portable path, node id) — the tuple is the deterministic order.
    let mut candidates: Vec<(SeedRank, String, String)> = Vec::new();
    let mut ambiguous_observed = false;

    let is_named_symbol = |node: &GraphNode, identifier: &str| {
        node.kind == "symbol"
            && node.confidence == "confirmed"
            && node.name.as_deref() == Some(identifier)
    };

    for task_path in &signals.paths {
        let files = nodes
            .iter()
            .filter(|(node, path)| node.kind == "file" && path == task_path)
            .collect::<Vec<_>>();
        if files.len() > 1 {
            ambiguous_observed = true;
        }
        for (file, file_path) in &files {
            candidates.push((
                SeedRank::UniqueExactFilePath,
                (*file_path).clone(),
                file.node_id.clone(),
            ));
            for identifier in &signals.identifiers {
                for (symbol, symbol_path) in nodes
                    .iter()
                    .filter(|(node, path)| path == file_path && is_named_symbol(node, identifier))
                {
                    candidates.push((
                        SeedRank::UniqueSymbolInExactPath,
                        symbol_path.clone(),
                        symbol.node_id.clone(),
                    ));
                }
            }
        }
    }

    for identifier in &signals.identifiers {
        let symbols = nodes
            .iter()
            .filter(|(node, _)| is_named_symbol(node, identifier))
            .collect::<Vec<_>>();
        // An ambiguous identifier no longer aborts the scan. It is retained at
        // its own rank so a name shared by a subclass and the parent that
        // actually carries the defect can still reach the map.
        let rank = if symbols.len() > 1 {
            ambiguous_observed = true;
            SeedRank::GloballyAmbiguousSymbol
        } else {
            SeedRank::GloballyUniqueSymbol
        };
        for (symbol, path) in symbols {
            candidates.push((rank, path.clone(), symbol.node_id.clone()));
        }
    }

    candidates.sort();
    candidates.dedup_by(|left, right| left.2 == right.2);

    let mut selection = StructuralSeedSelection::default();
    let admitted = candidates.len().min(MAX_STRUCTURAL_SEEDS);
    if candidates.len() > MAX_STRUCTURAL_SEEDS {
        selection.unknowns.push("structural_seed_limit_reached");
    }
    for (rank, _, node_id) in candidates.into_iter().take(admitted) {
        selection.seeds.push(StructuralSeed {
            node_id,
            reason_code: rank.reason_code(),
        });
    }
    if ambiguous_observed {
        selection.unknowns.push("structural_seed_ambiguous");
    }
    if selection.seeds.is_empty() {
        selection.unknowns.push("structural_seed_unavailable");
    }
    selection.unknowns.sort_unstable();
    selection.unknowns.dedup();
    Ok(selection)
}

/// Merge per-seed traversals into one deterministic result.
///
/// The primary seed supplies `start_node`; nodes and edges are the deduplicated
/// union in identity order, so the merged identity is stable for a snapshot.
fn merge_structural_traversals(
    mut results: Vec<StructuralQueryResult>,
) -> Option<StructuralQueryResult> {
    let mut merged = results.first().cloned()?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut unknowns = Vec::new();
    let mut truncated = false;
    for result in results.drain(..) {
        truncated |= result.truncated;
        nodes.extend(result.nodes);
        edges.extend(result.edges);
        unknowns.extend(result.unknowns);
    }
    nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    nodes.dedup_by(|left, right| left.node_id == right.node_id);
    edges.sort_by(|left, right| left.edge_id.cmp(&right.edge_id));
    edges.dedup_by(|left, right| left.edge_id == right.edge_id);
    unknowns.sort();
    unknowns.dedup();
    merged.nodes = nodes;
    merged.edges = edges;
    merged.unknowns = unknowns;
    merged.truncated = truncated;
    Some(merged)
}

fn graph_node_portable_path(node: &GraphNode) -> Result<String, context_core::CoreErrorCode> {
    PathIdentity::from_encoded_native_units(
        &node.path.platform_family,
        &node.path.unit_encoding,
        &node.path.relative_units_base64url,
    )
    .and_then(|path| path.to_portable_relative_path())
    .map_err(|_| context_core::CoreErrorCode::IntegrityFailure)
}

fn lexical_task_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|term| !term.is_empty() && term.len() <= 64)
        .map(str::to_ascii_lowercase)
        .collect()
}

fn extract_quoted_signals(query: &str, output: &mut Vec<String>) {
    let mut open = None;
    for (index, character) in query.char_indices() {
        if let Some((delimiter, start)) = open {
            if character == delimiter {
                let candidate = &query[start..index];
                if valid_explicit_signal(candidate) {
                    push_unique_text(output, candidate.to_owned());
                }
                open = None;
            }
        } else if matches!(character, '\'' | '"' | '`') {
            open = Some((character, index + character.len_utf8()));
        }
    }
}

fn normalize_signal_token(token: &str) -> Option<String> {
    let token = token.trim_end_matches(['.', ':']);
    valid_explicit_signal(token).then(|| token.replace('\\', "/"))
}

fn valid_explicit_signal(signal: &str) -> bool {
    !signal.is_empty()
        && signal.len() <= MAX_TASK_SIGNAL_BYTES
        && signal.is_ascii()
        && !signal.bytes().any(|byte| byte.is_ascii_control())
}

fn is_portable_path_signal(token: &str) -> bool {
    if token.starts_with('/')
        || token.split('/').any(|component| component == "..")
        || token.contains("//")
    {
        return false;
    }
    if token.contains('/') {
        return true;
    }
    token.rsplit_once('.').is_some_and(|(stem, extension)| {
        !stem.is_empty()
            && (1..=16).contains(&extension.len())
            && extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
            // A version or measurement such as `1.22.3` or `99.9` is not a
            // file path. Require a letter in the final component.
            && extension.bytes().any(|byte| byte.is_ascii_alphabetic())
    })
}

/// True when the token is shaped like source code rather than prose or markup.
///
/// Separator shapes (`snake_case`, `kebab-case`, `path::qualified`) and interior
/// capitalization (`CamelCase`) both qualify. Markup runs such as `--` and
/// `---`, which open and close HTML comments in issue templates, do not.
fn is_code_identifier_signal(token: &str) -> bool {
    if !token.starts_with(|character: char| character.is_ascii_alphabetic() || character == '_') {
        return false;
    }
    token.contains('_')
        || token.contains('-')
        || token.contains("::")
        || has_interior_capital(token)
}

/// True when an uppercase letter follows a lowercase letter, as in `TimeSeries`
/// or `ValueError`. A leading capital alone, as in `This`, is ordinary prose.
fn has_interior_capital(token: &str) -> bool {
    token
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0].is_ascii_lowercase() && pair[1].is_ascii_uppercase())
}

fn push_unique_text(output: &mut Vec<String>, value: String) {
    if !output.contains(&value) {
        output.push(value);
    }
}

fn is_task_stop_word(term: &str) -> bool {
    matches!(
        term,
        "and"
            | "are"
            | "can"
            | "could"
            | "does"
            | "find"
            | "fix"
            | "for"
            | "from"
            | "how"
            | "into"
            | "please"
            | "should"
            | "show"
            | "that"
            | "the"
            | "this"
            | "use"
            | "using"
            | "was"
            | "what"
            | "when"
            | "where"
            | "which"
            | "with"
            | "would"
    )
}

fn planner_omission(candidate: &str, reason_code: &str) -> PlannerOmission {
    PlannerOmission {
        candidate: candidate.into(),
        reason_code: reason_code.into(),
        count: "1".into(),
    }
}

fn planner_coverage() -> Vec<PlannerCoverage> {
    vec![
        planner_coverage_item(
            PlannerEvidenceClass::ExactPath,
            "available",
            "exact_path_available",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::Filename,
            "available",
            "filename_available",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::Literal,
            "available",
            "literal_available",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::Lexical,
            "available",
            "lexical_available",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::StructuralRelationship,
            "unavailable",
            "structural_relationship_evidence_not_connected",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::ChangeSet,
            "unavailable",
            "change_set_evidence_unavailable",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::AssociatedTest,
            "unavailable",
            "associated_test_evidence_unavailable",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::ConfigurationToCodeReference,
            "unavailable",
            "configuration_to_code_reference_unavailable",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::ConventionExemplar,
            "unavailable",
            "convention_exemplar_evidence_unavailable",
        ),
        planner_coverage_item(
            PlannerEvidenceClass::RepositoryOrientation,
            "unavailable",
            "repository_orientation_map_not_connected",
        ),
    ]
}

fn planner_coverage_item(
    evidence_class: PlannerEvidenceClass,
    status: &str,
    reason_code: &str,
) -> PlannerCoverage {
    PlannerCoverage {
        evidence_class,
        status: status.into(),
        reason_code: reason_code.into(),
    }
}

fn structural_planner_query(
    edge_kinds: &[String],
    result: StructuralQueryResult,
) -> Result<StructuralPlannerQuery, context_core::CoreErrorCode> {
    let mut edge_kinds = edge_kinds.to_vec();
    edge_kinds.sort();
    edge_kinds.dedup();
    let mut query = StructuralPlannerQuery {
        query_id: format!("sha256:{}", "0".repeat(64)),
        edge_kinds,
        result,
    };
    query.query_id = structural_query_identity(&query)?;
    Ok(query)
}

fn apply_structural_query_to_plan(
    plan: &mut DeterministicContextPlan,
    structural_query: &StructuralPlannerQuery,
    reason_code: &str,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::StructuralRelationship)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "available".into();
    coverage.reason_code = reason_code.into();
    plan.omitted_candidates.retain(|item| {
        item.candidate != "structural_relationship"
            || item.reason_code != "structural_relationship_evidence_not_connected"
    });
    plan.structural_query = Some(structural_query.clone());
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn apply_structural_annotation_to_plan(
    plan: &mut DeterministicContextPlan,
    structural_query: Option<&StructuralPlannerQuery>,
    structural_annotation: Option<StructuralPlanAnnotation<'_>>,
) -> Result<(), context_core::CoreErrorCode> {
    match (structural_query, structural_annotation) {
        (Some(query), Some(StructuralPlanAnnotation::Available(reason_code))) => {
            apply_structural_query_to_plan(plan, query, reason_code)
        }
        (None, Some(StructuralPlanAnnotation::Omitted(reason_code))) => {
            apply_structural_omission_to_plan(plan, reason_code)
        }
        (None, None) => Ok(()),
        _ => Err(context_core::CoreErrorCode::IntegrityFailure),
    }
}

fn apply_structural_omission_to_plan(
    plan: &mut DeterministicContextPlan,
    reason_code: &str,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::StructuralRelationship)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "unavailable".into();
    coverage.reason_code = reason_code.into();
    plan.omitted_candidates
        .retain(|item| item.candidate != "structural_relationship");
    plan.omitted_candidates
        .push(planner_omission("structural_relationship", reason_code));
    plan.omitted_candidates.sort_by(|left, right| {
        left.candidate
            .cmp(&right.candidate)
            .then(left.reason_code.cmp(&right.reason_code))
    });
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn apply_declared_change_set_to_plan(
    plan: &mut DeterministicContextPlan,
    declared_change_set: &VerifiedDeclaredChangeSet,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::ChangeSet)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "available".into();
    coverage.reason_code = "declared_change_set_current_snapshot_verified".into();
    plan.omitted_candidates.retain(|item| {
        item.candidate != "change_set" || item.reason_code != "change_set_evidence_unavailable"
    });
    plan.declared_change_set = Some(declared_change_set.clone());
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn apply_declared_associated_tests_to_plan(
    plan: &mut DeterministicContextPlan,
    associated: &VerifiedDeclaredAssociatedTests,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::AssociatedTest)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "available".into();
    coverage.reason_code = "declared_associated_test_current_snapshot_verified".into();
    plan.omitted_candidates
        .retain(|item| item.candidate != "associated_test");
    plan.declared_associated_tests = Some(associated.clone());
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn apply_declared_convention_exemplars_to_plan(
    plan: &mut DeterministicContextPlan,
    conventions: &VerifiedDeclaredConventionExemplars,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::ConventionExemplar)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "available".into();
    coverage.reason_code = "declared_convention_exemplar_current_snapshot_verified".into();
    plan.declared_convention_exemplars = Some(conventions.clone());
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn apply_repository_orientation_to_plan(
    plan: &mut DeterministicContextPlan,
    orientation: &RepositoryOrientationMap,
) -> Result<(), context_core::CoreErrorCode> {
    let coverage = plan
        .coverage
        .iter_mut()
        .find(|item| item.evidence_class == PlannerEvidenceClass::RepositoryOrientation)
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?;
    coverage.status = "available".into();
    coverage.reason_code = "validated_repository_orientation_map_available".into();
    plan.repository_orientation = Some(orientation.clone());
    plan.plan_id = deterministic_plan_identity(plan)?;
    Ok(())
}

fn repository_orientation_identity(
    orientation: &RepositoryOrientationMap,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected = serde_json::to_value(orientation)
        .map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("map_id");
    canonical_identity("repository-orientation-map", &projected)
}

fn declared_associated_tests_identity(
    value: &VerifiedDeclaredAssociatedTests,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected =
        serde_json::to_value(value).map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("association_id");
    canonical_identity("declared-associated-tests", &projected)
}

fn declared_convention_exemplars_identity(
    value: &VerifiedDeclaredConventionExemplars,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected =
        serde_json::to_value(value).map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("declaration_id");
    canonical_identity("declared-convention-exemplars", &projected)
}

fn declared_change_set_identity(
    declared_change_set: &VerifiedDeclaredChangeSet,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected = serde_json::to_value(declared_change_set)
        .map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("declaration_id");
    canonical_identity("declared-change-set", &projected)
}

fn structural_query_identity(
    query: &StructuralPlannerQuery,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected =
        serde_json::to_value(query).map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("query_id");
    canonical_identity("structural-planner-query", &projected)
}

fn deterministic_plan_identity(
    plan: &DeterministicContextPlan,
) -> Result<String, context_core::CoreErrorCode> {
    let mut projected =
        serde_json::to_value(plan).map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    projected
        .as_object_mut()
        .ok_or(context_core::CoreErrorCode::IntegrityFailure)?
        .remove("plan_id");
    canonical_identity("deterministic-context-plan", &projected)
}

fn canonical_identity(
    kind: &str,
    value: &serde_json::Value,
) -> Result<String, context_core::CoreErrorCode> {
    let canonical = serde_json_canonicalizer::to_vec(value)
        .map_err(|_| context_core::CoreErrorCode::IntegrityFailure)?;
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(CONTRACT_VERSION.as_bytes());
    hasher.update(b"\0");
    hasher.update(canonical);
    let mut identity = String::from("sha256:");
    for byte in hasher.finalize() {
        use fmt::Write as _;
        write!(identity, "{byte:02x}").expect("string write");
    }
    Ok(identity)
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

/// Supported files the build will visit, which is the fact-quota divisor.
fn supported_file_count(
    artifacts: &[context_workspace::ArtifactRecord],
    file_limit: u64,
    scope: Option<&std::collections::BTreeSet<String>>,
) -> u32 {
    u32::try_from(
        artifacts
            .iter()
            .take(usize::try_from(file_limit).unwrap_or(usize::MAX))
            .filter(|artifact| artifact_in_scope(&artifact.path.display_path, scope))
            .filter(|artifact| structural_language(&artifact.path.display_path).is_some())
            .count(),
    )
    .unwrap_or(u32::MAX)
}

/// Whether one artifact belongs to the admitted structural scope.
///
/// No scope means a whole-repository build, which is thin but complete. A scope
/// means a nominated build, which is dense but partial.
fn artifact_in_scope(
    display_path: &str,
    scope: Option<&std::collections::BTreeSet<String>>,
) -> bool {
    scope.is_none_or(|paths| paths.contains(display_path))
}

fn structural_fact_quota(remaining_facts: u32, remaining_files: u32) -> Option<u32> {
    (remaining_facts > 0 && remaining_files > 0).then(|| remaining_facts.div_ceil(remaining_files))
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
            FactClass::Reference,
        ],
        max_facts: limits.facts,
        max_nesting_depth: limits.depth,
        max_response_bytes: limits.response_bytes,
        parser_version: "tree-sitter-0.26.13".into(),
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
        Some(extension) if extension.eq_ignore_ascii_case("py") => Some(StructuralLanguage::Python),
        Some(extension) if extension.eq_ignore_ascii_case("java") => Some(StructuralLanguage::Java),
        Some(extension)
            if extension.eq_ignore_ascii_case("kt") || extension.eq_ignore_ascii_case("kts") =>
        {
            Some(StructuralLanguage::Kotlin)
        }
        Some(extension) if extension.eq_ignore_ascii_case("cs") => Some(StructuralLanguage::CSharp),
        Some(extension)
            if extension.eq_ignore_ascii_case("c") || extension.eq_ignore_ascii_case("h") =>
        {
            Some(StructuralLanguage::C)
        }
        Some(extension)
            if matches!(
                extension.to_ascii_lowercase().as_str(),
                "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx"
            ) =>
        {
            Some(StructuralLanguage::Cpp)
        }
        Some(extension) if extension.eq_ignore_ascii_case("rb") => Some(StructuralLanguage::Ruby),
        Some(extension) if extension.eq_ignore_ascii_case("php") => Some(StructuralLanguage::Php),
        Some(extension) if extension.eq_ignore_ascii_case("swift") => {
            Some(StructuralLanguage::Swift)
        }
        Some(extension) if extension.eq_ignore_ascii_case("scala") => {
            Some(StructuralLanguage::Scala)
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("ex") || extension.eq_ignore_ascii_case("exs") =>
        {
            Some(StructuralLanguage::Elixir)
        }
        Some(extension)
            if matches!(
                extension.to_ascii_lowercase().as_str(),
                "clj" | "cljs" | "cljc"
            ) =>
        {
            Some(StructuralLanguage::Clojure)
        }
        Some(extension) if matches!(extension.to_ascii_lowercase().as_str(), "hs" | "lhs") => {
            Some(StructuralLanguage::Haskell)
        }
        Some(extension) if extension.eq_ignore_ascii_case("go") => Some(StructuralLanguage::Go),
        Some(extension) if extension.eq_ignore_ascii_case("rs") => Some(StructuralLanguage::Rust),
        Some(extension) if extension.eq_ignore_ascii_case("toml") => Some(StructuralLanguage::Toml),
        Some(extension)
            if extension.eq_ignore_ascii_case("yaml") || extension.eq_ignore_ascii_case("yml") =>
        {
            Some(StructuralLanguage::Yaml)
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("json") && is_strict_json_configuration(path) =>
        {
            Some(StructuralLanguage::Json)
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("json") && is_jsonc_configuration(path) =>
        {
            Some(StructuralLanguage::Jsonc)
        }
        Some(extension)
            if extension.eq_ignore_ascii_case("jsonc") && is_jsonc_configuration(path) =>
        {
            Some(StructuralLanguage::Jsonc)
        }
        _ => None,
    }
}

fn is_strict_json_configuration(path: &str) -> bool {
    matches!(
        Path::new(path).file_name().and_then(|value| value.to_str()),
        Some("package.json" | "deno.json" | "composer.json" | "manifest.json")
    )
}

fn is_jsonc_configuration(path: &str) -> bool {
    let path = Path::new(path);
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("jsonc"))
    {
        return true;
    }
    let Some(filename) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    matches!(
        filename,
        "tsconfig.json" | "jsconfig.json" | "devcontainer.json"
    ) || (path
        .parent()
        .is_some_and(|parent| parent.ends_with(".vscode"))
        && matches!(
            filename,
            "settings.json" | "tasks.json" | "launch.json" | "extensions.json"
        ))
}

const fn grammar_version(language: StructuralLanguage) -> &'static str {
    match language {
        StructuralLanguage::TypeScript | StructuralLanguage::Tsx => "tree-sitter-typescript-0.23.2",
        StructuralLanguage::JavaScript | StructuralLanguage::Jsx => "tree-sitter-javascript-0.25.0",
        StructuralLanguage::Python => "tree-sitter-python-0.25.0",
        StructuralLanguage::Java => "tree-sitter-java-0.23.5",
        StructuralLanguage::Kotlin => "tree-sitter-kotlin-ng-1.1.0",
        StructuralLanguage::CSharp => "tree-sitter-c-sharp-0.23.5",
        StructuralLanguage::C => "tree-sitter-c-0.24.2",
        StructuralLanguage::Cpp => "tree-sitter-cpp-0.23.4",
        StructuralLanguage::Ruby => "tree-sitter-ruby-0.23.1",
        StructuralLanguage::Php => "tree-sitter-php-0.24.2",
        StructuralLanguage::Swift => "tree-sitter-swift-0.7.3",
        StructuralLanguage::Scala => "tree-sitter-scala-0.26.2",
        StructuralLanguage::Elixir => "tree-sitter-elixir-0.3.5",
        StructuralLanguage::Clojure => "tree-sitter-clojure-orchard-0.2.8",
        StructuralLanguage::Haskell => "tree-sitter-haskell-0.23.1",
        StructuralLanguage::Json | StructuralLanguage::Jsonc => "tree-sitter-json-0.24.8",
        StructuralLanguage::Toml => "tree-sitter-toml-ng-0.7.0",
        StructuralLanguage::Yaml => "tree-sitter-yaml-0.7.2",
        StructuralLanguage::Go => "tree-sitter-go-0.25.0",
        StructuralLanguage::Rust => "tree-sitter-rust-0.24.2",
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

fn structural_cache_unavailable(
    context: &RequestContext,
    workspace: &str,
    snapshot: &str,
) -> EngineError {
    failure(
        context,
        Capability::StructureBuild,
        PublicErrorCode::InternalFailure,
        "structural cache is unavailable",
        Some(workspace),
        Some(snapshot),
        None,
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

/// Derive the audit identity for one structural traversal.
///
/// `ordinal` distinguishes the traversals of a multi-seed selection, because
/// the audit store rejects a duplicate event identity and every seed records
/// its own traversal.
/// Derive the audit identity for one task-scoped structural build.
///
/// The caller's context is still to be used for the context build that follows,
/// and the audit store rejects a duplicate event identity.
fn derived_task_scope_context(context: &RequestContext) -> RequestContext {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0task-scoped-structure-event\0");
    hasher.update(context.event_id.as_bytes());
    let mut event_id = String::from("evt_");
    for byte in hasher.finalize() {
        use fmt::Write as _;
        write!(event_id, "{byte:02x}").expect("string write");
    }
    RequestContext {
        request_id: context.request_id.clone(),
        event_id,
        subject: context.subject.clone(),
        occurred_at: context.occurred_at.clone(),
    }
}

fn derived_structure_query_context(context: &RequestContext, ordinal: usize) -> RequestContext {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context\0structural-impact-query-event\0");
    hasher.update(context.event_id.as_bytes());
    hasher.update(b"\0");
    hasher.update(ordinal.to_be_bytes());
    let mut event_id = String::from("evt_");
    for byte in hasher.finalize() {
        use fmt::Write as _;
        write!(event_id, "{byte:02x}").expect("string write");
    }
    RequestContext {
        request_id: context.request_id.clone(),
        event_id,
        subject: context.subject.clone(),
        occurred_at: context.occurred_at.clone(),
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn contract_sha256(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in Sha256::digest(bytes) {
        use fmt::Write as _;
        write!(value, "{byte:02x}").expect("writing to a string cannot fail");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::validate_packet;
    use context_dashboard::{
        BudgetCeilings, BudgetSelector, LocalBudgetPolicyDraft, LocalBudgetRule, compile_policy,
    };
    use jsonschema::Registry;
    use serde::Serialize;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn scope_admits_only_nominated_files_and_no_scope_admits_everything() {
        let nominated: std::collections::BTreeSet<String> =
            ["a.py".to_owned(), "pkg/b.py".to_owned()]
                .into_iter()
                .collect();
        assert!(artifact_in_scope("a.py", Some(&nominated)));
        assert!(artifact_in_scope("pkg/b.py", Some(&nominated)));
        assert!(!artifact_in_scope("pkg/c.py", Some(&nominated)));
        // A prefix or suffix of a nominated path is a different file.
        assert!(!artifact_in_scope("b.py", Some(&nominated)));
        assert!(!artifact_in_scope("pkg/b.pyc", Some(&nominated)));

        // No scope keeps whole-repository behaviour exactly.
        assert!(artifact_in_scope("anything.py", None));

        // An empty scope admits nothing rather than everything.
        let empty = std::collections::BTreeSet::new();
        assert!(!artifact_in_scope("a.py", Some(&empty)));
    }

    #[test]
    fn scoping_is_what_produces_density() {
        // The allowance is fixed and the divisor is the supported file count,
        // which is the whole argument for nominating files: the same budget
        // over sixteen files is hundreds of facts each, and over a repository
        // is one.
        let allowance = 10_000;
        let scoped = structural_fact_quota(allowance, 16).expect("scoped quota");
        let whole_repository = structural_fact_quota(allowance, 1_172).expect("repository quota");
        assert!(scoped > 600, "scoped quota was {scoped}");
        assert!(
            whole_repository < 10,
            "repository quota was {whole_repository}"
        );
        assert!(scoped > whole_repository * 60);
    }

    #[test]
    fn structural_fact_quota_is_repository_wide_and_fair() {
        assert_eq!(structural_fact_quota(10_000, 1_173), Some(9));
        assert_eq!(structural_fact_quota(9_991, 1_172), Some(9));
        assert_eq!(structural_fact_quota(3, 5), Some(1));
        assert_eq!(structural_fact_quota(0, 5), None);
        assert_eq!(structural_fact_quota(5, 0), None);
    }

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

    #[test]
    fn cached_worker_results_rebind_only_transport_correlation() {
        let source = b"export function value() { return 1; }";
        let path = PathIdentity::from_portable_relative_path("src/value.ts").expect("path");
        let request = WorkerRequest {
            schema_name: "structural-worker-request".into(),
            schema_version: PROTOCOL_VERSION.into(),
            request_id: "req_cached_worker_a".into(),
            language: StructuralLanguage::TypeScript,
            path: WorkerPath {
                display_path: path.display_path,
                platform_family: path.platform_family.into(),
                unit_encoding: path.unit_encoding.into(),
                relative_units_base64url: path.relative_units_base64url,
            },
            content_hash: contract_sha256(source),
            source_base64url: URL_SAFE_NO_PAD.encode(source),
            fact_classes: vec![FactClass::Declaration],
            max_facts: 100,
            max_nesting_depth: 8,
            max_response_bytes: 65_536,
            parser_version: "tree-sitter-0.26.13".into(),
            grammar_version: "tree-sitter-typescript-0.23.2".into(),
            resolver_version: RESOLVER_VERSION.into(),
            graph_version: GRAPH_VERSION.into(),
        };
        let response = context_structural::process_request(&request).expect("worker result");
        let payload = serde_json::to_vec(&response).expect("payload");
        let mut repeated = request.clone();
        repeated.request_id = "req_cached_worker_b".into();
        let rebound = cached_worker_success(&payload, &repeated).expect("rebound cache result");
        assert_eq!(rebound.request_id, repeated.request_id);

        let mut corrupt = response;
        corrupt.content_hash = contract_sha256(b"different");
        assert!(
            cached_worker_success(
                &serde_json::to_vec(&corrupt).expect("corrupt payload"),
                &repeated
            )
            .is_none()
        );
    }

    #[test]
    fn task_signal_extraction_is_bounded_closed_and_high_signal() {
        let signals = task_signals(
            "Please inspect `panic in parser` in src/parser.rs and fix parse_node for hello-rust",
        );
        assert_eq!(signals.quoted, vec!["panic in parser"]);
        assert_eq!(signals.paths, vec!["src/parser.rs"]);
        assert_eq!(signals.identifiers, vec!["parse_node", "hello-rust"]);
        assert!(signals.lexical.contains(&"parser".into()));
        assert!(signals.lexical.contains(&"hello".into()));
        assert!(!signals.lexical.contains(&"please".into()));

        let flooded = (0..100)
            .map(|index| format!("identifier_{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let flooded_signals = task_signals(&flooded);
        assert_eq!(flooded_signals.identifiers.len(), MAX_TASK_SIGNAL_TOKENS);
    }

    #[test]
    fn issue_template_prose_does_not_starve_or_pollute_code_signals() {
        // A real bug report opens with a title and an HTML comment block. The
        // first code signal appears far past the first sixteen words.
        let query = "TimeSeries: misleading exception when required column check fails.\n             <!-- This comments are hidden when you submit the issue,\n             so you do not need to remove them! -->\n             <!-- Please be sure to check out our contributing guidelines. -->\n             ### Description\n             ts.remove_column(\"flux\") raises after setting _required_columns.\n             See astropy/timeseries/sampled.py for the check.\n             Numpy 1.22.3 and Python 3.9.10.";
        let signals = task_signals(query);

        // Markup runs are not code identifiers.
        assert!(!signals.identifiers.iter().any(|value| value == "--"));
        assert!(!signals.identifiers.iter().any(|value| value == "---"));

        // Interior capitalization is a code shape; a leading capital is prose.
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "TimeSeries")
        );
        assert!(!signals.identifiers.iter().any(|value| value == "This"));
        assert!(!signals.identifiers.iter().any(|value| value == "Please"));

        // Signals past the sixteenth word still reach the lists.
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "_required_columns")
        );
        assert!(
            signals
                .paths
                .iter()
                .any(|value| value == "astropy/timeseries/sampled.py")
        );

        // Versions and measurements are not file paths.
        assert!(!signals.paths.iter().any(|value| value == "1.22.3"));
        assert!(!signals.paths.iter().any(|value| value == "3.9.10"));

        // Each list stays bounded.
        assert!(signals.paths.len() <= MAX_TASK_SIGNAL_TOKENS);
        assert!(signals.identifiers.len() <= MAX_TASK_SIGNAL_TOKENS);
    }

    #[test]
    fn attribute_chains_yield_the_member_a_graph_node_actually_carries() {
        // A report writes `ts.remove_column("flux")`, but the graph holds
        // `remove_column`. Without the final component the signal never matches.
        let signals = task_signals(
            "ts._required_columns = [\"time\"] then ts.remove_column(\"flux\") \
             and numpy.__version__ printed",
        );
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "_required_columns")
        );
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "remove_column")
        );
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "__version__")
        );

        // The whole chain is retained too, so an exact dotted name still works.
        assert!(
            signals
                .identifiers
                .iter()
                .any(|value| value == "ts._required_columns")
        );

        // A member that is not code-shaped is not admitted.
        let prose = task_signals("The end. Another sentence.");
        assert!(!prose.identifiers.iter().any(|value| value == "Another"));
    }

    #[test]
    fn scanned_token_ceiling_still_bounds_a_hostile_query() {
        let hostile = (0..100_000)
            .map(|index| format!("word{index} -- ."))
            .collect::<Vec<_>>()
            .join(" ");
        let signals = task_signals(&hostile);
        assert!(signals.paths.len() <= MAX_TASK_SIGNAL_TOKENS);
        assert!(signals.identifiers.len() <= MAX_TASK_SIGNAL_TOKENS);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn structural_seed_selection_is_deterministic_bounded_and_non_oracular() {
        let source = TestRoot::new("structural-seed-source");
        let cache = TestRoot::new("structural-seed-cache");
        fs::write(
            source.0.join("review.ts"),
            b"export function reviewed_change() { return helper(); }\nfunction helper() { return 1; }\nfunction duplicate_name() {}\n",
        )
        .expect("review source");
        fs::write(
            source.0.join("other.ts"),
            b"export function duplicate_name() { return 2; }\n",
        )
        .expect("other source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 2_048, 2_048, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 30, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("snapshot");
        let snapshot = engine.snapshot.as_ref().expect("snapshot");
        let provenance = context_structural::FactProvenance {
            method: "tree_sitter".into(),
            parser_version: "tree-sitter-0.26.13".into(),
            grammar_version: "tree-sitter-typescript-0.23.2".into(),
            resolver_version: RESOLVER_VERSION.into(),
            graph_version: GRAPH_VERSION.into(),
        };
        let declaration = |local_key: &str, name: &str, start_byte: u64, end_byte: u64| {
            context_structural::StructuralFact {
                class: FactClass::Declaration,
                local_key: local_key.into(),
                syntax_kind: "function_declaration".into(),
                name: Some(name.into()),
                module: None,
                start_byte,
                end_byte,
                parent_key: None,
                confidence: "confirmed".into(),
                provenance: provenance.clone(),
            }
        };
        let inputs = snapshot
            .artifacts
            .iter()
            .map(|artifact| {
                let facts = if artifact.path.display_path == "review.ts" {
                    vec![
                        declaration("reviewed", "reviewed_change", 16, 55),
                        declaration("helper", "helper", 56, 87),
                        declaration("duplicate_review", "duplicate_name", 88, 116),
                        context_structural::StructuralFact {
                            class: FactClass::Call,
                            local_key: "reviewed_calls_helper".into(),
                            syntax_kind: "call_expression".into(),
                            name: Some("helper".into()),
                            module: None,
                            start_byte: 45,
                            end_byte: 53,
                            parent_key: Some("reviewed".into()),
                            confidence: "heuristic".into(),
                            provenance: provenance.clone(),
                        },
                    ]
                } else {
                    vec![declaration("duplicate_other", "duplicate_name", 16, 44)]
                };
                GraphFileInput {
                    path: WorkerPath {
                        display_path: artifact.path.display_path.clone(),
                        platform_family: artifact.path.platform_family.into(),
                        unit_encoding: artifact.path.unit_encoding.into(),
                        relative_units_base64url: artifact.path.relative_units_base64url.clone(),
                    },
                    response: context_structural::WorkerSuccess {
                        schema_name: "structural-worker-success".into(),
                        schema_version: PROTOCOL_VERSION.into(),
                        request_id: "req_structural_seed".into(),
                        content_hash: artifact.content_hash.clone(),
                        syntax_errors: false,
                        facts,
                        warnings: Vec::new(),
                    },
                }
            })
            .collect::<Vec<_>>();
        let graph = context_structural::build_graph(&snapshot.snapshot_id, inputs).expect("graph");

        // Structural preparation opens the shared namespace before lexical
        // retrieval. An open cache is not proof that this snapshot has a
        // promoted lexical generation.
        engine.cache = Some(
            WorkspaceCache::open(&cache.0, engine.workspace.identity()).expect("open shared cache"),
        );

        let exact = structural_seed_selection(
            &graph,
            "Inspect reviewed_change in review.ts and explain its helper call",
        )
        .expect("seed");
        assert_eq!(
            exact.seeds.first().map(|seed| seed.reason_code),
            Some("unique_symbol_in_exact_path")
        );
        let reordered_non_signal = structural_seed_selection(
            &graph,
            "Could you carefully explain reviewed_change in review.ts",
        )
        .expect("reordered non-signal seed");
        assert_eq!(exact, reordered_non_signal);

        let file = structural_seed_selection(&graph, "Inspect review.ts").expect("file seed");
        assert_eq!(
            file.seeds.first().map(|seed| seed.reason_code),
            Some("unique_exact_file_path")
        );

        let global =
            structural_seed_selection(&graph, "Inspect reviewed_change").expect("global seed");
        assert_eq!(
            global.seeds.first().map(|seed| seed.reason_code),
            Some("globally_unique_symbol")
        );

        // An ambiguous name is now retained and disclosed rather than
        // abandoned. Returning nothing was strictly worse than returning a
        // ranked list with the ambiguity recorded.
        let ambiguous = structural_seed_selection(&graph, "Investigate duplicate_name")
            .expect("ambiguous seed");
        assert!(ambiguous.seeds.len() > 1);
        assert!(
            ambiguous
                .seeds
                .iter()
                .all(|seed| seed.reason_code == "globally_ambiguous_symbol")
        );
        assert!(ambiguous.unknowns.contains(&"structural_seed_ambiguous"));

        // Selection yields nothing only when no signal matches any node.
        let unavailable = structural_seed_selection(
            &graph,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ../escape",
        )
        .expect("unavailable seed");
        assert!(unavailable.seeds.is_empty());
        assert!(
            unavailable
                .unknowns
                .contains(&"structural_seed_unavailable")
        );

        // Every seed set stays bounded and deterministically ordered.
        assert!(exact.seeds.len() <= MAX_STRUCTURAL_SEEDS);
        assert!(ambiguous.seeds.len() <= MAX_STRUCTURAL_SEEDS);
        let repeated = structural_seed_selection(&graph, "Investigate duplicate_name")
            .expect("repeat ambiguous seed");
        assert_eq!(ambiguous, repeated);

        let seeded = engine
            .build_profiled_seeded_structural_context(
                &request(3, "structural_seed"),
                TaskProfile::BugInvestigation,
                "Inspect reviewed_change in review.ts and explain its helper call",
                &StructuralSeedRequest {
                    graph: graph.clone(),
                    edge_kinds: vec!["calls".into()],
                },
                budget(),
            )
            .expect("seeded packet");
        let structural = seeded
            .plan
            .structural_query
            .as_ref()
            .expect("structural query");
        assert_eq!(structural.result.edges.len(), 1);
        assert_eq!(structural.result.edges[0].kind, "calls");
        assert!(structural.result.nodes.len() <= 16);
        assert!(structural.result.edges.len() <= 16);
        assert!(seeded.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::StructuralRelationship
                && coverage.status == "available"
                && coverage.reason_code == "unique_symbol_in_exact_path"
        }));
        assert!(
            seeded
                .packet
                .observed_evidence
                .iter()
                .any(|evidence| { evidence.extraction.method == "structural_graph_edge" })
        );
        let structural_position = seeded
            .packet
            .observed_evidence
            .iter()
            .position(|evidence| evidence.extraction.method == "structural_graph_edge")
            .expect("structural evidence position");
        assert!(
            structural_position > 0,
            "exact task anchors must rank first"
        );
        assert!(
            seeded.packet.observed_evidence[..structural_position]
                .iter()
                .any(|evidence| evidence.artifact.path.display_path == "review.ts")
        );
        validate_packet(&seeded.packet).expect("valid seeded packet");

        // An ambiguous name now produces structural context covering every
        // candidate, with the ambiguity disclosed, instead of suppressing the
        // query. Abandoning on ambiguity is what left maps empty on real tasks.
        let ambiguous_packet = engine
            .build_profiled_seeded_structural_context(
                &request(4, "structural_seed_fallback"),
                TaskProfile::BugInvestigation,
                "Investigate duplicate_name",
                &StructuralSeedRequest {
                    graph: graph.clone(),
                    edge_kinds: vec!["calls".into()],
                },
                budget(),
            )
            .expect("ambiguous packet");
        let ambiguous_query = ambiguous_packet
            .plan
            .structural_query
            .as_ref()
            .expect("ambiguous structural query");
        assert!(
            ambiguous_query
                .result
                .unknowns
                .iter()
                .any(|unknown| unknown == "structural_seed_ambiguous")
        );
        assert!(ambiguous_packet.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::StructuralRelationship
                && coverage.reason_code == "globally_ambiguous_symbol"
        }));
        validate_packet(&ambiguous_packet.packet).expect("valid ambiguous packet");

        // Nothing matching still yields no structural query.
        let unavailable_packet = engine
            .build_profiled_seeded_structural_context(
                &request(7, "structural_seed_unavailable"),
                TaskProfile::BugInvestigation,
                "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb ../escape",
                &StructuralSeedRequest {
                    graph: graph.clone(),
                    edge_kinds: vec!["calls".into()],
                },
                budget(),
            )
            .expect("unavailable packet");
        assert!(unavailable_packet.plan.structural_query.is_none());
        assert!(unavailable_packet.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::StructuralRelationship
                && coverage.status == "unavailable"
                && coverage.reason_code == "structural_seed_unavailable"
        }));
        validate_packet(&unavailable_packet.packet).expect("valid unavailable packet");

        let mut stale_graph = graph;
        stale_graph.workspace_snapshot =
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into();
        let stale = engine
            .build_profiled_seeded_structural_context(
                &request(5, "structural_seed_stale"),
                TaskProfile::BugInvestigation,
                "Inspect reviewed_change in review.ts",
                &StructuralSeedRequest {
                    graph: stale_graph,
                    edge_kinds: vec!["calls".into()],
                },
                budget(),
            )
            .expect_err("stale graph");
        assert_eq!(stale.envelope().code, PublicErrorCode::StaleState);
    }

    #[test]
    fn descriptive_task_plan_preserves_exact_identifier_and_path_signals() {
        let query = "Find the Rust greeting hello-rust in rust.rs";
        let plan = deterministic_plan(
            TaskProfile::BugInvestigation,
            query,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            128,
        )
        .expect("plan");
        assert_eq!(plan.steps[0].step.kind, QueryKind::Literal);
        assert_eq!(plan.steps[0].step.query, query);
        assert!(plan.steps.iter().any(|step| {
            step.step.kind == QueryKind::Literal
                && step.step.query == "hello-rust"
                && step.reason_code == "task_signal_code_identifier"
        }));
        assert!(plan.steps.iter().any(|step| {
            step.step.kind == QueryKind::Filename
                && step.step.query == "rust.rs"
                && step.reason_code == "task_signal_portable_path"
        }));
        assert!(plan.steps.len() <= MAX_PROFILE_STEPS);
    }

    #[test]
    fn literal_steps_respect_the_actual_excerpt_budget() {
        let query = "Find the Rust greeting hello-rust in rust.rs";
        let plan = deterministic_plan(
            TaskProfile::BugInvestigation,
            query,
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            10,
        )
        .expect("plan");
        assert!(
            plan.steps.iter().all(|step| {
                step.step.kind != QueryKind::Literal || step.step.query.len() <= 10
            })
        );
        assert!(plan.steps.iter().any(|step| {
            step.step.kind == QueryKind::Literal
                && step.step.query == "hello-rust"
                && step.reason_code == "task_signal_code_identifier"
        }));
        assert!(plan.omitted_candidates.iter().any(|omission| {
            omission.candidate == "original_query"
                && omission.reason_code == "original_query_exceeds_retrieval_contract"
        }));
    }

    #[test]
    fn exact_and_descriptive_tasks_recover_the_same_explicit_anchor() {
        let source = TestRoot::new("task-signal-source");
        let cache = TestRoot::new("task-signal-cache");
        fs::write(
            source.0.join("rust.rs"),
            b"pub const GREETING: &str = \"hello-rust\";\n",
        )
        .expect("source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("snapshot");
        let exact = engine
            .build_profiled_context(
                &request(3, "bug_investigation"),
                TaskProfile::BugInvestigation,
                "hello-rust",
                budget(),
            )
            .expect("exact packet");
        let descriptive = engine
            .build_profiled_context(
                &request(4, "bug_investigation"),
                TaskProfile::BugInvestigation,
                "Find the Rust greeting hello-rust in rust.rs",
                budget(),
            )
            .expect("descriptive packet");
        let exact_anchor = exact
            .packet
            .observed_evidence
            .iter()
            .find(|item| {
                item.extraction.method == "literal_search"
                    && item
                        .span
                        .start_byte
                        .parse::<u64>()
                        .ok()
                        .zip(item.span.end_byte.parse::<u64>().ok())
                        .is_some_and(|(start, end)| end.saturating_sub(start) == 10)
            })
            .expect("exact anchor");
        assert!(descriptive.packet.observed_evidence.iter().any(|item| {
            item.evidence_id == exact_anchor.evidence_id
                && item.artifact.path.display_path == "rust.rs"
        }));
        validate_packet(&exact.packet).expect("valid exact packet");
        validate_packet(&descriptive.packet).expect("valid descriptive packet");
    }

    #[test]
    fn unsupported_controls_are_rejected_but_multiline_tasks_are_allowed() {
        let snapshot = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let policy = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        assert!(
            deterministic_plan(
                TaskProfile::BugInvestigation,
                "line one\nhello-rust",
                snapshot,
                policy,
                128,
            )
            .is_ok()
        );
        assert_eq!(
            deterministic_plan(
                TaskProfile::BugInvestigation,
                "hello\u{0007}rust",
                snapshot,
                policy,
                128,
            )
            .expect_err("control"),
            context_core::CoreErrorCode::InvalidInput
        );
    }

    #[test]
    fn long_and_adversarial_tasks_are_decomposed_without_syntax_authority() {
        let snapshot = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let policy = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let long_query = (0..40)
            .map(|index| format!("ordinaryword{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let long_plan = deterministic_plan(
            TaskProfile::Implementation,
            &long_query,
            snapshot,
            policy,
            128,
        )
        .expect("long plan");
        assert!(long_plan.steps.len() <= MAX_PROFILE_STEPS);
        assert!(
            long_plan
                .steps
                .iter()
                .all(|step| step.step.query != long_query)
        );
        assert!(long_plan.omitted_candidates.iter().any(|omission| {
            omission.candidate == "original_query"
                && omission.reason_code == "original_query_exceeds_retrieval_contract"
        }));

        let hostile = "$(touch /tmp/pwn) ../outside.rs foo* OR bar | sh";
        let hostile_plan = deterministic_plan(
            TaskProfile::BugInvestigation,
            hostile,
            snapshot,
            policy,
            128,
        )
        .expect("hostile text stays inert");
        assert!(hostile_plan.steps.len() <= MAX_PROFILE_STEPS);
        assert!(
            hostile_plan
                .steps
                .iter()
                .all(|step| step.step.kind != QueryKind::ExactPath)
        );
        assert!(
            hostile_plan.steps.iter().skip(1).all(|step| {
                !step.step.query.starts_with('/') && !step.step.query.contains("..")
            })
        );
    }

    #[test]
    fn repository_read_telemetry_requires_an_exhaustive_current_snapshot() {
        let source = TestRoot::new("read-telemetry-source");
        let cache = TestRoot::new("read-telemetry-cache");
        fs::write(source.0.join("lib.rs"), b"fn answer() {}\n").expect("source");
        fs::create_dir(source.0.join("target")).expect("excluded directory");
        fs::write(source.0.join("target/generated"), b"generated").expect("excluded source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(20, 4096, 4096, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576)
                .expect("audit"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "telemetry"), &source.0).expect("open");

        let before = engine.repository_read_telemetry();
        assert!(!before.complete);
        assert_eq!(before.repository_file_reads, 0);
        assert_eq!(before.source_fingerprint_sha256.len(), 71);

        engine
            .build_snapshot(&request(2, "telemetry"), budget())
            .expect("snapshot");
        let after = engine.repository_read_telemetry();
        assert!(!after.complete, "a skipped object must fail closed");
        assert_eq!(after.repository_file_reads, 1);
        assert_eq!(after.repeated_repository_file_reads, 0);
        assert_eq!(after.source_bytes_read, 15);
    }

    #[test]
    fn exact_owned_local_policy_narrows_runtime_and_audit_limits() {
        let source = TestRoot::new("budget-policy-source");
        let cache = TestRoot::new("budget-policy-cache");
        let state_parent = TestRoot::new("budget-policy-state");
        fs::write(source.0.join("one.rs"), b"fn one() {}\n").expect("one");
        fs::write(source.0.join("two.rs"), b"fn two() {}\n").expect("two");
        let policy = compile_policy(LocalBudgetPolicyDraft {
            schema_name: "local-budget-policy".into(),
            schema_version: "1.0.0".into(),
            revision: "1".into(),
            created_at: "2026-08-21T00:00:00Z".into(),
            expires_at: None,
            rules: vec![LocalBudgetRule {
                rule_id: "snapshot_one_file".into(),
                selector: BudgetSelector {
                    purpose: None,
                    capability: Some(Capability::SnapshotBuild),
                },
                deny: false,
                ceilings: BudgetCeilings {
                    max_files: Some("1".into()),
                    ..BudgetCeilings::default()
                },
            }],
        })
        .expect("policy");
        let policy_root = state_parent.0.join("policy");
        PolicyStore::apply(&policy_root, policy, None, None).expect("policy apply");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 30, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) = LocalEngine::open_with_budget_policy_store(
            config,
            &request(1, "workspace_open"),
            &source.0,
            &policy_root,
        )
        .expect("open");
        let status = engine
            .build_snapshot(&request(2, "snapshot_build"), budget())
            .expect("snapshot");
        assert_eq!(status.eligible_files, "1");
        assert_eq!(status.completeness, "partial");
        let denied_policy = compile_policy(LocalBudgetPolicyDraft {
            schema_name: "local-budget-policy".into(),
            schema_version: "1.0.0".into(),
            revision: "2".into(),
            created_at: "2026-08-21T00:00:00Z".into(),
            expires_at: None,
            rules: vec![LocalBudgetRule {
                rule_id: "deny_search".into(),
                selector: BudgetSelector {
                    purpose: None,
                    capability: Some(Capability::CodeSearch),
                },
                deny: true,
                ceilings: BudgetCeilings::default(),
            }],
        })
        .expect("denied policy");
        let current = PolicyStore::open(&policy_root)
            .expect("store")
            .current()
            .expect("current")
            .expect("installed");
        PolicyStore::apply(
            &policy_root,
            denied_policy,
            Some(&current.policy_id),
            Some(&current.revision),
        )
        .expect("live policy update");
        let denied = engine
            .search(
                &request(3, "implementation"),
                QueryKind::Filename,
                "one.rs",
                &budget(),
            )
            .expect_err("search denied after live update");
        assert_eq!(denied.envelope().code, PublicErrorCode::PolicyDenied);
        drop(engine);

        let audit = context_store::AuditReader::open(&cache.0).expect("audit reader");
        let events = audit.recent(10).expect("events").events;
        let event = events
            .iter()
            .find(|event| event.capability == Capability::SnapshotBuild)
            .expect("snapshot event");
        assert_eq!(event.outcome, AuditOutcome::Limited);
        assert_eq!(event.limits.max_files, "1");
        let denied_event = events
            .iter()
            .find(|event| event.capability == Capability::CodeSearch)
            .expect("denied event");
        assert_eq!(denied_event.outcome, AuditOutcome::Denied);
    }

    #[test]
    fn structural_language_recognizes_python() {
        assert_eq!(
            structural_language("src/service.py"),
            Some(StructuralLanguage::Python)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Python),
            "tree-sitter-python-0.25.0"
        );
    }

    #[test]
    fn structural_language_recognizes_java() {
        assert_eq!(
            structural_language("src/main/java/example/Service.java"),
            Some(StructuralLanguage::Java)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Java),
            "tree-sitter-java-0.23.5"
        );
    }

    #[test]
    fn structural_language_recognizes_kotlin() {
        assert_eq!(
            structural_language("src/main/kotlin/example/Service.kt"),
            Some(StructuralLanguage::Kotlin)
        );
        assert_eq!(
            structural_language("build.gradle.kts"),
            Some(StructuralLanguage::Kotlin)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Kotlin),
            "tree-sitter-kotlin-ng-1.1.0"
        );
    }

    #[test]
    fn structural_language_recognizes_csharp() {
        assert_eq!(
            structural_language("src/Service.cs"),
            Some(StructuralLanguage::CSharp)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::CSharp),
            "tree-sitter-c-sharp-0.23.5"
        );
    }

    #[test]
    fn structural_language_recognizes_c_sources_and_headers() {
        for path in ["src/service.c", "include/service.h"] {
            assert_eq!(structural_language(path), Some(StructuralLanguage::C));
        }
        assert_eq!(
            grammar_version(StructuralLanguage::C),
            "tree-sitter-c-0.24.2"
        );
    }

    #[test]
    fn structural_language_recognizes_unambiguous_cpp_sources_and_headers() {
        for path in [
            "src/service.cc",
            "src/service.cpp",
            "src/service.cxx",
            "include/service.hh",
            "include/service.hpp",
            "include/service.hxx",
        ] {
            assert_eq!(structural_language(path), Some(StructuralLanguage::Cpp));
        }
        assert_eq!(
            grammar_version(StructuralLanguage::Cpp),
            "tree-sitter-cpp-0.23.4"
        );
        assert_eq!(
            structural_language("include/service.h"),
            Some(StructuralLanguage::C),
            "ambiguous .h remains deterministically owned by the C admission"
        );
    }

    #[test]
    fn structural_language_recognizes_ruby() {
        assert_eq!(
            structural_language("lib/service.rb"),
            Some(StructuralLanguage::Ruby)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Ruby),
            "tree-sitter-ruby-0.23.1"
        );
    }

    #[test]
    fn structural_language_recognizes_php() {
        assert_eq!(
            structural_language("src/Service.php"),
            Some(StructuralLanguage::Php)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Php),
            "tree-sitter-php-0.24.2"
        );
    }

    #[test]
    fn structural_language_recognizes_swift() {
        assert_eq!(
            structural_language("Sources/App/Service.swift"),
            Some(StructuralLanguage::Swift)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Swift),
            "tree-sitter-swift-0.7.3"
        );
    }

    #[test]
    fn structural_language_recognizes_functional_languages() {
        for (path, language, grammar) in [
            (
                "src/Service.scala",
                StructuralLanguage::Scala,
                "tree-sitter-scala-0.26.2",
            ),
            (
                "lib/service.ex",
                StructuralLanguage::Elixir,
                "tree-sitter-elixir-0.3.5",
            ),
            (
                "test/service.exs",
                StructuralLanguage::Elixir,
                "tree-sitter-elixir-0.3.5",
            ),
            (
                "src/service.clj",
                StructuralLanguage::Clojure,
                "tree-sitter-clojure-orchard-0.2.8",
            ),
            (
                "src/service.cljs",
                StructuralLanguage::Clojure,
                "tree-sitter-clojure-orchard-0.2.8",
            ),
            (
                "src/service.cljc",
                StructuralLanguage::Clojure,
                "tree-sitter-clojure-orchard-0.2.8",
            ),
            (
                "src/Service.hs",
                StructuralLanguage::Haskell,
                "tree-sitter-haskell-0.23.1",
            ),
            (
                "src/Service.lhs",
                StructuralLanguage::Haskell,
                "tree-sitter-haskell-0.23.1",
            ),
        ] {
            assert_eq!(structural_language(path), Some(language));
            assert_eq!(grammar_version(language), grammar);
        }
        assert_eq!(structural_language("data/example.edn"), None);
    }

    #[test]
    fn structural_language_separates_named_strict_json_and_jsonc_configurations() {
        assert_eq!(
            structural_language("packages/web/package.json"),
            Some(StructuralLanguage::Json)
        );
        assert_eq!(
            structural_language("data/catalog.json"),
            None,
            "arbitrary JSON data is not a configuration-evidence claim"
        );
        assert_eq!(
            structural_language("tsconfig.json"),
            Some(StructuralLanguage::Jsonc)
        );
        assert_eq!(
            structural_language(".vscode/settings.json"),
            Some(StructuralLanguage::Jsonc)
        );
        assert_eq!(
            structural_language("config/tooling.jsonc"),
            Some(StructuralLanguage::Jsonc)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Json),
            "tree-sitter-json-0.24.8"
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Jsonc),
            "tree-sitter-json-0.24.8"
        );
    }

    #[test]
    fn structural_language_recognizes_toml_configuration() {
        assert_eq!(
            structural_language("crates/context-core/Cargo.toml"),
            Some(StructuralLanguage::Toml)
        );
        assert_eq!(
            structural_language(".cargo/config.toml"),
            Some(StructuralLanguage::Toml)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Toml),
            "tree-sitter-toml-ng-0.7.0"
        );
    }

    #[test]
    fn structural_language_recognizes_yaml_configuration() {
        assert_eq!(
            structural_language("deploy/service.yaml"),
            Some(StructuralLanguage::Yaml)
        );
        assert_eq!(
            structural_language(".github/workflows/ci.yml"),
            Some(StructuralLanguage::Yaml)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Yaml),
            "tree-sitter-yaml-0.7.2"
        );
    }

    #[test]
    fn structural_language_recognizes_go() {
        assert_eq!(
            structural_language("cmd/server/main.go"),
            Some(StructuralLanguage::Go)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Go),
            "tree-sitter-go-0.25.0"
        );
    }

    #[test]
    fn structural_language_recognizes_rust() {
        assert_eq!(
            structural_language("crates/core/src/lib.rs"),
            Some(StructuralLanguage::Rust)
        );
        assert_eq!(
            grammar_version(StructuralLanguage::Rust),
            "tree-sitter-rust-0.24.2"
        );
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
    fn planned_context_deduplicates_evidence_and_reports_empty_steps() {
        let source = TestRoot::new("planned-source");
        let cache = TestRoot::new("planned-cache");
        fs::write(source.0.join("sample.rs"), b"pub fn alpha() {}\n").expect("source");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 1024, 1024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("snapshot");
        let plan = ContextPlan {
            steps: vec![
                ContextPlanStep {
                    kind: QueryKind::Literal,
                    query: "alpha".into(),
                },
                ContextPlanStep {
                    kind: QueryKind::Lexical,
                    query: "alpha".into(),
                },
                ContextPlanStep {
                    kind: QueryKind::Literal,
                    query: "missing_symbol".into(),
                },
            ],
        };
        let packet = engine
            .build_planned_context(&request(3, "planned_review"), &plan, budget())
            .expect("planned packet");
        validate_packet(&packet).expect("valid planned packet");
        assert_eq!(packet.observed_evidence.len(), 1, "deduplicated evidence");
        assert!(packet.unknowns.contains(&"plan_step_2_no_evidence".into()));
    }

    #[test]
    fn profiled_context_is_deterministic_and_reports_unavailable_evidence() {
        let source = TestRoot::new("profiled-source");
        let first_cache = TestRoot::new("profiled-first-cache");
        let second_cache = TestRoot::new("profiled-second-cache");
        fs::write(
            source.0.join("review.rs"),
            b"pub fn reviewed_change() { verify_change(); }\n",
        )
        .expect("source");
        let config_for = |cache_root: PathBuf| EngineConfig {
            cache_root,
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
                .expect("retention"),
        };
        let profile_request = request(3, "change_review");
        let (mut first_engine, _) = LocalEngine::open(
            config_for(first_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("first open");
        first_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("first snapshot");
        let first = first_engine
            .build_profiled_context(
                &profile_request,
                TaskProfile::ChangeReview,
                "reviewed_change",
                budget(),
            )
            .expect("first profiled packet");
        drop(first_engine);

        let (mut second_engine, _) = LocalEngine::open(
            config_for(second_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("second open");
        second_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("second snapshot");
        let second = second_engine
            .build_profiled_context(
                &profile_request,
                TaskProfile::ChangeReview,
                "reviewed_change",
                budget(),
            )
            .expect("second profiled packet");

        assert_eq!(first.plan, second.plan);
        assert_eq!(first.packet.packet_id, second.packet.packet_id);
        assert_schema("deterministic-context-plan.schema.json", &first.plan);
        assert!(first.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::ChangeSet
                && coverage.status == "unavailable"
                && coverage.reason_code == "change_set_evidence_unavailable"
        }));
        assert!(first.plan.omitted_candidates.iter().any(|omission| {
            omission.candidate == "change_set"
                && omission.reason_code == "change_set_evidence_unavailable"
        }));
        assert!(
            first
                .plan
                .steps
                .iter()
                .all(|step| !step.reason_code.is_empty())
        );
        validate_packet(&first.packet).expect("valid profiled packet");
    }

    #[test]
    fn declared_associated_tests_are_verified_and_recovered() {
        let source = TestRoot::new("associated-test-source");
        let cache = TestRoot::new("associated-test-cache");
        fs::write(source.0.join("lib.rs"), b"pub fn subject() {}\n").expect("source");
        fs::write(source.0.join("lib_test.rs"), b"fn subject_test() {}\n").expect("test");
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
                .expect("retention"),
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(1, "open"), &source.0).expect("open");
        let snapshot = engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("snapshot");
        let entry = |name: &str| {
            let found = engine
                .snapshot
                .as_ref()
                .expect("snapshot")
                .artifacts
                .iter()
                .find(|item| item.path.display_path == name)
                .expect("artifact");
            DeclaredChangeEntry {
                path: DeclaredChangePath {
                    platform_family: found.path.platform_family.into(),
                    unit_encoding: found.path.unit_encoding.into(),
                    relative_units_base64url: found.path.relative_units_base64url.clone(),
                },
                content_hash: found.content_hash.clone(),
            }
        };
        let declaration = DeclaredAssociatedTests {
            schema_name: "declared-associated-tests".into(),
            schema_version: CONTRACT_VERSION.into(),
            workspace_snapshot: snapshot.snapshot_id,
            associations: vec![DeclaredAssociatedTest {
                source: entry("lib.rs"),
                test: entry("lib_test.rs"),
            }],
        };
        let packet = engine
            .build_profiled_declared_associated_test_context(
                &request(4, "test_selection"),
                "subject",
                &declaration,
                budget(),
            )
            .expect("packet");
        assert!(packet.plan.declared_associated_tests.is_some());
        assert!(
            packet
                .packet
                .observed_evidence
                .iter()
                .any(|item| item.artifact.path.display_path == "lib.rs")
        );
        assert!(
            packet
                .packet
                .observed_evidence
                .iter()
                .any(|item| item.artifact.path.display_path == "lib_test.rs")
        );
        let mut self_pair = declaration;
        self_pair.associations[0].test = self_pair.associations[0].source.clone();
        assert_eq!(
            engine
                .build_profiled_declared_associated_test_context(
                    &request(5, "test_selection"),
                    "subject",
                    &self_pair,
                    budget()
                )
                .expect_err("self pair")
                .envelope()
                .code,
            PublicErrorCode::InvalidInput
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn declared_change_set_context_is_snapshot_verified_and_deterministic() {
        let source = TestRoot::new("declared-change-set-source");
        let first_cache = TestRoot::new("declared-change-set-first-cache");
        let second_cache = TestRoot::new("declared-change-set-second-cache");
        fs::write(source.0.join("review.rs"), b"pub fn reviewed_change() {}\n").expect("source");
        fs::write(source.0.join("other.rs"), b"pub fn other() {}\n").expect("source");
        let config_for = |cache_root: PathBuf| EngineConfig {
            cache_root,
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
                .expect("retention"),
        };
        let declaration_for = |engine: &LocalEngine| {
            let snapshot = engine.snapshot.as_ref().expect("snapshot");
            let artifact = snapshot
                .artifacts
                .iter()
                .find(|artifact| artifact.path.display_path == "review.rs")
                .expect("review artifact");
            DeclaredChangeSet {
                schema_name: "declared-change-set".into(),
                schema_version: CONTRACT_VERSION.into(),
                workspace_snapshot: snapshot.snapshot_id.clone(),
                asserted_base_revision: Some("unavailable-revision".into()),
                entries: vec![DeclaredChangeEntry {
                    path: DeclaredChangePath {
                        platform_family: artifact.path.platform_family.into(),
                        unit_encoding: artifact.path.unit_encoding.into(),
                        relative_units_base64url: artifact.path.relative_units_base64url.clone(),
                    },
                    content_hash: artifact.content_hash.clone(),
                }],
            }
        };
        let profile_request = request(3, "declared_change_review");
        let (mut first_engine, _) = LocalEngine::open(
            config_for(first_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("first open");
        first_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("first snapshot");
        let declaration = declaration_for(&first_engine);
        let first = first_engine
            .build_profiled_declared_change_set_context(
                &profile_request,
                "reviewed_change",
                &declaration,
                budget(),
            )
            .expect("first declared packet");
        assert_schema("deterministic-context-plan.schema.json", &first.plan);
        assert!(first.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::ChangeSet
                && coverage.status == "available"
                && coverage.reason_code == "declared_change_set_current_snapshot_verified"
        }));
        let verified = first
            .plan
            .declared_change_set
            .as_ref()
            .expect("verified declaration");
        assert_eq!(verified.entries, declaration.entries);
        assert_eq!(verified.base_revision_status, "unavailable_or_mismatched");
        assert!(
            first
                .packet
                .unknowns
                .contains(&"asserted_base_revision_unavailable_or_mismatched".into())
        );
        assert!(first.packet.observed_evidence.iter().any(|evidence| {
            evidence.artifact.path.display_path == "review.rs"
                && evidence.artifact.content_hash == declaration.entries[0].content_hash
        }));
        validate_packet(&first.packet).expect("valid declared packet");
        drop(first_engine);

        let (mut second_engine, _) = LocalEngine::open(
            config_for(second_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("second open");
        second_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("second snapshot");
        let second_declaration = declaration_for(&second_engine);
        let second = second_engine
            .build_profiled_declared_change_set_context(
                &profile_request,
                "reviewed_change",
                &second_declaration,
                budget(),
            )
            .expect("second declared packet");
        assert_eq!(first.plan, second.plan);
        assert_eq!(first.packet.packet_id, second.packet.packet_id);

        let mut mismatched = second_declaration.clone();
        mismatched.entries[0].content_hash = format!("sha256:{}", "0".repeat(64));
        let error = second_engine
            .build_profiled_declared_change_set_context(
                &request(4, "mismatched_change_review"),
                "reviewed_change",
                &mismatched,
                budget(),
            )
            .expect_err("mismatched hash must fail closed");
        assert_eq!(error.envelope().code, PublicErrorCode::StaleState);

        let mut duplicate = second_declaration;
        duplicate.entries.push(duplicate.entries[0].clone());
        let error = second_engine
            .build_profiled_declared_change_set_context(
                &request(5, "duplicate_change_review"),
                "reviewed_change",
                &duplicate,
                budget(),
            )
            .expect_err("duplicate entry must fail closed");
        assert_eq!(error.envelope().code, PublicErrorCode::InvalidInput);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn profiled_structural_context_recovers_exact_bounded_graph_evidence() {
        let source = TestRoot::new("structural-profile-source");
        let first_cache = TestRoot::new("structural-profile-first-cache");
        let second_cache = TestRoot::new("structural-profile-second-cache");
        fs::write(
            source.0.join("review.ts"),
            b"export function reviewed_change() { return 1; }\n",
        )
        .expect("source");
        let config_for = |cache_root: PathBuf| EngineConfig {
            cache_root,
            discovery: DiscoveryPolicy::new(10, 1_024, 1_024, 8).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 30, 1_048_576)
                .expect("retention"),
        };
        let graph_for = |engine: &LocalEngine| {
            let snapshot = engine.snapshot.as_ref().expect("snapshot");
            let artifact = snapshot.artifacts.first().expect("artifact");
            let path = WorkerPath {
                display_path: artifact.path.display_path.clone(),
                platform_family: artifact.path.platform_family.into(),
                unit_encoding: artifact.path.unit_encoding.into(),
                relative_units_base64url: artifact.path.relative_units_base64url.clone(),
            };
            let provenance = context_structural::FactProvenance {
                method: "tree_sitter".into(),
                parser_version: "tree-sitter-0.26.13".into(),
                grammar_version: "tree-sitter-typescript-0.23.2".into(),
                resolver_version: RESOLVER_VERSION.into(),
                graph_version: GRAPH_VERSION.into(),
            };
            context_structural::build_graph(
                &snapshot.snapshot_id,
                vec![GraphFileInput {
                    path,
                    response: context_structural::WorkerSuccess {
                        schema_name: "structural-worker-success".into(),
                        schema_version: PROTOCOL_VERSION.into(),
                        request_id: "req_00000002".into(),
                        content_hash: artifact.content_hash.clone(),
                        syntax_errors: false,
                        facts: vec![context_structural::StructuralFact {
                            class: FactClass::Declaration,
                            local_key: "declared_reviewed_change".into(),
                            syntax_kind: "function_declaration".into(),
                            name: Some("reviewed_change".into()),
                            module: None,
                            start_byte: 0,
                            end_byte: 33,
                            parent_key: None,
                            confidence: "confirmed".into(),
                            provenance,
                        }],
                        warnings: Vec::new(),
                    },
                }],
            )
            .expect("graph")
        };
        let profile_request = request(3, "security_review");
        let (mut first_engine, _) = LocalEngine::open(
            config_for(first_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("first open");
        first_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("first snapshot");
        let first_graph = graph_for(&first_engine);
        let first_start = first_graph
            .nodes
            .iter()
            .find(|node| node.kind == "file")
            .expect("file node")
            .node_id
            .clone();
        let first = first_engine
            .build_profiled_structural_context(
                &profile_request,
                TaskProfile::SecurityReview,
                "reviewed_change",
                &StructuralImpactRequest {
                    graph: first_graph.clone(),
                    start_node: first_start.clone(),
                    edge_kinds: vec!["declares".into()],
                },
                budget(),
            )
            .expect("first structural profile");
        assert_schema("deterministic-context-plan.schema.json", &first.plan);
        assert!(first.plan.coverage.iter().any(|coverage| {
            coverage.evidence_class == PlannerEvidenceClass::StructuralRelationship
                && coverage.status == "available"
                && coverage.reason_code == "validated_structural_relationship_available"
        }));
        let structural = first
            .plan
            .structural_query
            .as_ref()
            .expect("structural query");
        assert_eq!(structural.result.graph_id, first_graph.graph_id);
        assert_eq!(structural.result.start_node, first_start);
        assert_eq!(structural.edge_kinds, vec!["declares"]);
        assert_eq!(structural.result.edges.len(), 1);
        assert!(
            first
                .packet
                .observed_evidence
                .iter()
                .any(|evidence| evidence.extraction.method == "structural_graph_edge")
        );
        validate_packet(&first.packet).expect("valid structural packet");
        drop(first_engine);

        let (mut second_engine, _) = LocalEngine::open(
            config_for(second_cache.0.clone()),
            &request(1, "open"),
            &source.0,
        )
        .expect("second open");
        second_engine
            .build_snapshot(&request(2, "snapshot"), budget())
            .expect("second snapshot");
        let second_graph = graph_for(&second_engine);
        let second_start = second_graph
            .nodes
            .iter()
            .find(|node| node.kind == "file")
            .expect("file node")
            .node_id
            .clone();
        let second = second_engine
            .build_profiled_structural_context(
                &profile_request,
                TaskProfile::SecurityReview,
                "reviewed_change",
                &StructuralImpactRequest {
                    graph: second_graph,
                    start_node: second_start,
                    edge_kinds: vec!["declares".into()],
                },
                budget(),
            )
            .expect("second structural profile");
        assert_eq!(first.plan, second.plan);
        assert_eq!(first.packet.packet_id, second.packet.packet_id);
        drop(second_engine);

        fs::write(
            source.0.join("review.ts"),
            b"export function reviewed_change() { return 2; }\n",
        )
        .expect("changed source");
        let stale_cache = TestRoot::new("structural-profile-stale-cache");
        let (mut stale_engine, _) = LocalEngine::open(
            config_for(stale_cache.0.clone()),
            &request(4, "open"),
            &source.0,
        )
        .expect("stale open");
        stale_engine
            .build_snapshot(&request(5, "snapshot"), budget())
            .expect("changed snapshot");
        let stale = stale_engine
            .build_profiled_structural_context(
                &request(6, "security_review"),
                TaskProfile::SecurityReview,
                "reviewed_change",
                &StructuralImpactRequest {
                    graph: first_graph,
                    start_node: first_start,
                    edge_kinds: vec!["declares".into()],
                },
                budget(),
            )
            .expect_err("stale graph must fail before evidence recovery");
        assert_eq!(stale.envelope().code, PublicErrorCode::StaleState);
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
