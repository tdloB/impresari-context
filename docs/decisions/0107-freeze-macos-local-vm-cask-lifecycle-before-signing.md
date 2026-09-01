# ADR-0107: Freeze The macOS Local-VM Cask Lifecycle Before Signing

- Status: Accepted for source-free implementation; live distribution remains gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Cask Lifecycle Contract PRD](../product/macos-local-vm-cask-lifecycle-prd.md)
- Architecture: [macOS Local-VM Cask Lifecycle Contract ARD](../architecture/macos-local-vm-cask-lifecycle-contract-ard.md)

## Context

The active local-VM guest metadata is sealed, but the selected one-cask user
experience still lacks an exact ownership and lifecycle contract. Creating an
app bundle first would let packaging details become de facto policy, while
signing or notarizing first would cross a manual credential boundary without a
closed install-through-uninstall design.

## Decision

Freeze one source-free, exact package contract before app assembly. It defines
one `Impresari Context.app`, one Homebrew-managed `impresari-context` CLI link,
a closed embedded component-role layout, whole-bundle version alignment,
formula-conflict rejection, and exact cask-owned uninstall scope.

The deterministic evaluator may claim only that the contract is frozen. It
must keep app assembly, publication attestation, Developer ID signing,
notarization, live cask lifecycle, sealed distribution, production admission,
macOS IAR-1B, and analyzer execution false.

## Consequences

- Later packaging has an exact reviewable target and cannot silently add a
  daemon, privileged helper, package script, extra link, or broad uninstall.
- The cask remains CLI-compatible without exposing internal helpers as public
  commands.
- Whole-bundle replacement prevents mixed supervisor/controller/guest versions.
- This checkpoint produces no installable software and does not reduce any
  remaining Apple, Homebrew, security-review, or runtime gate.

## Alternatives

- Assemble an unsigned bundle before freezing policy: rejected as premature and
  harder to review.
- Move directly to Developer ID signing/notarization: rejected because it needs
  manual credential custody and live external evidence.
- Use a formula plus separately installed helper bundle: rejected because the
  founder selected one cask with one version and uninstall lifecycle.

## Revisit triggers

Revisit before exposing another command, adding a GUI, installing background or
privileged services, changing guest delivery, supporting Intel Macs, enabling
automatic updates, adding a package script, or allowing Homebrew `zap` to
remove user-owned data.
