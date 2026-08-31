# ADR-0086 Roadmap Maintenance Automation

- Status: Implemented and live evidenced
- Date: 2026-08-30
- Decision: [ADR-0086](../decisions/0086-scheduled-roadmap-maintenance-automation.md)

## Admitted Implementation

The scheduled maintenance boundary consists of:

- a closed, versioned source set at `maintenance/client-sources.json`;
- a metadata-only observer with fixed HTTPS hosts and paths, a 10-second
  timeout, a streaming 524,288-byte response ceiling, and content-type,
  redirect, tag-prefix, and version validation;
- a pure evaluator that rechecks each manifest, owned artifact, evidence record,
  and freshness date before emitting a bounded receipt;
- a pure issue planner that creates, updates, closes, or leaves unchanged only
  an exact-owned issue identified by label and hidden ownership marker;
- a separate issue application step with `issues: write`, read-only contents,
  and no contents-write, pull-request, release, package, signing, or provider
  authority; and
- a monthly default-branch candidate rehearsal retaining artifacts for seven
  days without creating a tag or release.

Cursor deliberately returns `unavailable` until a documented authoritative
metadata endpoint is reviewed and added to the closed source set. This is a
visible withdrawal, not a fallback to scraping.

## Deterministic Evidence

`scripts/check-roadmap-maintenance-automation.rb` uses frozen responses and
proves:

1. `current`, `stale`, `changed`, `new_version`, `unavailable`, and `invalid`;
2. redirect rejection, malformed metadata rejection, source unavailability,
   response identity binding, and exact tag-prefix parsing;
3. claim withdrawal for stale, changed, unavailable, and invalid evidence;
4. preservation of only the already-admitted exact-version claim when a new
   upstream version is observed;
5. exact-owned issue create, update/no-op, duplicate closure, condition change,
   resolution, and non-owned issue preservation; and
6. static rejection of broad workflow permissions and non-allowlisted hosts.

The JSON Schema corpus separately rejects observation and receipt authority
expansion. Pull-request tests use no network and no write token.

## Default-Branch Live Evidence

The first dispatch, run
[`33344770117`](https://github.com/tdloB/impresari-context/actions/runs/33344770117),
failed closed because an unavailable GitHub expression produced no timestamp.
The observer stopped before artifact upload and the dependent issue writer was
skipped, so no partial issue state was created. PR 164 added runner-derived
canonical UTC time plus a regression guard.

Corrected run
[`33345269371`](https://github.com/tdloB/impresari-context/actions/runs/33345269371)
passed both jobs from default-branch commit
`d196f4cfc0332fd3bcfa6e93ec3bb95f5d8706ff`:

- workflow artifact `9741774301` had GitHub digest
  `sha256:3e3d10a247841df6eaea57ff721637bd4c5953ade3ff02ae93053fafb2851adf`;
- `observations.json` SHA-256 was
  `d07238c5693ac628329e400659cfc6ae5bb702df487b1bf468433d263360e099`;
- `receipts.json` SHA-256 was
  `a6973f62365868d6a8db6b212f8a826c0b54cdea1b26871533f92b709b261e53`;
- Codex `0.151.0`, Claude Code `2.1.251`, Copilot CLI `1.0.82`, and
  VS Code `1.135.0` were `new_version`; none was admitted and only each
  previously admitted exact-version claim was retained;
- Cursor was `unavailable`, withdrew its recorded claim, and used no scraping
  fallback; and
- exact-owned issues
  [#165](https://github.com/tdloB/impresari-context/issues/165),
  [#166](https://github.com/tdloB/impresari-context/issues/166),
  [#167](https://github.com/tdloB/impresari-context/issues/167),
  [#168](https://github.com/tdloB/impresari-context/issues/168), and
  [#169](https://github.com/tdloB/impresari-context/issues/169) were created.

Repeat run
[`33345318603`](https://github.com/tdloB/impresari-context/actions/runs/33345318603)
also passed. It updated those same five issue numbers and created no duplicate.
Its artifact `9741790216` had GitHub digest
`sha256:7a086db7b8b8314d780cd4d6a514546c0505054e207a7ad12bbb8a3ca85a20df`.
Both artifacts expire on 2026-09-07. Ephemeral runner state was discarded; no
tag, release, package, branch, manifest, admitted-version, or risk decision was
created by either run.
