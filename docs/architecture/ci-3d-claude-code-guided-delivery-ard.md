# Impresari Context — CI-3d Claude Code Guided-Delivery ARD

- Status: Accepted for implementation; live admission pending
- Date: 2026-08-29
- Product requirement: [CI-3d PRD](../product/ci-3d-claude-code-guided-delivery-prd.md)
- Decision: [ADR-0061](../decisions/0061-claude-safe-mode-programmatic-guided-delivery.md)

## Context and boundary

Claude Code supports non-interactive print mode, streaming JSON, safe mode,
empty built-in tool selection, disabled slash commands, and disabled session
persistence. Safe mode disables repository/user instruction and extension
surfaces while preserving the client's authentication and provider transport.
That combination supports deterministic packet delivery without granting
Claude repository authority.

The boundary is one process and one reviewed packet. Impresari owns packet
selection and verification. Claude owns only its intrinsic hosted-model
request. Model output is parsed for terminal evidence and discarded.

## Components

- `context-adapters` admits the exact Claude identity tuple.
- `context-claude-code` owns the immutable envelope, preview rehydration,
  process boundary, event validator, and bounded receipt.
- `context-cli` exposes `client delivery claude preview` and separately gated
  `client delivery claude apply`.
- `claude-code-delivery.schema.json` publishes the envelope, preview, and
  receipt contract.

## Trust and authority rules

- Preview is pure and serializable; canonical packet bytes are omitted and
  re-derived on apply.
- Apply requires `--apply` and the expected packet identity.
- The Claude binary, runtime parent, and authenticated user home must be
  absolute, canonical, and separated; the home is used in place.
- The child environment is cleared except `HOME` and the caller's `PATH`.
- The fixed invocation uses `--safe-mode --print --tools ""
  --disable-slash-commands --no-session-persistence --input-format stream-json
  --output-format stream-json --verbose`.
- The prompt travels over stdin, not argv. The parser requires its exact echoed
  bytes, empty initialized tool and MCP lists, zero tool-use blocks, one success
  result, bounded stdout/stderr, and a zero exit.
- The source workspace path and cache path never enter the child process.
- Cleanup removes only the runtime directory created by the adapter.

## Failure handling

Pre-I/O failures produce `no_delivery`. Once a process starts, incomplete or
ambiguous evidence produces `degraded`; tool use always degrades. No fallback
to interactive mode, hooks, native reads, broader flags, or another version is
allowed.

## Verification

Deterministic unit and CLI tests cover every pre-I/O binding and negative event
case. Live admission adds two immutable rehearsal records only after explicit
provider disclosure authorization. Exact-version drift triggers lifecycle
maintenance rather than silent admission.
