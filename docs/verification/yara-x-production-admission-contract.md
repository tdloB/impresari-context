# YARA-X Production Admission Contract Evidence

- Date: 2026-08-31
- Decision: [ADR-0103](../decisions/0103-separate-yara-x-production-artifact-ruleset-and-release-admission.md)
- Policy: `yara-x-production-admission-v1`
- Policy SHA-256: `fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c`
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
