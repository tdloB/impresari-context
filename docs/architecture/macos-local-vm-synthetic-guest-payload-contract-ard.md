# macOS Local-VM Synthetic Guest Payload Contract Architecture

- Status: Implemented; source-free contract only
- Date: 2026-09-01
- PRD: [macOS Local-VM Synthetic Guest Payload Contract PRD](../product/macos-local-vm-synthetic-guest-payload-contract-prd.md)
- Decision: [ADR-0111](../decisions/0111-freeze-the-macos-synthetic-guest-payload-before-materialization.md)

## Boundary

The Option C app has one closed guest directory. ADR-0111 defines that
directory as a runtime projection of the existing six-component synthetic
guest manifest, not as a copy of the entire build workspace.

```text
authenticated Alpine APK + exact project build inputs
                         |
             future private materialization
                         |
        +----------------+----------------+
        |                                 |
      Image                    impresari-initramfs.gz
        |                                 |
        +------ closed guest payload -----+
                         |
              ordinary controller path
```

## Runtime projection

| Bundle member | Source component | Runtime role |
| --- | --- | --- |
| `Image` | `linux-kernel-image` | Virtualization framework ARM64 boot kernel |
| `impresari-initramfs.gz` | `synthetic-guest-initramfs` | Ordinary synthetic PID 1 and exact storage module |

The standalone init and module are inputs already contained in the initramfs.
The resource init and resource initramfs serve the separate test-only resource
canary path. None is an additional ordinary runtime package member.

## Identity and custody

The contract binds the ADR-0091 manifest and metadata seal, the controller
source digest and literal asset names, exact component identities, and exact
future build inputs. A later materializer must use a fresh private root,
authenticate and hash the one public APK, extract only the two named members,
build only the ordinary synthetic init, produce canonical initramfs bytes,
remeasure both outputs, and delete the root before retaining metadata.

No materializer is added by this decision. The source-free checker performs no
network access or process execution beyond its own offline validation process.

## Failure behavior

Reject any missing, extra, duplicated, traversing, linked, special, executable,
or identity-drifted member. Reject controller-name drift, manifest/seal drift,
unfrozen upstream or build inputs, incomplete cleanup, and any claim that the
contract itself materialized a guest, completed a release, launched a VM, ran
an analyzer, or admitted macOS IAR-1B.

## Sequencing

The next checkpoint may materialize these exact two synthetic guest resources
in a disposable private root, verify their identities, and delete them. It
must remain separate from product-byte retention, app assembly, Apple
credentials, signing, notarization, cask installation/publication, VM launch,
real-analyzer execution, production admission, and macOS IAR-1B.
