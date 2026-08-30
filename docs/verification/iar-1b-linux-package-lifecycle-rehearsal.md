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

## Hosted Evidence

Release-candidate run `33297882070`, Linux job `99220559617`, passed from exact
merged source `9d3e5de6fc09b57447c2798ea1bf09ee481229d0` on the GitHub-hosted
Ubuntu x86_64 runner. The macOS and Windows package jobs in the same workflow
also passed their complete repository, build, package, clean-install, and
artifact-upload gates.

The Linux candidate archive was
`6c7121b457462194906323b4763416ccaaf425ab6c54fe049fc07cf85405fe68`.
It was distinct from the checksum-verified v0.1.0 baseline archive
`5b3c71025128e847d8a336f33a6938afaccb105b71a6ec0b30f0fb8c814049b3`.
Both receipts bound the exact three-binary package scope and left service-unit,
authorization-policy, unexpected-package-file, and staged-source state absent.

- A receipt identity
  `fc85fdc66b51ea62d8441601a09efd69c5024ebccd8eca2ea60b339de8f0baba`
  returned `package_lifecycle_partial`. Install, upgrade, rollback, and uninstall
  passed; `logout_login` remained `not_observed` with
  `real_login_session_required`.
- C receipt identity
  `5c23d69982f486963dbd4570a0ecb2a57bbdb989a275ff720656f067441754e0`
  returned `package_lifecycle_candidate`. Install, upgrade, rollback, exact safe
  CLI operator relaunch, and uninstall passed.

Every authority claim remained false. These receipts do not contain or imply
topology revalidation, cancellation, crash recovery, health withdrawal, full
lifecycle admission, production admission, real-analyzer authority, privileged
installation, or a persistent service. The next C-profile checkpoint composes
this exact package receipt with fresh external-topology, cancellation, crash,
and health-withdrawal evidence. A remains partial until a genuine fresh login
session supplies its separate reentry evidence.
