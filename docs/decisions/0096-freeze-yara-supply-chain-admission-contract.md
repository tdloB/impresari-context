# ADR-0096: Freeze The YARA Supply-Chain Admission Contract

- Status: Accepted for contract-only implementation
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive

## Context

ADR-0089 requires a pinned YARA executable and a small reviewed Impresari
ruleset before live analyzer admission. ADR-0095 freezes the adapter/result
shape but deliberately has no executable or ruleset supply-chain authority.

The official YARA v4.5.8 GitHub release identifies source commit
`84b0e3cc0e42f8f8e6b84d19c97ec3ac6ff8aee8` and contains no uploaded release
assets. Treating a mutable `latest` URL, an unrecorded local package, or an
unreviewed rule file as the admitted artifact would make later substitution,
rollback, and revocation impossible to detect.

## Decision

Freeze `yara-supply-chain-admission-v1` as a source-selection and future
artifact-admission contract. The source candidate is YARA v4.5.8 at the exact
official tag commit and BSD-3-Clause license-file identity observed on
2026-08-31. The selection expires after 30 days and must be re-observed before
use.

This checkpoint does not fetch or retain the source archive. It admits no
upstream binary because the official release has no uploaded assets. A future
executable can be admitted only as an Impresari-built, per-target artifact with
an exact source-archive SHA-256, locked build environment, dependency closure,
SBOM, provenance, reproducibility disposition, signature, vulnerability
review, license record, expiry, and revocation identity.

The future ruleset must be project-owned and separately versioned, reviewed,
licensed, compiled, signed, and content-addressed. Repository-provided rules,
includes, external paths, custom modules, update credentials, network retrieval,
and in-job updates remain forbidden. No single record may represent source,
executable, and ruleset as the same identity.

The deterministic checker must keep the candidate in
`contract_fixture_only`. Any expired source observation, changed tag commit,
claimed upstream binary, missing future artifact evidence, or authority claim
fails closed. Revocation takes precedence over freshness and rollback.

## Consequences

- The selected upstream source is precise without pretending a binary exists.
- A later release job must create and independently admit the executable and
  ruleset artifacts; this contract cannot activate them.
- Changing the YARA version, commit, license identity, artifact requirements,
  ruleset policy, expiry interval, or revocation semantics requires review and
  a new version or superseding ADR.
- The committed fixtures contain metadata only. They contain no YARA source,
  executable, rules, malware, repository source, credentials, or network
  capture.

## Alternatives

- Download `latest`: rejected because it is mutable and cannot support exact
  provenance or rollback rejection.
- Use an OS package-manager YARA build: rejected for initial admission because
  compiler, module, dependency, signing, and patch identities vary by package.
- Treat the GitHub source archive as a binary release: rejected because the
  official v4.5.8 release publishes no binary asset.
- Accept repository-provided rules: rejected because hostile repository content
  must not control the analyzer.

## Revisit Triggers

Revisit before fetching source for a build, creating or signing a YARA binary,
authoring or compiling the production ruleset, loading rules, invoking YARA,
accepting repository-derived analyzer input, using network or credentials, or
claiming executable, ruleset, IAR-2, production, confinement, or safety
admission.
