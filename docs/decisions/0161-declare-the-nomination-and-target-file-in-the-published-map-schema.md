# ADR-0161: Declare the Nomination and Target File in the Published Map Schema

- Status: Accepted
- Date: 2026-09-13
- Related PRDs: [Nomination Disclosure](../product/nomination-disclosure-prd.md),
  [Target File Disclosure](../product/target-file-disclosure-prd.md)
- Architecture: [Nomination Disclosure](../architecture/nomination-disclosure-ard.md),
  [Target File Disclosure](../architecture/target-file-disclosure-ard.md)
- Follows: [ADR-0142](0142-disclose-the-files-a-scoped-graph-was-built-over.md),
  [ADR-0146](0146-name-the-file-a-relationship-points-into.md)

## Context

ADR-0142 added a `scope` to every progressive disclosure map, and ADR-0146 added
a `target_display_path` to an item whose relationship points into another file.
Neither change touched `schemas/v1/progressive-disclosure-map.schema.json`, which
refuses unknown fields at both levels. So every map the product has emitted
since ADR-0142 fails its own published schema.

No test validated an emitted map against that schema; the conformance suite
checks only its hand-written fixtures. The drift surfaced in the evaluation
harness, whose paid-path dispatcher reads the map just as strictly: it refused
both fields, and every treatment session failed to open.

## Decision

Declare both fields in the published schema, as optional fields of schema
version `1.0.0`:

- `scope` carries exactly the seven fields ADR-0142 emits. Its schema is the
  nomination's own (`impresari_context_file_nomination`, version `1.1`); each
  file names a path, one of the four reason codes the nomination assigns, and
  its matched identifier count.
- `target_display_path` is a non-empty path, like `display_path`.

Add conformance fixtures for a scoped map naming a target file, a scope that
leaks the nomination's admitted identifiers, and an empty target path.

Add a product test that validates emitted maps against the published schema:
the whole-repository map a progressive build returns, and that map with a
scoped nomination and an item naming its target file.

## Consequences

Consumers that validate against the published schema now accept what the
product emits. A field added to the map without the schema fails the product's
own tests, before any consumer sees it.

The version stays `1.0.0`. Both fields already shipped under it, both are
optional, and a map without them still validates.

A new nomination reason code, or a nomination schema version, must now be added
to this schema in the same change.
