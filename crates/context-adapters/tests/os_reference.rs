// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "OS-shaped reference adapter integration test."]

use std::{fs, path::PathBuf};

use context_adapters::{ADAPTER_CONTRACT_VERSION, OsContextRequest, acquire_for_os};
use context_core::{ResourceBudget, validate_packet};
use context_engine::{ContextPlanStep, EngineConfig, LocalEngine, QueryKind, RequestContext};
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "impresari-os-adapter-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).expect("create root");
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn budget() -> ResourceBudget {
    ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 67_108_864).expect("budget")
}

#[test]
fn os_adapter_translates_but_does_not_reimplement_the_engine() {
    let source = TestRoot::new("source");
    let cache = TestRoot::new("cache");
    fs::write(
        source.0.join("authentication.rs"),
        b"pub fn authenticate() {}\n",
    )
    .expect("source");
    let config = EngineConfig {
        cache_root: cache.0.clone(),
        discovery: DiscoveryPolicy::new(10, 1024, 1024, 8).expect("discovery"),
        audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
            .expect("retention"),
    };
    let open = RequestContext {
        request_id: "req_osopen0001".into(),
        event_id: "evt_osopen0001".into(),
        subject: context_core::PolicySubject {
            caller_id: "consumer_osadapter01".into(),
            role: "orchestrator".into(),
            purpose: "open".into(),
        },
        occurred_at: "2026-08-22T00:00:01Z".into(),
    };
    let (mut engine, _) = LocalEngine::open(config, &open, &source.0).expect("open");
    let snapshot_context = RequestContext {
        request_id: "req_ossnapshot01".into(),
        event_id: "evt_ossnapshot01".into(),
        subject: open.subject.clone(),
        occurred_at: "2026-08-22T00:00:02Z".into(),
    };
    engine
        .build_snapshot(&snapshot_context, budget())
        .expect("snapshot");
    let response = acquire_for_os(
        &mut engine,
        OsContextRequest {
            adapter_contract_version: ADAPTER_CONTRACT_VERSION.into(),
            request_id: "req_oscontext001".into(),
            event_id: "evt_oscontext001".into(),
            consumer_id: "consumer_osadapter01".into(),
            role: "orchestrator".into(),
            purpose: "implementation_review".into(),
            occurred_at: "2026-08-22T00:00:03Z".into(),
            steps: vec![ContextPlanStep {
                kind: QueryKind::Literal,
                query: "authenticate".into(),
            }],
            budget: budget(),
        },
    )
    .expect("adapter response");
    validate_packet(&response.packet).expect("valid packet");
    assert!(!response.packet.observed_evidence.is_empty());
    assert!(!response.orchestration_authority_added);
}
