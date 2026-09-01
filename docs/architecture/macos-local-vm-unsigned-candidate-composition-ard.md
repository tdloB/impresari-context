# macOS Local-VM Unsigned Candidate Composition ARD

- Status: Implemented; candidate assembly remains gated
- PRD: [macOS Local-VM Unsigned Candidate Composition PRD](../product/macos-local-vm-unsigned-candidate-composition-prd.md)
- Decision: [ADR-0113](../decisions/0113-compose-macos-unsigned-candidate-readiness-without-overclaim.md)

## Boundary

This is a pure metadata composition. It launches no process, reads no
workspace, accesses no network or credential, and writes no app tree. Exact
prior records are evidence inputs, not executable inputs.

## Composition

The projection contains eight future regular files. Product executable
identities come from ADR-0110; the future `Info.plist` identity comes from the
deterministic ADR-0108 synthetic assembly; the metadata seal is copied exact;
and the two guest identities come from ADR-0112. Paths are closed by ADR-0107.

The prospective compound identity canonicalization is:

1. labelled source revision, product version, and target rows;
2. sorted bundle-path rows containing `path`, `file`, required mode, bytes, and
   prefixed SHA-256;
3. tab-delimited fields, LF-terminated rows, then SHA-256.

## Evidence distinction

The record distinguishes measured byte identities from future filesystem
modes and co-custody. Product and guest bytes were created in different
private roots at different times and were deleted. Therefore this checkpoint
cannot satisfy the existing materialized unsigned-candidate schema.

## Next gate

A later complete ephemeral candidate rehearsal must rebuild all components,
assemble the exact app tree in one private root, verify modes and the compound
identity, execute nothing, and delete everything before a real unsigned
candidate record may be admitted.
