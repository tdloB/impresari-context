# ADR-0150: Upgrade Tree-sitter to 0.27.0 as a Recorded Provenance Change

- Status: Accepted
- Date: 2026-09-10
- Related PRD: [Tree-sitter 0.27 Upgrade](../product/tree-sitter-0-27-upgrade-prd.md)
- Architecture: [Tree-sitter 0.27 Upgrade](../architecture/tree-sitter-0-27-upgrade-ard.md)
- Extends: [ADR-0004](0004-source-language-and-parser-strategy.md)
- Follows: [ADR-0143](0143-retain-a-candidate-sbom-beside-its-record.md)
- Depends on: [ADR-0151](0151-select-structural-context-by-what-it-is.md)

## Context

ADR-0004 makes structural facts the output of isolated Tree-sitter workers. Every
fact records the parser that produced it as `parser_version`, and the worker's
toolchain identity, which is part of every structural cache key, includes it.

Dependabot's #289 moved the exact `tree-sitter` pin from 0.26.13 to 0.27.0 and
changed nothing else. The recorded version is a separate constant,
`PARSER_VERSION`, and the test tying it to the pin failed on #289. That is the
guard working: merged alone, the bump would have made every structural fact
claim 0.26.13 while being produced by 0.27.0.

The lockfile change also moves `tree-sitter-language` from 0.1.7 to 0.1.8 and
drops `regex-syntax` from `tree-sitter`'s dependencies, so the SBOM must be
regenerated. ADR-0143 is what allows that without moving frozen release
evidence.

Upstream, 0.27.0 changes library behaviour in four places relevant here: a
query fix for captures in alternations with quantifiers (#5317),
`Node::child_count` returning `u32` (#5313), strict-aliasing fixes in the array
type (#5242), and an action-overflow fix (#5273). The extractor walks syntax
trees without Tree-sitter queries and never calls `child_count`, so the first two
cannot reach it. Parser-internal fixes can still change a tree, and that is what
a parity check has to show.

## Decision

Upgrade the exact pin to 0.27.0, with `tree-sitter-language` 0.1.8 through the
lockfile. Set `PARSER_VERSION` to `tree-sitter-0.27.0`, move the conformance
fixtures that carry it, and regenerate the SBOM from the lockfile.

Accept that every structural cache entry is invalidated once. The toolchain
identity includes `parser_version`, so entries built by 0.26.13 become
unreachable and are rebuilt on next use; none is served stale.

Show structural parity on the twenty-two-task astropy corpus before merging.

Going forward, a change to the Tree-sitter pin lands only as a recorded upgrade
that moves `PARSER_VERSION`, its fixtures and the SBOM together and reports
parity. An automated bump that moves the pin alone is closed in its favour;
#289 is closed by this decision.

## Consequences

Parity was measured on the twenty-two-task astropy corpus with three builds that
differ only in the parser and the version they record:

| build | parser | recorded version | map items against `main` | map symbol recall |
| --- | --- | --- | --- | --- |
| `main` | 0.26.13 | 0.26.13 | — | 15/34 |
| control | 0.27.0 | 0.26.13 | identical: all 1,440, in the same order | 15/34 |
| this decision | 0.27.0 | 0.27.0 | 280 of 1,440 replaced | 13/34 |

In all three, nomination, evidence, map file recall (22 of 27) and the number of
map items for each task are identical.

The control establishes that Tree-sitter 0.27.0 produces the same structural
output as 0.26.13 on this corpus: the same graph identities and the same map
items in the same order. Every difference in this decision's build comes from
the recorded version alone. An edge's identity hashes its fact provenance, which
includes `parser_version`, and the traversal takes each node's edges in identity
order until it reaches its edge limit. Changing the recorded version reorders the
edges and changes which ones survive the limit, so the delivered map changes
although no fact did. On this corpus that moves map symbol recall from 15 to 13
of 34: `astropy-13579` loses `pixel_to_world_values` and
`world_to_pixel_values`, `astropy-14369` loses `_make_parser`, and
`astropy-8872` gains `__new__`.

That selection behaviour predated this decision and would have moved the
delivered map on any change to fact provenance. ADR-0151 has since removed
identity from selection. Measured again on top of it, from the same directory as
`main`, this decision's build delivers maps identical to `main`'s on all
twenty-two tasks: the same 1,371 items in the same order, map symbol recall of
15 of 34 and map file recall of 22 of 27. Only identities differ, as they
must.

`grammar_version` is still the constant `mixed-pinned-grammars`, with no guard
tying it to the grammar crates. A grammar-crate bump would change facts without
changing the version they record, the same defect this decision closes for the
parser. That is a separate follow-up.

## Alternatives considered

**Merge #289 as proposed.** Rejected: every structural fact would record a
parser version that did not produce it, and the version guard fails.

**Move the pin and the constant without a parity check.** Rejected: nothing
would show that extraction still behaves as before.

**Derive `PARSER_VERSION` from the lockfile at build time.** Deferred: the guard
test already fails whenever the two disagree, and deriving it would add a build
script that reads repository files to the worker's crate.

**Stay on 0.26.13.** Rejected: holding the pin indefinitely defers upstream
fixes into a larger jump later, and this upgrade is narrow enough to verify now.
