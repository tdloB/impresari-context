# YARA-X Live Synthetic Envelope Evidence

- Date: 2026-08-31
- Decision: [ADR-0102](../decisions/0102-compose-real-yara-x-synthetic-output-with-frozen-adapter.md)
- Profile: `yara-x-live-synthetic-envelope-v1`
- Profile SHA-256: `b95bbe55b604c7e266bb620981b8e5c3fca052c22842222537b1b0effed7bbf0`
- State: implemented; hosted synthetic matrix pending

## Implemented Boundary

The test-only coordinator uses the single audited Analyzer Runner process site
to invoke an exact ephemeral YARA-X v1.20.0 executable through the admitted
Linux cgroup-v2, Landlock, and seccomp launcher. Executable, compiled-rules,
launcher, and generated-artifact identities are verified before launch. Stdout
is bounded to 131,072 bytes, stderr must equal the 203-byte confinement
preflight, elapsed time is bounded to ten seconds, and each case receives a
fresh cgroup limited to four tasks and 512 MiB.

Complete stdout stays in Rust memory and is passed directly to the pure
ADR-0100 adapter after exact job and cgroup cleanup. The resulting path-free
normalized rule identifiers must equal the frozen expectation for all five
generated cases.

## Local Evidence

- Unit tests cover successful real-engine capture composition, identity drift,
  case mismatch, cleanup failure, and overclaim rejection without launching an
  analyzer locally.
- Registry schemas close the profile, control, and receipt.
- Digest-bound fixture provenance contains no YARA-X binary, third-party
  content, malware, repository source, credential, or network capture.
- Boundary checks preserve exactly one production Rust child-process site.
- The manual workflow builds both the pinned YARA-X candidate with Rust 1.93.0
  and the locked static coordinator with workspace Rust 1.98.0.

## Hosted Evidence

Pending the manual empty-workspace Ubuntu 24.04 workflow run for the merged
source commit. No hosted-pass claim is made in advance.

## Non-Claims

No executable or ruleset is admitted, signed, published, cached, uploaded, or
retained. No repository content or credential is read by the analyzer. This
checkpoint does not open production use or IAR-2 and makes no detection,
safety, or malware-free claim.
