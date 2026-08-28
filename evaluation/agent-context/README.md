# Agent-context A/B/A evaluation

This directory contains the developer-only agent-context study described by
the [focused PRD](../../docs/product/agent-context-evaluation-harness-prd.md),
[ADR-0059](../../docs/decisions/0059-developer-agent-evaluation-adapter-boundary.md),
and [ADR-0061](../../docs/decisions/0061-human-readable-evaluation-model-context.md).
It does not add process execution to the engine, MCP server, or extension
contract.

## Frozen deterministic study

`v1/study.json` defines five tasks over TypeScript, Python, Go, Rust, and strict
JSON source files. The deterministic adapter is offline test infrastructure: it
proves study ordering, accounting, evidence verification, source
fingerprinting, consent, and persistence behavior. It does not measure model
quality.

From the repository root:

```console
cargo build -p context-evaluation --bins --locked
target/debug/impresari-context-agent-eval validate-spec evaluation/agent-context/v1/study.json
target/debug/impresari-context-agent-eval run evaluation/agent-context/v1/study.json /absolute/output/directory --allow-adapter-execution
target/debug/impresari-context-agent-eval validate-runs evaluation/agent-context/v1/study.json /absolute/output/directory
target/debug/impresari-context-agent-eval summarize evaluation/agent-context/v1/study.json /absolute/output/directory
```

The output directory must be outside the evaluated source directory. `run` is
the only executing operation and requires the consent flag every time.
`validate-spec`, `validate-runs`, and `summarize` never execute adapters.

## Authoring a real study

- Use evaluation schema `1.1` and freeze
  `model_context_renderer_identifier`, `model_context_renderer_version`, and
  `max_rendered_context_bytes`. Corrected production adapters require
  `impresari-evaluation-model-context` version `1.0.0`.
- Freeze an exact source-file allowlist and ground-truth evidence ranges before
  scoring.
- Use direct argv arrays. Shells and `-c`/`--command` command strings are
  rejected.
- Pin meaningful adapter, model, runtime, revision, and pricing-basis labels.
- Choose finite timeout and stdout/stderr limits at or below the library caps.
- Keep adapter-specific variables under the `IMPRESARI_EVAL_` prefix and avoid
  secrets. The child environment is otherwise cleared.
- Run only trusted executables. The harness detects source changes and reduces
  inherited authority, but it is not an OS sandbox and does not prevent a
  malicious adapter from accessing other host or network resources.
- Use an independently managed container or VM when the adapter or model client
  requires stronger isolation.

Each repetition is executed in fixed `baseline_a`, `treatment`, `baseline_b`
order. Interpret treatment results against both cold baselines and report their
drift. Incorrect answers are retained as measured outcomes. Evidence-integrity
failures invalidate a record, and efficiency gains cannot compensate for them.

For a production treatment, the adapter validates the canonical packet, binds
every evidence excerpt back to the frozen allow-listed source, decodes strict
UTF-8, and sends one deterministic JSON data rendering to either provider. It
does not send the packet's Base64URL wire excerpts, and it does not rank,
deduplicate, resize, summarize, or omit packet evidence. Renderer identity,
rendered bytes, SHA-256, and evidence count are persisted without source text.

## Data handling

Run records contain measurements, identifiers, digests, and verified evidence
coordinates. They deliberately omit prompts, answers, packet bodies, source
excerpts, raw stdout/stderr, environment values, and secrets. Do not modify the
format to retain those payloads without a new privacy and architecture review.

## Verification

`crates/context-evaluation/tests/agent_evaluation_cli.rs` executes the frozen
study end to end, proves explicit consent is required, rejects a shell command
string, validates all 15 records, and checks persisted JSON for sensitive
fixture sentinels. The complete repository gate remains `./scripts/check.sh`.
