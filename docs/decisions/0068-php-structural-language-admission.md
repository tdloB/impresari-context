# ADR-0068: PHP structural-language admission

- Status: Accepted for implementation
- Date: 2026-08-29

## Context

The founder explicitly requested PHP repository scanning under the approved
five-language Phase 5 program in ADR-0064. PHP includes, autoloading, framework
conventions, and runtime extension state require a strict syntax-only boundary.

## Decision

Admit PHP `.php` files through the isolated structural worker using pinned
`tree-sitter-php 0.24.2` (MIT). Emit only syntax-confirmed namespaces, classes,
interfaces, traits, enums, functions, methods, literal direct include/require
forms, direct named function calls, references, and containment.

## Boundary

Do not execute PHP or invoke an interpreter, Composer, package manager,
framework, extension, language server, generated-code tool, executable, or
network service. Do not claim autoload resolution, framework conventions,
dynamic include evaluation, extension state, method dispatch, type resolution,
or runtime behavior. Computed include and call targets are omitted rather than
guessed.

## Consequences

- PHP facts retain exact parser, grammar, resolver, graph, snapshot, and content
  identities.
- Literal includes are syntax evidence, not resolved dependencies.
- Hosted acceptance remains mandatory before the compatibility manifest can
  advertise PHP structural support.
