# Impresari Context — Governing Objective

## North star

**Correctness first. Compression is a constraint, not a target.**

The product owner's standing guidance: *"I'm fine with more compression, but I
want to focus on correctness. Even 50-60% compression with very high correctness
is very acceptable."*

So the objective is a floor, not a ratio to hit: **compression of at least 50%,
and as much more as comes free, in service of the highest achievable
correctness.** A change that improves correctness and costs compression, while
staying above the floor, is a good change. A change that improves compression
and costs correctness is not, at any ratio.

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

**Correctness is whether the model, given this context, produces a change that
passes the task's own tests.** That is the number the objective is about, and it
requires a graded run against a benchmark with real model calls.

**Task-relative recall is the cheap proxy**, not the goal. For a task with a
known correct change, it asks whether the delivered context surfaced the files
and symbols that change touches. It is computable offline with no model call and
no cost, which makes it the right instrument for iterating quickly.

A map that is dense, fast, and points at the wrong file scores zero on both.

### The proxy is not the objective

Recall is a **precondition** for correctness — a model cannot patch a file it
never saw — and it is not correctness. A run may name every reference file and
still produce a patch that fails.

Two rules follow, both learned the hard way:

1. **Never report a recall movement as though it were a correctness result.**
   Say which was measured.
2. **A long run of proxy improvements owes a correctness check.** Recall can be
   optimised against conventions of whichever repository is being measured, and
   an unbroken sequence of proxy wins is exactly when that is least visible.

### Compression needs a denominator

"78% compression" is uncheckable until it says *compressed against what*. The
denominator is **what a baseline agent reads without Impresari on the same
task** — not the size of the repository, which no agent reads, and not a fixed
byte budget inherited from a harness.

Report both halves from the same run, or report neither.
