// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Thin command-line adapter over the shared Impresari Context engine."]

use std::{
    fs,
    io::{self, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use context_core::{
    Capability, ContextPacket, ErrorEnvelope, EvidenceRecord, PolicySubject, PublicErrorCode,
    RecoveryAction, ResourceBudget, error_envelope,
};
use context_engine::{
    EngineConfig, EngineError, LocalEngine, QueryKind, RequestContext, SnapshotStatus,
};
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;
use serde::Serialize;

const HELP: &str = "\
Impresari Context (working name)\n\
Usage:\n\
  impresari-context [global-options] workspace open <root> <cache-root>\n\
  impresari-context [global-options] snapshot build <root> <cache-root>\n\
  impresari-context [global-options] snapshot status <root> <cache-root> <expected-snapshot>\n\
  impresari-context [global-options] search <root> <cache-root> <exact_path|filename|literal|lexical> <query>\n\
  impresari-context [global-options] context build <root> <cache-root> <kind> <query> <purpose>\n\
  impresari-context [global-options] evidence expand <root> <cache-root> <evidence-json> <before> <after> <max>\n\
  impresari-context [global-options] packet validate <root> <cache-root> <packet-json>\n\
  impresari-context [global-options] handoff export <root> <cache-root> <packet-json> <export-root> <filename>\n\
Global options:\n\
  --human                 Add a concise diagnostic to stderr.\n\
  --at <UTC>              Deterministic RFC3339 operation time.\n\
  --cutoff <UTC>          Explicit audit retention cutoff.\n\
  --id-seed <8-64 chars>  Deterministic request/event identifier seed.\n\
  --help                  Show this help.\n";

#[derive(Debug)]
struct GlobalOptions {
    human: bool,
    at: String,
    cutoff: String,
    id_seed: String,
    command: Vec<String>,
}

struct ContextSequence {
    next: u64,
    seed: String,
    at: String,
}

impl ContextSequence {
    fn next(&mut self, purpose: &str) -> RequestContext {
        self.next += 1;
        let suffix = format!("{}{:02}", self.seed, self.next);
        RequestContext {
            request_id: format!("req_{suffix}"),
            event_id: format!("evt_{suffix}"),
            subject: PolicySubject {
                caller_id: "caller_local_cli".into(),
                role: "local_user".into(),
                purpose: purpose.into(),
            },
            occurred_at: self.at.clone(),
        }
    }
}

/// Executes one CLI invocation with injectable output streams.
///
/// Machine-readable success or error JSON is written to stdout. Optional human
/// diagnostics are written only to stderr. The return value is a process code.
pub fn execute(arguments: &[String], stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    if arguments.iter().any(|argument| argument == "--help") {
        let _ = stderr.write_all(HELP.as_bytes());
        return 0;
    }
    let options = match parse_globals(arguments) {
        Ok(options) => options,
        Err(message) => {
            return emit_parse_error(stdout, stderr, &message);
        }
    };
    let mut contexts = ContextSequence {
        next: 0,
        seed: options.id_seed.clone(),
        at: options.at.clone(),
    };
    let result = dispatch(&options, &mut contexts);
    match result {
        Ok(output) => {
            if write_json(stdout, &output.value).is_err() {
                return 74;
            }
            if options.human {
                let _ = writeln!(stderr, "{} completed", output.label);
            }
            0
        }
        Err(error) => {
            if write_json(stdout, error.envelope()).is_err() {
                return 74;
            }
            if options.human {
                let _ = writeln!(stderr, "{}", error.envelope().message);
            }
            1
        }
    }
}

struct Output {
    label: &'static str,
    value: serde_json::Value,
}

impl Output {
    fn new(label: &'static str, value: &impl Serialize) -> Result<Self, EngineError> {
        let value = serde_json::to_value(value).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InternalFailure,
                "response serialization failed",
            )
        })?;
        Ok(Self { label, value })
    }
}

#[allow(clippy::too_many_lines)]
fn dispatch(
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<Output, EngineError> {
    match options
        .command
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["workspace", "open", root, cache] => {
            let (_, handle) = open_engine(root, cache, options, contexts)?;
            Output::new("workspace open", &handle)
        }
        ["snapshot", "build", root, cache] => {
            let (mut engine, _) = open_engine(root, cache, options, contexts)?;
            let status =
                engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
            Output::new("snapshot build", &status)
        }
        ["snapshot", "status", root, cache, expected] => {
            let (mut engine, _) = open_engine(root, cache, options, contexts)?;
            let _ = engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
            let status = engine.snapshot_status_against(
                &contexts.next("snapshot_status"),
                default_budget(),
                Some(expected),
            )?;
            Output::new("snapshot status", &status)
        }
        ["search", root, cache, kind, query] => {
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.search(
                &contexts.next("search"),
                parse_kind(kind)?,
                query,
                &default_budget(),
            )?;
            Output::new("search", &result)
        }
        ["context", "build", root, cache, kind, query, purpose] => {
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.build_context(
                &contexts.next(purpose),
                parse_kind(kind)?,
                query,
                default_budget(),
            )?;
            Output::new("context build", &result)
        }
        [
            "evidence",
            "expand",
            root,
            cache,
            evidence_path,
            before,
            after,
            maximum,
        ] => {
            let evidence: EvidenceRecord =
                read_json(Path::new(evidence_path), Capability::EvidenceExpand)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.expand_evidence(
                &contexts.next("evidence_recovery"),
                &evidence,
                parse_u64(before, Capability::EvidenceExpand)?,
                parse_u64(after, Capability::EvidenceExpand)?,
                parse_u64(maximum, Capability::EvidenceExpand)?,
                default_budget(),
            )?;
            Output::new("evidence expand", &result)
        }
        ["packet", "validate", root, cache, packet_path] => {
            let packet: ContextPacket =
                read_json(Path::new(packet_path), Capability::ContextValidate)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.validate_context_packet(
                &contexts.next("packet_validation"),
                &packet,
                default_budget(),
            )?;
            Output::new("packet validate", &result)
        }
        [
            "handoff",
            "export",
            root,
            cache,
            packet_path,
            export_root,
            filename,
        ] => {
            let packet: ContextPacket =
                read_json(Path::new(packet_path), Capability::HandoffExport)?;
            let (mut engine, _) = prepared_engine(root, cache, options, contexts)?;
            let result = engine.export_handoff(
                &contexts.next("handoff"),
                &packet,
                &default_budget(),
                Path::new(export_root),
                filename,
            )?;
            Output::new("handoff export", &result)
        }
        _ => Err(synthetic_error(
            Capability::WorkspaceOpen,
            PublicErrorCode::InvalidInput,
            "invalid command shape; use --help",
        )),
    }
}

fn open_engine(
    root: &str,
    cache: &str,
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<(LocalEngine, context_engine::WorkspaceHandle), EngineError> {
    LocalEngine::open(
        config(Path::new(cache), &options.cutoff)?,
        &contexts.next("workspace_open"),
        Path::new(root),
    )
}

fn prepared_engine(
    root: &str,
    cache: &str,
    options: &GlobalOptions,
    contexts: &mut ContextSequence,
) -> Result<(LocalEngine, SnapshotStatus), EngineError> {
    let (mut engine, _) = open_engine(root, cache, options, contexts)?;
    let status = engine.build_snapshot(&contexts.next("snapshot_build"), default_budget())?;
    Ok((engine, status))
}

fn config(cache: &Path, cutoff: &str) -> Result<EngineConfig, EngineError> {
    Ok(EngineConfig {
        cache_root: cache.to_owned(),
        discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InternalFailure,
                "default discovery policy is invalid",
            )
        })?,
        audit_retention: AuditRetention::new(cutoff, 10_000, 10_485_760).map_err(|_| {
            synthetic_error(
                Capability::WorkspaceOpen,
                PublicErrorCode::InvalidInput,
                "invalid audit retention cutoff",
            )
        })?,
    })
}

fn default_budget() -> ResourceBudget {
    ResourceBudget::conservative(65_536, 100, 10_000, 4096, 1000, 32, 30_000, 536_870_912)
        .expect("versioned default budget")
}

fn parse_kind(value: &str) -> Result<QueryKind, EngineError> {
    match value {
        "exact_path" => Ok(QueryKind::ExactPath),
        "filename" => Ok(QueryKind::Filename),
        "literal" => Ok(QueryKind::Literal),
        "lexical" => Ok(QueryKind::Lexical),
        _ => Err(synthetic_error(
            Capability::CodeSearch,
            PublicErrorCode::InvalidInput,
            "unsupported query kind",
        )),
    }
}

fn parse_u64(value: &str, capability: Capability) -> Result<u64, EngineError> {
    value.parse().map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "invalid numeric argument",
        )
    })
}

fn read_json<T: serde::de::DeserializeOwned>(
    path: &Path,
    capability: Capability,
) -> Result<T, EngineError> {
    let metadata = fs::metadata(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::PathNotFound,
            "input file not found",
        )
    })?;
    if !metadata.is_file() || metadata.len() > 4_194_304 {
        return Err(synthetic_error(
            capability,
            PublicErrorCode::ResourceLimit,
            "input file is not a bounded regular file",
        ));
    }
    let bytes = fs::read(path).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InternalFailure,
            "input read failed",
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|_| {
        synthetic_error(
            capability,
            PublicErrorCode::InvalidInput,
            "input JSON is invalid",
        )
    })
}

fn parse_globals(arguments: &[String]) -> Result<GlobalOptions, String> {
    let now = unix_seconds()?;
    let mut options = GlobalOptions {
        human: false,
        at: timestamp(now),
        cutoff: timestamp(now.saturating_sub(7 * 24 * 60 * 60)),
        id_seed: unique_seed()?,
        command: Vec::new(),
    };
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--human" => options.human = true,
            "--at" | "--cutoff" | "--id-seed" => {
                let flag = arguments[index].as_str();
                index += 1;
                let value = arguments
                    .get(index)
                    .ok_or_else(|| format!("missing value for {flag}"))?;
                match flag {
                    "--at" => options.at.clone_from(value),
                    "--cutoff" => options.cutoff.clone_from(value),
                    "--id-seed" => options.id_seed.clone_from(value),
                    _ => unreachable!(),
                }
            }
            value if value.starts_with('-') => return Err("unknown global option".into()),
            _ => options.command.push(arguments[index].clone()),
        }
        index += 1;
    }
    if options.id_seed.len() < 8
        || options.id_seed.len() > 64
        || !options
            .id_seed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err("invalid --id-seed".into());
    }
    context_core::validate_utc_timestamp(&options.at).map_err(|_| "invalid --at".to_owned())?;
    context_core::validate_utc_timestamp(&options.cutoff)
        .map_err(|_| "invalid --cutoff".to_owned())?;
    Ok(options)
}

fn synthetic_error(capability: Capability, code: PublicErrorCode, message: &str) -> EngineError {
    let envelope = error_envelope(
        code,
        message,
        false,
        capability,
        "req_clierror0",
        None,
        None,
        false,
        Some(RecoveryAction::None),
    )
    .expect("constant CLI error");
    engine_error(envelope)
}

fn engine_error(envelope: ErrorEnvelope) -> EngineError {
    // The engine intentionally owns construction. Round-trip through a tiny
    // local operation is avoided by exposing this crate-private adapter below.
    context_engine::adapter_error(envelope)
}

fn emit_parse_error(stdout: &mut dyn Write, stderr: &mut dyn Write, message: &str) -> i32 {
    let error = synthetic_error(
        Capability::WorkspaceOpen,
        PublicErrorCode::InvalidInput,
        "invalid command-line arguments",
    );
    let _ = write_json(stdout, error.envelope());
    let _ = writeln!(stderr, "{message}; use --help");
    2
}

fn write_json(output: &mut dyn Write, value: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *output, value)?;
    output.write_all(b"\n")
}

fn unix_seconds() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "system clock precedes Unix epoch".into())
}

fn unique_seed() -> Result<String, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("{}{}", std::process::id(), duration.as_nanos()))
        .map_err(|_| "system clock precedes Unix epoch".into())
}

fn timestamp(seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

fn civil_from_days(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

    struct TestRoot(PathBuf);
    impl TestRoot {
        fn new(label: &str) -> Self {
            let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-cli-{label}-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create root");
            Self(path)
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn direct_context(sequence: u64, purpose: &str) -> RequestContext {
        RequestContext {
            request_id: format!("req_abcdefgh{sequence:02}"),
            event_id: format!("evt_abcdefgh{sequence:02}"),
            subject: PolicySubject {
                caller_id: "caller_local_cli".into(),
                role: "local_user".into(),
                purpose: purpose.into(),
            },
            occurred_at: "2026-08-21T12:00:00Z".into(),
        }
    }

    fn invoke(command: &[String], seed: &str) -> (i32, serde_json::Value) {
        let mut arguments = vec![
            "--at".into(),
            "2026-08-21T12:00:00Z".into(),
            "--cutoff".into(),
            "2026-08-14T12:00:00Z".into(),
            "--id-seed".into(),
            seed.into(),
        ];
        arguments.extend_from_slice(command);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let code = execute(&arguments, &mut stdout, &mut stderr);
        assert!(stderr.is_empty());
        let value = serde_json::from_slice(&stdout).expect("machine JSON");
        (code, value)
    }

    #[test]
    fn cli_search_is_semantically_identical_to_direct_library_use() {
        let source = TestRoot::new("source");
        let cli_cache = TestRoot::new("cli-cache");
        let library_cache = TestRoot::new("library-cache");
        fs::write(source.0.join("sample.rs"), b"fn alpha() { beta(); }\n").expect("source");
        let arguments = vec![
            "--at".into(),
            "2026-08-21T12:00:00Z".into(),
            "--cutoff".into(),
            "2026-08-14T12:00:00Z".into(),
            "--id-seed".into(),
            "abcdefgh".into(),
            "--human".into(),
            "search".into(),
            source.0.to_string_lossy().into_owned(),
            cli_cache.0.to_string_lossy().into_owned(),
            "literal".into(),
            "beta".into(),
        ];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&arguments, &mut stdout, &mut stderr), 0);
        let cli_value: serde_json::Value = serde_json::from_slice(&stdout).expect("CLI JSON");
        assert_eq!(
            String::from_utf8(stderr).expect("stderr"),
            "search completed\n"
        );

        let direct_config = EngineConfig {
            cache_root: library_cache.0.clone(),
            discovery: DiscoveryPolicy::new(10_000, 536_870_912, 1_048_576, 32).expect("discovery"),
            audit_retention: AuditRetention::new("2026-08-14T12:00:00Z", 10_000, 10_485_760)
                .expect("retention"),
        };
        let (mut engine, _) = LocalEngine::open(
            direct_config,
            &direct_context(1, "workspace_open"),
            &source.0,
        )
        .expect("direct open");
        engine
            .build_snapshot(&direct_context(2, "snapshot_build"), default_budget())
            .expect("direct snapshot");
        let direct = engine
            .search(
                &direct_context(3, "search"),
                QueryKind::Literal,
                "beta",
                &default_budget(),
            )
            .expect("direct search");
        assert_eq!(
            cli_value,
            serde_json::to_value(direct).expect("direct JSON")
        );
    }

    #[test]
    fn parse_errors_are_machine_readable_and_clock_conversion_is_stable() {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(execute(&["unknown".into()], &mut stdout, &mut stderr), 1);
        let envelope: ErrorEnvelope = serde_json::from_slice(&stdout).expect("error envelope");
        assert_eq!(envelope.code, PublicErrorCode::InvalidInput);
        assert_eq!(timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(timestamp(1_704_067_200), "2024-01-01T00:00:00Z");
    }

    #[test]
    fn cli_packet_recovery_validation_and_handoff_lifecycle_is_complete() {
        let source = TestRoot::new("lifecycle-source");
        let cache = TestRoot::new("lifecycle-cache");
        let export = TestRoot::new("lifecycle-export");
        fs::write(source.0.join("lib.rs"), b"pub fn verified() {}\n").expect("source");
        let source_arg = source.0.to_string_lossy().into_owned();
        let cache_arg = cache.0.to_string_lossy().into_owned();
        let (code, packet_value) = invoke(
            &[
                "context".into(),
                "build".into(),
                source_arg.clone(),
                cache_arg.clone(),
                "literal".into(),
                "verified".into(),
                "review".into(),
            ],
            "lifecyclea",
        );
        assert_eq!(code, 0);
        let packet: ContextPacket = serde_json::from_value(packet_value).expect("packet");
        let packet_path = cache.0.join("packet-input.json");
        fs::write(
            &packet_path,
            serde_json::to_vec(&packet).expect("packet JSON"),
        )
        .expect("packet input");
        let evidence_path = cache.0.join("evidence-input.json");
        fs::write(
            &evidence_path,
            serde_json::to_vec(&packet.observed_evidence[0]).expect("evidence JSON"),
        )
        .expect("evidence input");

        let (code, validation) = invoke(
            &[
                "packet".into(),
                "validate".into(),
                source_arg.clone(),
                cache_arg.clone(),
                packet_path.to_string_lossy().into_owned(),
            ],
            "lifecycleb",
        );
        assert_eq!(code, 0);
        assert_eq!(validation["status"], "valid_current");
        let (code, expanded) = invoke(
            &[
                "evidence".into(),
                "expand".into(),
                source_arg.clone(),
                cache_arg.clone(),
                evidence_path.to_string_lossy().into_owned(),
                "2".into(),
                "2".into(),
                "32".into(),
            ],
            "lifecyclec",
        );
        assert_eq!(code, 0);
        assert_eq!(
            expanded["evidence_id"],
            packet.observed_evidence[0].evidence_id
        );
        let (code, receipt) = invoke(
            &[
                "handoff".into(),
                "export".into(),
                source_arg,
                cache_arg,
                packet_path.to_string_lossy().into_owned(),
                export.0.to_string_lossy().into_owned(),
                "handoff.json".into(),
            ],
            "lifecycled",
        );
        assert_eq!(code, 0);
        assert_eq!(receipt["packet_id"], packet.packet_id);
        assert!(export.0.join("handoff.json").is_file());
    }
}
