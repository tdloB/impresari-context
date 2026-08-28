//! Safe, reproducible A/B/A evaluation support for agent-context studies.

#![forbid(unsafe_code)]

use context_engine::ContextPlanStep;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SCHEMA_VERSION: &str = "1.0";
const MAX_ADAPTER_STREAM_BYTES: usize = 1024 * 1024;

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
    /// Model identifier recorded for comparison.
    pub model_identifier: String,
    /// Container or runtime image identifier recorded for comparison.
    pub container_image: String,
    /// Frozen normalized UTC timestamp used by deterministic packet operations.
    #[serde(default = "default_operation_timestamp")]
    pub operation_timestamp: String,
    /// Maximum agent turns permitted for each run.
    pub turn_limit: u32,
    /// Human-readable basis for adapter-reported cost estimates.
    pub pricing_basis: String,
    /// Frozen machine-readable token pricing used to verify adapter costs.
    #[serde(default)]
    pub pricing_schedule: PricingSchedule,
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
    /// Frozen pricing schedule used to compute provider cost.
    pub pricing_schedule: PricingSchedule,
    /// Fixed runtime image identifier.
    pub container_image: String,
    /// Frozen normalized UTC timestamp used by packet operations.
    pub operation_timestamp: String,
    /// Hard agent turn limit.
    pub turn_limit: u32,
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
                let record = execute_run(spec, &source_root, task, arm, repetition, sequence)?;
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
            || record.pricing_basis != spec.execution.pricing_basis
            || record.pricing_schedule != spec.execution.pricing_schedule
            || record.container_image != spec.execution.container_image
            || record.operation_timestamp != spec.execution.operation_timestamp
            || record.turn_limit != spec.execution.turn_limit
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
) -> Result<RunRecord, String> {
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
            pricing_schedule: spec.execution.pricing_schedule.clone(),
            container_image: spec.execution.container_image.clone(),
            operation_timestamp: spec.execution.operation_timestamp.clone(),
            turn_limit: spec.execution.turn_limit,
            packet: None,
        };
        let (response, elapsed) =
            execute_adapter::<PacketResponse, _>(spec, &spec.packet_command, &request, false)?;
        if response.source_fingerprint_sha256 != fingerprint_before {
            return Err(format!(
                "packet adapter returned a mismatched source fingerprint for {}",
                task.id
            ));
        }
        validate_usage(&response.usage, &spec.execution.pricing_schedule)?;
        packet_generation_millis = elapsed;
        packet_bytes =
            u64::try_from(response.packet.len()).map_err(|_| "packet too large".to_owned())?;
        if packet_bytes > u64::try_from(spec.max_stdout_bytes).unwrap_or(u64::MAX) {
            return Err(format!(
                "packet is larger than {} bytes",
                spec.max_stdout_bytes
            ));
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
        pricing_schedule: spec.execution.pricing_schedule.clone(),
        container_image: spec.execution.container_image.clone(),
        operation_timestamp: spec.execution.operation_timestamp.clone(),
        turn_limit: spec.execution.turn_limit,
        packet,
    };
    let (response, agent_wall_clock_millis) =
        execute_adapter::<AgentResponse, _>(spec, &spec.agent_command, &request, true)?;
    if response.source_fingerprint_sha256 != fingerprint_before {
        return Err(format!(
            "agent adapter returned a mismatched source fingerprint for {}",
            task.id
        ));
    }
    validate_usage(&response.usage, &spec.execution.pricing_schedule)?;
    let correctness = answer_is_correct(task, &response.answer);
    validate_returned_evidence(source_root, &spec.source_files, &response.evidence)?;
    let evidence_verified = verify_expected_evidence(task, &response.evidence).is_ok();
    if source_fingerprint(spec, source_root)? != fingerprint_before {
        return Err(format!("evaluated source changed during run {sequence}"));
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
        pricing_basis: spec.execution.pricing_basis.clone(),
        pricing_schedule: spec.execution.pricing_schedule.clone(),
        container_image: spec.execution.container_image.clone(),
        operation_timestamp: spec.execution.operation_timestamp.clone(),
        turn_limit: spec.execution.turn_limit,
        command_timeout_seconds: spec.command_timeout_seconds,
        max_stdout_bytes: spec.max_stdout_bytes,
        max_stderr_bytes: spec.max_stderr_bytes,
        packet_generation_millis,
        packet_bytes,
        packet_sha256,
        packet_usage,
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
    })
}

fn execute_adapter<T: DeserializeOwned, R: Serialize>(
    spec: &StudySpec,
    command: &[String],
    request: &R,
    include_agent_secrets: bool,
) -> Result<(T, u64), String> {
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
    let mut child = configured
        .spawn()
        .map_err(|error| format!("start adapter {:?}: {error}", command[0]))?;
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
    let limit = Duration::from_secs(spec.command_timeout_seconds);
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("check adapter status: {error}"))?
            .is_some()
        {
            break;
        }
        if started.elapsed() > limit {
            child
                .kill()
                .map_err(|error| format!("stop timed-out adapter: {error}"))?;
            let _ = child.wait();
            return Err(format!(
                "adapter exceeded {} seconds",
                spec.command_timeout_seconds
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child
        .wait_with_output()
        .map_err(|error| format!("collect adapter output: {error}"))?;
    if output.stdout.len() > spec.max_stdout_bytes {
        return Err(format!(
            "adapter stdout exceeded {} bytes",
            spec.max_stdout_bytes
        ));
    }
    if output.stderr.len() > spec.max_stderr_bytes {
        return Err(format!(
            "adapter stderr exceeded {} bytes",
            spec.max_stderr_bytes
        ));
    }
    if !output.status.success() {
        return Err(format!(
            "adapter exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let response = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("parse adapter response: {error}"))?;
    Ok((response, duration_millis(started.elapsed())))
}

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
            || task.expected_answer_fragments.is_empty()
            || task.required_evidence.is_empty()
            || task.required_evidence.len() > 32
        {
            return Err(format!("task {:?} is incomplete or duplicated", task.id));
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
