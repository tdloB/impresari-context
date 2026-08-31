# ADR-0095: Freeze The YARA Adapter Contract Before Execution

- Status: Accepted for contract-only implementation
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive

## Context

ADR-0089 selects YARA as the first real analyzer but does not authorize its
execution. Its activation gate requires implemented contracts and fixtures
before a real analyzer, real ruleset, or repository-derived input can cross the
Analyzer Runner boundary.

The existing IAR-0 protocol proves source-free process and accounting behavior.
It intentionally permits no findings, so it cannot define how a later YARA
adapter binds a rule observation to exact content without either broadening the
IAR-0 claim or prematurely creating an analyzer execution path.

## Decision

Freeze a separate `yara-adapter-contract-v1` profile and closed schemas for a
production-shaped but original-synthetic YARA result and its deterministic
normalization receipt. Version 1 carries only digests, identifiers, categorical
status, and bounded byte ranges. It carries no paths, source bytes, matched
bytes, raw stdout/stderr, commands, arguments, rules, include directives,
module data, network destinations, or credentials.

The contract checker may normalize only the committed original-synthetic
fixture. It must prove complete artifact accounting, canonical ordering, exact
match identities, fixed limits, fixture provenance, and constant false values
for analyzer execution, OS confinement, production admission, IAR-2 admission,
safety, ordinary-host authority, and added authority. It must also prove that
no YARA implementation or launch reference entered production Rust code.

## Consequences

- The eventual adapter boundary is reviewable before executable supply-chain
  or hostile-input risk is introduced.
- A YARA rule observation is untrusted derived data and binds to an exact
  artifact digest and byte range; it is not a safety verdict.
- Version 1 cannot accept a live analyzer result. Activating YARA requires a
  separately reviewed execution contract and evidence under ADR-0089.
- The fixture does not contain malware, third-party rules, executable content,
  repository content, credentials, or network captures.

## Alternatives

- Reuse the IAR-0 result schema: rejected because that schema correctly fixes
  findings to an empty array.
- Permit live and synthetic origins in one schema: rejected because dormant
  execution authority would make this checkpoint harder to audit.
- Parse raw YARA output now: rejected because no real analyzer execution or
  deep hostile-format parser is authorized.

## Revisit Triggers

Revisit before installing, building, signing, downloading, discovering, or
executing YARA; compiling or loading rules; accepting live analyzer output;
reading repository-derived analyzer input; adding a parser or process launch;
using network or credentials; or claiming IAR-2, production, safety, or broad
platform support.
