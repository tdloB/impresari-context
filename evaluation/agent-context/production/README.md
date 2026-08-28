# Production agent-context study staging

This directory contains non-executing templates for the first provider-backed
study. They remain intentionally invalid until a frozen repository revision,
complete source allowlist, reviewed tasks, exact evidence, runtime identity,
and repetition design replace every placeholder.

The approved adapter targets are:

- OpenAI Responses API: `gpt-5.6-sol`, reasoning effort `high`;
- Anthropic Messages API: `claude-opus-5`, effort `high`; and
- the same local Impresari Context packet adapter in both provider strata.

Both provider templates use evaluation schema `1.1`, production adapter
version `1.1.0`, and the shared
`impresari-evaluation-model-context` renderer version `1.0.0`. The renderer
validates and source-binds the canonical packet, then supplies decoded strict
UTF-8 evidence as deterministic untrusted JSON. Raw Base64URL packet excerpts
must never be sent to either provider.

Each provider runs a separate A/B/A study. Do not pool raw provider token or
cost measurements. The agent tools remain identical in every arm; only the
treatment receives the packet.

## Preparing the first smoke repository

1. Create a source-only directory from one frozen Impresari Context commit.
   Do not include `.git`, build outputs, caches, credentials, or unrelated
   working-tree files.
2. Populate `source_files` with every regular file the study may expose.
3. Replace the repository revision placeholder with the exact commit SHA.
4. Author three to five tasks, their expected answer fragments, exact source
   ranges, and explicit packet retrieval plans.
5. Have a second reviewer verify the answers and ranges before any arm runs.
6. Record the runtime/container identity, normalized UTC operation timestamp,
   and smoke repetition count.
7. Run `validate-spec`; an untouched template must fail validation.

## Credentials and execution

The study file contains only the allow-listed variable name. The harness reads
the secret value from its own process and passes it only to the agent adapter;
the packet adapter does not receive it. Never write a key into a study file,
command argument, `.env` file in this repository, run record, or report.

A real invocation additionally requires `--allow-adapter-execution`. That flag
authorizes only the particular local invocation. Repository transmission and
paid API use must be reviewed immediately before the run.

## Frozen pricing evidence

The template prices were recorded on 2026-08-27 from the official provider
documentation and must be reviewed again before execution:

- [OpenAI GPT-5.6 Sol](https://developers.openai.com/api/docs/models/gpt-5.6-sol)
- [Anthropic model overview](https://platform.claude.com/docs/en/about-claude/models/overview)
- [Anthropic prompt-cache pricing](https://platform.claude.com/docs/en/about-claude/pricing)

The adapters reject prompt-cache reads or writes for this cold-arm design, but
cache rates remain frozen in the schedule so any unexpected provider counters
are visible and independently costable.
