# Tree-sitter 0.27 Upgrade PRD

## Document Control

- PRD ID/version: IC-TSU-150 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Tree-sitter 0.27 Upgrade ARD](../architecture/tree-sitter-0-27-upgrade-ard.md).
- Governing decision:
  [ADR-0150](../decisions/0150-upgrade-tree-sitter-as-a-recorded-provenance-change.md).
- Extends: [ADR-0004](../decisions/0004-source-language-and-parser-strategy.md).

## Problem

An automated dependency update proposed moving the Tree-sitter pin from 0.26.13
to 0.27.0. Merged as proposed, it would have shipped a false provenance claim:
every structural fact would record `tree-sitter-0.26.13` while being produced by
0.27.0. The parser-version guard caught it, and the update cannot land without
the changes that make the recorded version true.

## Product Outcome

The product parses with Tree-sitter 0.27.0, every structural fact says so, and
there is evidence that extraction behaves as it did before.

## Functional Requirements

1. The `tree-sitter` dependency is pinned exactly to 0.27.0.
2. `PARSER_VERSION` records `tree-sitter-0.27.0`, and the guard test tying it to
   the pin passes.
3. Conformance fixtures that carry the parser version move with it.
4. The SBOM matches the new lockfile.
5. The one-time invalidation of structural cache entries is accepted and
   disclosed; no entry built by the previous parser is served.
6. Structural output on the evaluation corpus is compared before and after.

## Acceptance Criteria

- The parser-version guard and the SBOM check pass.
- A build linking 0.27.0 while recording the previous version produces
  structural output identical to `main` on the twenty-two-task astropy corpus,
  and every difference in the upgraded build is attributed.
- The full repository gate passes.
- #289 is closed in favour of this change.

## Non-Goals

- Changing grammar crate versions.
- Guarding `grammar_version`, which remains a separate follow-up.
- Changing any structural behaviour.
