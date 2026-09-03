# Cache-Stable Context Prefix PRD

## Document Control

- PRD ID/version: IC-CSCP-127 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Cache-Stable Context Prefix ARD](../architecture/cache-stable-context-prefix-ard.md).
- Governing decision:
  [ADR-0127](../decisions/0127-emit-a-cache-stable-context-prefix.md).

## Problem

A context packet delivered once is billed on every subsequent request, because
every provider re-sends conversation history each turn.

Measured on a single instance: a 10,453-byte packet, approximately 2,613 tokens,
across 23 provider requests is roughly 60,099 re-billed input tokens — about 46%
of that arm's entire input increase over its baseline.

Providers already solve this. A stable prefix that never changes can be cached,
and cached input is billed at a fraction of fresh input. A context packet placed
at the front of a conversation and never modified is close to the ideal shape
for that mechanism.

Impresari does not currently declare whether its packet is safe to cache, and
offers no key by which a client could detect that the packet has changed. A
careful client therefore cannot cache it, and a careless one might cache a
packet that has silently changed underneath it.

Arithmetic on the measured instance suggests caching would recover roughly 40%
of that arm's cost gap. It does not close the gap, and it is not a substitute
for [IC-HEH-126](host-executed-context-hooks-prd.md). It is a discount that
becomes worth collecting once the product is otherwise winning.

## Product Outcome

Impresari emits a context packet whose stable portion is byte-identical for
identical inputs, is separated from anything volatile, and carries an explicit
cache key and stability declaration a client can rely on.

The product performs no caching itself and issues no provider request. It makes
a packet *cacheable* and says so; the client decides.

## Functional Requirements

1. For an identical workspace snapshot, task text, budget, policy, and product
   identity, the stable portion of the packet must serialize to identical bytes.
2. Separate the stable portion from any volatile portion. Timestamps, request
   identifiers, elapsed measurements, and per-call accounting are volatile and
   must not appear inside the stable portion.
3. Emit a cache key derived from exactly the inputs that determine the stable
   bytes. Any change to those inputs must change the key; no change to them may
   change it.
4. Declare stability explicitly, so a client can distinguish "safe to cache
   under this key" from "not declared stable."
5. Behaviour must not change when a client ignores the declaration entirely.
   The packet remains valid and complete on its own.
6. The cache key must disclose nothing. It is derived from identities already
   safe in source-free records and must not encode task text, source bytes,
   paths, queries, or excerpts.
7. Impresari issues no provider request, holds no provider credential, and
   implements no provider cache protocol.

## Acceptance Criteria

- Building twice with identical inputs yields byte-identical stable portions and
  an identical cache key.
- Changing the workspace snapshot, task text, budget, policy, or product
  identity changes the key.
- Changing only a timestamp or request identifier does not change the key, and
  those values do not appear in the stable portion.
- A static check proves the stable portion contains no volatile field.
- A property test proves the key encodes no source-derived text.
- The full repository gate passes.

## Non-Goals

- Implementing provider prompt-cache calls. That is client work.
- Changing the evaluator's `StrictCold` profile. Cold measurement remains the
  correct default for controlled comparison; a production-realistic profile is
  separate, later, and out of scope here.
- Any claim that caching improves correctness. It changes price, not content.
