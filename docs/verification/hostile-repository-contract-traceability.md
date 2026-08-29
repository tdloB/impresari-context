# ADR-0073 HRA-0/HRA-1/HRA-2/HRA-3/HRA-4/HRA-5 Traceability

- Scope: static contracts, bounded read-only inventory, narrow exact-evidence
  observations, non-executing coverage/assessment construction, and pure
  deterministic reference policy evaluation.
- Authorization: founder-approved HRA-0 and separately approved HRA-1 on
  2026-08-29.
- Runtime status: HRA-1 inventory, the HRA-2 npm lifecycle and canonical
  Compose privilege corpora, HRA-3 coverage/result/assessment construction,
  and the HRA-4 pure reference evaluator are implemented. ADR-0074 analyzer
  execution and ADR-0075 quarantine execution remain absent. HRA-5 passed the
  exact-commit three-platform release-candidate matrix.

| Requirement | Authoritative artifact | Verification |
| --- | --- | --- |
| Artifact inventory | `security-artifact-inventory.schema.json` | Valid bounded Windows-oriented fixture; closed schema and fixed collection limits |
| Exact observed and untrusted-derived findings | `security-finding.schema.json` | Observed fixture requires exact evidence; derived classes require analyzer provenance |
| Canonical coverage independent of findings | `analyzer-coverage.schema.json` | Unavailable mandatory analyzer fixture remains explicit; completed state requires provenance and freshness |
| Immutable assessment without safety authority | `repository-security-assessment.schema.json` | Valid partial assessment plus rejected `safety_claimed: true` fixture |
| Deterministic policy data | `repository-admission-policy.schema.json` | Closed rule/effect fields and external-human exception owner |
| Four-state admission decision | `repository-admission-decision.schema.json` | Closed decision enum; rejected ordinary-host execution authority fixture |
| Fixed resource and authority boundary | `hostile-repository-resource-profile.schema.json` and `profiles/v1/hra-static-contract-v1.json` | Exact fixture equality and committed SHA-256 sidecar |
| Fixture provenance | `hostile-repository-fixture-provenance.json` | Per-file SHA-256 verification and all prohibited provenance flags false |
| Threat coverage | `docs/security/threat-model.md` SEC-T-042 through SEC-T-044 and SEC-REQ-017 | Repository security and policy checks |
| Evaluation gates | `docs/product/evaluation-prd.md` ADR-0073 HRA-0 hard gates | Full local and hosted conformance suites |
| Runtime inventory | `crates/context-admission/src/lib.rs` | Schema validation, exact snapshot/hash/length binding, deterministic cross-platform classification, explicit exclusions, source immutability, stale-snapshot failure, and symlink non-following unit tests |
| npm lifecycle observations | `crates/context-admission/src/lib.rs` and `docs/verification/hra-2-npm-lifecycle-rule-corpus.md` | Closed rule corpus, exact finding/evidence schema validation, false-positive cases, unsupported syntax, stale-source failure, command-value non-retention, and source immutability |
| Compose privilege observations | `crates/context-admission/src/lib.rs` and `docs/verification/hra-2-compose-privilege-rule-corpus.md` | Exact basename/layout/value corpus, exact key-token evidence, lookalike rejection, unsupported YAML cases, and no semantic/runtime inference |
| Coverage and assessment construction | `crates/context-admission/src/lib.rs` and `docs/verification/hra-3-coverage-assessment-corpus.md` | Deterministic grouping, identity recomputation, schema validation, unavailable mandatory analysis, coverage-laundering rejection, immutable assessment identity, and no safety/authority claim |
| Synthetic analyzer-result intake | `schemas/v1/analyzer-result-envelope.schema.json`, `crates/context-extensions/src/lib.rs`, and `crates/context-admission/src/lib.rs` | ADR-0013 bounded normalization, closed categorical payload, exact provenance/freshness/artifact binding, stale/mismatch/authority rejection, and untrusted-derived findings |
| Reference admission evaluation | `crates/context-admission-policy/src/lib.rs` and `docs/verification/hra-4-reference-policy-corpus.md` | Exact immutable-input and policy-digest validation, four-state truth table, complete stable matched reasons, monotonic restriction, missing-analysis and exception denial, and no I/O or execution authority |
| Step 1 release readiness | `docs/verification/hra-5-step1-release-readiness.md` and `docs/verification/release-evidence.md` | Exact commit `12a46c1b9d934830450019470c3a74c9a1b47bf8`; run 33266846683 passed package and clean-install rehearsal on all three Tier A targets without publication |

## Authority audit

The profile fixes source mutation, process execution, analyzer execution,
network access, artifact upload, deep hostile-format parsing, and ordinary-host
execution to false. The assessment and decision schemas also fix safety claims,
ordinary-host execution authorization, and added authority to false. HRA-1 adds
only the `context-admission` library crate and reuses the existing guarded
read-only workspace capability. It adds no command, transport, provider,
credential, parser dependency, finding, policy decision, or execution
capability. HRA-2 adds deterministic observed findings over already-admitted
JSON and deliberately canonical Compose YAML only. HRA-3 planning, normalized
synthetic result intake, and assembly add no analyzer discovery, command,
network, policy, or execution surface.
HRA-4 adds a separate pure consumer of those immutable records. Its dependency
graph adds no workspace, process, transport, model, credential, or runtime
adapter, and every decision fixes safety, ordinary-host execution, and added
authority to false.
HRA-5 adds only verification and public limitation evidence. Its workflow
created temporary candidate artifacts and no tag, release, package publication,
signature, credential use, analyzer execution, or quarantine authority.
