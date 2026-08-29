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
recorded managed clients. Homebrew distribution now has a reviewable
[proposed PRD](homebrew-distribution-prd.md), architecture, and ADR; it remains
unapproved until its separate tap and trust boundary receive an explicit
founder decision. Automatic update installation remains a later, separate
future increment with a reviewable
[proposed PRD](automatic-update-installation-prd.md), architecture, and ADR.
It remains unapproved and separately gated on signing-root custody, scheduled
background execution, and live-rehearsal authority so neither adoption feature
can silently expand installer or background authority.

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
