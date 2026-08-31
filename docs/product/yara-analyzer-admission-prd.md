# Impresari Context — YARA Analyzer Admission PRD

- Status: ADR-0099 synthetic artifact compatibility implementation in progress; production artifacts, live adapter, and IAR-2 remain gated
- Date: 2026-08-31
- Owner: Aaron Boldt
- Decision: ADR-0089, superseded engine direction by ADR-0097 and bounded compatibility by ADR-0099

## Objective

Admit YARA as the first real static analyzer behind the separate Analyzer
Runner on each independently production-admitted platform.

## User Outcome

An operator requests a bounded security analysis. Impresari reports which exact
artifacts YARA examined, the exact pinned analyzer and ruleset identities, each
normalized match, complete coverage, failures, and limitations without claiming
that a repository is safe or malware-free.

## Scope

- One pinned YARA executable build per admitted platform and architecture.
- One project-owned, reviewed, versioned ruleset artifact with exact source,
  license, build, digest, expiry, and rollback identity.
- Manifest-selected regular-file inputs already admitted by HRA inventory.
- No includes, repository rules, external modules, callbacks, network, process
  launch, or writable path-backed analyzer storage.
- All-or-nothing bounded result conversion through the existing untrusted
  analyzer-result normalization boundary.

## Non-goals

- ClamAV, archive extraction, packer emulation, dynamic execution, online
  reputation, automatic remediation, deletion, quarantine, a `clean` verdict,
  or running YARA on an IAR-1A/application-only platform.

## Acceptance Criteria

- YARA runs only within a fresh production-admitted IAR-1B job.
- Executable and ruleset substitution, expiry, rollback, malformed rules,
  excessive matches, timeout, crash, partial output, and unsupported files fail
  closed with complete coverage accounting.
- Safe original-synthetic fixtures prove matches and non-matches without live
  malware or third-party sample redistribution.
- Every finding binds rule, ruleset, analyzer, artifact, snapshot, byte range
  where available, method, confidence, and limitations.
- Analyzer absence or stale rules produces explicit incomplete analysis and
  cannot authorize a later stage.
- Ruleset updates are separately built and admitted between jobs; the analyzer
  has no update or network capability.

## Platform Sequence

Linux is first after production IAR-1B and release admission. Windows and
macOS follow only after their own confinement, packaging, maintenance, and
review gates pass.

## Contract-Only Checkpoint

ADR-0095 freezes the original-synthetic `yara-adapter-contract-v1` profile,
production-shaped input, deterministic normalized receipt, exact byte-range
bindings, closed limits, and fixture provenance. The checkpoint does not
install or execute YARA, load rules, read repository-derived analyzer input, or
claim confinement, production support, IAR-2, safety, or authority.

ADR-0096 separately freezes the exact upstream source candidate and the closed
admission requirements for a future per-target executable and project-owned
ruleset. YARA v4.5.8 at commit
`84b0e3cc0e42f8f8e6b84d19c97ec3ac6ff8aee8` is source-selected only. Its
official GitHub release has no uploaded release assets, so no upstream binary
is accepted. No source archive, executable, ruleset, rule, signing material, or
credential is committed or used by this checkpoint. Selection expiry,
revocation, tag movement, substitution, and missing evidence all withdraw the
candidate without activating IAR-2.

## YARA-X Selection Gate

ADR-0097 records the official upstream transition from maintenance-focused
YARA to stable YARA-X and selects YARA-X. This PRD cannot name a production
artifact, ruleset, module subset, or live output contract until replacement
YARA-X contracts are frozen. The selection retains every confinement,
supply-chain, coverage, accuracy, hosted-evidence, and independent-review
acceptance criterion above.

ADR-0098 completes the replacement contract-only checkpoint. It pins YARA-X
v1.20.0 and every official asset digest as metadata, selects separately rebuilt
and Impresari-signed production artifacts, freezes a module-free project-owned
ruleset surface, and closes the exact single-file NDJSON invocation and
zero-byte output boundary. It downloads or admits no artifact, authors no rule,
implements no live parser, and runs no analyzer. Those remain later acceptance
gates.

## Synthetic Artifact Compatibility Checkpoint

ADR-0099 authorizes only the next evidence step. It creates a digest-pinned,
module-free patch over the exact v1.20.0 source, applies two compatible
lockfile security updates, authors one original-synthetic Impresari ruleset,
and runs a bounded compatibility corpus on generated synthetic bytes inside
the existing Linux delegated-cgroup, Landlock, and seccomp boundary.

Acceptance for this checkpoint requires exact source, patch, toolchain, lock,
feature-graph, rule-source, executable, compiled-rule, invocation, output,
host, isolation, and cleanup identities. No source, binary, compiled rule,
output, or receipt may be uploaded or retained. The checkpoint is successful
only as compatibility evidence; it does not admit an executable or ruleset,
scan repository content, implement the live parser, measure detection quality,
open IAR-2, or claim production or safety.
