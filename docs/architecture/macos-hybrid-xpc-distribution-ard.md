# macOS hybrid XPC distribution architecture

- Status: Accepted for synthetic implementation; production distribution gated
- Date: 2026-08-29
- Governing PRD: [macOS hybrid XPC distribution PRD](../product/macos-hybrid-xpc-distribution-prd.md)
- Governing decision: [ADR-0076](../decisions/0076-macos-hybrid-xpc-cask.md)

## Component topology

```text
Homebrew cask
└── Impresari Context.app (one signed and notarized version)
    ├── Contents/MacOS/impresari-context
    │   └── Rust CLI and exact-job supervisor
    └── Contents/XPCServices/ImpresariAnalyzer.xpc
        └── App Sandbox service and in-process admitted analyzer

Homebrew prefix/bin/impresari-context
└── link or generated wrapper to the supported bundled entry point
```

The cask moves the intact app bundle and exposes the embedded CLI entry point.
It must not rewrite Mach-O files, entitlements, plists, or nested bundle
contents after signing. The command-line surface remains primary; the app host
is background-only.

## Runtime boundary

The CLI resolves its containing bundle identity and rejects a detached,
substituted, unsigned, mixed-version, or writable-component topology. It starts
one background host for one job and receives a bounded preparation record that
identifies the private XPC service before analyzer work begins.

The frozen Rust-to-host request carries only the request ID, profile identity,
canonical job digest, artifact count/bytes, and expected sealed-bundle/host/XPC
identities. Closed schemas and Rust validation reject a repository path,
arbitrary arguments, caller environment, credentials, network authority,
analyzer execution, unknown fields, mismatched preparation identity, partial
readiness, retained source, or an IAR-1B/production claim. Transport is one
bounded canonical request and one bounded source-free preparation record.

The XPC harness applies irreversible CPU, address-space-growth, file/output,
descriptor, and process-count limits, then calls the admitted analyzer library
in-process. `RLIMIT_NPROC=0`, absent process authority, and the closed service
implementation prohibit descendants. The Rust supervisor enforces the wall
deadline, verifies the exact prepared service identity, terminates only that
job, rejects partial output, and validates source-byte cleanup.

No service persists independently of its client. No network entitlement,
repository path, home access, credential access, user-selected-file grant,
shell, arbitrary executable, or analyzer discovery enters the service.

The exact `iar-macos-xpc-hybrid-v1` profile fixes one job, 64 artifacts,
1 MiB per artifact, 4 MiB total input, 256 KiB output, 16 KiB stderr, 8 MiB
per-file temporary output, 32 descriptors, 30 CPU seconds, 128 MiB address-space
growth, zero descendants, and a 60-second supervisor wall deadline. The native
synthetic harness verifies the effective CPU, address-space, descendant,
descriptor, and file-size limits after XPC launch. This profile still fixes
`analyzer_execution` and `production_admitted` to false.

## Signing and release order

Nested code is signed from the innermost XPC executable/bundle outward to the
host application. The complete sealed bundle is archived, notarized, stapled,
and verified before its immutable release checksum and provenance are emitted.
The cask refers only to that accepted artifact and checksum.

Signing credentials exist only in the protected macOS release environment.
The application, test fixtures, cask, tap, and ordinary runtime never contain
or inspect them.

## Homebrew lifecycle

- Install: move the sealed bundle and create the CLI-compatible link.
- Upgrade: replace the complete bundle only after cask verification; never
  patch nested components independently.
- Rollback: install another complete previously accepted bundle.
- Uninstall: remove the link and bundle, leaving workspaces untouched.
- Migration: detect the old formula first and require one deterministic
  transition; never select between two installed supervisors implicitly.

Linux retains a normal formula and separate platform confinement backend. The
tap may contain both platform-appropriate definitions, but release automation
must not assume their artifacts or acceptance gates are interchangeable.

## Verification stages

1. Ad hoc synthetic hybrid-XPC resource/lifecycle matrix. Complete on the
   recorded development host.
2. Frozen production resource and launch protocols with fault injection.
   Complete for the synthetic candidate; no analyzer or production claim.
3. Developer ID nested-signing and notarization rehearsal.
4. Local test cask: install, CLI link, identity, upgrade, rollback, migration,
   uninstall, and retained-user-state tests.
5. Clean-machine Gatekeeper and supported-macOS matrix.
6. Reviewed tap update and explicit release publication.

Stages 3 through 6 cannot be inferred from the ad hoc prototype. The subsequent
Tier A checkpoint found aggregate-disk and cross-job-container isolation
failures, so stages 3 through 6 are not pursued as IAR-1B admission gates for
this exact runtime topology. Option C remains the packaging architecture for a
future admitted macOS backend or defense-in-depth distribution.
