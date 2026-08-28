# Agent-Context Production Study Preparation

## Status

This document defines the work required to move from the deterministic
five-language conformance fixture to a meaningful model study. The fixture
proves harness mechanics only. It is not model-quality evidence.

On 2026-08-27, the maintainer approved implementation and offline testing for
two production-adapter targets:

- OpenAI Responses API, model `gpt-5.6-sol`, reasoning effort `high`; and
- Anthropic Messages API, model `claude-opus-5`, effort `high`.

This approval does not authorize credentials, paid API calls, repository
transmission, or execution of a production study. Each of those remains behind
the harness's explicit execution consent and the readiness gate below.

The three production adapter implementations and offline tests now exist in
`context-evaluation`. No production study is ready to run until every remaining
item in the readiness gate is resolved and the corpus, pricing schedule, and
execution manifest are frozen before any result is inspected.

## Adapter Boundary

The harness starts a new adapter process for every arm and sends one bounded
JSON request on standard input. The adapter returns exactly one bounded JSON
response on standard output. Standard error is diagnostic-only and bounded.
Commands are direct argument arrays; shells are prohibited.

Two production adapters are required:

1. The packet adapter invokes the Impresari Context library against the
   allow-listed frozen source and returns the packet, its source fingerprint,
   and measured packet-generation usage.
2. The agent adapter creates one new provider request/session, exposes the
   fixed repository-read tool set, optionally supplies the treatment packet,
   and returns the answer, provider usage, tool-boundary counters, citations,
   and source fingerprint.

The agent adapter is provider-specific. The OpenAI and Anthropic adapters share
the harness protocol and tool dispatcher but retain distinct API translation,
cache accounting, provider usage parsing, and pricing rules.

Provider results are separate strata. OpenAI treatment is compared with the
two OpenAI baselines, and Anthropic treatment is compared with the two
Anthropic baselines. Raw tokens and costs are not pooled across providers
because tokenizers and accounting rules differ.

## Cold-Arm Contract

Every Baseline A, Treatment, and Baseline B invocation must use:

- a new adapter process and provider request/session;
- no conversation or response identifier from an earlier arm;
- no retained model-tool state or shared agent scratch directory;
- provider prompt-cache reads disabled where the provider supports that
  control, otherwise separately reported and treated as a study limitation;
- the same system instructions, native repository tools, model version, turn
  limit, timeout, and sampling parameters; and
- no Impresari packet in either baseline.

Treatment retains native repository-read tools. This keeps tool authority
identical between arms and permits the experiment to measure whether the
packet reduces native reads rather than preventing them by construction.

## Tool-Boundary Measurements

Repository reads are counted by the adapter's tool dispatcher, never by model
self-report. Each successful or failed invocation of a repository-reading tool
increments `tool_calls`. A successful read of an allow-listed repository file
increments `repository_file_reads`. A read of a path already successfully read
in the same arm increments `repeated_repository_file_reads`.

Counters reset for every arm. Canonical repository-relative paths are used as
identities, and attempted root escape or symlink traversal fails the run. The
adapter must not offer an uninstrumented shell or alternate file-reading tool.

## Provider Usage And Pricing

Input and output tokens come from the provider response, not local estimation.
The study must freeze a machine-readable pricing schedule containing currency,
effective date, model identifier, input price per million tokens, output price
per million tokens, and any provider-specific cached-input or reasoning-token
rules. Cost is derived from the provider counters and that frozen schedule.
The OpenAI adapter rejects any individual request above 272,000 input tokens
because the provider applies a different input/output price tier above that
boundary.

The pricing schedule is evidence, not a live lookup. It must be reviewed and
committed before execution so later price changes cannot alter historical
results. Packet-generation cost is recorded separately from agent cost and is
included in treatment totals.

## Evidence Contract

The agent returns repository-relative paths and inclusive line ranges. The
adapter computes the SHA-256 value from the exact cited line bytes after the
model response; it must not accept a model-invented digest. The harness checks
that every citation is allow-listed, in range, and matches the frozen source.

## Frozen Corpus

The production corpus must contain multiple repositories, revisions, and task
types. Before results are observed, record for every repository:

- origin and license, immutable revision, and source allowlist;
- task identifier and prompt;
- expected answer fragments or an equally deterministic scoring rule;
- required source paths and line ranges;
- task author and independent reviewer; and
- a corpus manifest SHA-256 value.

Do not tune tasks or expected evidence after viewing arm outcomes. Corrections
create a new corpus version and invalidate comparison with the old version.

## Execution Manifest

Freeze the following before the first production run:

- provider and immutable model/version;
- adapter identifiers and source revisions;
- system prompt and tool schemas;
- temperature and all sampling controls;
- turn limit, command timeout, and output limits;
- repetitions per task and the fixed A/T/B order;
- repository and task corpus revision;
- pricing schedule revision; and
- host/container image and run date window.

The initial manifests fix `gpt-5.6-sol` plus `high` reasoning effort for
OpenAI, and `claude-opus-5` plus `high` effort for Anthropic. The adapter must
also record the model identifier returned by the provider. If the OpenAI API
does not expose a more specific snapshot at execution time, that limitation
must appear in the final report. Anthropic documents `claude-opus-5` as a fixed
model identifier despite its dateless form.

Repetition count should be selected with a power or precision calculation from
a pilot corpus that is not reused for the final claim. Report task-level paired
results and uncertainty; do not treat correlated arms as independent samples.

## Headline Report

The report must show Baseline A, Treatment, and Baseline B separately. Its
headline comparison includes correctness, evidence verification, total tokens,
cost, tool calls, repository reads, repeated reads, and elapsed time. It also
shows Baseline B minus Baseline A drift so ordering or provider drift cannot be
hidden by averaging the baselines.

Incorrect answers and absent or incorrect expected citations remain valid
quality measurements. Unsafe citation paths/ranges, forged citation digests,
execution-boundary failures, source mutation, or accounting failures invalidate
the affected run.

## Readiness Gate

A production study may begin only when:

- packet and selected-provider agent adapters pass protocol and adversarial
  tests;
- cold-session and cache behavior is demonstrated or explicitly qualified;
- read counters are tested at the tool boundary;
- provider usage is reconciled with the frozen pricing schedule;
- citation digests are adapter-derived and independently verified;
- the reviewed multi-repository corpus is frozen;
- execution parameters, normalized UTC operation timestamp, and repetitions are frozen; and
- a dry run produces a source-free report with all three arms and drift.

The secure credential boundary is implemented as an explicit provider variable
name inherited only by the agent adapter; no values enter specifications or
records. The unresolved inputs are the frozen corpus repository revisions and
licenses, reviewed tasks/evidence, the statistical target used to choose
repetitions, the final runtime identity, a same-day pricing recheck, and
separate authorization for paid production execution and repository
transmission.
