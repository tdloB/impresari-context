# IAR-1B Linux Candidate Maintenance Verification

- Date: 2026-08-30
- Scope: ADR-0077 candidate evidence only
- Production admitted: No
- Real analyzer authorized: No

## Bound Evidence

The released manifest binds:

- Ubuntu 24.04 x86_64: PR 131 job `99197119262`, runner image
  `20260823.283.1`, kernel `6.17.0-1022-azure`, Landlock ABI 7.
- Ubuntu 24.04 arm64: PR 132 job `99198568879`, runner image
  `20260823.101.1`, kernel `6.17.0-1022-azure`, Landlock ABI 7.
- Ubuntu 22.04 and 26.04 PR 133 receipts as kernel-diversity-only evidence.
- Exact frozen profile, probe, composite-check, and receipt SHA-256 values.

## Deterministic Verification

`ruby scripts/check-linux-isolation-maintenance.rb` verifies exact repository
bindings and seven cases: the six public states plus a diversity-only target
that must remain unsupported. Every result must deny authority, production
admission, and real-analyzer authorization. The test also rejects malformed
manifests and preserves all inputs.

Schema conformance accepts the closed manifest and compatible receipt and
rejects a production overclaim. The Linux provenance inventory covers every
fixture and records original synthetic origin with no executable, malware,
third-party, private, provider, or network data.

## Claim Boundary

This record maintains a narrow, expiring candidate claim. It does not establish
broad Linux compatibility, a distributable production sandbox, or permission
to execute YARA or any other analyzer. Those remain separate roadmap gates.
