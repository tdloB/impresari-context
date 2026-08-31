# IAR-1B Windows BaseContainer Capability Routing

- Status: Hosted observation complete; target unsupported by frozen build gate
- Date: 2026-08-31
- Decision: [ADR-0094](../decisions/0094-prefer-windows-basecontainer-without-automatic-host-preparation.md)

## Checkpoint

The build-26100 legacy LPAC worker attempt failed closed before worker code ran.
This checkpoint does not retry or add administrator host preparation. It
observes whether an independently hosted Windows 11 arm64 image exposes the
minimum exact facts needed to design a later BaseContainer synthetic rehearsal.

The profile is `iar-windows-basecontainer-capability-v1`, digest
`sha256:9f5c8f589cf5f7ce3e6d87b6b7752aeac4da530a81edbb1bf036bf5eb7e84305`.

## Required Evidence

- Windows workstation/server product type, build, arm64 architecture, and
  normalized workspace-volume filesystem;
- trusted System32 `processmodel.dll` presence;
- `Experimental_CreateProcessInSandbox` and
  `Experimental_CreateProcessAsUserInSandbox` export presence;
- deterministic routing under the frozen minimum build `26600` contract;
- every mutation, launch, confinement, production, analyzer, and authority
  field false.

## Claim Boundary

The probe does not call either export, construct a sandbox specification,
create a profile, launch a worker, mutate the host, or contact a network. A
`ready_for_basecontainer_rehearsal` result is API-capability evidence only. It
cannot establish Windows IAR-1B or authorize a real analyzer.

## Hosted Result

PR 184 run `33367923249`, job `99412379212`, executed the probe from source
`e74be1bda94c6285671120723610fd611a9fdd27` on the GitHub-hosted
`windows-11-arm` image. The fresh host reported Windows 11 Enterprise build
`26200`, arm64, with `processmodel.dll` and both required experimental exports
present. The frozen minimum is build `26600`, so the validated receipt returned
`unsupported` with reason `unsupported_build`.

The worker did not launch, the sandbox API was not invoked, and every host
mutation, elevation, confinement, production, analyzer, and authority field
remained false. API export presence on build `26200` does not override the
version-bound compatibility contract.
