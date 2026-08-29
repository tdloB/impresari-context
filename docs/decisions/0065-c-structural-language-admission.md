# ADR-0065: C structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-29

## Context

The founder explicitly requested C repository scanning and overrode the general
Phase 5 demand gate in ADR-0064. C and C++ require separate grammar identities
and claims even though their file conventions overlap.

## Decision

Admit C `.c` and `.h` files through the isolated structural worker using pinned
`tree-sitter-c 0.24.2` (MIT). Emit only syntax-confirmed named functions,
structs, unions, enums, and typedefs; direct preprocessor includes; direct
calls; references; and containment.

For the C admission, `.h` is interpreted as C. The later C++ slice must not
silently reinterpret `.h`; any C++ header policy requires explicit,
deterministic project evidence and its own recorded decision.

## Boundary

Do not invoke a C compiler, linker, preprocessor, build system, package manager,
generated-code tool, executable, or network service. Do not claim macro
expansion, conditional-compilation truth, include resolution, type checking,
linkage, ABI, data layout, build-flag awareness, or runtime behavior.

## Consequences

- C facts retain exact parser, grammar, resolver, graph, snapshot, and content
  identities.
- Preprocessor includes are literal syntax evidence, not resolved dependencies.
- Hosted acceptance remains mandatory before the compatibility manifest can
  advertise C structural support.

