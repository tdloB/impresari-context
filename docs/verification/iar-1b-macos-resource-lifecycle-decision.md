# IAR-1B macOS Resource And Lifecycle Decision

- Date: 2026-08-29
- Decision: ADR-0074
- Candidate: App Sandbox host with a private XPC service
- Result: Not admitted; macOS remains at IAR-1A

## Question

Can the independently distributed App Sandbox/private-XPC candidate satisfy
the remaining IAR-1B hard CPU, memory, process-count/tree, timeout, and cleanup
requirements without a privileged daemon, private API, VM, or weaker claim?

## Documented platform constraints

The installed macOS `xpcservice.plist(5)` contract exposes service type,
run-loop type, audit-session joining, and environment variables. It does not
expose the `launchd.plist(5)` hard-resource dictionaries for a private embedded
XPC service. XPC services launch on demand and may be terminated after an idle
period, but that is not a caller-selected hard deadline or a deterministic
process-tree reap contract.

The installed `getrlimit(2)` contract provides useful but incomplete controls:

- `RLIMIT_CPU` limits CPU seconds for each process;
- `RLIMIT_FSIZE` limits file size and `RLIMIT_NOFILE` limits open files;
- `RLIMIT_NPROC` limits simultaneous processes for the entire user ID, not one
  Impresari job;
- `RLIMIT_RSS` expresses a resident-set preference used under memory pressure,
  not an unconditional hard memory ceiling; and
- limits are per process and inherited by processes it creates, but they do not
  supply a job object that proves every descendant is found and reaped.

The broader `launchd.plist(5)` contract describes resource-limit keys for
launchd jobs, but its process-count limit is also per user ID and its
resident-set behavior is not a hard per-job memory kill boundary. Installing a
separate privileged or persistent launchd supervisor would create a different
packaging, authority, patching, and removal architecture and is not authorized
by the XPC feasibility decision.

## Decision

The candidate materially fails the frozen IAR-1B resource and lifecycle gate.
The earlier native prototype remains valid evidence for selected access-control
denials, but it is not adopted as an analyzer backend. macOS remains
`application_enforced` at IAR-1A and reports IAR-1B analyzer execution as
unsupported.

Impresari Context will not replace hard per-job memory, process-tree, timeout,
and teardown requirements with advisory limits, per-user controls, idle
reclamation, or a claim based only on absence of entitlements. No Developer ID
credential, notarization operation, release package, Homebrew cask, real
analyzer, repository artifact, network service, or production launch daemon is
needed or used for this decision.

The next platform evaluation is Linux using independently verified
`no_new_privs`, Landlock, seccomp, descriptor closure, and a delegated cgroup v2
leaf. Linux remains unsupported unless every required primitive and effective
policy check passes without root or a permissive fallback.

## Reproduction references

```sh
man 2 setrlimit
man 5 xpcservice.plist
man 5 launchd.plist
```

This is a documented-interface feasibility decision, not an escape-test claim
for controls that macOS does not provide under the selected architecture.
