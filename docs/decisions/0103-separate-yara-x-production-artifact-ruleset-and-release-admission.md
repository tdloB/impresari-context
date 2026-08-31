# ADR-0103: Separate YARA-X Artifact, Ruleset, And Release Admission

- Status: Source-free evaluator and closed candidate schemas implemented; activation not authorized
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive
- Related: ADR-0074, ADR-0082, ADR-0098, ADR-0099, ADR-0100, ADR-0102

## Context

ADR-0102 proves that an exact ephemeral YARA-X v1.20.0 candidate can execute
over five generated Impresari fixtures inside the admitted Linux synthetic
boundary and deliver real output through the frozen adapter. That evidence is
not a distributable product: the executable and compiled rules were unsigned,
ephemeral, unretained, and never admitted. Linux external-delegation production
support is also still gated on a new immutable release under ADR-0082.

A production pipeline must not turn one successful compatibility run into
implicit authority. Engine provenance, rule policy, platform confinement, and
the product release are different trust decisions with different revocation
and update lifecycles.

## Decision

Use three independently admitted, content-addressed bundles and one final
binding manifest.

### Engine bundle

The engine bundle contains only the per-target `yr` executable and its
machine-readable evidence. Admission requires the exact upstream source and
Impresari patch identities already frozen by ADR-0099, a locked toolchain and
dependency closure, a complete SBOM, build provenance, vulnerability and
license dispositions, a reproducibility disposition, an Impresari-controlled
signature, an expiry, and a revocation identity. Every target is admitted
independently. The first eligible target is Linux x86-64 only; macOS and
Windows cannot inherit its admission.

### Ruleset bundle

The ruleset bundle is separate from the engine. It contains reviewed
project-owned source, the compiled rules, license and ownership records,
compatibility results, a human review record, a signature, an expiry, a
rollback floor, and a revocation identity. Version 1 retains ADR-0098's narrow
module-free literal/hex surface. Repository-provided rules, includes, imports,
external variables, regular expressions, base64, XOR, network retrieval, and
in-job compilation remain prohibited.

The existing synthetic compatibility rules are evidence fixtures, not the
production ruleset, and cannot be relabeled as production rules.

### Release binding manifest

The final manifest binds exact digests for the engine bundle, ruleset bundle,
ADR-0100 adapter profile, invocation/resource profiles, one fresh compatible
ADR-0082 Linux production-support receipt, product version, source commit,
release archive, expiry, rollback floor, and revocation set. The evaluator is
source-free and returns only one of these states:

- `missing_evidence`;
- `release_pending`;
- `stale`;
- `changed`;
- `revoked`;
- `unsupported`;
- `compatible_not_activated`;
- `active`.

Only a separately reviewed activation change may produce `active`. All other
states keep production and IAR-2 false. The analyzer process receives no
signing, update, publication, or network credential.

## Required Stage Order

1. Freeze schemas and source-free evaluators for all three bundles and the
   binding manifest.
2. Build retained candidate engine bundles in a no-secret release job and
   record exact evidence without admitting them.
3. Create and independently review a production ruleset; build its compiled
   bundle outside scan jobs.
4. Select and implement the signing and publication mechanism, then verify
   install, update, rollback rejection, revocation, expiry, and removal.
5. Bind a fresh compatible Linux production-support receipt and perform a
   separate activation review.
6. Only after activation may a new ADR consider repository-derived IAR-2
   requests.

No later stage may backfill a missing earlier stage by implication.

## Consequences

- An engine update cannot silently change the ruleset, and a rules update
  cannot silently change the executable or platform claim.
- Revocation can withdraw one component or the complete binding without
  deleting evidence.
- Candidate build and signing work can proceed without opening repository
  scanning or IAR-2.
- Production remains unavailable until the Linux release gate, retained and
  signed bundles, human ruleset review, lifecycle evidence, and final
  activation all pass.
- The deferred independent product security review remains visible but does
  not block contract and candidate-pipeline work; it must be resolved before
  the final production activation required by the governing PRD.

## Alternatives

- Publish one monolithic executable-plus-rules archive: rejected because
  engine and rule updates need independent identity, review, and revocation.
- Admit the successful ADR-0102 executable digest: rejected because it is a
  per-run ephemeral compatibility identity without retained artifact,
  signature, or production provenance.
- Use upstream release binaries directly: rejected by ADR-0098 because their
  publication does not satisfy the selected Impresari build and signing
  contract.
- Compile rules inside analyzer jobs: rejected because the scan boundary must
  not accept mutable or repository-controlled policy.
- Open IAR-2 while packaging is incomplete: rejected because an unavailable or
  unverifiable analyzer cannot provide complete coverage.

## Activation Gate

This decision authorizes only in-repository contract and evaluator work. It
does not authorize retaining or publishing a YARA-X executable, creating a
production ruleset, using signing identity, uploading release assets, scanning
repository-derived content, activating production support, opening IAR-2, or
claiming detection quality, safety, or malware-free status. Those operations
require the later stage-specific review described above.

The first contract-only implementation is
`yara-x-production-admission-v1`, SHA-256
`fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c`.
Its source-free evaluator returns `release_pending` for the current evidence
and cannot emit `active` while the frozen policy keeps `activated=false`.
The registered `yara-x-production-admission.schema.json`, SHA-256
`eda3497fcc6a56a07ded32c5bec3b3f2f922af6d1d4c02792827fb425d2deb54`,
now closes the engine-bundle candidate, ruleset-bundle candidate, and
release-binding candidate shapes. Its negative fixtures prove that an admitted
engine, synthetic production ruleset, or activated release binding is rejected.
