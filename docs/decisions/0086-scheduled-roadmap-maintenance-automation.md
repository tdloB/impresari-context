# ADR-0086: Add Fail-Closed Scheduled Roadmap Maintenance Automation

- Status: Implemented and live evidenced
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

Impresari's client and isolation claims are deliberately bound to exact
versions, hosts, and freshness windows. Manual checking alone can leave stale
claims visible, while unrestricted bots could silently promote untested client
versions or gain unnecessary repository authority.

## Decision

Add scheduled source-free maintenance evaluation and a separately permissioned
exact-owned GitHub issue adapter. Automation may detect, withdraw, rehearse,
and request evaluation; it may not admit, merge, publish, repair, sign, or
accept risk.

## Consequences

- Stale or changed claims become visible promptly and fail closed.
- Maintainers receive deduplicated work items rather than raw scheduled logs.
- The project must maintain documented metadata adapters and bounded fixtures.
- Live authenticated client delivery remains a controlled rehearsal.

## Implementation

The implementation uses `maintenance/client-sources.json` as its closed adapter
allowlist. Four clients use the official GitHub latest-release API; Cursor
remains explicitly unavailable because no documented authoritative metadata
endpoint has been admitted. `scripts/roadmap-maintenance-observe.rb` applies a
10-second timeout, a 524,288-byte ceiling, rejects redirects and non-JSON
responses, and emits only bounded metadata observations.

`scripts/roadmap-maintenance-evaluate.rb` binds every observation to the exact
released compatibility manifest, owned artifact digest, evidence digest, and
freshness date. It emits one of `current`, `stale`, `changed`, `new_version`,
`unavailable`, or `invalid`; only `current` and the already-admitted exact
version during `new_version` retain an existing claim. No result can promote a
version or mutate a manifest.

`.github/workflows/roadmap-maintenance.yml` separates its read-only observation
job from its `issues: write` reconciliation job. The writer owns only issues
carrying the `impresari-maintenance` label and exact hidden ownership marker.
The monthly schedule in `.github/workflows/release-candidate.yml` builds the
default-branch SHA and retains artifacts for seven days without tag, release,
package-publication, or signing authority.

## Alternatives

- Fully manual maintenance: rejected because expiry and upstream drift are
  predictable and safely detectable.
- Automatic client promotion: rejected because version existence is not
  compatibility evidence.
- A privileged maintenance service: rejected; GitHub Actions with narrow,
  per-job permissions is sufficient for repository maintenance.

## Revisit Triggers

Review before adding customer notification, provider authentication, automatic
merge, automatic release, non-GitHub mutation, or background user telemetry.
