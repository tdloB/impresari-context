# YARA-X Reproducibility Diagnostic Evidence

- Date: 2026-08-31
- Decision: [ADR-0105](../decisions/0105-diagnose-yara-x-build-reproducibility-before-retention.md)
- Profile: `yara-x-reproducibility-diagnostic-v1`
- Profile SHA-256: `4948ca0a448f1083cc3fe52519b57f62555c319146e91ff0999f696d69a8dbf4`
- Result: `baseline_changed_canonical_same`
- Production admitted: No
- IAR-2 admitted: No

## Boundary

One manual, no-secret GitHub-hosted Ubuntu 24.04 x86-64 job downloaded the
exact pinned public YARA-X v1.20.0 source, verified its archive, applied the
frozen Impresari patch, checked the patched lockfile and dependency policy,
and acquired the locked dependency closure once. It then performed four clean
offline builds in distinct source and target roots.

The baseline pair used the existing static-CRT build flags. The canonical pair
also fixed `SOURCE_DATE_EPOCH` to `1787565021`, locale and time zone, disabled
incremental compilation, set deterministic archive time, and remapped the
distinct compiler-visible source and target roots to canonical paths.

The workflow emitted only four SHA-256 identities and one closed result. It did
not compile rules, execute `yr`, upload data, scan repository content, or use
credentials. An `always()` step verified deletion of the disposable build root
and absence of a built `yr` in the workspace.

## Hosted Evidence

The initial run
[33442361993](https://github.com/tdloB/impresari-context/actions/runs/33442361993)
reached the diagnostic step but did not build: permission normalization on the
frozen source archive made the shell script non-executable. The workflow exited
with code 126 and its mandatory cleanup step passed. PR 222 corrected only the
invocation by calling the unchanged script through Bash.

The corrected run
[33443483096](https://github.com/tdloB/impresari-context/actions/runs/33443483096),
job `99657000024`, completed successfully in 21 minutes 4 seconds:

- dispatch head: `5155589b6821f3f9bf6c20ed8cc697cb46faa5d3` on `main`;
- immutable Impresari source root:
  `ae4e0bea1ed9576abecb998250ad06fc2081f2a8`;
- Impresari source archive: 27,959,111 bytes, SHA-256
  `2e6323cffce957108429c804dd4f9876a6a0d27fdef31569029213807c3e04a2`;
- GitHub runner `2.337.0`, Ubuntu `24.04.4`, `ubuntu-24.04` image version
  `20260823.283.1`, with the workflow's `x86_64` Linux gate satisfied;
- baseline A SHA-256:
  `748c2751180f895aaa5ef3585f82a837250ae5e66c345fd253711086c8d62d32`;
- baseline B SHA-256:
  `523e276e9e4b31f0d331027b8b179b5c335b840fa4d05a49ffabec7918033efd`;
- canonical A and B SHA-256:
  `a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d`.

The receipt verified, mandatory cleanup passed, and the GitHub artifacts API
reported `total_count: 0`.

## Interpretation

The differing baseline digests reproduce the prior identity instability. The
matching canonical digests show that fixed time, build state, locale, archive
time, and compiler-visible paths are sufficient for byte identity between two
clean builds in distinct roots inside this one exact hosted job.

This is same-job evidence only. It does not prove cross-run, cross-host, or
digest-pinned-image reproducibility and does not create a retained, signed,
attested, published, installed, or admitted engine. ADR-0104 remains a separate
artifact-custody decision.

No production ruleset, repository-derived scan, credential access, detection
quality, safety, malware-free claim, production support, or IAR-2 authority
follows from this result.
