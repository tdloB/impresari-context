# Provider-Free Product Read Observer — Architecture Requirements and Design

- Status: Implemented provider-free.
- Date: 2026-09-01.
- Governing PRD:
  [Provider-Free Product Read Observer PRD](../product/provider-free-product-read-observer-prd.md).
- Governing decision:
  [ADR-0116](../decisions/0116-observe-repository-reads-at-the-workspace-boundary.md).

## Architecture outcome

```text
fresh MCP process
  AuthorizedWorkspace::open
  LocalEngine::build_snapshot
    AuthorizedWorkspace::read_exact ─┐
  context_build                      ├─ process-local read ledger
    planner/retrieval read_exact ────┘
             │
             ├─ exact snapshot source fingerprint
             └─ context_build.read_telemetry
```

`read_exact` is the authoritative I/O boundary because discovery, retrieval,
structural evidence, and exact packet excerpts already converge there under a
directory capability. Higher layers may classify work, but cannot change the
measured counters.

## Components

### Workspace read ledger

`AuthorizedWorkspace` owns a process-local synchronized ledger containing:

- total repository file reads;
- repeated repository file reads;
- exact bytes materialized;
- the set of lossless relative path identities previously read; and
- a completeness bit.

The ledger records only after a regular file has been opened and a bounded
read has been attempted. It records the bytes materialized before returning a
later overflow, mutation, or I/O error. Objects rejected before file content is
read contribute neither a read nor bytes. Counter overflow and synchronization
failure saturate the affected counter and clear completeness.

### Snapshot source identity

During deterministic discovery, each successfully admitted portable path and
its exact bytes contribute to a second SHA-256 stream:

```text
UTF8(relative_path) || NUL || exact_file_bytes || NUL
```

Entries must arrive in strictly increasing portable path order. A path that
cannot round-trip through the portable path contract, a non-increasing path,
or any skipped discovery object clears fingerprint compatibility. The digest
remains present for diagnostics, but it cannot carry `complete=true`.

This identity is deliberately distinct from the location-bound workspace ID
and the domain-separated snapshot ID. It exists only to bind an isolated
evaluation source tree to the product's measured reads.

### Engine projection

`LocalEngine` exposes a value-only telemetry projection. It joins current
workspace counters with the current snapshot fingerprint. Completeness is the
logical conjunction of:

- ledger completeness;
- a current snapshot;
- snapshot completeness;
- portable fingerprint compatibility; and
- zero skipped objects.

The engine projection contains no root path, individual file path, source
bytes, query, packet text, or cache location.

### MCP result

After a successful `context_build`, the thin MCP adapter adds:

```json
{
  "read_telemetry": {
    "schema_name": "impresari_context_repository_read_telemetry",
    "schema_version": "1.0",
    "source_fingerprint_sha256": "sha256:<64 lowercase hex>",
    "repository_file_reads": 0,
    "repeated_repository_file_reads": 0,
    "source_bytes_read": 0,
    "complete": true
  }
}
```

The evaluator's admitted lifecycle is a newly launched MCP process containing
one startup snapshot and one `context_build`. The counters are therefore
cumulative for exactly that lifecycle. General long-lived MCP clients may
inspect the same process-to-date values, but must not reinterpret them as a
per-call delta.

## Invariants

- `repeated_repository_file_reads <= repository_file_reads`.
- No successful `read_exact` byte materialization bypasses the ledger.
- No packet field or caller argument can increment or reset the ledger.
- A source fingerprint never substitutes for per-evidence content hashes.
- `complete=false` is fail-closed and remains a valid product response; the
  evaluator rejects it for effectiveness admission.
- Telemetry adds no filesystem, orchestration, model, network, execution,
  persistence, or publication authority.

## Verification

1. Workspace tests exercise nested files, repeated reads, empty files,
   pre-read rejection, discovery omission, and exact fingerprint compatibility.
2. Engine tests prove safe projection and completeness conjunction.
3. MCP tests validate the closed result shape and authority flags.
4. The independent evaluator launches the exact built MCP executable against
   a disposable allowlisted fixture and validates the attestation without any
   provider or grader.

## Failure behavior

Observer overflow, poisoned synchronization, incompatible path identity,
partial discovery, skipped objects, or absent snapshot sets `complete=false`.
The product remains available for ordinary bounded context use; evaluation
admission fails without changing the underlying context result.
