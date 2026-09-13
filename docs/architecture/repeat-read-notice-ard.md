# Repeat Read Notice — Architecture Requirements and Design

- ARD ID/version: IC-RRN-ARD-158 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-12.
- Governing PRD: [IC-RRN-158](../product/repeat-read-notice-prd.md).
- Decision: [ADR-0158](../decisions/0158-tell-a-session-it-already-holds-an-unchanged-read.md).

## Where the memory lives

`context-mcp`'s `read_memory` module holds a `ReadMemory`: for each session, a
map from `(display_path, requested_symbol)` to the identity of the answer last
delivered and the request that delivered it. `McpServer` owns one, and
`context_session_close` forgets the session's entry.

## Identity

`json_contract_identity("read-substitution-answer", …)` over the path, the
symbol, each span's `(start_byte, end_byte, content_sha256)`, `omitted_spans`
and `unknowns`. A span's hash already commits to its text, so the text is not
hashed again and is never stored.

## Answering an offer

`context_read_substitute` computes the substitution exactly as before, then
hands it to `ReadMemory::answer`:

```text
repeat not set, same key, same identity -> read-substitution-repeat notice
anything else                            -> the full answer; remember it
```

The notice carries `unchanged_since_request`, `answer_identity`,
`withheld_bytes` (the answer's `returned_bytes`), `whole_file_bytes`,
`to_receive_it_again`, and `authority_added: false`. The tool schema gains an
optional boolean `repeat`.

A session remembers at most 4,096 answers. Past the bound, a new key is not
recorded and is answered in full every time; replacing an existing key's answer
is still allowed.

## Verification

- `a_repeated_unchanged_answer_names_the_request_that_delivered_it`.
- `a_repeat_request_a_changed_source_or_another_session_gets_the_full_answer`.
- `a_closed_session_is_forgotten_and_memory_stops_at_its_bound`.
