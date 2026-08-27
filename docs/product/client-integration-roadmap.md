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

## Current checkpoint — 2026-08-26

| Track | Current evidence-backed status | Next gate |
| --- | --- | --- |
| CI-1 managed connections | Shared render/inspect/validate/install/remove capability and deterministic fixture coverage exist. Codex, Claude Code, Cursor, and GitHub Copilot CLI L1 local evidence are complete for their recorded macOS/client-version scopes. VS Code Copilot now has a portable Agent Host `.mcp.json` strict-stdio candidate and disposable operator-evidence runner, but is still L0 pending live-client evidence. Copilot's CLI and VS Code surfaces remain separate claims. | Maintain the four admitted clients after upstream changes; complete only user-consented, disposable lifecycle evidence for the distinct VS Code Copilot surface. |
| CI-2 native guidance | Versioned Codex, Claude Code, Cursor, and Copilot templates; deterministic render/inspect/validate/install/remove lifecycle; static authority-boundary checks; and exact v1 legacy-removal support are implemented. Codex, Claude Code, Cursor, and GitHub Copilot CLI L2 are admitted for their recorded macOS/client-version scopes through isolated native-surface smokes. Codex has no separate deterministic instruction-source signal from its App Server. | Maintain all four admitted client records after upstream changes. Evaluate CI-3's client-neutral consent and receipt contract before considering any delivery adapter; VS Code Copilot remains a separate L1 client surface. |
| CI-3 guided delivery | CI-3a implements a strict client-neutral intent/receipt contract. CI-3b adds an experimental Codex-only preview/apply adapter for CLI `0.149.0-alpha.4.1` on macOS aarch64: exact preview rehydration, ephemeral read-only/no-network App Server session, authority denial, bounded receipt, and temporary-runtime cleanup. Its first isolated live record safely degraded on turn timeout, so no L3 client promotion is claimed. | Reassess only stable official lifecycle surfaces one client at a time. Codex needs a successful repeatable lifecycle record under its exact scope before L3 admission; every other client still needs its own consent, removal, redaction, degradation, source-immutability, and live lifecycle evidence. |
| CI-4 lifecycle maintenance | PRD/ADR/ARD define source-free on-demand compatibility checks. No health adapter is implemented or claimed. | Admit a source-free manifest/receipt contract only after one client has a stable L1/L2/L3 surface. |

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
| Codex | L1/L2 | L3 where native surfaces permit | L1 is admitted only for Codex CLI `0.149.0-alpha.4.1` on macOS aarch64: explicit user-home kit, malformed-client rejection, deterministic lifecycle, packet equivalence, and exact removal are recorded. L2 is separately admitted for a stand-alone project `AGENTS.md` v2 artifact through a disposable live CLI smoke. CI-3b provides an experimental explicit preview/apply App Server path at this exact version; the first no-authority isolated smoke timed out safely, so it is not an L3 admission or a repeatability claim. |
| Claude Code | L1/L2 | L3 through native skills/lifecycle surfaces | L1 is admitted only for Claude Code CLI `2.1.241` on macOS aarch64: explicit local scope, malformed-client rejection, native add/get/removal, bounded model-directed lifecycle, packet equivalence, and source immutability are recorded. L2 is separately admitted for the project skill at `.claude/skills/impresari-context/SKILL.md`; its model-directed smoke is not a repeatability guarantee. Deep behavior remains later consented delivery, not silent prompt injection. |
| Cursor | L1/L2 | L3 through rules and supported configuration | L1 is admitted only for Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64: explicit project scope, malformed-client rejection, native enable/list-tools/disable, bounded guarded Agent-mode lifecycle, packet equivalence, and source immutability are recorded. L2 is separately admitted for the opt-in `.cursor/rules/impresari-context.mdc` v2 rule through an isolated Agent-mode smoke that installs, validates, applies, and exactly removes the owned rule. Conversational rule selection remains non-deterministic; user-owned IDE enablement remains explicit. |
| GitHub Copilot CLI | L1/L2 | L3 where native surfaces permit | L1 is admitted only for Copilot CLI `1.0.80` on macOS aarch64: explicit project scope, malformed-client rejection, isolated native `list/get` discovery after exact workspace trust, bounded prompt-mode packet equivalence, and exact owned-entry/trust removal are recorded. L2 is separately admitted for the opt-in `.github/instructions/impresari-context.instructions.md` v2 instruction through an isolated Copilot CLI session with custom instructions enabled only for that session. Conversational tool selection remains non-deterministic. |
| VS Code Copilot | L0 | L1, then L2/L3 where native surfaces permit | It is distinct from the Copilot CLI and needs its own extension-host/Agent Host, approval, lifecycle, packet, removal, platform, and version evidence. |
| Gemini CLI | L0 | No further depth planned | Preserve legacy kit; reassess only for stable Antigravity successor. |

## Deliberate Differences From Competitors

Impresari adopts native setup, guidance, health, and context delivery—but not
hidden installers, unbounded automatic context injection, provider proxying,
shell-output rewriting, persistent-memory promotion, or agent orchestration.
Every external configuration change, guidance artifact, or lifecycle integration
requires explicit user approval, dry-run preview, an ownership marker, narrow
removal, and source-free diagnostics.
