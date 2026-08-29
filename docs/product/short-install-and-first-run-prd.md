# Impresari Context — Short installation and first-run PRD

- Status: Approved for implementation
- Date: 2026-08-28
- Authority: Founder-requested roadmap increment
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Related decision: [Short installation and first-run ARD](../architecture/short-install-and-first-run-ard.md)

## Objective

Reduce a new user's path from a published release to a previewed first-class
local MCP connection while preserving explicit artifact version, workspace,
cache, client target, and write consent.

## Scope

- A portable macOS ARM64 and Linux x86-64 installer for an explicitly selected
  release version.
- SHA-256 verification before extraction or installation.
- Installation of the CLI, MCP server, and structural worker as sibling
  binaries in one caller-selected directory.
- One `quickstart` command that locates only the sibling MCP binary, validates
  the explicit workspace/cache boundary, and previews or installs one explicit
  managed client entry.
- A single machine-readable first-run receipt with prerequisite checks, exact
  connection effect, limitations, and client-controlled next steps.

## Non-goals

- Resolving `latest`, automatic updates, background checks, package-manager
  publication, Windows PowerShell installation, default client-home discovery,
  workspace discovery, cache discovery, client sign-in, trust, startup,
  approval, invocation, or automatic native-guidance installation.
- Weakening the existing preview-by-default and exact-owned-removal contracts.

## Acceptance criteria

- The installer requires an explicit Semantic Versioning release, chooses only
  a supported platform archive, verifies its published checksum, refuses to
  overwrite an existing binary, and leaves no temporary files after exit.
- `quickstart` accepts an explicit client, workspace, separate cache, and
  configuration target, and derives only an MCP executable located beside the
  running CLI.
- Without `--apply`, `quickstart` writes nothing. With `--apply`, it changes
  only the exact named managed configuration entry through the existing L1
  implementation.
- Tests prove preview, explicit apply, owned-entry output, and source-workspace
  immutability. The complete repository gate passes.
