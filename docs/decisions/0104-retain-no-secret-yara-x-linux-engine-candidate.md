# ADR-0104: Retain A No-Secret YARA-X Linux Engine Candidate

- Status: Approved and implemented; first authorized dispatch pending
- Date: 2026-08-31
- Related PRD: [YARA Analyzer Admission PRD](../product/yara-analyzer-admission-prd.md)
- Related architecture: [YARA Analyzer Admission ARD](../architecture/yara-analyzer-admission-ard.md)
- Related decisions: ADR-0015, ADR-0016, ADR-0074, ADR-0082, ADR-0098, ADR-0099, ADR-0102, ADR-0103

## Context

ADR-0102 proved an ephemeral YARA-X v1.20.0 build against original-synthetic
fixtures inside the Linux isolation boundary. ADR-0103 then separated engine,
ruleset, and release admission and implemented closed candidate schemas. The
next ordered stage needs a retained engine candidate so its exact bytes, SBOM,
provenance, dependency closure, and review dispositions can be examined across
jobs and by a human without rebuilding a potentially different artifact.

Retention is a new data-lifecycle boundary. The existing compatibility job
deletes its executable and compiled synthetic rules and uploads nothing. A
retained candidate must therefore define its storage, contents, lifetime,
permissions, verification, deletion, and non-claims before any workflow is
allowed to create it.

An authorized no-upload revalidation in run `33439251952`, job `99643176845`,
passed all five synthetic cases and cleanup but produced executable SHA-256
`b75b3fdd1133424cb3c9fea10d7eed7e7327b33cd260bf3afda490f5ef539721`,
which differs from the earlier live run's executable despite the same pinned
source root, patch, toolchains, runner-image version, kernel, architecture, and
rules identity. This proposal therefore cannot treat rebuilds as
byte-reproducible. Its required reproducibility disposition must investigate
or explicitly account for that difference before any later admission.

## Decision

Create one manually dispatched, no-secret GitHub Actions workflow
for the `x86_64-unknown-linux-gnu` engine candidate only.

### Build identity

- Dispatch only from the exact current `main` commit in the canonical public
  repository. Accept no free-form source, ref, URL, target, feature, command,
  retention, or output input.
- Download the exact public Impresari source snapshot and the exact upstream
  YARA-X v1.20.0 source at commit
  `60ad06971467029e77967e59d580cbbe85a1474d`.
- Require upstream archive SHA-256
  `8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee`,
  Impresari patch SHA-256
  `b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd`,
  patched lockfile SHA-256
  `e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e`,
  Rust `1.93.0`, target `x86_64-unknown-linux-gnu`, profile `release-lto`,
  feature `pulley`, and static-CRT Rust flags.
- Build inside an exact digest-pinned Linux build image. A mutable image tag or
  runner label alone is insufficient. Record the GitHub runner image identity,
  kernel, architecture, and build-image digest as evidence, but do not treat
  the host runner label as the locked userland identity.
- Permit network only while acquiring the two exact public source archives,
  the locked dependency closure, and exact public advisory data. Build and
  package offline after acquisition. Any changed byte, missing dependency,
  unexpected feature, new advisory, or incomplete license record fails before
  retention.

### Candidate contents

Create one deterministic archive no larger than 256 MiB containing only:

- the `yr` executable for the one selected target;
- an exact file manifest and SHA-256 checksum file;
- the complete dependency closure and SPDX SBOM;
- build provenance naming every identity above and the exact command;
- BSD-3-Clause license and required dependency notices;
- vulnerability, license, and reproducibility dispositions; and
- an engine-candidate record valid under ADR-0103 with `admitted=false` and
  every authority claim false.

The archive must not contain source code, a ruleset, compiled rules, scan
input, scan output, credentials, environment captures, logs, caches, signing
material, or unrelated repository files.

### Storage and verification

- Upload exactly one authenticated, non-release GitHub Actions artifact using
  a commit-pinned official upload action, with a fixed seven-day retention and
  no overwrite. Because the repository is public, any signed-in GitHub user
  with repository read access may download it; it is not maintainer-only.
- Use no repository, release, package, OIDC, attestation, or signing write
  permission. The job receives no maintainer secret or user credential.
- A separate same-run verification job downloads the artifact, rejects links
  and unexpected members, recomputes every digest, validates the SBOM and
  candidate schema, and records only bounded identities. It does not execute
  `yr`.
- Record the run, job, artifact, source, toolchain, image, executable, archive,
  SBOM, provenance, byte-count, creation, expiry, and verification identities.
- Expiry or deletion makes the candidate unavailable. It does not erase the
  bounded evidence record and cannot be represented as admission or
  revocation of a production artifact that never existed.

### Authority boundary

This stage creates a reviewable candidate only. It does not sign, attest,
publish, install, execute, or admit the executable. It does not create or
compile a production ruleset, scan repository content, access credentials,
open IAR-2, activate Linux production support, or make detection-quality,
safety, or malware-free claims. The artifact runtime token is a CI transport
capability, never an analyzer capability.

## Consequences

- Reviewers can inspect one immutable candidate and its evidence without
  relying on a fresh rebuild.
- The seven-day authenticated-reader artifact is intentionally temporary and
  is not a release asset. A later signing and publication decision must rebuild
  or promote only after verifying the exact candidate identity; this decision
  supplies no publication authority.
- GitHub Actions becomes a temporary candidate custodian. Repository
  compromise, action compromise, retention failure, and CI-platform compromise
  remain residual risks and must be represented in provenance and later
  signing review.
- The production ruleset remains an independent later stage and cannot be
  smuggled into the engine archive.

## Rejected alternatives

- Retain the binary on a maintainer laptop: rejected because local credentials,
  ambient state, custody, and reproducibility are harder to bound and audit.
- Upload a draft or public GitHub Release asset: rejected because that crosses
  the publication boundary and can be mistaken for a supported analyzer.
- Store the executable in an Actions cache: rejected because caches are keyed
  for build reuse rather than immutable review identity and explicit expiry.
- Keep rebuilding ephemeral candidates: rejected because reviewers cannot
  establish that they examined the same bytes later proposed for signing.
- Include synthetic or production rules: rejected because ADR-0103 requires a
  separately reviewed and separately revocable ruleset bundle.

## Decision outcome

Aaron Boldt approved this ADR on 2026-08-31 and selected the precise option-1
access model: exactly one manually dispatched, no-secret workflow may upload
one authenticated, non-release seven-day Linux x86-64 candidate. In this
public repository the artifact is unavailable anonymously but is downloadable
by signed-in repository readers; it is not maintainer-only. The implementation
pins the official Rust 1.93.0 Bookworm Linux/amd64 image manifest at
`sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28`
and its OCI index at
`sha256:d0a4aa3ca2e1088ac0c81690914a0d810f2eee188197034edf366ed010a2b382`.
The actual build runs in a separate `--network none` container. Packaging and
verification tools have no network or analyzer-launch capability.

The manual dispatch may occur only after the implementation merges to exact
current `main`. Selecting a signing identity, requesting `id-token` or
attestation permissions, publishing any asset, creating production rules,
executing the retained candidate, admitting it, scanning repository content,
or opening IAR-2 remains unauthorized and requires a later separate decision.
