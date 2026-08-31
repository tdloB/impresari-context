# Impresari Context — Windows Native Feasibility Contract PRD

- Status: Accepted for implementation; worker confinement remains gated
- Date: 2026-08-31
- Authority: ADR-0088 accepted synthetic-feasibility sequence
- Architecture: [Windows native feasibility contract ARD](../architecture/windows-native-feasibility-contract-ard.md)
- Decision: [ADR-0092](../decisions/0092-freeze-windows-native-feasibility-contract.md)

## Objective

Freeze an exact, no-authority Windows resource and identity contract, then prove
on one fresh Windows 2025 x86-64 host that the native primitives selected by
ADR-0088 are available before any synthetic worker is launched.

## User outcome

Windows support can advance through small, attributable gates. A passing
preflight means only that the host can create and query an empty Job Object and
complete a zero-capability AppContainer profile lifecycle. It cannot be
misread as a working analyzer sandbox.

## Scope

- One digest-bound target profile for LPAC/AppContainer identity, Job Object
  limits, process mitigations, staged input, bounded output, and cleanup.
- One Windows-only, first-party native probe compiled from reviewed source on a
  fresh GitHub-hosted Windows 2025 x86-64 VM.
- Exact Windows build, architecture, NTFS, required API, empty Job Object, and
  AppContainer create/derive/delete observations.
- A bounded, source-free receipt with every unmeasured confinement claim false.
- Deterministic schemas, fixtures, provenance, and cross-platform static checks.

## Non-goals

- Launching a worker, creating an LPAC token, applying staged-file ACLs,
  assigning a process to a Job Object, or testing network/path/resource denial.
- Repository-derived input, real analyzers, executable repository artifacts,
  installer execution, PowerShell execution, administrator services, Windows
  Sandbox, Hyper-V, or kernel drivers.
- Production admission, Windows IAR-1B, packaging, signing, publication, or
  automatic fallback.

## Acceptance criteria

- The profile is schema-valid, checksum-bound, and exact.
- The live probe runs only when GitHub identifies a fresh hosted runner.
- The target is Windows x86-64 on NTFS and records the exact OS build.
- Required AppContainer, launch-attribute, mitigation, and Job Object APIs are
  present.
- One unique profile is created with zero capabilities, its SID equals an
  independently derived SID, all SID allocations are freed, and the profile is
  deleted before success.
- One unnamed empty Job Object accepts kill-on-close and one-active-process
  limits, returns them exactly, and contains no breakaway flag.
- Any unsupported host, partial cleanup, identity mismatch, API failure, schema
  drift, or claim escalation fails closed.
- Worker launch, network denial, path boundary, resource enforcement,
  descendant containment, full cleanup, OS confinement, production, and
  analyzer claims remain false.

## Next gate

The next separately reviewed checkpoint may launch only the pinned synthetic
worker suspended, apply the exact LPAC/AppContainer and Job Object boundary,
and measure denial and cleanup. This preflight does not authorize that gate by
itself.
