# ADR-0070: Distribute release binaries through a separately governed Homebrew tap

- Status: Proposed; founder decision required
- Date: 2026-08-29
- Related PRD: [Homebrew distribution PRD](../product/homebrew-distribution-prd.md)
- Related architecture: [Homebrew distribution ARD](../architecture/homebrew-distribution-ard.md)

## Context

Impresari Context has a checksum-verified pinned installer and a short
`quickstart` path. A Homebrew formula can reduce installation friction and use
the package manager's established user-invoked update lifecycle. It also adds a
new trust boundary: a non-official tap contains Ruby code that Homebrew loads
with the user's authority, and maintaining it requires cross-repository release
credentials and tests.

## Proposed decision

If approved, distribute the three existing native release executables through
one formula in a dedicated `homebrew-tap` repository.

- Prefer direct fully qualified formula installation over instructions that
  trust the entire tap.
- Consume only immutable accepted GitHub Release archives and exact SHA-256
  values for macOS ARM64 and Linux x86-64.
- Keep application release, tap update proposal, tap acceptance, and formula
  publication as distinct evidence and authority boundaries.
- Allow automation to open a deterministic formula-update pull request only
  after release completion; never allow it to merge that pull request.
- Treat `brew update`, `brew upgrade`, `brew pin`, and `brew uninstall` as
  Homebrew-owned, user-controlled lifecycle operations.
- Do not add an Impresari self-updater, background check, daemon, scheduled
  task, installation-time client setup, or application runtime network access.

## Consequences

- Installation becomes one package-manager command for the recorded platform
  tuples, and explicit upgrades no longer require manually repeating the pinned
  installer flow.
- The project must maintain a second repository, formula CI, ownership rules,
  scoped cross-repository credentials, and release-to-formula traceability.
- Formula availability may lag a release until its independent tests and human
  review pass. That lag is an intentional safety property.
- Homebrew's metadata refresh behavior is not represented as an Impresari
  automatic-update claim.
- Unsupported platforms retain the existing manual-download or installer path.

## Rejected alternatives

- A formula stored in the application repository is not a conventional,
  independently governed tap.
- A moving `latest` URL or checksum discovered during installation is not
  reproducible and cannot inherit release assurance.
- Automatic formula merge collapses independent acceptance into publication.
- A cask or self-updater does not match the current three-CLI-binary product
  shape and would introduce unnecessary lifecycle authority.

## Decision gate

This ADR records a reviewable proposal, not an accepted decision. Change its
status only after explicit founder approval. Repository creation, credential
grant, publication, and release remain separate manual actions.
