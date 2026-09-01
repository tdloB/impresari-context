# Deterministic Task-Signal Selection — Architecture Requirements and Design

- Status: Implemented and provider-free verified.
- Date: 2026-09-01.
- Governing PRD:
  [Deterministic Task-Signal Selection PRD](../product/deterministic-task-signal-selection-prd.md).
- Governing decision:
  [ADR-0117](../decisions/0117-decompose-task-text-before-repository-retrieval.md).

## Architecture outcome

```text
bounded task text
      │
      ├─ original profile operation
      │
      └─ deterministic signal extractor
           ├─ quoted/backticked spans ── literal
           ├─ portable path candidates ─ filename + literal basename
           ├─ code-like identifiers ──── literal
           └─ filtered lexical terms ─── lexical, one term per step
                         │
                         ▼
             existing exact-source retrieval
                         │
                         ▼
              existing bounded packet builder
```

The signal extractor changes only planning. It grants no read or path
capability and does not alter the retrieval implementations. Every selected
match is still verified against the current capability-relative source.

## Signal model

The extractor performs one bounded byte scan and emits internal candidates with
four fields: exact query text, retrieval kind, priority class, and original byte
offset. Candidate text is never executed or interpreted as syntax by an
external subsystem.

Priority classes are:

1. original profile-defined operation;
2. explicit quoted/backticked literal;
3. path-like filename and code-like literal;
4. filtered lexical fallback.

Within a class, exact candidates are ordered by first occurrence. Lexical
fallback ordering is independent of prose order: longer terms sort first, then
ASCII lexical order. This makes noisy reorderings stable while retaining
deterministic bounds.

## Extraction rules

- The scanner accepts printable ASCII separators and UTF-8 task prose but only
  emits ASCII retrieval signals in version 1.0.
- Balanced single quote, double quote, and backtick delimiters emit their
  non-empty interior without the delimiters.
- Token characters are ASCII alphanumeric plus `_`, `-`, `.`, `/`, `:`, and
  `\\`. Boundary punctuation is trimmed.
- A path-like token contains a path separator or a non-leading dot followed by
  a 1-16 character ASCII alphanumeric extension.
- A code-like token contains `_`, `-`, or `::`, or contains both a non-leading
  dot and an ASCII alphanumeric character.
- Lexical terms are lowercase ASCII alphanumeric sequences of 3-64 bytes,
  excluding the frozen version-1 stop-word set.
- At most 16 path/code source tokens and eight final plan steps are retained;
  lexical sorting is bounded by the 4,096-byte task ceiling.

## Profile composition

Each profile keeps its existing first operation when it is no longer than 256
bytes and satisfies the selected retrieval kind's closed input contract. A
literal operation must also fit the request's actual per-item excerpt limit.
An incompatible original query is omitted with a stable reason rather than
being allowed to abort later high-signal operations. Derived literal signals
are subject to the same runtime limit. The remaining slots are filled from
applicable signal classes. Duplicate `(kind, query)` pairs are removed.
If no derived signal is available, the profile retains a compatible legacy
secondary operation so existing behavior does not become narrower.

The plan reason codes distinguish original, quoted, path, identifier, and
lexical-fallback operations. The plan identity therefore binds the exact
decomposition result without adding a new public schema.

## Security and limits

- No signal can select a file directly; filename retrieval still searches only
  admitted snapshot paths.
- `..`, absolute-looking paths, shell metacharacters, FTS operators, glob
  characters, and regular-expression punctuation receive no special power.
- Invalid or truncated quoted text is ignored as a quoted candidate and may
  contribute only through ordinary bounded token rules.
- All arithmetic uses checked or saturating operations and every emitted slice
  is on a verified UTF-8 boundary.
- The extractor stores no source, task, plan, or signal outside the existing
  request lifecycle.

## Verification

1. Unit vectors prove closed extraction, ordering, deduplication, and limits.
2. Planner vectors prove stable reason codes and a maximum of eight operations.
3. Engine fixtures prove exact and descriptive queries recover the same anchor.
4. Negative tests prove control, traversal, shell, glob, FTS, and flood inputs
   remain inert and bounded.
5. The independent provider-free evaluator fixture confirms task-noise
   selection stability after the product PR is merged.

## Deferred architecture

The next increment may map admitted exact identifier/path evidence into the
existing structural graph and traverse typed edges. Progressive, reversible
views modeled after LeanCTX remain later work and require independent utility
evidence after selection is stable.
