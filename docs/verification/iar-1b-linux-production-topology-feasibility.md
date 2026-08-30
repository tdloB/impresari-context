# IAR-1B Linux Production-Topology Feasibility

- Date: 2026-08-30
- Decision: ADR-0078
- Scope: source-free topology contracts and bounded read-only rootless host preflight
- Production admitted: No
- Real analyzer authorized: No
- Privileged installation authorized: No

## Selected Boundary

The accepted policy selects two independently evaluated profiles:

- `rootless_user_manager`: use an existing systemd user manager and a
  foreground transient user service or scope;
- `externally_managed`: accept only an inherited directory file descriptor for
  an operator-provisioned delegated subtree.

The administrator-provisioned profile is deferred. Automatic `sudo`, `pkexec`,
privileged daemons, raw cgroup paths, and application-only fallback while
claiming IAR-1B are denied.

## Closed Feasibility Contract

`linux-iar-1b-production-topology-v1.json` freezes the selected profiles, the
unsupported fallback, and the evidence trigger for reconsidering privileged
installation. The source-free evaluator consumes only caller-supplied bounded
metadata. It performs no host discovery, service mutation, execution, network
access, credential access, or repair.

The evaluator checks unified cgroup v2, the selected parent contract, effective
delegation, CPU/memory/pids controller availability, process containment,
exclusive descendant ownership, and one declared synthetic child lifecycle.
The external profile additionally requires a verified inherited directory file
descriptor and rejects arbitrary paths.

## Bounded Rootless Host Preflight

`linux-rootless-host-preflight.rb` adds the first live host observation for the
default profile. It has no arguments and reads only the bundled topology policy
and fixed Linux platform files: kernel release, the current cgroup membership,
the unified cgroup interface, the current UID's systemd user-manager cgroup,
and its local user-manager transport socket. It records no raw cgroup path,
repository identity, username, environment, source, cache, or credential data.

The observer performs no child-process launch, D-Bus request, service mutation,
write to cgroupfs, network access, privilege request, or repair. A ready result
means only that an existing user manager, required controllers, and a writable
delegation marker were observed. The synthetic child lifecycle remains
explicitly unexecuted, and OS confinement, production, privileged installation,
and real analyzers remain closed.

## Deterministic Verification

Run:

```sh
ruby scripts/check-linux-isolation-topology-feasibility.rb
ruby scripts/check-linux-rootless-host-preflight.rb
```

The checker covers two feasible candidates and seven fail-closed cases:
deferred administrator topology, legacy cgroups, unavailable user manager,
missing controller, raw external path, unverified external boundary, and failed
synthetic child lifecycle. Every receipt keeps production, real analyzers, and
privileged installation closed and denies all authority.

Schema conformance accepts the policy and rootless receipt and rejects a receipt
that attempts to overclaim production or analyzer readiness. Fixture provenance
records original synthetic content only.

The host-preflight checker adds one synthetic ready state and nine fail-closed
states covering non-Linux, unavailable and legacy cgroups, malformed current
membership, three unavailable user-manager signals, missing controllers, and a
missing delegation write marker. Hosted Linux jobs print their bounded live
receipt without treating an unavailable user manager as a CI failure or
attempting elevation.

## Next Gate

This increment proves the bounded read-only portion of the rootless host path.
The next Linux checkpoint must create one foreground transient unit only through
an already-running systemd user manager, reproduce the complete source-free
synthetic confinement corpus below that unit, and verify cleanup. Hosts whose
preflight is unavailable remain unsupported; there is no sudo or privileged-
service fallback. The external profile still requires its own independent
launcher and corpus. Neither checkpoint may execute a real analyzer or admit
production.
