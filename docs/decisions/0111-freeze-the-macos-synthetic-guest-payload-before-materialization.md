# ADR-0111: Freeze The macOS Synthetic Guest Payload Before Materialization

- Status: Implemented; source-free contract only
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Synthetic Guest Payload Contract PRD](../product/macos-local-vm-synthetic-guest-payload-contract-prd.md)
- Architecture: [macOS Local-VM Synthetic Guest Payload Contract ARD](../architecture/macos-local-vm-synthetic-guest-payload-contract-ard.md)

## Context

ADR-0110 leaves the guest role unmaterialized. The current synthetic guest
manifest names six artifacts because it describes both build provenance and
two runtime test paths. Packaging all six would expose intermediates and make
the ordinary release payload larger and less precise than the controller
requires.

## Decision

Define the ordinary synthetic guest payload as exactly two mode-`0644` regular
files: `Image` and `impresari-initramfs.gz`. Bind them to the exact ADR-0091
guest manifest and metadata seal and to the exact ordinary controller names,
bytes, and SHA-256 identities.

Freeze, but do not execute, a later materialization recipe using the exact
publisher-authenticated Alpine `linux-virt-6.18.48-r0.apk`, exact extracted
kernel and `virtio_blk` module, Impresari-owned `GuestInit/main.c`, Zig 0.16.0
for `aarch64-linux-musl`, and the canonical Ruby initramfs builder. A later
implementation must use and delete a fresh private root.

The contract is source-free. It performs no download, build, artifact
retention, app assembly, signing, notarization, cask lifecycle, VM launch,
analyzer execution, production admission, or macOS IAR-1B admission.

## Consequences

- The cask guest role now has a minimal deterministic runtime layout.
- Resource-canary and standalone build artifacts remain explicit but excluded.
- Upstream authentication and build provenance cannot be substituted silently.
- A later guest materialization can be evaluated independently from product,
  app, Apple-identity, distribution, and runtime gates.

## Alternatives

- Package all six guest components: rejected because four are intermediates or
  test-only resources and would expand custody and attack surface.
- Leave the directory unspecified: rejected because an open payload root
  prevents deterministic assembly and permits unreviewed members.
- Materialize the guest in this decision: rejected because contract review and
  executable custody are independent gates.

## Revisit triggers

Revisit before downloading or materializing the guest, changing the guest or
controller names, adding a shell, package manager, network, persistent storage,
real analyzer, app assembly, Apple credential access, signing, notarization,
cask installation/publication, VM launch, production, or macOS IAR-1B claims.
