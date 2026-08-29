# Impresari Context — Local real-time dashboard and budget control PRD

- Status: Proposed; founder approval required before implementation
- Date: 2026-08-29
- Authority: Future observability and control increment
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Architecture: [Local dashboard and budget control ARD](../architecture/local-dashboard-budget-control-ard.md)
- Decision: [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md)

## Objective

Give a local operator a continuously updating view of Impresari Context usage,
outcomes, latency, budget requests, and limit pressure, plus an explicit way to
set local budget ceilings that can only narrow the engine's accepted hard
limits. Preserve the product's source-verifiable, local-first boundary without
turning the dashboard into telemetry, source browsing, agent orchestration, or
a hosted control plane.

## User outcome

An operator starts a foreground local dashboard, receives a browser URL with an
ephemeral access capability, and sees new metadata events appear without page
reload. They can inspect recent and aggregate request behavior, preview a
budget-policy change, apply an exact owned policy, verify its fingerprint and
effective ceilings, restore the prior policy, and close the server. Closing the
server ends all dashboard access and background work.

## Proposed scope

- A source-free `dashboard serve` command bound only to loopback on an
  operating-system-selected port, foreground by default, with no remote mode.
- Bundled immutable UI assets; no CDN, remote script, analytics, fonts,
  cookies, service worker, account, or browser storage beyond ephemeral state.
- A bounded live stream derived from validated local audit events and current
  engine health, with periodic aggregate snapshots for recovery after gaps.
- Views for request outcome, capability, duration, requested hard limits,
  limit utilization where currently measured, engine version, policy decision,
  and opaque workspace/snapshot identity presence.
- Filters by time window, outcome, capability, and pseudonymous operator-owned
  workspace label; no source path, filename, query, excerpt, packet content,
  prompt, client response, credential, or environment display.
- Versioned local budget policies for global and per-purpose/per-capability
  ceilings. Policies may deny or reduce caller budgets but can never raise
  compiled, release, workspace, capability, or caller limits.
- Preview-by-default create/update/remove operations, exact policy identity,
  optimistic concurrency, atomic write, rollback receipt, and immediate
  effective-policy visibility.

## Non-goals

- A hosted or remotely reachable dashboard, organization control plane,
  multi-user access, cross-device synchronization, cloud telemetry, fleet
  policy, billing, quota enforcement, model-token accounting, provider cost
  estimation, or agent/model/tool control.
- Reading source, recovering evidence, opening packets, displaying queries,
  editing profiles, changing authorization, raising resource maxima, bypassing
  request budgets, or deciding whether a task, change, or release proceeds.
- A daemon, login item, background monitor, automatic browser launch, implicit
  audit-retention increase, or exposure through `0.0.0.0`, LAN, tunnel, proxy,
  iframe, or editor webview.
- Calling this a LeanCTX-compatible dashboard or copying LeanCTX code, UI,
  prompts, assets, private/internal contracts, or unsupported product claims.

## Dashboard contract

- Startup requires explicit canonical audit-store and policy-store roots and
  rejects workspace/cache overlap, symlinks, non-owned paths, and broad home or
  filesystem roots.
- The command prints one loopback URL containing a high-entropy, process-local
  capability. The token is never persisted, accepted in query strings after
  initial bootstrap, reflected, logged, or exposed to child processes.
- Every response uses a restrictive content security policy, no-store caching,
  no framing, no MIME sniffing, same-origin requests, and explicit origin and
  host validation. Non-loopback bind resolution fails closed.
- The event stream is bounded by count, bytes, age, rate, and client backpressure.
  Slow or disconnected clients receive an explicit gap and aggregate refresh;
  they never cause unbounded buffering or audit retention.
- Dashboard reads use validated audit-schema fields only. Malformed, unknown,
  future, or partially migrated rows are counted as unavailable and never
  rendered from raw bytes.

## Budget-control contract

- A policy declares schema version, policy ID, revision, creation time, optional
  expiry, scope selectors, deny rules, and ceilings for existing
  `ResourceBudget` fields. It contains no source patterns or executable logic.
- Effective limits are the field-wise minimum of immutable engine maxima,
  authorized workspace/capability policy, local dashboard policy, and the
  caller's explicit request. Missing values never mean unlimited.
- Equal canonical policy content has equal identity. Unknown fields, duplicate
  selectors, ambiguous precedence, invalid time, oversized input, or a ceiling
  above its governing maximum fails validation before write.
- Specific deny rules precede ceilings; otherwise exact selector specificity
  and canonical lexical order determine a single stable match. There is no
  first-match file-order behavior.
- Applying a policy requires the current policy identity and revision. A stale
  browser cannot overwrite a newer CLI or dashboard change.
- Removal restores the absence of the dashboard-owned layer; it does not alter
  any engine, workspace, client, or caller budget.

## Acceptance criteria

- Hosted tests prove loopback-only binding, ephemeral capability bootstrap,
  origin/host/CSRF enforcement, secure response headers, no external requests,
  bounded streaming, reconnect/gap behavior, and complete shutdown cleanup.
- Browser tests render frozen valid audit events and prove adversarial strings,
  malformed rows, unknown schemas, HTML/script payloads, and oversized fields
  cannot execute or appear as trusted values.
- Source-secrecy tests seed distinctive paths, names, queries, content, prompts,
  credentials, and environment values and prove they never enter HTTP, UI,
  logs, policy, receipts, screenshots, or exported aggregates.
- Property tests prove every effective numeric limit is less than or equal to
  every governing layer and that equal inputs produce identical effective
  budgets and policy identities.
- Concurrency tests prove stale revisions, simultaneous writes, crash points,
  symlink swaps, permission loss, and malformed policies cannot partially apply
  or expand authority.
- Runtime integration tests prove denied/limited requests record the effective
  policy decision and limits through the existing audit contract without
  duplicating source or packet data.
- A live local rehearsal uses only synthetic audit metadata and a disposable
  policy store, then proves exact shutdown and removal.

## Manual boundary

Implementation requires explicit founder acceptance of ADR-0072. Any future
remote, hosted, organization, billing, telemetry, or source-viewing capability
requires a separate decision, threat-model update, and external-data approval.
