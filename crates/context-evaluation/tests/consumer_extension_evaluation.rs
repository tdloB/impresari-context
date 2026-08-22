// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Frozen consumer-adapter equivalence and extension-quarantine release gates."]

use std::{fs, path::PathBuf};

use context_adapters::{ADAPTER_CONTRACT_VERSION, OsContextRequest, acquire_for_os};
use context_core::{PolicySubject, ResourceBudget, validate_packet};
use context_engine::{
    ContextPlan, ContextPlanStep, EngineConfig, LocalEngine, QueryKind, RequestContext,
};
use context_extensions::{
    CapabilityRequest, EXTENSION_CONTRACT_VERSION, ExtensionKind, ExtensionManifest,
    ExtensionOutput, ExtensionPolicy, NormalizationVerdict, RequestedCapabilities,
    normalize_output,
};
use context_store::AuditRetention;
use context_workspace::DiscoveryPolicy;

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "impresari-integration-eval-{label}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).expect("create evaluation root");
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn budget() -> ResourceBudget {
    ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 67_108_864)
        .expect("evaluation budget")
}

fn engine(source: &TestRoot, cache: &TestRoot) -> LocalEngine {
    let open = RequestContext {
        request_id: "req_evaladapteropen".into(),
        event_id: "evt_evaladapteropen".into(),
        subject: PolicySubject {
            caller_id: "consumer_evaladapter".into(),
            role: "orchestrator".into(),
            purpose: "open".into(),
        },
        occurred_at: "2026-08-22T00:00:01Z".into(),
    };
    let config = EngineConfig {
        cache_root: cache.0.clone(),
        discovery: DiscoveryPolicy::new(10, 4096, 4096, 8).expect("discovery"),
        audit_retention: AuditRetention::new("2026-08-01T00:00:00Z", 20, 1_048_576)
            .expect("retention"),
    };
    let (mut engine, _) = LocalEngine::open(config, &open, &source.0).expect("open");
    engine
        .build_snapshot(
            &RequestContext {
                request_id: "req_evaladaptersnap".into(),
                event_id: "evt_evaladaptersnap".into(),
                subject: open.subject,
                occurred_at: "2026-08-22T00:00:02Z".into(),
            },
            budget(),
        )
        .expect("snapshot");
    engine
}

#[test]
fn os_adapter_is_semantically_equivalent_and_adds_no_authority() {
    let source = TestRoot::new("source");
    let direct_cache = TestRoot::new("direct-cache");
    let adapter_cache = TestRoot::new("adapter-cache");
    fs::write(
        source.0.join("authentication.rs"),
        b"pub fn authenticate() { validate_session(); }\n",
    )
    .expect("source");
    let steps = vec![ContextPlanStep {
        kind: QueryKind::Literal,
        query: "authenticate".into(),
    }];
    let context = RequestContext {
        request_id: "req_evaladapterctx".into(),
        event_id: "evt_evaladapterctx".into(),
        subject: PolicySubject {
            caller_id: "consumer_evaladapter".into(),
            role: "orchestrator".into(),
            purpose: "implementation_review".into(),
        },
        occurred_at: "2026-08-22T00:00:03Z".into(),
    };
    let mut direct = engine(&source, &direct_cache);
    let expected = direct
        .build_planned_context(
            &context,
            &ContextPlan {
                steps: steps.clone(),
            },
            budget(),
        )
        .expect("direct packet");
    let mut adapted = engine(&source, &adapter_cache);
    let response = acquire_for_os(
        &mut adapted,
        OsContextRequest {
            adapter_contract_version: ADAPTER_CONTRACT_VERSION.into(),
            request_id: context.request_id,
            event_id: context.event_id,
            consumer_id: context.subject.caller_id,
            role: context.subject.role,
            purpose: context.subject.purpose,
            occurred_at: context.occurred_at,
            steps,
            budget: budget(),
        },
    )
    .expect("adapter packet");
    validate_packet(&response.packet).expect("valid adapter packet");
    assert_eq!(response.packet, expected);
    assert!(!response.orchestration_authority_added);
}

fn manifest() -> ExtensionManifest {
    ExtensionManifest {
        schema_name: "extension-manifest".into(),
        schema_version: EXTENSION_CONTRACT_VERSION.into(),
        extension_id: "evaluation.analyzer".into(),
        extension_version: "1.0.0".into(),
        publisher: "evaluation-only".into(),
        artifact_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .into(),
        engine_contract: EXTENSION_CONTRACT_VERSION.into(),
        kind: ExtensionKind::Analyzer,
        requested_capabilities: RequestedCapabilities::deny_all(),
        max_output_bytes: "1024".into(),
        deterministic: true,
        model_dependent: false,
        data_retention: "none".into(),
        output_fields: vec!["findings".into()],
    }
}

#[test]
fn extension_adversarial_corpus_is_fully_quarantined_without_raw_retention() {
    let manifest = manifest();
    let decision = ExtensionPolicy::new(vec![manifest.artifact_digest.clone()])
        .expect("policy")
        .decide(&manifest)
        .expect("decision");
    assert!(!decision.privileged_capabilities_granted);
    assert!(!decision.artifact_execution_authorized);

    let valid = ExtensionOutput {
        schema_name: "extension-output".into(),
        schema_version: EXTENSION_CONTRACT_VERSION.into(),
        extension_id: manifest.extension_id.clone(),
        extension_version: manifest.extension_version.clone(),
        artifact_digest: manifest.artifact_digest.clone(),
        kind: manifest.kind,
        output_fields: vec!["findings".into()],
        payload: serde_json::json!({"findings": ["derived only"]}),
        claims_exact_source_authority: false,
    };
    let mut authority = valid.clone();
    authority.claims_exact_source_authority = true;
    let mut spoofed = valid.clone();
    spoofed.extension_id = "attacker.spoof".into();
    let corpus = [
        serde_json::to_vec(&authority).expect("authority case"),
        serde_json::to_vec(&spoofed).expect("spoof case"),
        br#"{"schema_name":"extension-output","unknown_control":"execute"}"#.to_vec(),
        vec![b'x'; 1025],
    ];
    for bytes in corpus {
        let NormalizationVerdict::Quarantined(record) =
            normalize_output(&manifest, &decision, &bytes)
        else {
            panic!("adversarial extension output was accepted")
        };
        assert!(!record.authority_added);
        let encoded = serde_json::to_vec(&record).expect("quarantine record");
        assert!(!encoded.windows(bytes.len()).any(|window| window == bytes));
    }

    let NormalizationVerdict::Accepted(normalized) = normalize_output(
        &manifest,
        &decision,
        &serde_json::to_vec(&valid).expect("valid output"),
    ) else {
        panic!("valid bounded output was not accepted")
    };
    assert_eq!(normalized.trust, "untrusted_derived_data");
    assert!(!normalized.authority_added);

    let mut privileged = manifest;
    privileged.requested_capabilities.process = CapabilityRequest::Requested;
    let denied = ExtensionPolicy::new(vec![privileged.artifact_digest.clone()])
        .expect("policy")
        .decide(&privileged)
        .expect("decision");
    assert!(!denied.output_submission_enabled);
    assert!(!denied.privileged_capabilities_granted);
    assert!(!denied.artifact_execution_authorized);
}
