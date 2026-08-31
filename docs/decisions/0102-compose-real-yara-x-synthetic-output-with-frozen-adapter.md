# ADR-0102: Compose Real YARA-X Synthetic Output With The Frozen Adapter

- Status: Implemented; hosted synthetic matrix pending
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive and explicit bounded YARA-X authorization
- Related: ADR-0074, ADR-0098, ADR-0099, ADR-0100, ADR-0101

## Context

ADR-0099 proved that an exact source-built YARA-X v1.20.0 candidate can scan
five Impresari-owned synthetic cases inside the admitted Linux isolation
boundary. Its shell assertions did not pass the real engine output through the
frozen Rust adapter. ADR-0101 separately proved the runner-to-adapter handoff
with an Impresari synthetic emitter but correctly recorded that YARA-X did not
execute. The remaining narrow gap is to compose those two already bounded
paths without admitting the executable, ruleset, or repository scanning.

## Decision

Add a test-only live synthetic coordinator. It invokes the exact ephemeral
YARA-X executable and compiled synthetic rules through the existing Linux
launcher and the single audited Analyzer Runner child-process site. Each of the
five closed generated cases receives a fresh cgroup and an exact read-only job
root. The runner verifies launcher, executable, ruleset, and artifact digests;
captures bounded stdout and the exact confinement preflight in memory; and
returns no raw bytes on failure.

After successful process termination, the coordinator removes the exact job
root and empty cgroup, then passes the captured bytes directly to the ADR-0100
pure adapter. A receipt is emitted only when the normalized rule identifiers
exactly match the closed case expectation and accounting is complete.

The outer receipt may assert only that YARA-X executed over Impresari-owned
synthetic bytes, the process was OS-confined, composition completed in memory,
and cleanup completed. Executable admission, ruleset admission, production,
IAR-2, repository scanning, credential access, uploads, detection quality,
safety, and added authority remain false. The pure adapter result keeps its
own `analyzer_executed=false` field because parser output alone cannot prove
process execution; the authenticated outer receipt carries that distinct fact.

## Consequences

- Real engine output reaches the production-shaped parser without creating a
  production analyzer API.
- Raw output is not written by the new coordinator and is discarded after
  normalization.
- The compatibility executable and rules remain ephemeral, unsigned, and
  unadmitted.
- Repository-derived input and ordinary-host execution remain unavailable.
- A later ADR must separately admit signed artifacts, production manifests,
  coverage policy, and IAR-2.

## Alternatives

- Treat ADR-0099 shell validation as parser evidence: rejected because it did
  not exercise the frozen Rust adapter.
- Treat ADR-0101's emitter as YARA-X execution: rejected because the emitter is
  not an analyzer.
- Open repository scanning now: rejected because artifact/ruleset admission,
  policy, and coverage gates remain incomplete.
- Persist raw output for debugging: rejected because the source-free boundary
  can be proved without retaining analyzer output.

## Activation Gate

This decision authorizes downloading the exact pinned YARA-X v1.20.0 source,
performing the locked ephemeral build, compiling only the Impresari synthetic
ruleset, and executing only the five generated compatibility cases within the
already admitted Linux isolation boundary. It authorizes no repository scan,
credential access, upload, executable or ruleset admission, production claim,
IAR-2 claim, detection-quality claim, or safety claim.

Hosted completion requires the manual empty-workspace workflow to verify the
exact Impresari source archive, exact YARA-X archive and patch, locked feature
graph, dependency review, all five live receipts, complete cleanup, and no
artifact upload. Until that run passes, the implementation state is local and
the hosted evidence remains pending.
