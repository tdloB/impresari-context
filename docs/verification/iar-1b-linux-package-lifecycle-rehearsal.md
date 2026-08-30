# IAR-1B Linux Package-Lifecycle Rehearsal

- Date: 2026-08-30
- Decision: ADR-0080
- Profiles: A rootless user manager and C externally managed delegation
- Production admitted: No
- Real analyzer authorized: No

## Exact Package Boundary

The rehearsal accepts two checksum-paired Linux release archives. It requires a
closed `release-candidate-manifest`, the `x86_64-unknown-linux-gnu` target, one
exact candidate source commit, and exactly these packaged executables:

- `impresari-context`;
- `impresari-context-mcp`; and
- `impresari-context-structural-worker`.

Safe extraction rejects absolute paths, traversal, links, devices, more than 128
entries, more than 64 MiB compressed, more than 128 MiB expanded, missing or
unmanifested files, checksum drift, manifest drift, candidate source drift, and
identical baseline/candidate archives.

## Hosted Rehearsal

The Linux release-candidate job downloads the public v0.1.0 archive and checksum,
builds and packages the exact selected workflow commit, and invokes
`scripts/linux-package-lifecycle-rehearsal.rb` once for each selected profile.
All installs occur inside one automatically removed temporary directory.

Both profiles verify clean install, exact candidate replacement, exact baseline
rollback, and removal. C also verifies a foreground operator relaunch through
the CLI's exact exit-1 machine-readable safe usage envelope. A emits
`package_lifecycle_partial` with `real_login_session_required`; the workflow does
not claim a process restart as logout/login evidence.

## Verification

`ruby scripts/check-linux-package-lifecycle-rehearsal.rb` checks the no-network,
no-privilege, no-service-manager collector boundary, the exact candidate and
distinct-archive requirements, both profile claim ceilings, and Linux-only
synthetic package execution. Contract checks accept both bounded receipts and
reject a full-lifecycle or production overclaim.

The first exact hosted run will be recorded after this workflow change reaches
`main`. Until that run passes and its evidence is composed with the remaining
topology, cancellation, crash, withdrawal, and real A-profile login-session
gates, full lifecycle and production remain closed.
