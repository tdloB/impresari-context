# Phase 5 Swift Structural Admission PRD

- Status: Complete; hosted acceptance passed in PR 107

## Outcome

Add snapshot-bound structural evidence for Swift `.swift` source files.

## Requirements

- Pin `tree-sitter-swift 0.7.3` inside the isolated structural worker.
- Emit only syntax-confirmed named type, protocol, function, method, and type
  alias declarations; direct imports and receiver-free calls; references; and
  containment.
- Preserve exact content identity, bounded fact/depth/response limits, explicit
  syntax-recovery warnings, and deterministic output ordering.
- Do not invoke Swift, SwiftPM, Xcode, a compiler, macro or plugin host,
  language server, generated-code tool, signing tool, executable, or network
  service.
- Do not claim module lookup, conditional-compilation truth, macro expansion,
  type checking, overload resolution, Objective-C bridging, build, signing, or
  runtime behavior.

## Acceptance

- Unit fixtures cover `.swift` admission, named declarations, direct imports,
  direct calls, references, containment, and malformed-source recovery.
- Dependency policy, lockfile/SBOM inputs, compatibility manifest, repository
  policy checks, formatting, linting, tests, and hosted acceptance all pass.
- No public Swift structural-support claim is made until hosted acceptance
  succeeds.
