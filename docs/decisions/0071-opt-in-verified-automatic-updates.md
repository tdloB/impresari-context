# ADR-0071: Use a separately installed, explicitly enrolled, verified updater

- Status: Proposed; founder decision required
- Date: 2026-08-29
- Related PRD: [Automatic update installation PRD](../product/automatic-update-installation-prd.md)
- Related architecture: [Automatic update installation ARD](../architecture/automatic-update-installation-ard.md)

## Context

Unattended software replacement needs stronger guarantees than a convenient
download command. It adds persistent scheduling, network and disk authority,
metadata freshness, signing-key operations, executable replacement, and
rollback. The accepted core architecture deliberately has no daemon,
background service, automatic updater, or resident telemetry. Any exception
must remain optional and outside the source-facing runtime.

## Proposed decision

If approved, provide automatic update installation only through a separately
packaged updater that an operator explicitly previews and enrolls for one
recognized user-owned portable installation.

- Keep the updater out of every core/runtime process and disabled by default.
- Use user-level platform schedulers with fixed invocations and exact-owned
  lifecycle operations; never require root.
- Trust versioned, expiring, threshold-signed TUF metadata rooted in offline
  keys, not mutable release listings or adjacent checksums alone.
- Verify accepted build provenance and the signed target identity before any
  executable is staged or activated.
- Activate all three sibling runtime executables as one transactional version,
  retaining a known-good rollback target.
- Reject Homebrew and other package-managed installations rather than crossing
  their ownership boundary.
- Limit update receipts to operational identities and reason codes; never read
  or record workspace, cache, client, credential, or unrelated home data.

## Consequences

- Eligible operators can receive unattended stable updates under an explicit
  channel, version, cadence, and maintenance-window policy.
- The project assumes substantial long-term responsibility for offline root
  custody, online delegated keys, metadata availability and expiry, incident
  response, atomic platform behavior, rollback, and scheduler compatibility.
- Automatic availability can lag publication until signed metadata and updater
  admission gates pass. Failure preserves the current runnable version.
- Portable and Homebrew installations have intentionally different lifecycle
  owners and receipts.
- The current no-background-authority baseline remains true unless the optional
  updater is separately installed and enrolled.

## Rejected alternatives

- A self-update subcommand in the primary CLI would place network and executable
  replacement authority inside the normal runtime package.
- Enabled-by-default checks would turn installation into implicit enrollment.
- GitHub Release API state plus SHA-256 is insufficient for robust update
  freshness and rollback protection.
- Sequential binary replacement cannot guarantee a coherent sibling runtime.
- Automatic major-version movement weakens operator control over compatibility
  and policy changes.

## Decision gate

This ADR is a proposal only. Acceptance requires explicit founder approval.
Implementation additionally requires separate approval of signing-root custody,
scheduled background execution, and external live rehearsal. Publication of a
release does not imply any of those approvals.
