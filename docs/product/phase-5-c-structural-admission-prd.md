# Phase 5 C Structural Admission PRD

- Status: Complete; hosted acceptance passed in PR 103

## Outcome

Add snapshot-bound structural evidence for C `.c` source files and `.h` headers.

## Requirements

- Pin `tree-sitter-c 0.24.2` inside the existing isolated structural worker.
- Emit only syntax-confirmed function, struct, union, enum, and typedef
  declarations; direct preprocessor includes; direct calls; references; and
  containment.
- Treat `.h` as C in this slice. C++ header admission is deferred to the
  independent C++ slice so one path never receives competing grammar claims.
- Preserve exact content identity, bounded fact/depth/response limits, explicit
  syntax-recovery warnings, and deterministic output ordering.
- Do not invoke a compiler, linker, preprocessor, build system, package manager,
  generated-code tool, executable, or network service.

## Acceptance

- Unit fixtures cover source/header recognition, named declarations, direct
  includes, direct calls, references, and malformed-source recovery.
- Dependency policy, lockfile/SBOM inputs, compatibility manifest, repository
  policy checks, formatting, linting, tests, and hosted acceptance all pass.
- No public C structural-support claim is made until hosted acceptance succeeds.
