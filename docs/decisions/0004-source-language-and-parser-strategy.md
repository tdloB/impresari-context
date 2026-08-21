# ADR-0004: Source-language and parser strategy

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: MVP lexical coverage and Slice B structural intelligence

## Context

The MVP promises exact path and lexical evidence, not semantic code
understanding. The later structural slice needs parsers that tolerate incomplete
or erroneous source, support incremental updates, and can map syntax to one
original canonical graph contract.

Attempting many languages at once would weaken provenance, confidence, and
evaluation. Executing repository compilers, plugins, build scripts, or language
servers would also violate the initial no-execution boundary.

## Decision

Separate **text eligibility** from **structural-language support**.

### MVP text coverage

- Exact hashing and metadata discovery operate on eligible regular files as raw
  bytes, regardless of language.
- Lexical search and excerpts support UTF-8 text in the MVP.
- UTF-8 with BOM may be accepted after stripping the BOM for text interpretation
  while hashing the original raw bytes.
- Invalid UTF-8, UTF-16, binary, generated, oversized, or policy-excluded files
  remain discoverable as metadata when safe but are explicitly unsupported for
  text search/excerpts unless a later encoding decision adds support.
- Language/extension labels are hints for filtering and reporting; they do not
  create semantic claims.

### First structural language family

Slice B will first support:

- TypeScript (`.ts`);
- TSX (`.tsx`);
- JavaScript (`.js`, `.mjs`, `.cjs`);
- JSX (`.jsx`).

JSON and JSON-with-comments configuration may receive a narrow configuration
adapter, but that does not imply full JSON Schema or runtime configuration
semantics.

Python is the preferred second structural language after the TypeScript/
JavaScript adapter passes quality, security, and provenance gates. It is not
part of the first structural acceptance scope.

### Parser selection

Use **Tree-sitter** as the initial concrete-syntax parser framework through its
official Rust binding and pinned grammar revisions.

Tree-sitter produces concrete syntax trees, supports incremental parsing, and
is designed to remain useful in the presence of syntax errors. The parser tree
is an input to this project's resolver; it is not the public graph contract.

### Resolver boundary

- Write original project-owned resolver adapters that translate syntax nodes to
  the canonical graph schema.
- Pin the Tree-sitter runtime, Rust binding, and every grammar by version and
  artifact digest/revision.
- Record parser, grammar, resolver, and graph-contract versions on every fact.
- Distinguish syntax-confirmed facts from heuristically or incompletely resolved
  relationships.
- Do not execute `package.json` scripts, TypeScript plugins, loaders, bundlers,
  compilers, language servers, or repository code.
- The first adapter provides syntactic declarations, containment, imports,
  exports, and locally supported references. Cross-package/module resolution is
  conservative and reports unresolved edges.
- Calls are emitted only when the resolver can support the declared confidence;
  textual name similarity is not a confirmed call edge.

### Process isolation

Tree-sitter and grammar code enter in Slice B, not the MVP. The first structural
implementation runs parsing in a dedicated worker process separated from the
policy/control process.

- The worker receives bounded source bytes and parser configuration over a
  versioned local protocol, not arbitrary filesystem paths.
- It receives no network, model, repository-write, general environment, or
  command-execution capability.
- Time, input, output, memory, and crash behavior are bounded and observable.
- Process separation is defense-in-depth, not a claim of a complete OS sandbox.
- Worker output re-enters through validation and evidence normalization before
  graph storage.

If a supported platform cannot enforce a claimed resource control, the
limitation must be explicit and evaluated rather than silently ignored.

Before Slice B implementation, a dedicated worker-protocol ADR must fix:

- message framing, schema versions, maximum request/response bytes, and rejection
  behavior for unknown or duplicate fields;
- worker executable and grammar identity verification;
- inherited handle, environment, current-directory, filesystem, network, and
  standard-stream policy for every Tier A platform;
- wall-time, CPU, memory, nesting/depth, input, output, and restart limits;
- cancellation, crash, panic, malformed-output, partial-output, and repeated-
  failure behavior;
- the exact distinction between enforced OS isolation, application-enforced
  limits, and unsupported claims on each platform.

The policy process treats all worker bytes as hostile, validates the complete
response before use, and never promotes partial worker output.

## Rationale

TypeScript/JavaScript is the best first structural family because the reference
AI App Builder OS and many AI application repositories use it, one related
grammar family covers several common extensions, and it exercises modern module,
route, test, and configuration patterns.

Tree-sitter offers a shared incremental parsing framework and official Rust
binding while allowing the project to own graph semantics. It avoids making a
language compiler or LSP the canonical index and does not require repository
execution.

## Consequences

### Positive

- MVP remains broadly useful for UTF-8 repositories without overclaiming
  semantics.
- First structural scope is narrow enough for exact evaluation.
- Parser replacement does not change the public graph contract.
- Syntax errors and incomplete working trees can still yield labeled facts.
- Process crashes or malformed parser output need not corrupt the control plane.

### Costs

- Tree-sitter includes a native C runtime and third-party grammars that require
  supply-chain review.
- A separate worker and protocol add implementation complexity.
- Syntactic analysis cannot fully resolve dynamic JavaScript or TypeScript
  module/type behavior.
- Users of other languages receive lexical retrieval before structural support.

## Alternatives Considered

### TypeScript compiler API / language service

Deferred. It can provide richer TypeScript semantics but introduces a Node.js
runtime, repository configuration/plugins, higher resource cost, and a more
complex execution boundary. A future optional semantic resolver may be evaluated
behind the same graph/provenance contract.

### Language Server Protocol processes

Deferred because language servers vary by language, may execute workspace
configuration, and create broad process/environment access. They are not needed
for the first deterministic graph.

### Regex-only structural extraction

Rejected because it cannot reliably establish declarations, nesting, imports,
or calls and would encourage false confirmed edges.

### Tree-sitter parsers loaded dynamically in the core process

Rejected for the initial structural implementation because it broadens the
trusted computing base and makes native parser faults control-plane faults.

### Support many grammars immediately

Rejected because parser availability is not the same as a reviewed resolver,
provenance model, or passing evaluation suite.

## Verification

- MVP encoding and unsupported-file conformance fixtures.
- Pinned parser/grammar inventory with licenses and digests.
- Syntax-error, malformed-input, huge-depth, long-token, cancellation, crash,
  and resource-limit parser tests.
- Golden graph fixtures authored for this project.
- Every fact validated for source span, parser/grammar/resolver version,
  extraction method, and confidence.
- TypeScript/JavaScript structural quality gates added to IC-EVAL-001 before
  Slice B release.
- Worker-protocol conformance, malformed-frame, capability-denial, crash-loop,
  and platform resource-enforcement evidence.

## Official References

- [Tree-sitter introduction](https://tree-sitter.github.io/tree-sitter/)
- [Tree-sitter parser use and official bindings](https://tree-sitter.github.io/tree-sitter/using-parsers/)
- [Tree-sitter grammar model](https://tree-sitter.github.io/tree-sitter/creating-parsers/3-writing-the-grammar.html)

## Review Triggers

Review if Tree-sitter or its grammars cannot meet security/provenance gates, the
worker boundary is impractical on a Tier A platform, TypeScript semantics require
a compiler-backed resolver for target tasks, or evaluation supports promotion of
another language.
