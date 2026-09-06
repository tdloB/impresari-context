# Host Read Substitution PRD

## Document Control

- PRD ID/version: IC-HRS-136 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-05.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Host Read Substitution ARD](../architecture/host-read-substitution-ard.md).
- Governing decision:
  [ADR-0136](../decisions/0136-substitute-a-host-read-with-declaration-spans.md).
- Implements the read-substitution half of
  [IC-HEH-126](host-executed-context-hooks-prd.md) and
  [ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md).
- Governing objective: [CLAUDE.md](../../CLAUDE.md) — correctness first,
  compression a floor.

## Problem

Impresari supplies a map and the agent reads the repository anyway.

Measured on `astropy-13033` against current `main`, one task, Claude Opus 5:

| arm | repository reads | total tokens |
| --- | --- | --- |
| baseline, no Impresari | 3 | 138,372 |
| **treatment, with Impresari** | **7** | **239,534** |
| second baseline | 2 | 39,220 |

The treatment arm read more than either baseline and cost 1.7× the first.
[ADR-0126](../decisions/0126-answer-host-executed-operations-without-execution-authority.md)
predicted this and named the cause: an extra tool offered beside existing tools
can only add.

The reason is structural, not a failure of map quality. The agent receives a map
naming a file, a symbol, and a byte range — and has no way to fetch those bytes
from Impresari. Its only route to source is the host's own file reader, so
following a pointer means reading the whole file. The map is a dead end that
costs tokens to deliver.

[IC-HEH-126](host-executed-context-hooks-prd.md) specifies the answer and this
PRD builds the half that addresses reads. Output reduction, the other half,
acts after a command has already run and cannot reduce a repository read.

## Product Outcome

A host about to read a file may offer that read to Impresari and receive the
file's declarations as exact spans — each with a content hash and byte range —
instead of the whole file. Following a pointer becomes cheaper than opening the
file, so substitution is the economical choice rather than an act of faith.

## Functional Requirements

1. Accept a read offer naming one portable path in the authorized snapshot, and
   return the declaration spans that path contains.
2. Return only bytes recovered from the admitted source. Never synthesize,
   paraphrase, summarize, or reorder content.
3. Attest every span with an independently computed content hash and byte range,
   so the host can verify each returned byte against source it already controls.
4. Bound the response explicitly: a maximum returned byte ceiling the host sets,
   and a maximum span count. Reaching either is disclosed, never silent.
5. Declare the accounting the exchange needs: bytes the whole file would have
   cost, bytes returned, and spans omitted with a reason. A host must be able to
   decide whether the substitution was worth taking, and a measurement must be
   able to attribute the saving.
6. Offer, never instruct. The response carries no veto, no approval, and no
   recommendation. A host may discard it and read the file.
7. Fail closed and static. An unknown path, an unparseable file, a stale
   snapshot, or an oversized payload yields a closed category and no partial
   content.
8. Launch no process, open no socket, write nothing to the source workspace, and
   retain nothing across invocations.
9. Treat the payload as untrusted data. A path or its content cannot alter
   policy, capability, budget, or selection.

## Acceptance Criteria

- A read offer for an admitted, parseable path returns that file's declaration
  spans, each verifying byte-for-byte against the file on disk.
- Every returned span's content hash matches an independent hash of those exact
  bytes taken from the source.
- A file with no admitted structural language, an unknown path, and a path
  outside the snapshot each yield a closed category and no content.
- The byte ceiling and span ceiling are honoured, and reaching either is
  disclosed.
- The response reports whole-file bytes, returned bytes, and omitted spans.
- A static check proves the module launches no process, opens no socket, and
  writes nothing.
- **Measured, offline, over the corpus:** returned bytes are materially smaller
  than whole-file bytes for the files an agent actually reads. This is the
  substitution ratio, and it is reported before any claim that reads got
  cheaper.
- The full repository gate passes.

## Non-Goals

- Deciding for the host. Impresari answers; the host chooses.
- Search substitution. A `search_repository_text` offer is a different shape
  with a different answer, and it is not built here.
- Serving a byte range the host names. This substitutes a *file read* with
  declarations; a host that already knows its range does not need Impresari.
- Reducing command output. That is the other half of IC-HEH-126, and it cannot
  act on a repository read.
- Proving a token saving end to end. That needs a harness that exposes this hook
  to an agent, which does not exist yet and is recorded as the blocking
  dependency.
