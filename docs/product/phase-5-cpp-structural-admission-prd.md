# Phase 5 C++ Structural Admission PRD

- Status: Complete; hosted acceptance passed in PR 104

## Outcome

Add snapshot-bound structural evidence for unambiguous C++ source and header
extensions: `.cc`, `.cpp`, `.cxx`, `.hh`, `.hpp`, and `.hxx`.

## Requirements

- Pin `tree-sitter-cpp 0.23.4` inside the isolated structural worker.
- Emit only syntax-confirmed namespace, class, struct, union, enum, alias, and
  function declarations; direct preprocessor includes; direct calls;
  references; and containment.
- Keep ambiguous `.h` files assigned to the C admission. Do not infer language
  from repository layout, build files, neighboring files, or file contents.
- Preserve exact content identity, bounded fact/depth/response limits, explicit
  syntax-recovery warnings, and deterministic output ordering.
- Do not invoke a compiler, linker, preprocessor, build system, package manager,
  generated-code tool, executable, or network service.

## Acceptance

- Unit fixtures cover all admitted extensions, `.h` non-reclassification,
  named declarations, direct includes, calls, references, and malformed-source
  recovery.
- Dependency policy, lockfile/SBOM inputs, compatibility manifest, repository
  policy checks, formatting, linting, tests, and hosted acceptance all pass.
- No public C++ structural-support claim is made until hosted acceptance
  succeeds.
