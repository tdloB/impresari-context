# IAR-1B macOS App Sandbox/XPC Feasibility Evidence

- Date: 2026-08-29
- Decision: ADR-0074
- Prototype: `iar-macos-xpc-feasibility-v1`
- Result: Partial; not production-admitted

## Scope

This record covers one development-only, synthetic macOS feasibility
prototype. It does not package or run an analyzer, read a repository, use a
production signing identity, contact a network service, notarize a release, or
change the ordinary Impresari Context scan path.

The prototype is a non-UI App Sandbox host with one private embedded App
Sandbox XPC service. The host sends one bounded synthetic request and accepts
one bounded source-free receipt. The service has no network client or server
entitlement and receives only four synthetic canary paths, one unrelated
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
- access to an unrelated process was denied;
- a live unsandboxed loopback listener observed no service connection while
  the service received an OS permission-denied result;
- request and receipt bytes have explicit upper bounds; and
- the receipt contained no source bytes and granted no authority.

## Unresolved gates

This prototype does not establish device denial, hard CPU, memory, disk, or
process-count limits, descendant process-tree containment, or a fault-injected timeout. The
service deliberately does not launch a descendant, so the existing exact-count
production child-launch guard is unchanged.

The app's synthetic container contents were removed after the rehearsal.
macOS retained only its protected `.com.apple.containermanagerd.metadata.plist`
records and denied ordinary deletion of those records. Complete container
removal is therefore OS-managed and unverified, not claimed as a passing
cleanup gate.

Developer ID signing, notarization, Homebrew packaging, update compatibility,
and a second native host have not been rehearsed. No production signing key or
notarization credential was inspected or used.

## Decision consequence

App Sandbox with a private XPC service remains the preferred macOS candidate,
but this partial result does not adopt it as the IAR-1B backend. The macOS
backend remains unsupported for real analyzers until every IAR-1B hard gate is
demonstrated on an exact supported host profile. IAR-2 YARA execution remains
closed.

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
