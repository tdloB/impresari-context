# Impresari Context — Roadmap Maintenance Automation PRD

- Status: Accepted for staged implementation
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0086

## Objective

Detect stale compatibility evidence, upstream client releases, dependency
changes, and release-candidate drift without silently promoting a client,
changing user configuration, or publishing a release.

## User Outcome

Maintainers receive one bounded GitHub issue for each actionable maintenance
condition. Public compatibility claims withdraw deterministically when their
evidence expires or changes, while new client versions remain unadmitted until
their own controlled rehearsal passes.

## Scope

- Scheduled source-free evaluation of every released L4 compatibility manifest.
- Allowlisted, metadata-only upstream version observation for admitted clients.
- Exact-owned GitHub issue creation or update using stable deduplication keys.
- Monthly release-candidate rehearsal that creates short-retention artifacts
  but no tag, release, package publication, or signing operation.
- Scheduled dependency, workflow-action, sandbox-profile, OS-support, and
  future guest-image freshness checks.
- Machine-readable receipts for `current`, `stale`, `changed`, `new_version`,
  `unavailable`, and `invalid` states.

## Non-goals

- Automatic compatibility promotion, live authenticated L3 rehearsal, client
  installation, credential use, user notification, release publication,
  automatic remediation, branch merge, or policy/risk acceptance.
- Reading repository source, user workspaces, client homes, environment
  credentials, provider conversations, or telemetry.
- Scraping undocumented endpoints or treating popularity as product demand.

## Acceptance Criteria

- Every scheduled workflow has least-privilege permissions and fixed network
  destinations or source-free inputs.
- A repeated condition updates one exact-owned issue instead of producing issue
  spam; resolution closes only that owned issue.
- Network failure or schema drift returns `unavailable` or `invalid` and never
  preserves a higher claim.
- A new upstream version opens an evaluation issue but cannot modify a
  compatibility manifest.
- Monthly candidates are built from the default-branch SHA, retained for a
  bounded period, and cannot create tags or releases.
- Pull-request tests use frozen fixtures and require no network or write token.
- Logs and issues contain versions, dates, digests, and reason codes only—no
  source, credentials, account identity, or private paths.

## Dependencies

- Existing CI-4 manifests and health evaluator.
- Existing Dependabot configuration and release-candidate workflow.
- GitHub Actions scheduled workflows and issue-scoped repository permission.

## Manual Boundaries

Admitting a client version, merging a dependency update, publishing a release,
rotating a signing root, or changing an allowlisted upstream source remains a
separate reviewed decision.
