// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "OS-shaped reference adapter integration test."]

use std::{fs, path::PathBuf};

use context_adapters::{
    ADAPTER_CONTRACT_VERSION, GUIDED_DELIVERY_CONTRACT_VERSION, GuidedDeliveryIntent,
    GuidedDeliveryReceipt, OsContextRequest, acquire_for_os, prepare_guided_delivery,
};
use context_core::{PolicySubject, ResourceBudget, packet_bytes, validate_packet};
use context_engine::{
    ContextPlanStep, EngineConfig, LocalEngine, ProfiledContextPacket, QueryKind, RequestContext,
    TaskProfile,
};
use context_store::{AuditRetention, AuditStore};
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
    let direct_snapshot = direct
        .build_snapshot(&snapshot, budget())
        .expect("direct snapshot");
    let adapter_snapshot = adapter
        .build_snapshot(&snapshot, budget())
        .expect("adapter snapshot");
    assert_eq!(direct_snapshot, adapter_snapshot);
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
        workspace_identity: adapter_snapshot.workspace_identity.clone(),
        workspace_snapshot: adapter_snapshot.snapshot_id.clone(),
        task_profile: TaskProfile::Implementation,
        query: "authenticate".into(),
        budget: budget(),
    };
    let prepared = prepare_guided_delivery(&mut adapter, intent.clone()).expect("prepared");
    let exact_packet_bytes = prepared.packet_bytes.clone().expect("canonical bytes");
    let packet = prepared.prepared.expect("packet");
    assert_exact_packet_bytes(&exact_packet_bytes, &direct_packet, &packet);
    assert_prepared_receipt(
        &prepared.receipt,
        &context,
        &adapter_snapshot.workspace_identity,
        &packet,
    );

    let mut without_consent = intent;
    without_consent.consent = false;
    let no_delivery = prepare_guided_delivery(&mut adapter, without_consent).expect("no delivery");
    assert!(no_delivery.prepared.is_none());
    assert!(no_delivery.packet_bytes.is_none());
    assert_eq!(no_delivery.receipt.reason_code, "explicit_consent_required");
    assert_eq!(no_delivery.receipt.scope, "process_local");
    assert_eq!(no_delivery.receipt.request_id, context.request_id);
    assert_eq!(no_delivery.receipt.event_id, context.event_id);
    assert!(!no_delivery.receipt.client_io_performed);
}

fn assert_exact_packet_bytes(
    exact_packet_bytes: &[u8],
    direct_packet: &ProfiledContextPacket,
    adapter_packet: &ProfiledContextPacket,
) {
    assert_eq!(
        exact_packet_bytes,
        packet_bytes(&direct_packet.packet).expect("direct canonical bytes")
    );
    assert_eq!(
        exact_packet_bytes,
        packet_bytes(&adapter_packet.packet).expect("adapter canonical bytes")
    );
}

fn assert_prepared_receipt(
    receipt: &GuidedDeliveryReceipt,
    context: &RequestContext,
    workspace_identity: &str,
    packet: &ProfiledContextPacket,
) {
    assert_eq!(receipt.outcome, "prepared");
    assert_eq!(receipt.scope, "process_local");
    assert_eq!(receipt.client_version, GUIDED_DELIVERY_CONTRACT_VERSION);
    assert_eq!(receipt.request_id, context.request_id);
    assert_eq!(receipt.event_id, context.event_id);
    assert_eq!(
        receipt.workspace_identity.as_deref(),
        Some(workspace_identity)
    );
    assert_eq!(
        receipt.packet_id.as_deref(),
        Some(packet.packet.packet_id.as_str())
    );
    assert!(!receipt.client_io_performed);
    assert!(!receipt.authority_added);
}

#[test]
fn guided_delivery_rejects_malformed_intents_before_engine_work() {
    let source = TestRoot::new("guided-invalid-source");
    let cache = TestRoot::new("guided-invalid-cache");
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
        request_id: "req_guidedinvalidopen".into(),
        event_id: "evt_guidedinvalidopen".into(),
        subject: PolicySubject {
            caller_id: "consumer_guidedinvalid".into(),
            role: "local_user".into(),
            purpose: "open".into(),
        },
        occurred_at: "2026-08-22T00:00:01Z".into(),
    };
    let (mut engine, _) = LocalEngine::open(config, &open, &source.0).expect("open");
    let mut malformed = guided_delivery_intent();
    malformed.request_id = "Invalid_12345678".into();
    let rejected = prepare_guided_delivery(&mut engine, malformed).expect("rejected intent");
    assert!(rejected.prepared.is_none());
    assert!(rejected.packet_bytes.is_none());
    assert_eq!(rejected.receipt.reason_code, "invalid_declared_intent");
    assert!(!rejected.receipt.client_io_performed);

    let mut malformed_budget = guided_delivery_intent();
    malformed_budget.budget.requested = "08192".into();
    let rejected_budget =
        prepare_guided_delivery(&mut engine, malformed_budget).expect("rejected budget");
    assert_eq!(
        rejected_budget.receipt.reason_code,
        "invalid_declared_intent"
    );

    let mut unsupported = guided_delivery_intent();
    unsupported.client = "unrecognized".into();
    let rejected_identity =
        prepare_guided_delivery(&mut engine, unsupported).expect("rejected identity");
    assert_eq!(
        rejected_identity.receipt.reason_code,
        "unsupported_client_lifecycle"
    );

    let mut encoded = serde_json::to_value(guided_delivery_intent()).expect("intent JSON");
    encoded
        .as_object_mut()
        .expect("intent object")
        .insert("unrecognized_field".into(), serde_json::Value::Bool(true));
    assert!(serde_json::from_value::<GuidedDeliveryIntent>(encoded).is_err());

    drop(engine);
    let audit = AuditStore::open(&cache.0).expect("open audit");
    let events = audit.recent(10).expect("recent audit events");
    assert_eq!(events.len(), 1, "invalid intents must not reach the engine");
    assert_eq!(events[0].event_id, open.event_id);
}

#[test]
fn guided_delivery_returns_no_delivery_for_a_stale_declared_snapshot() {
    let source = TestRoot::new("guided-stale-source");
    let cache = TestRoot::new("guided-stale-cache");
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
        request_id: "req_guidedstaleopen1".into(),
        event_id: "evt_guidedstaleopen1".into(),
        subject: PolicySubject {
            caller_id: "consumer_guidedstale".into(),
            role: "local_user".into(),
            purpose: "open".into(),
        },
        occurred_at: "2026-08-22T00:00:01Z".into(),
    };
    let (mut engine, _) = LocalEngine::open(config, &open, &source.0).expect("open");
    let snapshot_context = RequestContext {
        request_id: "req_guidedstalesnap1".into(),
        event_id: "evt_guidedstalesnap1".into(),
        subject: open.subject.clone(),
        occurred_at: "2026-08-22T00:00:02Z".into(),
    };
    let snapshot = engine
        .build_snapshot(&snapshot_context, budget())
        .expect("snapshot");
    let mut intent = guided_delivery_intent();
    intent.workspace_identity = snapshot.workspace_identity;
    intent.workspace_snapshot = format!("sha256:{}", "0".repeat(64));
    let event_id = intent.event_id.clone();
    let rejected = prepare_guided_delivery(&mut engine, intent).expect("stale intent");
    assert!(rejected.prepared.is_none());
    assert!(rejected.packet_bytes.is_none());
    assert_eq!(rejected.receipt.outcome, "no_delivery");
    assert_eq!(rejected.receipt.reason_code, "snapshot_stale");
    assert!(!rejected.receipt.client_io_performed);
    assert!(!rejected.receipt.authority_added);

    drop(engine);
    let audit = AuditStore::open(&cache.0).expect("open audit");
    let events = audit.recent(10).expect("recent audit events");
    assert!(
        events.iter().all(|event| event.event_id != event_id),
        "a stale intent must not invoke the planner with its declared event"
    );
}

fn guided_delivery_intent() -> GuidedDeliveryIntent {
    GuidedDeliveryIntent {
        adapter_contract_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
        client: "reference".into(),
        scope: "process_local".into(),
        client_version: GUIDED_DELIVERY_CONTRACT_VERSION.into(),
        lifecycle_point: "prepare".into(),
        consent: true,
        request_id: "req_guidedintent01".into(),
        event_id: "evt_guidedintent01".into(),
        consumer_id: "consumer_guidedintent".into(),
        role: "local_user".into(),
        purpose: "implementation".into(),
        occurred_at: "2026-08-22T00:00:03Z".into(),
        workspace_identity: format!("sha256:{}", "0".repeat(64)),
        workspace_snapshot: format!("sha256:{}", "0".repeat(64)),
        task_profile: TaskProfile::Implementation,
        query: "authenticate".into(),
        budget: budget(),
    }
}
