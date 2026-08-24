# Compatibility Matrix

- Version: 1.0
- Published: 2026-08-23
- Status: Phase 1 configuration-evidence update
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
| Strict JSON configuration (`package.json`, `deno.json`, `composer.json`, `manifest.json`) | Yes | Yes | Yes | A strict JSON gate precedes the pinned JSON Tree-sitter grammar and emits static configuration-key and nesting facts only. Comments, arbitrary JSON data, JSON Schema, interpolation, loaders, and runtime configuration semantics are unsupported. |
| JSONC configuration (all `.jsonc` files; `tsconfig.json`, `jsconfig.json`, `devcontainer.json`, and selected `.vscode/*.json`) | Yes | Yes | Yes | Pinned JSON grammar emits decoded object-key and nesting facts only. Comments are syntax only; no editor, compiler, container, include, interpolation, runtime, or configuration-to-code semantics are claimed. |
| TOML configuration (`.toml`) | Yes | Yes | Yes | Pinned TOML grammar emits raw syntax-derived key, table, table-array, and nesting facts only. It does not evaluate values, includes, interpolation, package resolution, toolchains, build scripts, or runtime behavior. Malformed TOML emits no facts. |
| YAML configuration (`.yaml`, `.yml`) | Yes | Yes | Yes | Pinned YAML grammar emits only raw direct-scalar mapping keys and syntactic containment. Aliases, anchors, tags, merge behavior, directives, scalar values, sequences, schemas, and consumer/runtime semantics are unsupported. Syntax-malformed YAML emits no facts. |
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
| Codex | Generic local MCP | A project-scoped, non-mutating configuration kit and read-only TOML validator are checked against Codex CLI `0.149.0-alpha.4.1` on macOS aarch64. A deterministic App Server rehearsal directly calls the local MCP tool lifecycle and verifies packet equivalence; see the [Phase 1 kit record](../verification/phase-1-codex-connection-kit.md). | Not first-class: trusted-project clean-install/configuration-parser evidence, wider platform/version coverage, and entry-removal behavior still need evidence. |
| Claude Code | Generic local MCP | A non-mutating local-scope command and read-only JSON validator preserve the fixed local-stdio contract. Claude Code CLI `2.1.241` on macOS aarch64 completed an isolated temporary-config MCP lifecycle with no persistent registration; see the [Phase 1 kit record](../verification/phase-1-claude-code-connection-kit.md). | Not first-class: the conversational model-directed lifecycle is not deterministic, and configuration-parser, broader platform/version, packet-equivalence, and entry-removal evidence remain. |
| Cursor | Generic local MCP | A non-mutating project/global configuration guide and read-only validator accept Cursor's documented type-less stdio entry while rejecting environment forwarding. Signed-in Cursor Agent CLI `3.17.8` on macOS aarch64 discovered an isolated temporary project entry without enabling it; see the [Phase 1 kit record](../verification/phase-1-cursor-connection-kit.md). | Not first-class: user-approved MCP use, tool lifecycle and packet evidence, malformed-configuration behavior, broader platform/version coverage, removal, and source-immutability evidence remain. |
| Gemini CLI | Generic local MCP | A project `.gemini/settings.json` guide and read-only validator require an absolute local binary, fixed arguments, `trust: false`, and exactly the four released MCP tools. Gemini CLI `0.56.0` was authenticated on macOS aarch64. | Its normal client startup was rejected by the current free-tier service as unsupported; no lifecycle, packet, removal, or platform/version evidence exists. |
| GitHub Copilot CLI | Generic local MCP | A project `.mcp.json` guide and read-only validator require the local transport, fixed arguments, and exactly the four released MCP tools. Copilot CLI `1.0.80` on macOS aarch64 completed one isolated, model-directed temporary-config `context_session_open` call without persistent configuration; see the [Phase 2 kit record](../verification/phase-2-copilot-cli-connection-kit.md). | Not first-class: the conversational lifecycle is not deterministic, and tool sequence, packet equivalence, configuration-parser, removal, and broader platform/version evidence remain. |
| VS Code Copilot | Generic local MCP | A workspace `.vscode/mcp.json` guide and read-only validator require the fixed local command and arguments. | No VS Code extension-host/Agent Host, user-approval, lifecycle, packet, removal, or platform/version evidence yet. |
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
