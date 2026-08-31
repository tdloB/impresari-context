# IAR-1B Windows Native Contract Preflight

- Status: Contract frozen; native hosted evidence pending
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

## Next evidence

After this preflight passes, a separate architecture/security checkpoint must
launch only the pinned synthetic worker suspended and measure the complete
LPAC/AppContainer, Job Object, handle, mitigation, path, network, resource,
descendant, and cleanup matrix. No real analyzer may run first.
