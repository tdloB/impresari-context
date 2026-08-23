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
- Pending: all three broader-agent connection kits.

## Non-goals

- Compiler, package manager, language server, module/crate resolution, runtime
  behavior, remote transport, or automatic client-configuration mutation.

## Acceptance criteria

- The Rust and Go worker identities, facts, limitations, and public claims stay
  pinned and fail closed.
- Each broader-agent kit satisfies the same first-class admission evidence as
  Phase 1 without adding source-write, network, execution, or orchestration
  authority.
