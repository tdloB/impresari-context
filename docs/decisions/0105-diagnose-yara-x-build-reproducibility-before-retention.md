# ADR-0105: Diagnose YARA-X Build Reproducibility Before Retention

- Status: Implemented; same-job canonical equality passed; cross-run and production reproducibility remain unproven
- Date: 2026-08-31
- Decider: Aaron Boldt through explicit bounded YARA-X build and synthetic-compatibility authorization
- Related PRD: [YARA Analyzer Admission PRD](../product/yara-analyzer-admission-prd.md)
- Related architecture: [YARA Analyzer Admission ARD](../architecture/yara-analyzer-admission-ard.md)
- Related decisions: ADR-0099, ADR-0102, ADR-0103, ADR-0104

## Context

Two ADR-0102 runs used the same pinned Impresari source root, YARA-X source,
patch, Rust toolchains, runner-image version, kernel, architecture, and
synthetic-rules identity but produced different `yr` executable digests. Both
runs passed compatibility, confinement, and cleanup. The difference is not a
compatibility failure, but it prevents any byte-reproducibility claim and adds
uncertainty to ADR-0104's proposed retained candidate.

The founder authorized downloading the exact pinned YARA-X v1.20.0 source,
locked local or CI builds, Impresari-owned synthetic rules, and synthetic-only
compatibility execution in already admitted isolation boundaries. The same
authorization expressly prohibits uploads, repository scans, credential
access, and production admission. A no-upload build-only diagnostic is within
that boundary and can narrow the discrepancy before artifact custody is
considered.

## Decision

Create `yara-x-reproducibility-diagnostic-v1` as a manual, no-secret,
GitHub-hosted Ubuntu 24.04 x86-64 diagnostic. It downloads only exact public
source archives, validates the frozen YARA-X archive, applies the frozen patch,
fetches the locked dependency closure once, and performs all four builds
offline.

The job creates four independent clean source and target roots:

1. two baseline builds using the exact ADR-0099 static-CRT flags; and
2. two canonicalized builds that additionally set the upstream commit time as
   `SOURCE_DATE_EPOCH`, disable incremental compilation, fix locale and time
   zone, and remap each distinct source and target root to the same canonical
   compiler paths.

The diagnostic records only the four executable SHA-256 identities and one of
four closed outcomes:

- `baseline_same_canonical_same`;
- `baseline_changed_canonical_same`;
- `baseline_changed_canonical_changed`; or
- `baseline_same_canonical_changed`.

A completed changed outcome is evidence, not a workflow failure. Only the two
canonical digests being equal supports same-job reproducibility under the exact
host. It does not establish cross-run, cross-host, image-pinned, or production
reproducibility. Every binary, source tree, dependency cache, target tree, and
receipt is deleted before the job ends.

The diagnostic never compiles rules and never executes `yr`. ADR-0102 remains
the only authorized real-engine synthetic execution path and continues to run
inside the admitted Linux isolation boundary.

## Hosted Evidence

The first dispatch, run `33442361993`, stopped before building because the
permission-normalized source archive made the diagnostic script non-executable.
Its mandatory cleanup check passed. PR 222 changed only the workflow invocation
to call the unchanged frozen script through Bash.

[GitHub Actions run 33443483096](https://github.com/tdloB/impresari-context/actions/runs/33443483096),
job `99657000024`, then completed successfully from `main` commit
`5155589b6821f3f9bf6c20ed8cc697cb46faa5d3` in 21 minutes 4 seconds. The job
used GitHub runner `2.337.0`, Ubuntu `24.04.4`, and `ubuntu-24.04` image version
`20260823.283.1`; its platform gate required `x86_64` Linux. It downloaded the
frozen Impresari source root `ae4e0bea1ed9576abecb998250ad06fc2081f2a8`
and verified its 27,959,111-byte archive at SHA-256
`2e6323cffce957108429c804dd4f9876a6a0d27fdef31569029213807c3e04a2`.

The closed result was `baseline_changed_canonical_same`:

- baseline A: `748c2751180f895aaa5ef3585f82a837250ae5e66c345fd253711086c8d62d32`;
- baseline B: `523e276e9e4b31f0d331027b8b179b5c335b840fa4d05a49ffabec7918033efd`;
- canonical A and B:
  `a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d`.

The two ordinary clean builds differed while the two fixed-time,
path-remapped clean builds were byte-identical. The receipt verified, mandatory
cleanup passed, and the GitHub artifacts API returned zero artifacts. No rule
was compiled and no analyzer was executed.

## Consequences

- Path and time normalization can be evaluated without retaining executable
  bytes or opening a production boundary.
- The observed canonical match narrows later ADR-0104 work but cannot admit or
  publish an artifact.
- A canonical mismatch leaves reproducibility unresolved and requires deeper
  toolchain or dependency investigation before production admission.
- The mutable GitHub runner label remains a limitation; same-job equality does
  not replace the digest-pinned build-image requirement proposed by ADR-0104.

## Rejected alternatives

- Upload both binaries for comparison: rejected because uploads remain
  explicitly unauthorized and digest comparison is sufficient for this stage.
- Execute the four binaries: rejected because build reproducibility does not
  require analyzer execution and ADR-0102 already supplies isolated synthetic
  compatibility evidence.
- Treat two successful scans as reproducibility: rejected because compatibility
  and byte identity are separate claims.
- Ignore the digest difference: rejected because later signing and release
  binding require an exact executable identity.

## Activation gate

This decision authorizes only the ephemeral, build-only diagnostic above. It
does not authorize artifact retention or upload, signing, attestation,
publication, installation, analyzer execution, production rules, repository
input, credentials, production admission, or IAR-2. ADR-0104 remains proposed
and separately gated.
