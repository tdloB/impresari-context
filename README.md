# Impresari Context

Impresari Context is a local-first evidence compiler for AI-assisted software development. It transforms an exact repository snapshot into bounded, task-specific context packets with recoverable source evidence, explicit exclusions, integrity checks, and freshness validation.

It is intentionally not an agent orchestrator or an all-in-one development runtime. Impresari Context works beneath existing coding agents, CI systems, and orchestration frameworks without taking control of execution, permissions, approvals, or business policy. This separation keeps the trusted core smaller, reduces competing authority, and allows adopters to add context infrastructure without replacing their existing workflow.

Impresari Context is an independent implementation informed by publicly demonstrated ideas in LeanCTX and Graft. It is not a fork, merger, official successor, or source-code combination of either project.

## Status

The approved implementation roadmap through the safe, declarative portion of
Slice D is complete and gated. Release-candidate engineering and the local
stdio MCP transport are implemented and have passed native hosted rehearsal.
The Rust workspace now includes capability-scoped workspace reads,
deterministic snapshots, isolated SQLite cache and audit stores, bounded exact
path/filename/literal/lexical retrieval, byte-verifiable evidence, immutable
context packets, packet validation, no-overwrite handoff export, one shared
in-process capability service, and a thin JSON CLI. The first Slice B milestone
adds pinned, short-lived TypeScript/JavaScript parser workers, a canonical
snapshot-bound structural graph, and bounded deterministic graph traversal with
explicit unresolved and truncated states. Slice C adds deterministic
multi-strategy context plans, consumer-scoped packet references, a thin
OS-shaped adapter contract, an independent non-OS reference client, and a
fail-closed native-read fallback decision that never grants filesystem access.
The first Slice D milestone adds integrity-pinned extension declarations and
metadata-only output quarantine while keeping all extension artifact loading,
execution, network, model, environment, and filesystem capabilities disabled.
The MCP adapter exposes the existing engine and process-local session
capabilities over bounded newline-delimited JSON-RPC. Its launch configuration
fixes the authorized workspace, cache, consumer identity, and role; MCP cannot
broaden them.

This is not a public release. The name remains provisional, executable or
privileged extensions and long-lived transports remain separately gated, and
package publication and release remain gated. The complete local gate and
hosted Tier A matrix and native clean-install release-candidate rehearsals pass.
The release provenance/signing decision and independent review remain
public-release requirements.

The workspace pins Rust 1.98.0 and declares Rust 1.96 as its initial MSRV. Run
the complete local quality gate with `./scripts/check.sh`. Current milestones
are tested on Rust 1.96.0, 1.97.0, and stable, with Clippy warnings denied,
Draft 2020-12 response validation, deterministic identity/path/JCS vectors,
dependency policy, and RustSec auditing.

## Local CLI

The CLI is a thin adapter over the same `context-engine::LocalEngine` handlers
used by in-process clients. It emits versioned JSON to stdout; `--human` adds a
short, source-free diagnostic to stderr.

```text
cargo run -p context-cli -- --help
cargo run -p context-cli -- snapshot build <workspace-root> <cache-root>
cargo run -p context-cli -- search <workspace-root> <cache-root> literal <query>
cargo run -p context-cli -- context build <workspace-root> <cache-root> lexical <query> <purpose>
cargo run -p context-cli -- structure build <workspace-root> <cache-root> <worker> <worker-sha256> <empty-dir>
cargo run -p context-cli -- structure query <workspace-root> <cache-root> <graph-json> <start-node> all
```

Each invocation receives the explicit workspace and cache roots. This avoids a
durable ambient mapping from an opaque handle to an absolute source path. Use
`--at`, `--cutoff`, and `--id-seed` for deterministic automation and conformance
tests. See `--help` for evidence recovery, packet validation, snapshot status,
handoff export, and structural forms. Structural build never downloads or
discovers a parser: the embedding distribution must provide the exact worker,
its expected SHA-256 identity, and an existing empty non-workspace directory.

## Local MCP

`impresari-context-mcp` is a local child-process transport pinned to MCP
revision `2025-11-25`. It communicates only through stdin/stdout, emits only MCP
messages on stdout, has no HTTP listener, and adds no orchestration, execution,
approval, model, network, or filesystem authority.

```text
cargo run -p context-mcp -- \
  --workspace <workspace-root> \
  --cache <cache-root> \
  --consumer-id <consumer-id> \
  --role <policy-role> \
  --occurred-at <UTC-timestamp>
```

The client must complete MCP initialization before using the four tools:
`context_session_open`, `context_build`, `context_packet_resolve`, and
`context_session_close`. The process is intentionally single-client and
process-local. It is not a remote service or an agent runtime.

## Design documents

- [Architecture](docs/architecture.md): responsibilities, components, data
  flow, canonical contracts, and initial capability surface.
- [System boundaries](docs/boundaries.md): ownership, trust zones, OS
  integration, non-goals, and deployment separation.
- [Influences and provenance](docs/influences-and-provenance.md): upstream
  acknowledgment and rules that preserve an independent implementation.
- [ADR-0001](docs/decisions/0001-independent-core-and-thin-adapters.md): the
  decision to build one neutral core with thin consumer-specific adapters.
- [ADR-0009](docs/decisions/0009-path-and-identity-encoding.md): exact native
  path, workspace identity, canonical JSON value, and hash-envelope contracts.
- [ADR-0010](docs/decisions/0010-structural-worker-protocol-and-isolation.md):
  the pinned, capability-reduced parser-worker boundary and all-or-nothing
  structural promotion contract.
- [ADR-0012](docs/decisions/0012-context-plans-consumer-adapters-and-fallback.md):
  deterministic context plans, thin consumer integration, and governed
  native-read fallback.
- [ADR-0013](docs/decisions/0013-extension-contracts-without-code-loading.md):
  pinned extension contracts and output quarantine without a plugin runtime.
- [ADR-0014](docs/decisions/0014-local-stdio-mcp-transport.md): local-only MCP
  over stdio as a bounded, authority-neutral transport.
- [ADR-0015](docs/decisions/0015-release-candidate-builds-and-provenance.md):
  exact-commit native release candidates, manifests, checksums, and rehearsal.
- [ADR index](docs/decisions/README.md): the accepted runtime, platform,
  parser, identity, storage, budget, license, and governance decisions.
- [Master Product PRD](docs/product/master-prd.md): product mission, users,
  release slices, requirements, outcomes, and implementation decision gates.
- [Verifiable Local Context MVP PRD](docs/product/verifiable-local-context-mvp-prd.md):
  the exact scope and acceptance contract for the first executable slice.
- [Security Threat Model](docs/security/threat-model.md): trust zones, threats,
  controls, residual risks, and release-blocking security evidence.
- [Evaluation PRD](docs/product/evaluation-prd.md): benchmark corpus, baselines,
  metrics, reproducibility requirements, and release gates.
- [Release evidence](docs/verification/release-evidence.md): archived hosted
  native-matrix results and explicitly open release gates.
- [MCP and release traceability](docs/verification/mcp-release-traceability.md):
  direct-engine equivalence, transport hardening, and packaging evidence.

## Mission

Give AI agents the repository evidence they need and give humans a reliable way to verify exactly what they received—without allowing untrusted workspace content to control tools, policy, or workflow.

## Architecture principles

1. Evidence before summary.
2. Deterministic structure before model-generated interpretation.
3. Every derived claim must be recoverable to exact source evidence.
4. Repository content and extension output are untrusted data, never
   instructions.
5. One canonical structural graph per workspace snapshot.
6. The core is read-only with respect to source workspaces.
7. Network access, code execution, editing, and durable-memory promotion are
   separate capabilities and denied by default.
8. MCP is a transport adapter, not the internal architecture.
9. Consumers own orchestration, approvals, and business policy.
10. A small stable capability surface is preferable to overlapping tools.

## License and contributions

Original project work is licensed under Apache License 2.0. Contributions use
DCO 1.1 sign-off and contributor-retained copyright. The legal steward,
security contact, enforcement contact, and owner/counsel/public-name gates in
ADR-0008 remain unresolved and must be completed before publication.
