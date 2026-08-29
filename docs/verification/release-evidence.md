# Release Evidence Record

This record archives externally observable verification evidence. It does not
replace release signing, clean-install testing, an owner decision to publish, or
an independent review when ADR-0017 makes one mandatory. Automated and
AI-assisted evidence is not an independent audit.

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

- Independent human security and release review before `v1.0.0`, or earlier if
  a qualifying trust-boundary expansion triggers ADR-0017. This is encouraged
  but is not an open `v0.1.0` release blocker.

## Current release status

There is no open `v0.1.0` publication gate. Future releases still require an
exact-commit hosted native matrix, clean-install evidence, checksums,
provenance attestations, reviewed release notes, and explicit owner
authorization. The provenance policy and implementation are recorded in
ADR-0016. Independent human review remains the deferred assurance target above.

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
