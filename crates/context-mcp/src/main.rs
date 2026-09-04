// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Local stdio MCP process for Impresari Context."]

use context_core::{PolicySubject, ResourceBudget, validate_utc_timestamp};
use context_engine::{EngineConfig, LocalEngine, RequestContext};
use context_mcp::{
    DeliveryMode, McpServer, ServerConfig, StructuralLifecycleReceipt, StructuralRuntime,
    TaskScopedStructure,
};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_structural::WorkerLauncher;
use context_workspace::DiscoveryPolicy;
use std::{
    fs,
    io::{self, BufReader},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn main() {
    if let Err(message) = run() {
        eprintln!("impresari-context-mcp: {message}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), &'static str> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let startup = parse_startup_args(&args)?;
    let seed = unique_seed()?;
    let consumer_id = startup.consumer_id.clone();
    let role = startup.role.clone();
    let context = RequestContext {
        request_id: format!("req_{seed}open"),
        event_id: format!("evt_{seed}open"),
        subject: PolicySubject {
            caller_id: consumer_id.clone(),
            role: role.clone(),
            purpose: "mcp_startup".into(),
        },
        occurred_at: startup.occurred_at,
    };
    let workspace = startup.workspace;
    let cache = startup.cache;
    let structural = startup.structural;
    let config = EngineConfig {
        cache_root: cache.clone(),
        discovery: DiscoveryPolicy::new(100_000, 2_147_483_648, 16_777_216, 64)
            .map_err(|_| "invalid discovery policy")?,
        audit_retention: AuditRetention::new("2026-01-01T00:00:00Z", 100_000, 67_108_864)
            .map_err(|_| "invalid audit policy")?,
    };
    let (mut engine, _) =
        LocalEngine::open(config, &context, &workspace).map_err(|_| "engine open failed")?;
    engine
        .build_snapshot(
            &RequestContext {
                request_id: format!("req_{seed}snapshot"),
                event_id: format!("evt_{seed}snapshot"),
                subject: context.subject.clone(),
                occurred_at: context.occurred_at.clone(),
            },
            ResourceBudget::conservative(
                1_048_576,
                10_000,
                100_000,
                65_536,
                10_000,
                64,
                60_000,
                2_147_483_648,
            )
            .map_err(|_| "invalid budget")?,
        )
        .map_err(|_| "snapshot failed")?;
    let structural_runtime = structural
        .map(|structural| {
            prepare_structural_runtime(&mut engine, &context, &workspace, &cache, structural, &seed)
        })
        .transpose()?;
    let mut server = McpServer::new(
        engine,
        ServerConfig {
            consumer_id,
            role,
            session_policy: SessionPolicy::new(64, 256, 67_108_864)
                .map_err(|_| "invalid session policy")?,
            structural_runtime,
            delivery_mode: startup.delivery_mode,
        },
    );
    server
        .serve(BufReader::new(io::stdin().lock()), &mut io::stdout().lock())
        .map_err(|_| "transport failure")
}

struct StartupArgs {
    workspace: PathBuf,
    cache: PathBuf,
    consumer_id: String,
    role: String,
    occurred_at: String,
    structural: Option<StructuralStartup>,
    delivery_mode: DeliveryMode,
}

struct StructuralStartup {
    worker: PathBuf,
    worker_sha256: String,
    empty_directory: PathBuf,
}

fn parse_startup_args(args: &[String]) -> Result<StartupArgs, &'static str> {
    let valid_prefix = args.len() >= 8
        && args[0] == "--workspace"
        && args[2] == "--cache"
        && args[4] == "--consumer-id"
        && args[6] == "--role";
    if !valid_prefix {
        return Err(
            "usage: --workspace PATH --cache PATH --consumer-id ID --role ROLE [--occurred-at UTC] [--delivery-mode ordinary|eager_structural|progressive_structural] [--structural-worker PATH --structural-worker-sha256 SHA256 --structural-empty-directory PATH]",
        );
    }
    let mut index = 8;
    let occurred_at = if args.get(index).map(String::as_str) == Some("--occurred-at") {
        let value = args.get(index + 1).ok_or("missing --occurred-at value")?;
        index += 2;
        value.clone()
    } else {
        timestamp_now()?
    };
    validate_utc_timestamp(&occurred_at).map_err(|_| "invalid --occurred-at")?;
    let explicit_delivery_mode = if args.get(index).map(String::as_str) == Some("--delivery-mode") {
        let value = args.get(index + 1).ok_or("missing --delivery-mode value")?;
        index += 2;
        Some(match value.as_str() {
            "ordinary" => DeliveryMode::Ordinary,
            "eager_structural" => DeliveryMode::EagerStructural,
            "progressive_structural" => DeliveryMode::ProgressiveStructural,
            _ => return Err("invalid --delivery-mode"),
        })
    } else {
        None
    };
    let structural = if index == args.len() {
        None
    } else if args.len().saturating_sub(index) == 6
        && args[index] == "--structural-worker"
        && args[index + 2] == "--structural-worker-sha256"
        && args[index + 4] == "--structural-empty-directory"
    {
        let worker = PathBuf::from(&args[index + 1]);
        let worker_sha256 = args[index + 3].clone();
        let empty_directory = PathBuf::from(&args[index + 5]);
        if !worker.is_absolute() || !empty_directory.is_absolute() || !valid_sha256(&worker_sha256)
        {
            return Err("invalid structural startup tuple");
        }
        Some(StructuralStartup {
            worker,
            worker_sha256,
            empty_directory,
        })
    } else {
        return Err("invalid structural startup tuple");
    };
    let delivery_mode = explicit_delivery_mode.unwrap_or(if structural.is_some() {
        DeliveryMode::EagerStructural
    } else {
        DeliveryMode::Ordinary
    });
    if matches!(
        delivery_mode,
        DeliveryMode::EagerStructural | DeliveryMode::ProgressiveStructural
    ) != structural.is_some()
    {
        return Err("delivery mode and structural startup tuple must agree");
    }
    Ok(StartupArgs {
        workspace: PathBuf::from(&args[1]),
        cache: PathBuf::from(&args[3]),
        consumer_id: args[5].clone(),
        role: args[7].clone(),
        occurred_at,
        structural,
        delivery_mode,
    })
}

fn prepare_structural_runtime(
    engine: &mut LocalEngine,
    startup_context: &RequestContext,
    workspace: &Path,
    cache: &Path,
    structural: StructuralStartup,
    seed: &str,
) -> Result<StructuralRuntime, &'static str> {
    validate_structural_paths(
        workspace,
        cache,
        &structural.worker,
        &structural.empty_directory,
    )?;
    let launcher = WorkerLauncher {
        executable: structural.worker,
        expected_sha256: structural.worker_sha256.clone(),
        empty_working_directory: structural.empty_directory,
        timeout: Duration::from_secs(5),
    };
    launcher
        .validate()
        .map_err(|_| "invalid structural worker boundary")?;
    let started = Instant::now();
    // Nomination needs this and it reads each admitted file once. Doing it here
    // keeps the cost out of every request's context read budget.
    engine
        .build_identifier_index(&RequestContext {
            request_id: format!("req_{seed}identifier"),
            event_id: format!("evt_{seed}identifier"),
            subject: PolicySubject {
                caller_id: startup_context.subject.caller_id.clone(),
                role: startup_context.subject.role.clone(),
                purpose: "mcp_identifier_index_startup".into(),
            },
            occurred_at: startup_context.occurred_at.clone(),
        })
        .map_err(|_| "identifier index preparation failed")?;
    let graph = engine
        .build_structure(
            &RequestContext {
                request_id: format!("req_{seed}structure"),
                event_id: format!("evt_{seed}structure"),
                subject: PolicySubject {
                    caller_id: startup_context.subject.caller_id.clone(),
                    role: startup_context.subject.role.clone(),
                    purpose: "mcp_structural_startup".into(),
                },
                occurred_at: startup_context.occurred_at.clone(),
            },
            &structural_budget()?,
            &launcher,
        )
        .map_err(|_| "structural preparation failed")?;
    let preparation_elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    Ok(StructuralRuntime {
        receipt: StructuralLifecycleReceipt {
            schema_name: "impresari_context_structural_lifecycle".into(),
            schema_version: "1.0".into(),
            enabled: true,
            state: "prepared".into(),
            graph_id: Some(graph.graph_id.clone()),
            snapshot_id: Some(graph.workspace_snapshot.clone()),
            worker_sha256: Some(structural.worker_sha256),
            graph_completeness: Some(graph.completeness.clone()),
            preparation_elapsed_ms,
        },
        graph,
        edge_kinds: Vec::new(),
        // Density comes from scoping, so the server builds a graph over the
        // files each task nominates and keeps the startup graph as a fallback.
        task_scoped: Some(TaskScopedStructure {
            launcher,
            budget: structural_budget()?,
        }),
    })
}

fn structural_budget() -> Result<ResourceBudget, &'static str> {
    ResourceBudget::conservative(
        1_048_576,
        10_000,
        100_000,
        65_536,
        10_000,
        64,
        60_000,
        2_147_483_648,
    )
    .map_err(|_| "invalid structural budget")
}

fn validate_structural_paths(
    workspace: &Path,
    cache: &Path,
    worker: &Path,
    empty_directory: &Path,
) -> Result<(), &'static str> {
    let workspace = fs::canonicalize(workspace).map_err(|_| "invalid workspace path")?;
    let cache = fs::canonicalize(cache).map_err(|_| "invalid cache path")?;
    let worker = canonical_non_symlink(worker, false)?;
    let empty_directory = canonical_non_symlink(empty_directory, true)?;
    for left in [&worker, &empty_directory] {
        for right in [&workspace, &cache] {
            if paths_overlap(left, right) {
                return Err("structural path overlaps product roots");
            }
        }
    }
    if paths_overlap(&worker, &empty_directory) {
        return Err("structural worker and empty directory overlap");
    }
    Ok(())
}

fn canonical_non_symlink(path: &Path, directory: bool) -> Result<PathBuf, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "structural path is unavailable")?;
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err("invalid structural path kind");
    }
    fs::canonicalize(path).map_err(|_| "invalid structural path")
}

fn paths_overlap(left: &Path, right: &Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unique_seed() -> Result<String, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("mcp{}{}", std::process::id(), duration.as_nanos()))
        .map_err(|_| "system clock precedes Unix epoch")
}

fn timestamp_now() -> Result<String, &'static str> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock precedes Unix epoch")?
        .as_secs();
    let days = i64::try_from(seconds / 86_400).map_err(|_| "system clock is invalid")?;
    let day_seconds = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    ))
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
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    fn arguments() -> Vec<String> {
        [
            "--workspace",
            "workspace",
            "--cache",
            "cache",
            "--consumer-id",
            "consumer",
            "--role",
            "reader",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    /// Distinguishes concurrent fixtures within one test binary.
    ///
    /// `unique_seed` is pid plus nanoseconds, and every test in a binary shares
    /// the pid. Two tests scheduled inside the same clock tick — routine on
    /// Windows, whose timer is coarser — then derive the same root, and each
    /// removes it on the way out, so the other's fixture vanishes mid-test.
    static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn absolute_fixture() -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
        let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "context-mcp-main-test-{}-{sequence}",
            unique_seed().unwrap()
        ));
        let workspace = root.join("workspace");
        let cache = root.join("cache");
        let worker = root.join("worker");
        let empty = root.join("empty");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&cache).unwrap();
        fs::create_dir_all(&empty).unwrap();
        fs::write(&worker, b"pinned worker bytes").unwrap();
        (root, workspace, cache, worker, empty)
    }

    fn structural_arguments(worker: &Path, empty: &Path, digest: &str) -> Vec<String> {
        let mut args = arguments();
        args.extend([
            "--occurred-at".into(),
            "2026-08-23T12:00:00Z".into(),
            "--structural-worker".into(),
            worker.display().to_string(),
            "--structural-worker-sha256".into(),
            digest.into(),
            "--structural-empty-directory".into(),
            empty.display().to_string(),
        ]);
        args
    }

    fn worker_digest() -> String {
        "sha256:2f60225e24d6e3f76bb5eeb373977ad2f5a159bed992f5ca568ebba8d45e3935".into()
    }

    #[test]
    fn startup_arguments_allow_a_local_clock_or_a_deterministic_override() {
        let automatic = parse_startup_args(&arguments()).expect("automatic startup time");
        validate_utc_timestamp(&automatic.occurred_at).expect("valid local clock time");
        assert_eq!(automatic.delivery_mode, DeliveryMode::Ordinary);

        let mut deterministic = arguments();
        deterministic.extend(["--occurred-at".into(), "2026-08-23T12:00:00Z".into()]);
        let parsed = parse_startup_args(&deterministic).expect("deterministic startup time");
        assert_eq!(parsed.occurred_at, "2026-08-23T12:00:00Z");
    }

    #[test]
    fn startup_arguments_reject_unknown_or_invalid_timestamp_forms() {
        let mut invalid = arguments();
        invalid.extend(["--occurred-at".into(), "not-a-timestamp".into()]);
        assert_eq!(
            parse_startup_args(&invalid).err(),
            Some("invalid --occurred-at")
        );
        let mut unknown = arguments();
        unknown.extend(["--unknown".into(), "value".into()]);
        assert!(parse_startup_args(&unknown).is_err());
    }

    #[test]
    fn startup_arguments_accept_only_the_complete_structural_tuple() {
        let (root, _, _, worker, empty) = absolute_fixture();
        let digest = worker_digest();
        let complete = structural_arguments(&worker, &empty, &digest);
        let parsed = parse_startup_args(&complete).expect("complete structural tuple");
        assert_eq!(parsed.delivery_mode, DeliveryMode::EagerStructural);
        let structural = parsed.structural.expect("structural startup");
        assert_eq!(structural.worker, worker);
        assert_eq!(structural.worker_sha256, digest);
        assert_eq!(structural.empty_directory, empty);

        let mut partial = complete.clone();
        partial.truncate(partial.len() - 2);
        assert_eq!(
            parse_startup_args(&partial).err(),
            Some("invalid structural startup tuple")
        );

        let mut noncanonical_digest = complete;
        let digest_index = noncanonical_digest
            .iter()
            .position(|value| value == "--structural-worker-sha256")
            .unwrap()
            + 1;
        noncanonical_digest[digest_index] =
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".into();
        assert_eq!(
            parse_startup_args(&noncanonical_digest).err(),
            Some("invalid structural startup tuple")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn delivery_modes_are_closed_and_require_matching_trusted_startup() {
        let mut ordinary_with_structural_mode = arguments();
        ordinary_with_structural_mode
            .extend(["--delivery-mode".into(), "progressive_structural".into()]);
        assert_eq!(
            parse_startup_args(&ordinary_with_structural_mode).err(),
            Some("delivery mode and structural startup tuple must agree")
        );

        let (root, _, _, worker, empty) = absolute_fixture();
        let mut progressive = arguments();
        progressive.extend([
            "--delivery-mode".into(),
            "progressive_structural".into(),
            "--structural-worker".into(),
            worker.display().to_string(),
            "--structural-worker-sha256".into(),
            worker_digest(),
            "--structural-empty-directory".into(),
            empty.display().to_string(),
        ]);
        assert_eq!(
            parse_startup_args(&progressive)
                .expect("progressive startup")
                .delivery_mode,
            DeliveryMode::ProgressiveStructural
        );

        let mut ordinary = progressive;
        ordinary[9] = "ordinary".into();
        assert_eq!(
            parse_startup_args(&ordinary).err(),
            Some("delivery mode and structural startup tuple must agree")
        );

        let mut unknown = arguments();
        unknown.extend(["--delivery-mode".into(), "adaptive".into()]);
        assert_eq!(
            parse_startup_args(&unknown).err(),
            Some("invalid --delivery-mode")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn structural_paths_must_be_disjoint_from_product_roots() {
        let (root, workspace, cache, worker, empty) = absolute_fixture();
        validate_structural_paths(&workspace, &cache, &worker, &empty)
            .expect("disjoint structural paths");

        let overlapping = workspace.join("structural-empty");
        fs::create_dir(&overlapping).unwrap();
        assert_eq!(
            validate_structural_paths(&workspace, &cache, &worker, &overlapping).err(),
            Some("structural path overlaps product roots")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launcher_preflight_rejects_a_nonempty_empty_directory() {
        let (root, _, _, worker, empty) = absolute_fixture();
        fs::write(empty.join("unexpected"), b"authority leak").unwrap();
        let launcher = WorkerLauncher {
            executable: worker,
            expected_sha256: worker_digest(),
            empty_working_directory: empty,
            timeout: Duration::from_secs(1),
        };
        assert!(launcher.validate().is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn fixed_structural_budget_is_admitted_by_the_core_profile() {
        let budget = structural_budget().expect("admitted structural budget");
        assert_eq!(budget.max_matches, "10000");
        assert_eq!(budget.max_elapsed_ms, "60000");
    }

    #[cfg(unix)]
    #[test]
    fn structural_paths_reject_direct_symlinks() {
        use std::os::unix::fs::symlink;

        let (root, workspace, cache, worker, empty) = absolute_fixture();
        let worker_link = root.join("worker-link");
        symlink(&worker, &worker_link).unwrap();
        assert_eq!(
            validate_structural_paths(&workspace, &cache, &worker_link, &empty).err(),
            Some("invalid structural path kind")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
