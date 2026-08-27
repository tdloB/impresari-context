# Impresari Context — Phase 2: Infrastructure Languages and Agent Expansion PRD

- Status: Approved; implementation in progress
- Date: 2026-08-23
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)

## Objective

Extend evidence-grade structural support to Rust and Go for infrastructure and
platform teams, while adding tested local integrations for Gemini CLI, GitHub
Copilot CLI, and VS Code Copilot.

## Scope

- Rust and Go structural language admission through isolated pinned grammars.
- Tested, versioned, opt-in local connection kits for Gemini CLI, GitHub Copilot
  CLI, and VS Code Copilot.

## Current delivery state

- Complete: Rust and Go structural evidence, including public compatibility,
  dependency/SBOM, and full hosted verification evidence.
- Complete: non-mutating preadmission guides and read-only configuration
  validators for Gemini CLI, GitHub Copilot CLI, and VS Code Copilot. Each
  preserves fixed local stdio authority and rejects remote transport,
  environment forwarding, and automatic approval.
- Complete: GitHub Copilot CLI `1.0.80` on macOS aarch64 is First-class for
  its recorded project scope. It has malformed-configuration rejection,
  isolated native project `list/get` discovery after exact workspace trust, a
  bounded prompt-mode lifecycle with direct packet equivalence, source
  immutability, and exact project-entry/trust removal evidence.
- In implementation: VS Code Copilot's distinct extension-host/Agent Host
  admission now has a portable `.mcp.json` strict-stdio contract and a
  disposable operator-evidence runner. It remains generic until a signed-in
  real-client record establishes trust, discovery, tool use, immutability, and
  exact removal for the recorded version/OS scope.
  Gemini CLI `0.56.0` is authenticated but its current free-tier service
  rejects normal client startup as unsupported, so it remains generic legacy
  compatibility rather than an active depth target.

## Non-goals

- Compiler, package manager, language server, module/crate resolution, runtime
  behavior, remote transport, or automatic client-configuration mutation.

## Acceptance criteria

- The Rust and Go worker identities, facts, limitations, and public claims stay
  pinned and fail closed.
- Each broader-agent kit satisfies the same first-class admission evidence as
  Phase 1 without adding source-write, network, execution, or orchestration
  authority.
