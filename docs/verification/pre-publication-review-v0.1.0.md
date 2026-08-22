# Pre-publication security and release review — v0.1.0

## Review status

- Date: 2026-08-22
- Scope: threat model, local MCP boundary, release packaging, dependency posture,
  and files intended for the public repository.
- Method: structured internal AI-assisted review using the AI App Builder OS
  security, Rust, architecture, quality, and release criteria, supplemented by
  direct command evidence.
- Independence: this is a rigorous internal review, not a third-party audit and
  not a substitute for an independent human security assessment.
- Current disposition: **conditionally ready**. The technical gates pass. The
  public GitHub security settings, version update, hosted external scans, and
  explicit publication approval remain open.

## Executive summary

No release-blocking vulnerability, secret exposure, workspace-boundary failure,
or MCP authority expansion was found. The complete local test gate passed,
including adversarial filesystem, cross-workspace isolation, packet-integrity,
MCP-equivalence, extension-quarantine, resource-limit, and clean-output tests.
RustSec reported no known vulnerable locked dependency.

The review found one dependency-policy defect: the permissive Zlib license used
by a transitive dependency was not in the project's allowlist. The allowlist was
corrected and the dependency license/source/advisory gate now passes. A dedicated
CI job was added so dependency security and license policy are checked on every
push and pull request.

## Threat model

### Result

The threat model is suitable for the authorized v0.1.0 scope. It identifies the
source workspace as untrusted, keeps policy and orchestration outside the
engine, denies ambient network and general extension execution, requires exact
source-byte recovery, binds evidence to a workspace snapshot, and treats
authorization and isolation failures as release blockers.

### Residual risks

- A user or process with equivalent operating-system access can still read or
  alter local files.
- An authorized consumer can misuse correctly returned source evidence.
- The MVP provides content-integrity identities but does not identify a human
  signer or provide non-repudiation for context packets.
- The one pinned structural worker is a deliberately narrow process boundary;
  general executable and privileged extensions remain outside this release.

These risks are accurately disclosed and do not contradict the v0.1.0 scope.

## MCP boundary

### Result

The MCP implementation is a thin local stdio adapter over the existing engine.
It opens no HTTP listener, accepts one process-local client, fixes workspace,
cache, consumer identity, and role at launch, and cannot grant filesystem,
network, model, execution, approval, or orchestration authority.

Malformed, duplicate, batched, and oversized messages fail closed. Tests confirm
that MCP results are semantically equivalent to direct engine use and that
stdout contains only protocol messages. No release blocker was found.

## Packaging and workflow

### Result

The release-candidate workflow:

- builds an explicitly supplied 40-character commit SHA;
- verifies the checked-out commit;
- uses the pinned Rust toolchain and locked dependencies;
- uses read-only GitHub permissions;
- pins third-party GitHub Actions to full commit SHAs;
- includes the CLI, structural worker, MCP binary, SBOM, license, notices, and
  support/security documents;
- emits a manifest and SHA-256 checksum; and
- rehearses installation on macOS ARM64, Linux x64, and Windows x64.

Previous hosted rehearsals passed on all three native targets. Publication and
tag creation remain manual, which is appropriate for the first release.

### Open provenance gate

The recommended v0.1.0 policy is SHA-256 checksums plus GitHub artifact
attestations. Checksums provide a universally readable file fingerprint.
Attestations add a verifiable statement that GitHub Actions built that exact
file from this repository, workflow, and commit. The workflow must not enable
attestation until the repository is public and the required permission is
available.

## Dependencies and supply chain

- `Cargo.lock` is committed and release builds use `--locked`/`--offline` after
  dependency fetch.
- `cargo audit --deny warnings`: passed; no known vulnerable locked dependency.
- `cargo deny check`: passed after adding the required Zlib license to the
  explicit allowlist.
- Dependency sources are restricted to crates.io; unknown registries and Git
  dependencies are denied.
- Multiple-version warnings remain visible. They increase size and review
  surface but are not a v0.1.0 security blocker.
- The repository contains a checked SBOM covering 177 packages.

## Public repository contents

- The OS secret scan covered the working tree and Git history and reported no
  secret finding. Redaction behavior was verified.
- No local absolute workspace path was found in tracked public files.
- GitHub Actions use least-privilege read access and immutable action SHAs.
- The Apache-2.0 license text, acknowledgments, contribution policy, governance,
  maintainer record, code of conduct, security policy, and NOTICE file exist.
- LeanCTX and Graft are credited as architectural influences without claiming
  endorsement, succession, or copied source.
- Generated brand explorations and the palette document contain no observed
  personal author or location metadata. They are optional public-repository
  content rather than release blockers.

## Required actions before publication

1. Change the workspace and package version from `0.0.0` to `0.1.0` and update
   release filenames/tests.
2. Run normal CI and the native release-candidate matrix for the final commit.
3. Make the repository public only with explicit owner approval.
4. Immediately enable GitHub private vulnerability reporting, Dependabot
   alerts/security updates, secret scanning, push protection, and code scanning
   where GitHub exposes them for the public repository.
5. Enable OpenSSF Scorecard and review the first CodeQL and Scorecard results.
6. Configure branch protection/rules once GitHub makes enforcement available.
7. Publish/tag `v0.1.0` only after explicit owner approval and successful final
   evidence review.

## Evidence

- Complete local gate: `./scripts/check.sh` — passed 2026-08-22.
- Dependency gate: `./scripts/audit-dependencies.sh` — passed 2026-08-22.
- OS baseline: working-tree and history secret scans, unsafe-pattern inventory,
  and immutable-action policy completed with no recorded finding.
- Hosted CI and release-candidate evidence is archived in
  `docs/verification/release-evidence.md`.
