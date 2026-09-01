# macOS Local-VM Unsigned Candidate Composition PRD

- Status: Implemented; source-free readiness only
- Architecture: [macOS Local-VM Unsigned Candidate Composition ARD](../architecture/macos-local-vm-unsigned-candidate-composition-ard.md)
- Decision: [ADR-0113](../decisions/0113-compose-macos-unsigned-candidate-readiness-without-overclaim.md)

## Outcome

Determine whether the retained ADR-0110 product metadata and ADR-0112 guest
metadata form one closed prospective macOS app payload without claiming that a
complete candidate was assembled or retained.

## Requirements

- Bind the ADR-0109 release contract, ADR-0107 package contract, ADR-0110
  product record, ADR-0108 synthetic tree metadata, ADR-0111 guest contract,
  ADR-0112 guest record, and guest metadata seal by exact digest.
- Project exactly eight future regular files: four product executables,
  `Info.plist`, the metadata seal, and two guest resources.
- Calculate one deterministic prospective compound identity over sorted paths,
  required modes, bytes, and SHA-256 identities.
- Preserve that product and guest bytes existed only in separate deleted
  rehearsals and that executable modes were not verified in one app tree.
- Keep every materialization, assembly, Apple identity, distribution, runtime,
  analyzer, production, and IAR-1B claim false.

## Acceptance

An offline checker reproduces the exact projection and prospective digest,
verifies every source binding and unresolved gate, and emits a schema-valid
readiness receipt. A negative fixture rejects a complete-candidate overclaim.
