# ADR-0110: Build And Delete Ephemeral macOS Product Candidates

- Status: Implemented; product-only rehearsal passed, release remains gated
- Date: 2026-09-01
- Decider: Aaron Boldt through the accepted roadmap continuation
- Related PRD: [macOS Local-VM Ephemeral Product Candidate PRD](../product/macos-local-vm-ephemeral-product-candidate-prd.md)
- Architecture: [macOS Local-VM Ephemeral Product Candidate ARD](../architecture/macos-local-vm-ephemeral-product-candidate-ard.md)

## Context

ADR-0109 deliberately stops before compilation. Moving directly to a retained
complete app would combine product build reproducibility, guest custody, app
assembly, Apple identity, cask lifecycle, and runtime confinement. The first
four product units can be rehearsed independently and deleted.

## Decision

Build the exact three Cargo units and one Swift controller from revision
`aca656771f9286b13fbcc046b133ade62b58da2a` twice under distinct private
temporary roots. Use locked offline dependencies, non-incremental Cargo,
private Swift caches, a fixed locale, and the candidate commit timestamp.

Inspect exact output identities without executing them. Record the linker
ad-hoc code identity honestly and keep Developer ID signing false. Run existing
dependency, license, and advisory checks without fetching. Retain only bounded
metadata after deleting every runnable byte, cache, and raw build log.

The product receipt is not the full ADR-0109 unsigned release candidate. Guest,
app, archive, signing, notarization, cask, install, publication, VM, analyzer,
release, production, and macOS IAR-1B claims remain false.

## Observed result

- Two corrected independent builds produced four byte-identical arm64 Mach-O
  outputs each.
- All four outputs were linker ad-hoc signed with no Team Identifier or
  Developer ID signature.
- Cargo Audit found no matching advisory in the exact recorded local database;
  this is not a vulnerability-free claim.
- Cargo Deny passed advisories, bans, licenses, and sources offline, with only
  documented unmatched-license-allowance and duplicate-version warnings.
- Two superseded roots from a corrected source-epoch setup and both accepted
  roots were deleted. No executable or raw log was retained.

## Consequences

- The product build units and one-host byte identities are now concrete rather
  than hypothetical.
- Same-host same-toolchain byte equality is established for this exact
  rehearsal; cross-run, cross-host, and production reproducibility are not.
- The modern linker ad-hoc signature boundary is explicit before Apple signing.
- A complete release still cannot be claimed without the guest, compound
  release identity, app assembly, and later distribution gates.

## Alternatives

- Retain the binaries: rejected because custody, signing, and expiry have not
  been admitted for macOS product artifacts.
- Build the guest and app in the same step: rejected because it would collapse
  independent product, guest, packaging, and signing gates.
- Strip the linker ad-hoc code directories to call the files unsigned:
  rejected because it would mutate the observed compiler output and obscure
  the exact Apple artifact state.
- Treat one-host equality as reproducible release evidence: rejected because
  it says nothing about later runs or independent hosts.

## Revisit triggers

Revisit before retaining runnable macOS artifacts, materializing or substituting
a guest, assembling an app with real bytes, accessing Apple credentials,
Developer ID signing, notarization, cask creation or installation, publication,
VM launch, analyzer execution, production admission, or macOS IAR-1B claims.
