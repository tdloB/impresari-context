# Trusted MCP Structural Lifecycle — Architecture Requirements and Design

- Status: Proposed; implementation follows the hosted provider-free utility gate.
- Date: 2026-09-01.
- Governing PRD:
  [Trusted MCP Structural Lifecycle PRD](../product/trusted-mcp-structural-lifecycle-prd.md).
- Governing decision:
  [ADR-0120](../decisions/0120-bind-structural-runtime-to-trusted-mcp-startup.md).

## Architecture outcome

```text
trusted process launch
  ├─ ordinary: workspace + cache + identity
  └─ structural: ordinary + worker path + SHA-256 + empty directory
          │
          ▼
 open engine -> complete snapshot -> digest-bound graph preparation
          │                              │
          └──────── same MCP tools ──────┘
                         │
             identical context_build input
                         │
          ordinary profile or product-owned seeded profile
                         │
        packet + plan + cumulative reads + lifecycle receipt
```

The worker tuple is process launch authority, not tool input. The MCP adapter
owns no retrieval policy: it stores an optional prepared graph and delegates
task-signal extraction, seed choice, traversal, evidence recovery, ordering,
and verification to the shared engine.

## Startup contract

The current eight or ten ordinary arguments remain valid. Structural startup
adds exactly these three named values as one closed tuple:

- `--structural-worker <absolute regular file>`;
- `--structural-worker-sha256 sha256:<64 lowercase hex>`;
- `--structural-empty-directory <absolute existing empty directory>`.

The parser rejects duplication, reordering, unknown flags, partial tuples,
relative paths, workspace overlap, symlinks, and invalid digests. The worker
launcher re-reads and hashes the executable immediately before every executed
request, clears its environment, uses the empty directory, applies the
existing timeout/output caps, and accepts one framed response.

## Cache provenance

The worker request carries the trusted executable digest. The internal worker
protocol version is advanced because the field is required and
deny-unknown-fields remains active. `worker_toolchain_identity` hashes the
digest with the parser runtime, grammar, resolver, graph version, fact classes,
language, and traversal controls. Therefore an old parser result cannot be
replayed after the configured executable changes even if all declared version
strings remain equal.

The digest is not repeated on every graph fact or evidence record. The MCP
lifecycle receipt records it once; graph identity remains derived from
validated graph content and snapshot identity. A digest change with identical
output may retain an equal graph ID while still producing a distinct cache
lineage and receipt.

## Preparation and failure behavior

1. Open the authorized workspace and build the same complete startup snapshot
   used by ordinary MCP.
2. In structural mode, construct a `WorkerLauncher` solely from the trusted
   tuple and call `LocalEngine::build_structure` with the fixed startup budget.
3. Record elapsed preparation time around the engine call. Product read
   telemetry naturally includes exact source reads performed during graph
   construction; cache hits never erase those reads.
4. Validate graph completeness and exact snapshot binding before constructing
   `McpServer`.
5. Any failure exits before writing a valid MCP initialization response. The
   process never falls back to ordinary mode after structural intent exists.

No-worker startup constructs the existing server state and performs no worker
metadata access.

## Context-build routing

`ServerConfig` receives an optional private structural runtime containing the
prepared graph and immutable receipt. The public `context_build` grammar does
not gain graph-oracle fields for this path. For the existing `(profile, query)`
form:

- no runtime: call `build_profiled_context`;
- runtime present: call `build_profiled_seeded_structural_context` with the
  prepared graph and a closed default edge-kind set.

Existing explicit graph/start-node forms remain compatibility paths but are
not admitted for controlled evaluation. The independent evaluator must reject
responses whose lifecycle receipt does not match the intended arm.

## Receipt

The result adds a closed object:

```json
{
  "structural_lifecycle": {
    "schema_name": "impresari_context_structural_lifecycle",
    "schema_version": "1.0",
    "enabled": true,
    "state": "prepared",
    "graph_id": "sha256:<...>",
    "snapshot_id": "sha256:<...>",
    "worker_sha256": "sha256:<...>",
    "preparation_elapsed_ms": 0
  }
}
```

Ordinary mode emits the same object with `enabled=false`, `state="disabled"`,
zero elapsed time, and absent graph/worker identities. The object contains no
paths, source text, task text, cache location, environment values, or secrets.

## Evaluation interpretation

For a cold comparison, each arm receives a fresh process and distinct empty
cache. End-to-end latency starts before launch; the receipt's preparation time
is diagnostic decomposition only. Cumulative product telemetry is the source
of repository-read counts. A later warm-cache study must be separately named
and may not be mixed with cold results.

This realizes the applicable Graft pattern—deterministic structural graph,
content-addressed refresh, and product-owned query—without adopting generated
LLM summaries, repository wiring, or hidden background refresh. LeanCTX-style
progressive disclosure remains downstream because Impresari must first measure
one complete static structural delivery path.

## Security verification

- reject source-root, cache-root, or ancestor/descendant overlap with the empty
  worker directory;
- reject worker and empty-directory symlinks;
- reject wrong digest, uppercase/noncanonical digest, missing executable,
  nonempty directory, oversized stdout/stderr, timeout, abnormal exit,
  malformed frame, and stale graph;
- prove caller graph/start-node values are unnecessary for the admitted route;
- prove source hashes and source tree remain unchanged;
- prove the no-worker server performs no structural executable read or launch;
- prove cache reuse changes when only the worker digest changes.

## Next decision

After this lifecycle and an independent provider-free MCP comparison pass,
specify a LeanCTX-informed progressive disclosure contract using existing
evidence handles, exact hashes, bounded expansion, and cumulative session
budgets. Do not add durable memory or request-proxy rewriting as part of that
decision.
