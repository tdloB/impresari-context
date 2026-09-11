# Tree-sitter 0.27 Upgrade — Architecture Requirements and Design

- ARD ID/version: IC-TSU-ARD-150 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Governing PRD: [IC-TSU-150](../product/tree-sitter-0-27-upgrade-prd.md).
- Decision: [ADR-0150](../decisions/0150-upgrade-tree-sitter-as-a-recorded-provenance-change.md).

## Where the parser version lives

`context_structural::PARSER_VERSION` is the only place the version is written.
Every structural fact's provenance, the worker's request validation, the engine,
the MCP server and the evaluation gates all read it. The test
`parser_version_matches_the_linked_tree_sitter_pin` compares it with the exact
pin in `crates/context-structural/Cargo.toml` and fails when they disagree.

## What changes

| File | Change |
| --- | --- |
| `crates/context-structural/Cargo.toml` | `tree-sitter` pinned to `=0.27.0` |
| `Cargo.lock` | `tree-sitter` 0.27.0, `tree-sitter-language` 0.1.8, `regex-syntax` no longer a `tree-sitter` dependency |
| `crates/context-structural/src/lib.rs` | `PARSER_VERSION = "tree-sitter-0.27.0"` |
| `tests/conformance/v1/valid/structural-graph.json`, `structural-query.json` | `parser_version` moves with it |
| `artifacts/sbom.spdx.json` | regenerated from the lockfile, 192 packages |

No other source file changes.

## Cache invalidation

`worker_toolchain_identity` hashes the request's protocol, language, fact
classes, limits, `parser_version`, `grammar_version`, resolver and graph
versions, and `worker_cache_identity` binds that identity to the worker
executable's digest. Moving `PARSER_VERSION` changes the toolchain identity, and
the rebuilt worker changes the digest, so every entry built by 0.26.13 becomes
unreachable. Entries are rebuilt on first use. Nothing can serve a 0.26.13 entry
as a 0.27.0 result.

## SBOM and frozen evidence

The live SBOM tracks the lockfile and is regenerated with
`scripts/generate-sbom.rb`. The frozen macOS candidate record validates the SBOM
retained beside it under ADR-0143, which this change does not touch.

## Upstream exposure

| 0.27.0 change | Exposure |
| --- | --- |
| Query captures in alternations with quantifiers (#5317) | None: the extractor uses no Tree-sitter queries |
| `Node::child_count` returns `u32` (#5313) | None: the extractor does not call it |
| Strict-aliasing fixes in the array type (#5242) | Parser internals; covered by parity |
| Action-overflow fix (#5273) | Parser internals; covered by parity |

## Parity method

The twenty-two-task astropy corpus is built three times from identical source:
with 0.26.13 binaries from current `main`; with a control that links 0.27.0 but
keeps recording `tree-sitter-0.26.13`; and with this branch. For each task the
comparison checks the nominated files; every disclosure map item's path, symbol
label, relationship class, target path, confidence, freshness and unknowns, and
their order; the map state and omissions; and every evidence record's path,
content hash, span, excerpt bytes and extraction method. Graph and item
identities are compared separately, because they include the recorded version.
The comparison was first run on one build against itself and reported every
check identical.

## Parity result

| against `main` | control | this branch |
| --- | --- | --- |
| nomination, evidence, map state | identical | identical |
| map items (1,440) | identical, same order | 280 replaced |
| graph and item identities | identical | all differ |
| map symbol recall | 15/34 | 13/34 |
| map file recall | 22/27 | 22/27 |

The control shows the parse is unchanged: 0.27.0 yields the same graph, with the
same identities, as 0.26.13. The branch's differences therefore come from the
recorded version, through edge selection:

```text
edge_id    = H(workspace, kind, source, target, module, resolution, span, provenance)
provenance includes parser_version
query_graph: each node's edges in edge_id order, until max_edges
```

Relabelling reorders every node's edges, so a traversal that reaches its edge
limit keeps a different subset. Item counts per task are unchanged because the
limits are unchanged; which items fill them is not. Selecting edges by identity
order is independent of this upgrade and makes the delivered map sensitive to any
provenance change; it should be fixed before this merges.

ADR-0151 has since fixed it. Rebuilt on top of it and run from the same directory
as `main`, this branch's map is identical to `main`'s: all 1,371 items in the same
order, map symbol recall of 15 of 34 and map file recall of 22 of 27.
Only graph and item identities differ.

## Verification

- `parser_version_matches_the_linked_tree_sitter_pin` passes.
- `scripts/check-sbom.rb` passes on the regenerated SBOM.
- The structural evaluation gates run against `PARSER_VERSION` in the full
  repository gate.
