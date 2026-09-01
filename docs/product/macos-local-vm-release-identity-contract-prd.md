# macOS Local-VM Build And Release-Identity Contract PRD

- Status: Accepted for source-free implementation; candidate production remains gated
- Date: 2026-09-01
- Architecture: [macOS Local-VM Build And Release-Identity Contract ARD](../architecture/macos-local-vm-release-identity-contract-ard.md)
- Decision: [ADR-0109](../decisions/0109-freeze-macos-build-and-release-identity-before-candidates.md)

## Problem

ADR-0108 proves the complete app tree with synthetic non-runnable markers, but
there is no closed contract for replacing those markers with real unsigned
product and guest candidates. Building first would make source, toolchain,
artifact, license, vulnerability, rollback, and release identities ambiguous.

## Outcome

Freeze one offline contract that binds the current source baseline, product
version, direct build-control inputs, exact role-to-build-unit mapping, Rust
target and toolchain, existing guest metadata seal, mandatory future candidate
record, and whole-bundle rollback semantics. Retain no executable or package.

## Requirements

1. Bind repository `tdloB/impresari-context`, baseline revision
   `f160c2042c287d0a7188f16516610b21711db8d4`, and version `0.2.0`.
2. Hash the 15 direct build-control inputs with one canonical set digest.
   The later candidate still requires its own exact full Git revision and
   source-archive SHA-256; the direct inventory does not replace full source
   closure.
3. Map exactly the four product executable roles and one guest-payload role
   from ADR-0107 to closed build commands or a closed authenticated guest
   substitution path.
4. Require exact build-host, toolchain, per-artifact bytes/digests/formats,
   product SBOM, license, vulnerability, reproducibility, and guest evidence
   before candidate substitution.
5. Treat the current contract baseline separately from the future candidate
   source revision. Contract acceptance is not release identity.
6. Preserve first-cask and guest rollback identities and reject mixed versions.
7. Validate only project metadata in-process and offline. Do not compile,
   launch a child process, write a candidate, archive, install, sign, notarize,
   publish, launch a VM, execute an analyzer, or add authority.
8. Reject any receipt claiming a materialized candidate, release bundle,
   distribution, production, or macOS IAR-1B admission.

## Non-goals

- producing product or guest candidate bytes;
- proving reproducible macOS builds;
- generating a product SBOM or vulnerability assessment;
- resolving minimum macOS compatibility;
- using Apple credentials or GitHub publication services;
- creating or installing a Homebrew cask;
- launching the local VM or any analyzer.

## Acceptance

The offline checker validates all exact project inputs, canonical source-set
digest, package and guest bindings, role coverage, future evidence requirements,
registered schemas, approved original fixtures, and negative overclaim fixture.
The full repository suite passes without retaining an artifact.
