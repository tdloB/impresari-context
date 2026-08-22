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

### Release versioning and tags (`version_unique`, `version_semver`, `version_tags`)

Each public release must have a unique Semantic Versioning identifier and a
matching protected Git tag, with public evidence connecting the source commit,
release artifacts, checksums, and provenance.

Required work:

- Publish the first approved release as `v0.1.0` rather than the internal
  `0.0.0` development version.
- Create the matching Git tag through the approved release workflow.
- Protect release tags against unauthorized modification or deletion.
- Link the GitHub release, tag, checksums, attestations, and exact source commit
  from the release evidence documentation.

Acceptance evidence:

- The public release and tag use the same valid Semantic Versioning identifier.
- Release artifacts and checksums are reproducible from the tagged commit.
- GitHub artifact attestations bind the published artifacts to the repository,
  workflow, and source commit.
- The protected tag and release evidence URLs are publicly accessible.

### Human-readable release notes (`release_notes`)

Every public release must include human-readable release notes that explain its
major changes, upgrade relevance, and expected user impact. A raw commit log is
not sufficient.

Required work:

- Create release notes for `v0.1.0` as part of the approved release process.
- Summarize added capabilities, compatibility or migration considerations,
  security-relevant changes, known limitations, and unresolved risks.
- For later releases, identify every publicly known project vulnerability fixed
  by that release, including its CVE or equivalent identifier when one exists.
- Link the notes from the GitHub release and release evidence documentation.

Acceptance evidence:

- The public GitHub release contains reviewed prose rather than an unedited
  version-control log.
- The notes allow a user to understand whether and how to adopt or upgrade.
- Applicable fixed project vulnerabilities and identifiers are explicitly
  listed, or the notes explicitly state that none are known for that release.

## Tracking rule

Add any later unknown or unmet badge criterion to this file with its criterion
identifier, required work, and objective acceptance evidence. Do not mark a
criterion met solely to improve the badge score.
