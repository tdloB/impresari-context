# ADR-0069: Swift structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-29

## Context

The founder explicitly requested Swift repository scanning under the approved
five-language Phase 5 program in ADR-0064. Swift macros, conditional
compilation, module lookup, build settings, and overload resolution require a
strict distinction between concrete syntax and toolchain meaning.

## Decision

Admit Swift `.swift` files through the isolated structural worker using pinned
`tree-sitter-swift 0.7.3` (MIT). Emit only syntax-confirmed named type,
protocol, function, method, and type-alias declarations, direct imports,
receiver-free direct calls, references, and containment.

## Boundary

Do not invoke Swift, SwiftPM, Xcode, a compiler, macro or plugin host, language
server, generated-code tool, signing tool, executable, or network service. Do
not claim module lookup, conditional-compilation truth, macro expansion, type
checking, overload resolution, Objective-C bridging, build settings, signing,
or runtime behavior. Ambiguous and computed forms are omitted rather than
guessed.

## Consequences

- Swift facts retain exact parser, grammar, resolver, graph, snapshot, and
  content identities.
- Imports are syntax evidence, not resolved modules or packages.
- Hosted acceptance remains mandatory before the compatibility manifest can
  advertise Swift structural support.
