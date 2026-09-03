# Cache-Stable Context Prefix — Architecture Requirements and Design

- ARD ID/version: IC-CSCP-ARD-127 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-03.
- Governing PRD: [IC-CSCP-127](../product/cache-stable-context-prefix-prd.md).
- Decision: [ADR-0127](../decisions/0127-emit-a-cache-stable-context-prefix.md).

## Shape

```text
context packet
├── stable prefix        ── byte-identical for identical inputs
│     ├── evidence spans, hashes, paths
│     ├── disclosure map
│     └── budget and policy identities
│                     │
│                     └──▶ cache_key = H(snapshot, task, budget, policy, product)
│
└── volatile suffix     ── never cached
      ├── request and event identifiers
      ├── timestamps and elapsed measurements
      └── per-call accounting and receipts
```

A client places the stable prefix at the front of the conversation and never
modifies it. Everything that changes per request lives after it, where it cannot
invalidate the cached prefix.

## Why the product stops at "cacheable"

Impresari holds no provider credential and issues no provider request. It cannot
cache anything itself, and it should not try: a cache protocol is provider- and
client-specific, changes independently of this product, and would drag provider
concerns across a boundary that is currently clean.

What only the product can do is guarantee **that the bytes are worth caching** —
deterministic, separated, and keyed. That guarantee is the deliverable.

## Determinism requirements

Byte-identity is stronger than value-identity. Two packets with the same content
in a different field order, or with different float formatting, are not the same
bytes and will not hit a cache. The stable portion therefore requires a fixed
field order, fixed numeric formatting, and ordering of every collection by an
identity already used elsewhere in the product.

This is not a new burden. Existing map identities, receipt identities, and
graph identities already require deterministic serialization; this extends the
same discipline to the delivered prefix.

## Key derivation and disclosure

The key is a digest over the identities that determine the stable bytes:
workspace snapshot, task identity, budget identity, policy identity, and product
identity. Every one of those is already carried in source-free records.

The key must not be derived from task text, source bytes, paths, queries, or
excerpts. A cache key travels further than a packet — into client logs, provider
telemetry, and support channels — so it is treated as control metadata under the
existing data classification and must remain safe at that classification.

## Relationship to the other two records

Caching changes the price of what is delivered. It does not change what is
delivered, and it does not turn addition into substitution.

Arithmetic on the measured instance: caching recovers roughly 40% of that arm's
cost gap; the treatment arm still costs more than its baseline. Recall
([IC-TRFC-125](../product/task-recall-first-context-selection-prd.md)) decides
whether the delivered context is worth anything, and hooks
([IC-HEH-126](../product/host-executed-context-hooks-prd.md)) decide whether it
displaces work. This record is third for that reason.

## Preserved invariants

No security invariant changes. The key is control metadata under the existing
classification, the packet gains no authority, and `SEC-INV-007` is untouched:
declaring bytes cacheable requires no network and no execution.
