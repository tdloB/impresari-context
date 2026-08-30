# ADR-0076: Package the macOS hybrid XPC boundary as one CLI-compatible cask

- Status: Accepted; staged implementation, production publication gated
- Date: 2026-08-29
- Related PRD: [macOS hybrid XPC distribution PRD](../product/macos-hybrid-xpc-distribution-prd.md)
- Related architecture: [macOS hybrid XPC distribution ARD](../architecture/macos-hybrid-xpc-distribution-ard.md)
- Related security decision: [ADR-0074](0074-separate-isolated-analyzer-runner.md)

## Context

ADR-0074 requires a macOS OS-confinement backend before a real analyzer may
run. The selected candidate combines a sandboxed private XPC service with the
Rust supervisor and public OS resource limits. Because XPC services are nested
signed code inside an application bundle, independently packaging the CLI and
service would introduce mixed-version, signature, upgrade, and removal states.

## Decision

Target one Developer ID-signed, hardened, notarized application bundle and one
Homebrew cask with CLI compatibility.

- Embed the Rust supervisor, background host, and private XPC service in one
  versioned sealed bundle.
- Keep `impresari-context` as the user-facing terminal command by linking or
  wrapping the supported embedded entry point without modifying the bundle.
- Distribute outside the Mac App Store; no graphical workflow is required.
- Let Homebrew own install, explicit upgrade, rollback, and uninstall.
- Require explicit migration from the preceding formula package.
- Keep Linux on a formula and admit its confinement backend independently.
- Do not add a persistent service, privileged helper, private sandbox API,
  self-updater, or VM to this decision.

## Consequences

- All security-sensitive macOS executables remain version-aligned and covered
  by one nested-signing and notarization chain.
- macOS installation changes from a formula-shaped artifact to a cask-shaped
  artifact once the admitted release exists.
- Release CI gains macOS bundle construction, signing, notarization, cask, and
  clean-machine testing obligations.
- Existing releases and `v0.1.0` claims do not change.
- The selected topology is not a supported distribution claim until the
  remaining IAR-1B and packaging gates pass.

## Alternatives considered

- **Plain cask without CLI compatibility:** rejected because it degrades the
  primary terminal experience.
- **Formula plus separately installed signed helper:** retained only as a
  fallback if the embedded CLI topology fails; it creates two update and
  partial-uninstall domains.
- **Privileged or persistent service package:** rejected for unnecessary
  authority and lifecycle complexity.
- **VM or remote sandbox:** deferred; they conflict with the current local,
  lightweight roadmap and are unnecessary for this feasibility stage.

## Review triggers

Reopen the decision if Homebrew cannot expose the embedded CLI without
invalidating signatures, Gatekeeper rejects the independently distributed
App Sandbox/XPC topology, a supported macOS release removes a required public
resource primitive, or the full Tier A corpus demonstrates an escape.

## 2026-08-29 Tier A review

The review trigger fired: bounded synthetic probes demonstrated an aggregate-
disk escape and cross-job persistence in the selected XPC runtime. This does
not invalidate the one-cask/CLI-compatible packaging decision; it does prevent
that package from being represented as an IAR-1B backend. Production signing
and publication for the security claim are deferred while Linux advances as
the next independent IAR-1B candidate. Any future macOS confinement layer must
re-enter this ADR before publication.
