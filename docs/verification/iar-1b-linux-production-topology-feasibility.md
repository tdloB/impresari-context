# IAR-1B Linux Production-Topology Feasibility

- Date: 2026-08-30
- Decision: ADR-0078
- Scope: source-free topology contracts only
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

## Deterministic Verification

Run:

```sh
ruby scripts/check-linux-isolation-topology-feasibility.rb
```

The checker covers two feasible candidates and seven fail-closed cases:
deferred administrator topology, legacy cgroups, unavailable user manager,
missing controller, raw external path, unverified external boundary, and failed
synthetic child lifecycle. Every receipt keeps production, real analyzers, and
privileged installation closed and denies all authority.

Schema conformance accepts the policy and rootless receipt and rejects a receipt
that attempts to overclaim production or analyzer readiness. Fixture provenance
records original synthetic content only.

## Next Gate

This increment proves contract behavior, not the real host path. The next Linux
checkpoint must implement bounded source-free preflight on independently pinned
targets and reproduce the complete synthetic confinement corpus below each
selected topology. It cannot execute a real analyzer or admit production.
