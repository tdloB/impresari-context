# IAR-1B OS-Confinement Feasibility Record

- Date: 2026-08-29
- Decision: ADR-0074
- Scope: read-only platform inventory; no confinement implementation or claim

## Observed macOS host

The local design host reported macOS `26.5.1` (`25F80`) on arm64 with Darwin
kernel `25.5.0`. `/usr/bin/sandbox-exec`, `/bin/launchctl`, and
`/usr/sbin/taskpolicy` were present.

The installed `sandbox-exec(1)` manual explicitly marks the command deprecated
and directs application developers toward App Sandbox. Presence of that binary
therefore does not establish a durable product confinement mechanism. The
repository's existing network-denied test may continue using `sandbox-exec` as
a bounded test harness, but that test is not evidence that an analyzer worker
is generally filesystem-, process-, handle-, device-, or network-confined.

## Design consequences

- Do not promote the existing network-denied test harness into the Analyzer
  Runner's production security boundary.
- Do not claim macOS OS confinement from a deny-network profile alone.
- Evaluate a separately signed and entitled macOS worker/launcher architecture
  against App Sandbox distribution, update, subprocess, staged-input, and
  audit requirements before selecting it.
- Keep the current `application_enforced` posture and all OS/network/descendant
  limitations until an admitted mechanism passes the complete Tier A suite.
- Evaluate Linux and Windows independently; this macOS inventory provides no
  evidence for namespaces/seccomp/cgroups or restricted tokens/job objects.

## Apple-documented architecture constraints

Apple describes App Sandbox as a kernel-enforced access-control boundary whose
capabilities are restored through signed entitlements. Apple also states that
an embedded command-line tool must inherit the containing app's sandbox. For a
direct child, the documented inheritance contract carries only the parent's
static entitlement rights—not file access granted dynamically after launch.
Apple recommends an XPC service over a child process for privilege separation,
and describes an XPC service as privately embedded, launchd-managed, and
sandboxed with a minimal default environment.

These constraints make the credible macOS candidate a separately signed app
bundle containing a minimal XPC service or inheriting helper, with staged bytes
passed through a closed IPC contract. They do not support treating the existing
standalone Cargo CLI plus `Command::new` as App Sandbox-confined. Before that
candidate can be selected, the roadmap needs evidence for:

- Developer ID/App Store signing and entitlement custody;
- non-App-Store distribution and notarization compatibility;
- exact staged-byte transfer without broad user-selected-file grants;
- helper/XPC identity, lifecycle, crash, timeout, and update pinning;
- absence of network entitlements and verification of effective denial;
- bounded service restart behavior and complete-result semantics; and
- removal that leaves no container, launch service, credential, or retained
  source data beyond the documented macOS lifecycle.

Primary references:

- [Configuring the macOS App Sandbox](https://developer.apple.com/documentation/xcode/configuring-the-macos-app-sandbox)
- [Protecting user data with App Sandbox](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox)
- [Enabling App Sandbox inheritance](https://developer.apple.com/library/archive/documentation/Miscellaneous/Reference/EntitlementKeyReference/Chapters/EnablingAppSandbox.html)
- [Creating XPC services](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingXPCServices.html)
- [Accessing files from the macOS App Sandbox](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox)

## Linux candidate composition

The Linux kernel documents complementary primitives rather than one complete
runner sandbox:

- `no_new_privs` persists across `fork`, `clone`, and `execve` and prevents an
  executed program from gaining privileges through set-ID bits or file
  capabilities;
- seccomp BPF reduces the available syscall surface and, when fork/exec remain
  allowed, constrains descendants with the inherited filter;
- Landlock supplies unprivileged, stackable filesystem restrictions and
  descendant inheritance, with ABI-dependent filesystem, TCP/UDP, IPC, and
  ioctl coverage; and
- cgroup v2 supplies hierarchical memory, process-count, CPU, and I/O controls,
  but usable delegation and enabled controllers are host configuration, not a
  portable assumption.

The Linux candidate is therefore a tiny exact-pinned launcher that installs
`no_new_privs`, closes unrelated descriptors, applies a version-negotiated
deny-by-default Landlock ruleset and architecture-specific seccomp filter, then
executes the worker inside a pre-created delegated cgroup v2 leaf. Admission
must fail closed when a required ABI, filter action, controller, delegation, or
kill/empty-state verification is unavailable. Landlock explicitly documents
special-filesystem and ABI limitations, so Landlock alone cannot satisfy the
Runner threat model.

Primary references:

- [Landlock unprivileged access control](https://www.kernel.org/doc/html/latest/userspace-api/landlock.html)
- [No New Privileges](https://www.kernel.org/doc/html/latest/userspace-api/no_new_privs.html)
- [Seccomp BPF](https://www.kernel.org/doc/html/latest/userspace-api/seccomp_filter.html)
- [Control Group v2](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)

## Windows candidate composition

Microsoft documents AppContainer as isolating files/registry, network,
processes, windows, devices, and credentials unless capabilities grant access.
Job Objects separately group a process hierarchy, enforce resource limits and
accounting, and can terminate associated descendants when the last job handle
closes with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Job Objects do not replace
per-process security restrictions; Microsoft requires those to be applied to
each process on current Windows versions.

The Windows candidate is an exact-pinned worker launched with an AppContainer
or low-privilege restricted token, no network capability, a non-breakaway Job
Object, headless mitigation policy, explicit staged-object ACLs, and hard
memory/CPU/process limits. The newer `Create Process In Sandbox` API may compose
some of these controls, but its target-version, packaging, servicing, API
availability, and exact effective-policy evidence must be established before
selection. Admission must fail closed if AppContainer identity, ACLs, job
assignment, mitigation, or kill-on-close verification is missing.

Primary references:

- [AppContainer isolation](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-isolation)
- [Launching an AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
- [Create Process In Sandbox APIs](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox)

## macOS prototype follow-up

The separately authorized, synthetic-only App Sandbox/XPC prototype is
recorded in [IAR-1B macOS App Sandbox/XPC feasibility evidence](iar-1b-macos-xpc-feasibility.md).
It demonstrated a private XPC transport and several native denial boundaries,
but it did not establish hard resource/process-tree controls, a rehearsed
timeout, or complete OS-managed container removal. It therefore keeps
`os_confined` and `production_admitted` false and does not select a backend.
The follow-up
[resource and lifecycle decision](iar-1b-macos-resource-lifecycle-decision.md)
finds the remaining hard per-job controls unavailable under the selected
documented architecture. macOS remains at IAR-1A and Linux is evaluated next.

## Cross-platform selection gate

No platform candidate is selected by this inventory. Selection requires an
exact minimum OS/kernel matrix, packaging and signing boundary, dependency and
license review, reproducible build plan, removal semantics, and Tier A tests
that attempt filesystem, descriptor/handle, credential, device, process,
network, resource, mutation, crash, timeout, orphan, and cleanup escapes. A
platform that lacks any required primitive remains unsupported rather than
falling back to application-only supervision under an OS-confinement claim.

## Evidence commands

```sh
uname -a
sw_vers
command -v sandbox-exec launchctl taskpolicy
man sandbox-exec
```

No analyzer, repository artifact, network operation, credential, upload,
quarantine, or hostile-format parser was used during this inventory.
