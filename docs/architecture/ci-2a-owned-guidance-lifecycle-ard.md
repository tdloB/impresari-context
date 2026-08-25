# CI-2a Owned Native-Guidance Lifecycle — Architecture Requirements and Design

- Status: Approved for implementation
- Date: 2026-08-25
- Governing product record: [CI-2a PRD](../product/client-integration-l2-owned-guidance-lifecycle-prd.md)
- Governing decision: [ADR-0045](../decisions/0045-owned-native-guidance-artifact-lifecycle.md)

## Target mapping

| Client | Fixed target relative to project root |
| --- | --- |
| Codex | `AGENTS.md` |
| Claude Code | `.claude/skills/impresari-context/SKILL.md` |
| Cursor | `.cursor/rules/impresari-context.mdc` |
| GitHub Copilot | `.github/instructions/impresari-context.instructions.md` |

## State machine

```text
absent + existing non-symlink parent --preview--> preview_ready (no write)
                                         |
                                         +--explicit --apply--> owned
owned --inspect/validate--> owned
owned --remove preview--> preview_ready (no write)
owned --explicit --apply remove--> absent
any existing non-exact, symlinked, malformed, or oversized target --> reject, no write
```

## Invariants

1. The root is canonicalized once and is a non-symlink directory. The derived
   target must remain beneath it; its immediate parent must already exist and
   be a non-symlink directory.
2. The only accepted owned state is exact equality to a compiled released
   template (including its ownership marker), bounded to 16 KiB and valid UTF-8.
3. Render, inspect, validate, and all previews are read-only. Apply uses a
   same-directory atomic create or removal after revalidation.
4. No operation parses, reads, changes, or preserves unrelated instruction
   content; an existing target always rejects rather than being modified.
5. Receipts contain client, fixed relative target, ownership identity, content
   digest, state, planned effect, and write status—never project source text.

## Verification

- Every target has deterministic render and full round-trip coverage.
- Fixtures prove parent/file symlink rejection, missing-parent rejection,
  unowned/conflicting content rejection, bounded-size rejection, no-write
  preview/rejection, exact removal, and a source file untouched across the
  sequence.
- Template bytes are checked against the released template contract so the
  CLI cannot silently drift from CI-2's reviewed guidance.
