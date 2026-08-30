# IAR-1B Linux External Delegation Capability Contract

- Date: 2026-08-30
- Decision: ADR-0078
- Profile: `externally_managed`
- Scope: source-free inherited-capability transport only
- Cgroup mutation: No
- Production admitted: No
- Real analyzer authorized: No

## Boundary

The external launcher may convey exactly one already-open directory descriptor
at fixed slot 3. Impresari accepts no cgroup path, configurable descriptor
number, command, environment, source location, credential, service request, or
privilege request. The receiver verifies that slot 3 is open and refers to a
directory, immediately marks it close-on-exec, and records neither descriptor
identity nor a raw cgroup path.

This is capability transport, not delegation proof. Before a future host-bound
rehearsal may mutate a descendant, it must independently verify through the
descriptor that the filesystem is unified cgroup v2, the boundary is owned by
the launching identity, the supervisor is contained, descendants are
exclusively managed, and CPU, memory, and pids controllers are delegated.

## Deterministic Evidence

Run:

```sh
ruby scripts/check-linux-external-delegation-capability.rb
```

On Linux the checker launches only the fixed source-free probe with an inherited
descriptor to an original-synthetic temporary directory. It also exercises five
fail-closed states: missing descriptor, raw path, configurable slot, non-directory
descriptor, and descriptor leakage. Non-Linux hosts validate the same closed
receipt contract without pretending to rehearse Linux descriptor transport.

The valid receipt keeps `os_confined=false`, production false, analyzers false,
privileged installation false, and cgroup mutation denied. Schema conformance
rejects an overclaim fixture. Fixture provenance records original synthetic
content only.

## Next Gate

An operator-provided ephemeral Linux environment must pass bounded live
revalidation and the complete original-synthetic confinement corpus below the
inherited delegated subtree. That rehearsal must not accept a path, request
sudo, install a service, use repository input, access credentials or network,
or run a real analyzer. Failure leaves the external profile unsupported.
