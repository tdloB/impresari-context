# Impresari Context — Dedicated Agent-Evaluation PR Extraction PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-EVAL-PR-001 / 0.1.
- Status: Proposed for sequence-1 review; no push or pull request is authorized
  by this document alone.
- Date: 2026-08-28.
- Scope: Extract the completed agent-context evaluation harness from its local
  recovery history into one dedicated Impresari Context pull request.
- Governing architecture:
  [Dedicated Agent-Evaluation PR Extraction ARD](../architecture/agent-evaluation-dedicated-pr-extraction-ard.md).
- Governing decision:
  [ADR-0063](../decisions/0063-dedicated-agent-evaluation-harness-pull-request.md).
- Existing harness requirements:
  [Agent-Context Evaluation Harness PRD](agent-context-evaluation-harness-prd.md),
  [Model-Context Rendering PRD](agent-evaluation-model-context-rendering-prd.md),
  and [Observable And Efficient Agent Evaluation PRD](agent-evaluation-recovery-prd.md).

## Program Position

This is sequence 1 of the six-sequence evaluation program:

1. isolate the completed current harness in a dedicated pull request;
2. define the SWE-bench schemas and security boundary;
3. integrate the official grader and deterministic fixtures;
4. add the disposable patch-producing sandbox;
5. add paired provider execution, reporting, and pilot manifests; and
6. freeze and publish the approved study manifests and source-free reports.

Only sequence 1 is authorized here. Every later sequence stops at its entry
gate and receives a new PRD, ARD, and ADR or an explicit documented decision
that an existing record remains sufficient. Knowledge gained from the prior
sequence must be incorporated before the next sequence begins.

## Problem

The harness is complete on a local recovery branch and has passed the complete
repository gate, but it is not yet separated into a reviewable remote pull
request. Continuing SWE-bench or unrelated product work on the same branch
would mix a proven read-only evaluation boundary with a future writable,
command-executing benchmark boundary.

The local branch also coexists with an intentionally preserved historical
checkout and a stale local `main` reference. An ordinary branch switch,
reset, clean, or broad copy could lose recovery evidence, select the wrong
base, or include unrelated work.

"Extraction" in this PRD means Git and review isolation inside the existing
Impresari Context repository. It does not mean moving `context-evaluation` to
a separate repository, publishing live run records, or changing the product
runtime boundary.

## Current Evidence

At document creation time, the authoritative read-only inspection established:

- current recovery branch: `codex/agent-eval-live-smoke`;
- current harness head: `1ebba00`;
- refreshed `origin/main`: `fe91cb4`;
- ancestry: the recovery branch is zero commits behind and six commits ahead
  of refreshed `origin/main`;
- worktree: clean before these documents were added;
- harness series: `b5ec813`, `d7c2cab`, `e4c97a9`, `117db6e`, `0f0177d`, and
  `1ebba00`; and
- the complete `scripts/check.sh` gate passed at harness head before this
  extraction increment.

These values are planning evidence, not permanent assumptions. The extraction
must refresh and revalidate `origin/main` immediately before branch publication.
The stale local `main` reference is never an acceptable comparison or PR base.

## Outcome

Sequence 1 ends with one new, dedicated pull request targeting current
`origin/main`. The pull request contains only the completed harness and its
directly required core, security, dependency, fixture, and governing-document
changes. It contains no unrelated product work, live source-bearing artifacts,
credentials, future SWE-bench implementation, or changes copied from the
historical crash-associated task.

The pull request must be independently reviewable and must leave the maintainer
free to continue other Impresari Context work on separate branches or
worktrees.

## Functional Requirements

### IC-EVAL-PR-FR-001 — Preserve Recovery State

The existing recovery branch and historical dirty checkout remain recoverable
until the dedicated pull request exists and its remote head is verified. The
extraction must not use `git reset`, `git clean`, `git stash`, destructive
checkout, or broad filesystem deletion. The original crash-associated Codex
task remains unopened and unmodified.

### IC-EVAL-PR-FR-002 — Use The Refreshed Remote Base

The implementation must fetch `origin/main` and prove the intended harness
head is based on it. A stale local `main` must not be used for ancestry, diff,
or pull-request targeting.

If the recovery branch is still zero commits behind, its commit series may be
published without rewriting. If `origin/main` advances, the implementation
must stop, inspect the new commits, and create a fresh worktree from the new
remote base before carrying only the approved harness series forward. The old
recovery branch remains the rollback anchor.

### IC-EVAL-PR-FR-003 — Dedicated Branch And New Pull Request

The harness must use a new remote branch and a new pull request. It must not be
pushed into, retargeted onto, or appended to any prior Impresari Context pull
request. The pull request targets `main` and carries a title and description
specific to the agent-context evaluation harness.

### IC-EVAL-PR-FR-004 — Exact Scope Inventory

The candidate diff may include only:

- `crates/context-evaluation/**`;
- the direct caller-priority and overlap changes in
  `crates/context-core/src/lib.rs` and `crates/context-engine/src/lib.rs`;
- `evaluation/agent-context/**` and its parent evaluation index;
- the harness PRDs, ARDs, ADRs, and ADR index;
- the directly affected boundary, threat-model, dependency, and SBOM records;
- the security-boundary check required by the harness; and
- dependency lockfile changes produced by the admitted evaluation adapters.

Every path outside this inventory is a stop condition until it is explained
and admitted in a document revision. Generated dependency and SBOM churn must
be mechanically attributable to the harness dependency graph.

### IC-EVAL-PR-FR-005 — Preserve Durable Private Evidence

Before temporary storage is allowed to expire, copy the source-free Anthropic
Rust, Ruby, and Shell summaries, the admitted manifests, the budget analyses,
the token preflight summaries, and the one parser-failure diagnostic into an
operator-controlled durable private evidence directory outside every evaluated
source root.

The public pull request may summarize those results, but it must not commit
credentials, prompts, answers, packets, source excerpts, provider bodies, raw
tool payloads, or live run directories. Evidence copying must be exact and
followed by hashes or a manifest so the retained files can be distinguished
from later runs.

### IC-EVAL-PR-FR-006 — Revalidate The Complete Candidate

After the final base and scope are established, the candidate must pass:

- formatting and `git diff --check`;
- the complete locked repository gate through `scripts/check.sh`;
- a clean working-tree check after any generated-file update;
- a secret and forbidden-payload scan of the candidate diff;
- a path-level review against IC-EVAL-PR-FR-004; and
- a commit-by-commit and aggregate diff review.

Earlier green output is useful recovery evidence but cannot substitute for the
post-extraction gate.

### IC-EVAL-PR-FR-007 — Reviewable Pull-Request Record

The pull-request description must state:

- the purpose and developer-only boundary;
- the exact base and head commits;
- the six harness commits or their reviewed equivalent;
- the complete validation command and result;
- the Anthropic smoke-study limitation of one task and one repetition per
  language;
- the OpenAI HTTP 401 preflight blocker and confirmation that no OpenAI
  generation occurred;
- the two direct core/engine behavior changes;
- the absence of live result payloads and secrets; and
- the explicit exclusion of SWE-bench patch execution.

No 100% correctness, product-wide efficiency, or statistical-significance
claim may be made from the smoke studies.

### IC-EVAL-PR-FR-008 — Sequence Stop And Handoff

After the remote branch and new pull request are verified, sequence 1 stops.
The resulting PR URL, remote head SHA, base SHA, check evidence, unresolved
review findings, and any changed architectural assumptions form the input to
the sequence-2 documentation review. No SWE-bench schema or implementation
work begins on the sequence-1 branch.

## Security And Privacy Requirements

- No command in sequence 1 may execute an evaluation adapter or provider.
- No provider credential is required or read during extraction.
- The Git diff and PR metadata must remain source-free with respect to live
  prompts, answers, packets, and raw provider responses.
- Private study evidence remains outside evaluated repositories and public Git
  history.
- GitHub publication exposes only already reviewed source, tests, fixtures,
  and governance documents.
- Existing adapter consent, environment clearing, source containment, and
  fixed-endpoint rules remain unchanged.

## Acceptance Evidence

| Requirement | Required evidence |
| --- | --- |
| Recovery state preserved | Original recovery branch SHA and untouched historical checkout status |
| Correct base | Fresh fetch output, merge-base equality, and ahead/behind count against `origin/main` |
| Dedicated PR | New remote branch and new PR URL targeting `main` |
| Exact scope | Path inventory and reviewed aggregate diff |
| Durable private evidence | Private evidence manifest with hashes and no source-bearing public files |
| Complete validation | Successful post-extraction `scripts/check.sh` and `git diff --check` |
| No secrets or live payloads | Candidate-diff scan plus manual persisted-field review |
| Honest claims | PR description explicitly labels the three live studies as smoke evidence |
| Sequence stop | Handoff record contains PR/base/head/check facts and no SWE-bench implementation |

## Non-Goals

- Moving the harness to another repository or Cargo workspace.
- Rewriting the completed harness architecture.
- Running another paid model study.
- Fixing or rerunning the rejected OpenAI credential.
- Adding patch mutation, shell execution, Docker orchestration, or SWE-bench.
- Publishing temporary live run records.
- Merging the pull request without ordinary maintainer review.

## Rollout

1. Review and accept this PRD, its ARD, and ADR-0063.
2. Preserve and hash the private smoke evidence.
3. Refresh `origin/main` and establish the dedicated candidate branch without
   modifying the recovery anchor.
4. Audit the path inventory and resolve only in-scope conflicts.
5. run the complete post-extraction validation gate;
6. review commits and aggregate diff;
7. publish a new remote branch and new pull request; and
8. record the sequence-1 handoff and stop.

Any new conflict, dependency, security, privacy, or scope fact discovered in
these steps requires review before proceeding. It becomes explicit input to
the next document set rather than an undocumented workaround.
