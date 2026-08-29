# Impresari Context — Phase 5 Five-Language Expansion PRD

- Status: Founder-approved for implementation
- Date: 2026-08-29
- Languages: C, C++, Ruby, PHP, and Swift
- Roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Architecture: [Five-Language Expansion ARD](../architecture/phase-5-five-language-expansion-ard.md)
- Decision: [ADR-0064](../decisions/0064-founder-approved-five-language-expansion.md)

## Objective

Allow Impresari Context to discover and recover exact bounded evidence from
repositories containing C, C++, Ruby, PHP, and Swift, and to emit deliberately
bounded structural facts for each language. The founder's explicit request is
the attributable demand record and overrides the earlier requirement to select
only one Phase 5 candidate at a time.

## Delivery sequence

1. C establishes the native-language declaration, include, call, reference,
   and containment boundary.
2. C++ extends the native boundary with namespaces, classes, methods, and
   direct includes without claiming template or compiler semantics.
3. Ruby adds modules, classes, methods, direct calls, requires, references,
   and containment without executing metaprogramming.
4. PHP adds namespaces, classes, functions/methods, direct calls, includes,
   references, and containment without Composer or runtime evaluation.
5. Swift adds named declarations, direct imports/calls, references, and
   containment without SwiftPM, macro, compiler, or type-checker authority.

Each numbered slice requires its own admission PRD, ADR, pinned grammar,
compatibility inventory update, adversarial corpus, deterministic facts, SBOM
review, full local gate, and hosted acceptance before its public structural
claim is made. One language failing does not silently weaken or block the
already admitted languages.

## Required evidence classes

For every language:

- discovery recognizes only the declared portable extensions;
- lexical evidence remains exact eligible UTF-8 source evidence;
- structural parsing runs only in the existing isolated worker;
- declarations, direct imports/includes/requires, direct calls, references,
  and containment are emitted only where deterministic fixtures prove them;
- syntax recovery and unsupported semantics remain visible;
- facts are snapshot-bound, budgeted, canonical, and source-verified; and
- malformed, oversized, identity-mismatched, or unsupported input fails closed.

## Language-specific exclusions

- C: no preprocessing, macro expansion, conditional-compilation evaluation,
  header search, compiler ABI, build system, linker, or runtime claims.
- C++: all C exclusions plus no template instantiation, overload resolution,
  argument-dependent lookup, concepts, module resolution, or type inference.
- Ruby: no `eval`, metaprogram execution, monkey-patch resolution, Bundler,
  Rails convention inference, autoloading, gems, or runtime dispatch claims.
- PHP: no PHP execution, Composer/autoload resolution, framework convention
  inference, dynamic include evaluation, extension state, or runtime dispatch.
- Swift: no macro/plugin execution, SwiftPM/Xcode project resolution, module
  lookup, conditional-compilation evaluation, type checking, overload
  resolution, Objective-C bridging, build, signing, or runtime claims.

## Acceptance criteria

- Every language has independent PRD/ADR/evaluation evidence and an exact
  pinned grammar identity.
- The structural worker retains a fixed language inventory; it never loads a
  grammar from the scanned repository or network.
- Tier-A platform tests, fuzzing, security boundaries, license policy, SBOM,
  schemas, compatibility claims, and deterministic evaluation all pass.
- Public documentation distinguishes discovery, lexical, and structural
  support and states every unsupported semantic boundary.
- No language adds source-write, compiler, package-manager, build-system,
  process, credential, database, or network authority.

## Completion

This program is complete only when all five individual admissions have merged
with hosted evidence. Until then, the roadmap lists exactly which slices are
accepted and which remain in progress.
