# Phase 5 Five-Language Expansion — Architecture Requirements and Design

- Status: Implemented; all five independent admissions accepted
- Date: 2026-08-29
- Product requirement: [Five-Language Expansion PRD](../product/phase-5-five-language-expansion-prd.md)
- Decision: [ADR-0064](../decisions/0064-founder-approved-five-language-expansion.md)

## Architectural objective

Extend the existing source-free parser-worker inventory with five statically
pinned grammars while preserving the same workspace, budget, identity,
isolation, and authority boundaries used by the admitted structural languages.

## Required architecture

```text
authorized snapshot + eligible extension + exact bytes
                         |
                         v
fixed engine language map and pinned worker identity
                         |
                         v
isolated tree-sitter worker with bounded request/response
                         |
                         v
validated declarations / direct relations / containment
                         |
                         v
snapshot-bound graph, planner evidence, and explicit unknowns
```

- `context-engine` owns extension admission and the expected grammar identity.
- `context-structural` owns the fixed grammar enum, parser selection, bounded
  extraction, and syntax-recovery facts.
- No repository file selects, configures, or supplies a grammar.
- No parser action invokes a compiler, interpreter, package manager, build
  system, language server, framework, database, shell, or network.
- C and C++ are separate structural languages even where their syntax overlaps;
  a file extension maps to one admitted grammar and never receives fallback
  parsing through the other.

## Fact contract

The common fact vocabulary remains `declaration`, `import`, `call`,
`reference`, and `containment`. Language-specific syntax is normalized only
when the source node deterministically provides the named fact. Ambiguous or
computed forms are omitted or surfaced through explicit partial/unknown state;
they are never guessed from naming conventions.

## Resource and security constraints

- Existing file, snapshot, request, worker-frame, fact-count, depth, byte, and
  deadline limits apply unchanged.
- Grammar identity participates in cache and worker identity.
- Worker output is rejected on identity, hash, framing, fact-count, range, or
  snapshot mismatch.
- Exact source bytes remain authoritative; parser output never becomes source
  or policy authority.

## Verification strategy

Each language slice adds positive, syntax-recovery, malformed, hostile-name,
limit, identity, deterministic-output, and public-manifest tests. C/C++ fixtures
must distinguish their separate grammar mappings. Dynamic-language fixtures
must include computed constructs that are deliberately not promoted into
static facts. Full repository and hosted gates are mandatory per slice.
