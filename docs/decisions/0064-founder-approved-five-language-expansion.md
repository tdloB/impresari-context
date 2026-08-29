# ADR-0064: Admit a founder-approved five-language expansion program

- Status: Accepted
- Date: 2026-08-29
- Scope: Phase 5 C, C++, Ruby, PHP, and Swift structural evidence

## Context

ADR-0040 originally required a separate attributable-demand decision before
selecting each additional Phase 5 language. The founder has explicitly directed
Impresari Context to scan Swift, PHP, Ruby, C, and C++ repositories and has
overridden that sequencing restriction. The request supplies attributable
demand for all five languages, but it does not justify collapsing their
independent correctness or security gates.

Maintained Rust grammar releases are available for evaluation: `tree-sitter-c
0.24.2`, `tree-sitter-cpp 0.23.4`, `tree-sitter-ruby 0.23.1`,
`tree-sitter-php 0.24.2`, and `tree-sitter-swift 0.7.3`. Version availability is
not itself an admission claim.

## Decision

Approve all five languages for implementation in the order C, C++, Ruby, PHP,
and Swift. Preserve one PRD, ADR, grammar identity, evaluation corpus, public
claim, and hosted acceptance gate per language. Reuse the existing isolated
worker and common structural fact vocabulary; do not add compiler, runtime,
package-manager, build-system, framework, language-server, or network authority.

ADR-0040 continues to govern languages outside this named batch. This decision
overrides only its one-candidate selection gate for these five languages.

## Consequences

- The roadmap may commit to all five outcomes without pretending they are one
  atomic or already delivered feature.
- Partial progress remains explicit; a language is unsupported structurally
  until its own hosted admission merges.
- C and C++ remain separate grammars and claims.
- Dynamic and compiler-dependent semantics remain explicit omissions.

## References

- [Five-Language Expansion PRD](../product/phase-5-five-language-expansion-prd.md)
- [Five-Language Expansion ARD](../architecture/phase-5-five-language-expansion-ard.md)
- [ADR-0040](0040-demand-led-language-admission.md)
