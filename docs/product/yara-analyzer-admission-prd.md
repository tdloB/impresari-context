# Impresari Context — YARA Analyzer Admission PRD

- Status: ADR-0104 no-secret retention and same-run verification passed; ADR-0106 ruleset boundary proposed; signing, activation, and IAR-2 remain gated
- Date: 2026-08-31
- Owner: Aaron Boldt
- Decision: ADR-0089, superseded engine direction by ADR-0097, bounded compatibility by ADR-0099, pure adapter boundary by ADR-0100, synthetic envelope by ADR-0101, real-engine synthetic composition by ADR-0102, separated production admission by ADR-0103, approved retained-engine custody by ADR-0104, ephemeral reproducibility diagnosis by ADR-0105, and proposed production-ruleset boundary by ADR-0106

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
YARA to stable YARA-X and selects YARA-X. At selection time, this PRD could not
name a production artifact, ruleset, module subset, or live output contract;
ADRs 0098–0100 now freeze those contract boundaries without admitting them for
production. The selection retains every confinement,
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

Run `33406541396`, job `99535422988`, satisfied this checkpoint on the exact
Ubuntu 24.04 hosted candidate. The result admits compatibility evidence only.

## Pure NDJSON Adapter Checkpoint

ADR-0100 completes the next independently reviewable boundary. It implements only a
pure Rust transformation over bounded committed original-synthetic YARA-X
NDJSON fixtures. It must validate exact one-line framing, UTF-8, the staged
path, closed fields, identifiers, tags, zero-byte match markers, integer and
range bounds, complete accounting, and canonical ordering before emitting a
path-free source-free normalized result.

The checkpoint has no filesystem, process, network, environment, clock, or
credential capability. Parser success cannot claim analyzer execution. No
repository-derived bytes, live runner linkage, production artifact/ruleset,
IAR-2, detection quality, safety, or malware-free status enters this step.

The implemented profile is
`sha256:e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c`.
Its pure Rust API consumes only an in-memory byte slice and explicit control
metadata. Closed schemas and digest-bound original-synthetic fixtures cover the
profile, controls, output, positive/no-match records, and fail-closed cases.
ADR-0101 now selects synthetic runner-envelope composition as the next separate
decision. The production artifact pipeline remains a later gate.

## Synthetic Runner-Envelope Checkpoint

ADR-0101 implements the synthetic runner-envelope checkpoint before production
artifact work. A dedicated content-addressed Impresari emitter may output only
the exact committed valid-match or valid-no-match NDJSON record. It accepts no
repository bytes, paths, rules, arbitrary arguments, network destinations,
credentials, or ambient configuration.

Acceptance requires bounded stdout, empty stderr, exact emitter and output
digests, fresh job and cleanup evidence, exact ADR-0100 control bindings,
deterministic normalization, and a source-free composition receipt. The
receipt distinguishes synthetic-emitter execution from analyzer execution and
fixes every YARA-X, production, IAR-2, detection, safety, and authority claim
to false.

The frozen envelope profile is
`sha256:356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b`.
The implementation reuses the single Analyzer Runner launch site and the
existing Linux cgroup/Landlock/seccomp launcher. Its local suite composes only
in-memory captures; the emitter itself may run only in the manual ephemeral
hosted matrix.

Run `33419412353`, job `99577842304`, satisfied the hosted acceptance criteria
for both closed synthetic cases and mandatory cleanup on Ubuntu 24.04. It did
not execute YARA-X, admit an executable or ruleset, open IAR-2, or establish a
detection or safety claim.

## Live YARA-X Synthetic Composition Checkpoint

ADR-0102 joins the two previously separate evidence paths without opening
production. The exact ephemeral v1.20.0 candidate from ADR-0099 may execute
only over the five generated Impresari cases, through the one audited Analyzer
Runner launch site and admitted Linux isolation boundary. Complete bounded
stdout is composed in memory with the ADR-0100 parser and discarded.

Acceptance requires exact executable, ruleset, launcher, artifact, profile,
case-result, confinement, resource, and cleanup identities. The outer receipt
may record real YARA-X execution and OS confinement. It must keep executable
and ruleset admission, repository scanning, credentials, uploads, production,
IAR-2, detection quality, safety, and authority false. Run `33432469614`, job
`99620875408`, passed the manual empty-workspace Ubuntu 24.04 matrix for all
five generated cases and mandatory cleanup. Its source-free receipt kept
`production_admitted=false` and `iar_2=false`.

## Production Admission Architecture

ADR-0103 separates the next work into an engine bundle, a project-owned
ruleset bundle, and a final release-binding manifest. Each bundle has its own
content identity, review, signature, expiry, rollback, and revocation state.
The binding manifest must also name the exact adapter and resource profiles and
a fresh compatible ADR-0082 Linux production-support receipt.

The three candidate shapes are implemented as one closed registered schema.
They deliberately cannot represent an admitted engine, a synthetic production
ruleset, or an activated release. Retained artifact creation, production rule
authorship, signing, publication, and activation remain separate later gates.

ADR-0104 implements the first retained-artifact checkpoint. One approved
manual no-secret workflow builds only the Linux x86-64
engine from the exact frozen source, patch, lockfile, toolchain, feature, and
digest-pinned build image. One authenticated, non-release seven-day Actions
artifact may contain only `yr`, its exact manifest/checksums, dependency
closure, SPDX SBOM,
provenance, licenses, review dispositions, and an explicitly unadmitted
candidate record. Source, rules, scan inputs/outputs, credentials, signing
material, and unrelated repository content remain ineligible. The workflow
and its non-executing verifier passed in exact-main run `33460329608`; exactly
one artifact was retained for seven days.

ADR-0105 inserts a no-upload diagnostic before that decision because two
otherwise matching hosted builds produced different executable digests. Four
clean builds in one ephemeral job compare the current flags with fixed
time/path-remapped flags after one locked dependency acquisition. Only digests
and a closed result are emitted; no binary is executed, retained, or uploaded.
Run `33443483096` returned `baseline_changed_canonical_same`: the ordinary
clean builds differed, while both canonicalized clean builds produced SHA-256
`a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d`.
This narrows the reproducibility gap but cannot establish cross-run,
cross-host, or production reproducibility.

Contract and candidate-pipeline work may proceed while every production and
IAR-2 claim remains false. The single ADR-0104 short-lived candidate is the
only authorized retention; production rules, signing, publication, final
activation, and repository-derived scans are independent later gates. The
first eligible scope is Linux x86-64 external delegation only;
macOS, Windows, broad Linux, and the rootless profile remain independent.

ADR-0106 proposes the next ordered stage as a small original project-owned
ruleset, not a relabeling of compatibility fixtures and not a third-party
feed. Each rule would carry an exact purpose, limitations, ownership, and
generated positive, near-miss, benign-collision, and mutation fixtures under
the frozen module-free literal/hex surface. An independent attributable human
ruleset review remains mandatory before the source can be review-complete.
Until founder approval and that later review, no production-rule source,
compiled rules, execution, retention, or detection-quality claim exists.
