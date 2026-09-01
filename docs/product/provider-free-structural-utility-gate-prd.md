# Provider-Free Structural Utility Gate PRD

- PRD ID/version: IC-PFSUG-119 / 1.0.
- Status: Implemented; local provider-free gate passed, hosted CI required for merge.
- Date: 2026-09-01.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Provider-Free Structural Utility Gate ARD](../architecture/provider-free-structural-utility-gate-ard.md).
- Governing decision:
  [ADR-0119](../decisions/0119-require-provider-free-utility-before-external-structural-delivery.md).

## Problem

ADR-0118 adds deterministic product-owned structural seed selection, but a
working selector does not prove useful context. External MCP/evaluator graph
lifecycle work would add protocol, worker-custody, cold-start, and accounting
complexity. That work must not proceed until a frozen provider-free comparison
shows that seeded traversal adds relevant relationships without displacing the
exact anchors that selected them or hiding unbounded repository work.

## Outcome

A frozen local fixture matrix compares ordinary profiled packets with seeded
structural packets built from the same source, task, snapshot policy, and
resource budget. The gate reports only model-neutral mechanics: anchor
retention and order, new verified structural evidence, overlap, packet bytes,
repository reads, repeated reads, determinism, and source immutability.

## Requirements

1. Use at least six frozen fixtures spanning at least three supported
   structural languages, with at least one quarter held out from development.
2. Run fresh baseline and seeded engines against byte-identical disposable
   sources; never reuse a cache, snapshot, engine, or read ledger across arms.
3. Use the same task, task profile, policy, budget, and exact source bytes in
   both arms. The seeded arm may differ only by its validated graph input.
4. Require every baseline evidence identity to remain in the seeded packet in
   the same relative order and ahead of every newly added structural item.
5. Require at least one new exact-source-verified structural relationship per
   fixture. Count overlap by evidence identity and reject duplicate delivery.
6. Measure exact serialized packet bytes and permit no more than 8,192 added
   bytes per fixture. This is a safety ceiling, not a reduction claim.
7. Measure repository file reads, repeated reads, and source bytes through the
   product-owned complete telemetry projection. Added seeded reads must be no
   greater than the structural edges actually recovered.
8. Rebuild each seeded case and require identical plan, packet identity,
   graph identity, selection reason, ordering, and accounting.
9. Hash every source before and after both arms and require zero mutation.
10. Perform no provider/model call, official grading, network access, Git
    execution, source write by the product, publication, or benchmark claim.

## Acceptance

- The frozen matrix satisfies every requirement on macOS, Linux, and Windows
  hosted quality jobs.
- Unique symbol/path seeds add bounded relationship evidence after all exact
  anchors; ambiguous/no-seed behavior remains covered by ADR-0118 tests.
- The gate emits no token, cost, latency, correctness, SWE-bench, or product
  superiority conclusion.
- Formatting, warnings-denied Clippy, all-target tests, docs, repository
  policy, and hosted CI pass.

## Non-Goals

- External MCP or independent-evaluator graph lifecycle.
- Paid OpenAI or Anthropic evaluation.
- Claiming that structural evidence improves agent correctness or efficiency.
- LeanCTX-style progressive delivery, durable memory, or context replacement.
- Selecting thresholds after observing provider outcomes.

## Local Result

The frozen six-fixture, three-language gate added six novel verified structural
evidence records while preserving every baseline anchor as the ordered packet
prefix. It measured six added reads, six added repeated reads, 485 added source
bytes, and a maximum 3,130-byte profiled-packet increase. These are fixture
mechanics, not token, latency, correctness, cost, or product-effect claims.

Graph construction is intentionally outside this first comparison because the
engine API accepts a prevalidated graph. External graph lifecycle work must
measure its own cold build/read cost before evaluation admission.

## Stop Condition

External structural delivery remains blocked if this gate fails. A passing
gate authorizes architecture work on graph lifecycle only; it does not
authorize provider tests, official grading, publication, or benchmark claims.
