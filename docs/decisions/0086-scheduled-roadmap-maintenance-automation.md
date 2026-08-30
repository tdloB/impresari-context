# ADR-0086: Add Fail-Closed Scheduled Roadmap Maintenance Automation

- Status: Accepted for staged implementation
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
