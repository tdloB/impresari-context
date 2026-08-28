# Impresari Context — Agent-Context Evaluation Harness PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-EVAL-AGENT-001 / 0.1.
- Status: Implementation contract for the recovery branch; acceptance remains
  subject to the repository validation gate and maintainer review.
- Date: 2026-08-27.
- Scope: Developer-only, local A/B/A evaluation of agent answers with and
  without an Impresari Context packet.
- Parent requirements: [Master PRD](master-prd.md) and
  [Evaluation PRD](evaluation-prd.md).
- Architecture decisions:
  [ADR-0059](../decisions/0059-developer-agent-evaluation-adapter-boundary.md)
  and [ADR-0060](../decisions/0060-provider-backed-agent-evaluation-adapters.md).
- Production-study readiness:
  [Agent-Context Production Study Preparation](agent-context-production-study-preparation.md).

## Problem And Purpose

The existing evaluation baseline measures engine evidence and resource
properties, but it does not directly determine whether supplying an Impresari
Context packet improves a bounded agent task compared with cold native
repository use. This harness supplies that missing experiment without turning
the engine, extension contract, or MCP surface into a process-execution host.

The harness is for development and research. It is not a product runtime,
extension loader, benchmark leaderboard, or proof that one model generalizes
to all repositories.

## Users And Decisions

Maintainers and evaluators use the harness to decide whether a context change:

1. improves task correctness or evidence quality at matched conditions;
2. reduces input/output tokens, cost, repository reads, repeated reads, tool
   calls, or elapsed time without weakening correctness;
3. remains beneficial when ordering drift is estimated by a second cold
   baseline; and
4. is safe and reproducible enough to enter the release-evaluation record.

## Hypotheses And A/B/A Design

Each study task runs in this fixed order:

1. **Baseline A:** a cold agent run without an Impresari Context packet.
2. **Treatment:** the same task with a packet produced from the frozen source.
3. **Baseline B:** a second cold run without the packet.

Baseline B estimates order, service, or model drift. Results must retain all
three arms; the treatment must never be compared only with a selected better or
worse baseline. A study is invalid when task identity, adapter identity,
source fingerprint, limits, or scoring rules change between arms.

The primary hypothesis is that treatment improves correctness and/or evidence
efficiency against both cold baselines. Incorrect answers are valid measured
outcomes and must remain in the comparison; otherwise the harness could not
measure a treatment that corrects a failing baseline. A treatment result is not
a win when an evidence-integrity, privacy, source-integrity, or
execution-boundary gate fails.

## Inputs And Frozen Identity

A versioned study specification must define:

- study and task identifiers;
- source root and source-file allowlist;
- prompt/task text supplied at execution time;
- answer and source-range expectations;
- packet and agent adapter argv arrays;
- adapter identity/version and model label;
- timeout and stdout/stderr byte ceilings;
- token and cost accounting inputs; and
- output directory for source-free run records.

Before and after every arm, the harness computes a deterministic SHA-256 source
fingerprint over allowed regular files, relative paths, and bytes. A changed
fingerprint invalidates the study.

## Measurements And Scoring

Each arm records, when supplied by the adapter:

- input, output, and total tokens;
- estimated monetary cost and its declared pricing basis;
- tool calls;
- repository reads and repeated reads;
- elapsed time;
- answer correctness;
- expected and cited source ranges; and
- source fingerprint and non-sensitive adapter metadata.

Correctness and evidence rules are specified before execution. Answer
correctness and failure to return the expected citation are scored outcomes,
not record-validity conditions. Every returned citation must identify an
allowed source-relative file and valid line range with an adapter-derived
digest. Malformed, out-of-root, symlink-mediated, or digest-mismatched evidence
invalidates the arm. Aggregate comparisons report treatment versus Baseline A,
treatment versus Baseline B, and the A-to-B drift; they do not hide individual
records.

## Reproducibility And Five-Language Fixture

The repository must include a deterministic, offline-safe adapter and a frozen
synthetic study spanning TypeScript/JavaScript, Python, Go, Rust, and strict
JSON. Repeated runs over identical bytes and configuration must produce
identical correctness, evidence, accounting, and source fingerprints, apart
from explicitly non-deterministic wall-clock duration.

The fixture is conformance evidence, not a claim of production model quality.
Real model studies must record model/version, adapter version, platform,
hardware, date, pricing assumptions, and known non-determinism.

## Consent, Security, And Privacy Requirements

- Only the `run` command may execute adapters, and every invocation requires an
  explicit `--allow-adapter-execution` flag. Configuration files cannot grant
  consent and consent is not persisted.
- Commands are non-empty argv arrays executed directly. Shell command strings,
  shell interpolation, and implicit shell launch are prohibited.
- The child environment is cleared and replaced only with the documented
  minimal variables required for deterministic operation.
- Source roots and files are canonicalized and contained; symlink traversal,
  non-regular files, root escape, and duplicate/ambiguous paths fail closed.
- Time and stdout/stderr limits are mandatory, bounded, and enforced. Timeout,
  spawn failure, malformed JSON, non-zero exit, or excessive output fails the
  arm without accepting a partial result.
- The harness must detect source mutation and never authorize source writes.
  It is not a security sandbox; evaluators remain responsible for choosing a
  trusted adapter and an appropriately isolated host.
- Persisted records must omit prompt text, agent answers, packet contents,
  source excerpts, raw adapter stdout/stderr, environment values, and secrets.
  Validation and summaries operate on source-free structured records.
- Packet and answer data may exist in bounded process memory only for the
  duration required to execute and score the study.

## CLI Contract

The `impresari-context-agent-eval` binary provides separate commands to run a
study, validate existing records against the original study specification, and
summarize valid records. Validation and summarization must not execute an
adapter. Invalid arguments, absent consent, unsafe specifications, and invalid
records return a non-zero status with an actionable source-free diagnostic.

## Acceptance Criteria

1. The recovered library and CLI compile with the locked workspace.
2. CLI-to-library signatures pass explicit execution consent and the study
   specification wherever validation requires it.
3. Unit tests cover valid A/B/A execution, scoring, fingerprinting, and record
   validation.
4. Negative tests cover absent consent, shell-like commands, empty argv,
   source-root escape, symlinks, mutation, timeouts, output overflow, malformed
   responses, non-zero exit, and sensitive-field non-persistence.
5. CLI tests prove only `run --allow-adapter-execution` can spawn adapters and
   that validate/summarize remain non-executing.
6. The deterministic five-language study runs offline and validates all three
   arms with repeatable source-free output.
7. Documentation explains authoring, running, validating, interpreting, and
   safely isolating a study.
8. `cargo fmt`, locked clippy/tests/doc-tests, repository/security checks, and
   the complete local `scripts/check.sh` L05-equivalent gate pass.
9. A requirement-to-evidence audit contains no missing or indirect evidence.

## Non-Goals

- Executing extensions, analyzers, or adapters inside the core engine or MCP
  server.
- Providing OS-, container-, VM-, or network-level sandboxing.
- Installing agents, models, dependencies, or credentials.
- Mutating a source repository, client configuration, or user environment.
- Persisting prompts, answers, packets, excerpts, or raw process output.
- Establishing universal model-quality or cost claims from synthetic fixtures.

## Requirement-To-Evidence Matrix

| Requirement | Required evidence |
| --- | --- |
| Fixed A/B/A order and drift reporting | Library tests plus deterministic three-arm fixture records |
| Correctness, source-range, token, cost, tool/read, and timing metrics | Record schema/validation tests and documented sample summary |
| Explicit per-run consent | CLI negative/positive tests and `run_study` API test |
| Argv-only, cleared environment, bounded execution | Unit/integration tests for rejection, environment, timeout, and output ceilings |
| Source containment, symlink denial, and mutation detection | Adversarial filesystem tests and before/after fingerprint assertions |
| No sensitive persisted content | Serialized-record field audit and sentinel-leak negative test |
| Deterministic safe adapter | Offline repeated-run comparison |
| Five-language coverage | Frozen TypeScript/JavaScript, Python, Go, Rust, and strict-JSON fixture manifest |
| CLI/library API agreement | Locked all-target compilation and CLI integration tests |
| Governing architecture compatibility | ADR-0059 review against ADR-0013, ADR-0018, boundaries, and threat model |
| Complete quality/security gate | Successful `scripts/check.sh` output from the recovery branch |
| Roadmap alignment | Recorded post-increment review; ADR/roadmap update only if durable scope changes |

## Rollout And Change Control

Implementation proceeds in three increments: API/consent repair; deterministic
adapter, fixtures, and adversarial coverage; then documentation and the full
gate. After each increment, maintainers re-check this PRD, ADR-0059, the Master
PRD, Evaluation PRD, threat model, boundaries, and roadmap. Metric, corpus,
privacy, consent, or trust-boundary changes require a documented review before
results are compared. A durable execution-boundary change requires a new or
superseding ADR.
