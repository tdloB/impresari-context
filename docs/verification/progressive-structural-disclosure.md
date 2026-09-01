# Progressive Structural Disclosure Verification

- Date: 2026-09-01
- Governing decision: [ADR-0121](../decisions/0121-use-bounded-progressive-structural-disclosure.md)
- Classification: provider-free mechanics evidence only
- Provider calls, official grading, publication, and benchmark submission: none

## Frozen inputs

- Gate manifest: `evaluation/v1/progressive-structural-manifest.json`
- Corpus: `evaluation/v1/structural-utility-manifest.json`
- Frozen corpus SHA-256:
  `sha256:406557b5c5268c754c6f233fc484918b5be68e64fef1e2c0298e00cfdacd2e0f`
- Languages: TypeScript, Rust, and Ruby
- Fixtures: six, including development, validation, and held-out labels
- Scripted policy: lookup `all_admitted`, then expand every returned handle

## Verified mechanics

Each fixture uses one immutable source root. The gate runs fresh ordinary,
eager-structural, progressive-structural, and repeated progressive-structural
MCP servers plus a separately named eager-structural warm-cache seed/reuse arm,
and checks:

1. byte-equal advertised tool definitions across all delivery modes;
2. a smaller initial progressive tool result than the eager tool result;
3. exact preservation of every ordinary initial evidence anchor;
4. byte-identical evidence across fresh and warm-cache eager arms;
5. deterministic disclosure maps and map-receipt identities across equal fresh
   progressive runs;
6. byte-identical exact structural evidence after scripted full expansion;
7. receipt accounting against independently serialized MCP tool-result bytes;
8. source immutability before and after every arm; and
9. zero provider calls.

Focused negative tests additionally cover missing sessions, forged and
cross-session handles, closed sessions, changed source, startup mode/tuple
mismatch, and cumulative exhaustion before an additional repository read.
Closed conformance fixtures reject source excerpts in compact maps and authority
claims in receipts.

## Commands

```text
cargo test -p context-evaluation --test progressive_structural_gate
cargo test -p context-mcp
cargo test -p context-conformance --test schema_conformance
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
```

The full test command requires local loopback binding for the unrelated
dashboard-server tests. The gate itself opens no listener and performs no
network operation.

## Claim boundary

Passing this gate proves the local transport, recovery, accounting, ownership,
and deterministic-delivery mechanics. It does not show how a model chooses
handles, how often it expands them, or whether Impresari changes correctness,
tokens, cost, latency, or tool calls. Those questions remain reserved for a
separately frozen controlled pilot with official grading where applicable.
