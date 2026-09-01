# ADR-0108: Assemble A Non-Runnable macOS Bundle Before Signing

- Status: Accepted for source-free implementation; release assembly remains gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Unsigned Synthetic Bundle Assembly PRD](../product/macos-local-vm-unsigned-bundle-assembly-prd.md)
- Architecture: [macOS Local-VM Unsigned Bundle Assembly ARD](../architecture/macos-local-vm-unsigned-bundle-assembly-ard.md)

## Context

The package contract is frozen, but substituting real binaries immediately
would conflate structural assembly with build provenance, executable custody,
Apple signing, and runtime behavior. A smaller checkpoint can prove the exact
tree and cleanup lifecycle without producing runnable or distributable bytes.

## Decision

Permit one offline checker to assemble the closed `.app` tree twice inside
private temporary roots. It may write only generated non-executable synthetic
markers, generated synthetic bundle metadata, and one exact copy of the
ADR-0091 metadata seal. It must validate the canonical tree digest and delete
both roots before success.

The receipt distinguishes `synthetic_app_bundle_assembled: true` from
`release_app_bundle_assembled: false`. Every installation, cask, signing,
notarization, publication, release identity, VM, analyzer, production, IAR-1B,
and authority claim remains false.

## Consequences

- The Option C one-cask structure now has deterministic, cleanup-verified
  filesystem evidence.
- A successful check cannot be mistaken for an installable application because
  all apparent executable destinations are mode `0644` text markers.
- No artifact survives the checker and no release or Apple credential boundary
  is crossed.
- Real release-candidate substitution remains a separate architecture and
  supply-chain decision.

## Alternatives

- Package real unsigned binaries now: rejected because release identity and
  executable provenance are not yet closed for this bundle.
- Create a cask around the synthetic tree: rejected because that would publish
  or install a deliberately non-runnable artifact.
- Move directly to signing/notarization: rejected because structure and
  lifecycle evidence should precede manual credential use.

## Revisit triggers

Revisit before adding executable permission, compiled bytes, a real guest,
archive creation, cask syntax, installation, a public download, Apple signing,
notarization, automatic updates, VM launch, or analyzer execution.
