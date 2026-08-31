# Roadmap Maintenance Automation ARD

- Status: Implemented; default-branch live reconciliation evidence pending
- Date: 2026-08-30
- Governing PRD: [Roadmap Maintenance Automation PRD](../product/roadmap-maintenance-automation-prd.md)
- Decision: [ADR-0086](../decisions/0086-scheduled-roadmap-maintenance-automation.md)

## Architecture

```text
schedule / manual dispatch
          |
          v
fixed manifest set + allowlisted metadata adapters
          |
          v
closed observation -> deterministic evaluator -> bounded receipt
                                                |
                                                v
                                  exact-owned GitHub issue adapter
```

The evaluator is pure and source-free. Network adapters and the GitHub issue
writer are separate workflow steps. Pull-request tests exercise only frozen
metadata fixtures and mocked issue state.

## Components

1. `maintenance-observation` closed schema records the named client/component,
   source identity, observed version or expiry, checked time, and outcome.
2. Client adapters accept one documented metadata endpoint and a byte/time
   ceiling. Redirects, HTML fallback, unknown fields, and version ambiguity
   fail closed.
3. The evaluator compares observations with released manifests and emits no
   mutation instruction.
4. The issue adapter derives an ownership key from component and condition,
   searches only the dedicated label, and creates, updates, or closes one issue.
5. Candidate rehearsal remains a separate read-only workflow with artifact
   upload permission only.

## Authority Model

- Source-free health jobs: `contents: read`.
- Issue reconciliation job: `contents: read`, `issues: write` only.
- Candidate rehearsal: `contents: read`, `actions: write` only as required for
  bounded artifact retention; no tag or release permission.
- No job receives provider credentials, release credentials, signing keys,
  workspace content, or a broad repository token.

## Failure And Idempotence

- Each observation is bounded and all-or-nothing.
- Unknown versions and stale evidence withdraw claims through existing CI-4
  semantics; an issue failure cannot restore the claim.
- Issue bodies are generated from a closed template and escaped data fields.
- Concurrent runs use one component/condition ownership key and optimistic
  reconciliation rather than duplicate creation.

## Verification

- Frozen fixtures cover current, new, stale, changed, malformed, unavailable,
  redirect, oversized, duplicate-issue, and permission-denied states.
- Static checks reject workflow permission expansion and non-allowlisted URLs.
- A disposable repository rehearsal proves exact issue lifecycle without
  touching product source or release state.
