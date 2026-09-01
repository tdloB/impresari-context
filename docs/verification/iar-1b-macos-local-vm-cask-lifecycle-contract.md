# IAR-1B macOS Local-VM Cask Lifecycle Contract Checkpoint

- Status: Source-free cask contract passed; app assembly and distribution remain open
- Date: 2026-09-01
- Decision: [ADR-0107](../decisions/0107-freeze-macos-local-vm-cask-lifecycle-before-signing.md)
- Profile: `iar-macos-local-vm-cask-lifecycle-v1`

## Result

The selected one-cask, CLI-compatible macOS distribution now has a closed
source-free ownership and lifecycle contract. It fixes one app bundle, one
public CLI link, four exact executable roles, one closed guest payload root,
and the embedded ADR-0091 metadata-seal role.

The contract digest is:

`sha256:4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676`

The profile digest is:

`sha256:1373511a5ed419337df562bd66a9bfd57441bd58c5ae9f0d0d9333fc64fb5213`

## Lifecycle boundary

Install, upgrade, and rollback operate on the whole app bundle. Migration
rejects formula/cask coexistence before mutation. Uninstall owns only the app
bundle and CLI link. Package scripts, `zap`, internal helper links, privileged
helpers, daemons, agents, login items, and automatic-update authority are all
absent.

## Evidence

The deterministic checker verifies the exact package contract and profile,
the ADR-0091 release-metadata seal and guest release, the path-sorted component
inventory, app-relative paths, role bindings, one public entrypoint, exact
Homebrew ownership, and all lifecycle nonclaims. The fixtures are original
synthetic or project metadata and contain no executable, malware, repository,
private, or customer content.

## Boundary

This checkpoint did not assemble an app, write a cask, run Homebrew, install or
remove files, access credentials or a network, sign, notarize, publish, boot a
VM, or execute an analyzer. Its receipt therefore keeps app assembly,
publication attestation, Developer ID signing, notarization, live cask
lifecycle, sealed distribution, production, macOS IAR-1B, analyzer execution,
and added authority false.

## Next gate

The next reversible checkpoint may assemble one unsigned, synthetic-only app
bundle and validate the exact byte layout without installing it. Live Apple
signing/notarization, clean-machine cask lifecycle, multi-host and genuine
interruption evidence, complete advisory disposition, independent human
review, production, and real analyzers remain separately gated.
