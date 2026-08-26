# Impresari Context — Phase 1: Language, Configuration, and Client Admission PRD

- Status: Approved; implementation in progress
- Date: 2026-08-23
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related decision: [ADR-0028](../decisions/0028-codex-deterministic-mcp-tool-conformance.md)

## Objective

Make Impresari Context a tested local evidence layer for TypeScript/JavaScript,
Python, and the configuration files that define how modern repositories build
and run, with first-class local integrations for Codex, Claude Code, and Cursor.

## Scope

- Structural evidence for Python, strict JSON, JSONC, TOML, and deliberately
  bounded YAML.
- Configuration facts only for keys, containment, specified manifest fields,
  safe defined include/reference relationships, and exact evidence-backed
  configuration-to-code references.
- Versioned, tested, opt-in local connection kits for Codex, Claude Code, and
  Cursor, with no automatic third-party configuration mutation.

## Current delivery state

- Complete: Python, narrow strict-JSON, bounded JSONC, bounded TOML, and
  deliberately bounded YAML configuration evidence.
- Complete: Codex's non-mutating project-scoped pre-admission kit, read-only
  configuration validator, and deterministic App Server direct-tool lifecycle
  plus packet-equivalence rehearsal. Codex remains Generic local MCP pending
  full client admission.
- Complete: Claude Code and Cursor's non-mutating generic local-MCP guides and
  read-only JSON configuration validation. Claude Code `2.1.241` completed a
  malformed strict-configuration check and an isolated temporary-config
  lifecycle with direct packet equivalence, without source mutation or
  persistent registration. Signed-in Cursor Agent CLI `3.17.8` on macOS
  aarch64 discovered an isolated temporary project configuration without
  enabling it; its malformed temporary project configuration was not loaded
  and left the fixture source unchanged. Cursor's documented command/args
  stdio form is accepted without allowing environment forwarding.
- Pending: first-class admission for Codex, Claude Code, and Cursor. Codex
  still needs trusted-project clean-install/configuration-parser and exact
  owned-entry-removal evidence; its initial supported scope is the recorded
  Codex CLI/macOS aarch64 combination. Claude Code needs a user-reviewed
  local-scope installation/removal record; Cursor still needs a user-approved
  real-client lifecycle, packet-equivalence, platform/version, and
  user-owned project-entry removal evidence.

## Non-goals

- Runtime configuration evaluation, interpolation, arbitrary configuration
  semantics, package-manager execution, or source-write authority.
- Remote MCP, daemon processes, agent orchestration, prompt injection, model
  calls, routing, persistent memory, or automatic client configuration edits.

## Acceptance criteria

- Each admitted grammar has pinned worker identity, bounded syntax-derived
  facts, compatibility entries, dependency/SBOM review, and hosted evidence.
- Each first-class kit defines version and OS scope, user/project scope,
  copyable configuration, dry-run rendering, verification, failure behavior,
  and entry-specific removal instructions.
- Clean-install client conformance proves lifecycle, packet equivalence,
  malformed-configuration handling, source immutability, and no new authority.
- Deterministic conformance is limited to the product-controlled connection
  contract and any direct client RPC surface. A model-directed client must
  instead supply a bounded live-client smoke record; repeating a natural
  language prompt is not an admission test.
