# Release Evidence Record

This record archives externally observable verification evidence. It does not
replace release signing, clean-install testing, an owner decision to publish, or
an independent review when ADR-0017 makes one mandatory. Automated and
AI-assisted evidence is not an independent audit.

## 2026-08-30 — Linux rootless genuine login-session candidate

- Exact source commit: `bf2504f78ddb4e709407a0ac5c23d5d0ecc534a6`.
- Protected workflow:
  [GitHub Actions 33341872303](https://github.com/tdloB/impresari-context/actions/runs/33341872303),
  job `99338854149`, successful on GitHub-hosted Ubuntu 24.04 x86-64, kernel
  `6.17.0-1022-azure`.
- Source-free receipt SHA-256:
  `50ceac6df76bf90f40f6e888bb931ac84e5d18acaa7d8a442834adbcbe2538d4`.
- GitHub artifact digest:
  `sha256:7986c3ebd64e5871e4823898699b1dfa54221ca0d02593700b01bdd222149c8b`.
- Package lifecycle receipt identity:
  `38d20aea124021afcfdd7c252dde6b4f87ea8554518e1412ecb0872f02663fde`.
- The run used one temporary non-privileged, non-lingering user and an isolated
  loopback-only SSH/PAM transport. It recorded two distinct hashed logind
  session identities and two distinct hashed user-manager invocation
  identities, verified first-manager termination, preserved exact package
  identity, passed the existing original-synthetic corpus in both sessions,
  and recorded all cleanup conditions true.
- This completes profile A's genuine logout/login reentry evidence only. The
  receipt status is `login_session_candidate`; production support, real
  analyzers, privileged installation, persistent services, tagging, and
  publication remain false.

## 2026-08-29 — ADR-0073 HRA-5 Step 1 candidate matrix

- Candidate commit: `12a46c1b9d934830450019470c3a74c9a1b47bf8`.
- Candidate rehearsal:
  [GitHub Actions 33266846683](https://github.com/tdloB/impresari-context/actions/runs/33266846683),
  successful on all three native targets.
- Targets: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and
  `x86_64-pc-windows-msvc`.
- Each target verified exact source identity, ran the complete repository gate,
  built and checksummed the release archive, rehearsed a clean CLI and real MCP
  initialization/tools exchange, confirmed tracked source immutability, and
  uploaded only a seven-day candidate artifact.
- The exact ordered six-tool MCP contract was checked from the packaged server.
- The run created no tag, GitHub release, package publication, signature, or
  release credential and makes no claim that a repository is safe, trusted,
  clean, or malware-free.
- This candidate run is release-readiness evidence for HRA-0 through HRA-4. It
  does not evidence the later ADR-0074 synthetic-supervision baseline, any real
  analyzer execution, or ADR-0075 quarantine execution.

## 2026-08-23 UTC — `v0.1.0` published

- Release:
  [`v0.1.0`](https://github.com/tdloB/impresari-context/releases/tag/v0.1.0)
- Tagged commit: `c77e95ce95b2fde99da2582707d4e4d58a512122`
- Publish workflow:
  [GitHub Actions 32606121468](https://github.com/tdloB/impresari-context/actions/runs/32606121468),
  successful.
- Published native targets: `aarch64-apple-darwin`,
  `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.
- Each target's release archive has an adjacent SHA-256 file and a GitHub build
  provenance attestation produced by the release workflow from the tagged
  commit.
- The reviewed release notes describe capabilities, compatibility, security
  boundaries, limitations, and the absence of an independent third-party
  security audit.
- This closes the publication gate for `v0.1.0`. Later default-branch
  capabilities are not claims about these published artifacts.

## 2026-08-22 — Slices A–C native matrix

- Commit: `cea4a36f3d28e84cc7b429702f94d09daa15f126`
- Run: [GitHub Actions 32559752496](https://github.com/tdloB/impresari-context/actions/runs/32559752496)
- Result: success in all five jobs.
- Platforms/toolchains:
  - macOS 14, Apple silicon, Rust 1.98;
  - Windows 2025, x86-64, Rust 1.98;
  - Ubuntu 24.04, x86-64, Rust 1.98;
  - Ubuntu 24.04, x86-64, Rust 1.96 minimum-supported version;
  - Ubuntu 24.04, x86-64, Rust 1.97 compatibility version.
- Gate contents: repository policy, formatting, workspace tests, adversarial
  tests, closed-schema conformance, identity/path/JCS vectors, SBOM validation,
  frozen retrieval and structural evaluation, scale evaluation, abrupt-cache
  recovery, Clippy with warnings denied, and documentation tests.
- Limitation: hosted application tests do not prove OS-level sandbox
  confinement. No such claim is made.

## Deferred assurance targets

- Independent human security and release review was not an open `v0.1.0`
  blocker. The post-v0.1.0 external-client, authentication-reference, and child-
  process boundaries now satisfy ADR-0017's earlier-review trigger. ADR-0083
  makes an attributable independent human report mandatory before the proposed
  v0.2.0 tag or publication. ADR-0084 first backlogged reviewer engagement until
  candidate freeze. ADR-0085 now treats that candidate as immutable historical
  evidence and continues ordinary roadmap development; the review remains
  mandatory only after a later final candidate is frozen.

## Current release status

There is no open `v0.1.0` publication gate. Future releases still require an
exact-commit hosted native matrix, clean-install evidence, checksums,
provenance attestations, reviewed release notes, and explicit owner
authorization. The provenance policy and implementation are recorded in
ADR-0016. The proposed v0.2.0 feature release remains review-gated. The first
product candidate was frozen at
`1a9923c0e5d671581f6b7da3bc4248b604971d63`. Release-candidate run
[`33323269945`](https://github.com/tdloB/impresari-context/actions/runs/33323269945)
passed on all three native release targets, including the Linux
v0.1.0-to-v0.2.0 package lifecycle and external composition checks. Its exact
archive, manifest, and workflow-artifact identities are recorded in the
immutable historical candidate scope. ADR-0085 prevents that scope from being
used after later production changes. Accepted roadmap development now precedes
a new final-candidate freeze. Review admission, tagging, publication,
production analyzer support, and real-analyzer execution remain false.

ADR-0095's YARA adapter checkpoint is contract-only. Its committed records are
original-synthetic and prove bounded deterministic normalization, not analyzer
execution or malware detection. No YARA executable, ruleset, repository-derived
analyzer input, confinement claim, production claim, IAR-2 claim, or release
artifact enters the checkpoint.

ADR-0096 adds a metadata-only YARA supply-chain checkpoint. The exact official
v4.5.8 tag commit, publication timestamp, source-archive API URL, zero uploaded
release assets, BSD-3-Clause identifier, and `COPYING` git-blob identity were
observed from the VirusTotal/YARA GitHub repository on 2026-08-31. The profile
expires on 2026-09-30 and requires a fresh official observation after expiry.
It admits no archive, executable, ruleset, IAR-1B backend, live result, IAR-2,
production support, or safety verdict. The offline checker uses no network or
credentials and rejects tag movement, overclaim, expiry, revocation, and absent
future artifact evidence.

ADR-0097 records the post-checkpoint engine evaluation. Official metadata
observed on 2026-08-31 identifies YARA-X v1.20.0 at commit
`60ad06971467029e77967e59d580cbbe85a1474d`, published 2026-08-24 with six
multi-platform assets carrying GitHub-recorded SHA-256 digests. The upstream
README describes YARA-X as the Rust successor intended to replace YARA and
documents stable production use; its CLI documents JSON/NDJSON output and no
process-scanning support. The same official sources disclose incompatible APIs
and rule-language differences. These were decision inputs only: no asset was
downloaded, authenticated as an Impresari release, built, admitted, or run.
The founder selected YARA-X on 2026-08-31; replacement source/build, ruleset,
adapter, and output contracts remain required before any artifact or execution
work.

ADR-0098 freezes the replacement `yara-x-contract-v1` boundary. It records
YARA-X v1.20.0 at commit
`60ad06971467029e77967e59d580cbbe85a1474d`, the six uploaded release assets
and GitHub-recorded SHA-256 digests, and the upstream release workflow's
mutable `stable` toolchain and `*-latest` runner posture. Official assets are
therefore evidence candidates only. The selected production strategy requires
a separately digest-pinned source archive, locked per-target rebuild,
dependency closure, SBOM, provenance, reproducibility disposition,
vulnerability/license review, and Impresari signature. The exact future CLI
surface is one compiled project ruleset and one staged file with no modules,
includes, external variables, recursive/list scan, mmap, ambient config, or
arbitrary arguments. Bounded NDJSON uses `--print-strings=0` so offsets and
lengths can be normalized without retaining matched bytes. This checkpoint
downloads nothing, creates no artifact or rule, implements no live parser, and
runs no analyzer or compatibility corpus.

## 2026-08-22 — Slice D and expanded evaluation matrix

- Commit: `70b71ca2bf77797fd594aa64191314252e36848b`
- Run: [GitHub Actions 32560518924](https://github.com/tdloB/impresari-context/actions/runs/32560518924)
- Result: success in all five jobs.
- This run includes the frozen OS-adapter semantic-equivalence gate and the
  adversarial extension-output quarantine gate in addition to the complete
  evidence suite listed above.

## 2026-08-22 — Local MCP and release-candidate matrix

- Candidate commit: `3310b8da66a81cc280c582d534832616e6481fc4`
- Normal CI: [GitHub Actions 32577444138](https://github.com/tdloB/impresari-context/actions/runs/32577444138), success in all five jobs.
- Candidate rehearsal: [GitHub Actions 32577840031](https://github.com/tdloB/impresari-context/actions/runs/32577840031), success on all three native targets.
- Candidate targets and temporary artifacts:
  - `aarch64-apple-darwin`, 3,818,818-byte workflow artifact;
  - `x86_64-unknown-linux-gnu`, 4,420,492-byte workflow artifact;
  - `x86_64-pc-windows-msvc`, 4,151,983-byte workflow artifact.
- Each target passed the complete repository gate, release compilation,
  exact-source manifest generation, archive checksum verification, clean
  extraction, CLI smoke check, real MCP initialize/tools exchange, tracked
  source immutability check, and candidate-only artifact upload.
- The MCP release evaluation requires direct-engine packet equivalence, hostile
  repository text to remain untrusted data, no source mutation, bounded framing,
  strict lifecycle handling, and no orchestration or filesystem authority.
- Artifacts have seven-day retention. This run created no tag, GitHub release,
  package publication, signature, or release credential.
- An earlier candidate run exposed and rejected a Windows-only GNU tar drive-
  prefix ambiguity. Commit `3310b8d` changed archive arguments to relative names;
  the successful matrix above verifies the correction.

## 2026-08-31 — ADR-0086 scheduled maintenance admission

- Product commit: `d196f4cfc0332fd3bcfa6e93ec3bb95f5d8706ff`.
- Corrected live run:
  [GitHub Actions 33345269371](https://github.com/tdloB/impresari-context/actions/runs/33345269371),
  successful in both the read-only observation and separately permissioned
  exact-owned issue jobs.
- Receipt artifact: `9741774301`, GitHub digest
  `sha256:3e3d10a247841df6eaea57ff721637bd4c5953ade3ff02ae93053fafb2851adf`;
  bounded observation and receipt file SHA-256 identities are recorded in the
  ADR-0086 verification record.
- Four official sources reported newer, explicitly unadmitted versions. Cursor
  failed closed as unavailable because no documented authoritative source is
  admitted. Issues `#165` through `#169` carry the exact-owned label and hidden
  ownership key.
- Idempotence run:
  [GitHub Actions 33345318603](https://github.com/tdloB/impresari-context/actions/runs/33345318603),
  successful. It updated the same five issues and created no duplicate. Its
  artifact `9741790216` has GitHub digest
  `sha256:7a086db7b8b8314d780cd4d6a514546c0505054e207a7ad12bbb8a3ca85a20df`.
- Earlier run `33344770117` failed before issue reconciliation when its timestamp
  expression was unavailable. The write job was skipped; PR 164 corrected the
  timestamp source and added a regression guard.
- These runs created no compatibility admission, manifest repair, merge, tag,
  release, package publication, signature, or risk acceptance.

## 2026-08-31 — ADR-0099 YARA-X synthetic compatibility

- Workflow commit: `f73803d2e37cc261a414c4b23ec52ce316df7968`.
- Immutable Impresari source root:
  `9a69e1e0d3fff58676ef91f33b8dd9f6b8330ae7`.
- Successful manual run:
  [GitHub Actions 33406541396](https://github.com/tdloB/impresari-context/actions/runs/33406541396),
  job `99535422988`.
- Exact host: GitHub `ubuntu-24.04` image `20260823.283.1`, kernel
  `6.17.0-1022-azure`, x86-64, Landlock ABI 7.
- The frozen v1.20.0 patched build passed the Linux composite, five
  Impresari-owned synthetic cases, and the separate mandatory cleanup step.
- Bounded executable SHA-256:
  `f238098b1351303ad53cd240ffe1b591f4a0d7f625ac26ba9d22a7ac1ab3b718`.
- Bounded compiled-rules SHA-256:
  `010ea0e190fa5bf8f07fa08b6cb594ad154fa352fa53931e9eb85e1bf5847f35`.
- The workflow uploaded no source, executable, rules, raw output, or receipt
  artifact and scanned no repository content.
- This is compatibility evidence only. Reproducibility, signatures, live
  parsing, artifact/ruleset admission, production, IAR-2, detection quality,
  safety, and malware-free claims remain false.

## 2026-08-31 — ADR-0100 pure YARA-X NDJSON adapter

- Frozen profile SHA-256:
  `e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c`.
- `context-yara-x-adapter` performs an all-or-nothing in-memory transform from
  one bounded original-synthetic YARA-X record plus separate exact controls to
  a deterministic path-free, source-free result.
- Closed registered schemas cover the profile, control metadata, and output.
  Every committed positive and negative fixture is bound by exact digest in a
  reviewed original-synthetic provenance record.
- Offline tests cover match/no-match, deterministic order and identity,
  duplicate and unknown fields, framing, UTF-8, path substitution, marker
  grammar, checked ranges, limits, and malformed mutations without panics.
- The production crate has no filesystem, process, network, environment,
  clock, credential, or embedded-file capability. It does not execute YARA-X
  and fixes analyzer execution, confinement, production, IAR-2, safety, and
  added-authority claims to false.

## 2026-08-31 — ADR-0102 live YARA-X synthetic composition

- Workflow source commit: `04228fbfa1babfcf3bdba71dcba4cacaff006c40`.
- Immutable Impresari source root:
  `3b74648cbdf78453dcd71ab34da1ecd876093862`.
- Successful manual run:
  [GitHub Actions 33432469614](https://github.com/tdloB/impresari-context/actions/runs/33432469614),
  job `99620875408`.
- Exact host: GitHub `ubuntu-24.04` image `20260823.283.1`, Ubuntu `24.04.4`,
  runner `2.337.0`, kernel `6.17.0-1022-azure`, x86-64, Landlock ABI 7.
- Live-envelope profile SHA-256:
  `2aa5e203f71089688baa41556c6775e7dcca98c7e6aab726442ff99fb5f8cd26`.
- YARA-X v1.20.0 executable SHA-256:
  `9e7424e62b714ee7b7be9fb0b67367b12209a9ec6967ff8c9b4c4959d6a17549`.
- Compiled Impresari synthetic-rules SHA-256:
  `f92cda545be5514258a1d64f721522afbc960630212e2ddea50a3430847f86f0`.
- Five generated cases passed real YARA-X execution, Linux confinement,
  in-memory ADR-0100 adapter composition, exact accounting, and mandatory
  cleanup. The receipt recorded `production_admitted=false` and `iar_2=false`.
- No source, executable, compiled rules, raw output, or receipt artifact was
  uploaded or retained. No repository content or credential was scanned. This
  evidence does not admit artifacts or rules, open production or IAR-2, or
  establish detection-quality, safety, or malware-free claims.

## 2026-08-31 — ADR-0103 production-admission candidate contracts

- Policy SHA-256:
  `fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c`.
- Registered candidate schema SHA-256:
  `eda3497fcc6a56a07ded32c5bec3b3f2f922af6d1d4c02792827fb425d2deb54`.
- Closed engine, ruleset, and release-binding definitions reject engine
  admission, synthetic production-rule provenance, and release activation.
- Six exact-digest positive/negative fixtures pass the repository Draft
  2020-12 conformance suite and the dedicated source-free checker.
- Current state remains `release_pending`; no executable or ruleset is
  retained, signed, uploaded, published, or admitted, and production and IAR-2
  remain false.

## 2026-08-31 — ADR-0105 YARA-X reproducibility diagnostic

- Corrected workflow dispatch head:
  `5155589b6821f3f9bf6c20ed8cc697cb46faa5d3`.
- Successful manual run:
  [GitHub Actions 33443483096](https://github.com/tdloB/impresari-context/actions/runs/33443483096),
  job `99657000024`, completed in 21 minutes 4 seconds.
- Exact host evidence: GitHub runner `2.337.0`, Ubuntu `24.04.4`,
  `ubuntu-24.04` image version `20260823.283.1`; the job enforced x86-64
  Linux.
- Frozen diagnostic profile SHA-256:
  `4948ca0a448f1083cc3fe52519b57f62555c319146e91ff0999f696d69a8dbf4`.
- Baseline SHA-256 identities differed:
  `748c2751180f895aaa5ef3585f82a837250ae5e66c345fd253711086c8d62d32`
  and
  `523e276e9e4b31f0d331027b8b179b5c335b840fa4d05a49ffabec7918033efd`.
- Both canonicalized builds produced SHA-256
  `a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d`;
  the closed result was `baseline_changed_canonical_same`.
- Receipt verification and mandatory cleanup passed. The GitHub artifacts API
  returned zero artifacts. No rules were compiled and no analyzer executed.
- This is same-job evidence only. Cross-run, cross-host, retained-artifact,
  signing, publication, production, and IAR-2 claims remain false.

## 2026-08-31 — ADR-0104 retained YARA-X engine candidate implementation

- Frozen retention profile SHA-256:
  `c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da`.
- Exact official Rust 1.93.0 Bookworm Linux/amd64 image manifest:
  `sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28`.
- The manual no-input workflow, offline build boundary, twelve-member archive,
  seven-day single-artifact retention, non-executing same-run verifier, closed
  schema, overclaim fixture, and static implementation checker are complete.
- Initial run `33459619776` failed closed before upload when the runner's
  temporary filesystem denied execution of cargo-audit build scripts. Cleanup
  passed, verification did not run, and the artifacts API returned zero.
- PR [#230](https://github.com/tdloB/impresari-context/pull/230) moved only the
  cargo-audit build intermediates into the bounded Cargo mount and deletes them
  after the advisory check.
- Corrected exact-main run
  [`33460329608`](https://github.com/tdloB/impresari-context/actions/runs/33460329608)
  passed at commit `ca77a6112b04167df5cc029e23e9f369224b5227`.
  Build job `99708941391`, non-executing verification job `99710540087`, and
  both mandatory cleanup gates passed.
- The artifacts API returned exactly one artifact, ID `9783099367`, created
  `2026-09-01T02:00:08Z`, expiring `2026-09-08T02:00:07Z`, with GitHub stored
  size `8537799` bytes. The candidate archive is `8537583` bytes at SHA-256
  `8ba47a6ce0b5e84b5356751eb813c9eb35e375bd938f0aa81746d32ae2feffa6`.
- The unexecuted `yr` SHA-256 is
  `92b8abe893588b02e54c4759ff1fe8cd0173de3e6a0eba32d8d1f05923be62f5`;
  SPDX SBOM SHA-256 is
  `d1da23a412a85eb03cc3821a18c42c4f80282e9a302e643a73242a36cbc5dd86`;
  provenance SHA-256 is
  `78eca26cb6cccd60a3779166b16242924494acbfad34c3be73d554d3d979b142`.
- The receipt fixed execution, signing, attestation, publication, admission,
  rules, repository scanning, production, IAR-2, detection-quality, and safety
  claims as false.

## 2026-09-01 — ADR-0110 ephemeral macOS product candidates

- Candidate source revision:
  `aca656771f9286b13fbcc046b133ade62b58da2a`; deterministic source-archive
  SHA-256:
  `f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b`.
- Exact host: macOS `26.5.1` build `25F80`, arm64; Xcode `26.6` build
  `17F113`; SDK `26.5` build `25F70`; Swift `6.3.3`; Rust/Cargo `1.98.0`.
- Two accepted independent private-root builds used locked offline Cargo,
  non-incremental compilation, private Swift caches, locale `C`, and source
  epoch `1788243888` from the candidate commit.
- All four product outputs were byte-identical across the two builds. The
  canonical product identity is
  `sha256:7bd280339e2a8cf30c26fc2ad96225f52cad5593c63ea621e7e44ba62b9bd5ca`.
- Each output was thin arm64 Mach-O with a linker ad-hoc code directory, no
  Team Identifier, and no Developer ID signature. None was executed.
- The frozen SPDX SBOM, locked dependency graph, no-fetch Cargo Audit, and
  offline Cargo Deny license/source checks are digest-bound. No matching
  advisory was found in exact local database revision
  `ba9db2a77a6a0fe93bc63a3d9b730e08b145aff5`; no vulnerability-free claim is
  made.
- Two superseded setup roots and both accepted roots were deleted. No
  executable, cache, or raw build log was retained.
- Guest completion, app assembly, signing, notarization, cask lifecycle, VM,
  analyzer, release identity, production, and macOS IAR-1B remain false.
