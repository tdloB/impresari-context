# Impresari Context — Governing Objective

## North star

**98% quality at 78% compression.**

Quality first. Compression second. Every design decision, PR, and benchmark in
this repository is judged against that order.

This deliberately inverts the market. LeanCTX publishes roughly 98% compression
while "preserving 78% quality." Graft publishes 42% token reduction. Competing
on compression ratio is a race won by whoever is willing to destroy the most
information. The defensible position is the inverse: context so faithful that
its small size is a consequence, not a trade.

## The rule that follows from it

Impresari is a **substitution** tool, never an **addition** tool.

- *Substitution* replaces an expensive operation. The agent reads a compact,
  exact answer instead of the file. Tokens go down.
- *Addition* supplies context alongside native repository reads that remain
  available and are still used. Tokens go up, and the product has failed its
  own purpose.

A measured run where treatment performs **more** repository reads than baseline
is a failure of this objective, regardless of how good the context looked.

## What this means in practice

1. Judge delivered context by whether it contains what the task actually
   needed, before judging how small it is.
2. Never report a compression figure without the quality figure beside it.
   A ratio alone is not a result.
3. Prefer omission you can detect over deletion you cannot. Dropping a byte
   range and recording the omission is recoverable; stripping comments and
   whitespace is not.
4. Progressive disclosure is a quality mechanism, not only a size mechanism:
   what is delivered stays byte-exact and citable, and what is withheld stays
   one exact call away.
5. Integrate where a native read can be replaced. An extra tool beside the
   agent's existing tools can only add.

## How quality is measured

Quality is **task-relative recall**, not text similarity.

For a task with a known correct change, the question is whether the delivered
context surfaced the files and symbols that change touches. This is computable
offline, with no model call and no cost, against any dataset that ships a
reference patch.

A map that is dense, fast, and points at the wrong file scores zero.
