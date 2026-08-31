# IAR-1B macOS Local-VM Guest Supply-Chain Checkpoint

- Status: Offline synthetic-candidate checkpoint passed; sealed distribution remains open
- Date: 2026-08-30
- Decision: [ADR-0087](../decisions/0087-macos-local-vm-analyzer-confinement.md)
- Host prepared-artifact run: macOS `26.5.1` arm64

## Result

The candidate guest release now has one exact, expiring release manifest for
Alpine Linux 3.24 on aarch64. It binds the two upstream downloads, all six
prepared guest components, every source/build input, an SPDX 2.3 SBOM, the
license record, provenance record, vulnerability policy, component-set digest,
and explicit initial rollback state.

The offline verifier passed once using repository metadata only and once
against every already-prepared component under
`target/iar-macos-vm-feasibility`. The prepared run recorded:

- release `iar-macos-local-vm-guest-2026-08-30.1`;
- manifest `sha256:02e5ba57ef2bb3be02cef4e978d3e518ec39a5db014988036164d2821e19b7e6`;
- profile `sha256:fb1d1d60f1be8cfe994d69b7222102ce497ab405b4f6238f144a9a55748b1714`;
- component set `sha256:926ccd4622620476e5b73a8d9b95e8c7377991946a8801e2969b9d926613392f`;
- `prepared_artifacts_verified=true`; and
- `offline_validation=true`.

The release and vulnerability-policy metadata expire at
`2026-09-30T00:00:00Z`. Expiry fails the normal repository gate until a
replacement candidate is deliberately reviewed and frozen.

## Exact Claim Boundary

The exact HTTPS origins, byte lengths, and SHA-256 values identify the reviewed
upstream bytes. They do **not** authenticate Alpine as the publisher. No live
vulnerability data was retrieved or assessed. No Developer ID signature,
notarized bundle, Homebrew cask, or production update metadata was created.

The schema therefore requires all of the following to remain false:

- `publisher_authentication_verified`;
- `vulnerability_assessment_complete`;
- `cryptographic_signature_verified`;
- `notarized_distribution_verified`;
- `sealed_distribution`;
- `production_admitted`; and
- `analyzer_execution`.

The guest cannot update itself, has no network device, and release changes may
occur only between jobs. This checkpoint adds no runtime authority and commits
no executable artifact.

## Reproduction

Repository-only, network-free validation:

```sh
ruby ./scripts/check-macos-vm-guest-supply-chain.rb
```

After the exact assets have been prepared and built, validate every output:

```sh
ruby ./scripts/check-macos-vm-guest-supply-chain.rb \
  --prepared-assets target/iar-macos-vm-feasibility \
  --output target/iar-macos-vm-feasibility/guest-supply-chain-receipt.json
```

The checker performs no download in either mode. Asset acquisition remains a
separate explicit preparation step.

## Remaining Gates

Authenticate the upstream publisher or an independently controlled release
signature, complete and record the current vulnerability review, define a
signed replacement and rollback chain, sign and notarize the complete macOS
bundle, rehearse the one-cask install/update/rollback/uninstall lifecycle, and
collect multi-host evidence. Genuine sleep/wake, reboot, abrupt power-loss,
and independent human review also remain open. macOS remains publicly at
IAR-1A.
