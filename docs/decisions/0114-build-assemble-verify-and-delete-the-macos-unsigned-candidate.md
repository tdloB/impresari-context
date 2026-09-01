# ADR-0114: Build, Assemble, Verify, And Delete The macOS Unsigned Candidate

- Status: Implemented for one bounded rehearsal; distribution and runtime remain gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Ephemeral Unsigned Release Candidate PRD](../product/macos-local-vm-ephemeral-unsigned-release-candidate-prd.md)
- Architecture: [macOS Local-VM Ephemeral Unsigned Release Candidate ARD](../architecture/macos-local-vm-ephemeral-unsigned-release-candidate-ard.md)

## Context

ADR-0113 established that the separately deleted ADR-0110 and ADR-0112 outputs
form a closed prospective app projection, but correctly refused to claim that a
complete candidate had existed.

## Decision

Perform one bounded macOS arm64 rehearsal under a single fresh private root.
Rebuild the exact four product outputs, authenticate and rebuild the exact two
synthetic guest outputs, assemble the exact eight-file app, reproduce both
frozen identities, inspect without execution, and delete the complete root.

Populate the frozen ADR-0109 unsigned-candidate schema only after simultaneous
custody, app-tree closure, filesystem modes, identities, and cleanup have all
been verified. Retain only source-free metadata and fixtures.

## Consequences

- The unsigned candidate and release identity may be recorded as temporarily
  materialized and bound.
- No runnable candidate is retained or distributed.
- Linker ad-hoc identity remains explicitly distinct from Developer ID signing.
- Apple credentials, signing, notarization, cask lifecycle, installation,
  publication, VM launch, analyzer execution, production, and macOS IAR-1B stay
  false and require later decisions.

## Alternatives

- Compose prior metadata only: completed by ADR-0113 but insufficient for the
  frozen candidate schema.
- Retain the unsigned app: rejected because signed-artifact custody and expiry
  are not yet admitted.
- Sign in the same rehearsal: rejected because Apple credential access is a
  separate manual boundary.

## Revisit triggers

Revisit before changing an input or identity, retaining candidate bytes,
creating an archive or cask, accessing Apple identity, signing, notarizing,
installing, publishing, launching a VM, executing an analyzer, or making a
production or macOS IAR-1B claim.
