//! Safe, reproducible A/B/A evaluation support for agent-context studies.

#![forbid(unsafe_code)]

use crate::model_context::MAX_RENDERED_CONTEXT_BYTES;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::ContextPacket;
use context_core::ResourceBudget;
use context_engine::ContextPlanStep;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::{Read, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SCHEMA_VERSION: &str = "1.2";
const MAX_ADAPTER_STREAM_BYTES: usize = 1024 * 1024;
const MAX_PROGRESS_EVENTS: usize = 256;
const PROGRESS_PREFIX: &str = "IMPRESARI_EVAL_PROGRESS ";

/// A study definition loaded from a JSON file.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StudySpec {
    /// The harness schema version.
    pub schema_version: String,
    /// Stable identifier for this study.
    pub study_id: String,
    /// Repository directory, relative to the specification file.
    pub repository: String,
    /// Exact repository-relative regular files included in the frozen source.
    pub source_files: Vec<String>,
    /// Stable label for the fixture or source revision.
    pub workspace_revision: String,
    /// Fixed execution conditions shared by every arm.
    pub execution: ExecutionSpec,
    /// Number of repetitions for every task and arm.
    pub repetitions: u32,
    /// Fixed argv command used for baseline and treatment agent calls.
    pub agent_command: Vec<String>,
    /// Fixed argv command used to prepare a treatment packet.
    pub packet_command: Vec<String>,
    /// Explicit, allow-listed variables passed to adapters.
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    /// Secret variable names inherited only by the production agent adapter.
    /// Values are read from the harness process and are never serialized.
    #[serde(default)]
    pub agent_secret_environment_variables: Vec<String>,
    /// Per-adapter time limit.
    pub command_timeout_seconds: u64,
    /// Per-adapter standard-output byte limit.
    pub max_stdout_bytes: usize,
    /// Per-adapter standard-error byte limit.
    pub max_stderr_bytes: usize,
    /// Tasks included in the study.
    pub tasks: Vec<TaskSpec>,
    #[serde(skip)]
    base_directory: PathBuf,
}

/// Execution conditions that must remain equal across arms.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSpec {
    /// Stable identifier for the agent adapter.
    pub agent_adapter_identifier: String,
    /// Version label for the agent adapter.
    pub agent_adapter_version: String,
    /// Stable identifier for the packet adapter.
    pub packet_adapter_identifier: String,
    /// Version label for the packet adapter.
    pub packet_adapter_version: String,
    /// Stable identifier for the model-context renderer.
    pub model_context_renderer_identifier: String,
    /// Version label for the model-context renderer.
    pub model_context_renderer_version: String,
    /// Hard byte ceiling for provider-bound rendered context.
    pub max_rendered_context_bytes: usize,
    /// Model identifier recorded for comparison.
    pub model_identifier: String,
    /// Container or runtime image identifier recorded for comparison.
    pub container_image: String,
    /// Frozen normalized UTC timestamp used by deterministic packet operations.
    #[serde(default = "default_operation_timestamp")]
    pub operation_timestamp: String,
    /// Maximum agent turns permitted for each run.
    pub turn_limit: u32,
    /// Frozen provider reasoning-effort control.
    pub provider_effort: String,
    /// Frozen provider output-token ceiling per request.
    pub provider_max_output_tokens: u64,
    /// Maximum duration of one provider request.
    pub provider_request_timeout_seconds: u64,
    /// Complete packet resource policy used by the treatment arm.
    pub packet_resource_policy: PacketResourceSpec,
    /// Optional offline packet policies analyzed before the live primary point.
    #[serde(default)]
    pub packet_budget_curve: Vec<PacketResourceSpec>,
    /// Human-readable basis for adapter-reported cost estimates.
    pub pricing_basis: String,
    /// Frozen machine-readable token pricing used to verify adapter costs.
    #[serde(default)]
    pub pricing_schedule: PricingSchedule,
}

/// Explicit integer representation of every packet resource limit.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(missing_docs)]
pub struct PacketResourceSpec {
    pub requested_bytes: u64,
    pub max_evidence_items: u64,
    pub max_files: u64,
    pub max_excerpt_bytes_per_item: u64,
    pub max_matches: u64,
    pub max_traversal_depth: u64,
    pub max_elapsed_ms: u64,
    pub max_memory_bytes: u64,
}

impl PacketResourceSpec {
    /// Convert the manifest representation to the validated core budget.
    ///
    /// # Errors
    ///
    /// Returns an error when any integer is outside the core policy bounds.
    pub fn to_resource_budget(&self) -> Result<ResourceBudget, String> {
        ResourceBudget::conservative(
            self.requested_bytes,
            self.max_evidence_items,
            self.max_files,
            self.max_excerpt_bytes_per_item,
            self.max_matches,
            self.max_traversal_depth,
            self.max_elapsed_ms,
            self.max_memory_bytes,
        )
        .map_err(|_| "packet resource policy is outside supported bounds".to_owned())
    }
}

/// Frozen provider token pricing in US dollars per million tokens.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PricingSchedule {
    /// ISO currency code. Version 1 requires `USD`.
    pub currency: String,
    /// Date or stable fixture label on which prices were frozen.
    pub effective_date: String,
    /// Uncached input-token price.
    pub input_usd_per_million: f64,
    /// Cache-read input-token price.
    pub cached_input_usd_per_million: f64,
    /// Cache-write input-token price.
    pub cache_write_input_usd_per_million: f64,
    /// Output-token price, including provider-reported reasoning tokens.
    pub output_usd_per_million: f64,
}

impl Default for PricingSchedule {
    fn default() -> Self {
        Self {
            currency: "USD".into(),
            effective_date: "offline-fixture".into(),
            input_usd_per_million: 0.0,
            cached_input_usd_per_million: 0.0,
            cache_write_input_usd_per_million: 0.0,
            output_usd_per_million: 0.0,
        }
    }
}

/// A task with ground truth that can be deterministically scored.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSpec {
    /// Stable identifier for the task.
    pub id: String,
    /// Frozen source-free explanation of why the task has one interpretation.
    pub uniqueness_rationale: String,
    /// Prompt delivered to the adapter.
    pub prompt: String,
    /// Case-insensitive fragments required in the answer.
    pub expected_answer_fragments: Vec<String>,
    /// Source citations the adapter must return.
    pub required_evidence: Vec<EvidenceRequirement>,
    /// Frozen retrieval plan used only to build the treatment packet.
    #[serde(default)]
    pub context_plan: Vec<ContextPlanStep>,
}

/// A source range required as evidence for a task.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRequirement {
    /// Repository-relative source path.
    pub path: String,
    /// Inclusive first line.
    pub line_start: u32,
    /// Inclusive final line.
    pub line_end: u32,
}

/// Usage reported by an adapter.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    /// Prompt or input token count.
    pub input_tokens: u64,
    /// Completion or output token count.
    pub output_tokens: u64,
    /// Sum of input and output tokens.
    pub total_tokens: u64,
    /// Estimated cost in US dollars.
    pub estimated_cost_usd: f64,
    /// Input tokens served from a provider prompt cache.
    #[serde(default)]
    pub cached_input_tokens: u64,
    /// Input tokens written to a provider prompt cache.
    #[serde(default)]
    pub cache_write_input_tokens: u64,
    /// Reasoning tokens included within `output_tokens`.
    #[serde(default)]
    pub reasoning_tokens: u64,
    /// Provider API requests made during this arm.
    #[serde(default)]
    pub provider_requests: u64,
    /// Adapter-reported tool calls.
    pub tool_calls: u64,
    /// Adapter-reported repository file reads.
    pub repository_file_reads: u64,
    /// Adapter-reported repeated repository file reads.
    pub repeated_repository_file_reads: u64,
}

/// The ordered A/B/A study arms.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    /// First cold baseline.
    BaselineA,
    /// Impresari Context packet treatment.
    Treatment,
    /// Second baseline to detect ordering drift.
    BaselineB,
}

/// A citation returned by an agent adapter.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceCitation {
    /// Repository-relative source path.
    pub path: String,
    /// Inclusive first line.
    pub line_start: u32,
    /// Inclusive final line.
    pub line_end: u32,
    /// SHA-256 of the cited source bytes.
    pub sha256: String,
}

/// One persistable result. Raw prompts, answers, and packets are deliberately
/// excluded from this format.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    /// The persisted record schema version.
    pub schema_version: String,
    /// Study identifier.
    pub study_id: String,
    /// Monotonic sequence number within the study.
    pub sequence: u64,
    /// One-based repetition number.
    pub repetition: u32,
    /// Evaluation arm.
    pub arm: Arm,
    /// Task identifier.
    pub task_id: String,
    /// Exact source fingerprint before and after the adapter run.
    pub source_fingerprint_sha256: String,
    /// Source revision label from the study.
    pub workspace_revision: String,
    /// Fixed model identifier.
    pub model_identifier: String,
    /// Stable agent-adapter identifier.
    pub agent_adapter_identifier: String,
    /// Agent-adapter version label.
    pub agent_adapter_version: String,
    /// Stable packet-adapter identifier.
    pub packet_adapter_identifier: String,
    /// Packet-adapter version label.
    pub packet_adapter_version: String,
    /// Stable model-context renderer identifier.
    pub model_context_renderer_identifier: String,
    /// Model-context renderer version label.
    pub model_context_renderer_version: String,
    /// Frozen maximum provider-bound rendered-context bytes.
    pub max_rendered_context_bytes: usize,
    /// Declared basis for estimated costs.
    pub pricing_basis: String,
    /// Frozen pricing schedule used for this run.
    #[serde(default)]
    pub pricing_schedule: PricingSchedule,
    /// Fixed runtime image identifier.
    pub container_image: String,
    /// Frozen normalized UTC timestamp used by packet operations.
    #[serde(default = "default_operation_timestamp")]
    pub operation_timestamp: String,
    /// Fixed turn limit.
    pub turn_limit: u32,
    /// Frozen provider reasoning-effort control.
    pub provider_effort: String,
    /// Frozen provider output-token ceiling per request.
    pub provider_max_output_tokens: u64,
    /// Frozen maximum duration of one provider request.
    pub provider_request_timeout_seconds: u64,
    /// Exact treatment packet resource policy.
    pub packet_resource_policy: PacketResourceSpec,
    /// Fixed adapter timeout.
    pub command_timeout_seconds: u64,
    /// Fixed standard-output limit.
    pub max_stdout_bytes: usize,
    /// Fixed standard-error limit.
    pub max_stderr_bytes: usize,
    /// Packet preparation duration for the treatment arm.
    pub packet_generation_millis: u64,
    /// Treatment packet byte size, if any.
    pub packet_bytes: u64,
    /// SHA-256 of the treatment packet, if any.
    pub packet_sha256: Option<String>,
    /// Packet preparation usage, if any.
    pub packet_usage: Option<Usage>,
    /// Exact provider-bound rendered-context bytes, zero for baselines.
    pub rendered_context_bytes: u64,
    /// SHA-256 of exact provider-bound rendered context, absent for baselines.
    pub rendered_context_sha256: Option<String>,
    /// Number of rendered packet evidence items, zero for baselines.
    pub rendered_context_evidence_count: u64,
    /// Agent usage for this arm.
    pub agent_usage: Usage,
    /// Combined packet and agent input tokens.
    pub total_input_tokens: u64,
    /// Combined packet and agent output tokens.
    pub total_output_tokens: u64,
    /// Combined packet and agent tokens.
    pub total_tokens: u64,
    /// Combined estimated cost.
    pub total_estimated_cost_usd: f64,
    /// Combined tool calls.
    pub total_tool_calls: u64,
    /// Combined repository reads.
    pub total_repository_file_reads: u64,
    /// Combined repeated reads.
    pub total_repeated_repository_file_reads: u64,
    /// Agent adapter wall-clock duration.
    pub agent_wall_clock_millis: u64,
    /// Packet plus agent wall-clock duration.
    pub total_wall_clock_millis: u64,
    /// Whether the answer met task ground truth.
    pub correctness: bool,
    /// Whether all declared and returned evidence was verified.
    pub evidence_verified: bool,
    /// Number of citations returned.
    pub evidence_count: u64,
    /// Returned source citations.
    pub evidence: Vec<EvidenceCitation>,
    /// Completed, source-free provider progress events.
    #[serde(default)]
    pub progress: Vec<AdapterProgressEvent>,
}

/// One bounded, source-free provider progress event emitted on stderr.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(missing_docs)]
pub struct AdapterProgressEvent {
    pub schema_name: String,
    pub schema_version: String,
    pub provider: String,
    pub arm: Arm,
    pub stage: ProgressStage,
    pub turn: u32,
    pub elapsed_millis: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered_context_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered_context_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id_sha256: Option<String>,
}

/// Closed set of observable adapter stages.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(missing_docs)]
pub enum ProgressStage {
    Rendered,
    TokenCounted,
    ProviderRequestStarted,
    ProviderResponseCompleted,
    ToolsDispatched,
    Completed,
}

/// Source-free record written for an unsuccessful arm.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(missing_docs)]
pub struct RunFailureRecord {
    pub schema_name: String,
    pub schema_version: String,
    pub study_id: String,
    pub sequence: u64,
    pub repetition: u32,
    pub arm: Arm,
    pub task_id: String,
    pub model_identifier: String,
    pub provider_effort: String,
    pub provider_max_output_tokens: u64,
    pub provider_request_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
    pub failure_stage: String,
    pub reason_code: String,
    pub elapsed_millis: u64,
    pub progress: Vec<AdapterProgressEvent>,
}

/// Source-free packet-budget analysis for one task and policy point.
#[derive(Clone, Debug, Serialize)]
#[allow(missing_docs)]
pub struct BudgetCurvePoint {
    pub task_id: String,
    pub policy: PacketResourceSpec,
    pub packet_bytes: u64,
    pub rendered_bytes: u64,
    pub evidence_items: u64,
    pub unique_source_bytes: u64,
    pub overlapping_source_bytes: u64,
    pub overlap_fraction: f64,
    pub expected_range_coverage: bool,
    pub evidence_precision_proxy: f64,
    pub source_density: f64,
    pub first_covering_rank: Option<u64>,
}

/// Source-free exact initial-request token count returned by a provider.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenPreflightResponse {
    /// Provider-reported exact input-token count.
    pub input_tokens: u64,
    /// Fingerprint independently observed by the adapter.
    pub source_fingerprint_sha256: String,
    /// Treatment rendering metadata, absent for baselines.
    pub rendered_context: Option<RenderedContextMetadata>,
}

/// Persistable source-free preflight measurement.
#[derive(Clone, Debug, Serialize)]
#[allow(missing_docs)]
pub struct TokenPreflightRecord {
    pub schema_name: String,
    pub schema_version: String,
    pub study_id: String,
    pub task_id: String,
    pub arm: Arm,
    pub model_identifier: String,
    pub input_tokens: u64,
    pub rendered_context_bytes: u64,
    pub rendered_context_sha256: Option<String>,
}

/// Aggregate outcomes for all arms.
#[derive(Clone, Debug, Serialize)]
pub struct EvaluationSummary {
    /// Study identifier.
    pub study_id: String,
    /// Number of persisted runs.
    pub run_count: usize,
    /// Per-arm aggregates.
    pub arms: Vec<ArmSummary>,
    /// Treatment comparison against the mean of the two baseline arms.
    pub treatment_vs_baseline: Comparison,
    /// Baseline B minus Baseline A, exposing ordering or service drift.
    pub baseline_drift: BaselineDrift,
}

/// Aggregate values for one arm.
#[derive(Clone, Debug, Serialize)]
pub struct ArmSummary {
    /// Arm name.
    pub arm: Arm,
    /// Number of runs.
    pub runs: usize,
    /// Share of runs scored correct.
    pub correctness_rate: f64,
    /// Share of runs with verified evidence.
    pub evidence_verification_rate: f64,
    /// Mean combined token count.
    pub mean_total_tokens: f64,
    /// Mean combined estimated cost.
    pub mean_estimated_cost_usd: f64,
    /// Mean combined tool calls.
    pub mean_tool_calls: f64,
    /// Mean combined repository reads.
    pub mean_repository_file_reads: f64,
    /// Mean combined repeated repository reads.
    pub mean_repeated_repository_file_reads: f64,
    /// Mean combined wall-clock time.
    pub mean_wall_clock_millis: f64,
}

/// Difference between treatment and mean baseline outcomes.
#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    /// Positive values mean treatment used fewer tokens.
    pub token_reduction_fraction: f64,
    /// Positive values mean treatment cost less.
    pub cost_reduction_fraction: f64,
    /// Positive values mean treatment used fewer tool calls.
    pub tool_call_reduction_fraction: f64,
    /// Positive values mean treatment used fewer repository reads.
    pub repository_read_reduction_fraction: f64,
    /// Positive values mean treatment repeated fewer repository reads.
    pub repeated_read_reduction_fraction: f64,
    /// Positive values mean treatment was faster.
    pub wall_clock_reduction_fraction: f64,
    /// Treatment correctness rate minus mean baseline correctness rate.
    pub correctness_rate_delta: f64,
    /// Treatment evidence-verification rate minus mean baseline rate.
    pub evidence_verification_rate_delta: f64,
}

/// Baseline B minus Baseline A for every headline outcome.
#[derive(Clone, Debug, Serialize)]
pub struct BaselineDrift {
    /// Change in mean total tokens.
    pub total_tokens_delta: f64,
    /// Change in mean estimated cost.
    pub estimated_cost_usd_delta: f64,
    /// Change in mean tool calls.
    pub tool_calls_delta: f64,
    /// Change in mean repository reads.
    pub repository_file_reads_delta: f64,
    /// Change in mean repeated repository reads.
    pub repeated_repository_file_reads_delta: f64,
    /// Change in mean wall-clock milliseconds.
    pub wall_clock_millis_delta: f64,
    /// Change in correctness rate.
    pub correctness_rate_delta: f64,
    /// Change in evidence-verification rate.
    pub evidence_verification_rate_delta: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// One bounded request sent by the harness to a packet or agent adapter.
pub struct AdapterRequest {
    /// Stable task identifier.
    pub task_id: String,
    /// Frozen task prompt.
    pub prompt: String,
    /// Current A/B/A arm.
    pub arm: Arm,
    /// Canonical evaluated repository root.
    pub workspace_root: String,
    /// Exact allow-listed repository-relative source files.
    pub source_files: Vec<String>,
    /// Frozen packet retrieval plan.
    pub context_plan: Vec<ContextPlanStep>,
    /// Fingerprint computed by the harness before adapter execution.
    pub source_fingerprint_sha256: String,
    /// Fixed model identifier.
    pub model_identifier: String,
    /// Stable model-context renderer identifier.
    pub model_context_renderer_identifier: String,
    /// Model-context renderer version label.
    pub model_context_renderer_version: String,
    /// Hard provider-bound rendered-context byte ceiling.
    pub max_rendered_context_bytes: usize,
    /// Frozen pricing schedule used to compute provider cost.
    pub pricing_schedule: PricingSchedule,
    /// Fixed runtime image identifier.
    pub container_image: String,
    /// Frozen normalized UTC timestamp used by packet operations.
    pub operation_timestamp: String,
    /// Hard agent turn limit.
    pub turn_limit: u32,
    /// Frozen provider reasoning-effort control.
    pub provider_effort: String,
    /// Frozen provider output-token ceiling per request.
    pub provider_max_output_tokens: u64,
    /// Maximum duration of one provider request.
    pub provider_request_timeout_seconds: u64,
    /// Outer arm deadline used for remaining-time calculations.
    pub command_timeout_seconds: u64,
    /// Complete explicit treatment packet resource policy.
    pub packet_resource_policy: PacketResourceSpec,
    /// Treatment packet; always absent in both baselines.
    pub packet: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Successful treatment-packet adapter response.
pub struct PacketResponse {
    /// Bounded serialized context packet.
    pub packet: String,
    /// Packet-build usage.
    pub usage: Usage,
    /// Fingerprint independently observed by the adapter.
    pub source_fingerprint_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
/// Successful production-agent adapter response.
pub struct AgentResponse {
    /// Agent answer retained only in process memory for scoring.
    pub answer: String,
    /// Provider and tool-boundary usage.
    pub usage: Usage,
    /// Fingerprint independently observed by the adapter.
    pub source_fingerprint_sha256: String,
    /// Adapter-derived verified citations.
    pub evidence: Vec<EvidenceCitation>,
    /// Source-free treatment rendering measurements; absent for baselines.
    pub rendered_context: Option<RenderedContextMetadata>,
}

/// Source-free identity and accounting for provider-bound treatment context.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderedContextMetadata {
    /// Stable model-context renderer identifier.
    pub renderer_identifier: String,
    /// Model-context renderer version label.
    pub renderer_version: String,
    /// Exact rendered UTF-8 byte count.
    pub bytes: u64,
    /// SHA-256 of the exact rendered bytes.
    pub sha256: String,
    /// Number of packet evidence items represented.
    pub evidence_count: u64,
}

/// Load and validate a study specification.
///
/// # Errors
///
/// Returns an error when the file cannot be read or the specification is invalid.
pub fn load_spec(path: &Path) -> Result<StudySpec, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut spec: StudySpec = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    spec.base_directory = path
        .parent()
        .ok_or_else(|| format!("specification has no parent: {}", path.display()))?
        .canonicalize()
        .map_err(|error| format!("canonicalize specification directory: {error}"))?;
    validate_spec(&spec)?;
    Ok(spec)
}

/// Execute the study only when the caller explicitly grants adapter execution.
///
/// # Errors
///
/// Returns an error when consent is absent, the study is invalid, an adapter
/// fails, source integrity changes, or a result cannot be persisted.
pub fn run_study(
    spec: &StudySpec,
    output_directory: &Path,
    explicit_consent: bool,
) -> Result<Vec<RunRecord>, String> {
    if !explicit_consent {
        return Err("adapter execution requires --allow-adapter-execution".to_owned());
    }
    validate_spec(spec)?;
    fs::create_dir_all(output_directory)
        .map_err(|error| format!("create {}: {error}", output_directory.display()))?;
    let output_directory = output_directory
        .canonicalize()
        .map_err(|error| format!("canonicalize output directory: {error}"))?;
    let source_root = source_root(spec)?;
    if output_directory.starts_with(&source_root) {
        return Err(
            "output directory must not be inside the evaluated source repository".to_owned(),
        );
    }

    let mut records = Vec::new();
    let mut sequence = 1_u64;
    for repetition in 1..=spec.repetitions {
        for task in &spec.tasks {
            for arm in [Arm::BaselineA, Arm::Treatment, Arm::BaselineB] {
                let record = match execute_run(spec, &source_root, task, arm, repetition, sequence)
                {
                    Ok(record) => record,
                    Err(failure) => {
                        let record = failure.to_record(spec, task, arm, repetition, sequence);
                        let path = output_directory.join(format!("failure-{sequence:05}.json"));
                        write_json(&path, &record)?;
                        return Err(failure.message);
                    }
                };
                let path = output_directory.join(format!("run-{sequence:05}.json"));
                write_json(&path, &record)?;
                records.push(record);
                sequence = sequence.saturating_add(1);
            }
        }
    }
    validate_records(spec, &records)?;
    Ok(records)
}

/// Execute only the packet adapter across the declared offline budget curve.
///
/// This command never starts an agent adapter or reads provider credentials.
///
/// # Errors
///
/// Returns an error for invalid studies, adapter failures, invalid packets,
/// source changes, or a curve point that loses required evidence.
pub fn analyze_budgets(
    spec: &StudySpec,
    explicit_consent: bool,
) -> Result<Vec<BudgetCurvePoint>, String> {
    if !explicit_consent {
        return Err("packet adapter execution requires --allow-adapter-execution".to_owned());
    }
    validate_spec(spec)?;
    let root = source_root(spec)?;
    let fingerprint = source_fingerprint(spec, &root)?;
    let policies = if spec.execution.packet_budget_curve.is_empty() {
        vec![spec.execution.packet_resource_policy.clone()]
    } else {
        spec.execution.packet_budget_curve.clone()
    };
    let mut points = Vec::new();
    for task in &spec.tasks {
        for policy in &policies {
            let request = adapter_request(
                spec,
                task,
                Arm::Treatment,
                &root,
                &fingerprint,
                policy.clone(),
                None,
            );
            let execution =
                execute_adapter::<PacketResponse, _>(spec, &spec.packet_command, &request, false)
                    .map_err(|failure| failure.message)?;
            if execution.response.source_fingerprint_sha256 != fingerprint {
                return Err("packet adapter returned a mismatched source fingerprint".to_owned());
            }
            let mut render_request = request;
            render_request.packet = Some(execution.response.packet.clone());
            let rendered = crate::model_context::render_model_context(&render_request, &root)?;
            let packet: ContextPacket = serde_json::from_str(&execution.response.packet)
                .map_err(|_| "packet adapter returned invalid packet JSON".to_owned())?;
            points.push(analyze_packet_point(
                &root,
                task,
                policy.clone(),
                &packet,
                rendered.metadata.bytes,
                u64::try_from(execution.response.packet.len()).unwrap_or(u64::MAX),
            )?);
        }
    }
    if points.iter().any(|point| !point.expected_range_coverage) {
        return Err("one or more packet budget points lost required evidence".to_owned());
    }
    Ok(points)
}

/// Count exact initial provider requests without authorizing generation.
///
/// # Errors
///
/// Returns an error for invalid studies, packet/preflight adapter failures,
/// source changes, or invalid provider accounting.
pub fn token_preflight(
    spec: &StudySpec,
    explicit_consent: bool,
) -> Result<Vec<TokenPreflightRecord>, String> {
    if !explicit_consent {
        return Err("token preflight requires --allow-adapter-execution".to_owned());
    }
    validate_spec(spec)?;
    let root = source_root(spec)?;
    let fingerprint = source_fingerprint(spec, &root)?;
    let mut command = spec.agent_command.clone();
    command.push("--count-tokens".into());
    let mut records = Vec::new();
    for task in &spec.tasks {
        let mut treatment_packet = None;
        for arm in [Arm::BaselineA, Arm::Treatment, Arm::BaselineB] {
            if arm == Arm::Treatment && treatment_packet.is_none() {
                let packet_request = adapter_request(
                    spec,
                    task,
                    arm,
                    &root,
                    &fingerprint,
                    spec.execution.packet_resource_policy.clone(),
                    None,
                );
                let packet = execute_adapter::<PacketResponse, _>(
                    spec,
                    &spec.packet_command,
                    &packet_request,
                    false,
                )
                .map_err(|failure| failure.message)?
                .response;
                if packet.source_fingerprint_sha256 != fingerprint {
                    return Err("packet adapter returned a mismatched source fingerprint".into());
                }
                treatment_packet = Some(packet.packet);
            }
            let request = adapter_request(
                spec,
                task,
                arm,
                &root,
                &fingerprint,
                spec.execution.packet_resource_policy.clone(),
                if arm == Arm::Treatment {
                    treatment_packet.clone()
                } else {
                    None
                },
            );
            let response =
                execute_adapter::<TokenPreflightResponse, _>(spec, &command, &request, true)
                    .map_err(|failure| failure.message)?
                    .response;
            if response.source_fingerprint_sha256 != fingerprint || response.input_tokens == 0 {
                return Err("provider token preflight returned invalid accounting".into());
            }
            records.push(TokenPreflightRecord {
                schema_name: "agent-evaluation-token-preflight".into(),
                schema_version: "1.0".into(),
                study_id: spec.study_id.clone(),
                task_id: task.id.clone(),
                arm,
                model_identifier: spec.execution.model_identifier.clone(),
                input_tokens: response.input_tokens,
                rendered_context_bytes: response
                    .rendered_context
                    .as_ref()
                    .map_or(0, |value| value.bytes),
                rendered_context_sha256: response.rendered_context.map(|value| value.sha256),
            });
        }
    }
    Ok(records)
}

fn adapter_request(
    spec: &StudySpec,
    task: &TaskSpec,
    arm: Arm,
    root: &Path,
    fingerprint: &str,
    packet_resource_policy: PacketResourceSpec,
    packet: Option<String>,
) -> AdapterRequest {
    AdapterRequest {
        task_id: task.id.clone(),
        prompt: task.prompt.clone(),
        arm,
        workspace_root: root.display().to_string(),
        source_files: spec.source_files.clone(),
        context_plan: task.context_plan.clone(),
        source_fingerprint_sha256: fingerprint.to_owned(),
        model_identifier: spec.execution.model_identifier.clone(),
        model_context_renderer_identifier: spec.execution.model_context_renderer_identifier.clone(),
        model_context_renderer_version: spec.execution.model_context_renderer_version.clone(),
        max_rendered_context_bytes: spec.execution.max_rendered_context_bytes,
        pricing_schedule: spec.execution.pricing_schedule.clone(),
        container_image: spec.execution.container_image.clone(),
        operation_timestamp: spec.execution.operation_timestamp.clone(),
        turn_limit: spec.execution.turn_limit,
        provider_effort: spec.execution.provider_effort.clone(),
        provider_max_output_tokens: spec.execution.provider_max_output_tokens,
        provider_request_timeout_seconds: spec.execution.provider_request_timeout_seconds,
        command_timeout_seconds: spec.command_timeout_seconds,
        packet_resource_policy,
        packet,
    }
}

fn analyze_packet_point(
    root: &Path,
    task: &TaskSpec,
    policy: PacketResourceSpec,
    packet: &ContextPacket,
    rendered_bytes: u64,
    packet_bytes: u64,
) -> Result<BudgetCurvePoint, String> {
    let mut intervals = Vec::<(String, u64, u64)>::new();
    let mut first_covering_rank = None;
    let mut relevant = 0_u64;
    for (index, evidence) in packet.observed_evidence.iter().enumerate() {
        let match_start = evidence
            .span
            .start_byte
            .parse::<u64>()
            .map_err(|_| "invalid evidence span")?;
        let excerpt_match_start = evidence
            .excerpt
            .match_start_byte
            .parse::<u64>()
            .map_err(|_| "invalid excerpt span")?;
        let start = match_start
            .checked_sub(excerpt_match_start)
            .ok_or("invalid excerpt interval")?;
        let length = u64::try_from(
            URL_SAFE_NO_PAD
                .decode(&evidence.excerpt.bytes_base64url)
                .map_err(|_| "invalid excerpt encoding")?
                .len(),
        )
        .unwrap_or(u64::MAX);
        let end = start
            .checked_add(length)
            .ok_or("excerpt interval overflow")?;
        let path = evidence.artifact.path.display_path.clone();
        let covers = task.required_evidence.iter().any(|requirement| {
            requirement.path == path
                && required_line_interval(root, requirement).is_ok_and(
                    |(required_start, required_end)| start <= required_start && end >= required_end,
                )
        });
        if covers {
            relevant = relevant.saturating_add(1);
            first_covering_rank.get_or_insert(u64::try_from(index + 1).unwrap_or(u64::MAX));
        }
        intervals.push((path, start, end));
    }
    let total_source_bytes = intervals.iter().fold(0_u64, |sum, (_, start, end)| {
        sum.saturating_add(end.saturating_sub(*start))
    });
    let mut unique_source_bytes = 0_u64;
    let mut by_path = BTreeMap::<String, Vec<(u64, u64)>>::new();
    for (path, start, end) in intervals {
        by_path.entry(path).or_default().push((start, end));
    }
    for ranges in by_path.values_mut() {
        ranges.sort_unstable();
        let mut current: Option<(u64, u64)> = None;
        for &(start, end) in ranges.iter() {
            current = match current {
                None => Some((start, end)),
                Some((left, right)) if start <= right => Some((left, right.max(end))),
                Some((left, right)) => {
                    unique_source_bytes = unique_source_bytes.saturating_add(right - left);
                    Some((start, end))
                }
            };
        }
        if let Some((start, end)) = current {
            unique_source_bytes = unique_source_bytes.saturating_add(end - start);
        }
    }
    let overlap = total_source_bytes.saturating_sub(unique_source_bytes);
    let item_count = u64::try_from(packet.observed_evidence.len()).unwrap_or(u64::MAX);
    Ok(BudgetCurvePoint {
        task_id: task.id.clone(),
        policy,
        packet_bytes,
        rendered_bytes,
        evidence_items: item_count,
        unique_source_bytes,
        overlapping_source_bytes: overlap,
        overlap_fraction: ratio(overlap, total_source_bytes),
        expected_range_coverage: first_covering_rank.is_some(),
        evidence_precision_proxy: ratio(relevant, item_count),
        source_density: ratio(unique_source_bytes, rendered_bytes),
        first_covering_rank,
    })
}

fn required_line_interval(
    root: &Path,
    requirement: &EvidenceRequirement,
) -> Result<(u64, u64), String> {
    let bytes = fs::read(resolve_source_file(root, &requirement.path)?)
        .map_err(|error| format!("read required evidence: {error}"))?;
    let mut starts = vec![0_usize];
    for (index, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' && index + 1 < bytes.len() {
            starts.push(index + 1);
        }
    }
    let start_index =
        usize::try_from(requirement.line_start - 1).map_err(|_| "line range overflow")?;
    let end_index = usize::try_from(requirement.line_end).map_err(|_| "line range overflow")?;
    let start = *starts
        .get(start_index)
        .ok_or("required line is outside source")?;
    let end = starts.get(end_index).copied().unwrap_or(bytes.len());
    Ok((
        u64::try_from(start).unwrap_or(u64::MAX),
        u64::try_from(end).unwrap_or(u64::MAX),
    ))
}

#[allow(clippy::cast_precision_loss)]
fn ratio(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// Load all persisted run records in deterministic file-name order.
///
/// # Errors
///
/// Returns an error when the directory cannot be read or a record is invalid JSON.
pub fn load_records(directory: &Path) -> Result<Vec<RunRecord>, String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("read {}: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("run-")
                        && Path::new(name)
                            .extension()
                            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
                })
        })
        .collect::<Vec<_>>();
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no run records found in {}", directory.display()));
    }
    paths
        .iter()
        .map(|path| read_json(path))
        .collect::<Result<Vec<RunRecord>, String>>()
}

/// Revalidate persisted records against the specification and current source.
///
/// # Errors
///
/// Returns an error when record identity, accounting, evidence, ordering, or
/// source freshness does not match the study specification.
#[allow(clippy::too_many_lines)]
pub fn validate_records(spec: &StudySpec, records: &[RunRecord]) -> Result<(), String> {
    validate_spec(spec)?;
    let expected = usize::try_from(spec.repetitions)
        .map_err(|_| "repetition count does not fit usize".to_owned())?
        .saturating_mul(spec.tasks.len())
        .saturating_mul(3);
    if records.len() != expected {
        return Err(format!(
            "expected {expected} records, found {}",
            records.len()
        ));
    }
    let source_root = source_root(spec)?;
    let current_fingerprint = source_fingerprint(spec, &source_root)?;
    for (index, record) in records.iter().enumerate() {
        let expected_arm = match index % 3 {
            0 => Arm::BaselineA,
            1 => Arm::Treatment,
            _ => Arm::BaselineB,
        };
        let expected_task = &spec.tasks[(index / 3) % spec.tasks.len()];
        let expected_repetition = u32::try_from(index / (spec.tasks.len() * 3) + 1)
            .map_err(|_| "record repetition does not fit u32".to_owned())?;
        if record.schema_version != SCHEMA_VERSION
            || record.study_id != spec.study_id
            || record.sequence != u64::try_from(index + 1).unwrap_or(u64::MAX)
            || record.repetition != expected_repetition
            || record.arm != expected_arm
            || record.task_id != expected_task.id
            || record.workspace_revision != spec.workspace_revision
            || record.model_identifier != spec.execution.model_identifier
            || record.agent_adapter_identifier != spec.execution.agent_adapter_identifier
            || record.agent_adapter_version != spec.execution.agent_adapter_version
            || record.packet_adapter_identifier != spec.execution.packet_adapter_identifier
            || record.packet_adapter_version != spec.execution.packet_adapter_version
            || record.model_context_renderer_identifier
                != spec.execution.model_context_renderer_identifier
            || record.model_context_renderer_version
                != spec.execution.model_context_renderer_version
            || record.max_rendered_context_bytes != spec.execution.max_rendered_context_bytes
            || record.pricing_basis != spec.execution.pricing_basis
            || record.pricing_schedule != spec.execution.pricing_schedule
            || record.container_image != spec.execution.container_image
            || record.operation_timestamp != spec.execution.operation_timestamp
            || record.turn_limit != spec.execution.turn_limit
            || record.provider_effort != spec.execution.provider_effort
            || record.provider_max_output_tokens != spec.execution.provider_max_output_tokens
            || record.provider_request_timeout_seconds
                != spec.execution.provider_request_timeout_seconds
            || record.packet_resource_policy != spec.execution.packet_resource_policy
            || record.command_timeout_seconds != spec.command_timeout_seconds
            || record.max_stdout_bytes != spec.max_stdout_bytes
            || record.max_stderr_bytes != spec.max_stderr_bytes
        {
            return Err(format!("invalid identity in run {}", record.sequence));
        }
        if record.source_fingerprint_sha256 != current_fingerprint {
            return Err(format!(
                "stale source fingerprint in run {}",
                record.sequence
            ));
        }
        validate_usage(&record.agent_usage, &spec.execution.pricing_schedule)?;
        if record.progress.len() > MAX_PROGRESS_EVENTS {
            return Err(format!(
                "run {} has too many progress events",
                record.sequence
            ));
        }
        for event in &record.progress {
            validate_progress_event(event)?;
            if event.arm != record.arm {
                return Err(format!(
                    "run {} has progress for another arm",
                    record.sequence
                ));
            }
        }
        if let Some(packet_usage) = &record.packet_usage {
            validate_usage(packet_usage, &spec.execution.pricing_schedule)?;
        }
        if record.arm == Arm::Treatment && record.packet_usage.is_none() {
            return Err(format!(
                "treatment run {} has no packet usage",
                record.sequence
            ));
        }
        if record.arm != Arm::Treatment
            && (record.packet_usage.is_some()
                || record.packet_sha256.is_some()
                || record.packet_bytes != 0)
        {
            return Err(format!(
                "baseline run {} includes packet data",
                record.sequence
            ));
        }
        validate_record_rendered_context(record)?;
        validate_record_totals(record)?;
        validate_returned_evidence(&source_root, &spec.source_files, &record.evidence)?;
        let expected_evidence_verified =
            verify_expected_evidence(expected_task, &record.evidence).is_ok();
        if record.evidence_verified != expected_evidence_verified {
            return Err(format!(
                "run {} has an inconsistent evidence outcome",
                record.sequence
            ));
        }
    }
    Ok(())
}

/// Summarize valid records by arm and treatment comparison.
///
/// # Errors
///
/// Returns an error when record validation fails.
pub fn summarize(spec: &StudySpec, records: &[RunRecord]) -> Result<EvaluationSummary, String> {
    validate_records(spec, records)?;
    let baseline_a = summarize_arm(records, Arm::BaselineA);
    let treatment = summarize_arm(records, Arm::Treatment);
    let baseline_b = summarize_arm(records, Arm::BaselineB);
    let baseline_tokens = mean_pair(baseline_a.mean_total_tokens, baseline_b.mean_total_tokens);
    let baseline_cost = mean_pair(
        baseline_a.mean_estimated_cost_usd,
        baseline_b.mean_estimated_cost_usd,
    );
    let baseline_tools = mean_pair(baseline_a.mean_tool_calls, baseline_b.mean_tool_calls);
    let baseline_reads = mean_pair(
        baseline_a.mean_repository_file_reads,
        baseline_b.mean_repository_file_reads,
    );
    let baseline_repeated_reads = mean_pair(
        baseline_a.mean_repeated_repository_file_reads,
        baseline_b.mean_repeated_repository_file_reads,
    );
    let baseline_time = mean_pair(
        baseline_a.mean_wall_clock_millis,
        baseline_b.mean_wall_clock_millis,
    );
    let baseline_correctness = mean_pair(baseline_a.correctness_rate, baseline_b.correctness_rate);
    let baseline_evidence = mean_pair(
        baseline_a.evidence_verification_rate,
        baseline_b.evidence_verification_rate,
    );
    Ok(EvaluationSummary {
        study_id: spec.study_id.clone(),
        run_count: records.len(),
        baseline_drift: BaselineDrift {
            total_tokens_delta: baseline_b.mean_total_tokens - baseline_a.mean_total_tokens,
            estimated_cost_usd_delta: baseline_b.mean_estimated_cost_usd
                - baseline_a.mean_estimated_cost_usd,
            tool_calls_delta: baseline_b.mean_tool_calls - baseline_a.mean_tool_calls,
            repository_file_reads_delta: baseline_b.mean_repository_file_reads
                - baseline_a.mean_repository_file_reads,
            repeated_repository_file_reads_delta: baseline_b.mean_repeated_repository_file_reads
                - baseline_a.mean_repeated_repository_file_reads,
            wall_clock_millis_delta: baseline_b.mean_wall_clock_millis
                - baseline_a.mean_wall_clock_millis,
            correctness_rate_delta: baseline_b.correctness_rate - baseline_a.correctness_rate,
            evidence_verification_rate_delta: baseline_b.evidence_verification_rate
                - baseline_a.evidence_verification_rate,
        },
        arms: vec![baseline_a, treatment.clone(), baseline_b],
        treatment_vs_baseline: Comparison {
            token_reduction_fraction: reduction(baseline_tokens, treatment.mean_total_tokens),
            cost_reduction_fraction: reduction(baseline_cost, treatment.mean_estimated_cost_usd),
            tool_call_reduction_fraction: reduction(baseline_tools, treatment.mean_tool_calls),
            repository_read_reduction_fraction: reduction(
                baseline_reads,
                treatment.mean_repository_file_reads,
            ),
            repeated_read_reduction_fraction: reduction(
                baseline_repeated_reads,
                treatment.mean_repeated_repository_file_reads,
            ),
            wall_clock_reduction_fraction: reduction(
                baseline_time,
                treatment.mean_wall_clock_millis,
            ),
            correctness_rate_delta: treatment.correctness_rate - baseline_correctness,
            evidence_verification_rate_delta: treatment.evidence_verification_rate
                - baseline_evidence,
        },
    })
}

/// Return the conventional JSON and Markdown output paths.
#[must_use]
pub fn default_summary_paths(output_directory: &Path) -> (PathBuf, PathBuf) {
    (
        output_directory.join("summary.json"),
        output_directory.join("summary.md"),
    )
}

/// Write a JSON document without retaining adapter input or output.
///
/// # Errors
///
/// Returns an error when serialization or writing fails.
pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("encode JSON: {error}"))?;
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

/// Write a concise human-readable result summary.
///
/// # Errors
///
/// Returns an error when formatting or writing fails.
pub fn write_markdown(path: &Path, summary: &EvaluationSummary) -> Result<(), String> {
    let mut text = format!(
        "# Agent-context evaluation: {}\n\nRuns: {}\n\n| Arm | Correctness | Evidence verified | Mean total tokens | Mean total cost (USD) | Mean agent tool calls | Mean native repo reads | Mean repeated native reads | Mean total wall-clock (ms) |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
        summary.study_id, summary.run_count
    );
    for arm in &summary.arms {
        writeln!(
            text,
            "| {:?} | {:.1}% | {:.1}% | {:.2} | {:.6} | {:.2} | {:.2} | {:.2} | {:.2} |\n",
            arm.arm,
            arm.correctness_rate * 100.0,
            arm.evidence_verification_rate * 100.0,
            arm.mean_total_tokens,
            arm.mean_estimated_cost_usd,
            arm.mean_tool_calls,
            arm.mean_repository_file_reads,
            arm.mean_repeated_repository_file_reads,
            arm.mean_wall_clock_millis
        )
        .map_err(|error| format!("format Markdown: {error}"))?;
    }
    writeln!(
        text,
        "\nTreatment versus mean baseline: {:.1}% fewer tokens, {:.1}% lower estimated cost, {:.1}% fewer tool calls, {:.1}% fewer repository reads, {:.1}% fewer repeated reads, {:.1}% faster; correctness delta {:+.1} points; evidence-verification delta {:+.1} points.\n",
        summary.treatment_vs_baseline.token_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.cost_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.tool_call_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.repository_read_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.repeated_read_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.wall_clock_reduction_fraction * 100.0,
        summary.treatment_vs_baseline.correctness_rate_delta * 100.0,
        summary.treatment_vs_baseline.evidence_verification_rate_delta * 100.0
    )
    .map_err(|error| format!("format Markdown: {error}"))?;
    writeln!(
        text,
        "\nBaseline B minus Baseline A drift: {:+.2} tokens, {:+.6} USD, {:+.2} tool calls, {:+.2} repository reads, {:+.2} repeated reads, {:+.2} ms; correctness {:+.1} points; evidence verification {:+.1} points.",
        summary.baseline_drift.total_tokens_delta,
        summary.baseline_drift.estimated_cost_usd_delta,
        summary.baseline_drift.tool_calls_delta,
        summary.baseline_drift.repository_file_reads_delta,
        summary.baseline_drift.repeated_repository_file_reads_delta,
        summary.baseline_drift.wall_clock_millis_delta,
        summary.baseline_drift.correctness_rate_delta * 100.0,
        summary.baseline_drift.evidence_verification_rate_delta * 100.0,
    )
    .map_err(|error| format!("format Markdown: {error}"))?;
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

#[allow(clippy::too_many_lines)]
fn execute_run(
    spec: &StudySpec,
    source_root: &Path,
    task: &TaskSpec,
    arm: Arm,
    repetition: u32,
    sequence: u64,
) -> Result<RunRecord, AdapterExecutionError> {
    let total_start = Instant::now();
    let fingerprint_before = source_fingerprint(spec, source_root)?;
    let workspace_root = source_root.display().to_string();
    let mut packet_generation_millis = 0_u64;
    let mut packet_bytes = 0_u64;
    let mut packet_sha256 = None;
    let mut packet_usage = None;
    let mut packet = None;

    if arm == Arm::Treatment {
        let request = AdapterRequest {
            task_id: task.id.clone(),
            prompt: task.prompt.clone(),
            arm,
            workspace_root: workspace_root.clone(),
            source_files: spec.source_files.clone(),
            context_plan: task.context_plan.clone(),
            source_fingerprint_sha256: fingerprint_before.clone(),
            model_identifier: spec.execution.model_identifier.clone(),
            model_context_renderer_identifier: spec
                .execution
                .model_context_renderer_identifier
                .clone(),
            model_context_renderer_version: spec.execution.model_context_renderer_version.clone(),
            max_rendered_context_bytes: spec.execution.max_rendered_context_bytes,
            pricing_schedule: spec.execution.pricing_schedule.clone(),
            container_image: spec.execution.container_image.clone(),
            operation_timestamp: spec.execution.operation_timestamp.clone(),
            turn_limit: spec.execution.turn_limit,
            provider_effort: spec.execution.provider_effort.clone(),
            provider_max_output_tokens: spec.execution.provider_max_output_tokens,
            provider_request_timeout_seconds: spec.execution.provider_request_timeout_seconds,
            command_timeout_seconds: spec.command_timeout_seconds,
            packet_resource_policy: spec.execution.packet_resource_policy.clone(),
            packet: None,
        };
        let execution =
            execute_adapter::<PacketResponse, _>(spec, &spec.packet_command, &request, false)?;
        let response = execution.response;
        let elapsed = execution.elapsed_millis;
        if response.source_fingerprint_sha256 != fingerprint_before {
            return Err(format!(
                "packet adapter returned a mismatched source fingerprint for {}",
                task.id
            )
            .into());
        }
        validate_usage(&response.usage, &spec.execution.pricing_schedule)?;
        packet_generation_millis = elapsed;
        packet_bytes =
            u64::try_from(response.packet.len()).map_err(|_| "packet too large".to_owned())?;
        if packet_bytes > u64::try_from(spec.max_stdout_bytes).unwrap_or(u64::MAX) {
            return Err(format!("packet is larger than {} bytes", spec.max_stdout_bytes).into());
        }
        packet_sha256 = Some(hash_bytes(response.packet.as_bytes()));
        packet_usage = Some(response.usage);
        packet = Some(response.packet);
    }

    let request = AdapterRequest {
        task_id: task.id.clone(),
        prompt: task.prompt.clone(),
        arm,
        workspace_root,
        source_files: spec.source_files.clone(),
        context_plan: task.context_plan.clone(),
        source_fingerprint_sha256: fingerprint_before.clone(),
        model_identifier: spec.execution.model_identifier.clone(),
        model_context_renderer_identifier: spec.execution.model_context_renderer_identifier.clone(),
        model_context_renderer_version: spec.execution.model_context_renderer_version.clone(),
        max_rendered_context_bytes: spec.execution.max_rendered_context_bytes,
        pricing_schedule: spec.execution.pricing_schedule.clone(),
        container_image: spec.execution.container_image.clone(),
        operation_timestamp: spec.execution.operation_timestamp.clone(),
        turn_limit: spec.execution.turn_limit,
        provider_effort: spec.execution.provider_effort.clone(),
        provider_max_output_tokens: spec.execution.provider_max_output_tokens,
        provider_request_timeout_seconds: spec.execution.provider_request_timeout_seconds,
        command_timeout_seconds: spec.command_timeout_seconds,
        packet_resource_policy: spec.execution.packet_resource_policy.clone(),
        packet,
    };
    let execution = execute_adapter::<AgentResponse, _>(spec, &spec.agent_command, &request, true)?;
    let response = execution.response;
    let agent_wall_clock_millis = execution.elapsed_millis;
    let progress = execution.progress;
    if response.source_fingerprint_sha256 != fingerprint_before {
        return Err(format!(
            "agent adapter returned a mismatched source fingerprint for {}",
            task.id
        )
        .into());
    }
    validate_usage(&response.usage, &spec.execution.pricing_schedule)?;
    validate_response_rendered_context(spec, arm, response.rendered_context.as_ref())?;
    let correctness = answer_is_correct(task, &response.answer);
    validate_returned_evidence(source_root, &spec.source_files, &response.evidence)?;
    let evidence_verified = verify_expected_evidence(task, &response.evidence).is_ok();
    if source_fingerprint(spec, source_root)? != fingerprint_before {
        return Err(format!("evaluated source changed during run {sequence}").into());
    }

    let packet_usage_value = packet_usage.clone().unwrap_or_default();
    let total_input_tokens = packet_usage_value
        .input_tokens
        .saturating_add(response.usage.input_tokens);
    let total_output_tokens = packet_usage_value
        .output_tokens
        .saturating_add(response.usage.output_tokens);
    let total_tokens = packet_usage_value
        .total_tokens
        .saturating_add(response.usage.total_tokens);
    let total_estimated_cost_usd =
        packet_usage_value.estimated_cost_usd + response.usage.estimated_cost_usd;
    let total_tool_calls = packet_usage_value
        .tool_calls
        .saturating_add(response.usage.tool_calls);
    let total_repository_file_reads = packet_usage_value
        .repository_file_reads
        .saturating_add(response.usage.repository_file_reads);
    let total_repeated_repository_file_reads = packet_usage_value
        .repeated_repository_file_reads
        .saturating_add(response.usage.repeated_repository_file_reads);
    let rendered_context = response.rendered_context.clone();

    Ok(RunRecord {
        schema_version: SCHEMA_VERSION.to_owned(),
        study_id: spec.study_id.clone(),
        sequence,
        repetition,
        arm,
        task_id: task.id.clone(),
        source_fingerprint_sha256: fingerprint_before,
        workspace_revision: spec.workspace_revision.clone(),
        model_identifier: spec.execution.model_identifier.clone(),
        agent_adapter_identifier: spec.execution.agent_adapter_identifier.clone(),
        agent_adapter_version: spec.execution.agent_adapter_version.clone(),
        packet_adapter_identifier: spec.execution.packet_adapter_identifier.clone(),
        packet_adapter_version: spec.execution.packet_adapter_version.clone(),
        model_context_renderer_identifier: spec.execution.model_context_renderer_identifier.clone(),
        model_context_renderer_version: spec.execution.model_context_renderer_version.clone(),
        max_rendered_context_bytes: spec.execution.max_rendered_context_bytes,
        pricing_basis: spec.execution.pricing_basis.clone(),
        pricing_schedule: spec.execution.pricing_schedule.clone(),
        container_image: spec.execution.container_image.clone(),
        operation_timestamp: spec.execution.operation_timestamp.clone(),
        turn_limit: spec.execution.turn_limit,
        provider_effort: spec.execution.provider_effort.clone(),
        provider_max_output_tokens: spec.execution.provider_max_output_tokens,
        provider_request_timeout_seconds: spec.execution.provider_request_timeout_seconds,
        packet_resource_policy: spec.execution.packet_resource_policy.clone(),
        command_timeout_seconds: spec.command_timeout_seconds,
        max_stdout_bytes: spec.max_stdout_bytes,
        max_stderr_bytes: spec.max_stderr_bytes,
        packet_generation_millis,
        packet_bytes,
        packet_sha256,
        packet_usage,
        rendered_context_bytes: rendered_context.as_ref().map_or(0, |value| value.bytes),
        rendered_context_sha256: rendered_context.as_ref().map(|value| value.sha256.clone()),
        rendered_context_evidence_count: rendered_context
            .as_ref()
            .map_or(0, |value| value.evidence_count),
        agent_usage: response.usage,
        total_input_tokens,
        total_output_tokens,
        total_tokens,
        total_estimated_cost_usd,
        total_tool_calls,
        total_repository_file_reads,
        total_repeated_repository_file_reads,
        agent_wall_clock_millis,
        total_wall_clock_millis: duration_millis(total_start.elapsed()),
        correctness,
        evidence_verified,
        evidence_count: u64::try_from(response.evidence.len()).unwrap_or(u64::MAX),
        evidence: response.evidence,
        progress,
    })
}

struct AdapterExecution<T> {
    response: T,
    elapsed_millis: u64,
    progress: Vec<AdapterProgressEvent>,
}

#[derive(Debug)]
struct AdapterExecutionError {
    message: String,
    reason_code: String,
    elapsed_millis: u64,
    progress: Vec<AdapterProgressEvent>,
}

impl AdapterExecutionError {
    fn plain(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            reason_code: "harness_validation_failed".into(),
            elapsed_millis: 0,
            progress: Vec::new(),
        }
    }

    fn to_record(
        &self,
        spec: &StudySpec,
        task: &TaskSpec,
        arm: Arm,
        repetition: u32,
        sequence: u64,
    ) -> RunFailureRecord {
        RunFailureRecord {
            schema_name: "agent-evaluation-failure".into(),
            schema_version: "1.0".into(),
            study_id: spec.study_id.clone(),
            sequence,
            repetition,
            arm,
            task_id: task.id.clone(),
            model_identifier: spec.execution.model_identifier.clone(),
            provider_effort: spec.execution.provider_effort.clone(),
            provider_max_output_tokens: spec.execution.provider_max_output_tokens,
            provider_request_timeout_seconds: spec.execution.provider_request_timeout_seconds,
            command_timeout_seconds: spec.command_timeout_seconds,
            failure_stage: self.progress.last().map_or_else(
                || "harness".into(),
                |event| progress_stage_name(event.stage).into(),
            ),
            reason_code: self.reason_code.clone(),
            elapsed_millis: self.elapsed_millis,
            progress: self.progress.clone(),
        }
    }
}

impl From<String> for AdapterExecutionError {
    fn from(message: String) -> Self {
        Self::plain(message)
    }
}

struct CapturedStream {
    bytes: Vec<u8>,
    total_bytes: usize,
}

fn capture_stream<R: Read>(mut stream: R, retained_limit: usize) -> CapturedStream {
    let mut bytes = Vec::new();
    let mut total_bytes = 0_usize;
    let mut chunk = [0_u8; 8192];
    loop {
        let count = match stream.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        total_bytes = total_bytes.saturating_add(count);
        if bytes.len() < retained_limit {
            let retain = count.min(retained_limit - bytes.len());
            bytes.extend_from_slice(&chunk[..retain]);
        }
    }
    CapturedStream { bytes, total_bytes }
}

fn parse_progress(stderr: &[u8]) -> Result<Vec<AdapterProgressEvent>, String> {
    let text = std::str::from_utf8(stderr).map_err(|_| "adapter stderr is not UTF-8".to_owned())?;
    let mut events = Vec::new();
    for line in text.lines() {
        let Some(encoded) = line.strip_prefix(PROGRESS_PREFIX) else {
            continue;
        };
        if events.len() >= MAX_PROGRESS_EVENTS {
            return Err("adapter progress event limit exceeded".to_owned());
        }
        let event: AdapterProgressEvent = serde_json::from_str(encoded)
            .map_err(|_| "adapter emitted malformed progress telemetry".to_owned())?;
        validate_progress_event(&event)?;
        events.push(event);
    }
    Ok(events)
}

fn validate_progress_event(event: &AdapterProgressEvent) -> Result<(), String> {
    if event.schema_name != "agent-evaluation-progress"
        || event.schema_version != "1.0"
        || !matches!(event.provider.as_str(), "openai" | "anthropic")
        || event.turn > 200
        || event
            .stop_reason
            .as_ref()
            .is_some_and(|value| value.len() > 64)
        || event
            .request_id_sha256
            .as_ref()
            .is_some_and(|value| !is_sha256(value))
        || event
            .rendered_context_sha256
            .as_ref()
            .is_some_and(|value| !is_sha256(value))
    {
        return Err("adapter emitted invalid progress telemetry".to_owned());
    }
    if let Some(usage) = &event.usage
        && (usage.total_tokens != usage.input_tokens.saturating_add(usage.output_tokens)
            || !usage.estimated_cost_usd.is_finite()
            || usage.estimated_cost_usd < 0.0)
    {
        return Err("adapter emitted invalid progress usage".to_owned());
    }
    Ok(())
}

const fn progress_stage_name(stage: ProgressStage) -> &'static str {
    match stage {
        ProgressStage::Rendered => "rendered",
        ProgressStage::TokenCounted => "token_counted",
        ProgressStage::ProviderRequestStarted => "provider_request_started",
        ProgressStage::ProviderResponseCompleted => "provider_response_completed",
        ProgressStage::ToolsDispatched => "tools_dispatched",
        ProgressStage::Completed => "completed",
    }
}

#[allow(clippy::too_many_lines)]
fn execute_adapter<T: DeserializeOwned, R: Serialize>(
    spec: &StudySpec,
    command: &[String],
    request: &R,
    include_agent_secrets: bool,
) -> Result<AdapterExecution<T>, AdapterExecutionError> {
    let input =
        serde_json::to_vec(request).map_err(|error| format!("encode adapter request: {error}"))?;
    let started = Instant::now();
    let mut configured = Command::new(&command[0]);
    configured
        .args(&command[1..])
        .current_dir(&spec.base_directory)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in &spec.environment {
        configured.env(key, value);
    }
    if include_agent_secrets {
        for key in &spec.agent_secret_environment_variables {
            let value = std::env::var_os(key)
                .ok_or_else(|| format!("required agent secret variable {key:?} is not set"))?;
            configured.env(key, value);
        }
    }
    let mut child = configured.spawn().map_err(|error| {
        AdapterExecutionError::plain(format!("start adapter {:?}: {error}", command[0]))
    })?;
    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "adapter stdin unavailable".to_owned())?;
        stdin
            .write_all(&input)
            .map_err(|error| format!("write adapter stdin: {error}"))?;
    }
    drop(child.stdin.take());
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "adapter stdout unavailable".to_owned())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "adapter stderr unavailable".to_owned())?;
    let stdout_limit = spec.max_stdout_bytes.saturating_add(1);
    let stderr_limit = spec.max_stderr_bytes.saturating_add(1);
    let stdout_thread = std::thread::spawn(move || capture_stream(stdout, stdout_limit));
    let stderr_thread = std::thread::spawn(move || capture_stream(stderr, stderr_limit));
    let limit = Duration::from_secs(spec.command_timeout_seconds);
    let mut timed_out = false;
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("check adapter status: {error}"))?
            .is_some()
        {
            break;
        }
        if started.elapsed() > limit {
            child.kill().map_err(|error| {
                AdapterExecutionError::plain(format!("stop timed-out adapter: {error}"))
            })?;
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let status = child.wait().map_err(|error| {
        AdapterExecutionError::plain(format!("collect adapter status: {error}"))
    })?;
    let stdout = stdout_thread
        .join()
        .map_err(|_| AdapterExecutionError::plain("stdout capture thread failed"))?;
    let stderr = stderr_thread
        .join()
        .map_err(|_| AdapterExecutionError::plain("stderr capture thread failed"))?;
    let elapsed_millis = duration_millis(started.elapsed());
    let progress = parse_progress(&stderr.bytes).map_err(|message| AdapterExecutionError {
        message,
        reason_code: "invalid_progress".into(),
        elapsed_millis,
        progress: Vec::new(),
    })?;
    let failure = |message: String, reason_code: &str| AdapterExecutionError {
        message,
        reason_code: reason_code.into(),
        elapsed_millis,
        progress: progress.clone(),
    };
    if timed_out {
        return Err(failure(
            format!("adapter exceeded {} seconds", spec.command_timeout_seconds),
            "adapter_deadline_exceeded",
        ));
    }
    if stdout.total_bytes > spec.max_stdout_bytes {
        return Err(failure(
            format!("adapter stdout exceeded {} bytes", spec.max_stdout_bytes),
            "stdout_overflow",
        ));
    }
    if stderr.total_bytes > spec.max_stderr_bytes {
        return Err(failure(
            format!("adapter stderr exceeded {} bytes", spec.max_stderr_bytes),
            "stderr_overflow",
        ));
    }
    if !status.success() {
        return Err(failure(
            format!(
                "adapter exited {status}: {}",
                String::from_utf8_lossy(&stderr.bytes).trim()
            ),
            "adapter_failed",
        ));
    }
    let response = serde_json::from_slice(&stdout.bytes).map_err(|error| {
        failure(
            format!("parse adapter response: {error}"),
            "invalid_response",
        )
    })?;
    Ok(AdapterExecution {
        response,
        elapsed_millis,
        progress,
    })
}

#[allow(clippy::too_many_lines)]
fn validate_spec(spec: &StudySpec) -> Result<(), String> {
    if spec.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported schema version {}",
            spec.schema_version
        ));
    }
    validate_identifier(&spec.study_id, "study id")?;
    if spec.repository.is_empty() || !is_relative_safe_path(&spec.repository) {
        return Err("repository must be a safe relative path".to_owned());
    }
    if spec.source_files.is_empty() || spec.source_files.len() > 10_000 {
        return Err("source_files must include between 1 and 10000 paths".to_owned());
    }
    let mut source_files = BTreeSet::new();
    for path in &spec.source_files {
        if !is_relative_safe_path(path) || !source_files.insert(path.as_str()) {
            return Err(format!("source file {path:?} is unsafe or duplicated"));
        }
    }
    if spec.workspace_revision.trim().is_empty() {
        return Err("workspace revision must not be empty".to_owned());
    }
    if !(1..=200).contains(&spec.execution.turn_limit) {
        return Err("turn limit must be between 1 and 200".to_owned());
    }
    if !matches!(
        spec.execution.provider_effort.as_str(),
        "low" | "medium" | "high"
    ) {
        return Err("provider effort must be low, medium, or high".to_owned());
    }
    if !(1..=65_536).contains(&spec.execution.provider_max_output_tokens) {
        return Err("provider max output tokens must be between 1 and 65536".to_owned());
    }
    if spec.execution.provider_request_timeout_seconds == 0
        || spec
            .execution
            .provider_request_timeout_seconds
            .saturating_add(5)
            >= spec.command_timeout_seconds
    {
        return Err(
            "provider request timeout must leave at least five seconds inside the command timeout"
                .to_owned(),
        );
    }
    spec.execution.packet_resource_policy.to_resource_budget()?;
    if spec.execution.packet_budget_curve.len() > 16 {
        return Err("packet budget curve must not exceed 16 points".to_owned());
    }
    for point in &spec.execution.packet_budget_curve {
        point.to_resource_budget()?;
    }
    if !(1..=50).contains(&spec.repetitions) {
        return Err("repetitions must be between 1 and 50".to_owned());
    }
    if !(1..=600).contains(&spec.command_timeout_seconds) {
        return Err("command timeout must be between 1 and 600 seconds".to_owned());
    }
    if !(1..=MAX_ADAPTER_STREAM_BYTES).contains(&spec.max_stdout_bytes)
        || !(1..=MAX_ADAPTER_STREAM_BYTES).contains(&spec.max_stderr_bytes)
    {
        return Err(format!(
            "adapter stdout and stderr limits must be between 1 and {MAX_ADAPTER_STREAM_BYTES} bytes"
        ));
    }
    validate_identifier(
        &spec.execution.agent_adapter_identifier,
        "agent adapter identifier",
    )?;
    validate_identifier(
        &spec.execution.packet_adapter_identifier,
        "packet adapter identifier",
    )?;
    validate_renderer_spec(&spec.execution)?;
    if spec.execution.agent_adapter_version.trim().is_empty()
        || spec.execution.packet_adapter_version.trim().is_empty()
        || spec.execution.model_identifier.trim().is_empty()
        || spec.execution.container_image.trim().is_empty()
        || spec.execution.pricing_basis.trim().is_empty()
    {
        return Err("execution metadata must not be empty".to_owned());
    }
    validate_pricing_schedule(&spec.execution.pricing_schedule)?;
    context_core::validate_utc_timestamp(&spec.execution.operation_timestamp)
        .map_err(|_| "execution operation_timestamp must be normalized UTC RFC 3339".to_owned())?;
    validate_command(&spec.agent_command, "agent command")?;
    validate_command(&spec.packet_command, "packet command")?;
    for (key, value) in &spec.environment {
        if !key.starts_with("IMPRESARI_EVAL_") || key.len() > 128 || value.len() > 4096 {
            return Err(format!("environment key {key:?} is not allow-listed"));
        }
    }
    validate_agent_secret_names(&spec.agent_secret_environment_variables)?;
    if spec.tasks.is_empty() || spec.tasks.len() > 100 {
        return Err("study must include between 1 and 100 tasks".to_owned());
    }
    let mut ids = BTreeSet::new();
    for task in &spec.tasks {
        validate_identifier(&task.id, "task id")?;
        if !ids.insert(task.id.as_str())
            || task.prompt.trim().is_empty()
            || task.uniqueness_rationale.trim().is_empty()
            || task.uniqueness_rationale.len() > 512
            || task.uniqueness_rationale.contains('\0')
            || task.expected_answer_fragments.is_empty()
            || task.required_evidence.is_empty()
            || task.required_evidence.len() > 32
        {
            return Err(format!("task {:?} is incomplete or duplicated", task.id));
        }
        let query_text = task
            .context_plan
            .iter()
            .map(|step| step.query.to_lowercase())
            .collect::<Vec<_>>()
            .join("\n");
        if task.expected_answer_fragments.iter().any(|fragment| {
            !fragment.trim().is_empty() && query_text.contains(&fragment.to_lowercase())
        }) {
            return Err(format!(
                "task {:?} leaks an expected answer fragment into its context plan",
                task.id
            ));
        }
        for requirement in &task.required_evidence {
            if !is_relative_safe_path(&requirement.path)
                || !source_files.contains(requirement.path.as_str())
                || requirement.line_start == 0
                || requirement.line_end < requirement.line_start
            {
                return Err(format!(
                    "task {:?} has unsafe evidence requirements",
                    task.id
                ));
            }
        }
        validate_context_plan(task)?;
    }
    let root = source_root(spec)?;
    for path in &spec.source_files {
        let _ = resolve_source_file(&root, path)?;
    }
    Ok(())
}

fn default_operation_timestamp() -> String {
    "1970-01-01T00:00:00Z".to_owned()
}

fn validate_renderer_spec(execution: &ExecutionSpec) -> Result<(), String> {
    validate_identifier(
        &execution.model_context_renderer_identifier,
        "model context renderer identifier",
    )?;
    if execution.model_context_renderer_version.trim().is_empty() {
        return Err("model context renderer version must not be empty".to_owned());
    }
    if !(1..=MAX_RENDERED_CONTEXT_BYTES).contains(&execution.max_rendered_context_bytes) {
        return Err(format!(
            "rendered context limit must be between 1 and {MAX_RENDERED_CONTEXT_BYTES} bytes"
        ));
    }
    Ok(())
}

fn validate_agent_secret_names(names: &[String]) -> Result<(), String> {
    let mut unique = BTreeSet::new();
    for key in names {
        if !matches!(key.as_str(), "OPENAI_API_KEY" | "ANTHROPIC_API_KEY") || !unique.insert(key) {
            return Err(format!(
                "agent secret variable {key:?} is not allow-listed or is duplicated"
            ));
        }
    }
    Ok(())
}

fn validate_context_plan(task: &TaskSpec) -> Result<(), String> {
    if task.context_plan.len() > 8
        || task.context_plan.iter().any(|step| {
            step.query.is_empty() || step.query.len() > 4096 || step.query.contains('\0')
        })
    {
        Err(format!("task {:?} has an invalid context plan", task.id))
    } else {
        Ok(())
    }
}

fn validate_pricing_schedule(schedule: &PricingSchedule) -> Result<(), String> {
    let rates = [
        schedule.input_usd_per_million,
        schedule.cached_input_usd_per_million,
        schedule.cache_write_input_usd_per_million,
        schedule.output_usd_per_million,
    ];
    if schedule.currency != "USD"
        || schedule.effective_date.trim().is_empty()
        || rates.iter().any(|rate| !rate.is_finite() || *rate < 0.0)
    {
        return Err("pricing schedule is invalid".to_owned());
    }
    Ok(())
}

fn validate_identifier(value: &str, description: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 80
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if valid {
        Ok(())
    } else {
        Err(format!(
            "{description} must be lowercase ASCII words separated by hyphens"
        ))
    }
}

fn validate_command(command: &[String], description: &str) -> Result<(), String> {
    if command.is_empty()
        || command.len() > 8
        || command
            .iter()
            .any(|argument| argument.is_empty() || argument.contains('\0'))
    {
        return Err(format!("{description} must be a fixed non-empty argv list"));
    }
    if command
        .iter()
        .any(|argument| argument == "-c" || argument == "--command")
    {
        return Err(format!(
            "{description} must not invoke a shell command string"
        ));
    }
    Ok(())
}

fn source_root(spec: &StudySpec) -> Result<PathBuf, String> {
    let path = spec.base_directory.join(&spec.repository);
    let root = path.canonicalize().map_err(|error| {
        format!(
            "canonicalize evaluated repository {}: {error}",
            path.display()
        )
    })?;
    if !root.starts_with(&spec.base_directory) || !root.is_dir() {
        return Err(
            "evaluated repository must be a directory under the study directory".to_owned(),
        );
    }
    Ok(root)
}

fn source_fingerprint(spec: &StudySpec, root: &Path) -> Result<String, String> {
    let mut files = spec.source_files.iter().collect::<Vec<_>>();
    files.sort();
    let mut hasher = Sha256::new();
    for relative in files {
        let path = resolve_source_file(root, relative)?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher
            .update(fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?);
        hasher.update([0]);
    }
    Ok(hex_digest(hasher.finalize()))
}

fn answer_is_correct(task: &TaskSpec, answer: &str) -> bool {
    let normalized = answer.to_lowercase();
    task.expected_answer_fragments
        .iter()
        .all(|fragment| normalized.contains(&fragment.to_lowercase()))
}

fn verify_expected_evidence(task: &TaskSpec, citations: &[EvidenceCitation]) -> Result<(), String> {
    if citations.len() != task.required_evidence.len() {
        return Err(format!(
            "task {} returned an unexpected evidence count",
            task.id
        ));
    }
    for required in &task.required_evidence {
        if !citations.iter().any(|citation| {
            citation.path == required.path
                && citation.line_start == required.line_start
                && citation.line_end == required.line_end
        }) {
            return Err(format!(
                "task {} omitted required evidence {}",
                task.id, required.path
            ));
        }
    }
    Ok(())
}

fn validate_returned_evidence(
    root: &Path,
    source_files: &[String],
    citations: &[EvidenceCitation],
) -> Result<(), String> {
    if citations.len() > 32 {
        return Err("agent returned more than 32 evidence citations".to_owned());
    }
    for citation in citations {
        if !source_files.iter().any(|path| path == &citation.path) {
            return Err("agent returned evidence outside the source allowlist".to_owned());
        }
        let bytes =
            source_range_values(root, &citation.path, citation.line_start, citation.line_end)
                .map_err(|_| "agent returned an invalid evidence range".to_owned())?;
        if citation.sha256 != hash_bytes(&bytes) {
            return Err("agent returned an invalid evidence digest".to_owned());
        }
    }
    Ok(())
}

fn source_range_values(
    root: &Path,
    relative: &str,
    line_start: u32,
    line_end: u32,
) -> Result<Vec<u8>, String> {
    if line_start == 0 || line_end < line_start {
        return Err("invalid evidence line range".to_owned());
    }
    let path = resolve_source_file(root, relative)?;
    let contents =
        fs::read_to_string(&path).map_err(|error| format!("read evidence {relative}: {error}"))?;
    let start = usize::try_from(line_start.saturating_sub(1))
        .map_err(|_| "invalid evidence line".to_owned())?;
    let end = usize::try_from(line_end).map_err(|_| "invalid evidence line".to_owned())?;
    let lines = contents.lines().collect::<Vec<_>>();
    if start >= lines.len() || end > lines.len() {
        return Err(format!("evidence range outside file: {relative}"));
    }
    Ok(lines[start..end].join("\n").into_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(hasher.finalize())
}

fn hex_digest(digest: impl IntoIterator<Item = u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::from("sha256:");
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn is_relative_safe_path(value: &str) -> bool {
    let path = Path::new(value);
    path.is_relative()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn resolve_source_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if !is_relative_safe_path(relative) {
        return Err(format!("unsafe source path {relative:?}"));
    }
    let mut current = root.to_path_buf();
    for component in Path::new(relative).components() {
        let Component::Normal(part) = component else {
            return Err(format!("unsafe source path {relative:?}"));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("inspect source {relative:?}: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symlink is not allowed in source path {relative:?}"
            ));
        }
    }
    let metadata =
        fs::metadata(&current).map_err(|error| format!("inspect source {relative:?}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!("source path {relative:?} is not a regular file"));
    }
    Ok(current)
}

fn validate_usage(usage: &Usage, schedule: &PricingSchedule) -> Result<(), String> {
    let expected_cost = estimated_cost(usage, schedule)?;
    let tolerance = 1e-9_f64.max(expected_cost.abs() * 1e-9);
    if usage.total_tokens != usage.input_tokens.saturating_add(usage.output_tokens)
        || !usage.estimated_cost_usd.is_finite()
        || usage.estimated_cost_usd < 0.0
        || usage.reasoning_tokens > usage.output_tokens
        || (usage.estimated_cost_usd - expected_cost).abs() > tolerance
    {
        return Err("adapter reported invalid usage accounting".to_owned());
    }
    Ok(())
}

/// Computes cost from normalized provider counters and a frozen schedule.
///
/// `input_tokens` includes uncached, cache-read, and cache-write input tokens.
///
/// # Errors
///
/// Returns an error when cache categories exceed total input or a rate is
/// invalid.
pub fn estimated_cost(usage: &Usage, schedule: &PricingSchedule) -> Result<f64, String> {
    validate_pricing_schedule(schedule)?;
    let categorized = usage
        .cached_input_tokens
        .checked_add(usage.cache_write_input_tokens)
        .ok_or_else(|| "provider cache token accounting overflowed".to_owned())?;
    let uncached = usage
        .input_tokens
        .checked_sub(categorized)
        .ok_or_else(|| "provider cache tokens exceed total input".to_owned())?;
    #[allow(clippy::cast_precision_loss)]
    let cost = ((uncached as f64) * schedule.input_usd_per_million
        + (usage.cached_input_tokens as f64) * schedule.cached_input_usd_per_million
        + (usage.cache_write_input_tokens as f64) * schedule.cache_write_input_usd_per_million
        + (usage.output_tokens as f64) * schedule.output_usd_per_million)
        / 1_000_000.0;
    if cost.is_finite() {
        Ok(cost)
    } else {
        Err("provider cost calculation is not finite".to_owned())
    }
}

fn validate_record_totals(record: &RunRecord) -> Result<(), String> {
    let packet = record.packet_usage.clone().unwrap_or_default();
    if record.total_input_tokens
        != packet
            .input_tokens
            .saturating_add(record.agent_usage.input_tokens)
        || record.total_output_tokens
            != packet
                .output_tokens
                .saturating_add(record.agent_usage.output_tokens)
        || record.total_tokens
            != packet
                .total_tokens
                .saturating_add(record.agent_usage.total_tokens)
        || record.total_tool_calls
            != packet
                .tool_calls
                .saturating_add(record.agent_usage.tool_calls)
        || record.total_repository_file_reads
            != packet
                .repository_file_reads
                .saturating_add(record.agent_usage.repository_file_reads)
        || record.total_repeated_repository_file_reads
            != packet
                .repeated_repository_file_reads
                .saturating_add(record.agent_usage.repeated_repository_file_reads)
        || !record.total_estimated_cost_usd.is_finite()
        || (record.total_estimated_cost_usd
            - (packet.estimated_cost_usd + record.agent_usage.estimated_cost_usd))
            .abs()
            > f64::EPSILON
        || record.evidence_count != u64::try_from(record.evidence.len()).unwrap_or(u64::MAX)
    {
        return Err(format!(
            "invalid aggregate accounting in run {}",
            record.sequence
        ));
    }
    Ok(())
}

fn validate_response_rendered_context(
    spec: &StudySpec,
    arm: Arm,
    rendered: Option<&RenderedContextMetadata>,
) -> Result<(), String> {
    match (arm, rendered) {
        (Arm::Treatment, Some(rendered))
            if rendered.renderer_identifier == spec.execution.model_context_renderer_identifier
                && rendered.renderer_version == spec.execution.model_context_renderer_version
                && rendered.bytes > 0
                && rendered.bytes
                    <= u64::try_from(spec.execution.max_rendered_context_bytes)
                        .unwrap_or(u64::MAX)
                && is_sha256(&rendered.sha256) =>
        {
            Ok(())
        }
        (Arm::BaselineA | Arm::BaselineB, None) => Ok(()),
        _ => Err("agent adapter returned invalid rendered-context metadata".to_owned()),
    }
}

fn validate_record_rendered_context(record: &RunRecord) -> Result<(), String> {
    match record.arm {
        Arm::Treatment
            if record.rendered_context_bytes > 0
                && record.rendered_context_bytes
                    <= u64::try_from(record.max_rendered_context_bytes).unwrap_or(u64::MAX)
                && record
                    .rendered_context_sha256
                    .as_deref()
                    .is_some_and(is_sha256) =>
        {
            Ok(())
        }
        Arm::BaselineA | Arm::BaselineB
            if record.rendered_context_bytes == 0
                && record.rendered_context_sha256.is_none()
                && record.rendered_context_evidence_count == 0 =>
        {
            Ok(())
        }
        _ => Err(format!(
            "invalid rendered-context accounting in run {}",
            record.sequence
        )),
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {}: {error}", path.display()))
}

#[allow(clippy::cast_precision_loss)]
fn summarize_arm(records: &[RunRecord], arm: Arm) -> ArmSummary {
    let arm_records = records
        .iter()
        .filter(|record| record.arm == arm)
        .collect::<Vec<_>>();
    ArmSummary {
        arm,
        runs: arm_records.len(),
        correctness_rate: mean(
            arm_records
                .iter()
                .map(|record| f64::from(record.correctness)),
        ),
        evidence_verification_rate: mean(
            arm_records
                .iter()
                .map(|record| f64::from(record.evidence_verified)),
        ),
        mean_total_tokens: mean(arm_records.iter().map(|record| record.total_tokens as f64)),
        mean_estimated_cost_usd: mean(
            arm_records
                .iter()
                .map(|record| record.total_estimated_cost_usd),
        ),
        mean_tool_calls: mean(
            arm_records
                .iter()
                .map(|record| record.agent_usage.tool_calls as f64),
        ),
        mean_repository_file_reads: mean(
            arm_records
                .iter()
                .map(|record| record.agent_usage.repository_file_reads as f64),
        ),
        mean_repeated_repository_file_reads: mean(
            arm_records
                .iter()
                .map(|record| record.agent_usage.repeated_repository_file_reads as f64),
        ),
        mean_wall_clock_millis: mean(
            arm_records
                .iter()
                .map(|record| record.total_wall_clock_millis as f64),
        ),
    }
}

#[allow(clippy::cast_precision_loss)]
fn mean(values: impl Iterator<Item = f64>) -> f64 {
    let values = values.collect::<Vec<_>>();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn mean_pair(left: f64, right: f64) -> f64 {
    left.midpoint(right)
}

fn reduction(baseline: f64, treatment: f64) -> f64 {
    if baseline == 0.0 {
        0.0
    } else {
        (baseline - treatment) / baseline
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::{PricingSchedule, Usage, estimated_cost, validate_identifier};

    #[test]
    fn identifiers_are_strict_and_predictable() {
        assert!(validate_identifier("valid-study-1", "test").is_ok());
        assert!(validate_identifier("Invalid", "test").is_err());
        assert!(validate_identifier("-invalid", "test").is_err());
    }

    #[test]
    fn cost_uses_frozen_uncached_cached_write_and_output_rates() {
        let usage = Usage {
            input_tokens: 1_000,
            output_tokens: 100,
            total_tokens: 1_100,
            estimated_cost_usd: 0.0,
            cached_input_tokens: 200,
            cache_write_input_tokens: 100,
            reasoning_tokens: 20,
            provider_requests: 1,
            tool_calls: 0,
            repository_file_reads: 0,
            repeated_repository_file_reads: 0,
        };
        let schedule = PricingSchedule {
            currency: "USD".into(),
            effective_date: "test".into(),
            input_usd_per_million: 4.0,
            cached_input_usd_per_million: 0.4,
            cache_write_input_usd_per_million: 5.0,
            output_usd_per_million: 20.0,
        };
        let cost = estimated_cost(&usage, &schedule).expect("cost");
        assert!((cost - 0.005_38).abs() < f64::EPSILON);
    }
}
