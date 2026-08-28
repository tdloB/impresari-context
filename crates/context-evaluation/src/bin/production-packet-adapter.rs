//! Production Impresari Context packet adapter for agent A/B/A studies.

#![forbid(unsafe_code)]

use context_core::{PolicySubject, ResourceBudget};
use context_engine::{ContextPlan, EngineConfig, LocalEngine, RequestContext};
use context_evaluation::agent_eval::{AdapterRequest, PacketResponse, Usage};
use context_evaluation::production_adapter::{materialize_source, source_fingerprint};
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;
use std::fs;
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_REQUEST_BYTES: u64 = 512 * 1024;

fn main() {
    if let Err(error) = run() {
        eprintln!("production packet adapter: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut input = Vec::new();
    std::io::stdin()
        .take(MAX_REQUEST_BYTES)
        .read_to_end(&mut input)
        .map_err(|error| format!("read request: {error}"))?;
    let request: AdapterRequest =
        serde_json::from_slice(&input).map_err(|error| format!("parse request: {error}"))?;
    if request.context_plan.is_empty() {
        return Err("treatment task requires a frozen context plan".to_owned());
    }
    if request.packet.is_some() {
        return Err("packet adapter request must not contain a packet".to_owned());
    }
    let original_root = Path::new(&request.workspace_root);
    let observed = source_fingerprint(original_root, &request.source_files)?;
    if observed != request.source_fingerprint_sha256 {
        return Err("source fingerprint changed before packet generation".to_owned());
    }

    let temporary = TemporaryDirectory::new()?;
    let isolated_source = temporary.path().join("source");
    let cache = temporary.path().join("cache");
    materialize_source(original_root, &request.source_files, &isolated_source)?;
    if source_fingerprint(&isolated_source, &request.source_files)? != observed {
        return Err("isolated source fingerprint mismatch".to_owned());
    }

    let mut engine = LocalEngine::open(
        engine_config(&cache)?,
        &request_context(
            "req_eval_packet_open",
            "evt_eval_packet_open",
            "packet_open",
            &request.operation_timestamp,
        ),
        &isolated_source,
    )
    .map_err(|error| format!("open context engine: {error}"))?
    .0;
    engine
        .build_snapshot(
            &request_context(
                "req_eval_packet_snapshot",
                "evt_eval_packet_snapshot",
                "packet_snapshot",
                &request.operation_timestamp,
            ),
            resource_budget(),
        )
        .map_err(|error| format!("build context snapshot: {error}"))?;
    let packet = engine
        .build_planned_context(
            &request_context(
                "req_eval_packet_build",
                "evt_eval_packet_build",
                "agent_context_evaluation",
                &request.operation_timestamp,
            ),
            &ContextPlan {
                steps: request.context_plan,
            },
            resource_budget(),
        )
        .map_err(|error| format!("build context packet: {error}"))?;
    let response = PacketResponse {
        packet: serde_json::to_string(&packet)
            .map_err(|error| format!("serialize context packet: {error}"))?,
        usage: Usage::default(),
        source_fingerprint_sha256: observed,
    };
    let bytes =
        serde_json::to_vec(&response).map_err(|error| format!("serialize response: {error}"))?;
    std::io::stdout()
        .write_all(&bytes)
        .map_err(|error| format!("write response: {error}"))
}

fn engine_config(cache: &Path) -> Result<EngineConfig, String> {
    Ok(EngineConfig {
        cache_root: cache.to_owned(),
        discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32)
            .map_err(|_| "create packet discovery policy".to_owned())?,
        audit_retention: AuditRetention::new("1970-01-01T00:00:00Z", 10_000, 10_485_760)
            .map_err(|_| "create packet audit policy".to_owned())?,
    })
}

fn resource_budget() -> ResourceBudget {
    ResourceBudget::conservative(65_536, 100, 10_000, 4096, 1000, 32, 30_000, 536_870_912)
        .expect("fixed packet budget is valid")
}

fn request_context(
    request_id: &str,
    event_id: &str,
    purpose: &str,
    operation_timestamp: &str,
) -> RequestContext {
    RequestContext {
        request_id: request_id.to_owned(),
        event_id: event_id.to_owned(),
        subject: PolicySubject {
            caller_id: "agent_evaluation_packet_adapter".into(),
            role: "local_evaluator".into(),
            purpose: purpose.to_owned(),
        },
        occurred_at: operation_timestamp.into(),
    }
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "system clock precedes Unix epoch".to_owned())?
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "impresari-agent-eval-packet-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|error| format!("create packet workspace: {error}"))?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
