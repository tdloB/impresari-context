# ADR-0067: Ruby structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-29

## Context

The founder explicitly requested Ruby repository scanning under the approved
five-language Phase 5 program in ADR-0064. Ruby's open classes and dynamic
metaprogramming require a strict distinction between concrete syntax evidence
and runtime meaning.

## Decision

Admit Ruby `.rb` files through the isolated structural worker using pinned
`tree-sitter-ruby 0.23.1` (MIT). Emit only syntax-confirmed modules, classes,
instance methods, singleton methods, literal direct `require`,
`require_relative`, and `load` forms, direct receiver-free calls, references,
and containment.

## Boundary

Do not execute Ruby or invoke an interpreter, Bundler, RubyGems, Rails,
language server, package manager, generated-code tool, executable, or network
service. Do not claim evaluation, macro or metaprogram expansion, monkey-patch
resolution, autoloading, framework conventions, gem resolution, method lookup,
dynamic dispatch, or runtime behavior. Computed import and call targets are
omitted rather than guessed.

## Consequences

- Ruby facts retain exact parser, grammar, resolver, graph, snapshot, and
  content identities.
- Literal requires are syntax evidence, not resolved dependencies.
- Hosted acceptance remains mandatory before the compatibility manifest can
  advertise Ruby structural support.
