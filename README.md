# Impresari Context

![Impresari Context pixel-art repository map framed by a deep burgundy theater curtain](assets/impresari-context-stage-banner.png)

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

[`v0.1.0`](https://github.com/tdloB/impresari-context/releases/tag/v0.1.0)
was published on 2026-08-23 UTC (2026-08-22 EDT) from commit
`c77e95ce95b2fde99da2582707d4e4d58a512122`. It provides checksummed,
provenance-attested archives for macOS ARM64, Linux x86-64, and Windows x86-64.
The default branch includes later work; its current capability claims must not
be attributed retroactively to the `v0.1.0` binaries. Executable or privileged
extensions and long-lived transports remain separately gated. The complete
local gate, hosted Tier A matrix, and native clean-install release checks pass.
The repository also runs CodeQL, OpenSSF Scorecard, dependency and license
audits, secret protection, and a bounded coverage-guided Rust fuzz target. The
release workflow implements the approved SHA-256 checksum and GitHub artifact
attestation design. Independent human security and release review remains a
documented assurance target before `v1.0.0`, or earlier if the project
materially expands its trust boundary. It was not a mandatory `v0.1.0` gate;
the published release discloses the absence of an independent third-party
security audit.

See the [`v0.1.0` conformance statement](CONFORMANCE.md) for the exact artifact
scope, normative contracts, verification basis, limitations, and non-claims.

The workspace pins Rust 1.98.0 and declares Rust 1.96 as its initial MSRV. Run
the complete local quality gate with `./scripts/check.sh`. Current milestones
are tested on Rust 1.96.0, 1.97.0, and stable, with Clippy warnings denied,
Draft 2020-12 response validation, deterministic identity/path/JCS vectors,
dependency policy, and RustSec auditing.

## Get the software, ask for help, or contribute

Download the published binaries and checksum files from the
[`v0.1.0` release](https://github.com/tdloB/impresari-context/releases/tag/v0.1.0),
or use the checksum-verifying installer on macOS ARM64 or Linux x86-64. Download
and inspect the installer before running it:

```text
curl --fail --location --output impresari-install.sh \
  https://raw.githubusercontent.com/tdloB/impresari-context/main/scripts/install.sh
less impresari-install.sh
sh impresari-install.sh --version v0.1.0
```

Beginning with the next release after `v0.1.0`, the same installer and its
checksum are attached to each GitHub release and receive build provenance
attestation. Use the installer attached to the version being installed.

The script never selects `latest`, changes shell startup files, or overwrites
an installed binary. Set `IMPRESARI_INSTALL_DIR` or pass `--install-dir` to
choose a different destination. Other platforms can use the release archives
directly.

Alternatively, clone the public repository and build the current source tree
with the standard Rust toolchain:

```text
git clone https://github.com/tdloB/impresari-context.git
cd impresari-context
cargo build --workspace --locked
./scripts/check.sh
```

Use [GitHub Issues](https://github.com/tdloB/impresari-context/issues) for
ordinary bugs and feature requests. Read [CONTRIBUTING.md](CONTRIBUTING.md)
before proposing a change. Do not report a suspected vulnerability in a public
issue; use the [private security advisory
channel](https://github.com/tdloB/impresari-context/security/advisories/new).

## First run

The release keeps `impresari-context` and `impresari-context-mcp` together, so
the CLI can preview a first-class connection without another binary-path
argument:

`quickstart` is a post-`v0.1.0` capability. It is available when building the
current source tree and will be included in the next published release; it is
not present in the existing `v0.1.0` binaries.

```text
impresari-context quickstart cursor \
  /absolute/path/to/workspace \
  /absolute/path/to/separate-cache \
  /absolute/path/to/workspace/.cursor/mcp.json
```

Review the machine-readable receipt, then repeat the same command with
`--apply`. Quickstart does not guess any of the three paths and does not trust,
start, sign in to, enable, approve, or invoke the client. Its receipt lists the
remaining client-controlled steps. The same command supports `codex`, `claude`,
`cursor`, `copilot`, and `vscode` within their recorded scopes.

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
cargo run -p context-cli -- dashboard serve <audit-cache-root> <policy-state-root>
```

Each invocation receives the explicit workspace and cache roots. This avoids a
durable ambient mapping from an opaque handle to an absolute source path. Use
`--at`, `--cutoff`, and `--id-seed` for deterministic automation and conformance
tests. See `--help` for evidence recovery, packet validation, snapshot status,
handoff export, and structural forms. Structural build never downloads or
discovers a parser: the embedding distribution must provide the exact worker,
its expected SHA-256 identity, and an existing empty non-workspace directory.

The post-`v0.1.0` dashboard command starts one foreground, loopback-only local
metadata session and prints a one-use fragment-bearing `bootstrap_url`. Open
that URL manually. The dashboard reads only validated audit metadata and an
exact-owned, separate budget-policy state root; it does not display source or
packets, open a browser, create a daemon, make outbound requests, or raise any
governing limit. The one-use bootstrap returns a separate 256-bit API-route
capability retained only in bundled-page memory; no cookie, query parameter,
or browser storage carries dashboard authority. DBC-1 through DBC-4 are
implemented, and the native-browser adversarial admission is recorded in
`docs/verification/dbc-4-native-browser-admission.md`.

## Local MCP

`impresari-context-mcp` is a local child-process transport that prefers MCP
revision `2025-11-25` and accepts `2025-06-18` for compatible clients. It
communicates only through stdin/stdout, emits only MCP messages on stdout, has
no HTTP listener, and adds no orchestration, execution, approval, model,
network, or filesystem authority.

```text
cargo run -p context-mcp -- \
  --workspace <workspace-root> \
  --cache <cache-root> \
  --consumer-id <consumer-id> \
  --role <policy-role>
```

The client must complete MCP initialization before using the six published
tools: `context_session_open`, `context_build`,
`context_convention_exemplar_build`, `structure_incremental_update`,
`context_packet_resolve`, and `context_session_close`. The process is
intentionally single-client and process-local. It is not a remote service or
an agent runtime.

The process records a local UTC startup time by default. Deterministic test or
rehearsal callers may append `--occurred-at <UTC-timestamp>`; persistent client
configuration must not hard-code a timestamp merely to start the process.

See the [CLI and local MCP interface reference](docs/reference/interfaces.md)
for complete commands, request and response contracts, error behavior,
versioning, limits, and security boundaries.

See the [compatibility matrix](docs/reference/compatibility-matrix.md) for the
exact difference between discovery, lexical evidence, and structural language
support, and between generic local-MCP compatibility and a first-class client
integration. In the current source tree, TypeScript/JavaScript, Python, Java,
Kotlin, C#, Scala, Elixir, Clojure, Haskell, Go, Rust, strict JSON, JSONC, TOML,
YAML, C, C++, Ruby, PHP, and Swift have bounded structural support. Codex,
Claude Code, Cursor, GitHub Copilot CLI, and VS Code Copilot are first-class
only for their recorded client/version/OS scopes. Gemini CLI remains a generic
local-MCP integration.

Codex, Claude Code, Cursor, GitHub Copilot CLI, and VS Code Copilot also have
recorded-scope L2 native guidance and L4 lifecycle-health evidence for their
exact recorded artifacts. Their independently versioned L3 claims remain
separate, and Codex L4 does not cover its App Server L3 version. These
claims describe the current source tree, not the earlier `v0.1.0` binaries.
Native guidance is opt-in and does not make conversational tool selection
deterministic. See the [compatibility matrix](docs/reference/compatibility-matrix.md),
the [Codex L2 record](docs/verification/phase-2-codex-native-guidance.md), the
[Claude Code L2 record](docs/verification/phase-2-claude-code-native-guidance.md),
the [Cursor L2 record](docs/verification/phase-2-cursor-native-guidance.md),
the [Copilot CLI L2 record](docs/verification/phase-2-copilot-cli-native-guidance.md),
and the [VS Code L2 record](docs/verification/ci-2-vscode-copilot-native-guidance.md).

Codex has a recorded-scope CI-3b guided-delivery preview/apply path for its exact
App Server `0.150.0-alpha.8` macOS arm64 scope. It is disabled by default,
requires an explicit consented intent, a separately inspected packet preview,
the displayed packet ID, and `--apply`. It starts only an ephemeral read-only/
no-network session and denies authority requests. Two isolated deliveries
completed with immutable source and no added authority; see the
[CI-3b verification record](docs/verification/ci-3b-codex-guided-delivery.md).

The [local MCP connection guides](docs/reference/local-mcp-connection-guides.md)
show user-invoked, non-mutating local stdio configurations for those clients.

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
- [ADR-0017](docs/decisions/0017-v0.1-release-assurance-policy.md): the
  independent-review assurance target and exact `v0.1.0` disclosure policy.
- [ADR index](docs/decisions/README.md): the accepted runtime, platform,
  parser, identity, storage, budget, license, and governance decisions.
- [Master Product PRD](docs/product/master-prd.md): product mission, users,
  release slices, requirements, outcomes, and implementation decision gates.
- [Verifiable Local Context MVP PRD](docs/product/verifiable-local-context-mvp-prd.md):
  the exact scope and acceptance contract for the first executable slice.
- [Security Threat Model](docs/security/threat-model.md): trust zones, threats,
  controls, residual risks, and release-blocking security evidence.
- [Hostile-repository admission limitations](docs/security/hostile-repository-admission-limitations.md):
  exact Step 1 capabilities, absent analyzers/quarantine, and prohibited safety
  or execution claims.
- [Hostile-repository security expansion](docs/product/hostile-repository-admission-prd.md):
  accepted HRA-0 contracts, HRA-1 bounded read-only inventory, completed narrow
  HRA-2 npm/Compose observations, HRA-3 unavailable-by-default coverage,
  bounded synthetic analyzer-result normalization and immutable assessment
  construction, the HRA-4 pure deterministic reference evaluator, and the
  completed HRA-5 three-platform release-readiness evidence plus the IAR-0
  protocol and IAR-1A application-enforced synthetic-supervision baseline. No
  real analyzer, verified OS/network sandbox, networking, upload, deep-parser,
  exception, approval, quarantine, or repository-execution implementation is
  admitted; the evaluator can return only an authority-neutral quarantine-stage
  eligibility classification.
- [Isolated Analyzer Runner](docs/product/isolated-analyzer-runner-prd.md): the
  accepted separated-runner requirements, closed protocol, private synthetic
  staging, exact executable pinning, bounded subprocess supervision, explicit
  application-only posture, and pending IAR-1B OS-confinement gate.
- [IAR-1A verification](docs/verification/iar-1-application-supervision.md):
  delivered controls, fail-closed process tests, exact non-claims, and
  reproduction commands for the synthetic-supervision baseline.
- [IAR-1B macOS feasibility](docs/verification/iar-1b-macos-xpc-feasibility.md):
  development-only App Sandbox/private-XPC evidence, exact native denials, and
  the resource, lifecycle, signing, packaging, and compatibility gates that
  keep the result partial and unadmitted.
- [IAR-1B macOS resource/lifecycle decision](docs/verification/iar-1b-macos-resource-lifecycle-decision.md):
  the selected hybrid App Sandbox/private-XPC plus Rust-supervisor candidate,
  its passing synthetic resource/lifecycle probes, and its remaining hard gates.
- [IAR-1B macOS local-VM host interruption](docs/verification/iar-1b-macos-local-vm-host-interruption.md):
  the source-free shared sleep-observer stop/cleanup/recovery implementation,
  its passing synthetic trigger, and the explicit remaining real-sleep gate.
- [IAR-1B macOS local-VM guest supply chain](docs/verification/iar-1b-macos-local-vm-guest-supply-chain.md):
  the exact expiring candidate manifest, SBOM/license/provenance/policy records,
  offline prepared-component verification, and explicit remaining signing and
  production gates.
- [IAR-1B macOS local-VM upstream authentication](docs/verification/iar-1b-macos-local-vm-upstream-authentication.md):
  the verified Alpine release-key/signature chain, exact embedded guest-input
  binding, and explicit separation from Impresari distribution sealing.
- [IAR-1B macOS local-VM vulnerability disposition](docs/verification/iar-1b-macos-local-vm-vulnerability-disposition.md):
  the exact stale-kernel denial, incomplete-advisory-coverage boundary, and
  mandatory replacement route without a vulnerability-free claim.
- [IAR-1B macOS local-VM current guest replacement](docs/verification/iar-1b-macos-local-vm-current-guest-replacement.md):
  the authenticated current Alpine package, versioned v2 guest identity chain,
  repeated native matrices, and continued fail-closed production denial.
- [IAR-1B macOS local-VM release-metadata sealing](docs/verification/iar-1b-macos-local-vm-release-metadata-sealing.md):
  the exact content-addressed active metadata set, deterministic offline
  receipt, and explicit separation from signed or sealed distribution.
- [IAR-1B Windows native contract preflight](docs/verification/iar-1b-windows-native-contract-preflight.md):
  the exact LPAC/AppContainer and Job Object target profile, hosted no-worker
  capability/lifecycle probe, and explicit separation from OS confinement.
- [IAR-1B Windows native synthetic worker matrix](docs/verification/iar-1b-windows-native-synthetic-worker-matrix.md):
  the closed suspended-worker launch, boundary, resource, cleanup, and
  cross-job contract plus the fail-closed hosted `unsupported_host` result;
  native worker evidence and admission remain pending.
- [IAR-1B Linux production-topology feasibility](docs/verification/iar-1b-linux-production-topology-feasibility.md):
  the accepted rootless plus externally managed delegation profiles, closed
  source-free evaluator, and deferred privileged installation boundary.
- [IAR-1B Linux external lifecycle composition](docs/verification/iar-1b-linux-external-lifecycle-composition.md):
  the exact C package, topology, interruption, crash, cleanup, explicit
  post-collection health-withdrawal contract, and hosted lifecycle-candidate
  evidence.
- [IAR-1B Linux external production-support admission](docs/verification/iar-1b-linux-external-production-support-admission.md):
  the exact, expiring C support scope and immutable-release gate; the current
  candidate remains `release_pending`, with production and real analyzers closed.
- [YARA-X artifact compatibility](docs/verification/yara-x-artifact-compatibility.md):
  the exact v1.20.0 hosted synthetic build, isolation, five-case corpus, cleanup
  evidence, and explicit non-production/non-IAR-2 boundary.
- [YARA-X NDJSON adapter](docs/verification/yara-x-ndjson-adapter.md): the pure
  bounded parser, closed schemas, deterministic path-free output, synthetic
  provenance, and explicit no-execution/no-production boundary.
- [macOS hybrid XPC distribution](docs/product/macos-hybrid-xpc-distribution-prd.md):
  the accepted Option C target—one signed/notarized cask with CLI compatibility—
  and the evidence required before it becomes a supported release path.
- [Evaluation PRD](docs/product/evaluation-prd.md): benchmark corpus, baselines,
  metrics, reproducibility requirements, and release gates.
- [Release evidence](docs/verification/release-evidence.md): archived hosted
  native-matrix results, published `v0.1.0` provenance, and current assurance
  targets.
- [v0.2 independent security review brief](docs/verification/v0-2-independent-security-review-brief.md):
  the exact product commit, reviewer independence requirements, assessment
  areas, questions, report format, and finding-disposition gate. ADR-0084
  backlogs engagement until candidate freeze; review remains mandatory before
  tag or publication.
- [MCP and release traceability](docs/verification/mcp-release-traceability.md):
  direct-engine equivalence, transport hardening, and packaging evidence.
- [Compatibility matrix](docs/reference/compatibility-matrix.md): versioned
  language and client capability claims.
- [Local MCP connection guides](docs/reference/local-mcp-connection-guides.md):
  reviewed, user-invoked generic-client configurations without auto-wiring.

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
DCO 1.1 sign-off and contributor-retained copyright. Ordinary bugs and feature
requests belong in [GitHub Issues](https://github.com/tdloB/impresari-context/issues),
and proposed changes follow [CONTRIBUTING.md](CONTRIBUTING.md). Security concerns should
be reported through [GitHub's private vulnerability reporting
channel](https://github.com/tdloB/impresari-context/security/advisories/new);
they should never be placed in a public issue.
