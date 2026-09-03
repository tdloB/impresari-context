# ADR-0126: Answer Host-Executed Operations Without Execution Authority

- Status: Accepted
- Date: 2026-09-03
- Related PRD: [Host-Executed Context Hooks](../product/host-executed-context-hooks-prd.md)
- Architecture: [Host-Executed Context Hooks](../architecture/host-executed-context-hooks-ard.md)

## Context

Impresari supplies context beside an agent's native repository tools, which stay
available. Measurement shows the predictable result: against an identical task,
the treatment arm performed 14 repository reads to its baseline's 7, took 23
provider requests to 20, and used 253,502 input tokens to 122,473. Roughly 46%
of that increase was a 10,453-byte packet re-sent every request; the rest was
the agent doing more work.

An extra tool offered beside existing tools can only add. To reduce work, the
product must intercept an operation the agent was going to perform anyway.

The obvious way to intercept is to perform the operation — read the file, run
the command, return a smaller answer. That is what a control-plane context layer
does, and it requires execution authority. Impresari's strongest and most
verifiable property is that it has none: `SEC-INV-007` denies process execution,
network, and telemetry outright, and the server advertises that its results add
no orchestration, approval, execution, or filesystem authority.

Trading that away to fix a token problem would exchange the product's most
defensible claim for a commodity one.

## Decision

Add a hook channel in which the **host performs every operation** and Impresari
only transforms data.

Two shapes. Read substitution: the host offers a read or search request and
receives spans recovered from the authorized snapshot, each with a content hash
and byte range. Output reduction: the host offers bytes it has already produced
and receives a bounded selection of them, with omissions recorded.

Impresari launches no process, opens no socket, writes nothing to the source
workspace, and holds no veto. A response is an offer the host may discard.
Payloads are untrusted data that cannot alter policy, capability, or budget.

## Consequences

The product can finally displace work rather than supplement it, which is the
precondition for the governing objective. Output reduction also reaches the
largest token sink in agent work — build, test, and package-manager output —
without Impresari ever running those commands.

The capability gap against a control-plane competitor stays open by choice.
Impresari cannot compress what a host declines to offer, cannot guard writes,
and cannot remember across sessions. In exchange it remains deployable where an
executing agent tool would not pass review, and its central security claim stays
literally true rather than procedurally defended.

Output reduction carries no injection surface: the response must be a selection
from bytes the host supplied in the same exchange, so the product cannot
introduce a byte the host did not already hold.

A future increment that lets a hook perform the operation itself is a different
product and requires a superseding record. This one grants no execution,
network, publication, or submission authority.
