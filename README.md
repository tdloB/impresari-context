# Impresari Context

> Internal working name only. The public project name remains subject to the
> project's naming and counsel gate.

This directory defines the requirements, architecture, and boundaries for a new, independent
open-source context engine for AI-assisted software development. The project is
informed by public capabilities demonstrated by LeanCTX and Graft, but it is not
a fork, merger, official successor, or source-code combination of either.

The engine will provide a secure, evidence-grounded way for an AI client to
understand a software workspace without becoming the client's orchestrator or
silently taking control of the developer's environment.

## Status

The approved local Verifiable Context Slice A is under active implementation.
The Rust workspace now includes capability-scoped workspace reads,
deterministic snapshots, isolated SQLite cache and audit stores, bounded exact
path/filename/literal/lexical retrieval, byte-verifiable evidence, immutable
context packets, packet validation, no-overwrite handoff export, one shared
in-process capability service, and a thin JSON CLI.

This is not a public release. The name remains provisional, parser/structural
work is not authorized, and external integrations, remote repository creation,
package publication, and release remain gated. Security/evaluation evidence and
native Tier A release rehearsals must still pass before release readiness can be
claimed.

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
```

Each invocation receives the explicit workspace and cache roots. This avoids a
durable ambient mapping from an opaque handle to an absolute source path. Use
`--at`, `--cutoff`, and `--id-seed` for deterministic automation and conformance
tests. See `--help` for evidence recovery, packet validation, snapshot status,
and handoff export forms.

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

## Mission

Make repository context compact enough for an AI agent to use, precise enough
for a human to verify, and constrained enough to operate safely in an untrusted
workspace.

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
