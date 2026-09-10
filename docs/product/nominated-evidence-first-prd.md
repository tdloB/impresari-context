# Nominated-File Evidence First PRD

## Document Control

- PRD ID/version: IC-NFE-148 / 1.0.
- Status: Accepted for implementation.
- Date: 2026-09-10.
- Product owner: Aaron Boldt.
- Governing architecture:
  [Nominated-File Evidence First ARD](../architecture/nominated-evidence-first-ard.md).
- Governing decision:
  [ADR-0148](../decisions/0148-search-nominated-files-first-for-packet-evidence.md).
- Extends: [ADR-0128](../decisions/0128-extract-structure-for-nominated-files-not-whole-repositories.md).

## Problem

The opening packet's evidence was chosen by where a match fell in the
snapshot's path order, not by whether it was about the task.

Every planned search walked the whole snapshot in the order of its encoded path
units, and both the search response and the packet kept the records found first.
Paths such as `.github/`, `CHANGES.rst` and `cextern/` sort before `astropy/` in
that order. On twenty-two astropy tasks, 31 of the 43 evidence records the
packets delivered came from changelog, continuous-integration, vendored or
documentation files, and the evidence named a file the accepted change touched
for 1 of 27 reference files. Each record carries an excerpt of up to the
requested size, 4 KiB in the evaluation, so the packet spent most of its
evidence budget on text unrelated to the task.

The product already knows which files a task is about: nomination chooses up to
sixteen before the packet is built. The evidence ignored them.

## Product Outcome

The opening packet's evidence comes first from the files the task nominated,
spread across them in nomination order, and only then from the rest of the
snapshot.

## Functional Requirements

1. When a task nominated files, each literal and lexical plan step searches the
   nominated files before the whole snapshot.
2. Each nominated file is searched on its own, so one file's matches cannot fill
   a search response and hide another's.
3. Evidence from nominated files is admitted one record per file per pass, in
   nomination order, ahead of evidence from the whole-snapshot search.
4. A match both searches find is delivered once.
5. A request without a nomination builds the same packet as before.
6. Every existing limit still applies, and a nominated search that reaches one
   is disclosed as `plan_step_N_nominated_limited`.

## Acceptance Criteria

- On the twenty-two-task astropy corpus, the evidence names 14 of 27 reference
  files, against 1 of 27 on `main`, and no task loses a reference file it had.
- Nomination recall, map file recall and map items are unchanged.
- Tests prove that a nominated file leads the evidence when another file sorts
  first, and still leads when an earlier-sorting nominated file holds more
  matches than one search response can carry. Each fails with the mechanism
  disabled.
- The full repository gate passes.

## Non-Goals

- Changing which files are nominated.
- Changing how many evidence records fit in a packet or the excerpt size a
  client requests.
- Changing which plan steps are searched.
