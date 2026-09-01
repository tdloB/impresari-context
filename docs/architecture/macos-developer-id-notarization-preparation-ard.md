# macOS Developer ID And Notarization Preparation Architecture

- Status: Accepted for source-free implementation; live rehearsal gated
- Date: 2026-09-01
- Governing PRD: [macOS Developer ID And Notarization Preparation PRD](../product/macos-developer-id-notarization-preparation-prd.md)
- Governing decision: [ADR-0115](../decisions/0115-freeze-developer-id-and-notarization-before-credential-use.md)

## Boundary

ADR-0114 supplies evidence for one deleted, synthetic-identity, unsigned app.
ADR-0115 freezes how a later operator may rebuild that exact candidate, use an
existing Apple identity and notarization profile by reference, verify the
result, and delete it. The contract itself is offline metadata and carries no
credential or executable.

```text
ADR-0114 exact unsigned candidate
  -> fresh private root and exact rebuild
  -> explicit inside-out Developer ID signing
  -> strict signature and entitlement verification
  -> ditto ZIP
  -> notarytool submit --wait via Keychain profile
  -> inspect notarization log
  -> staple + validate + Gatekeeper assess
  -> recreate final ZIP and record source-free digests
  -> delete complete private root
```

## Credential design

The contract names only two opaque references: a Developer ID Application
identity selector resolved by `codesign` from the current Keychain and a
`notarytool` Keychain profile name. A later operator may pass those references
only after founder authorization. It must never enumerate private keys, print
certificate subjects or email addresses, export identities, copy credentials,
unlock or modify a Keychain, or place secret values in arguments, environment,
logs, fixtures, or receipts.

The receipt may retain a certificate fingerprint and Team Identifier only as
one-way SHA-256 values. It records designated-requirement and CodeDirectory
digests per signed object, not certificate subject text.

## Signing order

The four nested Mach-O objects are signed individually before the outer app.
Hardened runtime and secure timestamps are mandatory. Only the VM controller
receives the frozen `com.apple.security.virtualization` entitlement. Signing
with `--deep` is forbidden; recursive `--deep --strict` is used only for
verification, consistent with Apple’s nested-code guidance.

## Notarization and custody

The app is archived using `ditto`, submitted using `notarytool`, and accepted
only when status is `Accepted`. The notarization log is fetched even after an
accepted result; errors are forbidden and warnings require explicit metadata
disposition. The app is stapled, the ticket is validated, Gatekeeper assesses
the app, and a final archive is created from the stapled app.

All material bytes and raw logs stay under one fresh private root. No produced
app or archive is launched, installed, uploaded anywhere except Apple’s notary
service, retained, published, or placed in Homebrew. The root must be absent
before the metadata-only rehearsal receipt is accepted.

## Later gates

Passing the future rehearsal proves only Developer ID signing and Apple
notarization for the exact synthetic candidate. It does not prove cask
lifecycle, clean-machine Gatekeeper behavior, production metadata, VM
confinement, analyzer execution, production admission, or macOS IAR-1B.
