# IAR-1B Linux delegated-cgroup feasibility checkpoint

- Status: component candidate passed; composite Linux admission pending
- Scope: ADR-0074 synthetic resource and lifecycle component only
- Profile: `iar-linux-cgroup-synthetic-v1`

## Boundary

The GitHub-hosted Ubuntu job may use `sudo systemd-run` only to create one
ephemeral transient service with `Delegate=yes`. The service runs as the
unprivileged runner user. It creates no persistent unit, accesses no credential
or network service, and receives no repository content as analyzer input.

Inside the delegated subtree the unprivileged supervisor enables only the CPU,
memory, and pids controllers. Each original synthetic worker enters a fresh
leaf atomically through `CLONE_INTO_CGROUP`; no stop-and-move placement is
accepted as evidence.

## Frozen checks

- CPU bandwidth plus a one-second CPU ceiling;
- 32 MiB memory ceiling with observed local OOM kill;
- one-process ceiling with descendant denial;
- exact `cgroup.kill` and `populated 0` verification;
- 64 KiB retained-output ceiling;
- bounded wall timeout;
- crash followed by a clean relaunch;
- leaf removal and cross-job kill isolation.

## Claim boundary

A complete pass sets only `resource_lifecycle_confined=true`. The receipt fixes
`os_confined=false`, `production_admitted=false`, `source_retained=false`, and
`authority_added=false`. Linux IAR-1B remains pending until the primitive and
resource/lifecycle components are composed source-free and repeated on every
claimed kernel and architecture. No real analyzer is executed.

## Hosted evidence

PR 130 job `99194709845` passed on the GitHub-hosted Ubuntu 24.04 image with
kernel `6.17.0-1022-azure` and architecture `x86_64`. The exact checkpoint
summary was:

```text
Linux delegated-cgroup feasibility: result=candidate_passed kernel=6.17.0-1022-azure arch=x86_64
```

The same job's ordinary primitive checkpoint remained `unsupported` because
its non-transient job cgroup was not delegated. This is expected and prevents
component evidence from being laundered into a composite IAR-1B claim.
