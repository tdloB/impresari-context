# ADR-0094: Prefer Windows BaseContainer Without Automatic Host Preparation

- Status: Accepted for no-worker capability implementation
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive

## Context

ADR-0093 implemented the fixed legacy LPAC worker boundary. On GitHub-hosted
Windows Server 2025 build `26100`, the native preflight passed but
`CreateProcessW` returned Win32 error `5` before the worker ran. Cleanup
completed and every confinement claim remained false.

Microsoft's current host-preparation guidance separates privileged one-time
AppContainer+DACL preparation from an unprivileged launcher. Adding that helper
would introduce UAC, system ACL ownership, patch compatibility, uninstall, and
per-boot device preparation into Impresari's ordinary installation boundary.
Microsoft also documents an experimental Windows 11 composable sandbox API,
and its MXC project identifies a newer BaseContainer tier that avoids host DACL
mutation when the OS capability is actually present.

## Decision

Prefer an exact-capability BaseContainer path for the next Windows native
candidate. First freeze and run a read-only, no-worker capability probe on the
independently hosted Windows 11 arm64 image. Do not add automatic elevation,
system-drive or null-device ACL preparation, a privileged service, ordinary
AppContainer downgrade, or Windows Sandbox/VM fallback.

The probe may declare only readiness for a later synthetic rehearsal. It must
keep Windows IAR-1B, OS confinement, production support, and analyzer execution
false.

## Consequences

- Ordinary installations remain no-admin and fail closed on unsupported hosts.
- The first candidate covers fewer Windows versions than a privileged legacy
  fallback would cover.
- The experimental API, specification version, export set, Windows product,
  build, and architecture become explicit compatibility inputs.
- ADR-0093's worker and evidence remain useful historical assets but cannot be
  promoted from the build-26100 unsupported receipt.
- A later BaseContainer worker needs a new protocol/resource contract because
  inherited handles are forbidden and nested Job Object behavior must be
  measured before resume.

## Alternatives

- Add an elevated host-preparation executable: rejected for the first product
  path because it changes durable host authority and release maintenance.
- Weaken LPAC to ordinary AppContainer: rejected because it silently expands
  access to resources granted to all application packages.
- Retry the same build-26100 launch: rejected because the exact host result is
  already attributable and unsupported.
- Package/MSIX activation: deferred; it changes Windows distribution and does
  not by itself prove LPAC-equivalent denial or resource controls.
- Windows Sandbox/Hyper-V VM: retained as a separately gated future fallback;
  it requires an optional feature, supported edition, virtualization, guest
  lifecycle, and a different performance/package contract.

## Revisit Triggers

Revisit before invoking a sandbox export, adding a FlatBuffer dependency,
launching a child, changing protocol transport, composing a Job Object,
changing host ACLs, requesting elevation, enabling an optional feature,
packaging, another Windows target, production support, or a real analyzer.
