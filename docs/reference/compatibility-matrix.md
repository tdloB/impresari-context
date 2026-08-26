# Compatibility Matrix

- Version: 1.2
- Published: 2026-08-26
- Status: recorded-scope client-admission update
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
| Java (`.java`) | Yes | Yes | Yes | Pinned Java Tree-sitter grammar and project-owned syntax resolver. Type and method declarations, non-static non-wildcard imports, direct unqualified method calls, and references are syntax-derived only; no Java compiler, classpath, package, module, build-tool, annotation, dependency, or runtime resolution is performed. |
| Kotlin (`.kt`, `.kts`) | Yes | Yes | Yes | Pinned Kotlin Tree-sitter grammar and project-owned syntax resolver. Named classes, objects, functions, and type aliases; non-wildcard/non-aliased imports; direct identifier calls; and references are syntax-derived only. No Kotlin compiler, Gradle, classpath, package, dependency, annotation, coroutine, extension-dispatch, or runtime resolution is performed. |
| C# (`.cs`) | Yes | Yes | Yes | Pinned C# Tree-sitter grammar and project-owned syntax resolver. Classes, records, structs, delegates, constructors, named methods, non-static/non-aliased using directives, direct identifier calls, and references are syntax-derived only. No .NET compiler, MSBuild, project, package, dependency, attribute, overload, dispatch, or runtime resolution is performed. |
| Scala (`.scala`) | Yes | Yes | Yes | Pinned Scala Tree-sitter grammar and project-owned syntax resolver. Classes, objects, traits, enums, named functions, direct imports, direct calls, and references are syntax-derived only. Wildcard, selector, and aliased imports; compiler, SBT, Mill, classpath, dependency, implicit, macro, extension-dispatch, and runtime semantics are unsupported. |
| Elixir (`.ex`, `.exs`) | Yes | Yes | Yes | Pinned Elixir Tree-sitter grammar and project-owned syntax resolver. Literal `defmodule`, direct function/macro declarations, direct `alias`/`import`/`require` forms, direct identifier calls, and references are syntax-derived only. Mix, Hex, BEAM, macro expansion, protocol dispatch, compile-time code, and runtime semantics are unsupported. |
| Clojure (`.clj`, `.cljs`, `.cljc`) | Yes | Yes | Yes | Pinned Clojure Tree-sitter grammar and project-owned syntax resolver. Direct literal `ns`, `def`, `defn`, `defmacro`, `defmulti`, and `defonce` forms plus direct list-head calls are syntax-derived only. Reader evaluation, syntax quoting, macro expansion, namespace/classpath, dependency, JVM/JS, and runtime semantics are unsupported. |
| Haskell (`.hs`, `.lhs`) | Yes | Yes | Yes | Pinned Haskell Tree-sitter grammar and project-owned syntax resolver. Named bindings and direct imports are syntax-derived only. GHC, Cabal, Stack, package, typeclass, type inference, Template Haskell, compiler, and runtime semantics are unsupported. |
| Go (`.go`) | Yes | Yes | Yes | Pinned Go Tree-sitter grammar and project-owned syntax resolver. Functions, methods, type specifications, imports, direct calls, and references are syntax-derived only; no Go toolchain, module cache, package, or runtime resolution is performed. |
| Rust (`.rs`) | Yes | Yes | Yes | Pinned Rust Tree-sitter grammar and project-owned syntax resolver. Structs, enums, unions, traits, named functions, `use` declarations, direct calls, and references are syntax-derived only; no Cargo, compiler, crate graph, macro expansion, build script, feature, package, or runtime resolution is performed. |
| Strict JSON configuration (`package.json`, `deno.json`, `composer.json`, `manifest.json`) | Yes | Yes | Yes | A strict JSON gate precedes the pinned JSON Tree-sitter grammar and emits static configuration-key and nesting facts only. Comments, arbitrary JSON data, JSON Schema, interpolation, loaders, and runtime configuration semantics are unsupported. |
| JSONC configuration (all `.jsonc` files; `tsconfig.json`, `jsconfig.json`, `devcontainer.json`, and selected `.vscode/*.json`) | Yes | Yes | Yes | Pinned JSON grammar emits decoded object-key and nesting facts only. Comments are syntax only; no editor, compiler, container, include, interpolation, runtime, or configuration-to-code semantics are claimed. |
| TOML configuration (`.toml`) | Yes | Yes | Yes | Pinned TOML grammar emits raw syntax-derived key, table, table-array, and nesting facts only. It does not evaluate values, includes, interpolation, package resolution, toolchains, build scripts, or runtime behavior. Malformed TOML emits no facts. |
| YAML configuration (`.yaml`, `.yml`) | Yes | Yes | Yes | Pinned YAML grammar emits only raw direct-scalar mapping keys and syntactic containment. Aliases, anchors, tags, merge behavior, directives, scalar values, sequences, schemas, and consumer/runtime semantics are unsupported. Syntax-malformed YAML emits no facts. |
| Swift | Yes | Yes | No | Lexical evidence only. |
| Other eligible UTF-8 regular files | Yes | Yes | No | Language labels are filtering hints, not semantic claims. |
| Binary, invalid UTF-8, excluded, generated, or oversized files | Metadata only when safely discoverable | No | No | Reported as explicit partial or unsupported states. |

Structural facts are conservative syntax-derived facts, not compiler, runtime,
or language-server semantics. The worker never executes repository code or
loads repository configuration.

## Client compatibility matrix

| Client or surface | Classification | Supported connection today | Limitation |
| --- | --- | --- | --- |
| Any client that can start a configured local stdio MCP child process | Generic local MCP | Launch `impresari-context-mcp` with fixed workspace, cache, consumer, role, and operation-time arguments. | No named client conformance or maintained configuration kit is claimed. |
| Codex | First-class L1; recorded-scope L2 guidance; experimental CI-3b delivery | An explicit user-level Codex-home TOML kit is validated, safely installed and removed in an isolated `CODEX_HOME`, rejected when malformed, and exercised through the deterministic App Server lifecycle with direct packet equivalence. Its stand-alone project `AGENTS.md` v2 guidance was also exercised by an isolated live Codex CLI smoke. CI-3b adds a separately inspected preview plus explicit `--apply` App Server path that starts only an ephemeral read-only/no-network thread and denies authority requests; see the [Phase 1 kit record](../verification/phase-1-codex-connection-kit.md), [L2 guidance record](../verification/phase-2-codex-native-guidance.md), and [CI-3b record](../verification/ci-3b-codex-guided-delivery.md). | Supported only for Codex CLI `0.149.0-alpha.4.1` on macOS aarch64. CI-3b's first isolated lifecycle safely timed out, so it is not L3 admission, successful delivery, deterministic project-instruction discovery, or conversational tool-call repeatability. Revalidate on an upstream client/configuration/protocol change. |
| Claude Code | First-class L1; recorded-scope L2 guidance | An explicit local-scope command and read-only JSON validator preserve the fixed local-stdio contract. Claude Code CLI `2.1.241` on macOS aarch64 rejected malformed strict temporary configuration, completed an isolated temporary-config lifecycle with direct packet equivalence, and completed native `claude mcp add/get/remove --scope local` against a disposable Claude home. Its owned project skill also completed an isolated L2 packet smoke. See the [Phase 1 kit record](../verification/phase-1-claude-code-connection-kit.md) and [L2 guidance record](../verification/phase-2-claude-code-native-guidance.md). | Supported only for Claude Code CLI `2.1.241` on macOS aarch64. The model-directed lifecycles are live smoke evidence, not deterministic prompt-repeatability. The native rehearsals do not mutate a default MCP configuration and remove only their named temporary entries. Revalidate on an upstream client/configuration change. |
| Cursor | First-class L1; recorded-scope L2 guidance | An explicit project `.cursor/mcp.json` kit and validator preserve the fixed local-stdio contract. Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64 rejected malformed configuration, completed native enable/list-tools/disable, and completed a guarded Agent-mode four-tool lifecycle with direct packet equivalence in a disposable project. Its opt-in project rule v2 also completed an isolated native-guidance smoke with exact removal. See the [Phase 1 kit record](../verification/phase-1-cursor-connection-kit.md) and [L2 guidance record](../verification/phase-2-cursor-native-guidance.md). | Supported only for the recorded Cursor CLI/macOS aarch64 scope. Ask mode blocks dynamic MCP calls; the recorded Agent-mode smoke used a test-only project permission file allowing only the four named Impresari MCP tools and denying shell/file/web actions. Conversational rule selection remains non-deterministic. Revalidate on an upstream client/configuration/approval/stream change. |
| Gemini CLI | Generic local MCP | A project `.gemini/settings.json` guide and read-only validator require an absolute local binary, fixed arguments, `trust: false`, and the minimal four-tool session/packet allowlist. Gemini CLI `0.56.0` was authenticated on macOS aarch64. | Its normal client startup was rejected by the current free-tier service as unsupported; no lifecycle, packet, removal, or platform/version evidence exists. |
| GitHub Copilot CLI | First-class L1; recorded-scope L2 guidance | An explicit project `.mcp.json` kit and validator preserve the fixed local-stdio contract. Copilot CLI `1.0.80` on macOS aarch64 rejected malformed configuration, completed isolated native project `list/get` discovery after an exact disposable workspace-trust entry, completed the bounded four-tool prompt lifecycle with direct packet equivalence, and removed only the owned project entry and temporary trust entry. Its opt-in repository instruction v2 also completed an isolated native-guidance smoke with custom instructions enabled only for that session. See the [Phase 2 kit record](../verification/phase-2-copilot-cli-connection-kit.md) and [L2 guidance record](../verification/phase-2-copilot-cli-native-guidance.md). | Supported only for Copilot CLI `1.0.80` on macOS aarch64. The prompt-mode lifecycle and custom-instruction use are live smoke evidence, not deterministic prompt-repeatability. The rehearsal uses an isolated `COPILOT_HOME`, no additional MCP configuration, and a four-tool-only client surface; it never changes a real Copilot home or source project. Revalidate on an upstream client/configuration/trust change. |
| VS Code Copilot | Generic local MCP | A workspace-root `.mcp.json` portable Agent Host guide and read-only validator require a strict `stdio` local command and fixed arguments. A disposable candidate-admission runner is available. | No recorded VS Code extension-host/Agent Host trust, discovery, tool, removal, or platform/version evidence yet; the candidate runner does not itself promote this client. |
| HTTP, remote MCP, daemon, or multi-client service consumers | Unsupported | None. | Impresari Context provides no network listener or remote transport. |

**Classification meanings:**

- **First-class** requires a maintained versioned connection kit and
  client-specific end-to-end conformance evidence.
- **Generic local MCP** means a client may be technically capable of launching
  the local stdio process; it is not a promise of a maintained integration.
- **Experimental** is a named, opt-in capability with a documented version/
  platform boundary and no stability or promotion promise. Codex CI-3b is the
  only published experimental client-delivery path.
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
