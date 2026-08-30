# IAR-1B Linux feasibility checkpoint

- Date: 2026-08-29
- Decision: ADR-0074
- Scope: source-free synthetic capability and primitive evidence only
- Profile: `iar-linux-synthetic-v1`
- Admission result: `unsupported` on the observed hosted Ubuntu runner; Linux is unadmitted

## Architecture correction

The earlier Linux candidate correctly combined `no_new_privs`, Landlock,
seccomp, descriptor closure, and cgroup v2, but it assigned too much to cgroup
v2. Cgroup v2 can constrain CPU, memory, process count, and I/O behavior; it
does not provide a portable aggregate file-storage quota.

The first candidate profile therefore exposes one read-only staged input and
no writable path-backed filesystem. Output and diagnostics remain bounded
pipes. Its aggregate writable-filesystem budget is exactly zero. This is a
deliberately narrow fit for the later pinned YARA adapter, not a promise that
every analyzer can run without temporary storage. An analyzer that requires a
writable temporary filesystem needs a separately admitted quota mechanism and
profile.

## Exact candidate composition

The source-free synthetic candidate requires all of the following:

1. `PR_SET_NO_NEW_PRIVS` is set and verified before confinement.
2. A version-negotiated Landlock ruleset grants read-only access to exactly one
   synthetic job directory and denies external files, credential canaries,
   devices, and every path-backed write.
3. An architecture-pinned, default-deny seccomp filter admits only the tiny
   synthetic probe syscall surface and denies network socket and descendant
   creation.
4. Every unrelated inherited descriptor is closed before the filter becomes
   effective.
5. The worker is placed in a pre-created delegated cgroup v2 leaf with the
   frozen CPU, memory, and process limits. Exact `cgroup.kill` and empty-state
   verification are mandatory.
6. The supervisor retains bounded output, wall-time, crash/relaunch, exact
   target, cleanup, and cross-job verification responsibility.

The first native checkpoint implements items 1–4 and inventories item 5. It
keeps the resource and lifecycle checks false until a genuinely delegated leaf
is available and the complete suite passes. Merely observing cgroup v2 or its
controllers is not delegation evidence.

## Fail-closed states

The receipt reports `unsupported` when any required ABI, architecture filter,
cgroup v2 controller, delegation, kill primitive, or empty-state verifier is
unavailable. It reports `partial` only when the implemented primitive checks
pass and the remaining resource/lifecycle suite is explicitly pending. Both
states keep `os_confined` and `production_admitted` false.

The conformance schema rejects `candidate_passed` unless every preflight and
Tier A check is true. CI success means the host was measured and the receipt
was honest; it does not mean the host was admitted.

## Evidence and nonclaims

The harness compiles an original test-only C probe on the native Linux host. It
uses only synthetic canaries under the build directory. It does not execute an
analyzer, read repository content, access credentials, contact a network,
install a service, configure a host cgroup, invoke `sudo`, upload an artifact,
or claim that the C probe is a production launcher.

The frozen fixture corpus contains JSON only. Provenance and exact SHA-256
digests are recorded in
`tests/conformance/v1/linux-isolation-fixture-provenance.json`.

## Observed hosted result

The GitHub-hosted Ubuntu 24.04 `x86_64` job observed kernel
`6.17.0-1022-azure` and Landlock ABI 7. The native primitive suite completed,
which establishes effective `no_new_privs`, one read-only synthetic Landlock
input, external-file, credential, device, and path-write denial, the exact
`x86_64` default-deny seccomp filter, network and descendant denial, and
unrelated-descriptor closure for that host.

The current job cgroup was not delegated. The closed receipt therefore
reported `result=unsupported`, `delegated_cgroup=false`, `os_confined=false`,
and `production_admitted=false`. This is the intended fail-closed outcome and
does not satisfy IAR-1B.

## Next gate

After native capability evidence is recorded, the next increment must provide
a controlled delegated cgroup v2 leaf without granting the worker additional
authority. It must then prove CPU, memory, process count, exact kill,
empty-state, bounded output, timeout, crash/relaunch, cleanup, and cross-job
isolation. If the hosted environment cannot provide that boundary, Linux
remains unsupported there; the product must not silently substitute
application-only supervision.
