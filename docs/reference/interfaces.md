# CLI and local MCP interface reference

This document describes the supported external interfaces for the current
release candidate. Both adapters call the same local engine. They read only the
workspace root explicitly authorized at launch and write only to the explicit
cache or export location.

## Compatibility and output contracts

- The CLI emits one versioned JSON value to stdout on success or failure.
  `--human` adds only a concise diagnostic to stderr.
- Public response objects use the bundled closed JSON Schema 2020-12 contracts
  in [`schemas/v1`](../../schemas/v1/) and its
  [registry](../../schemas/v1/registry.json).
- Schema contract version `1.0.0` is accepted. An incompatible packet version
  fails visibly rather than being partially interpreted.
- MCP uses protocol revision `2025-11-25`. The package version is returned in
  `serverInfo.version`; another protocol revision is rejected at initialization.

## CLI

Run `impresari-context --help`, or during development:

```text
cargo run -p context-cli -- --help
```

Global options may precede a command:

| Option | Meaning |
| --- | --- |
| `--human` | Add a source-free human diagnostic to stderr. |
| `--at <UTC>` | Use an RFC 3339 operation time for deterministic automation. |
| `--cutoff <UTC>` | Set the explicit audit-retention cutoff. |
| `--id-seed <8-64 chars>` | Derive deterministic request and event identifiers. |

Every command returns its named schema-shaped JSON object on stdout. Invalid
input, denial, stale state, integrity failure, or resource exhaustion returns a
versioned `error-envelope` and exit code `1`. Failure to write the response
returns `74`; success returns `0`.

| Command | Inputs | Output |
| --- | --- | --- |
| `workspace open <root> <cache-root>` | Authorized workspace and isolated cache paths | `workspace-handle` |
| `snapshot build <root> <cache-root>` | Workspace and cache paths | `snapshot-status` |
| `snapshot status <root> <cache-root> <expected-snapshot>` | Expected snapshot identity | `snapshot-status` with freshness comparison |
| `search <root> <cache-root> <kind> <query>` | `exact_path`, `filename`, `literal`, or `lexical` | `search` result |
| `context build <root> <cache-root> <kind> <query> <purpose>` | Search strategy and policy purpose | `context-packet` |
| `structure build <root> <cache-root> <worker> <worker-sha256> <empty-dir>` | Exact parser, digest, and empty non-workspace directory | `structural-graph` |
| `structure query <root> <cache-root> <graph-json> <start-node> <edge-kinds\|all>` | Graph file and `declares`, `contains`, `imports`, `exports`, or `calls` | `structural-query` result |
| `evidence expand <root> <cache-root> <evidence-json> <before> <after> <max>` | Evidence record and unsigned byte bounds | Expanded evidence |
| `packet validate <root> <cache-root> <packet-json>` | Context packet file | `packet-validation` |
| `handoff export <root> <cache-root> <packet-json> <export-root> <filename>` | Packet and authorized destination | `handoff-export`; no overwrite |

The structural worker is never downloaded or discovered. The caller supplies
its executable and SHA-256 identity. Numeric work is constrained by the
engine's conservative default resource budget.

## Local MCP process

Launch the single-client child process with fixed authority:

```text
impresari-context-mcp \
  --workspace <workspace-root> \
  --cache <cache-root> \
  --consumer-id <consumer-id> \
  --role <policy-role> \
  --occurred-at <UTC-timestamp>
```

Transport is newline-delimited JSON-RPC 2.0 over stdin/stdout. Messages are
limited to 1 MiB and the process remembers at most 10,000 request identifiers.
Batch requests, duplicate identifiers, malformed JSON, unknown fields, and
out-of-order lifecycle calls fail closed. A client sends `initialize`, then the
initialized notification, before listing or calling tools.

### Tools

All input objects reject unknown fields.

| Tool | Request | Successful structured content |
| --- | --- | --- |
| `context_session_open` | `{"session_id":"..."}` | Session ID, `opened: true`, `authority_added: false` |
| `context_build` | IDs, purpose, RFC 3339 time, 1-8 plan steps, budget, optional session ID | Immutable packet, optional reference, false authority flags |
| `context_packet_resolve` | `{"session_id":"...","packet_id":"..."}` | Owning-session reference and immutable packet |
| `context_session_close` | `{"session_id":"..."}` | Session ID, `closed: true`, `authority_added: false` |

A plan step is
`{"kind":"exact_path|filename|literal|lexical","query":"..."}`. The budget
follows [`resource-budget.schema.json`](../../schemas/v1/resource-budget.schema.json):
decimal counters are strings, `unit_kind` is `utf8_bytes`, and `hard` is true.

JSON-RPC protocol and parameter failures use JSON-RPC error objects. Tool-level
engine, policy, session, and validation failures return `isError: true` with a
stable source-free message and `authority_added: false`. No failure falls back
to a broader filesystem read.

## Security boundary

Neither adapter grants orchestration, approval, model, network, process,
editing, or ambient filesystem authority. Repository content is untrusted data,
not executable instructions. MCP sessions and packet references exist only in
the child process and are scoped to the configured consumer. The launching host
remains responsible for process identity, operating-system isolation, and its
arguments.

Normative behavior is defined by
[ADR-0014](../decisions/0014-local-stdio-mcp-transport.md), the
[threat model](../security/threat-model.md), and the
[schema registry](../../schemas/v1/registry.json).
