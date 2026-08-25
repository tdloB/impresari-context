// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "OS-shaped reference adapter integration test."]

use std::{fs, path::PathBuf};

use context_adapters::{
    ADAPTER_CONTRACT_VERSION, GUIDED_DELIVERY_CONTRACT_VERSION, GuidedDeliveryIntent,
    OsContextRequest, acquire_for_os, prepare_guided_delivery,
};
use context_core::{PolicySubject, ResourceBudget, validate_packet};
use context_engine::{
    ContextPlanStep, EngineConfig, LocalEngine, QueryKind, RequestContext, TaskProfile,
};
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

#[test]
fn guided_delivery_prepares_exact_planner_bytes_or_reports_no_delivery() {
    let source = TestRoot::new("guided-source");
    let direct_cache = TestRoot::new("guided-direct-cache");
    let adapter_cache = TestRoot::new("guided-adapter-cache");
    fs::write(
        source.0.join("authentication.rs"),
        b"pub fn authenticate() {}\n",
    )
    .expect("source");
    let config = |cache_root: PathBuf| EngineConfig {
        cache_root,
        discovery: DiscoveryPolicy::new(10, 1024, 1024, 8).expect("discovery"),
        audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
            .expect("retention"),
    };
    let open = RequestContext {
        request_id: "req_guidedopen01".into(),
        event_id: "evt_guidedopen01".into(),
        subject: PolicySubject {
            caller_id: "consumer_guided01".into(),
            role: "local_user".into(),
            purpose: "open".into(),
        },
        occurred_at: "2026-08-22T00:00:01Z".into(),
    };
    let (mut direct, _) =
        LocalEngine::open(config(direct_cache.0.clone()), &open, &source.0).expect("direct open");
    let (mut adapter, _) =
        LocalEngine::open(config(adapter_cache.0.clone()), &open, &source.0).expect("adapter open");
    let snapshot = RequestContext {
        request_id: "req_guidedsnapshot1".into(),
        event_id: "evt_guidedsnapshot1".into(),
        subject: open.subject.clone(),
        occurred_at: "2026-08-22T00:00:02Z".into(),
    };
    direct
        .build_snapshot(&snapshot, budget())
        .expect("direct snapshot");
    adapter
        .build_snapshot(&snapshot, budget())
        .expect("adapter snapshot");
    let context = RequestContext {
        request_id: "req_guidedpacket01".into(),
        event_id: "evt_guidedpacket01".into(),
        subject: PolicySubject {
            caller_id: "consumer_guided01".into(),
            role: "local_user".into(),
            purpose: "implementation".into(),
        },
        occurred_at: "2026-08-22T00:00:03Z".into(),
    };
    let direct_packet = direct
        .build_profiled_context(
            &context,
            TaskProfile::Implementation,
            "authenticate",
            budget(),
        )
        .expect("direct packet");
    let intent = GuidedDeliveryIntent {
        adapter_contract_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
        client: "reference".into(),
        scope: "process_local".into(),
        client_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
        lifecycle_point: "prepare".into(),
        consent: true,
        request_id: context.request_id.clone(),
        event_id: context.event_id.clone(),
        consumer_id: context.subject.caller_id.clone(),
        role: context.subject.role.clone(),
        purpose: context.subject.purpose.clone(),
        occurred_at: context.occurred_at.clone(),
        task_profile: TaskProfile::Implementation,
        query: "authenticate".into(),
        budget: budget(),
    };
    let prepared = prepare_guided_delivery(&mut adapter, intent.clone()).expect("prepared");
    let packet = prepared.prepared.expect("packet");
    assert_eq!(
        serde_json::to_vec(&packet.packet).expect("adapter bytes"),
        serde_json::to_vec(&direct_packet.packet).expect("direct bytes")
    );
    assert_eq!(prepared.receipt.outcome, "prepared");
    assert_eq!(
        prepared.receipt.packet_id.as_deref(),
        Some(packet.packet.packet_id.as_str())
    );
    assert!(!prepared.receipt.client_io_performed);
    assert!(!prepared.receipt.authority_added);

    let mut without_consent = intent;
    without_consent.consent = false;
    let no_delivery = prepare_guided_delivery(&mut adapter, without_consent).expect("no delivery");
    assert!(no_delivery.prepared.is_none());
    assert_eq!(no_delivery.receipt.reason_code, "explicit_consent_required");
    assert!(!no_delivery.receipt.client_io_performed);
}
