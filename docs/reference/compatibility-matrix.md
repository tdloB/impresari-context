# Compatibility Matrix

- Version: 1.3
- Published: 2026-08-27
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
| C (`.c`, `.h`) | Yes | Yes | Yes | Pinned C grammar emits named declarations, literal includes, direct calls, references, and containment. No preprocessing, macro expansion, compiler, ABI, build, linker, or runtime semantics are claimed. |
| C++ (`.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, `.hxx`) | Yes | Yes | Yes | Pinned C++ grammar emits named declarations, literal includes, direct calls, references, and containment. Ambiguous `.h` remains C. No preprocessing, templates, overload resolution, compiler, build, linker, or runtime semantics are claimed. |
| Ruby (`.rb`) | Yes | Yes | Yes | Pinned Ruby grammar emits modules, classes, methods, literal requires, receiver-free direct calls, references, and containment. No interpreter, Bundler, Rails, metaprogram, autoload, monkey-patch, dispatch, or runtime semantics are claimed. |
| PHP (`.php`) | Yes | Yes | Yes | Pinned PHP grammar emits namespaces, named declarations, literal includes/requires, direct named calls, references, and containment. No interpreter, Composer, framework, autoload, extension, dispatch, or runtime semantics are claimed. |
| Swift (`.swift`) | Yes | Yes | Yes | Pinned Swift grammar emits named declarations, direct imports and receiver-free calls, references, and containment. No SwiftPM, Xcode, macro/plugin, compiler, type, build, signing, bridging, or runtime semantics are claimed. |
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
| Other eligible UTF-8 regular files | Yes | Yes | No | Language labels are filtering hints, not semantic claims. |
| Binary, invalid UTF-8, excluded, generated, or oversized files | Metadata only when safely discoverable | No | No | Reported as explicit partial or unsupported states. |

Structural facts are conservative syntax-derived facts, not compiler, runtime,
or language-server semantics. The worker never executes repository code or
loads repository configuration.

## Hostile-repository admission capability matrix

These are library-level, authority-neutral capabilities on the default branch;
they are not present in the published `v0.1.0` artifacts unless a later release
explicitly includes them.

| Capability | Status | Supported scope | Explicit limitation |
| --- | --- | --- | --- |
| HRA-1 artifact inventory | Implemented | Bounded, read-only, snapshot-bound static classification with explicit exclusions on all Tier A hosts | Classification is not malware detection; archives and hostile formats are not deeply parsed. |
| HRA-2 execution-surface observations | Implemented | Exact npm lifecycle keys in strict `package.json` and exact `privileged: true` in the admitted canonical Compose layout | Values are not interpreted; unsupported JSON/YAML syntax is explicit; intent and runtime behavior are not inferred. |
| HRA-3 coverage and assessment | Implemented | Deterministic required-analysis planning, ADR-0013-normalized synthetic result intake, and immutable assessment assembly | No analyzer is discovered or run. Missing mandatory analysis remains incomplete; zero findings never means safe. |
| HRA-4 reference policy evaluation | Implemented | Pure deterministic four-state evaluation of exact immutable assessment, coverage, findings, and policy | No filesystem, process, network, model, credential, exception, approval, or ordinary-host authority. Eligibility names only one future quarantine profile. |
| ADR-0074 isolated analyzer runner | IAR-1A application supervisor | Closed protocols/profiles; private content-addressed synthetic staging; pinned short-lived worker; bounded transport/time/output; input rehash; direct-child reap; cleanup | Application-enforced only; IAR-1B remains pending. No OS/VM sandbox, verified network/descendant containment, ClamAV, YARA, reputation provider, analyzer discovery, signature lifecycle, or real analyzer execution. |
| ADR-0074 macOS IAR-1B candidate | Partial feasibility only | Ad hoc signed App Sandbox host with private App Sandbox XPC service; bounded synthetic IPC; absent network entitlement; external-file, credential-canary, unrelated-process, and live-loopback network denials on macOS `26.5.1` arm64 | Not production-admitted and not a real analyzer backend. Device denial, resource/process-tree limits, fault timeout, complete OS-managed container cleanup, Developer ID/notarization, Homebrew packaging, update compatibility, and multi-host evidence remain unverified. |
| ADR-0075 disposable quarantine runner | Not implemented | None | No VM/container provisioning, repository execution, behavior observation, or destruction proof exists. |

See the
[Step 1 limitations statement](../security/hostile-repository-admission-limitations.md)
before interpreting any inventory, finding, coverage, assessment, or decision.

## Client compatibility matrix

| Client or surface | Classification | Supported connection today | Limitation |
| --- | --- | --- | --- |
| Any client that can start a configured local stdio MCP child process | Generic local MCP | Launch `impresari-context-mcp` with fixed workspace, cache, consumer, role, and operation-time arguments. | No named client conformance or maintained configuration kit is claimed. |
| Codex | First-class L1; recorded-scope L2 guidance; recorded-scope L3 guided delivery | The L1 user-home MCP kit and L2 `AGENTS.md` guidance retain their separate records. L3 uses a separately inspected preview plus explicit `--apply`, a dedicated operator-authenticated Codex home, complete initialization and auth preflight, and an ephemeral read-only/no-network App Server thread. Two independent deliveries completed with immutable source, clean runtimes, and no added authority; see the [Phase 1 kit record](../verification/phase-1-codex-connection-kit.md), [L2 guidance record](../verification/phase-2-codex-native-guidance.md), and [CI-3b record](../verification/ci-3b-codex-guided-delivery.md). | L1/L2 remain limited to Codex CLI `0.149.0-alpha.4.1` on macOS aarch64. L3 is independently limited to App Server `0.150.0-alpha.8` on the recorded macOS arm64 platform and exact authenticated-home/protocol boundary. No other version, platform, automatic injection, or conversational repeatability is claimed. |
| Claude Code | First-class L1; recorded-scope L2 guidance | An explicit local-scope command and read-only JSON validator preserve the fixed local-stdio contract. Claude Code CLI `2.1.241` on macOS aarch64 rejected malformed strict temporary configuration, completed an isolated temporary-config lifecycle with direct packet equivalence, and completed native `claude mcp add/get/remove --scope local` against a disposable Claude home. Its owned project skill also completed an isolated L2 packet smoke. See the [Phase 1 kit record](../verification/phase-1-claude-code-connection-kit.md) and [L2 guidance record](../verification/phase-2-claude-code-native-guidance.md). | Supported only for Claude Code CLI `2.1.241` on macOS aarch64. The model-directed lifecycles are live smoke evidence, not deterministic prompt-repeatability. The native rehearsals do not mutate a default MCP configuration and remove only their named temporary entries. Revalidate on an upstream client/configuration change. |
| Cursor | First-class L1; recorded-scope L2 guidance | An explicit project `.cursor/mcp.json` kit and validator preserve the fixed local-stdio contract. Cursor Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64 rejected malformed configuration, completed native enable/list-tools/disable, and completed a guarded Agent-mode four-tool lifecycle with direct packet equivalence in a disposable project. Its opt-in project rule v2 also completed an isolated native-guidance smoke with exact removal. See the [Phase 1 kit record](../verification/phase-1-cursor-connection-kit.md) and [L2 guidance record](../verification/phase-2-cursor-native-guidance.md). | Supported only for the recorded Cursor CLI/macOS aarch64 scope. Ask mode blocks dynamic MCP calls; the recorded Agent-mode smoke used a test-only project permission file allowing only the four named Impresari MCP tools and denying shell/file/web actions. Conversational rule selection remains non-deterministic. Revalidate on an upstream client/configuration/approval/stream change. |
| Gemini CLI | Generic local MCP | A project `.gemini/settings.json` guide and read-only validator require an absolute local binary, fixed arguments, `trust: false`, and the minimal four-tool session/packet allowlist. Gemini CLI `0.56.0` was authenticated on macOS aarch64. | Its normal client startup was rejected by the current free-tier service as unsupported; no lifecycle, packet, removal, or platform/version evidence exists. |
| GitHub Copilot CLI | First-class L1; recorded-scope L2 guidance, L3 guided delivery, and L4 health | The existing L1/L2/L4 records remain separate. CI-3c additionally completed two explicit preview/apply deliveries through Copilot CLI `1.0.80` with exact packet binding, isolated authentication, zero tool requests or executions, immutable source, clean disposable runtimes, and no added authority. See the [Phase 2 kit record](../verification/phase-2-copilot-cli-connection-kit.md), [L2 guidance record](../verification/phase-2-copilot-cli-native-guidance.md), [CI-3c record](../verification/ci-3c-copilot-cli-guided-delivery.md), and [CI-4 record](../verification/ci-4-copilot-cli-lifecycle-maintenance.md). | Supported only for Copilot CLI `1.0.80` on macOS aarch64 within each recorded boundary. Copilot `1.0.81` is not admitted for L3 because its model request retained built-in tool schemas despite exclusion flags. Revalidate every scope on an upstream client, configuration, authentication, permission, or protocol change. |
| VS Code Copilot extension host | First-class L1; recorded-scope L2 guidance | A workspace `.vscode/mcp.json` kit and validator require the fixed local `stdio` contract, reject sandbox and authority-expanding fields, and support preview, explicit install, inspection, and exact owned removal. VS Code `1.134.0` on macOS arm64 visibly discovered the server, started it under operator-controlled trust, and invoked bounded session open/close in a disposable workspace for L1. Its owned v3 instruction then separately completed an Impresari-only open/build/resolve/close lifecycle with one complete exact-source packet, zero omissions, source immutability, and exact-owned cleanup; see the [CI-1 record](../verification/ci-1-vscode-copilot-admission.md) and [CI-2 record](../verification/ci-2-vscode-copilot-native-guidance.md). | Supported only for VS Code `1.134.0` on macOS arm64. The L2 result is live conversational evidence, not deterministic prompt-repeatability or L3 delivery. The portable Agent Host root `.mcp.json` surface remains generic and unadmitted. Revalidate on an upstream VS Code, Copilot, MCP schema, approval, configuration, or platform change. |
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
