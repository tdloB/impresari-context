# IAR-1B Linux composite feasibility checkpoint

- Status: implemented; hosted evidence pending
- Scope: exact-host, source-free ADR-0074 candidate only
- Profile: `iar-linux-synthetic-v1`

## Boundary

The checkpoint runs only on an ephemeral GitHub-hosted Ubuntu runner. It uses
`sudo systemd-run` once to create one temporary `Delegate=yes` transient
service, then continues as the unprivileged runner user. It creates no
persistent service and does not access a network, credential, real analyzer,
or repository artifact as analyzer input.

The supervisor moves itself out of the delegation root, enables only CPU,
memory, and pids, and creates fresh job leaves. For the composite job it writes
the frozen limits before calling `clone3(CLONE_INTO_CGROUP)`. The child is
therefore born inside the limited leaf; it is never forked into an ordinary
cgroup and moved afterward.

## Composite gate

The atomically placed worker must reproduce, in that process:

- effective `no_new_privs`;
- read-only Landlock access to one exact original-synthetic input;
- external-file, credential-canary, device, and path-write denial;
- architecture-pinned default-deny seccomp;
- network and descendant denial; and
- unrelated-descriptor closure.

The same delegated service must reproduce the frozen CPU, memory, process,
exact-kill, empty-state, bounded-output, timeout, crash/relaunch, cleanup, and
cross-job checks. Historical component receipts are not inputs to the result.

## Claim boundary

A complete result may set `os_confined=true` only for the exact observed
kernel and architecture candidate. It always keeps
`production_admitted=false`, `source_retained=false`, and
`authority_added=false`. It does not execute or admit YARA, ClamAV, or any real
analyzer. Additional independently admitted kernel and architecture evidence
remains required before Linux is a production backend or IAR-2 opens.
