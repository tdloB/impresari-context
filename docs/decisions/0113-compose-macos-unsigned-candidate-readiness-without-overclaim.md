# ADR-0113: Compose macOS Unsigned Candidate Readiness Without Overclaim

- Status: Implemented; source-free readiness only
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Unsigned Candidate Composition PRD](../product/macos-local-vm-unsigned-candidate-composition-prd.md)
- Architecture: [macOS Local-VM Unsigned Candidate Composition ARD](../architecture/macos-local-vm-unsigned-candidate-composition-ard.md)

## Context

ADR-0110 proved four product identities and ADR-0112 proved two guest
identities, but each rehearsal deleted its outputs. Combining their metadata
can expose whether the future app projection is closed; it cannot prove that a
complete candidate existed.

## Decision

Create a source-free readiness record with an exact eight-file projection and
prospective compound identity. Digest-bind every prior contract and record,
and mark every member as separately evidenced rather than co-materialized.

Do not populate the existing unsigned-release-candidate schema. That schema
requires `candidate_materialized` and `release_identity_bound` to be true, and
using it now would launder separate rehearsals into a nonexistent candidate.

## Consequences

- The next complete ephemeral assembly has an exact input projection and
  expected compound identity.
- Missing co-custody, filesystem-mode, complete-tree, and assembly evidence is
  explicit.
- No process, network, credential, app, Apple identity, signing, notarization,
  cask, installation, VM, analyzer, production, or IAR-1B authority is added.

## Revisit triggers

Revisit before changing any member, path, mode, identity, source revision,
canonicalization, or package role, or before materializing a complete candidate
or crossing any Apple identity, distribution, runtime, analyzer, production,
or IAR-1B boundary.
