# IAR-1B macOS App Sandbox/XPC Feasibility Evidence

- Date: 2026-08-29
- Decision: ADR-0074
- Prototype: `iar-macos-xpc-hybrid-feasibility-v4`
- Result: Partial; not production-admitted

## Scope

This record covers one development-only, synthetic macOS feasibility
prototype. It does not package or run an analyzer, read a repository, use a
production signing identity, contact a network service, notarize a release, or
change the ordinary Impresari Context scan path.

The prototype is a non-UI App Sandbox host with one private embedded App
Sandbox XPC service plus a synthetic stand-in for the selected Rust supervisor.
The host sends bounded synthetic requests and accepts bounded source-free
receipts. The service has no network client or server entitlement and receives
only a fixed synthetic payload, four synthetic canary paths, one unrelated
process identifier, and a loopback port controlled by the test harness.

## Observed environment

| Field | Observed value |
| --- | --- |
| macOS | `26.5.1` (`25F80`) |
| Architecture | `arm64` |
| Xcode | `26.6` (`17F113`) |
| Swift | `6.3.3` |
| Signing | Ad hoc development signing only |

The fixture is bound to
`schemas/v1/macos-xpc-sandbox-feasibility.schema.json`. Its closed result keeps
`os_confined`, `production_admitted`, `source_retained`, and `authority_added`
fixed to `false`.

## Demonstrated controls

- the private XPC service launched and returned the exact closed receipt;
- both host and service carried effective App Sandbox entitlements;
- the service carried no network client or server entitlement;
- its own app container remained readable and writable;
- reads of exact synthetic repository, home, cache, and credential canaries
  outside the container were denied;
- access to an exact synthetic pseudo-terminal character device created by the
  harness was denied;
- access to an unrelated process was denied;
- a live unsandboxed loopback listener observed no service connection while
  the service received an OS permission-denied result;
- request and receipt bytes have explicit upper bounds; and
- the receipt contained no source bytes and granted no authority.

The hybrid follow-up also demonstrated:

- one-second `RLIMIT_CPU` termination of an intentional CPU loop;
- an irreversible `RLIMIT_AS` bound derived from the service's startup virtual
  footprint plus 128 MiB, with a one-GiB `mmap` denied;
- `RLIMIT_NPROC=0` denial of both `fork` and `posix_spawn`;
- publication and verification of the exact embedded service PID/path before a
  supervisor terminated a deliberately hung service;
- XPC relaunch in a distinct process after CPU-limit termination; and
- write, read-back, removal, and absence verification for the exact bounded
  synthetic payload inside the service container.

The production-candidate follow-up also froze
`profiles/v1/iar-macos-xpc-hybrid-v1.json` and the source-free Rust-to-host
preparation handshake. The native service reported the exact profile digest and
verified effective 30-second CPU, current-footprint-plus-128-MiB address-space,
zero-descendant, 32-descriptor, and 8-MiB file-size limits. Rust and schema
tests reject repository paths, arbitrary arguments or environment, credentials,
network authority, analyzer execution, unknown fields, mismatched identity,
partial readiness, retained source, and premature confinement or production
claims.

The first decisive Tier A probes then demonstrated two remaining escapes. Nine
separately closed 1 MiB files exceeded the effective 8 MiB per-file limit in
aggregate, and a fresh XPC service process read a synthetic marker left by the
preceding service process in the shared service container. Both probes cleaned
up their synthetic files after recording the result. The detailed checkpoint
is [IAR-1B macOS Tier A checkpoint](iar-1b-macos-tier-a-checkpoint.md).

## Unresolved gates

This prototype fails the aggregate-disk and cross-job-isolation portions of the
Tier A corpus and does not establish the remaining complete corpus,
production signing/notarization, packaging, clean-machine behavior, or
multi-host compatibility. Its device, CPU, address-space-growth, process-count,
descendant, descriptor, file-size, crash/relaunch, exact-target timeout, and
synthetic-byte cleanup evidence is native but remains development-only.

The app's synthetic container contents were removed after the rehearsal.
macOS retained only its protected `.com.apple.containermanagerd.metadata.plist`
records and denied ordinary deletion of those records. Complete container
removal is therefore OS-managed and unverified, not claimed as a passing
cleanup gate.

Developer ID signing, notarization, Homebrew packaging, update compatibility,
and a second native host have not been rehearsed. No production signing key or
notarization credential was inspected or used.

## Decision consequence

The
[resource and lifecycle decision](iar-1b-macos-resource-lifecycle-decision.md)
corrected the earlier all-in-one assumption and selected the hybrid architecture
for continued feasibility. The decisive Tier A checkpoint now shows that exact
topology is insufficient for IAR-1B without another confinement layer. ADR-0076
still selects one CLI-compatible Homebrew cask as the intended packaging
topology, but packaging cannot convert the failed runtime boundaries into an
admission. macOS remains IAR-1A and IAR-2 YARA execution stays closed.

## Reproduction

On macOS with Xcode command-line tools, `jq`, and LaunchServices available:

```sh
./scripts/check-macos-xpc-feasibility.sh
cargo test -p context-conformance --test schema_conformance --locked
```

The first command builds only under
`target/iar-macos-xpc-feasibility`, uses ad hoc development signing, and creates
only synthetic canaries. On non-macOS hosts it reports the check as not
applicable.
