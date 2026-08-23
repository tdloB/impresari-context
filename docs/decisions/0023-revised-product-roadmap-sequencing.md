# ADR-0023: Revised product-roadmap sequencing

- Status: Accepted
- Date: 2026-08-23
- Scope: Phase ownership and sequencing for language, client, planner, and
  enterprise expansion work

## Decision

Adopt the founder-approved revised product roadmap as the phase-sequencing
source of truth. Phase 0 owns truthful public capability and doctor contracts;
Phase 1 owns Python/configuration evidence plus Codex, Claude Code, and Cursor
admission; Phase 2 owns Rust/Go plus Gemini CLI, GitHub Copilot CLI, and VS
Code Copilot admission; Phase 3 owns the deterministic context planner; Phase
4 owns Java/Kotlin/C# and impact evidence; and Phase 5 owns demand-led language
admission.

ADR-0019 and ADR-0020 remain Phase 1 delivery records. ADR-0021 and ADR-0022
remain Phase 2 delivery records. Their language-level acceptance does not mark
the client portions of their parent phases complete.

## Consequences

- Rust is no longer described as Phase 3 work; it is a completed Phase 2
  language slice alongside Go.
- Planner work begins only after the Phase 1 and Phase 2 scope has real
  language and client depth, without turning Impresari Context into agent
  governance.
- Every future grammar or client still needs its own admission record and full
  evidence; this ADR supplies sequencing, not blanket technical approval.

## References

- [Revised Product Roadmap](../product/revised-product-roadmap.md)
- [Phase 0 language and client foundation PRD](../product/phase-0-language-and-client-foundation-prd.md)
- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0018: First-class client integration and compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
