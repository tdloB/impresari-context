# ADR-0092: Freeze The Windows Native Confinement Contract Before Worker Launch

- Status: Accepted for implementation; worker confinement remains gated
- Date: 2026-08-31
- Decider: Aaron Boldt through the accepted ADR-0088 roadmap sequence

## Context

ADR-0088 selects LPAC/AppContainer plus Job Objects for Windows IAR-1B
feasibility, but a design document alone cannot show that the selected target
exposes those primitives or that their identity and cleanup contracts are
usable without administrator authority.

Attempting the complete worker boundary first would combine profile lifecycle,
tokens, launch attributes, mitigations, ACLs, handles, Job Objects, network,
resources, descendants, and cleanup into one ambiguous result.

## Decision

Freeze the complete intended target profile, then admit only a smaller first
checkpoint on a fresh GitHub-hosted Windows 2025 x86-64 VM:

1. observe exact Windows build and NTFS;
2. verify required native APIs are present;
3. create/query an empty unnamed Job Object with kill-on-close and one active
   process, with breakaway absent;
4. create one unique zero-capability AppContainer profile, derive and compare
   its SID, release both SID allocations, and delete the profile;
5. emit a closed receipt that keeps every worker and confinement claim false.

The probe must not launch a worker or receive repository-derived input. A
passing receipt is API and lifecycle preflight evidence only.

## Consequences

- Native assumptions fail early and attributably.
- The later broker and worker matrix consume one exact resource/identity
  contract rather than inventing limits during implementation.
- The CI runner receives one transient per-user profile mutation; cleanup is
  mandatory and the VM is ephemeral.
- Windows IAR-1B, OS confinement, network denial, resource enforcement,
  production admission, and analyzer execution remain false.

## Alternatives

- Documentation-only admission: rejected because it is not effective-host
  evidence.
- Full worker launch immediately: rejected because failures would be
  multi-causal and cleanup risk would be harder to bound.
- Experimental `CreateProcessInSandbox`: deferred because it is a newer
  experimental API with a separate FlatBuffer contract and support surface;
  ADR-0088's documented primitives remain the stable first candidate.
- Administrator service or VM: unchanged from ADR-0088 and out of scope.

## Revisit triggers

Revisit before a child process, LPAC launch, file ACL, handle inheritance,
process mitigation, network probe, resource fault, descendant, real analyzer,
production package, another Windows build/architecture, or experimental
consolidated sandbox API enters scope.
