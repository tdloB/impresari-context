# ADR-0086 Roadmap Maintenance Automation

- Status: Implemented; default-branch live reconciliation evidence pending
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

## Remaining Live Checkpoint

After merge, manually dispatch the default-branch maintenance workflow once.
Record the run identity, receipt artifact digest, exact-owned issue lifecycle,
and cleanup result. That checkpoint may validate the automation but cannot
admit a new client version, repair evidence, merge code, or publish a release.
