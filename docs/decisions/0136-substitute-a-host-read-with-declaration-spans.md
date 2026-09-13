# ADR-0136: Substitute a Host Read With Declaration Spans

- Status: Accepted
- Date: 2026-09-05
- Related PRD: [Host Read Substitution](../product/host-read-substitution-prd.md)
- Architecture: [Host Read Substitution](../architecture/host-read-substitution-ard.md)
- Implements: [ADR-0126](0126-answer-host-executed-operations-without-execution-authority.md)
- Amended by: [ADR-0137](0137-answer-the-symbol-a-map-names-not-the-path-it-sits-in.md) — the whole-path answer measured 92.6% of the source; the unit is the symbol, not the path

## Context

ADR-0126 decided that Impresari must intercept operations rather than sit beside
them, and specified two shapes: read substitution and output reduction. Only
output reduction was built, and it remains unmerged. Read substitution — the
shape that acts on a repository read — does not exist.

The cost of that gap is measured. On `astropy-13033` against current `main`, the
treatment arm performed 7 repository reads to a baseline's 3 and spent 239,534
tokens to its 138,372. A second baseline spent 39,220, so the baselines
themselves differ by 3.5× and a single task cannot settle the effect size — but
treatment exceeded both.

The mechanism is not map quality. Map file recall rose from 10% to 53% across
100 tasks during this work, and the treatment arm still read more. An agent
handed a pointer with no way to follow it uses the tool it has, which is the
host's own file reader. The map costs tokens to deliver and saves none.

## Decision

Answer a host's read offer with the named file's declaration spans, recovered
from the authorized snapshot, each attested with an independently computed
content hash and byte range.

Bound the answer by a host-supplied byte ceiling and a span ceiling, and disclose
reaching either. Report the bytes the whole file would have cost against the
bytes returned, so the exchange can be judged.

Keep the host as the actor and the decision-maker. Impresari launches no process,
opens no socket, writes nothing to the workspace, expresses no veto, and returns
an offer the host may discard.

Do not build search substitution, do not serve a host-named byte range, and do
not claim a token saving until a harness exposes this hook to an agent.

## Consequences

Following a map pointer becomes cheaper than opening the file, which is the
precondition for substitution being the economical choice rather than an act of
discipline. Whether agents then read less is an empirical question this decision
does not answer.

The security position is unchanged and that is the point. Performing the read on
the agent's behalf would be the obvious way to intercept it, and would require
the execution authority `SEC-INV-007` denies. Returning spans the host can verify
against its own source achieves interception without it: a byte that does not
verify is rejected by arithmetic rather than trust.

Declarations are a judgement about what a read is *for*. A host wanting a
specific byte range already knows it and does not need this; a host wanting to
understand a file wants its declarations. A file whose interesting content is not
a declaration — a data table, a configuration block — substitutes poorly, and the
accounting will say so rather than hide it.

The measurement gap is real and recorded rather than assumed away. The
substitution ratio is computable offline today. The token saving is not
measurable at all until the evaluation harness offers an agent something other
than six repository tools, which is now the blocking dependency for every claim
this project wants to make about cost.

No security invariant changes. This record grants no execution, network,
publication, or submission authority.
