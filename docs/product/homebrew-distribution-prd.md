# Impresari Context — Homebrew distribution PRD

- Status: Superseded for macOS by ADR-0076; Linux formula proposal remains
- Date: 2026-08-29
- Authority: Future adoption-experience increment
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Architecture: [Homebrew distribution ARD](../architecture/homebrew-distribution-ard.md)
- Decision: [ADR-0070](../decisions/0070-homebrew-tap-distribution.md)

## Objective

Provide a one-command, package-manager-native installation and user-invoked
upgrade path for the standalone Linux executables. The accepted
[macOS hybrid XPC distribution PRD](macos-hybrid-xpc-distribution-prd.md)
governs the separate signed cask topology.

## User outcome

After explicitly selecting the project tap and formula, a user can install,
inspect, upgrade, pin, and uninstall Impresari Context through Homebrew. The
installed CLI, MCP server, and structural worker remain sibling executables so
the existing `quickstart` contract continues to locate the MCP server without
searching the user's environment.

## Proposed scope

- A separately governed GitHub tap repository using Homebrew's conventional
  `homebrew-tap` name and a single `impresari-context` formula.
- Direct, fully qualified installation so the user trusts only the named
  formula rather than every current or future item in the tap.
- Binary release installation for Linux x86-64 from immutable GitHub Release
  archives with exact SHA-256 values.
- Installation of `impresari-context`, `impresari-context-mcp`, and
  `impresari-context-structural-worker` into Homebrew's managed prefix.
- A release-generated, reviewable pull request that updates the formula only
  after the normal release artifacts and checksums exist.
- Tap CI covering formula audit, install, a source-free smoke check, upgrade
  from the preceding supported Linux release, and uninstall.

## Non-goals

- Publication in `homebrew/core`, source builds, bottles, Windows
  package managers, resolving an unpublished version, or replacing the pinned
  portable installer.
- Automatic formula-PR merge, automatic GitHub Release publication, silent
  client configuration, `quickstart --apply`, shell-startup changes, client
  discovery, sign-in, trust, or live-client invocation.
- An Impresari-managed update check, self-update command, resident process,
  launch agent, scheduled task, telemetry, or network access during normal
  Impresari Context execution.

## Security and authority requirements

- Formulae are executable Ruby from a non-official tap. Documentation must
  present the fully qualified formula name and the tap trust boundary before
  the installation command.
- Every platform URL is immutable and every archive has the SHA-256 published
  by the corresponding accepted Impresari Context release.
- The formula performs no network operation beyond Homebrew's declared fetch,
  invokes none of the installed executables during installation, and runs no
  post-install mutation.
- A formula update is proposed only from a completed release and requires
  review in the tap repository. Credentials are isolated to the release-to-tap
  pull-request operation and never enter an archive or formula.
- `brew update` may refresh tap metadata and `brew upgrade` may replace an
  unpinned installation under Homebrew's own user-controlled lifecycle. This
  proposal does not label that behavior an Impresari automatic updater.

## Acceptance criteria

- The tap and formula pass their own policy and hosted CI on Linux x86-64
  before the installation path is documented as supported.
- A clean install yields exactly the three expected sibling executables, whose
  versions equal the formula version and whose packaged checksums trace to the
  accepted release evidence.
- The existing source-free CLI smoke and `quickstart` preview work from the
  Homebrew prefix without configuration writes or workspace mutation.
- Unsupported operating-system/architecture combinations fail before artifact
  installation with an explicit compatibility result.
- Upgrade tests prove the previous formula version is replaced only after an
  explicit Homebrew upgrade operation; pinning prevents that upgrade.
- Uninstall removes the formula-owned executables without touching Impresari
  cache, workspace, MCP configuration, guidance artifacts, or user receipts.
- A compromised, mismatched, missing, or unpublished release checksum cannot
  produce a formula-update pull request.

## Manual boundary

Linux formula implementation remains gated on its separate tap decision.
ADR-0076 independently authorizes staged macOS cask work, but publication and
credential-bearing release operations remain explicit owner actions.
