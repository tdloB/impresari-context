# Slice A Requirement Traceability Audit

- Audit date: 2026-08-21
- Scope: approved local Verifiable Context Slice A
- Evidence rule: only direct code, executable tests, or completed review output
  can receive `Pass`; absence of a failure is not evidence.
- Statuses: `Pass`, `Partial`, `Not applicable`, or `Externally gated`.

This is a living completion ledger, not a release declaration. `Partial` items
are implementation work. `Externally gated` items require authority or an
environment outside approval A-0029 and do not authorize that action.

## MVP Functional Requirements

| ID | Status | Direct evidence or gap |
| --- | --- | --- |
| MVP-FR-001 | Pass | CLI and `LocalEngine::open` require an explicit root; workspace root rejection tests. |
| MVP-FR-002 | Pass | `AuthorizedWorkspace::open` canonicalizes and capability-binds the root before discovery. |
| MVP-FR-003 | Pass | Path vectors plus traversal, intermediate/final symlink, cache-root, and export-root tests. |
| MVP-FR-004 | Pass | `check-security-boundaries.sh` compares tracked state; binary/adversarial tests compare source. |
| MVP-FR-005 | Pass | Versioned `DiscoveryPolicy`; deterministic hidden/VCS/build, binary, size/count/depth/link/special-file rules. |
| MVP-FR-006 | Pass | `SkippedSummary` aggregates safe reason/count fields without object names. |
| MVP-FR-007 | Pass | Every exact read re-resolves relative capability paths and rechecks metadata/hash. |
| MVP-FR-008 | Pass | Workspace snapshot identity includes workspace, artifact hashes, discovery policy, and versioned identity contract. |
| MVP-FR-009 | Partial | Snapshot schema permits revision/tree fields, but runtime does not yet report optional Git metadata. |
| MVP-FR-010 | Pass | Workspace-identity namespace, snapshot-bound generations, writer exclusion, replaceable cache. |
| MVP-FR-011 | Pass | Tamper, incompatible metadata, failed promotion, abrupt restart, and safe corruption tests. |
| MVP-FR-012 | Pass | Lexical cache yields candidates only; source bytes/hash are reverified before evidence. |
| MVP-FR-013 | Pass | Exact workspace purge is idempotent, writer-safe, and source-independent. |
| MVP-FR-014 | Pass | Exact path and bounded case-normalized filename search with conformance/evaluation cases. |
| MVP-FR-015 | Pass | Bounded literal and lexical search with exact spans and deterministic oracle comparison. |
| MVP-FR-016 | Not applicable | Pattern/regex search is not an advertised or accepted query kind in Slice A. |
| MVP-FR-017 | Pass | Source-verified results sort by native path identity and byte span; deterministic fixtures pass. |
| MVP-FR-018 | Pass | Request time/file/traversal/memory/match/excerpt/output ceilings narrow configured policy; snapshot/search/index tests exercise partial/failure outcomes. |
| MVP-FR-019 | Pass | Truncation and elapsed/file/match/memory/skipped states are explicit; invalid UTF-8 exact evidence is labeled `unsupported`. |
| MVP-FR-020 | Pass | Public kinds are exact path, filename, literal, lexical; unknowns avoid semantic/graph authority. |
| MVP-FR-021 | Pass | Evidence schema and conformance tests require snapshot/path/hash/span/kind/extraction/confidence/trust. |
| MVP-FR-022 | Pass | Byte spans and lossless platform path units are specified and tested with Unicode/newline/binary-safe evidence. |
| MVP-FR-023 | Pass | Draft 2020-12 packet schema plus full validator conformance. |
| MVP-FR-024 | Pass | Canonical byte accounting, monotonic removal, boundary/property tests, and evaluation budget gate. |
| MVP-FR-025 | Pass | Domain-separated packet identity covers packet fields; tampering and identity vectors pass. |
| MVP-FR-026 | Pass | Expansion rechecks workspace/snapshot/path/hash/span and enforces output ceiling. |
| MVP-FR-027 | Pass | Current, stale, corrupt, incompatible, denied, and partially unavailable states are tested. |
| MVP-FR-028 | Pass | Canonical no-overwrite export preserves packet ID/bytes and adds no authority. |
| MVP-FR-029 | Pass | All `LocalEngine` public capabilities use the shared deterministic policy gateway. |
| MVP-FR-030 | Pass | Caller/role/purpose are validated structured policy inputs; hostile content cannot alter them. |
| MVP-FR-031 | Pass | Metadata-only audit events include decision/scope/capability/outcome/effective limits/version and measured duration; persistence tests inspect them. |
| MVP-FR-032 | Pass | Seeded secret/query/control content is absent from safe errors and audit database bytes. |
| MVP-FR-033 | Pass | Versioned JSON output/schemas and stable error taxonomy across engine and CLI. |
| MVP-FR-034 | Pass | `--human` adds bounded diagnostics to stderr only; stdout remains one machine JSON value. |
| MVP-FR-035 | Pass | CLI is a thin adapter over `LocalEngine`; normalized full-response equivalence and lifecycle tests pass. |
| MVP-FR-036 | Pass | Static production scan plus full macOS test suite under denied networking; no telemetry/update/process surface. |

## MVP Acceptance Criteria

| IDs | Status | Evidence or gap |
| --- | --- | --- |
| MVP-AC-001–003 | Pass | Root/path adversarial tests and deterministic snapshot rebuild tests. |
| MVP-AC-004–006 | Pass | Controlled mutation, exact-span, and explicit match/file limit tests. |
| MVP-AC-007–010 | Pass | Injection/leakage, packet budget, recovery, stale/cross-workspace suites. |
| MVP-AC-011–013 | Pass | Source-state gate, network-denied suite, and complete CLI/library semantic equivalence. |
| MVP-AC-014 | Externally gated | Native clean installation on macOS/Linux/Windows and distributable package rehearsal require release/remote environments. |
| MVP-AC-015 | Partial | Frozen synthetic baseline gates pass; required public corpus and human review are still gated/missing. |

## Security Requirements

| IDs | Status | Evidence or gap |
| --- | --- | --- |
| SEC-REQ-001–004 | Pass | Capability paths, object restrictions, immutability, and workspace/snapshot binding suites. |
| SEC-REQ-005 | Pass | Configured policy is monotonically narrowed by request file/traversal/time/memory limits; search/index/output/cache/audit/export ceilings are tested. |
| SEC-REQ-006–009 | Pass | Structured separation, runtime capability denial, leakage tests, and corrupt-cache/packet behavior. |
| SEC-REQ-010 | Partial | Adversarial, deterministic fuzz/property/mutation, restart, permission, and network-denied cases pass locally; native platform and coverage-guided campaign evidence remain. |
| SEC-REQ-011 | Pass locally | Reproducible SPDX SBOM plus Cargo deny/RustSec evidence; repeat at release candidate. |
| SEC-REQ-012 | Pass | `docs/security/residual-risks.md`. |

## Evaluation Gates

| IDs | Status | Evidence or gap |
| --- | --- | --- |
| EVAL-G-001–010 | Pass for frozen local corpus | `context-evaluation`, adversarial suite, source-state gate, and network-denied run. |
| EVAL-G-011 | Pass for frozen local corpus | Controlled partial/limit states and unsupported-decoding evidence are explicitly labeled. |
| EVAL-G-012 | Pass | Full normalized CLI/library semantic suite. |
| EVAL-Q-001 | Pass for frozen local corpus | Required-evidence recall 1.00. |
| EVAL-Q-002 | Pass for frozen local corpus | Independently calculated native baseline recall is 1.00; engine recall delta is 0.00. |
| EVAL-Q-003 | Pass for frozen local corpus | Context reduction 0.9987585403 at matched declared tasks. |
| EVAL-Q-004 | Pass for frozen local corpus | Evidence precision 1.00. |
| EVAL-Q-005 | Externally gated | Independent randomized human usefulness review has not occurred. |

## Evaluation Composition And Reproducibility

| Requirement | Status | Gap |
| --- | --- | --- |
| 12 synthetic fixtures / >=25% held out | Pass | 12 original fixtures, 3 held out, frozen manifest digest. |
| Adversarial corpus | Partial | Relevant local classes are covered; platform-specific and race expansion continues under SEC-REQ-010. |
| 6 small + 4 medium public repositories | Externally gated | Download/fetch and license admission require explicit external-action approval. |
| 2 large/monorepo or approved generated equivalents | Pass locally | Two nested generated profiles (2,000/5,000 files), five cold/warm samples, timing percentiles, cache ratio, measured macOS RSS, and partial-limit evidence. |
| Reproducibility metadata | Partial | Manifest/lock/toolchain/config are recorded; engine Git revision, host/resource curves, raw result digests, and deviations need the scale/release report. |

## Remaining Authorized Local Work

1. Decide and implement safe optional Git revision/working-tree metadata without
   process execution (`MVP-FR-009`), or record a scoped deferral before release.
2. Re-run L05, Rust, security, dependency, network-denied, and toolchain gates.

## Preserved Gates

No item above authorizes parser work, third-party integration, public/remote
repository creation, corpus downloads, naming/legal clearance, publication,
release, signing, or human risk acceptance.
