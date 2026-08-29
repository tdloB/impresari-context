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

- 208 workspace and transitive packages are recorded.
- Every registry package records its Cargo checksum when supplied by Cargo.
- `cargo deny check` passes its advisory, ban, source, and license policy.
- RustSec scanning passes against 1,225 loaded advisories on 2026-08-23.
- No Git dependency is present in the current graph.
- Duplicate transitive `io-lifetimes` and Windows support crate versions are
  accepted warnings inherited through `cap-std`; they increase inventory size
  but do not create a second runtime authority.
- Declared licenses are within the current permissive allow policy. The SBOM
  preserves each package's exact Cargo license expression rather than
  simplifying dual-license choices.

## Release limitations

This evidence does not replace a release-candidate review of bundled license
texts, copyright notices, binary composition, signatures, provenance, or newly
published advisories. Re-run all commands from the exact clean release commit.
Naming/legal clearance, public publication, and release remain separately
gated.
