# macOS Developer ID And Notarization Preparation PRD

- Status: Accepted for source-free implementation; credential use remains manual
- Date: 2026-09-01
- Architecture: [macOS Developer ID And Notarization Preparation ARD](../architecture/macos-developer-id-notarization-preparation-ard.md)
- Decision: [ADR-0115](../decisions/0115-freeze-developer-id-and-notarization-before-credential-use.md)

## Objective

Turn the ADR-0114 unsigned-candidate evidence into one exact, reviewable manual
signing and notarization rehearsal contract without accessing an Apple identity,
creating a signed artifact, or contacting Apple.

## User outcome

When the founder elects to cross the Apple credential boundary, the operator
will use an already configured Developer ID Application identity and
`notarytool` Keychain profile in place. No secret value is copied into source,
arguments, environment, logs, fixtures, artifacts, or receipts.

## Requirements

1. Bind the exact ADR-0114 candidate record and synthetic app identity.
2. Sign each nested Mach-O explicitly from the innermost object outward; never
   use `codesign --deep` for signing.
3. Require hardened runtime, secure timestamps, the exact controller
   entitlement, consistent Team Identifier, and strict recursive verification.
4. Submit one `ditto` ZIP through `notarytool --wait` using only a Keychain
   profile reference; inspect the returned log even when accepted.
5. Staple and validate the app, assess it with Gatekeeper, then recreate the
   final archive from the stapled app.
6. Keep the signed app, archive, and raw notarization log inside one fresh
   private root and delete the whole root before accepting metadata.
7. Keep cask creation, installation, publication, VM launch, analyzer
   execution, production, and macOS IAR-1B outside this checkpoint.

## Acceptance

The source-free checker must reproduce an exact preparation receipt and reject
credential values, ambiguous identity selection, deep signing, missing
timestamp or hardened runtime, incomplete signing order, unreviewed notary
logs, retained artifacts, or any later-gate claim.

## Non-goals

No credential discovery or mutation, signing, network access, notarization,
artifact retention, cask lifecycle, publication, VM launch, analyzer execution,
production admission, or macOS IAR-1B admission occurs in this implementation.
