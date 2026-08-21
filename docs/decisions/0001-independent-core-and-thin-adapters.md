# ADR-0001: Independent core with thin consumer adapters

- Status: Accepted for architecture baseline
- Date: 2026-08-20
- Scope: Initial open-source architecture

## Context

The first intended consumer is the AI App Builder OS, but the new project must
be independently useful and must not inherit the OS's private agent hierarchy,
workflow phases, or approval policies. Building an OS-specific implementation
and later extracting a public version would create duplicated code, unclear
ownership, and a high risk of private assumptions leaking into the public core.

The architecture also draws lessons from LeanCTX and Graft while intentionally
avoiding source-code combination and duplicate context or graph stacks.

## Decision

Build one neutral open-source core and integrate it into the AI App Builder OS
through a thin, separately versioned adapter.

The public core owns:

- workspace snapshots;
- policy-enforced context capabilities;
- deterministic structural indexing;
- evidence and provenance;
- bounded context packets and recovery;
- session and handoff primitives;
- extension and transport contracts;
- local observability and evaluation interfaces.

The OS owns:

- task purpose and workflow phase;
- agent identity, hierarchy, and routing;
- public/private classification policy;
- human approval and go/no-go authority;
- risk acceptance and final synthesis;
- external-action authority.

The adapter translates between the two without copying the core into the OS or
compiling OS-specific policies into the public engine.

### Boundary fitness checks

- Core crates cannot depend on an AI App Builder OS package, prompt, agent,
  workflow, approval record, or private schema.
- Every public capability is invocable through the neutral serialized contract
  and at least one non-OS conformance harness.
- The OS adapter may translate identity, purpose, policy, and result fields but
  cannot reimplement authorization, snapshot, evidence, budget, or integrity
  semantics owned by the core.
- Core and adapter versions negotiate supported contract major versions and fail
  closed on an unknown major version. Version mismatch cannot silently drop
  policy, provenance, unknown, conflict, or budget fields.
- Cross-repository compatibility tests cover the oldest supported adapter/core
  pairing and the current pairing before an adapter release.
- If adapter-only code duplicates a core invariant or exceeds the core-facing
  integration code for two consecutive releases, L03 must review whether the
  boundary or capability contract is wrong; line count alone does not decide it.

## Consequences

### Positive

- The public project has a coherent identity and broader usefulness.
- The OS validates the core against real tasks from the first delivery slice.
- Core upgrades and OS policy changes can be versioned independently.
- Private prompts, credentials, and workflow rules stay outside the public
  repository.
- There is one structural graph and one evidence model.
- Other consumers can implement adapters without adopting the OS.

### Costs

- Contract and adapter versioning must be designed early.
- Some OS conveniences cannot be hard-coded into the core.
- Cross-repository tests are required for compatibility.
- Features discovered through the OS require deliberate abstraction before
  entering the public core.

## Rejected alternatives

### Build the OS implementation and extract OSS later

Rejected because it encourages private coupling, code duplication, and
sanitization work while obscuring which behavior is generally supported.

### Embed the OS in the public repository

Rejected because an agent operating system and a context engine have different
governance, security, and release boundaries.

### Run complete LeanCTX and Graft stacks behind one wrapper

Rejected because overlapping graph, search, memory, and tool surfaces create
staleness, inconsistent evidence, and excess attack surface.

### Put consumer-specific logic behind feature flags in the core

Rejected because feature flags do not create a trustworthy public/private or
governance boundary.

## Review trigger

Review this decision if:

- two unrelated consumers cannot use the capability contract;
- the adapter must duplicate substantial engine logic;
- an OS requirement cannot be expressed without weakening a core security
  invariant;
- compatibility tests require consumer-specific exceptions in the core;
- adapter translation duplicates authorization, evidence, or budgeting logic;
- a hosted multi-tenant deployment becomes an approved project objective.
