# Impresari Context — macOS Local-VM Cask Lifecycle Contract PRD

- Status: Accepted for source-free contract implementation; live distribution remains gated
- Date: 2026-09-01
- Authority: Founder-directed continuation of ADR-0087 and ADR-0091
- Architecture: [macOS Local-VM Cask Lifecycle Contract ARD](../architecture/macos-local-vm-cask-lifecycle-contract-ard.md)
- Decision: [ADR-0107](../decisions/0107-freeze-macos-local-vm-cask-lifecycle-before-signing.md)

## Objective

Freeze the exact ownership, layout, and lifecycle semantics of the selected
single-cask macOS distribution before any app bundle is assembled, signed,
notarized, installed, or published.

## User outcome

A future macOS user installs one Homebrew cask and continues to invoke
`impresari-context` from the terminal. Homebrew owns the app bundle and CLI
link as one version. Installation never creates a privileged helper,
persistent service, automatic updater, client configuration, or analyzer grant.

## Scope

- One source-free package contract for `Impresari Context.app` and the
  `impresari-context` CLI link.
- A closed embedded layout for the CLI, MCP server, structural worker, local-VM
  controller, guest payload directory, and release-metadata seal.
- Exact binding to the active ADR-0091 release-metadata seal and guest release.
- Homebrew-owned install, whole-bundle upgrade, explicit rollback, migration,
  and uninstall semantics.
- Deterministic offline validation with original-synthetic fixtures.

## Non-goals

- Creating an app bundle, cask Ruby file, archive, GitHub artifact, release, tap,
  or Homebrew repository.
- Developer ID signing, notarization, Gatekeeper admission, cask installation,
  Homebrew execution, system mutation, or Apple/GitHub credential access.
- Packaging a real analyzer, scanning repository content, starting a VM, or
  advancing macOS to IAR-1B.
- Defining automatic updates. Homebrew retains its own explicit lifecycle; the
  separately proposed portable updater remains outside this contract.

## Requirements

1. The package is one versioned app bundle with one Homebrew-managed terminal
   link; no component may be installed independently.
2. The embedded layout is closed and rejects path traversal, absolute paths,
   duplicate destinations, mutable download locations, or post-install code.
3. The contract binds the exact release-metadata seal, metadata-set digest, and
   guest release without claiming the prepared guest bytes are distributed.
4. Install and upgrade replace the whole bundle. Mixed-version component sets
   are invalid.
5. Migration fails closed if a preceding formula and the cask would coexist.
6. Uninstall removes only the cask-owned app bundle and CLI link. `zap` is
   intentionally absent so workspaces, caches, receipts, and client settings
   cannot be removed by this contract.
7. No lifecycle action may request root, install a daemon or agent, execute a
   package script, configure a client, or start the VM/analyzer.
8. A source-free receipt must keep assembly, signing, notarization, live
   lifecycle, sealed distribution, production, IAR-1B, and analyzer execution
   false.

## Acceptance criteria

- Closed JSON schemas accept the exact package contract, profile, and receipt
  and reject authority or lifecycle overclaims.
- The checker verifies the exact ADR-0091 seal, canonical embedded paths,
  component roles, one-version rule, cask ownership, migration behavior, and
  uninstall exclusions without network, credentials, process launch, or file
  mutation.
- Fixture provenance proves that every new fixture is original synthetic or
  project metadata and contains no executable, repository, customer, or malware
  content.
- Existing full repository checks pass.

## Later manual and external gates

Unsigned bundle assembly may be proposed only after this contract passes.
Developer ID signing/notarization, a real cask lifecycle on clean supported
Macs, GitHub publication attestation, multi-host and genuine interruption
evidence, complete advisory disposition, and independent human review remain
separate gates.
