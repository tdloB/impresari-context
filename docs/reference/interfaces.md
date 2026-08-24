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
- MCP prefers protocol revision `2025-11-25` and also accepts `2025-06-18` for
  client compatibility. On initialization it returns the supported revision
  requested by the client; another protocol revision is rejected. The package
  version is returned in `serverInfo.version`.

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
| `doctor inspect <root> <cache-root>` | Existing workspace and cache directories | `doctor-report` with metadata-only prerequisite checks |
| `doctor mcp <root> <cache-root>` | Existing workspace and separate cache directories | `doctor-report` with an in-process MCP initialization and tool-discovery check |
| `doctor codex-config <root> <cache-root> <config-toml>` | Existing workspace/cache and a Codex-format TOML config | `doctor-report` with a source-free fixed-stdio project-configuration check |
| `doctor cursor-config <root> <cache-root> <mcp-json>` | Existing workspace/cache and a Cursor-format config | `doctor-report` with a source-free fixed-stdio configuration check |
| `doctor claude-config <root> <cache-root> <mcp-json>` | Existing workspace/cache and a Claude-format config | `doctor-report` with a source-free fixed-stdio configuration check |

The structural worker is never downloaded or discovered. The caller supplies
its executable and SHA-256 identity. Numeric work is constrained by the
engine's conservative default resource budget.

### Doctor (Phase 0 baseline)

`doctor inspect` is intentionally metadata-only. It resolves the two existing
directories, verifies that the cache is separate from and not nested inside the
workspace, and checks the published Tier A platform shape. It writes neither
directory, reads no source file contents, runs no repository code, contacts no
network, and does not inspect or modify client configuration.

Its `doctor-report` is source-free: it contains stable check identifiers,
statuses, remediation classes, and explicit limitations, but never absolute
paths, source excerpts, raw configuration, or secret values. A successful
baseline report has status `partial`, not `ready`, because MCP lifecycle,
structural-worker identity, and client-configuration validation are deferred to
later Phase 0 increments.

`doctor mcp` adds a bounded in-process JSON-RPC exchange against the shipped
MCP implementation. It opens and snapshots the explicitly selected workspace
into the explicit cache, completes `initialize` and `notifications/initialized`,
validates discovery of the four published tools, and compares a bounded packet
from the direct engine with the corresponding MCP packet. It does not mutate
the source workspace, launch an arbitrary external binary, parse a third-party
client configuration, or prove an end-to-end client integration. It therefore
also reports `partial` on success.

`doctor codex-config` parses a bounded TOML file, validates the exact
`[mcp_servers."impresari-context"]` local-stdio entry, requires a real absolute
binary path, canonical workspace/cache matches, `prompt` tool approval, and no
environment forwarding or remote server options. It does not launch Codex,
modify its configuration, display a path or configuration value, or claim the
client is first-class.

`doctor cursor-config` and `doctor claude-config` parse a bounded JSON file,
but report only a stable pass/fail category and remediation class. They validate
the `impresari-context` entry's local-stdio shape, fixed argument order,
consumer and role identifiers, and a cache value that is not a workspace child.
Cursor's documented type-less command/args stdio form and an explicit `stdio`
type are accepted. Both validators reject environment forwarding and unrelated
entry fields. They do not display, normalize, or rewrite the configuration;
launch either client; verify the referenced binary; or claim the client is
first-class.

## Local MCP process

Launch the single-client child process with fixed authority:

```text
impresari-context-mcp \
  --workspace <workspace-root> \
  --cache <cache-root> \
  --consumer-id <consumer-id> \
  --role <policy-role>
```

Transport is newline-delimited JSON-RPC 2.0 over stdin/stdout. Messages are
limited to 1 MiB and the process remembers at most 10,000 request identifiers.
Batch requests, duplicate identifiers, malformed JSON, unknown fields, and
out-of-order lifecycle calls fail closed. A client sends `initialize`, then the
initialized notification, before listing or calling tools.

Tool calls may include the standard MCP `_meta` object. Its contents are
validated only as an object and are ignored: it cannot change the fixed launch
authority, request semantics, resource budget, or server behavior.

The process records the local UTC startup time when `--occurred-at` is omitted.
The optional flag remains available only for deterministic tests and rehearsals.
Operation times on individual `context_build` requests remain explicit,
validated request values.

### Tools

All tool argument objects reject unknown fields. The enclosing `tools/call`
object accepts only `name`, `arguments`, and the inert optional `_meta` object.

| Tool | Request | Successful structured content |
| --- | --- | --- |
| `context_session_open` | `{"session_id":"..."}` | Session ID, `opened: true`, `authority_added: false` |
| `context_build` | IDs, purpose, RFC 3339 time, budget, optional session ID, plus either 1-8 explicit plan steps or a declared profile and query | Immutable packet, optional deterministic plan, optional reference, false authority flags |
| `context_packet_resolve` | `{"session_id":"...","packet_id":"..."}` | Owning-session reference and immutable packet |
| `context_session_close` | `{"session_id":"..."}` | Session ID, `closed: true`, `authority_added: false` |

A plan step is
`{"kind":"exact_path|filename|literal|lexical","query":"..."}`. The budget
follows [`resource-budget.schema.json`](../../schemas/v1/resource-budget.schema.json):
decimal counters are strings, `unit_kind` is `utf8_bytes`, and `hard` is true.

For a deterministic planner request, replace `steps` with one of the declared
profiles—`orientation`, `implementation`, `bug_investigation`,
`change_review`, `security_review`, `test_selection`, or
`configuration_change`—and a bounded `query`. The result includes an exact
plan identity, ordered reason-coded steps, evidence-class coverage, explicit
omissions, and the packet identity. This is rule-based retrieval selection;
it does not interpret prompts, call a model, execute code, or grant authority.

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
