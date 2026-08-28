# ADR-0060: Provider-backed agent-evaluation adapters

- Status: Accepted for offline implementation; live execution remains separately gated
- Date: 2026-08-27
- Scope: Developer-only production adapters in `context-evaluation`

## Context

ADR-0059 permits the evaluation runner to execute trusted external adapters but
does not select provider transports, credential flow, cache policy, or native
repository tools. A meaningful study now needs separate OpenAI and Anthropic
agent strata plus a real Impresari Context packet adapter. This crosses the
otherwise denied external-provider boundary in the threat model and system
boundaries, but only for an explicitly authorized developer evaluation.

## Decision

Implement three production adapters in `context-evaluation`:

1. a local packet adapter that invokes the public `LocalEngine` library;
2. an OpenAI Responses API adapter fixed to `gpt-5.6-sol` and `high` effort;
3. an Anthropic Messages API adapter fixed to `claude-opus-5` and `high` effort.

The exception does not add network, model, credential, or process authority to
the core engine, MCP server, extension contract, consumer adapters, or product
runtime. Provider results are separate strata, each retaining Baseline A,
Treatment, Baseline B, and A-to-B drift.

Every arm starts a new agent-adapter process and a new provider conversation.
The OpenAI adapter uses no conversation or previous-response identifier and
requests non-persistence and the standard service tier. The Anthropic adapter
creates a new Messages API sequence. Prompt caching is disabled or omitted,
and any provider-reported cache read or write invalidates the run. Provider
completion status, model identity, and stop reason are checked before output is
accepted.

Both providers receive the same two custom repository tools: list the frozen
allowlist and read one bounded line range. There is no shell, hosted search,
hosted file search, code execution, MCP, or alternate file reader. Tool calls,
successful reads, and repeated path reads are counted in the trusted local
dispatcher. The treatment retains these native tools; the packet is the only
arm-level difference.

Repository text and packets are labeled untrusted data. Final answers use a
strict JSON contract. Models return source path and line range only; the local
adapter rereads the range and derives SHA-256. The harness independently checks
the citation against frozen expected evidence and verifies the source
fingerprint before and after execution.

Incorrect answers and failure to return the expected evidence remain measured
quality outcomes. Only malformed or unsafe evidence, a digest mismatch, source
mutation, or another protocol/integrity failure invalidates a run.

The harness clears ambient environment and inherits only the exact provider
secret name declared by the study. Values go only to the agent adapter, never
the packet adapter or persisted records. Provider endpoints are constants;
study files cannot redirect credentials to another host. Live keys, paid API
calls, and repository transmission require separate immediate authorization.

Provider token counters are normalized into uncached input, cached input,
cache-write input, output, and reasoning fields. Cost is recomputed from the
frozen machine-readable pricing schedule and validated by the harness. Total
cost and tokens include packet preparation; headline repository tool/read
metrics describe the agent's native tool boundary. The OpenAI adapter rejects
requests above the frozen schedule's standard-rate input threshold rather than
mispricing a long-context tier.

The packet adapter copies only allow-listed regular files into a temporary
isolated source view before opening `LocalEngine`. Its cache is outside the
evaluated source and is removed after the invocation. Unlisted files therefore
cannot enter the treatment packet.

## Alternatives Considered

### Use interactive Codex or Claude Code sessions

Rejected because retained sessions, hidden tools, product-level caching, and
usage accounting are harder to control and reproduce than direct APIs.

### Give the treatment fewer native tools

Rejected because it would make read reduction true by construction. Identical
tools let the study measure whether the packet changes agent behavior.

### Trust model-reported reads, citations, or cost

Rejected. Reads are counted at the local tool boundary, citation hashes are
derived from source, and cost is computed from provider counters and frozen
rates.

### Pass keys in study files or command arguments

Rejected because those values are likely to enter version control, process
listings, logs, and review artifacts.

## Consequences

- The evaluation crate gains a pinned HTTPS client dependency and narrowly
  scoped outbound provider adapters.
- A trusted adapter can still access host resources available to its process;
  ADR-0059's non-sandbox residual risk remains.
- Source is transmitted to a provider only through measured tool results and,
  for treatment, the generated packet. Operators must review provider data
  handling and repository sensitivity before live execution.
- Direct API behavior and serving infrastructure may still drift even with a
  fixed model ID. Baseline B and short execution windows expose but cannot
  eliminate that residual risk.

## Verification

- Offline request-shape tests for fixed model, effort, statelessness, cache
  controls, tool schema, and provider-usage normalization.
- Tool-boundary tests for successful, repeated, invalid, escaped, and
  symlinked reads plus adapter-derived citation hashes.
- Executable tests proving missing credentials fail before network access and
  the packet adapter excludes unlisted files.
- CLI tests proving an allow-listed secret reaches only the agent adapter and
  never persisted output.
- Locked formatting, Clippy, unit/integration/doc tests, dependency review, and
  the repository L05-equivalent validation gate.

## Review Triggers

- Another provider, model, effort level, endpoint, or provider-hosted tool.
- Prompt-cache use, response persistence, conversation reuse, or background
  execution.
- A credential source other than the explicit inherited variable boundary.
- Sending private/customer repositories or sensitive data.
- Network or model access outside `context-evaluation`.
- Persisting prompts, source, packets, answers, provider bodies, or secrets.
