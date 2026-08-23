# ADR-0018: First-class client integration and compatibility contract

- Status: Accepted
- Date: 2026-08-23
- Scope: Public language/client capability claims, named local MCP client
  connections, and read-only connection diagnostics

## Context

Impresari Context has a local stdio MCP adapter that preserves fixed
workspace, cache, consumer, and role authority at process launch. A client
that can launch a compatible stdio process may be technically interoperable,
but protocol interoperability does not establish a maintained product
integration. It says nothing about configuration scope, launch arguments,
client upgrades, diagnostic behavior, packet equivalence, or whether the
client's capabilities can preserve Impresari Context's safety invariants.

The same ambiguity exists in language claims. Raw-file discovery and UTF-8
lexical evidence are broadly useful, while structural evidence is currently
available only for the released TypeScript/JavaScript parser family. Calling
all of those cases "supported" would overstate the available graph semantics.

The project needs a durable public compatibility vocabulary before it expands
languages or publishes named agent integrations.

## Decision

Adopt versioned public language and client compatibility matrices and use them
as release-gated product contracts.

### Language classifications

The language matrix distinguishes discovery, lexical evidence, structural
evidence, and unsupported/partial states. A structural row is published only
when its pinned parser/grammar, project-owned resolver, source-provenance
validation, security tests, and evaluation evidence are released together.

The current initial structural set is TypeScript, TSX, JavaScript, and JSX.
Other eligible UTF-8 files may receive lexical evidence but are not described
as structurally supported until they satisfy this record and ADR-0004.

### Client classifications

The client matrix uses four classifications:

- **First-class:** versioned connection kit plus maintained, client-specific
  end-to-end conformance evidence.
- **Generic local MCP:** protocol capability may permit connection, but no
  first-class product claim is made.
- **Experimental:** named kit has declared limitations and does not carry a
  stability promise.
- **Unsupported:** no approved connection path safely preserves the required
  authority contract.

Codex, Claude Code, and Cursor are the initial intended first-class targets.
They receive that label only after passing the required conformance suite; this
ADR does not itself claim that they are currently first-class.

### Connection-kit boundary

A connection kit is a versioned integration artifact outside the neutral
engine's semantics. It may provide documentation, configuration templates,
dry-run rendering, validation, and precise removal instructions. It must:

- retain the current local stdio transport and fixed launch-time authority;
- make project/user scope explicit;
- use an explicit installed binary or independently verified package identity;
- be opt-in and show proposed external-file targets before any mutation;
- avoid automatic edits to global client configuration, repository instruction
  files, or shell hooks;
- avoid agent prompts, model proxying, routing, editing, execution, network,
  durable memory, and source-workspace mutation;
- fail closed if the target client cannot express required launch constraints.

The initial implementation is documentation/template-first. A future command
may render or validate a kit, but it is read-only unless a separately approved
decision authorizes narrowly scoped external configuration mutation.

### Read-only diagnostics

Adopt a `doctor` diagnostic contract. It checks local prerequisites, isolation,
worker identity, connection-kit syntax, and a real bounded MCP exchange. It
returns a versioned machine-readable report and a source-free human summary.
It neither repairs state nor changes source, client configuration, network,
dependency, parser, or shell state.

## Consequences

### Positive

- Users can distinguish useful lexical access from proven structural evidence.
- "Supports" becomes a testable contract rather than a marketing count.
- Named-agent support can improve adoption without making Impresari Context an
  agent runtime or configuration manager.
- Client upgrades and scope changes become observable compatibility events.
- A doctor report offers a safe path to diagnose local installation failures.
- The deterministic context planner can later reason over a truthful capability
  matrix and report unavailable evidence classes explicitly.

### Costs

- Each client integration needs separate maintenance and release testing.
- The project cannot claim broad client reach based only on MCP compatibility.
- Documentation/template-first setup is less convenient than auto-installers.
- Language addition remains slower than adding a Tree-sitter grammar alone.
- Doctor requires careful source-free diagnostics and a maintained fixture.

## Alternatives considered

### Treat every compatible MCP host as supported

Rejected. It would conceal client-specific launch, lifecycle, scope, and safety
differences and make a support claim without end-to-end evidence.

### Copy an auto-wiring installer that edits known agent files and hooks

Rejected for Phase 0. This creates hidden state and expands the project's
external write, configuration, and compatibility responsibilities. It also
conflicts with the explicit non-goal of silently modifying editor or shell
configuration.

### Add agent-specific behavior to the engine

Rejected. The engine treats consumers and roles as opaque and must remain
client-neutral. Agent adaptation belongs in thin connection kits.

### Publish a single language count

Rejected. It conflates lexical file handling, grammar availability, and
verified structural semantics.

## Verification

- Release tests compare published language rows to parser/resolver inventory and
  language-admission evidence.
- Each first-class client passes clean-install, project/user-scope, lifecycle,
  tool discovery, packet equivalence, source-immutability, and safe-removal
  tests on its declared supported platforms.
- Connection-kit dry runs report all proposed targets and never mutate them.
- Doctor succeeds on a controlled fixture and rejects invalid worker identity,
  unsafe root/cache layout, malformed configuration, and incompatible MCP
  behavior without exposing source or absolute workspace paths.
- Network-denied and hostile-repository tests prove that a kit or doctor run
  adds no authority.

## Explicitly deferred

- Automatic client configuration installation or repair.
- Global shell hooks, instruction-file injection, and background refresh.
- Remote MCP/HTTP, multi-client sessions, authentication, or hosted client
  management.
- Agent orchestration, prompts, model proxies, memory, editing, execution, or
  token-accounting features.
- Structural language expansion beyond the released TypeScript/JavaScript
  family; these follow ADR-0004 and their own admission evidence.

## References

- [Phase 0: Language and Client Foundation PRD](../product/phase-0-language-and-client-foundation-prd.md)
- [ADR-0004: Source-language and parser strategy](0004-source-language-and-parser-strategy.md)
- [ADR-0014: Local stdio MCP transport](0014-local-stdio-mcp-transport.md)
- [System boundaries](../boundaries.md)
