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

## Current checkpoint — 2026-08-25

| Track | Current evidence-backed status | Next gate |
| --- | --- | --- |
| CI-1 managed connections | Shared render/inspect/validate/install/remove capability and deterministic fixture coverage exist. An explicit owned-update implementation is locally release-gated and awaiting hosted acceptance. All named clients remain L0 until their independent real-client/version/OS admission records are complete. | Merge the owned-update decision and implementation after hosted CI; complete only user-consented, disposable client lifecycle evidence where required. |
| CI-2 native guidance | PRD/ADR accepted. Versioned Codex, Claude Code, Cursor, and Copilot templates plus a static authority-boundary checker are implemented locally and awaiting hosted acceptance. | Merge template implementation; then admit managed install/remove and client/version/OS smoke evidence separately per client. |
| CI-3 guided delivery | PRD/ADR/ARD define an explicit consented delivery intent, deterministic packet equivalence, and no-delivery degradation. No client adapter is implemented or claimed. | Admit a client-neutral intent/receipt contract, then evaluate only stable official lifecycle surfaces one client at a time. |
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

Exit: L2 requires an opt-in install/validate/remove round-trip test.

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
| Codex | L0 | L1, then L2/L3 where native surfaces permit | Direct lifecycle evidence exists; full L1 admission is incomplete. |
| Claude Code | L0 | L1, then L2/L3 through native skills/lifecycle surfaces | Deep behavior is later consented delivery, not silent prompt injection. |
| Cursor | L0 | L1, then L2/L3 through rules and supported configuration | User-owned IDE enablement remains explicit. |
| GitHub Copilot | L0 | L1, then L2/L3 across CLI and VS Code separately | CLI and VS Code need distinct evidence. |
| Gemini CLI | L0 | No further depth planned | Preserve legacy kit; reassess only for stable Antigravity successor. |

## Deliberate Differences From Competitors

Impresari adopts native setup, guidance, health, and context delivery—but not
hidden installers, unbounded automatic context injection, provider proxying,
shell-output rewriting, persistent-memory promotion, or agent orchestration.
Every external configuration change, guidance artifact, or lifecycle integration
requires explicit user approval, dry-run preview, an ownership marker, narrow
removal, and source-free diagnostics.
