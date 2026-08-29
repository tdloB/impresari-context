# ADR-0066: C++ structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-29

## Context

ADR-0064 records attributable founder demand for C++ repository scanning.
Several C++ extensions are unambiguous, while `.h` is shared with C and cannot
be assigned by extension alone without introducing nondeterministic inference.

## Decision

Admit `.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, and `.hxx` through the isolated
worker using pinned `tree-sitter-cpp 0.23.4` (MIT). Emit only syntax-confirmed
namespaces, classes, structs, unions, enums, aliases, and functions; direct
preprocessor includes; direct calls; references; and containment.

Keep `.h` assigned to the C grammar established by ADR-0065. Do not inspect
build configuration, adjacent files, repository conventions, or source text to
guess whether an ambiguous header is C++.

## Boundary

Do not invoke a C++ compiler, linker, preprocessor, build system, package
manager, generated-code tool, executable, or network service. Do not claim
template instantiation, overload resolution, macro expansion,
conditional-compilation truth, include resolution, type checking, linkage,
ABI, data layout, build-flag awareness, or runtime behavior.

## Consequences

- C++ facts retain exact parser, grammar, resolver, graph, snapshot, and content
  identities.
- Literal includes are syntax evidence rather than resolved dependencies.
- Deterministic handling of `.h` is narrower than some editor heuristics but
  preserves one grammar identity per admitted path.
- Hosted acceptance remains mandatory before advertising C++ structural
  support.

