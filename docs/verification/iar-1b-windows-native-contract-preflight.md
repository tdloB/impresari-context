# IAR-1B Windows Native Contract Preflight

- Status: Native hosted no-worker preflight passed; worker confinement remains gated
- Date: 2026-08-31
- Decisions: [ADR-0088](../decisions/0088-windows-native-analyzer-confinement.md), [ADR-0092](../decisions/0092-freeze-windows-native-feasibility-contract.md)

## Checkpoint

This checkpoint binds the intended Windows LPAC/AppContainer, Job Object,
mitigation, staging, resource, output, and cleanup profile before a worker is
launched. A dedicated Windows 2025 x86-64 hosted job must prove only native API
availability, NTFS, empty Job Object set/query behavior, and a unique
zero-capability AppContainer create/derive/delete lifecycle.

## Claim boundary

Passing this checkpoint does not mean that a process ran inside an
AppContainer. The receipt therefore keeps synthetic worker launch, LPAC worker
launch, network denial, path denial, resource enforcement, descendant
containment, complete cleanup, OS confinement, production support, and analyzer
execution false.

## Native host

GitHub documents `windows-2025` as a fresh hosted x64 VM for each standard
public-repository job. The receipt records the exact observed Windows build
rather than treating the mutable runner label as a permanent compatibility
claim.

## Hosted evidence

PR 181 run `33361303368`, job `99393036278`, passed from source commit
`393bc0b40d57fad0a5cb88cfe22394148f6bf464` on a fresh GitHub-hosted
Windows Server 2025 x86-64 runner. The runner reported Windows build `26100`,
NTFS, and the exact profile identity
`sha256:6b8f614387fc97321497e6b725213b9ee3c2159f3d1384fb800ffbe8af490a73`.

The native probe verified the required API exports, set and queried
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` plus one active process on an empty Job
Object with breakaway disabled, and completed one unique zero-capability
AppContainer create/derive/equal-SID/free/delete lifecycle. Its validated
receipt ended with `worker_launched=false` and `os_confined=false`.

This is one hosted build observation, not a maintained Windows support range.
A changed build, filesystem, API surface, profile identity, or lifecycle result
must be re-evaluated rather than inheriting this evidence.

## Next evidence

The next separate architecture/security checkpoint must
launch only the pinned synthetic worker suspended and measure the complete
LPAC/AppContainer, Job Object, handle, mitigation, path, network, resource,
descendant, and cleanup matrix. No real analyzer may run first.
