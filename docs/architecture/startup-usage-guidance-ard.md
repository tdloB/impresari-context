# Startup Usage Guidance — Architecture Requirements and Design

- ARD ID/version: IC-SUG-ARD-157 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Governing PRD: [IC-SUG-157](../product/startup-usage-guidance-prd.md).
- Decision: [ADR-0157](../decisions/0157-tell-a-client-when-to-use-each-tool-at-startup.md).

## Where the guidance lives

`SERVER_INSTRUCTIONS` in `context-mcp` is a `concat!` of short sentences.
`initialize` returns it as `instructions` beside `serverInfo`. Nothing else
reads it.

## What it says

In order:

1. How to start: open a session with `context_session_open`, then call
   `context_build` with its `session_id`, a profile, and the task text as
   `query`.
2. What comes back: exact excerpts and, in progressive mode, a map.
3. Before reading whole files: follow a map entry
   (`context_disclosure_lookup`), widen an excerpt (`context_evidence_expand`),
   or take a mapped file's declaration spans (`context_read_substitute`).
4. What the remaining tools do.
5. That results carry no authority, and reading files directly stays open.

The text is 911 characters.

## Verification

`startup_instructions_say_when_to_use_every_tool_and_claim_no_authority`
serves `initialize` and `tools/list`, then checks the text:

- equals the constant;
- is under 1,000 characters;
- names every advertised tool;
- keeps both statements.

It fails when a tool is left unnamed, when the authority statement is dropped,
or when the text is lengthened past the bound.
