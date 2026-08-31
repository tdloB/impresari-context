# IAR-1B macOS Local-VM Synthetic Fault-Matrix Evidence

- Status: Partial synthetic matrix passed; full IAR-1B remains pending
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Profile: `iar-macos-local-vm-synthetic-matrix-v1`
- Profile SHA-256: `a411dc8d896b9b516cb535786fe2d12f17c6bfed3b39b2104c040e7556507522`
- Prior checkpoint: [storage and cross-job feasibility](iar-1b-macos-local-vm-feasibility.md)

## Scope

This checkpoint extends the synthetic-only local Linux VM prototype on the same
macOS `26.5.1` arm64 host. It runs no analyzer, repository content, hostile
artifact, external service, credential, host directory share, or guest network
device. Every receipt remains source-free and retains `vm_confined=false`,
`production_admitted=false`, `analyzer_execution=false`, and
`authority_added=false`.

## Identity And Reproducibility Correction

The first matrix run exposed that Ruby's gzip writer substitutes the current
time when `mtime` is zero. The initramfs builder now uses the fixed nonzero
timestamp `1`; two consecutive builds from the same exact inputs produced the
same 38,208-byte initramfs:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| Static synthetic guest init | 37,616 | `68d5be977b2bd1bc7df2bcfc8bdb077bb03f9afc390d7c099f23437ced1598bf` |
| Reproducible synthetic initramfs | 38,208 | `cc87a9a68d06826277dd759befd318272a7876540b4287cfd6fe0ac67552bfbf` |
| Source-free composite receipt | generated | `8058e880f7c80a5927d8d5fde2327274f6ce91569dd682890974079c4f26c5e3` |

The controller now freezes the initramfs digest internally. A caller cannot
select an alternate guest merely by supplying a different digest. A modified
initramfs was rejected before any job directory was staged.

The locally ad hoc signed controller is deliberately not assigned a stable
artifact identity here: macOS signing metadata can change its bytes on each
build. Production admission will require a reproducible release process and an
attributable Developer ID identity rather than treating an ad hoc build hash as
a durable security identity.

## Passed Cases

The native matrix passed ten exact behaviors plus a post-fault recovery job:

- Two ordinary fresh jobs and one recovery job returned exact valid receipts.
- A malformed guest result was rejected and its job removed.
- A 128 KiB synthetic serial flood was drained into a memory-only capture,
  retained at most 65,536 bytes, rejected, and removed without a serial file.
- A non-terminating guest hit the two-second fault deadline; the whole VM was
  stopped and the job removed.
- A guest containing a synthetic forked descendant hit the same whole-VM stop
  path; the job and descendant VM state were removed.
- An early guest exit without a result failed closed and was removed.
- A deterministic controller cancellation request stopped the VM, returned
  `cancelled`, and removed the job.
- The earlier hard scratch capacity and cross-job-cleanliness results continued
  to pass.

## Still Open

This checkpoint intentionally does not claim:

- cancellation delivered by the future Rust supervisor or external POSIX
  signals;
- recovery after forced host-controller termination, host sleep, or host
  interruption;
- guest memory-pressure and CPU-accounting outcomes;
- the complete host home, repository, cache, credential, device, process, local
  network, and Internet canary corpus;
- multi-host, Developer ID, notarization, packaging, update, or independent
  review evidence; or
- production admission or any real analyzer.

The next checkpoint is the remaining resource, host-canary, and external
supervisor lifecycle work. macOS remains publicly at IAR-1A.
