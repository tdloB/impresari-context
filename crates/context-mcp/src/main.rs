// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Local stdio MCP process for Impresari Context."]

use context_core::{PolicySubject, ResourceBudget};
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
    if args.len() != 10
        || args[0] != "--workspace"
        || args[2] != "--cache"
        || args[4] != "--consumer-id"
        || args[6] != "--role"
        || args[8] != "--occurred-at"
    {
        return Err(
            "usage: --workspace PATH --cache PATH --consumer-id ID --role ROLE --occurred-at UTC",
        );
    }
    let seed = unique_seed()?;
    let context = RequestContext {
        request_id: format!("req_{seed}open"),
        event_id: format!("evt_{seed}open"),
        subject: PolicySubject {
            caller_id: args[5].clone(),
            role: args[7].clone(),
            purpose: "mcp_startup".into(),
        },
        occurred_at: args[9].clone(),
    };
    let config = EngineConfig {
        cache_root: PathBuf::from(&args[3]),
        discovery: DiscoveryPolicy::new(100_000, 2_147_483_648, 16_777_216, 64)
            .map_err(|_| "invalid discovery policy")?,
        audit_retention: AuditRetention::new("2026-01-01T00:00:00Z", 100_000, 67_108_864)
            .map_err(|_| "invalid audit policy")?,
    };
    let workspace = PathBuf::from(&args[1]);
    let (mut engine, _) =
        LocalEngine::open(config, &context, &workspace).map_err(|_| "engine open failed")?;
    engine
        .build_snapshot(
            &RequestContext {
                request_id: format!("req_{seed}snapshot"),
                event_id: format!("evt_{seed}snapshot"),
                subject: context.subject,
                occurred_at: context.occurred_at,
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
            consumer_id: args[5].clone(),
            role: args[7].clone(),
            session_policy: SessionPolicy::new(64, 256, 67_108_864)
                .map_err(|_| "invalid session policy")?,
        },
    );
    server
        .serve(BufReader::new(io::stdin().lock()), &mut io::stdout().lock())
        .map_err(|_| "transport failure")
}

fn unique_seed() -> Result<String, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("mcp{}{}", std::process::id(), duration.as_nanos()))
        .map_err(|_| "system clock precedes Unix epoch")
}
