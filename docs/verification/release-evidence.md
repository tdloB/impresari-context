# Release Evidence Record

This record archives externally observable verification evidence. It does not
replace independent review, release signing, clean-install testing, or an owner
decision to publish a release.

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

## Open release evidence

- Clean-install binary rehearsal from a release artifact.
- Release provenance/signing decision and implementation.
- Independent security and release review.

## 2026-08-22 — Slice D and expanded evaluation matrix

- Commit: `70b71ca2bf77797fd594aa64191314252e36848b`
- Run: [GitHub Actions 32560518924](https://github.com/tdloB/impresari-context/actions/runs/32560518924)
- Result: success in all five jobs.
- This run includes the frozen OS-adapter semantic-equivalence gate and the
  adversarial extension-output quarantine gate in addition to the complete
  evidence suite listed above.
