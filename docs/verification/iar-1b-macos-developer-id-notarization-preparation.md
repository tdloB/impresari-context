# IAR-1B macOS Developer ID And Notarization Preparation

## Scope

ADR-0115 is source-free preparation for one later manual Apple signing and
notarization rehearsal. It validates only committed metadata, schemas, exact
command arrays, opaque credential references, custody rules, and false claims.

## Verification

```sh
ruby scripts/check-macos-vm-developer-id-notarization-preparation.rb
ruby scripts/check-contracts.rb
cargo test -p context-conformance
```

The checker binds the exact ADR-0114 candidate, explicit inside-out signing
order, hardened runtime, secure timestamp, controller entitlement, strict
verification, `ditto` archive, `notarytool --wait`, accepted-log review,
stapling, Gatekeeper assessment, final archive recreation, and whole-root
cleanup. It emits the valid preparation receipt deterministically.

## Non-claim

This checkpoint does not access a Keychain identity or notarization profile,
run a process, contact Apple, rebuild the candidate, sign, notarize, staple,
create an archive or cask, install, publish, launch a VM, execute an analyzer,
admit production, or admit macOS IAR-1B.
