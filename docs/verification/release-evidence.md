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
