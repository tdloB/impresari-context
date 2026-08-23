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

## Open items

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

Preparation status: `CHANGELOG.md` contains reviewed draft `v0.1.0` notes. The
criterion remains open until those notes are attached to the published release.

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
