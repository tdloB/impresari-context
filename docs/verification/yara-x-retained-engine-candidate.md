# YARA-X Retained Engine Candidate Evidence

- Date: 2026-08-31
- Decision: [ADR-0104](../decisions/0104-retain-no-secret-yara-x-linux-engine-candidate.md)
- Profile: `yara-x-retained-engine-candidate-v1`
- Profile SHA-256: `c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da`
- Implementation state: complete; first authorized dispatch pending
- Production admitted: No
- IAR-2 admitted: No

## Implemented boundary

The manual workflow accepts no inputs and runs only on exact current `main` in
`tdloB/impresari-context`. It uses no secrets and declares no GitHub token
permissions. The build pins the official Rust 1.93.0 Bookworm Linux/amd64
manifest at
`sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28`.
After exact source, dependency, and advisory acquisition, the actual build runs
in a read-only container with `--network none` and the frozen target, profile,
feature, static-CRT flags, epoch, locale, and lockfile.

The one uploaded object is an authenticated, non-release GitHub Actions
artifact with requested retention of seven days, no overwrite, and exactly one
deterministic tar-gzip candidate. Because the repository is public, it is
unavailable anonymously but is downloadable by any signed-in repository
reader; it is not maintainer-only. Its twelve closed members are the unexecuted
`yr` bytes, manifest, checksums, selected dependency closure, SPDX 2.3 SBOM,
provenance, upstream and dependency notices, three explicit review
dispositions, and the closed ADR-0103 engine-candidate record.

A separate same-run job downloads the artifact and streams it through Ruby's
tar reader without extraction. It rejects links, duplicates, unsafe or unknown
members, size overflow, digest drift, malformed SBOM/candidate metadata, and
any signing, execution, admission, production, safety, or IAR-2 claim. The
verifier has no process-launch or network capability.

## Pending hosted evidence

The approved first dispatch occurs only after this implementation merges so
the workflow can bind the exact current `main` commit. The resulting run, job,
artifact, archive, executable, SBOM, provenance, creation, expiry, and
verification identities will be added here in a follow-up evidence change.

This checkpoint does not sign, attest, publish, install, or execute the
candidate. It contains no rules and scans no repository content. Expiry removes
availability; it does not turn the temporary candidate into a release or
production admission.
