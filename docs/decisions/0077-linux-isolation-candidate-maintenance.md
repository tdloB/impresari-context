# ADR-0077: Maintain Linux IAR-1B Candidate Claims With Exact Expiring Evidence

- Status: Accepted
- Date: 2026-08-30
- Deciders: Aaron Boldt

## Context

The Linux source-free composite passed on Ubuntu 24.04 x86_64 and arm64 and on
held-out Ubuntu 22.04 and 26.04 kernel lines. These are valuable candidate
results, but a finite CI corpus is not broad Linux or production support.
Runner images and kernels change, and an indefinitely retained claim would
eventually become misleading.

## Decision

Use one closed, versioned candidate manifest and a source-free, caller-supplied
observation evaluator. Candidate scope is initially limited to the exact Ubuntu
24.04 x86_64 and arm64 targets. Ubuntu 22.04 and 26.04 remain diversity-only
evidence because one is near runner retirement and one is a preview target.

Evidence expires. Any missing evidence, unavailable target, exact identity
change, bound-artifact drift, or expiry withdraws the candidate claim. There is
no weaker fallback and no automatic repair. Every health receipt denies all
authority and fixes production and real-analyzer admission false.

## Consequences

- The project can state precisely which candidate evidence is current.
- Drift and expiry become deterministic, testable withdrawal states.
- Candidate compatibility is not a production-support promise.
- A future production admission needs a separate decision covering supported
  distributions/kernels, installation and privilege topology, continuous
  renewal, release handling, and real-analyzer evidence.
- IAR-2 remains closed.

## Rejected Alternatives

- Treat all passing kernels as broad Linux support: evidence is too finite.
- Keep evidence current indefinitely: runner and kernel drift make this unsafe.
- Discover or repair hosts inside the health checker: that would add authority
  to a source-free compatibility surface.
- Promote diversity-only targets automatically: runner lifecycle and preview
  status require separate support decisions.
