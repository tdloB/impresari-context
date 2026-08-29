# IAR-1A Application-Enforced Synthetic Supervision

- Date: 2026-08-29
- Decision: ADR-0074
- Scope: private staging and first-party synthetic subprocess only

## Delivered controls

- Separate supervisor crate and short-lived synthetic worker binary.
- Exact worker executable digest and manifest validation before launch.
- Fresh job directory under one explicit absolute staging root; symlinked roots
  and preexisting job identities fail closed.
- Opaque content-digest filenames, create-new writes, owner-private Unix modes,
  and exact pre/post byte verification.
- Shell-free launch with an empty environment and the private job as CWD.
- One bounded control frame and one bounded complete response; bounded stderr,
  fixed wall timeout, direct-child kill/reap, and exact job cleanup.
- Complete no-op accounting with zero findings. Crash, timeout, input mutation,
  output flood, malformed output, identity mismatch, and staging collision
  produce no promoted partial result.

## Honest posture

The frozen `iar-application-supervisor-v1` profile records process and staging
as present. Its closed posture fixes `os_confined`, `vm_confined`, and
`network_denial_verified` false. It also names network-denial and descendant-
containment, executable-substitution-race, and staged-input-immutability
limitations. This baseline is not a sandbox and does not demonstrate safe
execution of a compromised native analyzer. The IAR-1B OS-confinement checkpoint
remains pending.

## Explicit non-claims

No ClamAV, YARA, parser, scanner, ruleset, signature database, updater,
reputation provider, credential, network request, upload, real hostile artifact,
quarantine, repository execution, finding, or safety claim is present.

## Gate-to-evidence matrix

| Application-enforced gate | Authoritative evidence | Result |
| --- | --- | --- |
| Fixed profile, fixture, and sidecar are exact | `context-conformance` profile/provenance test plus `check-contracts.rb` | Pass |
| Exact executable identity and no symlink substitution | `executable_pin_and_fresh_job_identity_are_mandatory` and `symlinked_worker_executable_is_rejected_before_staging` | Pass, with post-check/pre-exec substitution race still explicit |
| Fresh private staging outside source/cache roots | staging collision, symlink-root, root-permission, and excluded-root process tests | Pass |
| Exact bounded artifact staging and rehash | protocol resource tests plus no-op and input-mutation process tests | Pass |
| Bounded shell-free transport and cleared environment | implementation review, output-flood/malformed-output tests, and fixed profile | Pass for the first-party synthetic worker |
| Wall timeout, direct-child reap, and exact cleanup | crash/timeout fault matrix and empty-root assertions | Pass for the direct child |
| Complete source-free audit with no retained input or authority | no-op process test and closed audit schema fixtures | Pass |
| OS/VM isolation, network denial, unrelated-handle closure, immutable input, and descendant containment | closed confinement schema and fixed limitations | Not claimed; pending OS-confinement checkpoint |

This matrix evaluates the application-enforced baseline only. It does not close
the PRD's IAR-1B isolation gate, which also requires platform-specific CPU,
memory, process-count, disk, handle, process-tree, filesystem, and network
confinement evidence on every admitted host.

## Reproduction

```sh
cargo test -p context-analyzer-runner
cargo test -p context-conformance --test schema_conformance
ruby scripts/check-contracts.rb
./scripts/check.sh
```
