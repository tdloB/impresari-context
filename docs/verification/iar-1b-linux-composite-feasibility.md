# IAR-1B Linux composite feasibility checkpoint

- Status: exact-host candidate passed; broader Linux admission pending
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

## Hosted evidence

PR 131 job `99197119262` passed on the GitHub-hosted Ubuntu 24.04 image with
kernel `6.17.0-1022-azure`, architecture `x86_64`, and Landlock ABI 7. The exact
checkpoint summary was:

```text
Linux composite IAR-1B feasibility: result=candidate_passed kernel=6.17.0-1022-azure arch=x86_64 Landlock ABI=7
```

The ordinary, nondelegated primitive step in the same job continued to return
`unsupported`; the candidate pass came only from the explicitly delegated
single-service composition and does not reinterpret the earlier receipt.

## Independent architecture checkpoint

The next candidate used GitHub's standard ephemeral `ubuntu-24.04-arm` runner.
The probe has a separately architecture-pinned AArch64 audit identity and
default-deny syscall filter; it does not reuse or treat the x86_64 filter as
portable evidence. The dedicated job runs only the original-synthetic primitive
and single-service composite checkpoints. It did not inherit the x86_64 pass.

PR 132 job `99198568879` passed on kernel `6.17.0-1022-azure`, architecture
`aarch64`, and Landlock ABI 7. The exact checkpoint summary was:

```text
Linux composite IAR-1B feasibility: result=candidate_passed kernel=6.17.0-1022-azure arch=aarch64 Landlock ABI=7
```

The result independently admits that exact arm64 candidate only. It does not
broaden either architecture to other kernels or distributions.

Runner availability is grounded in GitHub's current
[hosted-runner documentation](https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job).

## Independent kernel checkpoint

The next held-out matrix uses native standard `ubuntu-22.04` and
`ubuntu-26.04` runners. GitHub's current image records identify materially
different 6.8 and 7.0 Azure kernel lines. Each matrix invocation runs the
original-synthetic primitive and composite checkpoints and creates exactly one
temporary delegated service. Neither result may inherit the Ubuntu 24.04
candidate, and no container or userspace-only version change counts as kernel
diversity.

Hosted results are pending. The target rationale is grounded in GitHub's
current [Ubuntu 22.04 image record](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2204-Readme.md)
and [Ubuntu 26.04 image record](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2604-Readme.md).
