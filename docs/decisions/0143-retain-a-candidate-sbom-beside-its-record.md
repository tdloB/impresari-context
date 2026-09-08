# ADR-0143: Retain a Candidate SBOM Beside Its Record

- Status: Accepted
- Date: 2026-09-07
- Related PRD: [Retained Candidate SBOM](../product/retained-candidate-sbom-prd.md)
- Architecture: [Retained Candidate SBOM](../architecture/retained-candidate-sbom-ard.md)
- Refines: [ADR-0017](0017-v0.1-release-assurance-policy.md)

## Context

No dependency update could pass the repository gate, and the reason was not a
missing step.

`artifacts/sbom.spdx.json` served two purposes that cannot both hold.
`scripts/check-sbom.rb` regenerates it from `Cargo.lock` and requires a
byte-for-byte match, so it must move with every dependency.
`scripts/check-macos-vm-ephemeral-product-candidate.rb` pinned its SHA-256 as
frozen evidence for the macOS candidate built at `aca6567`, so it must never
move.

Measured on a rebased dependency branch:

| `artifacts/sbom.spdx.json` | `check-sbom.rb` | candidate check |
| --- | --- | --- |
| as committed | fails | passes |
| regenerated from the new lock | passes | fails |

Three Dependabot pull requests were blocked, and every future one would have
been. A contributor could not resolve it, because no content of that file
satisfies both checks once the lock changes.

The candidate record's claim was never wrong. It states what
`artifacts/sbom.spdx.json` held at `aca6567`, which remains true of `aca6567`.
The error was verifying that historical claim against the working tree, where
the file has since moved for unrelated reasons.

## Decision

Retain the bytes the record attests as
`platform/macos-vm-feasibility/product-sbom-v1.spdx.json`, and verify the
candidate's SBOM evidence against the retained copy rather than the live
inventory.

Every other piece of that candidate's evidence — its license, vulnerability, and
reproducibility dispositions, and the guest SBOM — was already retained in that
directory. The product SBOM was the only one pointing at a moving file.

`artifacts/sbom.spdx.json` remains the current dependency inventory and is still
regenerated and compared against `Cargo.lock`.

## Consequences

A dependency update now passes the gate after regenerating the inventory alone.
Verified by applying #287's exact lockfile change, regenerating, and running the
full gate.

No frozen record changes. The record, its conformance fixture, the profile
pinning the record's digest, the profile's checksum sidecar, and the release
identity contract are untouched, and their digests in the checker are unchanged.
The retained artifact is byte-identical to what the record attests, verified
equal before retention, so nothing is re-attested.

The candidate check still fails if the retained artifact is altered by a single
byte. What it no longer does is fail because an unrelated dependency moved.

This removes a false blocker. It admits no dependency update on its own; each is
still judged on its merits, and #289 is held for an unrelated reason.

## Alternatives considered

**Update the frozen digests to the new SBOM.** Rejected, and this is the
important one. The license, vulnerability, and reproducibility dispositions
bound to that digest were never evaluated against the new dependency graph, so
updating it would assert something unchecked. Hand-editing release evidence to
make CI pass is the failure this structure exists to prevent.

**Repoint the record at the retained artifact.** Considered and implemented
first, then reverted. The record is cross-bound through five interlocking
digests — record, fixture, profile, profile sidecar, checker constants —
specifically so re-freezing it is a deliberate human act. Rewriting all five to
correct a checker's target inverts that: none of those records was wrong.

**Freeze `artifacts/sbom.spdx.json` and publish the live inventory elsewhere.**
Rejected: the conventional path would then be stale, which misleads every reader
who is not thinking about release evidence.
