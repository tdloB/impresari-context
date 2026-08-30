# Linux External Lifecycle Composition ARD

- Status: Accepted
- Date: 2026-08-30
- Decision: ADR-0081

## Architecture

```text
exact C package receipt ----------------------------+
fresh external live receipt ----+                   |
                                 +-- identity link --+-- composer
exact original-synthetic receipt +                   |      |
post-collection missing-FD health receipt -----------+      v
                                              lifecycle candidate
                                              production = false
```

The package collector runs before the external operator service. The live
rehearsal persists its final receipt only after the operator verifies that the
single transient service is collected. Its composite receipt is retained
separately so interruption and crash checks are verified directly rather than
inferred from a summary status.

## Health Withdrawal

The health collector runs after collection in a new foreground process with
fixed descriptor 3 explicitly closed. It reads only the two bounded JSON
receipts. An absent capability plus package/external clean state returns
`withdrawn`; an available descriptor or incomplete clean state returns
`withdrawal_failed`. It records no descriptor target, cgroup path, or unit name
and performs no host discovery, service operation, privilege request, repair,
or fallback.

## Composition

The composer reads four bounded JSON files and one exact expected source SHA.
It hashes the files as delivered, verifies every identity link, requires the
package/source/host/interruption/crash/cleanup/withdrawal matrix, and emits one
of six states. It does not spawn a child, inspect the repository, mutate the
host, or perform network access.

`lifecycle_candidate` describes only the complete C lifecycle evidence set. The
withdrawal probe demonstrates that an unavailable operator capability removes
the active runtime claim; it does not retain that capability. Production
support remains a later expiring admission and release-maintenance decision.
