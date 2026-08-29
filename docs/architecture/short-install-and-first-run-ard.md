# Short installation and first-run architecture

- Status: Accepted
- Date: 2026-08-28
- Governing PRD: [Short installation and first-run PRD](../product/short-install-and-first-run-prd.md)

## Decision

Provide two deliberately narrow layers:

1. `scripts/install.sh` downloads one caller-pinned GitHub release, verifies
   its adjacent SHA-256 file, and installs all three packaged binaries as
   siblings in an explicit or conventional user-local directory. The release
   workflow publishes the versioned installer, its checksum, and its build
   provenance attestation alongside the native archives.
2. `impresari-context quickstart` derives `impresari-context-mcp` only from the
   running CLI's canonical sibling directory, then delegates validation and
   mutation to the existing doctor and managed-connection implementation.

The quickstart receipt composes existing evidence instead of creating another
configuration serializer. Preview remains the default and `--apply` remains
the sole write authorization.

## Authority and failure boundaries

- The installer never resolves `latest`, runs installed binaries, edits shell
  startup files, changes `PATH`, or overwrites an existing binary.
- The CLI never searches `PATH` for an MCP server and never discovers a client
  configuration, workspace, or cache.
- The workspace and cache must already exist and remain disjoint.
- The named configuration parent must already exist and pass the existing
  regular-file, symlink, size, encoding, ownership, and atomic-write checks.
- Quickstart does not combine L2 guidance with L1 configuration because each
  has a separate ownership and consent lifecycle.
- Client-controlled trust, sign-in, enablement, approval, startup, and live
  verification remain visible next steps.

## Alternatives rejected

- An unpinned `curl | sh` latest installer would weaken provenance and make
  adoption non-reproducible.
- Default home/config discovery would make the shortest command the most
  authority-sensitive path.
- Reimplementing serializers in a setup wrapper would create contract drift.
- Automatically installing both L1 and L2 artifacts would combine distinct
  consent decisions and make exact rollback harder to explain.
