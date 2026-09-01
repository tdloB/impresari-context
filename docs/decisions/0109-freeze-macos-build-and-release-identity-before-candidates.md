# ADR-0109: Freeze macOS Build And Release Identity Before Candidates

- Status: Accepted for source-free implementation; executable candidates remain gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Build And Release-Identity Contract PRD](../product/macos-local-vm-release-identity-contract-prd.md)
- Architecture: [macOS Local-VM Build And Release-Identity Contract ARD](../architecture/macos-local-vm-release-identity-contract-ard.md)

## Context

ADR-0108 proved deterministic app structure using non-runnable markers. Real
candidate substitution would otherwise combine source selection, Rust and
Apple toolchains, artifact identity, guest identity, license and vulnerability
evidence, rollback, and eventual signing in one hard-to-review step.

## Decision

Freeze one source-free contract for the five material app roles. Bind the exact
contract baseline, product version, direct build-control input set, build-unit
commands, target, ADR-0107 package identity, ADR-0091 guest identity, mandatory
future candidate record, and rollback semantics.

The current receipt may say only that the contract is frozen. It must keep
candidate materialization, release identity, bundle assembly, signing,
notarization, installation, publication, VM launch, analyzer execution,
production, macOS IAR-1B, and authority false.

## Consequences

- Future real bytes have a closed admission record instead of ad hoc filenames.
- The contract baseline cannot be confused with the later candidate source
  revision or compound artifact identity.
- Product SBOM, vulnerability, reproducibility, and Apple build-host evidence
  become mandatory before substitution rather than post-build cleanup work.
- No executable, package, credential, network call, or runtime authority is
  introduced.

## Alternatives

- Build and inventory real binaries now: rejected because it crosses artifact
  custody and build-environment boundaries before their record is frozen.
- Pin this contract's future merge commit as the release revision: rejected as
  self-referential and misleading; a later candidate supplies its exact source.
- Reuse only the guest metadata seal: rejected because it does not identify the
  four product executables or their build environment.

## Revisit triggers

Revisit before compiling or retaining candidates, changing a build unit,
updating source/version/toolchains/guest release, producing an archive,
accessing Apple credentials, signing, notarizing, creating or installing a
cask, publishing, launching a VM, or executing an analyzer.
