# ADR-0142: Disclose the Files a Scoped Graph Was Built Over

- Status: Accepted
- Date: 2026-09-07
- Related PRD: [Nomination Disclosure](../product/nomination-disclosure-prd.md)
- Architecture: [Nomination Disclosure](../architecture/nomination-disclosure-ard.md)
- Completes: [ADR-0128](0128-extract-structure-for-nominated-files-not-whole-repositories.md) — which required this disclosure and shipped without it

## Context

ADR-0128 replaced a thin whole-repository graph with a dense graph over a
nominated handful of files. It named the failure mode this introduces in plain
terms: a scoped graph that misses the file that mattered will be "confident,
well-attributed, and wrong — and unlike today it will look healthy."

It answered that with two controls. Disclosure was one. Nomination recall as the
leading metric was the other, on the reasoning that a file never nominated can
never be mapped, so nomination recall bounds map recall from above.

Neither shipped. `build_task_scoped_structure` returns the nomination, the MCP
server projects the file order and admitted identifiers out of it for seeding,
and discards the rest.

The cost of that gap is not theoretical. Map file recall has sat at 19 of 27
across three separate investigations — structural density, nomination breadth,
and seed ranking each moved it by exactly zero — and none of them could
establish, for any task in the missing eight, whether the file the change
touches was nominated at all, because nothing in the product ever said.

## Decision

A progressive structural map discloses the nomination it was built from.

The disclosure carries the admitted files in rank order with the ground each was
admitted on and the number of distinct task identifiers it holds, the count of
candidates considered, and the nomination's own shortfall reasons. It states
explicitly whether coverage is scoped.

A whole-repository fallback emits the same field with
`scoped_to_nominated_files: false` and an empty list, rather than omitting it.
Absence would make an unscoped graph and an older build indistinguishable, which
is the confusion this record exists to remove.

## Consequences

Nomination recall becomes measurable offline, without product involvement, by
comparing the disclosed list against a reference change.

Its first measurement was immediately decisive. On `astropy__astropy-13033` the
file the accepted change touches — `astropy/timeseries/core.py` — was nominated
sixth of sixteen, out of 393 considered. Nomination recall is 1 of 1, and the
map still never names that file. Nomination is therefore **not** the ceiling on
that task, and the loss is downstream in seeding and traversal.

That is the point of the control: it retired a suspect that three prior
investigations could neither confirm nor eliminate.

The disclosure grants nothing. It is derived from inputs the consumer supplied,
it is emitted and never read back, and every path in it is one the graph was
built over and the map may already name. It adds one serialization of a value
already in memory, bounded at sixteen entries.

## Alternatives considered

**Counts only, no paths.** Cheaper and discloses less. Rejected: nomination
recall needs the paths, and the map already names paths from the same set, so
withholding them buys no confidentiality.

**A separate tool call.** Keeps the map smaller. Rejected: a consumer that has
to ask a second question to learn whether the first answer was scoped will read
the first answer as complete, which is the failure this record addresses.

**Unknown codes in the packet's existing string channel.** No contract change.
Rejected: encoding sixteen paths and their grounds into stable codes produces a
worse structure than the typed value that already exists.
