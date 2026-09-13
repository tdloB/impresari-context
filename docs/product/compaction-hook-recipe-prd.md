# Compaction Hook Recipe PRD

## Document Control

- PRD ID/version: IC-CHR-159 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-13.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Compaction Hook Recipe ARD](../architecture/compaction-hook-recipe-ard.md).
- Governing decision:
  [ADR-0159](../decisions/0159-offer-a-hook-recipe-for-evidence-lost-to-compaction.md).
- Follows: [ADR-0154](../decisions/0154-look-through-local-variables-when-building-a-map.md).

## Problem

After Claude Code compacts a conversation, Impresari evidence the model was
given may be gone from its context, and nothing says so.

## Product Outcome

A user who wants it can add one hook, by hand and to one project, that reminds
the model after each compaction to fetch Impresari evidence again before
relying on it. The hook holds no authority.

## Functional Requirements

1. A `SessionStart` hook, matched to `compact`, prints a fixed note and exits 0.
2. The script reads no repository file, opens no network connection, writes
   nothing, and changes no setting.
3. The recipe ships as templates with a published SHA-256 and manual install
   and removal steps for the project's `.claude/settings.json` only.
4. Impresari performs no installation.

## Acceptance Criteria

- The gate check fails when the checksum does not match, when an event other
  than an add-context event is used, when the command leaves the project's
  `.claude/hooks`, when the timeout exceeds 30 seconds, or when the script
  holds blocking output, a non-zero exit, a URL, or another program.
- The full repository gate passes.

## Non-Goals

- Hooks on every prompt, or any hook that reads the repository.
- Installing, enabling, or validating hooks from the CLI.
- Hooks for clients other than Claude Code.
