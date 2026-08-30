# Impresari Context — Revised Product Roadmap

- Status: Approved
- Date: 2026-08-23
- Owner: Aaron Boldt
- Authority: Founder-approved product roadmap
- Client depth: [Client Integration Depth Roadmap](client-integration-roadmap.md)

This roadmap is the phase-sequencing source of truth. It separates completed
delivery slices from the phase that owns their product outcome. Individual
language and client additions require their own admission evidence and must not
silently broaden the authority boundary.

Release baseline: `v0.1.0` was published on 2026-08-23 UTC from commit
`c77e95ce95b2fde99da2582707d4e4d58a512122`. This roadmap also records later
default-branch work; a completed roadmap item is not, by itself, a claim that
the capability exists in the `v0.1.0` binaries.

| Phase | Outcome | Current status |
| --- | --- | --- |
| 0 | Correct public language/client contract and read-only doctor | Complete |
| 1 | Python and configuration evidence; first-class Codex, Claude Code, and Cursor kits | Complete for recorded client/version/OS scopes: Python, narrow strict-JSON, bounded JSONC, bounded TOML, deliberately bounded YAML, and the three named integrations. Codex is first-class for Codex CLI `0.149.0-alpha.4.1` on macOS aarch64 after isolated user-home installation, malformed-configuration rejection, deterministic App Server lifecycle/packet equivalence, and exact removal evidence. Claude Code is first-class for CLI `2.1.241` on macOS aarch64 after malformed strict-configuration rejection, native isolated local-scope add/get/removal, and bounded temporary-config packet-equivalence evidence. Cursor is first-class for Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64 after isolated project enable/list-tools/disable/removal and guarded Agent-mode packet-equivalence evidence. |
| 2 | Rust and Go structural evidence; broader agent access | Complete for recorded scopes: Rust and Go are complete. GitHub Copilot CLI is first-class L1 with recorded-scope L2 guidance and L4 health for CLI `1.0.80` on macOS aarch64. VS Code Copilot extension host is independently first-class L1 with recorded-scope L2 guidance for VS Code `1.134.0` on macOS arm64. Gemini remains generic because normal-client testing is blocked by its current free-tier service. |
| 3 | Deterministic context planner | Complete (approved initial scope): profile-bound deterministic plans, coverage/omission reporting, exact plan and packet identities, CLI, and MCP support are implemented. Standalone profiles retain explicit structural, change-set, associated-test, and configuration-to-code omissions rather than inferring evidence. |
| 4 | Java, Kotlin, C#, impact evidence, and incremental updates | Complete: bounded Java, Kotlin, C#, structural-impact, declared change-set, caller-declared associated-test, repository orientation, explicit incremental-update, and convention/exemplar evidence are accepted after full hosted CI. |
| 5 | Demand-led language expansion | Complete for the accepted scope: Scala, Elixir, Clojure, Haskell, C, C++, Ruby, PHP, and Swift are admitted with bounded structural evidence after independent hosted acceptance. Additional languages remain demand-gated under ADR-0040. |

## Adoption experience track

The short-install and first-run increment provides a pinned, checksum-verified
macOS/Linux installer and one preview-by-default `quickstart` command for the
recorded managed clients. ADR-0076 now accepts staged macOS work toward one
signed/notarized [CLI-compatible cask](macos-hybrid-xpc-distribution-prd.md),
but publication remains gated on IAR-1B, signing/notarization, clean-machine,
migration, upgrade, rollback, and uninstall evidence. The earlier formula
proposal remains Linux-only and separately gated. Automatic update installation remains a later, separate
future increment with a reviewable
[proposed PRD](automatic-update-installation-prd.md), architecture, and ADR.
It remains unapproved and separately gated on signing-root custody, scheduled
background execution, and live-rehearsal authority so neither adoption feature
can silently expand installer or background authority.

## Observability and budget-control track

A local real-time dashboard and narrowing-only budget-control layer have a
reviewable [proposed PRD](local-dashboard-budget-control-prd.md), architecture,
and ADR. The proposal is foreground, loopback-only, metadata-only, and unable
to raise any governing hard limit. It remains unapproved. Remote, hosted,
organization, billing, telemetry, and source-viewing surfaces remain outside
the roadmap without a separate founder decision and external-data boundary.

## Hostile-repository security expansion track

The earlier `New malware feature chat` design has been recovered into three
reviewable increments: an
[evidence-only hostile-repository admission foundation](hostile-repository-admission-prd.md),
a separate [isolated analyzer runner](isolated-analyzer-runner-prd.md), and a
[disposable quarantine runner](disposable-quarantine-runner-prd.md). The first
increment is static-first. ClamAV and YARA are the initial local scanner
direction behind the isolated runner, and any optional online reputation check
is hash-only and explicit. ADR-0073 HRA-0 contract freezing and HRA-1 bounded,
read-only artifact inventory are implemented with closed schemas, a fixed
authority-denying resource profile, explicit exclusions, and original-synthetic
fixture provenance. HRA-2 is complete with closed npm lifecycle and canonical
Compose privileged-service corpora plus exact key-token evidence. HRA-3 is
complete with deterministic unavailable-by-default analyzer coverage planning,
closed synthetic result intake through ADR-0013 normalization, and immutable
assessment assembly. HRA-4 is complete as a separate pure deterministic
evaluator with monotonic restriction, explicit incomplete-analysis handling,
and no exception or ordinary-host authorization input. HRA-5 is complete with
an exact-commit, three-platform candidate build and clean-install rehearsal;
ADR-0073 Step 1 is complete. ADR-0074 IAR-0 and the IAR-1A
application-enforced baseline are complete with closed runner and supervisor
contracts, fixed profiles, reviewed fixture provenance, exact identities,
private synthetic staging, and short-lived worker supervision. The remaining
IAR-1B OS-confinement checkpoint is not complete. None of
these components authorizes scanner execution, repository execution, artifact upload,
threat-intelligence access, or VM/cloud provisioning. Each increment remains
bound to its accepted threat-model, platform, supply-chain, privacy, and
release-evidence scope. OS-specific confinement is next; real analyzers and
Step 3 quarantine remain later, separately evidenced increments.
The initial macOS feasibility inventory also records that the available
`sandbox-exec` command is deprecated and cannot serve as the durable production
boundary merely because it is already used by a network-denied test harness.
A synthetic App Sandbox/private-XPC prototype now records a partial macOS
result: native sandbox identity, bounded IPC, and selected filesystem,
credential, process, and network denials pass, while device denial and hard
resource/process-tree limits, fault timeout, complete OS-managed container
cleanup, production signing/notarization, packaging, and multi-host evidence
remain open. It does not admit macOS or open IAR-2.
The hybrid resource/lifecycle checkpoint subsequently passed native synthetic
CPU termination, bounded address-space growth, `fork`/`posix_spawn` denial,
exact-target timeout termination, crash/relaunch, and source-byte cleanup. The
candidate combines App Sandbox/private XPC with the Rust supervisor and public
resource limits, and ADR-0076 selects one CLI-compatible Homebrew cask as its
intended release topology. macOS remains at IAR-1A until device denial,
production profiles, Developer ID/notarization, cask lifecycle, the full Tier A
corpus, and multi-host evidence pass. No privileged daemon, private API,
persistent service, or VM fallback is added.

## Parallel Client Integration Depth Track

Codex, Claude Code, Cursor, and GitHub Copilot follow the separate
[Client Integration Depth Roadmap](client-integration-roadmap.md). A client is
first-class only after its L1 managed-connection evidence passes; native
guidance, planner-backed delivery, and deep lifecycle support are separately
admitted L2–L4 claims. This track runs in parallel with Phases 1–4 and does not
block structural-language or deterministic-planner releases.

Gemini CLI remains generic legacy compatibility because Google's consumer path
has moved to Antigravity. Do not invest in additional Gemini-specific depth
until a stable Antigravity surface has been evaluated.

## Phase 0 — Correct the public contract

Correct unsupported structural-language claims; publish discovery, lexical, and
structural evidence levels; publish generic, first-class, experimental, and
unsupported client classifications; and provide a read-only doctor command for
binary, parser worker, cache, workspace isolation, and MCP-handshake checks.

## Phase 1 — Python and configuration evidence

Deliver Python plus JSON, JSONC, TOML, and deliberately bounded YAML structural
evidence. Configuration claims are limited to key/object containment, manifest
fields, safe defined include/reference relationships, and only exact
configuration-to-code references. Deliver first-class tested local connection
kits for Codex, Claude Code, and Cursor.

## Phase 2 — Rust and Go, plus broader agent access

Deliver Rust and Go structural evidence and tested local connection kits for
Gemini CLI, GitHub Copilot CLI, and VS Code Copilot. The language slices and
GitHub Copilot CLI L1/L2/L4 and the distinct VS Code Copilot extension-host
L1/L2 admissions are complete for their recorded scopes; Gemini remains legacy
generic compatibility. Deeper Copilot work is governed by the parallel
client-integration roadmap.

## Phase 3 — Deterministic Context Planner

Deliver a bounded deterministic planner—not agent governance—that consumes a
declared task profile, query, snapshot, policy, budget, and supported evidence.
It emits an explicit retrieval plan, selection reason codes, coverage report,
omitted candidates and budget reasons, and exact packet identity. Initial
profiles are orientation, implementation, bug investigation, change review,
security review, test selection, and configuration change.

## Phase 4 — Enterprise-language expansion and impact evidence

Deliver Java, Kotlin, and C# structural evidence, then strengthen change-set
packets, bounded impact paths, associated-test evidence, orientation packets,
explicit incremental structural updates, and deterministic conventions and
exemplar evidence.

## Phase 5 — Demand-led language expansion

Scala, Elixir, Clojure, Haskell, C, C++, Ruby, PHP, and Swift are admitted with
bounded structural evidence. The founder-approved five-language program in
ADR-0064 completed its independent hosted acceptance gates in order. Evaluate
any other language—including F#, Elm, Dart, and carefully constrained SQL—only
from attributable adopter demand and evaluation evidence.
