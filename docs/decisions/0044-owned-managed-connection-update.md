# ADR-0044: Owned managed-connection update

- Status: Accepted for implementation
- Date: 2026-08-25
- Scope: CI-1 explicit updates of owned local-MCP connection entries

## Decision

Add update only as a compare-and-replace operation: the caller supplies both
the prior fixed local-stdio contract and the replacement contract, the CLI
proves the current target exactly matches the former, previews the latter, and
writes atomically only with explicit `--apply`.

## Rationale

“Impresari Context” is not a sufficient ownership marker. An existing entry
could be manually configured or owned by another workflow. Compare-and-replace
retains the existing exact-ownership model while allowing transparent contract
evolution.

## Constraints

- No prior contract match means no update and no write.
- No default target, automatic repair/migration, or scheduled update exists.
- The format adapters preserve unrelated content and refuse malformed,
  duplicate, ambiguous, symlinked, oversized, or unowned input.
- Receipts reveal only source-free contract metadata and exact operation state.

## Consequences

L1 gains a real safe-update lifecycle without weakening removal or installation
boundaries. User trust/approval and client-specific first-class evidence remain
outside this operation.

## References

- [CI-1a owned-update PRD](../product/client-integration-l1-owned-update-prd.md)
- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [ADR-0035: L1 managed client connection kits](0035-l1-managed-client-connection-kits.md)
