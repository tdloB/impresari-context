# IAR-1B Linux External Production-Support Admission

- Date: 2026-08-30
- Decision: ADR-0082
- Profile: `externally_managed`
- Current state: `release_pending`
- Production admitted: No
- Real analyzer authorized: No

## Verification

Run:

```sh
ruby scripts/check-linux-external-production-support-admission.rb
```

The check reproduces pending-release, stale, changed, missing, unsupported, and
unavailable results; verifies that each withdraws production support; rejects a
changed manifest identity; and confirms that every authority remains denied.
The conformance suite accepts the pending manifest and receipt and rejects a
pending-release production overclaim.

## Release boundary

The evidence source commit is
`8f8f9adb5d99f373fbd6456564dfa6233c37bc34`, but the only published release is
v0.1.0 from an older source. The exact candidate archive and lifecycle receipt
therefore remain evidence, not a supported release. A new version and immutable
tag must be published and bound in a reviewed follow-up; v0.1.0 must not be
retagged or overwritten.
