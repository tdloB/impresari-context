# CI-2 Native Guidance Artifacts — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-2 PRD](../product/client-integration-l2-native-guidance-prd.md)
- Governing decision: [ADR-0041](../decisions/0041-native-agent-guidance-artifacts.md)

## Architectural objective

Offer client-native evidence-use guidance without mutating existing project
instructions or turning a static artifact into automatic context delivery.
Each artifact is a small, independently owned file whose content is versioned,
auditable, and removable without parsing or rewriting unrelated instructions.

## Artifact model

| Client | Owned project artifact | Authority boundary |
| --- | --- | --- |
| Codex | stand-alone `AGENTS.md` template | Never overwrite existing project instructions. |
| Claude Code | `.claude/skills/impresari-context/SKILL.md` | Skill text has no shell/dynamic-injection content. |
| Cursor | `.cursor/rules/impresari-context.mdc` | Rule is agent-requested, not always attached. |
| GitHub Copilot | `.github/instructions/impresari-context.instructions.md` | Static repository instruction only. |

An artifact may say how to request an already-configured local MCP packet. It
cannot configure, enable, approve, invoke, serialize, or deliver one.

## Invariants

1. Artifact content is static, bounded, versioned, and owned by its exact path
   and complete content; no append/merge into an unrelated instruction file is
   permitted.
2. It asks for an explicit profile and hard budget, and preserves visible packet
   identity, plan ID, reason codes, coverage, and omissions.
3. It forbids configuration/trust/approval changes, source mutation, execution,
   environment forwarding, networking, inferred runtime claims, and fabricated
   evidence.
4. Missing or unavailable MCP delivery degrades to ordinary analysis with an
   explicit limitation; it never falls back to hidden prompts or another tool.
5. Any future installer must preview, validate, explicitly apply, inspect, and
   remove only the exact owned artifact; it must reject pre-existing unowned
   content and symlink/ambiguous paths.

## Verification requirements

- A deterministic template checker asserts all required client artifacts,
  expected format metadata, bounded size, required evidence language, and
  absence of URL, shell, environment-forwarding, automatic-approval, trust, or
  MCP-install/enable content.
- Template changes run through repository policy and full release checks.
- A future artifact installer must additionally prove no-write preview,
  exact-owned install/validate/remove, source immutability, and unrelated-file
  preservation for every client path.

## Deliberate deferral

Client-specific live guidance smoke evidence and managed artifact installation
remain separate per-client admissions. CI-2 does not elevate any client’s L1
classification, and CI-3 delivery adapters cannot use an artifact as implicit
consent.
