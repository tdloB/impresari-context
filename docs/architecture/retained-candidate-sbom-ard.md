# Retained Candidate SBOM — Architecture Requirements and Design

- ARD ID/version: IC-RCS-ARD-143 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-07.
- Governing PRD: [IC-RCS-143](../product/retained-candidate-sbom-prd.md).
- Decision:
  [ADR-0143](../decisions/0143-retain-a-candidate-sbom-beside-its-record.md).

## One file, two jobs

```text
                    ┌── check-sbom.rb ──────────▶ must equal Cargo.lock
artifacts/sbom.spdx.json
                    └── candidate check ────────▶ must equal a 2026 digest
```

The first requires the file to move with every dependency. The second requires
it never to move. Once `Cargo.lock` changes there is no content that satisfies
both, so the gate is unsatisfiable rather than merely failing.

## The claim was never wrong; the verification was

The candidate record states that at source revision `aca6567`,
`artifacts/sbom.spdx.json` held digest `bb249501…`. That statement is true of
`aca6567` and stays true forever. Nothing about a later dependency bump
falsifies it.

What was wrong is how it was checked: the script hashed the **working tree's**
copy of that path. A historical claim verified against present-day bytes fails
as soon as the present moves, which is not evidence of anything except that time
passed.

## The fix

Retain the bytes the record attests, beside the candidate's other frozen
evidence, and verify against those:

```text
platform/macos-vm-feasibility/
  product-sbom-v1.spdx.json            ← retained, byte-identical, verified
  product-license-disposition-v1.json  ← already retained here
  product-vulnerability-disposition-v1.json
  product-reproducibility-disposition-v1.json
  ephemeral-product-candidate-record-v1.json
```

Every sibling of this artifact was already retained in that directory. The
product SBOM was the one piece of the candidate's evidence pointing at a live
file, and that was the defect.

## What is deliberately not changed

No frozen record moves. The record, its conformance fixture, the profile that
pins the record's digest, the profile's checksum sidecar, and the release
identity contract are all untouched, and their digests in the checker are
unchanged.

That is a design constraint, not a convenience. The record is cross-bound
through five interlocking digests specifically so that re-freezing it is a
deliberate human act. Repointing the record would have required rewriting all
five, and none of them was wrong.

The retained artifact is byte-identical to what the record already attests —
`bb249501…`, verified equal before retention — so nothing is re-attested and no
disposition is claimed to hold for a dependency graph it was not evaluated
against.

## Preserved properties

The candidate check still fails if the retained artifact is altered by a single
byte; it is verified with the same `exact` helper, against the same digest, as
every other piece of candidate evidence. What it no longer does is fail because
an unrelated dependency moved.

`artifacts/sbom.spdx.json` remains the current dependency inventory and is
still regenerated and compared byte-for-byte against `Cargo.lock`.
