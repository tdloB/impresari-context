// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Independent non-OS programmatic reference consumer."]

use context_core::{ContextPacket, ResourceBudget};
use context_engine::{
    ContextPlan, ContextPlanStep, EngineError, LocalEngine, QueryKind, RequestContext,
};

/// Acquires a task packet using only the public adapter-neutral engine surface.
///
/// This client deliberately owns no policy, filesystem, orchestration, session,
/// or fallback authority. It demonstrates that the public engine is useful
/// without the AI App Builder OS adapter.
///
/// # Errors
///
/// Returns the engine's public structured failure.
pub fn acquire_task_context(
    engine: &mut LocalEngine,
    context: &RequestContext,
    task_query: &str,
    budget: ResourceBudget,
) -> Result<ContextPacket, EngineError> {
    engine.build_planned_context(
        context,
        &ContextPlan {
            steps: vec![
                ContextPlanStep {
                    kind: QueryKind::Filename,
                    query: task_query.to_owned(),
                },
                ContextPlanStep {
                    kind: QueryKind::Lexical,
                    query: task_query.to_owned(),
                },
            ],
        },
        budget,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::{PolicySubject, validate_packet};
    use context_engine::EngineConfig;
    use context_store::AuditRetention;
    use context_workspace::DiscoveryPolicy;
    use std::{fs, path::PathBuf};

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "impresari-reference-{label}-{}",
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

    fn context(sequence: u8, purpose: &str) -> RequestContext {
        RequestContext {
            request_id: format!("req_reference{sequence:02}"),
            event_id: format!("evt_reference{sequence:02}"),
            subject: PolicySubject {
                caller_id: "consumer_reference01".into(),
                role: "independent_client".into(),
                purpose: purpose.into(),
            },
            occurred_at: format!("2026-08-22T00:00:{sequence:02}Z"),
        }
    }

    fn budget() -> ResourceBudget {
        ResourceBudget::conservative(8192, 20, 100, 128, 100, 16, 30_000, 67_108_864)
            .expect("budget")
    }

    #[test]
    fn independent_client_builds_a_valid_packet() {
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
        let (mut engine, _) =
            LocalEngine::open(config, &context(1, "open"), &source.0).expect("open");
        engine
            .build_snapshot(&context(2, "snapshot"), budget())
            .expect("snapshot");
        let packet = acquire_task_context(
            &mut engine,
            &context(3, "reference_review"),
            "authentication",
            budget(),
        )
        .expect("packet");
        validate_packet(&packet).expect("valid packet");
        assert!(!packet.observed_evidence.is_empty());
        assert_eq!(packet.purpose, "reference_review");
    }
}
