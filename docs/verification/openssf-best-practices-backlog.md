# OpenSSF Best Practices backlog

This document tracks OpenSSF Best Practices passing-level criteria that cannot
yet be supported by clear public repository evidence. A criterion remains
unknown or unmet in the badge questionnaire until its acceptance evidence is
implemented, reviewed, and linked here.

Badge project: <https://www.bestpractices.dev/en/projects/14209>

## Completed evidence

### Project interaction instructions (`interact`)

The public README now distinguishes clone/build instructions, ordinary GitHub
Issues, contribution instructions, and private vulnerability reporting.

### External-interface reference documentation (`documentation_interface`)

`docs/reference/interfaces.md` documents CLI and local MCP inputs, outputs,
errors, lifecycle, limits, versioning, schemas, and security boundaries.

### New-functionality test policy (`test_policy`, `tests_are_added`, `tests_documented_added`)

`CONTRIBUTING.md` and the pull-request template now require test evidence or an
approved exception. The local MCP feature supplies recent evidence: protocol,
lifecycle, authority, direct-engine equivalence, and adversarial tests are
mapped in `docs/verification/mcp-release-traceability.md`.

### Release versioning, provenance, and notes

Criteria: `version_unique`, `version_semver`, `version_tags`, and
`release_notes`.

The published [`v0.1.0` release](https://github.com/tdloB/impresari-context/releases/tag/v0.1.0)
uses a matching Semantic Versioning tag and is bound to exact commit
`c77e95ce95b2fde99da2582707d4e4d58a512122`. Its three native archives include
adjacent SHA-256 files and GitHub build provenance attestations. Its reviewed
human-readable notes describe capabilities, adoption considerations, security
boundaries, known limitations, and the absence of an independent audit. The
exact evidence is linked from `docs/verification/release-evidence.md`.

## Open items

### Release-tag protection (`version_tags` hardening)

The `v0.1.0` tag and exact-commit release evidence are public, but repository
rules do not yet protect release tags against unauthorized modification or
deletion. Add and verify a `v*` tag ruleset before treating this hardening item
as complete.

### Human secure-development knowledge (`know_secure_design`, `know_common_errors`)

AI-assisted analysis and automated security tooling do not establish that a
primary human developer satisfies the OpenSSF secure-development knowledge
criteria.

Required work:

- Designate at least one primary human developer who can substantiate secure
  software design knowledge appropriate to a local repository-context engine.
- Ensure that developer can identify the project's common vulnerability classes
  and at least one practical mitigation for each, including path traversal and
  link attacks, resource exhaustion, injection and untrusted-content handling,
  stale or substituted evidence, sensitive-data disclosure, unsafe dependency
  behavior, and process or protocol boundary failures.
- Record suitable training, demonstrated experience, or an independent
  security-review relationship without publishing unnecessary personal data.

Acceptance evidence:

- A named primary human developer or retained security reviewer satisfies the
  applicable OpenSSF criterion details.
- The public security or governance documentation identifies the responsible
  role and relevant secure-development process.
- The badge justification cites verifiable, non-sensitive evidence rather than
  treating AI output or scanners as human expertise.

The future independent-review scope and evidence requirements are documented in
`docs/security/independent-review-guide.md`. ADR-0017 makes that review a
pre-`v1.0.0` assurance target rather than a mandatory `v0.1.0` gate. These
questionnaire criteria remain `Unmet` until the human qualification or
review-relationship evidence exists.

## Tracking rule

Add any later unknown or unmet badge criterion to this file with its criterion
identifier, required work, and objective acceptance evidence. Do not mark a
criterion met solely to improve the badge score.
