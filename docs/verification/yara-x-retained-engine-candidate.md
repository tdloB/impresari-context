# YARA-X Retained Engine Candidate Evidence

- Date: 2026-09-01
- Decision: [ADR-0104](../decisions/0104-retain-no-secret-yara-x-linux-engine-candidate.md)
- Profile: `yara-x-retained-engine-candidate-v1`
- Profile SHA-256: `c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da`
- Implementation state: complete; authorized hosted dispatch and verifier passed
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

## Hosted evidence

The initial run `33459619776` failed closed before upload because the runner's
temporary filesystem did not permit execution of `cargo-audit` build scripts.
Cleanup passed, the verification job did not run, and the run retained zero
artifacts. PR
[#230](https://github.com/tdloB/impresari-context/pull/230) moved only those
build intermediates into the bounded Cargo mount and deletes them immediately
after the advisory check.

The corrected manual run
[`33460329608`](https://github.com/tdloB/impresari-context/actions/runs/33460329608)
passed on exact `main` commit
`ca77a6112b04167df5cc029e23e9f369224b5227`. Build job `99708941391` and
non-executing verification job `99710540087` both passed, including mandatory
cleanup. The GitHub artifacts API returned exactly one artifact:

- Artifact ID: `9783099367`
- Name: `yara-x-v1.20.0-linux-x86_64-engine-candidate`
- Created: `2026-09-01T02:00:08Z`
- Expires: `2026-09-08T02:00:07Z`
- GitHub stored size: `8537799` bytes
- Candidate archive size: `8537583` bytes
- Candidate archive SHA-256:
  `8ba47a6ce0b5e84b5356751eb813c9eb35e375bd938f0aa81746d32ae2feffa6`
- Unexecuted `yr` SHA-256:
  `92b8abe893588b02e54c4759ff1fe8cd0173de3e6a0eba32d8d1f05923be62f5`
- SPDX SBOM SHA-256:
  `d1da23a412a85eb03cc3821a18c42c4f80282e9a302e643a73242a36cbc5dd86`
- Provenance SHA-256:
  `78eca26cb6cccd60a3779166b16242924494acbfad34c3be73d554d3d979b142`

The same-run receipt fixed `executed`, `admitted`, `production`, and `iar_2`
as false. The successful run did not sign, attest, publish, install, or execute
the candidate and contained no rules or repository scan input.

Expiry removes availability; it does not turn the temporary candidate into a
release or production admission.
