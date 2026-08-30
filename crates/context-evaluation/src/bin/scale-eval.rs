// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Generated scale and performance evaluation runner."]

use std::{
    fs,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use context_core::{PolicySubject, ResourceBudget};
use context_engine::{EngineConfig, LocalEngine, QueryKind, RequestContext};
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;
use serde::Serialize;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
const SAMPLE_COUNT: usize = 5;

#[derive(Clone, Copy)]
struct Profile {
    id: &'static str,
    files: u64,
    bytes_per_file: usize,
}

#[derive(Serialize)]
struct Percentiles {
    unit: &'static str,
    p50: u64,
    p95: u64,
    maximum: u64,
}

#[derive(Serialize)]
struct ProfileResult {
    id: &'static str,
    generated_files: u64,
    generated_bytes: u64,
    cold_snapshot: Percentiles,
    warm_snapshot: Percentiles,
    cold_lexical_query: Percentiles,
    warm_lexical_query: Percentiles,
    cache_bytes_max: u64,
    cache_to_source_ratio_max: f64,
    partial_limit_visible: bool,
}

#[derive(Serialize)]
struct ScaleReport {
    schema_name: &'static str,
    schema_version: &'static str,
    runner_version: &'static str,
    engine_version: &'static str,
    samples_per_profile: usize,
    platform_os: &'static str,
    platform_arch: &'static str,
    filesystem: &'static str,
    peak_rss_bytes: Option<u64>,
    profiles: Vec<ProfileResult>,
    deviations: Vec<&'static str>,
    failures: Vec<String>,
}

struct TempRoot(PathBuf);
impl TempRoot {
    fn new(label: &str) -> Result<Self, String> {
        let sequence = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "impresari-scale-{label}-{}-{sequence}",
            process::id()
        ));
        fs::create_dir(&path).map_err(|error| error.to_string())?;
        Ok(Self(path))
    }
}
impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn request(profile: usize, sample: usize, phase: &str) -> RequestContext {
    RequestContext {
        request_id: format!("req_scale{profile:02}{sample:02}{phase}"),
        event_id: format!("evt_scale{profile:02}{sample:02}{phase}"),
        subject: PolicySubject {
            caller_id: "caller_scale_eval".into(),
            role: "local_user".into(),
            purpose: "scale_evaluation".into(),
        },
        occurred_at: "2026-08-21T12:00:00Z".into(),
    }
}

fn budget(max_files: u64) -> ResourceBudget {
    ResourceBudget::conservative(
        4_194_304,
        100,
        max_files,
        1024,
        10_000,
        16,
        300_000,
        536_870_912,
    )
    .expect("scale budget")
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos().saturating_add(999_999) / 1_000_000)
        .unwrap_or(u64::MAX)
}

fn percentiles(mut samples: Vec<u64>) -> Percentiles {
    samples.sort_unstable();
    let p50 = samples[(samples.len() - 1) / 2];
    let p95_index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    Percentiles {
        unit: "milliseconds",
        p50,
        p95: samples[p95_index],
        maximum: *samples.last().expect("samples"),
    }
}

fn generate(profile: Profile, root: &Path) -> Result<u64, String> {
    let mut total = 0_u64;
    for index in 0..profile.files {
        let directory = root.join(format!("module-{:02}", index % 32));
        fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let marker = if index % 10 == 0 {
            format!("shared marker profile={} file={index}\n", profile.id)
        } else {
            format!("ordinary content profile={} file={index}\n", profile.id)
        };
        let padding = profile.bytes_per_file.saturating_sub(marker.len());
        let content = format!("{marker}{}", "x".repeat(padding));
        total =
            total.saturating_add(u64::try_from(content.len()).map_err(|error| error.to_string())?);
        fs::write(directory.join(format!("file-{index:05}.txt")), content)
            .map_err(|error| error.to_string())?;
    }
    Ok(total)
}

fn directory_bytes(root: &Path) -> Result<u64, String> {
    let mut total = 0_u64;
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.is_dir() {
            total = total.saturating_add(directory_bytes(&entry.path())?);
        } else if metadata.is_file() {
            total = total.saturating_add(metadata.len());
        }
    }
    Ok(total)
}

#[allow(clippy::too_many_lines)]
fn run_profile(profile_index: usize, profile: Profile) -> Result<ProfileResult, String> {
    let source = TempRoot::new(profile.id)?;
    let source_bytes = generate(profile, &source.0)?;
    let mut cold_snapshot = Vec::new();
    let mut warm_snapshot = Vec::new();
    let mut cold_query = Vec::new();
    let mut warm_query = Vec::new();
    let mut cache_max = 0_u64;
    for sample in 0..SAMPLE_COUNT {
        let cache = TempRoot::new("cache")?;
        let config = EngineConfig {
            cache_root: cache.0.clone(),
            discovery: DiscoveryPolicy::new(
                profile.files + 10,
                source_bytes + 1_048_576,
                u64::try_from(profile.bytes_per_file).map_err(|error| error.to_string())?,
                16,
            )
            .map_err(|error| error.to_string())?,
            audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 1000, 10_485_760)
                .map_err(|error| error.to_string())?,
        };
        let (mut engine, _) =
            LocalEngine::open(config, &request(profile_index, sample, "open"), &source.0)
                .map_err(|error| error.to_string())?;
        let started = Instant::now();
        engine
            .build_snapshot(
                &request(profile_index, sample, "cold"),
                budget(profile.files + 10),
            )
            .map_err(|error| error.to_string())?;
        cold_snapshot.push(elapsed_ms(started));
        let started = Instant::now();
        engine
            .build_snapshot(
                &request(profile_index, sample, "warm"),
                budget(profile.files + 10),
            )
            .map_err(|error| error.to_string())?;
        warm_snapshot.push(elapsed_ms(started));
        let started = Instant::now();
        let cold = engine
            .search(
                &request(profile_index, sample, "queryc"),
                QueryKind::Lexical,
                "shared marker",
                &budget(profile.files + 10),
            )
            .map_err(|error| error.to_string())?;
        cold_query.push(elapsed_ms(started));
        let started = Instant::now();
        let warm = engine
            .search(
                &request(profile_index, sample, "queryw"),
                QueryKind::Lexical,
                "shared marker",
                &budget(profile.files + 10),
            )
            .map_err(|error| error.to_string())?;
        warm_query.push(elapsed_ms(started));
        let expected =
            usize::try_from((profile.files / 10) * 2).map_err(|error| error.to_string())?;
        if cold.matches.len() != expected || warm.matches.len() != cold.matches.len() {
            return Err(format!(
                "{} retrieval count mismatch: expected {expected}, cold {}, warm {}",
                profile.id,
                cold.matches.len(),
                warm.matches.len()
            ));
        }
        cache_max = cache_max.max(directory_bytes(&cache.0)?);
    }
    let cache = TempRoot::new("partial-cache")?;
    let config = EngineConfig {
        cache_root: cache.0.clone(),
        discovery: DiscoveryPolicy::new(
            profile.files + 10,
            source_bytes + 1_048_576,
            u64::try_from(profile.bytes_per_file).map_err(|error| error.to_string())?,
            16,
        )
        .map_err(|error| error.to_string())?,
        audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 100, 1_048_576)
            .map_err(|error| error.to_string())?,
    };
    let (mut engine, _) = LocalEngine::open(config, &request(profile_index, 99, "open"), &source.0)
        .map_err(|error| error.to_string())?;
    let limited = engine
        .build_snapshot(
            &request(profile_index, 99, "limit"),
            budget(profile.files / 2),
        )
        .map_err(|error| error.to_string())?;
    let ratio = f64::from(u32::try_from(cache_max).map_err(|error| error.to_string())?)
        / f64::from(u32::try_from(source_bytes).map_err(|error| error.to_string())?);
    Ok(ProfileResult {
        id: profile.id,
        generated_files: profile.files,
        generated_bytes: source_bytes,
        cold_snapshot: percentiles(cold_snapshot),
        warm_snapshot: percentiles(warm_snapshot),
        cold_lexical_query: percentiles(cold_query),
        warm_lexical_query: percentiles(warm_query),
        cache_bytes_max: cache_max,
        cache_to_source_ratio_max: ratio,
        partial_limit_visible: limited.completeness == "partial"
            && limited
                .skipped
                .iter()
                .any(|item| item.reason == "limit_reached"),
    })
}

fn run() -> Result<ScaleReport, String> {
    let profiles = [
        Profile {
            id: "generated-large",
            files: 2_000,
            bytes_per_file: 1024,
        },
        Profile {
            id: "generated-monorepo",
            files: 5_000,
            bytes_per_file: 1024,
        },
    ];
    let mut results = Vec::new();
    let mut failures = Vec::new();
    for (index, profile) in profiles.into_iter().enumerate() {
        let result = run_profile(index, profile)?;
        if !result.partial_limit_visible {
            failures.push(format!("{} did not report partial limit", result.id));
        }
        results.push(result);
    }
    Ok(ScaleReport {
        schema_name: "scale-evaluation-report",
        schema_version: "1.0.0",
        runner_version: "1.0.0",
        engine_version: "0.2.0",
        samples_per_profile: SAMPLE_COUNT,
        platform_os: std::env::consts::OS,
        platform_arch: std::env::consts::ARCH,
        filesystem: "not_portably_detected",
        peak_rss_bytes: None,
        profiles: results,
        deviations: vec![
            "peak RSS requires the platform wrapper and is not measured by the safe Rust runner",
            "filesystem type is not portably detected without platform-specific integration",
            "generated profiles complement but do not replace the gated public corpus",
        ],
        failures,
    })
}

fn main() {
    match run() {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("report JSON")
            );
            if !report.failures.is_empty() {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("scale evaluation failed: {error}");
            process::exit(1);
        }
    }
}
