# Dedicated Agent-Evaluation PR Extraction — Architecture Requirements And Design

- Status: Accepted for sequence-1 implementation
- Date: 2026-08-28
- Governing product record:
  [Dedicated Agent-Evaluation PR Extraction PRD](../product/agent-evaluation-dedicated-pr-extraction-prd.md)
- Governing decision:
  [ADR-0063](../decisions/0063-dedicated-agent-evaluation-harness-pull-request.md)

## Architecture Outcome

The completed harness remains the `context-evaluation` developer subsystem in
the Impresari Context Cargo workspace, but its Git history and review surface
become one dedicated pull request. This is a change-isolation operation, not a
runtime extraction or repository split.

The future SWE-bench boundary is deliberately absent. Sequence 1 publishes the
read-only repository-analysis harness that already exists; it does not admit
writable benchmark workspaces, arbitrary shell execution, Docker control, or
hidden-test grading.

## Observed Repository State

After refreshing the remote during design:

```text
origin/main fe91cb4
    |
    +-- b5ec813 bounded harness
    +-- d7c2cab production adapters
    +-- e4c97a9 provider completion hardening
    +-- 117db6e renderer design records
    +-- 0f0177d source-bound renderer
    `-- 1ebba00 observable recovery and evidence selection
```

The recovery head is zero commits behind and six ahead of `origin/main`. The
worktree was clean before the sequence-1 documents. The local branch named
`main` is stale and is excluded from every extraction decision.

## Target Topology

```text
historical dirty checkout -------- preserved, untouched
original recovery branch -------- rollback and provenance anchor
                                      |
refreshed origin/main ----------------+--> dedicated candidate worktree/branch
                                               |
                                               +-- exact harness series
                                               +-- sequence-1 governance records
                                               +-- complete local gate
                                               `-- new remote branch/new PR

private evidence directory <--- hashed source-free smoke artifacts
public PR                  <--- source, tests, fixtures, docs, no live payloads
```

No step uses the historical crash-associated task as a source. The recovery
branch and files already present on disk are authoritative.

## Branch And Worktree Algorithm

### Fast Path

Immediately before extraction, fetch `origin/main` and calculate its merge
base and ahead/behind relationship with the recovery head. When the remote
base is still an ancestor and the branch is zero commits behind:

1. preserve the recovery branch name and SHA as the rollback anchor;
2. create a new dedicated branch reference at the reviewed head;
3. add only accepted sequence-1 document commits;
4. run the final gate and diff review; and
5. publish the dedicated branch as a new remote branch.

This path preserves the six causal harness commits without unnecessary history
rewriting.

### Remote-Advanced Path

If `origin/main` advances before publication:

1. do not reset, rebase, clean, stash, or switch the recovery checkout;
2. create a fresh worktree and dedicated branch at new `origin/main`;
3. carry the reviewed harness commit series in causal order;
4. resolve conflicts only after classifying each changed path against the
   extraction inventory;
5. retain the old branch and worktree as comparison evidence; and
6. rerun every validation and diff gate.

A conflict that requires a semantic product change is not an ordinary Git
resolution. It stops sequence 1 for PRD/ARD/ADR reassessment.

## Change Inventory

The candidate is organized into five review groups:

1. **Harness boundary:** `crates/context-evaluation/**`.
2. **Direct product dependency:** caller-priority packet retention in
   `context-core` and overlap-aware planned evidence in `context-engine`.
3. **Fixtures and operator contract:** `evaluation/agent-context/**` plus the
   evaluation index.
4. **Governance and security:** harness PRDs/ARDs/ADRs, boundary and threat
   model updates, dependency policy, and the security-boundary checker.
5. **Mechanical dependency evidence:** `Cargo.lock` and the generated SBOM.

The core and engine changes remain in the dedicated harness PR because the
corrected live treatment and its governing ADR depend on them, they are small
and directly tested, and extracting them into a separate precursor would leave
the harness review without the behavior it measured. The PR description must
surface them rather than presenting the change as evaluation-crate-only.

## Evidence Preservation

Temporary live-study files are not source inputs and are never copied into the
candidate worktree. A separate evidence operation copies only an explicit
allowlist to an operator-controlled private durable directory. It separates:

- source-free measurement and failure telemetry, including the successful run
  records needed to revalidate summaries; and
- restricted study definitions containing prompts, expected-answer fragments,
  retrieval queries, source allowlists, and expected evidence coordinates.

Both classes remain private. The second class is not called source-free and is
never eligible for PR attachment. Obsolete preliminary summaries, superseded
manifests, incomplete runs other than the admitted parser-failure record,
source-tree copies, and unrelated experiments are not preserved in the
sequence-1 evidence set.

The evidence manifest records, for each retained file:

- study and provider stratum;
- evidence class;
- artifact role;
- byte length;
- SHA-256 digest;
- original temporary path for local traceability; and
- preservation timestamp.

The preservation operation must reject symlinks, directories, unexpected file
names, oversized files, and credentials. Source-free artifacts must also
reject forbidden prompt, answer, packet, excerpt, and provider-body sentinels.
Restricted manifests may contain only their schema-admitted study-definition
fields; they still reject environment values, credentials, provider bodies,
packets, source excerpts, and run output. The parser-failure record is retained
as diagnostic evidence but is never combined with successful study records.

## Validation Pipeline

The extraction pipeline is serial because later assertions depend on the exact
candidate head:

```text
refresh remote
  -> prove ancestry
  -> establish candidate branch
  -> inventory paths
  -> inspect dependency/SBOM causality
  -> scan secrets and forbidden payloads
  -> cargo fmt --all --check
  -> scripts/check.sh
  -> git diff --check
  -> commit-by-commit review
  -> aggregate origin/main...HEAD review
  -> verify clean worktree
  -> push new branch
  -> create and inspect new PR
```

The push and PR are the only external mutations. They occur only after every
local gate passes and after explicit maintainer approval at the sequence stop.

## Pull-Request Contract

The PR targets refreshed `main` and uses a head branch dedicated to the
evaluation harness. Its description includes:

- architecture and execution-boundary summary;
- path groups and the two cross-crate changes;
- test commands and final head SHA;
- source-free smoke results with their sample-size limitations;
- OpenAI preflight failure without generation;
- security and privacy exclusions;
- no-SWE-bench statement; and
- follow-up sequence boundary.

The PR must not link or attach the private evidence directory. A maintainer may
request specific source-free summaries separately, but that requires an
artifact-level disclosure review.

## Failure And Rollback

- A failed local gate leaves the branch unpublished and the recovery anchor
  unchanged.
- An unexpected path leaves the candidate unpublished until admitted or
  removed without destructive commands.
- A rejected push or PR creation leaves local commits intact and retryable.
- An accidentally selected existing PR is a stop condition; do not retarget or
  overwrite it.
- A remote change after the final gate invalidates the ancestry evidence and
  returns the process to remote refresh.

No rollback path deletes branches, worktrees, temporary evidence, or the
historical checkout during sequence 1.

## Fitness Checks

- Refreshed `origin/main` is the merge base of the candidate or every carried
  commit is reviewed in a fresh worktree based on it.
- Candidate path inventory is a subset of the PRD allowlist.
- The six harness changes remain causally reviewable.
- Complete repository gate and whitespace checks pass at the final head.
- Candidate diff contains no credentials or live payloads.
- Private evidence manifest validates all retained digests.
- Remote head equals local reviewed head after push.
- New PR targets `main`, is not an existing PR, and shows only the candidate
  diff.

## Review Triggers

- The harness must leave the Impresari Context repository or Cargo workspace.
- A product-runtime crate must gain provider or process authority.
- A remote-main conflict changes packet, engine, security, or adapter behavior.
- Live run payloads are proposed for public Git history.
- The dependency/SBOM change cannot be attributed to admitted dependencies.
- The PR cannot be isolated without unrelated work.
- SWE-bench mutation, shell, Docker, or grading work enters the candidate.
