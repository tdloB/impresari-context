# YARA-X Live Synthetic Envelope Evidence

- Date: 2026-08-31
- Decision: [ADR-0102](../decisions/0102-compose-real-yara-x-synthetic-output-with-frozen-adapter.md)
- Profile: `yara-x-live-synthetic-envelope-v1`
- Profile SHA-256: `2aa5e203f71089688baa41556c6775e7dcca98c7e6aab726442ff99fb5f8cd26`
- State: hosted synthetic candidate passed; production and IAR-2 gated

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

- Workflow source commit: `04228fbfa1babfcf3bdba71dcba4cacaff006c40`.
- Immutable Impresari source root:
  `3b74648cbdf78453dcd71ab34da1ecd876093862`.
- Impresari source archive: 27,932,606 bytes, SHA-256
  `974b5d43512fa8be6d49c75277bc90d4b1b89eaf0f45b73a3363099e7f249d90`.
- Successful manual run:
  [GitHub Actions 33432469614](https://github.com/tdloB/impresari-context/actions/runs/33432469614),
  job `99620875408`.
- GitHub-hosted runner `2.337.0`; Ubuntu `24.04.4`; image
  `ubuntu-24.04` version `20260823.283.1`; kernel
  `6.17.0-1022-azure`; x86-64; Landlock ABI 7.
- YARA-X v1.20.0 executable SHA-256:
  `9e7424e62b714ee7b7be9fb0b67367b12209a9ec6967ff8c9b4c4959d6a17549`.
- Compiled Impresari synthetic-rules SHA-256:
  `f92cda545be5514258a1d64f721522afbc960630212e2ddea50a3430847f86f0`.
- All five generated cases passed the real-engine, in-memory adapter
  composition. The receipt reported `os_confined=true`,
  `yara_x_executed=true`, `production_admitted=false`, and `iar_2=false`.
- The separate cleanup step verified that the disposable build root, live
  coordinator target, YARA-X compatibility target, executable, and compiled
  rules were absent. The workflow uploaded no artifact.

### Authorized revalidation

Founder authorization on 2026-08-31 permitted the same exact pinned source
build and Impresari-owned synthetic corpus inside the already admitted
isolation boundary while expressly prohibiting repository scans, credential
access, uploads, and production admission.

- Dispatch commit: `764b7f43223b7a846262e555b0cec73989c4fd67`.
- Successful manual run:
  [GitHub Actions 33439251952](https://github.com/tdloB/impresari-context/actions/runs/33439251952),
  job `99643176845`, completed in 7 minutes 21 seconds.
- The run reused immutable Impresari source root
  `3b74648cbdf78453dcd71ab34da1ecd876093862` and the same 27,932,606-byte
  archive with SHA-256
  `974b5d43512fa8be6d49c75277bc90d4b1b89eaf0f45b73a3363099e7f249d90`.
- GitHub-hosted runner `2.337.0`; Ubuntu `24.04.4`; image
  `ubuntu-24.04` version `20260823.283.1`; kernel
  `6.17.0-1022-azure`; x86-64; Landlock ABI 7.
- YARA-X v1.20.0 executable SHA-256:
  `b75b3fdd1133424cb3c9fea10d7eed7e7327b33cd260bf3afda490f5ef539721`.
- Compiled Impresari synthetic-rules SHA-256:
  `f92cda545be5514258a1d64f721522afbc960630212e2ddea50a3430847f86f0`.
- All five cases again reported `os_confined=true`,
  `yara_x_executed=true`, `production_admitted=false`, and `iar_2=false`.
- Mandatory cleanup passed. The GitHub artifacts API returned zero artifacts
  for the run.

The executable digest differs from run `33432469614` despite the same pinned
source root, patch, Rust versions, runner image version, kernel, architecture,
and rules identity. Compatibility therefore remains proven, but byte-for-byte
reproducibility is not. The cause has not been established and must be resolved
or explicitly dispositioned before any production engine admission.

## Non-Claims

No executable or ruleset is admitted, signed, published, cached, uploaded, or
retained. No repository content or credential is read by the analyzer. This
checkpoint does not open production use or IAR-2 and makes no detection,
safety, or malware-free claim.
