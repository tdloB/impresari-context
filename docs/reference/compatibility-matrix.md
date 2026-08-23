# Compatibility Matrix

- Version: 1.0
- Published: 2026-08-23
- Status: Phase 2 language-admission update
- Evidence sources: [ADR-0004](../decisions/0004-source-language-and-parser-strategy.md),
  [ADR-0010](../decisions/0010-structural-worker-protocol-and-isolation.md),
  [dependency policy](../dependency-policy.md), and the released
  `context-engine` structural-language inventory.

This matrix describes shipped capabilities, not aspirations. A language label
on an evaluation fixture is not a structural-support claim.

The matching machine-readable release artifact is
[`compatibility-manifest-v1.json`](compatibility-manifest-v1.json).

## Evidence levels

| Level | Meaning |
| --- | --- |
| Discovery | Eligible regular files can be fingerprinted and represented as metadata. |
| Lexical evidence | UTF-8 files can participate in exact path, filename, literal, and lexical retrieval with recoverable evidence. |
| Structural evidence | The isolated project-owned worker and resolver can emit snapshot-bound structural facts with source provenance and explicit unresolved or truncated states. |
| Unsupported or partial | The engine reports an explicit limitation, exclusion, or unsupported state; it does not imply semantic analysis. |

Discovery and lexical evidence remain subject to the workspace policy, encoding,
file-size, generated-file, and exclusion rules. Invalid UTF-8 and binary files
may be discoverable as metadata but do not receive text evidence.

## Language capability matrix

| Language or file family | Discovery | Lexical evidence when UTF-8 eligible | Structural evidence | Notes |
| --- | --- | --- | --- | --- |
| TypeScript (`.ts`) | Yes | Yes | Yes | Pinned TypeScript Tree-sitter grammar and project-owned resolver. |
| TSX (`.tsx`) | Yes | Yes | Yes | Pinned TSX grammar and project-owned resolver. |
| JavaScript (`.js`, `.mjs`, `.cjs`) | Yes | Yes | Yes | Pinned JavaScript grammar and project-owned resolver. |
| JSX (`.jsx`) | Yes | Yes | Yes | Pinned JSX grammar and project-owned resolver. |
| Python (`.py`) | Yes | Yes | Yes | Pinned Python Tree-sitter grammar and project-owned syntax resolver. Python imports, declarations, containment, calls, and references are syntax-derived only; no interpreter, environment, package, or runtime resolution is performed. |
| Go (`.go`) | Yes | Yes | Yes | Pinned Go Tree-sitter grammar and project-owned syntax resolver. Functions, methods, type specifications, imports, direct calls, and references are syntax-derived only; no Go toolchain, module cache, package, or runtime resolution is performed. |
| Rust (`.rs`) | Yes | Yes | Yes | Pinned Rust Tree-sitter grammar and project-owned syntax resolver. Structs, enums, unions, traits, named functions, `use` declarations, direct calls, and references are syntax-derived only; no Cargo, compiler, crate graph, macro expansion, build script, feature, package, or runtime resolution is performed. |
| Strict JSON configuration (`package.json`, `deno.json`, `composer.json`, `manifest.json`) | Yes | Yes | Yes | Pinned JSON Tree-sitter grammar emits static configuration-key and nesting facts only. Arbitrary JSON data, JSONC, JSON Schema, interpolation, loaders, and runtime configuration semantics are unsupported. |
| Swift | Yes | Yes | No | Lexical evidence only. |
| Kotlin | Yes | Yes | No | Lexical evidence only. |
| Other eligible UTF-8 regular files | Yes | Yes | No | Language labels are filtering hints, not semantic claims. |
| Binary, invalid UTF-8, excluded, generated, or oversized files | Metadata only when safely discoverable | No | No | Reported as explicit partial or unsupported states. |

Structural facts are conservative syntax-derived facts, not compiler, runtime,
or language-server semantics. The worker never executes repository code or
loads repository configuration.

## Client compatibility matrix

| Client or surface | Classification | Supported connection today | Limitation |
| --- | --- | --- | --- |
| Any client that can start a configured local stdio MCP child process | Generic local MCP | Launch `impresari-context-mcp` with fixed workspace, cache, consumer, role, and operation-time arguments. | No named client conformance or maintained configuration kit is claimed. |
| Codex | Generic local MCP | A limited live local-stdio lifecycle check has passed against Codex CLI `0.149.0-alpha.4.1`; see the [conformance record](../verification/phase-0-codex-local-mcp-conformance.md). | Not first-class: the versioned kit, supported-platform scope, packet-corpus equivalence, malformed-config coverage, and safe-removal evidence remain incomplete. |
| Claude Code | Generic local MCP | May use the documented local stdio MCP process where the client can preserve the fixed launch contract. | Not first-class until a versioned kit and end-to-end conformance evidence are released. |
| Cursor | Generic local MCP | May use the documented local stdio MCP process where the client can preserve the fixed launch contract. | Not first-class until a versioned kit and end-to-end conformance evidence are released. |
| HTTP, remote MCP, daemon, or multi-client service consumers | Unsupported | None. | Impresari Context provides no network listener or remote transport. |

**Classification meanings:**

- **First-class** requires a maintained versioned connection kit and
  client-specific end-to-end conformance evidence.
- **Generic local MCP** means a client may be technically capable of launching
  the local stdio process; it is not a promise of a maintained integration.
- **Experimental** is reserved for a named kit with documented limitations and
  no stability promise. No experimental kits are published in this baseline.
- **Unsupported** means no approved connection path safely preserves the
  required authority contract.

## Authority contract for every local MCP launch

The client must provide fixed launch-time values for `--workspace`, `--cache`,
`--consumer-id`, and `--role`. MCP tool input and repository content cannot
change them. The local process records its startup time when `--occurred-at` is
omitted; the optional flag is for deterministic rehearsals. The workspace and
cache must be separate; the transport is local stdio only; and the process
neither executes repository code nor gains network, source-write, approval, or
orchestration authority.

See the [local MCP interface reference](interfaces.md) for the command and
protocol contract, and [ADR-0018](../decisions/0018-first-class-client-integration-and-compatibility-contract.md)
for the admission requirements for a first-class kit.
