# Impresari Context — Phase 1: Language, Configuration, and Client Admission PRD

- Status: Approved; implementation in progress
- Date: 2026-08-23
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)

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

- Complete: Python, narrow strict-JSON, and bounded JSONC configuration
  evidence.
- Complete: Codex's non-mutating project-scoped pre-admission kit and read-only
  configuration validator, while Codex remains Generic local MCP pending full
  client admission.
- Pending: TOML, YAML, Codex first-class admission, and all Claude Code
  and Cursor first-class kit work.

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
