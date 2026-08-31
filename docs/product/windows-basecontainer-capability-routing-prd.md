# Impresari Context — Windows BaseContainer Capability Routing PRD

- Status: Accepted for no-worker contract implementation
- Date: 2026-08-31
- Owner: Aaron Boldt
- Architecture: [Windows BaseContainer Capability Routing ARD](../architecture/windows-basecontainer-capability-routing-ard.md)
- Decision: [ADR-0094](../decisions/0094-prefer-windows-basecontainer-without-automatic-host-preparation.md)

## Objective

Replace repeated legacy-LPAC launch attempts with an exact, read-only Windows
host-capability decision that prefers the newer OS-provided BaseContainer path
and never silently adds administrator authority or weakens confinement.

## User Outcome

An ordinary Windows user either receives a precise supported-candidate state or
an explicit unsupported state. Impresari Context does not prompt for elevation,
rewrite host ACLs, install a privileged service, enable a Windows feature, or
fall back to a weaker token merely to make a sandbox launch.

## Scope

- One digest-bound BaseContainer capability profile.
- One source-free, no-worker probe on GitHub's independently hosted
  `windows-11-arm` image.
- Normalized observation of Windows product type, build, architecture,
  filesystem, the trusted inbox `processmodel.dll`, and the two documented
  experimental sandbox exports.
- Deterministic `ready_for_basecontainer_rehearsal`, unsupported-host-family,
  unsupported-filesystem, unsupported-build, and unsupported-API states.
- Closed schemas, fixtures, provenance, invalid-overclaim evidence, a
  source-free checker, and an exact hosted receipt.

## Non-goals

Calling a sandbox entry point; compiling a FlatBuffer sandbox specification;
creating AppContainer state; launching a worker; assigning a Job Object;
changing a file, registry, device, firewall, feature, service, package, or host
ACL; requesting elevation; enabling Windows Sandbox or Hyper-V; reading
credentials or repository content; contacting a network destination; running a
real analyzer; Windows production support; or an IAR-1B claim.

## Acceptance Criteria

- The exact profile and fixture copy are byte-identical and checksum-bound.
- The probe runs only on a fresh GitHub-hosted `windows-11-arm` context and
  records only normalized host facts.
- `ready_for_basecontainer_rehearsal` requires Windows workstation product
  type, NTFS, build `26600` or newer, `processmodel.dll`, and both exact sandbox
  exports.
- Every other observed combination returns one explicit unsupported state; no
  compatible state is inferred from a version number alone.
- Source checks reject process launch, sandbox invocation, profile mutation,
  host ACL mutation, elevation, service installation, and Windows-feature
  mutation syntax.
- Every receipt fixes worker launch, profile creation, host mutation,
  confinement, production, analyzer execution, and added authority to false.
- Full local quality checks and the dedicated hosted Windows 11 arm64 job pass.

## Next Gate

Only an exact `ready_for_basecontainer_rehearsal` receipt can open a separately
reviewed synthetic-worker contract. That later contract must resolve sandbox
specification generation, non-inherited protocol transport, pre-resume Job
Object composition, zero writable path-backed storage, cleanup, cross-job
isolation, signing, compatibility withdrawal, and lifecycle evidence. This
preflight does not authorize that gate by itself.
