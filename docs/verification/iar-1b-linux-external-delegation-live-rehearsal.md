# IAR-1B Linux External Delegation Live Rehearsal

- Date: 2026-08-30
- Decision: ADR-0078
- Profile: `externally_managed`
- Input: original synthetic only
- Production admitted: No
- Real analyzer authorized: No

## Provisioner Boundary

One dedicated GitHub-hosted Ubuntu job acts as the external operator. It uses
one temporary systemd system service with exactly `Delegate=cpu memory pids`,
runs it as the unprivileged runner identity, and requests wait, pipe, and
collection semantics. The operator uses privilege only to create that ephemeral
boundary. Impresari neither requests nor receives privilege, and no service is
installed or persisted.

Inside the delegated service, the CI provisioner opens its exact current cgroup
directory and conveys it at inherited descriptor slot 3. The receiver accepts
no path or configurable descriptor, marks slot 3 close-on-exec, and records no
unit name, descriptor identity, or raw cgroup path.

## Revalidation And Corpus

Before mutation the receiver verifies through fixed host metadata and the
descriptor that:

- the host uses unified cgroup v2;
- the descriptor and current process identify the same boundary;
- the boundary is owned and writable by the unprivileged receiver;
- no pre-existing descendant competes for ownership; and
- CPU, memory, and pids controllers are available.

Only then does it run the frozen complete original-synthetic Linux composite.
The inherited descriptor is already close-on-exec, so the worker cannot inherit
the external authority. The composite must prove Landlock, seccomp, network and
filesystem denial, descriptor closure, zero path-backed writes, CPU, memory,
process and output bounds, exact kill, timeout, crash/relaunch, cleanup, and
cross-job isolation. The receiver removes the empty supervisor descendant, and
the outer provisioner verifies the transient service is `not-found` afterward.

## Closed Contract

`linux-external-delegation-live-rehearsal.schema.json` admits an exact-host
candidate only if capability transport, every revalidation check, the complete
composite, descendant cleanup, and provisioner collection all pass. Five
deterministic failure states cover capability, revalidation, composite,
descendant cleanup, and provisioner cleanup failures.

Run the source-free contract checks with:

```sh
ruby scripts/check-linux-external-delegation-live-rehearsal.rb
```

The live script is restricted to ephemeral GitHub-hosted Linux and cannot run a
real analyzer or use repository-derived input. A pass remains exact-host
synthetic candidate evidence; production, package publication, and IAR-2 stay
closed until lifecycle and maintenance gates pass.

## Hosted Evidence

PR 140 run `33295514984`, job `99214382834`, passed on Ubuntu 24.04 x86_64,
kernel `6.17.0-1022-azure`:

- the provisioner service was created and collected;
- operator privilege was used only for that ephemeral service and Impresari
  privilege use remained false;
- descriptor slot 3 was received, verified as a directory, and marked
  close-on-exec without accepting or recording a raw path;
- unified cgroup v2, ownership, process containment, exclusive descendants,
  writability, and CPU/memory/pids delegation all revalidated true;
- the complete composite passed with receipt identity
  `d9bbcbc55831385b3f56962170622cb2f79dbf8a7237573a2c2d8f712d100c2c`;
- delegated descendants were removed; and
- production, real analyzers, and privileged installation remained false.

This independently establishes the selected external profile as an exact-host
synthetic candidate. It does not convert operator provisioning into an
Impresari privilege requirement or admit production.
