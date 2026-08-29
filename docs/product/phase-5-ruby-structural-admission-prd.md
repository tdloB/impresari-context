# Phase 5 Ruby Structural Admission PRD

- Status: Implemented; hosted acceptance pending

## Outcome

Add snapshot-bound structural evidence for Ruby `.rb` source files.

## Requirements

- Pin `tree-sitter-ruby 0.23.1` inside the isolated structural worker.
- Emit only syntax-confirmed module, class, instance-method, and singleton-method
  declarations; literal direct `require`, `require_relative`, and `load` forms;
  direct receiver-free calls; references; and containment.
- Preserve exact content identity, bounded fact/depth/response limits, explicit
  syntax-recovery warnings, and deterministic output ordering.
- Do not execute Ruby or invoke Bundler, RubyGems, Rails, an interpreter,
  package manager, language server, generated-code tool, executable, or network
  service.
- Do not claim `eval`, metaprogram execution, monkey-patch resolution,
  autoloading, framework conventions, gem resolution, dynamic dispatch, or
  runtime behavior.

## Acceptance

- Unit fixtures cover `.rb` admission, named declarations, literal requires,
  direct calls, references, containment, computed-form omission, and malformed
  source recovery.
- Dependency policy, lockfile/SBOM inputs, compatibility manifest, repository
  policy checks, formatting, linting, tests, and hosted acceptance all pass.
- No public Ruby structural-support claim is made until hosted acceptance
  succeeds.
