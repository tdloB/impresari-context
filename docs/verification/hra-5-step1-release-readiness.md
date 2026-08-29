# HRA-5 Step 1 Release Readiness

- Candidate status: complete. Exact merged-commit hosted evidence passed on all
  three Tier A native targets.
- Candidate source commit: `12a46c1b9d934830450019470c3a74c9a1b47bf8`.
- Candidate rehearsal:
  [GitHub Actions 33266846683](https://github.com/tdloB/impresari-context/actions/runs/33266846683),
  successful.
- Scope: ADR-0073 HRA-0 through HRA-4 only.
- Publication effect: none. This record does not create a tag or release and
  does not alter the already published `v0.1.0` artifacts.

## Required matrix

| Gate | Required evidence | Candidate evidence |
| --- | --- | --- |
| Evaluation | HRA schema/fixture conformance; inventory, observation, coverage, normalized-result, assessment, and four-state evaluator tests | `scripts/check.sh`; HRA-1 through HRA-4 tests; 38 schemas and 42 fixtures |
| Documentation | PRD, ARD, ADR, threat model, traceability, corpus records, and public limitations agree | Repository policy gate plus this record and the Step 1 limitations statement |
| Compatibility | Tier A Rust/platform matrix and explicit capability/non-capability table | Normal CI on macOS 14 ARM64, Ubuntu 24.04/Rust 1.96–1.98, and Windows 2025; compatibility matrix HRA section |
| Clean install | Exact-source packaged binaries pass checksum, extraction, CLI, MCP lifecycle, and source-immutability rehearsal | Run 33266846683 succeeded for `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc` |
| Security/supply chain | Boundary checks, CodeQL, dependency/license review, fuzzing, locked SBOM, no source mutation | Normal CI and release-candidate complete gate; 208-package frozen SBOM |

## Completed hosted sequence

1. PR #120 merged after all protected checks passed.
2. The first exact-source rehearsal exposed an obsolete four-tool assertion;
   PR #121 corrected it to the exact ordered six-tool MCP contract and merged
   after all protected checks passed.
3. `.github/workflows/release-candidate.yml` ran on the resulting exact `main`
   commit `12a46c1b9d934830450019470c3a74c9a1b47bf8`.
4. All three native package/clean-install jobs succeeded in run 33266846683.
5. This evidence-only change records the result and is subject to the normal
   protected checks before merge.

## Claim boundary

HRA-5 and ADR-0073 Step 1 are complete when this evidence record merges. Step 1
remains evidence-only and authority-neutral under the published
[limitations statement](../security/hostile-repository-admission-limitations.md).
No tag, GitHub release, package publication, signature, or release credential
was created or used by the candidate rehearsal. The HRA-5 gate requires
explicit founder approval before any Step 2 analyzer implementation begins.
