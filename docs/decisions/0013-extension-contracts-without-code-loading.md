# ADR-0013: Extension contracts without code loading

- Status: Accepted for Slice D implementation
- Date: 2026-08-22
- Scope: Declarative extension compatibility, local digest policy, and untrusted output intake

## Decision

Slice D begins with contracts, not a general plugin runtime. A closed manifest
classifies an extension as parser, retriever, analyzer, exporter, or transport;
pins its artifact digest and contract version; declares every requested
capability; bounds output; and records determinism, model dependency, retention,
and output fields.

The v1 policy can enable only bounded output submission from an exactly pinned
artifact declaration that requests zero privileged capabilities. Filesystem,
cache, process, network, environment, model, persistence, and artifact execution
remain unimplemented and denied. Digest approval is a local artifact pin, not a
verified publisher identity or signature claim.

Submitted output is length-checked before parsing, decoded through a closed
envelope, matched to the exact manifest/decision, labeled untrusted derived
data, and prevented from claiming exact-source authority. Invalid, excessive,
unauthorized, spoofed, or authority-claiming output becomes a metadata-only
quarantine record containing no raw output.

This decision does not authorize loading native/Wasm code, spawning processes,
reading workspaces, using models, reaching networks, persisting extension data,
or exposing MCP/HTTP. Each requires the future-scope threat analysis and an
additional ADR before implementation.

## Consequences

- Integrators can design against stable parser/retriever/analyzer/exporter/
  transport vocabulary without enlarging the runtime attack surface.
- The core can test manifest pinning, authority separation, and output
  normalization before any plugin execution mechanism exists.
- A digest proves byte identity only; publisher authenticity and revocation
  remain unresolved for a future loader.
- MCP is not included merely because `transport` is a manifest kind.

## Verification

- Closed JSON schemas and positive/negative conformance fixtures.
- Exact-pin, unpinned, and privileged-capability decision tests.
- Malformed, unknown-field, oversized, identity-spoofed, and exact-authority
  output quarantine tests.
- Quarantine serialization checks that raw hostile output is absent.

## Review trigger

Any artifact loading/execution, privileged grant, publisher trust/signature,
update/revocation, MCP/HTTP endpoint, model use, or extension persistence.
