# ADR-0011: Process-local session packet references

- Status: Accepted for Slice C implementation
- Date: 2026-08-22
- Scope: Temporary packet references shared by authorized local consumers

## Decision

Slice C begins with an in-memory session store. A consumer explicitly opens a
bounded session, attaches already-valid immutable packets, resolves them only
under the same opaque consumer identity, and explicitly closes the session.
Dropping the process store closes every session.

References preserve packet, workspace, snapshot, purpose, and canonical-size
identity. They add no workspace, execution, routing, approval, or durable-memory
authority. The store performs no filesystem or network access and does not
silently promote observations into knowledge.

Persistent or cross-process sessions require a later decision covering
encryption, retention, revocation, crash recovery, multi-user authorization,
and secure deletion. This milestone makes no memory-zeroization guarantee after
ordinary allocator deallocation and records that limitation as residual risk.

## Verification

- Consumer isolation and wrong-consumer denial.
- Packet integrity validation before attachment and resolution.
- Duplicate, count, and byte-limit enforcement.
- Reference invalidation when the session closes.
- Closed JSON Schema conformance for reference metadata.
