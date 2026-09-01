# YARA-X Production Admission Contract Evidence

- Date: 2026-08-31
- Decision: [ADR-0103](../decisions/0103-separate-yara-x-production-artifact-ruleset-and-release-admission.md)
- Policy: `yara-x-production-admission-v1`
- Policy SHA-256: `fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c`
- Candidate schema SHA-256: `eda3497fcc6a56a07ded32c5bec3b3f2f922af6d1d4c02792827fb425d2deb54`
- State: `release_pending`; activation closed

## Implemented Boundary

The content-addressed policy binds the exact ADR-0098 contract, ADR-0099
compatibility, ADR-0100 adapter, ADR-0102 live-envelope evidence, and ADR-0082
Linux external-delegation support manifest. It defines separate required
members for the engine bundle, production ruleset bundle, and final
release-binding manifest.

The evaluator reads only the explicit policy and observation files. It rejects
symlinks, unknown or missing fields, policy-identity drift, invalid dates, and
non-boolean observations. It has no process, network, environment, clock,
credential, build, signing, publication, repair, or analyzer capability.

The schema registry now contains three closed candidate definitions: engine
bundle, project-owned ruleset bundle, and release binding. Three valid fixtures
represent only absent or pending evidence. Three negative fixtures establish
that the contract rejects engine admission, synthetic ruleset provenance, and
release activation. All six fixtures and the schema are exact-digest inputs to
the production-admission checker and participate in the repository-wide Draft
2020-12 conformance suite.

## Deterministic States

The offline matrix covers `revoked`, `missing_evidence`, `changed`, `stale`,
`release_pending`, `unsupported`, and `compatible_not_activated`. Even a fully
positive synthetic observation remains `compatible_not_activated` because the
frozen v1 policy itself has `activated=false`. A later reviewed policy digest
is required before any evaluator can emit `active`.

The current fixture returns `release_pending` because ADR-0082 still requires
a new immutable release. It also reports the absent retained engine bundle,
reviewed production ruleset, binding, lifecycle evidence, and activation
review.

## Non-Claims

No YARA-X executable is retained, signed, published, uploaded, or admitted. No
production rule is authored, compiled, signed, or admitted. No repository
content or credential is accessed. Production, IAR-2, repository scanning,
detection quality, safety, malware-free status, and added authority remain
false.

## Next Proposed Stage

[ADR-0104](../decisions/0104-retain-no-secret-yara-x-linux-engine-candidate.md)
defines one authorized, authenticated-reader seven-day retained Linux x86-64
engine-candidate workflow. Because the repository is public, its non-release
Actions artifact is unavailable anonymously but is not maintainer-only. The
checkpoint preserves the current false admission and authority claims and
keeps signing, publication, rulesets, execution, and repository-derived input
outside the stage.
