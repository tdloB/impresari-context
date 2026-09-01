# ADR-0115: Freeze Developer ID And Notarization Before Credential Use

- Status: Accepted for source-free implementation; live credential use remains manual
- Date: 2026-09-01
- Decider: Aaron Boldt through the active roadmap continuation
- Related PRD: [macOS Developer ID And Notarization Preparation PRD](../product/macos-developer-id-notarization-preparation-prd.md)
- Architecture: [macOS Developer ID And Notarization Preparation ARD](../architecture/macos-developer-id-notarization-preparation-ard.md)

## Context

ADR-0114 proved that the exact unsigned synthetic candidate can exist under one
private root and be completely deleted. Moving directly to live signing would
combine credential selection, nested signing, notarization transport, ticket
stapling, Gatekeeper checks, evidence retention, and cleanup in an unfrozen
manual procedure.

## Decision

Freeze a source-free contract for one later manual signing and notarization
rehearsal. Bind the exact ADR-0114 candidate, inside-out code-object order,
hardened-runtime and timestamp requirements, controller entitlement, strict
verification, `ditto` archive, `notarytool` Keychain-profile submission, log
inspection, stapling, Gatekeeper assessment, final archive, metadata-only
receipt, and whole-root deletion.

The current implementation must keep network, credential access, process
launch, signing, notarization, artifact retention, installation, publication,
VM, analyzer, production, macOS IAR-1B, and authority false.

## Consequences

- The credential boundary becomes an exact manual input rather than an ad hoc
  release step.
- Secret values and attributable certificate subject data cannot enter source,
  fixtures, logs, artifacts, or receipts.
- A future accepted rehearsal can prove signing and notarization without
  retaining or distributing its synthetic candidate.
- Homebrew cask and production metadata remain separate later gates.

## Alternatives

- Sign immediately: rejected because the credential and retention boundary was
  not frozen.
- Use ad-hoc signing: rejected because it cannot prove Developer ID or Apple
  notarization.
- Use `codesign --deep` for signing: rejected because Apple recommends explicit
  inside-out signing of nested code.
- Retain the notarized app for cask work: rejected because distribution custody
  and expiry are not admitted by this checkpoint.

## Revisit triggers

Revisit before changing candidate identity, signing order, entitlement,
credential mechanism, archive format, notary transport, retained metadata,
artifact custody, installation, publication, VM launch, analyzer execution, or
any production or macOS IAR-1B claim.
