# Host-Executed Context Hooks — Architecture Requirements and Design

- ARD ID/version: IC-HEH-ARD-126 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Governing PRD: [IC-HEH-126](../product/host-executed-context-hooks-prd.md).
- Decision:
  [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md).

## The distinction this design rests on

```text
                 who performs the operation?
                             │
        ┌────────────────────┴────────────────────┐
        │                                         │
   the product                               the host
        │                                         │
  control plane                             evidence plane
  (execution authority)                     (no authority)
        │                                         │
   not this design                          this design
```

A context layer that runs commands must be trusted with execution. A context
layer that only *answers about* operations the host performs need not be. The
second is strictly weaker in capability and strictly stronger in deployability,
and it is the position Impresari already asserts in `SEC-INV-007`.

Hooks are the mechanism that lets a no-authority product still displace work.

## Data flow

```text
host decides to read / search                 host has already executed
        │                                              │
        ▼                                              ▼
   offer request                                  offer output bytes
        │                                              │
        ▼                                              ▼
   ┌──────────────────────────────────────────────────────────┐
   │ hook process: env_clear, bounded stdio, deadline,         │
   │ no spawn, no socket, no workspace write                   │
   └──────────────────────────────────────────────────────────┘
        │                                              │
        ▼                                              ▼
  spans + hashes + omissions              selected bytes + omissions
        │                                              │
        └──────────────► host decides whether to use ◀──┘
```

The arrow never reverses. Impresari receives, transforms, and returns. It does
not call back, request, instruct, or block.

## Trust zone

Hooks introduce **Z8 — host hook channel**, adjacent to Z4 (consumer interface)
and inheriting its rules. Payloads arriving on Z8 carry the highest sensitivity
of their contents and are classified as untrusted repository-derived content on
entry, exactly as workspace bytes are in Z2.

The critical property: Z8 is an *inbound data* zone, not a capability zone.
Nothing arriving on it can grant, widen, or redirect authority. A payload that
contains the text of a policy, a tool schema, an instruction, or a path is still
only bytes to be selected from.

## Why substitution is safe here but addition was not

Read substitution returns spans recovered from the already-authorized workspace
snapshot, each with an independently computed content hash and byte range, under
`SEC-INV-011`. The host can verify every returned byte against the source it
already controls. A hook therefore cannot smuggle content: anything it returns
that does not verify is rejected by the host, not by trust.

Output reduction is constrained further. The response must be a selection from
bytes the host supplied in the same exchange. Impresari cannot introduce a byte
the host did not already have, which removes the injection surface entirely for
that shape.

## Accounting

Every response declares the substitution it performed: bytes offered, bytes
returned, and what was omitted and why. This exists so a measurement can
attribute savings honestly rather than inferring them, and so the governing
objective's rule — treatment must not read more than baseline — is checkable
from the record rather than from belief.

## Preserved invariants

`SEC-INV-002` (never writes to the source workspace), `SEC-INV-003` (repository
content is data), `SEC-INV-007` (no execution, network, or telemetry),
`SEC-INV-008` (source excluded from logs), `SEC-INV-009` (budgets everywhere),
`SEC-INV-011` (no exact-source authority without hash and span), and
`SEC-INV-012` (consumer input cannot broaden capability) all hold unchanged.

This design deliberately adds an integration surface without adding a capability.
If a future increment gives a hook the ability to perform the operation itself,
that is a different product and requires a new decision record.

## Deferred work

Per-client adapters (Claude Code, Codex, Copilot, Cursor) map their own hook
formats onto this channel in the existing thin-adapter crates. No client gets a
forked core.
