# Release Evidence Record

This record archives externally observable verification evidence. It does not
replace release signing, clean-install testing, an owner decision to publish, or
an independent review when ADR-0017 makes one mandatory. Automated and
AI-assisted evidence is not an independent audit.

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

## Open release evidence

- Final exact-commit release-evidence review and explicit owner publication
  authorization. The provenance/signing policy and implementation are recorded
  in ADR-0016.

Local packaging and clean-install rehearsal pass on macOS ARM64, including a
real MCP initialize/tools exchange. This is implementation evidence, not a
substitute for the exact-commit hosted native candidate matrix.

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
