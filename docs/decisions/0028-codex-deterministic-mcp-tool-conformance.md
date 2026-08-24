# ADR-0028: Codex deterministic MCP tool conformance

- Status: Accepted
- Date: 2026-08-23
- Scope: Codex-specific Phase 1 local-MCP conformance only

## Context

The Phase 1 client contract requires repeatable lifecycle and packet evidence
before a named client can be called First-class. A conversational Codex run is
not a repeatable test client: even with a fixed prompt, the model may select a
different MCP tool or no MCP tool. It can provide usability evidence, but it
cannot prove a fixed client-transport lifecycle.

Codex App Server exposes an explicit local MCP tool-call RPC. It can initialize
a configured stdio child process and invoke a named tool with supplied JSON
arguments without asking a model to choose an operation.

## Decision

Use Codex App Server's direct MCP tool-call surface for deterministic Codex
conformance. The opt-in local rehearsal must:

- supply a one-use MCP definition through App Server command-line overrides;
- use an isolated temporary fixture workspace and separate caches;
- start an ephemeral read-only App Server thread;
- list the dedicated MCP server and directly invoke session open, context
  build, packet resolve, and session close with fixed values;
- prove the resolved session packet equals the delivered packet;
- prove the direct-engine/in-process-MCP equivalence through `doctor mcp`;
- prove a raw child-process MCP packet equals the Codex-delivered packet; and
- hash the fixture workspace before and after execution.

The rehearsal must not write a Codex configuration file, trust a project,
change approval state, invoke a conversational model, request network access,
or introduce a Codex dependency into the engine.

## Consequences

Codex has deterministic local transport and packet evidence on the observed
version/OS. It remains **Generic local MCP** until the separate First-class
criteria are also met: trusted-project clean-install/configuration-parser
evidence, malformed-configuration behavior, supported version/OS coverage,
and entry-specific removal evidence.

The test depends on Codex App Server's experimental protocol. A protocol,
configuration, or lifecycle change triggers revalidation and may remove this
evidence rather than silently broadening compatibility claims.

## Alternatives considered

### Model-directed `codex exec` prompt

Rejected as the conformance gate. Model tool selection is intentionally
non-deterministic and may vary with the same prompt.

### Automatic project or user configuration mutation

Rejected. It changes user-owned state and bypasses the opt-in client boundary.

### Codex SDK or engine dependency

Rejected. The core remains client-neutral; conformance tooling is an external
development-only Ruby script.

## Verification

- `ruby scripts/rehearse-codex-app-server.rb` passes on the recorded Codex
  version and macOS architecture.
- `./scripts/check.sh` syntax-checks the rehearsal and retains all existing
  repository, security, contract, Rust, and test gates.
- The public compatibility matrix and manifest accurately retain the Generic
  classification and link the recorded evidence.

## Revisit triggers

- Codex changes or removes App Server direct MCP tool calls.
- A rehearsal reaches a model, network, source-workspace, or client-config
  mutation path.
- Packet, lifecycle, or source-immutability equivalence fails.
- First-class admission evidence is complete and a separate client-status
  decision is required.
