# Linux IAR-1B Production Topology ARD

- Status: Accepted
- Date: 2026-08-30
- Decision: ADR-0078

## Platform Constraint

cgroup v2 containment begins only after a parent delegates a subtree. The
delegatee may then create and manage children but cannot move processes across
the delegation boundary. On systemd systems, `Delegate=` on a service or scope
is the supported single-writer boundary; systemd owns the unit cgroup and the
delegate owns only descendants.

Primary platform references:

- [Linux kernel cgroup v2 delegation](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html#delegation)
- [systemd control-group delegation](https://systemd.io/CGROUP_DELEGATION/)

## Selected Rootless Topology

```text
existing systemd user manager (platform-owned delegation)
                         |
                         v
       foreground transient Impresari user service/scope
                         |
                         v
           supervisor child cgroup (unprivileged)
                         |
                         v
       one profile-limited leaf per synthetic/analyzer job
```

The rootless candidate never talks to the system manager, invokes `sudo`, or
changes package policy. It verifies that the current user unit is genuinely
delegated, moves the supervisor into a child, enables only CPU, memory, and pids
controllers at the now-empty delegated root, and atomically creates each worker
inside a pre-limited leaf. All existing Landlock, seccomp, descriptor,
zero-write, resource, kill, empty-state, and cleanup gates remain unchanged.

## Preflight Contract

Preflight is source-free and returns a closed receipt containing only bounded
platform metadata and booleans. It must verify:

- unified cgroup v2 and exact mount/delegation identity;
- current process containment within the selected user unit;
- effective delegation marker or equivalent verified ownership;
- required controllers available to that unit;
- exclusive management of descendants under one fresh unit;
- ability to create/remove a synthetic child without crossing the boundary;
- no inherited source, cache, credential, network, or analyzer authority.

Any failure returns a non-feasible state such as `unsupported`, `unavailable`,
`insufficient_delegation`, or `invalid_contract`. Preflight cannot install
packages, start the system user manager, change login policy, contact D-Bus
outside the selected user manager, or request privilege.

The first implemented host slice is read-only. It reads only fixed procfs,
cgroupfs, and local user-manager transport identities; suppresses raw cgroup
paths from receipts; and never invokes `systemctl`, `systemd-run`, D-Bus, sudo,
or a worker. `ready_for_synthetic_rehearsal` is therefore eligibility evidence,
not proof of the synthetic child cycle, OS confinement, or production support.

The next rootless slice uses that ready state to request exactly one foreground
transient service from the existing user manager. Its `Delegate=` list is
limited to CPU, memory, and pids; it runs only the frozen original-synthetic
composite; and `--wait --pipe --collect` binds completion and removal to the
foreground invocation. The launcher verifies the exact unit becomes
`not-found`. No system-manager call, sudo, persistent unit, raw cgroup path,
repository-derived input, or real analyzer is permitted.

## Selected External Delegation Profile

The externally managed profile accepts an inherited directory/file-descriptor
capability only through a closed launcher contract. The caller must prove
ownership and containment; the supervisor revalidates the boundary and never
accepts a raw arbitrary path as authority. This profile is independently
admitted and cannot borrow the rootless desktop result.

The first external-profile slice fixes the inherited descriptor at slot 3,
rejects caller-selected slots and paths, verifies that it is a directory, and
marks it close-on-exec before any future worker launch. Transport success does
not prove cgroup delegation. Unified cgroup v2, ownership, current-process
containment, exclusive descendants, and CPU/memory/pids delegation must all be
revalidated through the capability before the first mutation.

## Deferred Administrator Profile

An administrator-provisioned profile would require a root-owned unit, exact
executable identity, a closed UID/job request, explicit local authorization
policy, bounded lifecycle, and complete package removal. It must never accept a
command, argv, environment, repository path, network destination, or analyzer
choice. No administrator-profile code or policy is authorized by this
decision.

## Packaging Consequences

The rootless profile adds no privileged package component. A Linux
formula/package may install only the CLI and first-party worker; runtime
support is conditional on an already-valid user delegation. The externally
managed profile adds documentation and a closed integration contract. An
administrator-provisioned profile would be a separate privileged package slice
with stronger signing, upgrade, rollback, and uninstall requirements.
