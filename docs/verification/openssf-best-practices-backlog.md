# OpenSSF Best Practices backlog

This document tracks OpenSSF Best Practices passing-level criteria that cannot
yet be supported by clear public repository evidence. A criterion remains
unknown or unmet in the badge questionnaire until its acceptance evidence is
implemented, reviewed, and linked here.

Badge project: <https://www.bestpractices.dev/en/projects/14209>

## Open items

### Project interaction instructions (`interact`)

The public project page must clearly explain how to obtain the software, provide
ordinary bug reports or enhancement feedback, and contribute changes.

Required work:

- Add explicit clone or installation instructions to `README.md`.
- Add an ordinary bug-report and feature-request path that is distinct from the
  private vulnerability-reporting path.
- Link the contribution instructions directly from that section.

Acceptance evidence:

- The public README contains one concise section covering all three paths.
- Each referenced GitHub URL is public and functional.

### External-interface reference documentation (`documentation_interface`)

The public documentation must describe the supported external interfaces,
including their inputs, outputs, versioning, failure behavior, and security
boundaries.

Required work:

- Consolidate the CLI command contract into public reference documentation.
- Document the local MCP tools, request and response shapes, errors, and
  process-local lifecycle.
- Link the applicable JSON schemas and compatibility/versioning rules.

Acceptance evidence:

- A user can integrate with the CLI or MCP interface using public documentation
  without reading implementation source.
- The reference documentation links to the normative schemas and is checked
  against the implemented interface during CI or release verification.

## Tracking rule

Add any later unknown or unmet badge criterion to this file with its criterion
identifier, required work, and objective acceptance evidence. Do not mark a
criterion met solely to improve the badge score.
