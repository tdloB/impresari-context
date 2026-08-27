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
- Complete: Codex's explicit user-level configuration kit, read-only
  configuration validator, malformed-configuration rejection, isolated
  install/client-recognition/exact-removal record, and deterministic App
  Server direct-tool lifecycle plus packet-equivalence rehearsal. Codex is
  First-class only for the recorded Codex CLI/macOS aarch64 scope.
- Complete: Claude Code's explicit local-scope configuration kit, read-only
  JSON validation, malformed strict-configuration rejection, isolated native
  `claude mcp add/get/remove --scope local` lifecycle, and bounded temporary
  model-directed lifecycle with direct packet equivalence. Claude Code is
  First-class only for the recorded CLI `2.1.241` macOS aarch64 scope.
- Complete: Cursor's explicit project configuration kit, read-only JSON
  validation, malformed-configuration rejection, native isolated
  enable/list-tools/disable/removal, and guarded Agent-mode four-tool lifecycle
  with direct packet equivalence. Cursor is First-class only for the recorded
  Agent CLI `3.17.8` (`2026.08.11-e8db854`) macOS aarch64 scope.
- Complete: Phase 1's named language/configuration evidence and first-class
  Codex, Claude Code, and Cursor recorded-scope integrations. Ongoing client
  maintenance and deeper L2–L4 integrations remain separate roadmap work.

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
