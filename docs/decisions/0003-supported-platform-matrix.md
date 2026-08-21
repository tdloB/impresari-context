# ADR-0003: Supported platform matrix

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: MVP development, conformance, release artifacts, and filesystem claims

## Context

Workspace authorization and evidence identity depend on filesystem behavior.
Case sensitivity, path separators, Unicode representation, symbolic links,
permissions, file identity, locks, and atomic replacement differ across
platforms. Claiming broad support without running the security and conformance
suites would contradict the product's evidence-first posture.

Rust's upstream target tier indicates compiler/toolchain support, not that this
project's filesystem and security behavior has been tested.

## Decision

Define project support independently from Rust's target tier.

### Tier A — Release supported

The first public MVP must build and pass the full conformance, security,
evaluation smoke, clean-install, and source-immutability suites on:

| Platform | Rust target | Project test environment |
| --- | --- | --- |
| macOS on Apple silicon | `aarch64-apple-darwin` | A currently vendor-supported macOS release on APFS; test case-insensitive default plus case-sensitive fixture where available |
| Linux x86-64 GNU | `x86_64-unknown-linux-gnu` | A current Ubuntu LTS-class environment on a local POSIX filesystem such as ext4 |
| Windows x86-64 MSVC | `x86_64-pc-windows-msvc` | A currently vendor-supported Windows release on NTFS |

Support means the project runs its own required suites; it does not inherit
support merely because Rust labels a target Tier 1.

### Tier B — Build or community supported

These may receive compile checks or community testing but are not release
supported until the full project suite is routinely available:

- `aarch64-unknown-linux-gnu`;
- `aarch64-pc-windows-msvc`;
- `x86_64-apple-darwin` while upstream/compiler and project capacity permit;
- Linux musl/static variants;
- other Rust Tier 1 or Tier 2 host targets.

Tier B failures are documented but do not block a release unless the project
has explicitly promoted the target.

### Unsupported in the MVP

- 32-bit targets;
- mobile operating systems;
- browser/Wasm execution;
- network filesystems and synchronized cloud-drive folders as cache locations;
- remote workspaces addressed over SSH or a provider API;
- filesystems that cannot provide the required local locking, atomic replace,
  permission, or path semantics;
- containers as a separate security claim. A container inherits the underlying
  supported Linux/filesystem behavior and does not create a trusted sandbox.

## Platform Rules

1. Source and cache roots must reside on local filesystems for the MVP.
2. Platform-native canonicalization and containment checks are tested before
   support is claimed.
3. Paths are not lowercased or Unicode-normalized to create identity.
4. Case/normalization collisions fail visibly.
5. Symlink/reparse-point behavior is denied or explicitly handled by platform;
   no platform silently falls back to lexical path checks.
6. Cache and export permissions use the most restrictive supported creation
   mode and document OS/ACL limitations.
7. Unsupported filesystem objects and encodings yield structured partial or
   denied states.
8. No global shell, Git, editor, model-provider, service, or registry setting is
   modified during installation.

## Release Matrix Evidence

For every Tier A target, the release record includes:

- exact OS, architecture, filesystem, Rust toolchain, and dependency lock;
- clean source build or verified artifact installation;
- path, symlink/reparse, traversal, case, Unicode, permissions, special-file,
  cancellation, cache-corruption, and export tests applicable to the platform;
- network-denied and source-immutability results;
- smoke performance and resource results;
- uninstall/cache-removal behavior.

If hosted CI cannot exercise a required filesystem behavior, a reproducible
maintainer-run release rehearsal is required and identified as such.

## Rationale

The three Tier A targets cover the principal desktop/server environments for
developer tools while keeping the security matrix bounded. Apple-silicon macOS,
x86-64 Linux GNU, and x86-64 Windows MSVC are Rust Tier 1 host-tool targets in
the current Rust platform documentation.

Linux ARM64 and Windows ARM64 are plausible additions, but release support is a
project testing commitment rather than a compiler checkbox.

## Consequences

### Positive

- Filesystem-security claims are tied to evidence.
- The first release covers the primary developer environments.
- The team can add targets without weakening current guarantees.
- Failures on untested platforms cannot be mistaken for supported behavior.

### Costs

- CI/release rehearsal spans three operating-system families.
- Windows path/reparse semantics require dedicated work.
- Intel macOS and ARM Linux users may initially build without a full support
  commitment.

## Alternatives Considered

### Linux-only MVP

Rejected because the first reference consumer and many individual developers
use macOS, while an OSS developer tool should validate Windows semantics early
rather than assuming POSIX behavior.

### Every Rust Tier 1 target

Rejected because Rust compiler testing does not validate this project's cache,
filesystem, path, and security contracts.

### POSIX platforms first, Windows later

Rejected because deferred Windows support often embeds unsafe path assumptions
into public contracts and cache identities.

## Official References

- [Rust platform support and current target tiers](https://doc.rust-lang.org/rustc/platform-support.html)
- [Rust target tier policy](https://doc.rust-lang.org/beta/rustc/target-tier-policy.html)
- [Rust Windows MSVC target details](https://doc.rust-lang.org/beta/rustc/platform-support/windows-msvc.html)

## Review Triggers

Review when a Tier B target has repeatable full-suite capacity, an upstream Tier
1 target changes status, a supported OS/filesystem loses required semantics, a
hosted/service deployment is proposed, or demand justifies a formal support
expansion.
