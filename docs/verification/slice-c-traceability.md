# Slice C Context Lifecycle And Reference Integration Traceability

## Milestone C1 — Deterministic context plans and reference consumers

Status: implemented locally; complete repository and native platform gates are
pending for this commit.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Richer context planning | Ordered one-to-eight-step exact-path, filename, literal, and lexical plans use one gateway and one hard budget | Multi-strategy engine tests, elapsed limit, Clippy |
| Bounded compression | Exact evidence is deduplicated by content identity before the existing monotonic byte-budget packager selects output | Packet integrity, deduplication, byte-accounting tests |
| Visible incompleteness | Empty and limited plan steps become stable packet unknowns; no strategy silently claims completeness | Planned-context assertions |
| Session references | Consumer-scoped in-memory immutable packet references | `context-session` ownership, integrity, limit, and close tests |
| Immutable handoff | Canonical packet bytes are exported atomically without overwrite or authority | Engine, CLI, adversarial, and schema tests |
| OS reference adapter | Versioned OS-shaped identity/purpose/plan/budget translation delegates to `LocalEngine::build_planned_context` | Adapter tests and closed consumer acquisition schema |
| Independent non-OS consumer | `context-reference-client` calls only the adapter-neutral engine surface | Valid packet integration test |
| Governed native-read fallback | Required mode never falls back; preferred mode only considers unavailable/unsupported; all security, integrity, stale, budget, and internal failures fail closed | Exhaustive fallback policy unit test and closed fallback schema |
| Authority separation | Adapter responses add no orchestration authority; fallback decisions perform and authorize no reads | Constant-false schema fields and tests |

## Open Slice C Work

- Complete hosted Tier-A CI evidence for this milestone.
- Add session reference and handoff operations to a long-lived transport when
  Slice D approves that transport; the process-local primitive is complete.
