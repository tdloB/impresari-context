# Impresari Context — macOS hybrid XPC distribution PRD

- Status: Accepted for staged implementation; publication remains gated
- Date: 2026-08-29
- Authority: Founder-selected macOS IAR-1B and packaging direction
- Architecture: [macOS hybrid XPC distribution ARD](../architecture/macos-hybrid-xpc-distribution-ard.md)
- Decision: [ADR-0076](../decisions/0076-macos-hybrid-xpc-cask.md)

## Objective

Deliver the admitted macOS analyzer boundary as one independently distributed,
Developer ID-signed and notarized bundle containing the Rust supervisor,
sandboxed host, and private XPC service, while preserving the ordinary
`impresari-context` terminal command through one Homebrew cask.

## User outcome

After an eventual supported release, a macOS user installs one package and
continues to invoke `impresari-context` from the terminal. The user does not
install, start, approve, update, or remove a second helper package or persistent
background service.

## Scope

- One versioned `.app` bundle with a background-only host, Rust CLI/supervisor,
  and private XPC service under `Contents/XPCServices`.
- Nested Developer ID signing, hardened runtime, App Sandbox entitlements, and
  notarization for distribution outside the Mac App Store.
- One Homebrew cask that installs the intact bundle and creates a CLI link to
  the supported embedded entry point.
- Atomic version alignment across the supervisor, host, XPC service, schemas,
  and resource profile.
- Explicit migration from any preceding formula installation; never leave both
  package forms active silently.
- Homebrew-owned explicit upgrade, rollback, and uninstall behavior.
- Linux remains eligible for the existing formula architecture; this decision
  does not impose an app bundle on non-macOS platforms.

## Non-goals

- Mac App Store publication, a graphical product requirement, an internal
  self-updater, a persistent LaunchAgent, a privileged helper/LaunchDaemon,
  private Seatbelt profiles, a system extension, or a VM.
- Modifying the signed bundle after notarization, downloading analyzer code at
  install time, automatic client configuration, repository access during
  installation, or use of signing credentials during ordinary execution.
- Retrofitting the architecture or claims onto `v0.1.0`.

## Acceptance criteria

- A clean cask install passes Gatekeeper and preserves every nested signature.
- The CLI link invokes the exact bundled supervisor without copying or mutating
  signed executable bytes.
- The supervisor validates the embedded host and XPC identities before work.
- The source-free launch handshake contains no repository path, arbitrary
  argument, environment, credential, endpoint, or analyzer-execution grant.
- The exact committed resource-profile bytes match the Rust and native
  effective-limit identities.
- A cask upgrade cannot produce a mixed-version component set.
- Formula-to-cask migration detects and resolves conflicts explicitly.
- Uninstall removes all executable components; `zap` behavior is separately
  reviewable and never removes user workspaces or unrelated configuration.
- The complete IAR-1B synthetic matrix passes on every claimed macOS target.
- Release automation can sign and notarize without printing, exporting, or
  placing Developer ID credentials in source, artifacts, logs, or caches.

## Publication gate

The design is accepted, but no supported cask claim or release publication is
allowed until IAR-1B admission, production nested signing/notarization, clean-
machine installation, migration, upgrade, rollback, and uninstall evidence are
all recorded. Any Apple Developer or Homebrew action requiring owner-controlled
credentials remains a manual release boundary.
