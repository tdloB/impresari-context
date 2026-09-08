# Retained Candidate SBOM PRD

## Document Control

- PRD ID/version: IC-RCS-143 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-07.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Retained Candidate SBOM ARD](../architecture/retained-candidate-sbom-ard.md).
- Governing decision:
  [ADR-0143](../decisions/0143-retain-a-candidate-sbom-beside-its-record.md).

## Problem

No dependency update can pass the repository gate.

`artifacts/sbom.spdx.json` is asked to be two incompatible things at once:

1. **The current dependency inventory.** `scripts/check-sbom.rb` regenerates it
   from `Cargo.lock` and requires a byte-for-byte match, so it must move
   whenever the lock moves.
2. **Frozen release evidence.** `scripts/check-macos-vm-ephemeral-product-candidate.rb`
   pins its SHA-256 as evidence for the macOS candidate built at source revision
   `aca6567`, so it must never move.

Measured on a rebased dependency branch, the two are mutually exclusive:

| `artifacts/sbom.spdx.json` | `check-sbom.rb` | candidate check |
| --- | --- | --- |
| as committed | fails | passes |
| regenerated from the new lock | passes | fails |

Three Dependabot pull requests are blocked on this, and every future one will
be. The blockage is not a missing step a contributor can perform; there is no
state of that file which satisfies both checks once the lock changes.

## Product Outcome

The candidate's SBOM is retained beside the candidate's other evidence, where
its siblings already live, and the live inventory is free to track the lock.

## Functional Requirements

1. The macOS candidate record names a retained SBOM held with the candidate's
   other frozen evidence, not the live inventory file.
2. The retained artifact is byte-identical to the SBOM the record already
   attests. No digest, disposition, or claim in the record changes.
3. `artifacts/sbom.spdx.json` remains the current dependency inventory and is
   regenerated with the lock.
4. A dependency update passes the gate after regenerating the inventory alone.
5. The candidate check continues to fail if the retained artifact is altered.

## Acceptance Criteria

- A simulated dependency bump passes the full gate after regenerating
  `artifacts/sbom.spdx.json` and nothing else.
- The retained artifact's digest equals the record's `spdx_2_3_sbom_sha256`.
- A test proves the candidate check fails when the retained artifact changes.
- The full repository gate passes.

## Non-Goals

- Re-evaluating the candidate's license, vulnerability, or reproducibility
  dispositions. Those are bound to a dependency graph this record does not
  touch.
- Admitting any dependency update. This removes a false blocker; each bump is
  still judged on its own merits.
- Changing what the candidate record claims.
