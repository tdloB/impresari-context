# Homebrew distribution architecture

- Status: Proposed; not approved for implementation
- Date: 2026-08-29
- Governing PRD: [Homebrew distribution PRD](../product/homebrew-distribution-prd.md)
- Governing decision: [ADR-0070](../decisions/0070-homebrew-tap-distribution.md)

## Context

The current pinned installer already publishes three sibling native binaries
for macOS ARM64 and Linux x86-64. Homebrew can shorten installation and provide
its normal explicit update lifecycle, but a non-official tap is executable
third-party code and therefore creates a separate repository, trust, release,
and credential boundary.

## Proposed architecture

### Repository boundary

Use a dedicated `homebrew-tap` repository. Keep the formula, tap tests,
ownership rules, and release-update history there; do not make the application
repository itself a Homebrew tap. The tap contains one formula and no external
commands or casks in the initial scope.

Users install the fully qualified formula directly. This lets Homebrew add the
tap while limiting the user's explicit trust choice to that formula rather
than encouraging whole-tap trust.

### Artifact boundary

The formula selects only these already-supported release tuples:

| Host | Release target | Installed artifacts |
| --- | --- | --- |
| macOS ARM64 | `aarch64-apple-darwin` | CLI, MCP server, structural worker |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | CLI, MCP server, structural worker |

Each branch uses the immutable versioned GitHub Release archive and its exact
SHA-256. The formula copies the three executables into `bin`; it does not build,
execute, configure, sign in, or discover anything. Unsupported tuples fail
closed.

### Release-to-tap boundary

After a release completes all existing assurance gates and publishes its
archives, checksums, provenance, and SBOM, a narrowly scoped workflow may:

1. read the final release tag and checksum assets;
2. verify the expected target set and archive names;
3. render the deterministic formula version/URL/checksum change;
4. run syntax and policy checks on the rendered result; and
5. open, but never merge, a pull request in the tap repository.

The tap repository runs its own hosted formula tests. It owns the credential
needed to accept changes; the application release credential cannot directly
change the tap default branch. Missing or extra targets, checksum mismatch,
prerelease tags, non-SemVer versions, and duplicate releases fail closed.

### Update and rollback boundary

Homebrew owns metadata refresh, explicit upgrade, pinning, rollback mechanics,
and uninstall. Impresari Context does not run `brew`, inspect Homebrew state, or
contact the network. Formula rollback is a reviewed tap change tied to an
existing accepted release; it never republishes or mutates an old release.

Uninstall removes only Homebrew-managed executables. Cache, workspaces,
managed-client entries, native-guidance artifacts, and receipts are separate
ownership domains and remain untouched.

## Verification design

- Static checks reject moving URLs, absent SHA-256 values, unrecognized
  platforms, post-install hooks, execution during install, and unexpected tap
  contents.
- Hosted clean-install tests compare all three `--version` results with the
  formula version and run a source-free CLI diagnostic.
- Upgrade tests install the preceding formula revision, perform an explicit
  upgrade, and verify the new version; a separate pin test proves no upgrade.
- Uninstall tests seed non-formula state and prove it remains byte-identical.
- The application repository verifies only deterministic formula-input
  generation. The tap is authoritative for actual Homebrew acceptance.

## Alternatives rejected

- **Publish directly from the release workflow:** collapses release and tap
  review authority and can expose users to an untested formula.
- **Whole-tap trust instructions:** grant unnecessary authority to future tap
  contents.
- **Run `quickstart --apply` after installation:** combines package installation
  with client-configuration consent.
- **Add an Impresari self-updater:** duplicates Homebrew ownership and adds
  network/background authority to the application.
- **Submit immediately to `homebrew/core`:** broadens support and source-build
  obligations before the tap path has adoption and maintenance evidence.

## Implementation gate

This architecture is deliberately non-operative. Founder approval and a
separately governed tap repository are required before code, workflows,
credentials, or publication steps are added.
