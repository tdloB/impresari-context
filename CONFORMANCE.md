# Impresari Context conformance statement

- Product: Impresari Context
- Release: `v0.1.0`
- Release date: 2026-08-23 UTC
- Source commit: `c77e95ce95b2fde99da2582707d4e4d58a512122`
- Statement status: project self-attestation
- Scope: official published `v0.1.0` artifacts only

## Statement

Impresari Context `v0.1.0` conforms to the versioned interfaces, schemas,
security boundaries, compatibility claims, and acceptance tests published with
that release.

This is a project-defined conformance claim. It is not certification by the
Model Context Protocol project, an independent security audit, or certification
against ISO, SOC 2, Common Criteria, or another external compliance framework.

## Conforming artifacts

This statement applies only to the official archives attached to the
[`v0.1.0` GitHub release](https://github.com/tdloB/impresari-context/releases/tag/v0.1.0):

- macOS ARM64: `aarch64-apple-darwin`;
- Linux x86-64: `x86_64-unknown-linux-gnu`; and
- Windows x86-64: `x86_64-pc-windows-msvc`.

Each archive has an adjacent SHA-256 checksum file and GitHub build provenance
attestation binding it to this repository, the release workflow, and the tagged
source commit. Locally modified binaries, builds from another commit, and
capabilities added after the release tag are outside this statement.

## Normative contracts

The conforming release is evaluated against:

1. the closed JSON Schema Draft 2020-12 contracts in `schemas/v1`;
2. schema contract version `1.0.0`;
3. the CLI behavior documented at the release commit;
4. the local stdio MCP interface documented at the release commit;
5. MCP revision `2025-11-25`, with compatible acceptance of revision
   `2025-06-18`;
6. the resource-policy, identity, path, canonical JSON, semantic,
   security-boundary, and adversarial test vectors at the release commit; and
7. the language and client capability claims recorded at the release commit.

Later documentation on the default branch does not expand this release's
conformance scope.

## Security-boundary conformance

Within its documented operating conditions, `v0.1.0`:

- reads only from an explicitly authorized workspace;
- writes only to an explicitly authorized cache or export location;
- does not modify the source workspace;
- treats repository content as untrusted data, not instructions;
- rejects symlinks, traversal, stale evidence, integrity failures,
  incompatible contracts, and over-budget operations according to its
  documented fail-closed behavior;
- exposes MCP only through a local, single-client stdio child process;
- provides no HTTP listener or remote MCP service;
- adds no network, source-write, model, approval, orchestration, or
  repository-code-execution authority; and
- reports unsupported, partial, omitted, unresolved, stale, or truncated
  evidence explicitly rather than silently broadening a claim.

## Verification basis

This self-attestation is supported by:

- the successful exact-commit release workflow;
- successful native testing on the three published targets;
- locked dependency builds;
- schema and deterministic-vector validation;
- adversarial and security-boundary tests;
- dependency, license, RustSec, CodeQL, and OpenSSF checks;
- clean-install CLI and MCP rehearsals;
- published SHA-256 checksums; and
- GitHub build provenance attestations.

The public evidence is recorded in
[`docs/verification/release-evidence.md`](docs/verification/release-evidence.md).

## Limitations and non-claims

This statement does not claim:

- an independent third-party security audit;
- OS-level sandbox confinement beyond the documented application controls;
- deterministic language-model behavior or deterministic MCP tool selection by
  an AI client;
- compiler, runtime, package-manager, or language-server semantics for
  syntax-derived structural evidence;
- compatibility with client versions, operating systems, languages,
  transports, or configurations outside the recorded release scope;
- remote MCP, daemon, multi-client, malware-scanning, autonomous-execution, or
  unrestricted-plugin capabilities; or
- that capabilities added after the `v0.1.0` tag exist in the `v0.1.0`
  artifacts.

## Revalidation and withdrawal

Conformance must be evaluated separately for every release. This statement
must be corrected, narrowed, or withdrawn if the artifact identity, normative
contract, test evidence, supported environment, or documented security boundary
can no longer be verified.
