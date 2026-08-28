# ADR-0059: Developer agent-evaluation adapter execution boundary

- Status: Accepted for evaluation-harness implementation
- Date: 2026-08-27
- Scope: Local developer evaluation tooling in `context-evaluation`

## Context

The agent-context evaluation harness must compare cold agent behavior with a
run that receives an Impresari Context packet. Producing the packet and invoking
an external agent require child processes. Existing architecture intentionally
denies process execution in the core and extension surfaces: ADR-0013 does not
authorize extension code loading, and ADR-0018 keeps connection kits and
diagnostics read-only. The threat model and `docs/boundaries.md` likewise treat
process, environment, filesystem, and network authority as explicit trust
boundaries.

Without a narrow decision, the harness could be mistaken for a product plugin
runtime or silently enlarge the engine's authority.

## Decision

Permit external adapter execution only in the developer-facing
`context-evaluation` study runner. This permission does not extend to the core
engine, extension contracts, MCP transport, client adapters, or product
runtime.

Every executing CLI invocation requires an explicit
`--allow-adapter-execution` flag. Consent is per invocation, cannot be supplied
by the study file, and is never persisted. Validation and summarization are
non-executing operations.

Adapters are non-empty argv arrays passed directly to the process API. The
runner rejects shell command strings and does not invoke a shell. It clears the
inherited environment and admits only a documented minimal environment. A
study must set finite timeout and stdout/stderr ceilings within library-defined
maximums. Spawn errors, timeouts, output overflow, non-zero exits, malformed or
unknown response fields, and incomplete results fail closed.

The source root and allowed files are canonicalized before execution. Every
path must remain beneath the canonical root, identify a regular non-symlink
file, and be unique in its normalized source-relative form. A deterministic
source fingerprint is checked before and after every arm. Escape, symlink
traversal, ambiguity, or mutation invalidates the study.

Run records contain only structured measurements and identities needed for
reproduction: arm/task/study identifiers, accounting, correctness/evidence
coordinates, timing, adapter/model labels, and source/packet digests where
applicable. They omit prompts, answers, packets, source excerpts, raw
stdout/stderr, environment values, and secrets. Sensitive payloads may exist
only in bounded process memory while executing and scoring an arm.

This boundary reduces accidental authority but is not a sandbox. The operator
must trust the selected executable or run it inside an independently managed
container/VM/OS sandbox. The harness makes no claim that clearing environment,
limiting I/O, or checking source fingerprints prevents all malicious behavior
or network access.

## Reconciliation With Existing Decisions

- **ADR-0013:** unchanged. The harness is not the extension runtime and does
  not execute extension manifests or grant extension capabilities.
- **ADR-0018:** unchanged. Product connection kits and `doctor` remain
  read-only; this separate developer tool requires conspicuous per-run consent.
- **Threat model:** child adapters are trusted external processes crossing
  process, filesystem, environment, model, and possibly network boundaries.
  Residual host compromise and access outside the declared source root remain
  operator-managed risks.
- **Boundaries document:** the engine remains authority-neutral. Evaluation
  orchestration and its temporary payloads stay outside engine APIs and
  persisted product state.

## Alternatives Considered

### Execute adapters inside the engine or extension framework

Rejected because it contradicts ADR-0013, expands product authority, and makes
evaluation-only risk part of the runtime contract.

### Accept a shell command string

Rejected because quoting, interpolation, and implicit shell features create an
unnecessary injection surface and make executable identity ambiguous.

### Treat the environment and host as a complete sandbox

Rejected because environment clearing and resource bounds do not constrain all
filesystem, network, process, or host capabilities.

### Persist prompts, answers, packets, or raw output for debugging

Rejected by default because source and secret retention would exceed the
minimum reproducibility need. A future opt-in diagnostic artifact would require
separate privacy, retention, redaction, and architecture approval.

### Omit Baseline B

Rejected because a single before/after comparison cannot expose ordering or
service drift with the same clarity as the bounded A/B/A design.

## Consequences

- Evaluators can measure agent-level context effects through arbitrary trusted
  executables without making those executables product plugins.
- The CLI contract is intentionally inconvenient enough to make execution
  visible and deliberate.
- Reproducibility improves through fixed arm order, strict input/output
  contracts, fingerprints, and source-free records.
- The runner cannot promise containment against a malicious adapter; strong
  isolation remains an external deployment responsibility.
- Adding a new execution path, persistence mode, network policy, or product
  integration requires architecture review.

## Verification

- Positive CLI test showing `run --allow-adapter-execution` executes the
  deterministic adapter and produces three valid arms.
- Negative CLI/API tests for missing consent, empty or shell-like argv, invalid
  limits, inherited environment reliance, timeout, output overflow, non-zero
  exit, malformed response, and unknown fields.
- Adversarial path tests for absolute/parent escape, symlinks, non-regular
  files, duplicate identity, and source mutation.
- Serialization tests with sentinel prompt/answer/packet/source/output/secret
  strings proving they do not appear in persisted records.
- Offline repeated-run five-language fixture and full locked repository gate.

## Review Triggers

- Adapter execution outside `context-evaluation`.
- Persistent or configuration-based consent.
- Shell invocation or command-string support.
- Inherited environment, unbounded process output, or unbounded execution.
- Network guarantees, sandbox claims, container/VM orchestration, or privilege
  management.
- Persisted prompts, answers, packets, excerpts, raw process output, or secrets.
- Source mutation, client-configuration mutation, or execution through MCP or
  extension contracts.
