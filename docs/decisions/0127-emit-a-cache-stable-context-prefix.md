# ADR-0127: Emit a Cache-Stable Context Prefix

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Cache-Stable Context Prefix](../product/cache-stable-context-prefix-prd.md)
- Architecture: [Cache-Stable Context Prefix](../architecture/cache-stable-context-prefix-ard.md)

## Context

Every provider re-sends conversation history on each request, so a packet
delivered once is billed many times. Measured on one instance: 10,453 bytes,
roughly 2,613 tokens, across 23 requests is about 60,099 re-billed input tokens,
approximately 46% of that arm's input increase over its baseline.

Providers already offer prompt caching for a stable prefix, and a context packet
placed at the front and never modified is close to the ideal shape for it.
Impresari neither declares its packet stable nor offers a key by which a client
could detect change, so a careful client cannot cache it safely.

Two things bound the value of fixing this. Arithmetic on the measured instance
suggests caching recovers roughly 40% of that arm's cost gap and does not close
it — the remaining 54% of the increase was the agent doing more work, which no
discount addresses. And the evaluator deliberately forbids caching under
`StrictCold`, which is correct for controlled comparison and means the product
has never been measured in the configuration it would ship in.

## Decision

Emit a context packet whose stable portion is byte-identical for identical
inputs, is separated from every volatile field, and carries an explicit cache
key and stability declaration.

Derive the key only from identities already safe in source-free records:
workspace snapshot, task, budget, policy, and product identity. Never from task
text, source bytes, paths, queries, or excerpts.

Impresari performs no caching, issues no provider request, and implements no
provider cache protocol. It makes the bytes worth caching and says so; the
client decides.

## Consequences

A client can cache the prefix and detect staleness by key. Behaviour is
unchanged for a client that ignores the declaration.

Byte-identity is a stricter obligation than value-identity: field order, numeric
formatting, and collection ordering in the stable portion become part of the
contract, and changing any of them is a breaking change. The existing map,
receipt, and graph identities already carry this discipline, so the cost is
consistency rather than novelty.

This record is sequenced third deliberately. It changes price, not content, and
a discount on context that points at the wrong file is worth nothing. Recall and
hooks come first.

No security invariant changes. The key is control metadata under the existing
data classification. This record grants no execution, network, publication, or
submission authority.
