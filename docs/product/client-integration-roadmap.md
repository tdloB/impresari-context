# Impresari Context — Client Integration Depth Roadmap

- Status: Approved roadmap amendment
- Date: 2026-08-24
- Owner: Aaron Boldt
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related decisions: [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md), [ADR-0029](../decisions/0029-progressive-client-integration-depth-and-consent.md)

## Purpose

Graft and LeanCTX show that an MCP entry alone is not the whole client
experience. Their public integrations combine native setup, agent guidance,
health/installation assistance, and, where a client supports it, session or
hook-based context delivery. Impresari Context will match those *levels of
integration* with original artifacts and an evidence-first security model. It
will not copy competitor code, prompts, files, assets, or opaque behavior.

Priority clients are Codex, Claude Code, Cursor, and GitHub Copilot (CLI and
VS Code surfaces). Gemini CLI remains generic legacy compatibility; Antigravity
is a future evaluation candidate, not a current commitment.

## Integration Levels

| Level | User outcome | Required capability | Security boundary |
| --- | --- | --- | --- |
| L0 — Generic MCP | Manual use through a compatible client. | Documented local-stdio launch contract. | No claim beyond protocol compatibility. |
| L1 — First-class managed connection | Predictable install, validation, update, and removal. | Versioned kit, platform/version scope, deterministic configuration/transport/policy checks, live-client smoke evidence, precise removal. | Explicit action; preview external targets; preserve unrelated configuration; fail closed. |
| L2 — Native agent guidance | Client-native guidance on requesting bounded evidence. | Versioned, owned instruction/rule/skill artifact where supported. | Opt-in, previewable, ownership marked; repository text cannot alter policy. |
| L3 — Guided context delivery | Authorized task packet may arrive at a supported lifecycle point without relying only on model tool choice. | Client lifecycle extension plus deterministic-planner packet. | Disabled by default; explicit profile/budget; packet ID, evidence, redactions, and reasons visible; no mutation, proxy, or hidden network. |
| L4 — Deep lifecycle integration | Client-specific health, freshness, and lifecycle signals. | Stable documented lifecycle surface and maintained version/OS evidence. | No broad shell hooks or background authority; explicit degradation. |

L1 is the product meaning of **first-class**. L2–L4 are optional depth
milestones, not prerequisites for language support or the deterministic
planner. Conversational tool choice is live smoke evidence, not a claim of
deterministic client behavior.

## Delivery Sequence

## Current checkpoint — 2026-08-29

| Track | Current evidence-backed status | Next gate |
| --- | --- | --- |
| CI-1 managed connections | Shared render/inspect/validate/install/remove capability and deterministic fixture coverage exist. Codex, Claude Code, Cursor, GitHub Copilot CLI, and VS Code Copilot extension host have recorded L1 evidence for their individual macOS/client-version scopes. The VS Code claim is limited to workspace `.vscode/mcp.json`, explicit user trust/start, server discovery, bounded session-tool use, source immutability, and exact owned removal. Portable Agent Host `.mcp.json` remains a separate unadmitted path. Copilot's CLI and VS Code surfaces remain separate claims. | Maintain each admitted client after upstream changes; do not infer packet equivalence or prompt-repeatability from the VS Code L1 record. |
| CI-2 native guidance | Versioned Codex, Claude Code, Cursor, and Copilot templates; deterministic render/inspect/validate/install/remove lifecycle; static authority-boundary checks; and exact legacy-removal support are implemented. Codex, Claude Code, and Cursor retain recorded v2 L2 scopes. GitHub Copilot CLI `1.0.80` revalidated the v3 instruction, and VS Code Copilot `1.134.0` separately completed its v3 same-session packet build/resolve and exact-owned cleanup on macOS arm64. Codex has no separate deterministic instruction-source signal from its App Server. | Maintain each recorded scope after upstream client, instruction, schema, approval, or platform changes; do not infer conversational repeatability. |
| CI-3 guided delivery | CI-3a implements a strict client-neutral intent/receipt contract. CI-3b admits Codex App Server `0.150.0-alpha.8` at recorded-scope L3. CI-3c independently admits Copilot CLI `1.0.80` through two zero-tool programmatic-prompt deliveries. CI-3d independently admits Claude Code `2.1.241` through two safe-mode streamed deliveries with exact prompt acknowledgment, empty tool/MCP inventories, immutable source, bounded receipts, existing authentication used in place, and runtime cleanup. | Maintain all exact scopes after upstream changes. Copilot `1.0.81` remains excluded because its model request retained built-in tool schemas despite exclusion flags. Cursor and VS Code Copilot still need independent evidence before L3. |
| CI-4 lifecycle maintenance | A versioned manifest/receipt contract and one-shot read-only checker are implemented for the exact GitHub Copilot CLI `1.0.80`, macOS aarch64, native-guidance v3 artifact. Deterministic checks cover compatible, stale, unsupported, unrecorded-version, client-unavailable, missing, changed, unowned, malformed, source-immutability, and exact-removal states. No other client or Copilot surface has an L4 claim. | Revalidate the recorded Copilot CLI evidence before its freshness window expires or after an upstream lifecycle change. Admit other clients only through independent manifests and regression evidence. |

This checkpoint preserves the roadmap sequence. It does not promote any client
based on templates, local fixtures, conversational tool selection, or pending
hosted checks.

### CI-1 — First-Class Managed Connections

Target: Codex, Claude Code, Cursor, and Copilot. For each client, release a
versioned kit with configuration rendering/validation, version/OS scope,
deterministic launch/policy fixture, real-client smoke record,
malformed-configuration coverage, source-workspace immutability proof, and
owned-entry removal proof. Distinguish user and project scope.

Exit: promote that individual client to L1 in the public matrix.

### CI-2 — Native Guidance Artifacts

Target: Codex instructions, Claude Code skills, Cursor rules, and Copilot
instructions/agent configuration where official client surfaces support them.
Artifacts are original, minimal, versioned, previewable, and removable by their
ownership marker. They describe evidence use, never authority expansion.

Exit: L2 requires an opt-in install/validate/remove round trip plus a
client-specific client/version/OS native-surface smoke record.

### CI-3 — Planner-Backed Guided Delivery

Dependency: Phase 3 deterministic context planner. Evaluate documented client
lifecycle extension points. Where safe, offer opt-in orientation,
implementation, change-review, security-review, test-selection, and
configuration-change packets. Every delivery records packet identity/reasons,
honors the budget, and has a no-delivery fallback.

Exit: L3 requires consent, packet-equivalence, redaction, degraded-mode, and
source-immutability evidence.

### CI-4 — Deep Lifecycle Maintenance

For clients with stable native surfaces, add source-free health, freshness, and
compatibility signals. Maintain version/OS evidence and demote an integration
when upstream changes invalidate its guarantee.

Exit: L4 requires documented, tested lifecycle behavior without hidden
background activity.

## Client Intent Matrix

| Client | Current level | Planned target | Notes |
| --- | --- | --- | --- |
| Codex | L1/L2 plus recorded-scope L3 | Maintain exact scopes | L1 remains admitted for Codex CLI `0.149.0-alpha.4.1`, L2 for its separately recorded `AGENTS.md` v2 smoke, and L3 independently for App Server `0.150.0-alpha.8` on macOS arm64. L3 requires explicit preview/apply, a dedicated operator-authenticated home, an ephemeral read-only/no-network thread, exact packet binding, and no added authority. |
| Claude Code | L1/L2/L3 | Maintain exact scopes | L1 is admitted only for Claude Code CLI `2.1.241` on macOS aarch64. L2 is separately admitted for the project skill at `.claude/skills/impresari-context/SKILL.md`; its model-directed smoke is not a repeatability guarantee. L3 is independently admitted through two explicit preview/apply deliveries using a zero-tool, no-session, safe-mode print process with exact prompt replay and without installing hooks. |
| Cursor | L1/L2 | L3 through rules and supported configuration | L1 is admitted only for Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64: explicit project scope, malformed-client rejection, native enable/list-tools/disable, bounded guarded Agent-mode lifecycle, packet equivalence, and source immutability are recorded. L2 is separately admitted for the opt-in `.cursor/rules/impresari-context.mdc` v2 rule through an isolated Agent-mode smoke that installs, validates, applies, and exactly removes the owned rule. Conversational rule selection remains non-deterministic; user-owned IDE enablement remains explicit. |
| GitHub Copilot CLI | L1/L2/L3/L4 | Maintain exact scopes | L1 is admitted only for Copilot CLI `1.0.80` on macOS aarch64: explicit project scope, malformed-client rejection, isolated native `list/get` discovery after exact workspace trust, bounded prompt-mode packet equivalence, and exact owned-entry/trust removal are recorded. L2 is recorded for the opt-in v3 instruction. L3 is independently admitted through two explicit zero-tool programmatic-prompt deliveries with immutable source and bounded receipts. L4 is limited to an explicit source-free check of that exact v3 artifact and recorded scope; it performs no client discovery or repair. Copilot `1.0.81` remains excluded. |
| VS Code Copilot extension host | L1/L2 | L3 where native surfaces permit | L1 and L2 are admitted only for VS Code `1.134.0` on macOS arm64. L1 records strict workspace `.vscode/mcp.json` configuration, explicit user trust/start, visible discovery, bounded lifecycle, source immutability, and exact removal. L2 separately records the active v3 instruction, successful same-session packet build/resolve, Copilot-compatible flat tool schema, source immutability, exact-owned cleanup, and hosted CI. It is distinct from the Copilot CLI; the root `.mcp.json` Agent Host surface remains generic/unadmitted. Conversational tool selection remains non-deterministic. |
| Gemini CLI | L0 | No further depth planned | Preserve legacy kit; reassess only for stable Antigravity successor. |

## Deliberate Differences From Competitors

Impresari adopts native setup, guidance, health, and context delivery—but not
hidden installers, unbounded automatic context injection, provider proxying,
shell-output rewriting, persistent-memory promotion, or agent orchestration.
Every external configuration change, guidance artifact, or lifecycle integration
requires explicit user approval, dry-run preview, an ownership marker, narrow
removal, and source-free diagnostics.
