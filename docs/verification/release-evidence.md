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

- Successful full native matrix for the Slice D/evaluation-gate commit.
- Clean-install binary rehearsal from a release artifact.
- Release provenance/signing decision and implementation.
- Independent security and release review.
