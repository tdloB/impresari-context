# ADR-0101: Prove Synthetic Runner-To-Adapter Envelope Before Artifact Admission

- Status: Implemented; hosted isolated synthetic matrix pending
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive
- Related: ADR-0074, ADR-0098, ADR-0099, ADR-0100

## Context

ADR-0100 implements a pure parser but deliberately has no process, runner, or
artifact capability. The next roadmap choice is between creating a production
YARA-X artifact pipeline and proving the handoff from bounded process output to
the parser. Beginning with production artifacts would combine signing,
publication, provenance, runner integration, and parser transport before the
composition boundary itself has evidence.

The existing Analyzer Runner is synthetic and source-free. It must not be
relabelled as YARA-X execution, and its no-op result does not contain analyzer
stdout. A dedicated original-synthetic emitter can exercise the same bounded
process-output handoff without downloading, linking, or executing YARA-X.

## Decision

Choose the synthetic envelope before the production artifact pipeline.

Freeze `yara-x-synthetic-runner-envelope-v1` as a test-only handoff from one
content-addressed Impresari-owned synthetic emitter to the ADR-0100 parser. The
emitter accepts no repository path or bytes, rule source, command choice,
network destination, credential, environment-derived input, or arbitrary
argument. It emits exactly one committed valid-match or valid-no-match record
selected by a closed synthetic case identifier.

The supervisor captures at most 131,072 stdout bytes and zero stderr bytes.
The envelope binds a fresh job identity, exact synthetic-emitter digest,
adapter profile ID and digest, workspace snapshot, manifest, artifact hash and
length, staged path, synthetic case, stdout length and SHA-256, executable and
ruleset evidence identities, completion time, resource profile, and confinement
receipt. Raw stdout exists only in memory between exact validation and parsing
and is not retained in the receipt.

Composition is all-or-nothing. It validates the envelope, captured-byte digest
and length, exact staged path and artifact controls, successful bounded process
termination, cleanup, and ADR-0100 result identity before emitting a
source-free composition receipt. Any mismatch, timeout, crash, truncation,
stderr, extra output, cleanup failure, parser failure, or identity drift emits
only a stable source-free failure and no normalized result.

The receipt must say that an Impresari synthetic emitter executed. It must also
fix `yara_x_executed`, `analyzer_executed`, `production_admitted`,
`iar_2_admitted`, `detection_quality_claimed`, `safety_claimed`, and
`authority_added` to false. A successful synthetic composition cannot become
artifact admission or evidence that YARA-X ran.

## Consequences

- Process-output transport and parser composition are tested before production
  signing or publication enters scope.
- The exact parser can remain pure; a separate coordinator owns process and
  cleanup evidence.
- The synthetic emitter is not an analyzer and cannot satisfy coverage.
- Production artifact creation, signing, publication, runner manifest
  admission, repository-derived input, and IAR-2 remain separate gates.

## Alternatives

- Build the production artifact pipeline first: deferred because it would add
  supply-chain and signing work before the output handoff is proven.
- Treat the current IAR-0 no-op result as YARA-X output: rejected because the
  result has no stdout and identifies `impresari.synthetic`.
- Feed a committed NDJSON file directly to the parser: already covered by
  ADR-0100 and does not test process-output transport.
- Let the synthetic emitter read a fixture or repository path: rejected because
  it would add filesystem acquisition and path-substitution authority.

## Activation Gate

This ADR authorizes only closed schemas/profiles, an Impresari-owned synthetic
emitter, bounded process-to-memory capture inside already admitted synthetic
isolation, composition with ADR-0100, original-synthetic fixtures, and offline
or synthetic CI tests. It does not authorize downloading or executing YARA-X,
scanning repository content, reading credentials, using network, uploading
artifacts, admitting or signing an executable/ruleset, production support,
IAR-2, detection-quality claims, safety claims, or malware-free status.

After this contract is implemented and its synthetic matrix passes, the next
decision may open the production artifact pipeline. Real YARA-X linkage still
requires exact signed artifact and manifest admission plus platform-specific
IAR-1B support.

## Implementation Evidence

The test-only `context-yara-x-envelope` crate embeds exactly the two reviewed
match and no-match records, performs all-or-nothing in-memory composition with
the ADR-0100 parser, and emits a source-free receipt. The Analyzer Runner reuses
its single reviewed Rust process-launch site to call the existing Linux
isolation launcher in a closed synthetic mode. No second Rust child-process
authority was added.

The launcher atomically places only the content-addressed emitter in a fresh
bounded cgroup, applies the admitted Landlock and seccomp controls, and reports
success only when emitter stderr is empty. The coordinator requires exact
stdout length and digest, exact cleanup of the job and cgroup, and a complete
normalized result before returning. Ordinary local tests do not invoke the
emitter; live execution remains gated to the manual ephemeral hosted workflow.
