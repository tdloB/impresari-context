# Phase 5 PHP Structural Admission PRD

- Status: Implemented; hosted acceptance pending

## Outcome

Add snapshot-bound structural evidence for PHP `.php` source files.

## Requirements

- Pin `tree-sitter-php 0.24.2` inside the isolated structural worker.
- Emit only syntax-confirmed namespaces, classes, interfaces, traits, enums,
  functions, and methods; literal direct include/require forms; direct named
  function calls; references; and containment.
- Preserve exact content identity, bounded fact/depth/response limits, explicit
  syntax-recovery warnings, and deterministic output ordering.
- Do not execute PHP or invoke Composer, an interpreter, package manager,
  framework, extension, language server, generated-code tool, executable, or
  network service.
- Do not claim autoload resolution, framework conventions, dynamic include
  evaluation, extension state, method dispatch, or runtime behavior.

## Acceptance

- Unit fixtures cover `.php` admission, named declarations, literal includes,
  direct calls, computed-include omission, and malformed-source recovery.
- Dependency policy, lockfile/SBOM inputs, compatibility manifest, repository
  policy checks, formatting, linting, tests, and hosted acceptance all pass.
- No public PHP structural-support claim is made until hosted acceptance
  succeeds.
