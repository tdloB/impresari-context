# CI-1a Owned Managed-Connection Update — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-1a PRD](../product/client-integration-l1-owned-update-prd.md)
- Governing decision: [ADR-0044](../decisions/0044-owned-managed-connection-update.md)

## Architectural objective

Allow an explicit local-MCP contract replacement without turning a connection
kit into a general configuration editor. Update is a transaction over one
caller-named target and two fully declared fixed contracts: prior and desired.

## State machine

```text
absent/unowned/malformed/drifted ----> reject, no write
             |
             | exact declared prior contract
             v
          owned prior
             |
             | preview
             v
        preview_ready (no write)
             |
             | explicit --apply + revalidated target
             v
         owned desired
```

The update request never discovers an existing entry from a client or treats
the `impresari-context` name as ownership. It derives both entry serializations
from canonical caller-supplied binary/workspace/cache contracts and compares
the current target against the prior serialization exactly.

## Required components

- Client/scope serializer selected from the existing versioned connection-kit
  manifest.
- Canonical regular-binary and canonical workspace/cache validators for both
  contracts.
- Target validator that rejects missing parent, symlinked parent/file,
  non-regular, oversized, non-UTF-8, malformed, duplicate, or ambiguous state.
- Token-local TOML/JSON remove-and-insert transformer that preserves unrelated
  content.
- Atomic replacement writer plus source-free receipt carrying redacted prior
  and desired owned-entry previews, target, planned effect, write state, and
  outcome.

## Invariants

1. Preview performs every validation but never creates a directory, file, or
   client state change.
2. Apply is the only write path and is accepted only after exact prior ownership
   verification; any read/validation failure leaves the target unchanged.
3. Update cannot change client, scope, transport class, environment forwarding,
   remote endpoint, consumer role, approval mode, trust, sign-in, or source
   workspace authority.
4. The desired entry remains within the released fixed local-stdio contract.
5. Removal after a successful update is still exact-owned-entry removal, not
   broad configuration cleanup.

## Verification requirements

- Every client serializer has install → preview update → applied update →
  validate desired → reject stale prior coverage.
- Fixtures prove unrelated TOML/JSON content preservation, source immutability,
  no write on preview/rejection, and atomic result visibility.
- Negative tests cover malformed, duplicate, conflicting, symlinked,
  non-explicit, absent, stale-prior, and invalid-binary/cache/workspace input.

## Rollout

The command is a local deterministic capability, not proof of a first-class
client. Per-client L1 promotion continues to require client/version/OS and
real-client admission evidence under the compatibility contract.
