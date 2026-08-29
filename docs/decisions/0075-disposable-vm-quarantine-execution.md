# ADR-0075: Disposable VM-backed quarantine execution

- Status: Proposed; implementation not authorized
- Date: 2026-08-26
- Scope: Dynamic execution of admitted repositories on macOS and Linux hosts

## Context

Read-only Context evidence and isolated static analyzers reduce uncertainty but
cannot observe all runtime behavior. Package installation, builds, tests,
servers, installers, and delayed or environment-aware payloads may behave
differently when executed. Running an unfamiliar repository directly on a
developer host can expose credentials, other source, browser sessions, cloud
access, local services, and the network.

Containers improve isolation but may share the host kernel and are commonly
weakened by bind mounts, host networking, privileges, capabilities, or access to
container-control sockets. Dynamic analysis therefore needs a separate,
stronger, disposable boundary and cannot be added to the Context or Analyzer
Runner process.

## Decision

1. Implement dynamic repository execution only in a separately packaged
   Disposable Quarantine Runner.
2. Require a fresh, exact, unexpired `isolated_execution_eligible` decision
   naming the exact snapshot, quarantine profile, image, and action plan.
3. Use a fresh VM-backed environment for every job. Containers alone do not
   satisfy the quarantine boundary.
4. Transfer an immutable snapshot copy into a guest-private writable workspace;
   never mount the original host repository, home, cache, or credential stores.
5. Give the guest no real credential and deny network by default at a boundary
   outside guest control.
6. Execute only a closed externally approved action/argv. Repository content
   cannot define host commands, VM settings, mounts, network, credentials,
   evidence, retention, or cleanup.
7. Enforce independent host and guest resource, process-tree, lease,
   cancellation, and teardown controls.
8. Produce an immutable behavior report with observed processes, filesystem
   delta, network attempts, resources, policy violations, sensor completeness,
   unknowns, and exact provenance.
9. Destroy the guest, writable state, ephemeral identity, and transfer copy
   after every job; cleanup failure quarantines the provider from new work.
10. Target macOS developer hosts and Linux CI first through a provider-neutral
    interface.
11. Keep Windows dynamic execution as a future Windows-guest provider requiring
    its own threat model, conformance, licensing, image-servicing, and approval.
12. Never emit a safe, clean, trusted, malware-free, or ordinary-host execution
    verdict.

## Consequences

### Positive

- Untrusted execution is separated from the normal workstation and static
  evidence/scanner processes.
- Exact admission, image, action, and profile identities make runs reproducible
  and auditable.
- Offline-first operation sharply limits credential and network exfiltration.
- Per-job destruction reduces persistence and cross-repository contamination.
- Provider neutrality leaves space for native macOS, Linux, and future Windows
  implementations without pretending their isolation is identical.

### Costs

- VM image building, patching, provenance, storage, boot time, host integration,
  sensors, and cleanup create substantial operational work.
- macOS and Linux providers require separate native conformance evidence.
- Offline builds may fail because dependencies are not present; mediated mirrors
  require a later security phase.
- Sandbox-aware, delayed, kernel-level, architecture-specific, and Windows-only
  malware may remain unobserved.
- Hypervisor and guest tooling become part of the trusted computing base.

## Alternatives Considered

### Run directly on the developer host

Rejected because process permissions would expose the user's account, files,
credentials, sessions, services, and network.

### Use Docker containers as the sole boundary

Rejected for the quarantine claim because container security depends on the
host kernel and is easily weakened by mounts, privileges, host namespaces, and
control sockets. Containers may still run inside the guest.

### Add execution to the Analyzer Runner

Rejected because static scanner confinement and whole-application dynamic
execution have different authority, images, network, lifecycle, and evidence
requirements.

### Allow unrestricted Internet with monitoring

Rejected initially because monitoring does not prevent secret exfiltration,
attacks on third parties, lateral movement, or payload retrieval.

### Reuse warm guest VMs and dependency caches

Rejected initially because writable reuse creates contamination, poisoning, and
cross-job confidentiality risks. Performance optimization can be reconsidered
only with equivalent isolation evidence.

### Include Windows dynamic execution immediately

Rejected as an initial implementation target, not as a product direction.
Windows requires its own guest licensing, image servicing, native persistence
and credential instrumentation, and Hyper-V/provider conformance. Windows
static analysis remains initial scope in steps 1 and 2.

## Verification

- A guest cannot access host home, source repositories, credentials, agents,
  sockets, devices, processes, LAN, or metadata endpoints in the supported
  provider matrix.
- Offline profiles emit no external network packets.
- Stale or mismatched admission/image/profile/action input is denied before VM
  creation or execution.
- Cancellation, timeout, supervisor loss, resource exhaustion, guest shutdown,
  and fork/daemon behavior terminate the complete VM.
- Original source remains byte-for-byte unchanged.
- No writable job state is reused; cleanup is verified or the provider is
  quarantined.
- Behavior reports bind all identities and make sensor failures and truncation
  explicit.
- Windows binaries are not silently executed under an unsupported Linux guest.
- No output field or UI language claims the repository is safe.

## Implementation Gate

This ADR authorizes no VM creation, guest/image download, code execution,
network access, or pilot. Implementation requires completed and accepted steps
1 and 2, a dynamic-execution threat-model update, exact provider/image records,
incident readiness, independent security review, and explicit founder approval
for each phase and pilot scope.

## Review Triggers

Review or supersede before container-only isolation, source/host bind mounts,
shared writable guests/caches, real credentials, package mirrors, external
egress, inbound connectivity, Windows dynamic execution, multi-tenant/hosted
operation, automated artifact application, or ordinary-host execution
authorization.
