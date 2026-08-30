# IAR-1B macOS Tier A checkpoint

- Date: 2026-08-29
- Decision: ADR-0074 and ADR-0076
- Candidate: App Sandbox/private XPC plus Rust supervision and public resource limits
- Observed host: macOS `26.5.1` (`25F80`) arm64
- Result: Materially incomplete; not IAR-1B-admitted

## Question

Does the selected unprivileged macOS topology confine a compromised synthetic
worker not only from the host, but also from other jobs and from aggregate disk
exhaustion?

## Exact synthetic probes

The ad hoc signed service ran under the frozen production-candidate limits. It
then attempted two bounded, local-only probes using only original synthetic
bytes:

1. It wrote nine separately closed 1 MiB files. All 9 MiB were accepted even
   though each file was individually below the effective 8 MiB per-file limit.
   This proves that limit is not an aggregate ceiling. The files were then
   removed.
2. One service process wrote the exact `synthetic-only` marker into its private
   container and exited. A fresh service process with a different PID read the
   same marker, proving cross-job container persistence, and then removed it.

No repository source, analyzer, credential, real device, external network,
upload, privileged service, production signature, or user file was involved.

## Result

App Sandbox and private XPC continue to deny the already recorded external
filesystem, credential, device, unrelated-process, and network probes. They do
not, however, impose a per-job quota on the service container or create a fresh
container for each job. `RLIMIT_FSIZE` is a per-file limit and cannot be
presented as aggregate-disk confinement.

Therefore this exact topology does not pass the complete IAR-1B threat model.
The evidence remains `partial`, with `os_confined` and `production_admitted`
fixed to false.

Other macOS versions may produce a different exact byte total. The native
matrix accepts only a self-consistent bounded receipt and prints its source-free
total; a passing bound on another host would narrow compatibility rather than
erase the recorded failure on this claimed host.

## Mitigations considered

- A trusted supervisor can terminate the service and purge its exact dedicated
  container after a job. That can improve ordinary cleanup, but it is not an
  OS-enforced per-job container and does not supply a hard aggregate-disk cap
  during execution.
- Polling container size can reduce exposure but has a race-dependent overshoot
  and is not a deterministic hard limit.
- Weakening the disk or cross-job requirements would change the threat model
  and is rejected.
- Per-job users, privileged services, private sandbox APIs, quota-managed
  volumes, or a VM add a second isolation mechanism and materially change the
  selected lightweight, unprivileged design. They remain separate future
  candidates rather than implicit fixes.

## Roadmap consequence

ADR-0076 Option C remains the selected macOS packaging topology: one signed,
notarized cask with CLI compatibility. Packaging topology is independent of an
IAR-1B admission claim.

macOS remains at IAR-1A. Developer ID/notarization and cask publication are not
the next security gate for this candidate because signing cannot correct these
runtime isolation failures. Per the agreed fail-closed roadmap, the next IAR-1B
feasibility work moves to an independently admitted Linux backend while macOS
XPC remains available for defense-in-depth and future reconsideration.
