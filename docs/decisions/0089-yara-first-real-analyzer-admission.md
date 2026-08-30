# ADR-0089: Admit YARA As The First Real Analyzer

- Status: Accepted for planning; execution remains gated
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

The hostile-repository foundation already plans required analysis and accepts
closed untrusted analyzer results, but no real analyzer is authorized. A first
adapter must validate the complete isolation, supply-chain, coverage, update,
and normalization path without turning Impresari into an antivirus engine.

## Decision

Use YARA as the first real analyzer, initially on Linux after that exact
platform has production-admitted IAR-1B support. Use only a pinned executable
and project-owned pinned ruleset; disable repository rules, includes, network,
updates, external modules, and arbitrary arguments. Admit macOS and Windows
independently later.

## Consequences

- The first real-analyzer scope remains narrow and reproducible.
- Rule quality and false-positive/false-negative limitations require ongoing
  review and explicit coverage reporting.
- A passing YARA result cannot produce a safe or malware-free claim.
- ClamAV and other analyzers remain separate later decisions.

## Alternatives

- ClamAV first: deferred because its engine/database/update footprint is larger.
- Multiple analyzers at once: rejected because failures would be harder to
  attribute across a new execution boundary.
- Repository-provided YARA rules: rejected because hostile content cannot
  define analyzer behavior.

## Activation Gate

This ADR does not authorize execution. Activation requires an exact platform
with current production IAR-1B admission, implemented contracts and fixtures,
signed analyzer/rules artifacts, complete hosted evidence, and the applicable
independent security review.
