# IAR-1B macOS Hybrid Resource And Lifecycle Decision

- Date: 2026-08-29
- Decision: ADR-0074
- Candidate: App Sandbox host and private XPC service supervised by the Rust
  control plane
- Result: Selected for continued feasibility; partial and not production-admitted

## Question

Can a private App Sandbox XPC service supply the native access boundary while
the existing Rust supervisor supplies exact job identity, wall-time control,
termination, and cleanup, without a privileged daemon, private API, or VM?

## Corrected decomposition

The earlier review treated the private XPC service as though it had to provide
access control, hard resource limits, lifecycle supervision, and packaging by
itself. The selected design instead assigns one bounded responsibility to each
layer:

- App Sandbox and the private XPC service deny undeclared filesystem, process,
  credential, device, and network capabilities;
- the trusted XPC harness applies irreversible per-process limits before
  invoking any analyzer code in-process;
- the Rust supervisor validates the prepared service identity and enforces the
  wall deadline, exact termination target, result contract, and cleanup; and
- one signed/notarized macOS app bundle keeps the supervisor, host, and XPC
  service at one version and is distributed as a Homebrew cask with a CLI
  compatibility link under ADR-0076.

No analyzer receives a shell or child-process surface. No persistent service,
LaunchAgent, privileged helper, private sandbox profile, or VM is selected.

## Synthetic feasibility result

The second native prototype demonstrated, on the recorded macOS arm64 host:

- `RLIMIT_CPU` applied before synthetic work and terminated an intentional CPU
  loop after its one-second limit;
- an `RLIMIT_AS` hard limit derived from the service's current virtual size
  denied a one-gibibyte `mmap` beyond the admitted 128-MiB growth allowance;
- `RLIMIT_NPROC=0` denied both `fork` and `posix_spawn` from the service;
- the service published its PID before work, and the supervisor verified the
  exact embedded executable path before terminating a deliberately hung job;
- CPU termination invalidated the connection and a subsequent request launched
  a distinct service process successfully;
- a bounded synthetic payload was written, re-read, removed, and verified
  absent from the service's temporary container;
- an exact synthetic pseudo-terminal character device was present and denied
  to the sandboxed service; and
- the earlier App Sandbox filesystem, credential, unrelated-process, and live
  loopback-network denials continued to pass.

The follow-up froze the exact `iar-macos-xpc-hybrid-v1` profile and a closed,
source-free Rust-to-host preparation handshake. Native evidence verifies the
profile's effective CPU, address-space-growth, process-count, descriptor, and
file-size limits. Rust/schema evidence rejects any repository path, arbitrary
argument, caller environment, credential, network authority, analyzer
execution, mismatched identity, partial readiness, retained source, or
premature confinement/production claim.

The fixed 512-MiB attempt from the first review returned `EINVAL` because it was
below the service's existing virtual footprint. Deriving the irreversible hard
limit from the observed startup footprint plus a frozen growth allowance was
accepted by the kernel and denied growth beyond that allowance. The evidence
therefore supports bounded address-space expansion, not a claim that resident
memory will equal one fixed number on every OS release.

## Remaining gates

The candidate remains `partial`, with `os_confined` and
`production_admitted` fixed to `false`, until all of the following pass:

- Developer ID nested signing and notarization without exposing credentials;
- the selected one-cask/CLI-compatible installation, upgrade, rollback,
  migration, and uninstall lifecycle;
- a clean-machine Gatekeeper rehearsal;
- the complete Tier A escape and mutation corpus; and
- evidence on every macOS version and architecture the release claims.

No real analyzer, repository artifact, production signing credential,
notarization submission, release upload, or Homebrew publication was used.

## Packaging consequence

ADR-0076 selects one cask with CLI compatibility as the target topology. That
selection is architectural, not a publication claim. The cask remains
unimplemented until this synthetic candidate passes the remaining admission
gates. Existing `v0.1.0` artifacts are unchanged and no retroactive claim is
made for them.

## Reproduction

```sh
./scripts/check-macos-xpc-feasibility.sh
cargo test -p context-conformance --test schema_conformance --locked
```

The first command builds only ad hoc signed synthetic artifacts beneath
`target/iar-macos-xpc-feasibility`. It deliberately creates no repository
input, analyzer, network authority, production identity, or installable cask.
