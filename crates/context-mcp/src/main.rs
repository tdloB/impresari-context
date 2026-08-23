// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Local stdio MCP process for Impresari Context."]

use context_core::{PolicySubject, ResourceBudget, validate_utc_timestamp};
use context_engine::{EngineConfig, LocalEngine, RequestContext};
use context_mcp::{McpServer, ServerConfig};
use context_session::SessionPolicy;
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;
use std::{
    io::{self, BufReader},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
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
    let config = EngineConfig {
        cache_root: startup.cache,
        discovery: DiscoveryPolicy::new(100_000, 2_147_483_648, 16_777_216, 64)
            .map_err(|_| "invalid discovery policy")?,
        audit_retention: AuditRetention::new("2026-01-01T00:00:00Z", 100_000, 67_108_864)
            .map_err(|_| "invalid audit policy")?,
    };
    let workspace = startup.workspace;
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
    let mut server = McpServer::new(
        engine,
        ServerConfig {
            consumer_id,
            role,
            session_policy: SessionPolicy::new(64, 256, 67_108_864)
                .map_err(|_| "invalid session policy")?,
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
}

fn parse_startup_args(args: &[String]) -> Result<StartupArgs, &'static str> {
    let valid_prefix = (args.len() == 8 || args.len() == 10)
        && args[0] == "--workspace"
        && args[2] == "--cache"
        && args[4] == "--consumer-id"
        && args[6] == "--role";
    if !valid_prefix || (args.len() == 10 && args[8] != "--occurred-at") {
        return Err(
            "usage: --workspace PATH --cache PATH --consumer-id ID --role ROLE [--occurred-at UTC]",
        );
    }
    let occurred_at = if args.len() == 10 {
        args[9].clone()
    } else {
        timestamp_now()?
    };
    validate_utc_timestamp(&occurred_at).map_err(|_| "invalid --occurred-at")?;
    Ok(StartupArgs {
        workspace: PathBuf::from(&args[1]),
        cache: PathBuf::from(&args[3]),
        consumer_id: args[5].clone(),
        role: args[7].clone(),
        occurred_at,
    })
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

    #[test]
    fn startup_arguments_allow_a_local_clock_or_a_deterministic_override() {
        let automatic = parse_startup_args(&arguments()).expect("automatic startup time");
        validate_utc_timestamp(&automatic.occurred_at).expect("valid local clock time");

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
}
