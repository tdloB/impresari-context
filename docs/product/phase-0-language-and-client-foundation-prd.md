# Impresari Context — Phase 0: Language and Client Foundation PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-P0-001 / 0.1.
- Status: Accepted.
- Date: 2026-08-23.
- Owner: Aaron Boldt.
- Accepted by: Founder, 2026-08-23.
- Related records:
  - [Master Product PRD](master-prd.md)
  - [Evaluation PRD](evaluation-prd.md)
  - [System boundaries](../boundaries.md)
  - [ADR-0004: Source-language and parser strategy](../decisions/0004-source-language-and-parser-strategy.md)
  - [ADR-0014: Local stdio MCP transport](../decisions/0014-local-stdio-mcp-transport.md)
  - [ADR-0018: First-class client integration and compatibility contract](../decisions/0018-first-class-client-integration-and-compatibility-contract.md)

## 1. Purpose

Phase 0 makes Impresari Context's actual language and client support clear,
reproducible, and safe to adopt. It corrects public capability claims before
expanding them, defines the distinction between generic protocol compatibility
and first-class client support, and prepares the project for the first
language-coverage and client-integration release.

This phase is a product-contract, documentation, verification, and diagnostic
foundation. It does not add a network service, execute repository code, edit
source, install hooks, or create a new agent runtime.

## 2. Problem

Impresari Context currently has broad raw-file discovery and UTF-8 lexical
evidence coverage, but its structural parser/resolver implementation supports
only the TypeScript/JavaScript family. Public materials must not blur those
levels of support. A user deciding whether to adopt the project needs to know
what the engine can prove for their repository today, not merely what its
architecture may support later.

The local stdio MCP adapter can technically interoperate with clients that can
launch a compatible local process. That alone is not a first-class integration:
a supported connection requires documented configuration, deterministic launch
authority, an end-to-end verification path, and known degraded behavior.

Without these distinctions, the project risks overclaiming language coverage,
creating unsafe configuration expectations, and competing on vague agent-count
marketing rather than verifiable capability.

## 3. Goal

Before Phase 1 expands structural language support, establish a public support
contract that lets a developer answer:

1. What evidence operations work for my files and language today?
2. Which agent clients can I connect through an officially tested path?
3. What exact project or user configuration would be created or required?
4. Can I validate the installation without granting additional authority?
5. What happens when a client cannot express Impresari Context's required
   workspace, cache, consumer, and role constraints?

## 4. Non-goals

Phase 0 does not:

- add Python, configuration-format, Rust, Go, or other parser/resolver support;
- change the canonical evidence, packet, workspace, cache, or policy contracts;
- add HTTP, remote MCP, a daemon, background indexing, or multi-client
  sessions;
- auto-install, modify, or delete an agent's configuration, instruction file,
  shell profile, or hooks;
- introduce agent routing, persistent memory, editing, execution, model calls,
  prompt injection, or token-proxying;
- make generic MCP compatibility a claim of tested client support;
- guarantee that any third-party client honors trust labels or uses every
  available tool.

## 5. Phase 0 Deliverables

### 5.1 Public language-support matrix

Publish one versioned matrix that reports support by capability rather than a
single language count. It must distinguish at least:

| Support level | Meaning |
| --- | --- |
| Discovery | Eligible regular files can be fingerprinted and represented as metadata. |
| Lexical evidence | UTF-8 source can participate in exact path, filename, literal, and lexical retrieval with exact evidence recovery. |
| Structural evidence | The project-owned resolver may emit supported graph facts with source spans, parser/grammar/resolver provenance, confidence, and explicit limits. |
| Unsupported | The engine reports a safe explicit unsupported, excluded, binary, or partial state rather than implying semantic analysis. |

The initial structural row must list only TypeScript (`.ts`), TSX (`.tsx`),
JavaScript (`.js`, `.mjs`, `.cjs`), and JSX (`.jsx`) unless the shipped worker,
dependency inventory, resolver, and evaluation evidence demonstrate otherwise.

### 5.2 Public client-compatibility matrix

Publish one versioned matrix with these classifications:

| Classification | Meaning |
| --- | --- |
| First-class | A maintained, versioned connection kit and end-to-end conformance evidence exist for a named client and scope. |
| Generic local MCP | The client may be technically capable of launching a local stdio MCP server, but Impresari Context does not yet claim a maintained integration. |
| Experimental | A named integration is available for evaluation with documented limitations and no stability promise. |
| Unsupported | The project has no approved or safely expressible connection path. |

The initial first-class target set is Codex, Claude Code, and Cursor. Phase 0
does not claim their support until the acceptance criteria in this PRD pass.

### 5.3 Connection-kit contract

Define a connection kit for each first-class client. A kit includes:

- the exact supported client version/range and operating-system scope;
- project- and user-scope availability and consequences;
- an explicit, copyable configuration form or generated snippet;
- the exact binary path, workspace root, isolated cache path, opaque consumer
  identity, opaque role, and required operation time behavior;
- a dry-run rendering that shows every proposed configuration target and value;
- a verification command that exercises the real MCP lifecycle and validates
  packet equivalence against direct-engine use;
- documented removal instructions limited to the integration-owned entries;
- degraded or unsupported behavior when a client cannot preserve a required
  launch invariant.

Connection kits may be distributed as documentation, checked-in templates, and
a read-only renderer/validator. They must not silently modify third-party
configuration or install shell hooks.

### 5.4 Read-only diagnostics

Specify `impresari-context doctor` as a read-only diagnostic surface. It must
produce a versioned machine-readable report and a source-free human summary.
It validates, within explicitly supplied authority:

- binary, package, and schema compatibility;
- supported platform and local prerequisites;
- source, cache, and export-root separation;
- cache ownership, permissions, and isolation prerequisites;
- structural worker presence, digest, and empty working-directory contract;
- connection-kit inputs and client configuration syntax;
- a real local stdio MCP initialization, tool-list, and bounded packet build;
- direct-engine/MCP packet semantic equivalence on a synthetic fixture or
  explicitly selected harmless workspace.

`doctor` must not write to a source workspace, execute repository code, install
or repair dependencies, contact a network, modify a client configuration, or
display source content, secrets, absolute workspace paths, or raw diagnostics
in its safe human output.

The initial implementation may split the diagnostic into metadata-only and
in-process MCP lifecycle checks. Neither check constitutes client-specific
first-class conformance or permission to modify client configuration.

## 6. Functional Requirements

| ID | Requirement | Priority |
| --- | --- | --- |
| P0-FR-001 | Maintain one public language matrix with discovery, lexical, structural, and unsupported distinctions. | Must |
| P0-FR-002 | Derive every structural-support claim from the released parser/resolver inventory and evaluation evidence. | Must |
| P0-FR-003 | Correct or retire language claims that exceed the released implementation. | Must |
| P0-FR-004 | Maintain one public client matrix separating generic MCP capability from first-class support. | Must |
| P0-FR-005 | Provide a versioned connection kit for every client labeled first-class. | Must |
| P0-FR-006 | Make all configuration rendering opt-in and dry-run-capable before an external file could be changed. | Must |
| P0-FR-007 | Provide a read-only doctor report that validates a supported connection and reports limitations without source disclosure. | Must |
| P0-FR-008 | Run a client-specific end-to-end conformance suite before first-class status is published. | Must |
| P0-FR-009 | Preserve existing stdio-only, fixed-launch-authority MCP invariants. | Must |
| P0-FR-010 | Document project- versus user-scope consequences, including any machine-wide behavior. | Must |
| P0-FR-011 | Remove only an integration-owned entry and never delete unrelated client configuration. | Should |
| P0-FR-012 | Produce a machine-readable compatibility manifest suitable for release checks. | Should |

## 7. Security and Authority Requirements

1. A connection kit cannot select, expand, or override a workspace root through
   MCP tool input, repository text, or client prompt content.
2. Cache and export paths remain explicit and separate from the source
   workspace.
3. Consumer identity and role remain launch-time opaque values, not values the
   repository or client conversation can set dynamically.
4. An unsupported client capability must fail closed with a documented reason;
   it must not fall back to a broader filesystem read or a less-constrained
   configuration.
5. Generated configuration must pin an installed local binary or a separately
   verified package identity. It must never use an unpinned `latest` download
   path as part of verification.
6. The connection kit must neither inject instructions into repository-managed
   agent files nor install global shell hooks.
7. A client-specific adapter cannot add source mutation, network, model,
   execution, approval, routing, or durable-memory authority.
8. Doctor output and installation diagnostics must follow the existing
   source-free error and metadata-first audit rules.

## 8. Acceptance Criteria

### Public contract

- The README and reference documentation link to the language and client
  matrices.
- No release document claims structural support for a language absent from the
  current parser/resolver inventory.
- Each matrix has a version, publication date, and linked evidence source.

### First-class client admission

For each of Codex, Claude Code, and Cursor:

- A clean supported-platform installation follows the documented connection kit
  without manual inference.
- The client launches only the configured local MCP binary with fixed root,
  cache, consumer, and role arguments.
- Initialize, tool discovery, session open, packet build, packet resolve, and
  session close pass against the real client transport where the client exposes
  the corresponding surface.
- A packet from the connection matches the frozen direct-engine corpus
  semantically.
- Malformed, missing, or incompatible configuration fails source-free and adds
  no authority.
- The source-workspace before/after state is identical.
- The project/user scope behavior is documented and tested.
- Removal affects only the exact integration-owned entry and preserves
  unrelated client configuration.

### Doctor

- `doctor --json` emits a schema-valid report with no source content or secret
  values.
- The normal human report contains no absolute workspace path, source excerpt,
  raw client configuration, or raw error details.
- All failed checks identify a stable category and a safe remediation class.
- Doctor succeeds against a controlled fixture for each first-class client and
  detects intentional worker-digest, cache-separation, root, and MCP
  configuration failures.

## 9. Evaluation

Phase 0 adds a compatibility corpus to the Evaluation PRD. It must cover:

- all published matrix rows and their evidence source;
- client configuration parsing and dry-run rendering;
- project and user scope, including a no-global-write case;
- missing executable, changed executable identity, invalid roots, cache/source
  overlap, stale packet, malformed MCP framing, and unsupported-client cases;
- no-network and no-source-mutation runs;
- direct-engine/MCP equivalence;
- source-free diagnostic and secret-like value leakage checks.

No adoption, token-savings, or client-performance claim is approved by this
phase alone.

## 10. Dependencies and Sequencing

Phase 0 completes before a client is called first-class or before Phase 1 makes
new structural language claims. Phase 1 then adds Python and configuration
evidence through the established language-admission process while retaining the
same client contract.

The deterministic context planner is not a Phase 0 deliverable. Its later
profiles must consume the public support matrix and report unavailable evidence
classes rather than treating a language's lexical support as structural support.

## 11. Risks and Controls

| Risk | Control |
| --- | --- |
| Marketing language outruns implementation | Release-gated matrices derived from inventory and evaluation evidence. |
| Agent configuration broadens authority | Fixed launch arguments, dry-run review, no automatic mutation, and client conformance tests. |
| Client version changes break a kit | Versioned compatibility manifest and explicit first-class revalidation. |
| Installer-style convenience creates hidden state | Documentation/template-first kits, user opt-in, and scoped removal behavior. |
| Doctor becomes a privileged repair tool | Read-only contract and source/workspace immutability tests. |
| Agent count becomes a misleading metric | First-class classification requires maintained end-to-end evidence. |

## 12. Exit Decision

Phase 0 is complete only when the public matrices, connection-kit contract,
doctor specification, and client-specific conformance evidence are accepted by
the project steward. Failure to complete a first-class kit leaves that client in
the generic local-MCP, experimental, or unsupported category; it does not block
the neutral core from remaining usable through its existing CLI and library
surfaces.
