# ADR-0063: Isolate the agent-evaluation harness in a dedicated pull request

- Status: Proposed for sequence-1 review
- Date: 2026-08-28
- Scope: Git, review, and release isolation for the completed developer
  agent-context evaluation harness

## Context

The agent-context evaluation harness was recovered and completed over six local
commits based on refreshed `origin/main`. It now includes the bounded A/B/A
runner, deterministic and provider adapters, source-bound rendering, execution
telemetry, explicit packet policies, evidence prioritization, fixtures,
security tests, and governing records. The complete repository gate passed,
and corrected Anthropic Rust, Ruby, and Shell smoke studies completed.

The next program increment introduces SWE-bench patch generation, disposable
writable workspaces, command execution, Docker grading, and stronger
containment. Combining that work with the proven read-only harness would make
the current review larger and would blur two materially different trust
boundaries.

The maintainer also needs to continue unrelated Impresari Context development
without carrying the evaluation branch. The historical recovery checkout and
crash-associated task must remain untouched until the harness is safely
published.

## Decision

Publish the completed harness as a new, dedicated pull request in the existing
Impresari Context repository before beginning SWE-bench implementation.

"Dedicated" means a new remote branch and a new pull request targeting current
`main`. It does not mean a separate repository, a separate Cargo workspace, or
an amendment to an existing pull request.

Use refreshed `origin/main`, never a stale local `main`, as the ancestry and
diff authority. Preserve the recovery branch as a rollback anchor. When the
remote base remains an ancestor, preserve the six causal harness commits. If
the remote advances, carry only the reviewed series through a fresh worktree;
do not destructively rewrite the recovery checkout.

Keep the two directly required `context-core` and `context-engine` evidence
selection changes in the harness PR and call them out explicitly. Keep
SWE-bench schemas, patch mutation, shell access, Docker orchestration, grader
integration, and paid follow-up studies out of this PR.

Preserve approved source-free smoke evidence privately with hashes before
temporary files expire. Do not commit live records, prompts, answers, packets,
provider bodies, source excerpts, or credentials.

After the new PR and remote head are verified, stop sequence 1. Reassess the
next PRD, ARD, and ADR using the actual extraction and review evidence.

## Consequences

- The completed harness receives a bounded, auditable review surface.
- Other product work can continue independently from the evaluation branch.
- The future writable SWE-bench boundary cannot silently inherit approval from
  the read-only harness.
- The PR is larger than an evaluation-crate-only change because it includes
  direct engine/core behavior, lockfile, SBOM, and security records.
- Preserving causal commits improves review and recovery; a maintainer may
  still choose a squash merge after review.
- The harness remains coupled to internal Context contracts, avoiding a
  premature cross-repository versioning and release problem.
- Private smoke evidence gains durable provenance without entering public Git
  history.

## Alternatives Rejected

### Continue adding SWE-bench work to the recovery branch

Rejected because it mixes read-only question answering with writable patch and
command authority, delays review of completed work, and makes failures harder
to attribute.

### Add the harness to an existing Impresari Context pull request

Rejected because the maintainer explicitly requires a new PR and because the
harness has its own product, security, and evaluation review surface.

### Move the harness to a separate repository now

Rejected because it imports internal Context types and behavior, participates
in the same locked gate, and has not yet established a stable cross-repository
API or release lifecycle.

### Split the core and engine changes into a precursor PR

Rejected for this sequence because those small changes directly govern the
corrected packet behavior already measured by the harness. Splitting them
would make the harness PR depend on an unreviewed behavioral precursor and
would separate ADR-0062 from its implementation. Revisit if maintainer review
finds independent product risk.

### Squash the recovery history before review

Rejected as the default because the six commits separate harness foundation,
provider adapters, provider hardening, renderer design, renderer
implementation, and recovery improvements. Squash-at-merge remains a
maintainer choice after review.

### Commit live study records as proof

Rejected because the repository needs reproducible code and source-free claims,
not transient provider payloads or environment-specific run artifacts. Private
hashed preservation and an honest PR summary are sufficient for sequence 1.

## Security And Privacy

Sequence 1 adds no execution authority. Extraction does not run adapters,
contact providers, read credentials, mutate evaluated source, or publish live
payloads. Existing consent, environment clearing, output bounds, source
containment, and fixed provider endpoints remain governed by ADR-0059 through
ADR-0062.

The remote Git branch and PR are public disclosure boundaries. Only reviewed
source, fixtures, tests, and governance records may cross them. Private smoke
evidence remains outside the source root and public Git history.

## Fitness Checks

- Fresh remote ancestry and ahead/behind evidence.
- Exact candidate path inventory.
- Complete post-extraction `scripts/check.sh` and `git diff --check` success.
- Secret and forbidden-payload scan.
- Private evidence manifest and hashes.
- Clean final worktree.
- New remote branch and new PR targeting `main`.
- Remote head SHA equals the locally reviewed candidate SHA.
- Sequence-1 handoff explicitly excludes SWE-bench implementation.

## Revisit Triggers

- The harness needs an independently versioned public API or release cadence.
- Core/engine review requires an independent precursor change.
- Remote-main conflicts alter architecture rather than applying mechanically.
- Public retention of live evaluation artifacts becomes necessary.
- The future SWE-bench boundary can no longer remain a separate PR sequence.
