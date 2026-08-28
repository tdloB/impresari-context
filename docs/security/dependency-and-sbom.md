# Dependency Inventory And SBOM

Status: local Slice A build evidence; not a release authorization.

## Reproducible inventory

Run:

```sh
ruby scripts/generate-sbom.rb
ruby scripts/check-sbom.rb
cargo deny check
cargo audit
```

The generator uses the complete locked, offline Cargo metadata graph. The
frozen SPDX 2.3 JSON document is `artifacts/sbom.spdx.json`; its namespace is
bound to the SHA-256 digest of `Cargo.lock`. `scripts/check-sbom.rb` regenerates
the document and requires byte equality, unique package identifiers, and a
non-empty SPDX package inventory. Package ordering uses portable package
identity fields and never Cargo's checkout-path-bearing package ID, so the
same lockfile produces the same document on Linux, macOS, and Windows.

## Current review result

- 296 workspace and transitive packages are recorded.
- Every registry package records its Cargo checksum when supplied by Cargo.
- `cargo deny check` passes its advisory, ban, source, and license policy.
- RustSec scanning passes against 1,226 loaded advisories on 2026-08-27.
- No Git dependency is present in the current graph.
- The evaluation-only provider adapters add pinned `reqwest` with Rustls for
  two fixed HTTPS endpoints. The security-boundary gate denies any second
  direct network-capable dependency or network source outside those adapters.
- Duplicate transitive `base64`, `io-lifetimes`, `syn`, and Windows support
  crate versions are accepted warnings. `reqwest` accounts for the new
  `base64`, `syn`, and Windows-family branches; the pre-existing capability
  filesystem graph accounts for `io-lifetimes` and additional Windows
  branches. They expand the reviewed inventory but not product-runtime network
  authority.
- Declared licenses are within the current permissive allow policy. The SBOM
  preserves each package's exact Cargo license expression rather than
  simplifying dual-license choices.

## Release limitations

This evidence does not replace a release-candidate review of bundled license
texts, copyright notices, binary composition, signatures, provenance, or newly
published advisories. Re-run all commands from the exact clean release commit.
Naming/legal clearance, public publication, and release remain separately
gated.
