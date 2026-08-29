# HRA-5 Step 1 Release Readiness

- Candidate status: implementation and disclosure prepared; exact merged-commit
  hosted evidence pending.
- Scope: ADR-0073 HRA-0 through HRA-4 only.
- Publication effect: none. This record does not create a tag or release and
  does not alter the already published `v0.1.0` artifacts.

## Required matrix

| Gate | Required evidence | Candidate evidence |
| --- | --- | --- |
| Evaluation | HRA schema/fixture conformance; inventory, observation, coverage, normalized-result, assessment, and four-state evaluator tests | `scripts/check.sh`; HRA-1 through HRA-4 tests; 38 schemas and 42 fixtures |
| Documentation | PRD, ARD, ADR, threat model, traceability, corpus records, and public limitations agree | Repository policy gate plus this record and the Step 1 limitations statement |
| Compatibility | Tier A Rust/platform matrix and explicit capability/non-capability table | Normal CI on macOS 14 ARM64, Ubuntu 24.04/Rust 1.96–1.98, and Windows 2025; compatibility matrix HRA section |
| Clean install | Exact-source packaged binaries pass checksum, extraction, CLI, MCP lifecycle, and source-immutability rehearsal | Release-candidate workflow on macOS ARM64, Linux x64, and Windows x64 after this record merges |
| Security/supply chain | Boundary checks, CodeQL, dependency/license review, fuzzing, locked SBOM, no source mutation | Normal CI and release-candidate complete gate; 208-package frozen SBOM |

## Required hosted sequence

1. Merge the disclosure/readiness change after all protected checks pass.
2. Dispatch `.github/workflows/release-candidate.yml` on the resulting exact
   `main` commit.
3. Require success from all three native package/clean-install jobs.
4. Record the exact commit, workflow run, job results, and non-publication
   statement in this file and `release-evidence.md`.
5. Re-run protected checks for that evidence-only commit and merge it.

## Claim boundary

Until step 4 is merged, HRA-5 remains pending. Even after HRA-5 completes,
Step 1 is evidence-only and authority-neutral under the published
[limitations statement](../security/hostile-repository-admission-limitations.md).
The HRA-5 gate requires explicit founder approval before any Step 2 analyzer
implementation begins.
