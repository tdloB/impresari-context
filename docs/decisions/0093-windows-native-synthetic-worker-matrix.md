# ADR-0093: Require A Suspended LPAC Synthetic Worker Matrix Before Windows Admission

- Status: Accepted for implementation; admission remains gated
- Date: 2026-08-31
- Decider: Aaron Boldt through the accepted ADR-0088 roadmap sequence

## Context

ADR-0092 proved native API and empty-object lifecycle availability on one
Windows Server 2025 build without launching a child. It cannot establish that
an actual worker begins inside the intended boundary, that exact staged input
is the only useful host data it can read, or that resource and cleanup controls
remain effective under faults.

Windows also creates writable per-profile AppContainer storage by default,
while the frozen Impresari profile promises zero writable path-backed bytes.
That mismatch must be measured and closed rather than hidden.

## Decision

Before any Windows IAR-1B or real-analyzer claim, run one complete original-
synthetic matrix with an exact first-party worker created suspended under:

- a fresh zero-capability LPAC/AppContainer identity;
- exact read/execute staging ACLs and denied profile-storage write;
- exact inherited pipe handles only;
- compatible frozen creation-time mitigations and child-process restriction;
- a fully configured and queried Job Object assigned before resume;
- bounded source-free result transport and idempotent complete cleanup;
- a second fresh identity proving cross-job isolation.

No external network destination, existing credential, repository artifact, or
real analyzer enters the checkpoint. A passing receipt remains exact-host
synthetic feasibility with `os_confined=false` and production false.

## Consequences

- Pre-resume containment and zero-writable-path semantics become measured
  requirements.
- The broker adds a reviewed Windows unsafe/FFI surface and must fail closed on
  every partial launch and cleanup path.
- Profile-storage hardening may prove infeasible. That result rejects this
  native candidate rather than weakening the frozen profile silently.
- Independent-host evidence, compatibility withdrawal, signing, packaging,
  production admission, and real analyzers remain later gates.

## Alternatives

- Promote the no-worker preflight: rejected because no process was confined.
- Launch normally then assign the Job Object: rejected because worker code may
  execute before resource and descendant containment.
- Allow AppContainer profile writes: rejected because it contradicts the
  frozen zero-writable-path contract.
- Package/MSIX or VM first: deferred by ADR-0088 until native feasibility is
  measured.

## Revisit Triggers

Revisit before another Windows build/architecture, changed mitigation or ACL
set, writable path-backed storage, independent-host admission, signing,
installer lifecycle, production support, real analyzer, or VM fallback enters
scope.
