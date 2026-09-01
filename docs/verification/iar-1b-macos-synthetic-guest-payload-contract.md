# macOS IAR-1B Synthetic Guest Payload Contract Evidence

- Date: 2026-09-01
- Decision: ADR-0111
- Outcome: source-free contract passed; materialization and admission remain gated

## Bound identities

- Contract baseline: `bd43b522987fd428008a2a1bb0d56123fd4c955c`.
- Contract SHA-256:
  `4e43e28f325d7ab67ff2bb23595eb9273320ff5e8597553b9a681bfdc51033d4`.
- Guest release: `iar-macos-local-vm-guest-2026-08-31.1`.
- Guest manifest SHA-256:
  `d0aad27ee855cac8969b189ab24cd10b58d6ceffae42f43ff0fbf4952c1785ff`.
- Metadata seal SHA-256:
  `c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1`.

## Closed runtime payload

| Member | Bytes | SHA-256 |
| --- | ---: | --- |
| `Image` | 36,175,872 | `4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5` |
| `impresari-initramfs.gz` | 38,207 | `89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b` |

Both future bundle members are mode `0644`. The exact manifest's standalone
init, extracted module, resource init, and resource initramfs are excluded as
build or test intermediates.

## Validation boundary

The offline checker verifies exact project metadata, controller literals,
authenticated future input identity, build-input digests, cleanup requirements,
valid fixtures, and the invalid materialization overclaim. It performs no
download, compilation, guest materialization, VM launch, or analyzer execution.

App assembly, Apple signing and notarization, cask lifecycle, release identity,
production, and macOS IAR-1B remain false.
