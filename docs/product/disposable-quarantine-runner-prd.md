# Disposable Quarantine Runner — Product Requirements Document

## Document Control

- Product: Impresari Quarantine Runner.
- PRD ID/version: IC-DQR-PRD-001 / 0.1.
- Status: Proposed; documentation and planning only. Implementation is not
  authorized by this record.
- Date: 2026-08-26.
- Owner: Aaron Boldt.
- Sequence: Security expansion step 3 of 3; depends on accepted and proven steps
  1 and 2.
- Related records:
  - [Hostile Repository Admission PRD](hostile-repository-admission-prd.md)
  - [Isolated Analyzer Runner PRD](isolated-analyzer-runner-prd.md)
  - [Disposable Quarantine Runner ARD](../architecture/disposable-quarantine-runner-ard.md)
  - [ADR-0075](../decisions/0075-disposable-vm-quarantine-execution.md)
  - [System Boundaries](../boundaries.md)
  - [Security Threat Model](../security/threat-model.md)

## Executive Decision

Create a separate, independently releasable Disposable Quarantine Runner for
the controlled dynamic execution of repositories that have passed the required
static admission policy. Each run uses a fresh VM-backed environment, an exact
immutable input snapshot, no host credentials, denied network by default,
bounded externally approved actions, complete process/resource supervision,
and destruction after evidence collection.

Initial host support targets macOS developer machines and Linux CI through a
provider-neutral virtualization interface. Windows-oriented repositories are
statically analyzed in steps 1 and 2, but Windows dynamic execution is a later
provider requiring its own Windows guest, licensing, instrumentation, threat
model, conformance evidence, and founder approval.

## Problem

Static analysis and malware reputation cannot reveal every runtime behavior.
An unfamiliar MVP may download a payload only during installation, inspect the
environment before acting, create persistence, modify unexpected files, contact
local services, or behave differently when tests or a development server run.

Running it directly on a developer workstation could expose credentials,
browser sessions, other repositories, cloud metadata, the local network, and
the operating system. A normal container may still share too much host kernel,
filesystem, networking, or Docker authority. Dynamic observation therefore
requires a stronger disposable boundary and a separate authorization step.

## Product Boundary

The Quarantine Runner is a controlled execution laboratory. It is not:

- an assurance that software is safe;
- permission to run the same software on an ordinary host;
- a production deployment, CI replacement, or developer environment;
- a credentialed integration test environment;
- an automated malware-remediation system;
- a general remote-code execution service;
- a source editor, patch applier, Git client, or delivery system.

## Goals

1. Execute explicitly approved repository actions without exposing the normal
   workstation or CI host filesystem and credentials.
2. Require a fresh, deterministic `isolated_execution_eligible` decision tied
   to the exact input snapshot and quarantine profile.
3. Use fresh VM-backed isolation for every untrusted run.
4. Default to no network and no credentials.
5. Observe processes, filesystem changes, attempted network activity, resource
   use, and policy violations.
6. Preserve bounded evidence without treating runtime silence as safety.
7. Destroy the guest and job state after collection unless local incident
   preservation is explicitly authorized.
8. Support macOS and Linux hosts without making one virtualization technology
   part of the product contract.
9. Preserve a future Windows provider boundary from the start.

## Non-Goals

The initial release will not:

- execute on Windows guests or claim Windows behavioral coverage;
- attach host home directories, SSH agents, password managers, browser profiles,
  Docker sockets, cloud credentials, production data, or unrelated repositories;
- use host networking, privileged containers, host PID/user namespaces, or
  arbitrary host device passthrough;
- allow inbound network connections to the guest;
- permit unrestricted Internet access;
- let repository content define the host command, VM profile, mounts, network
  policy, evidence policy, or cleanup behavior;
- copy arbitrary guest changes back into a trusted repository;
- continue offline beyond a bounded local lease;
- declare a repository clean after a successful run;
- conceal unsupported architecture, dependency, or platform requirements.

## Initial Platform Scope

### macOS hosts

- Apple-silicon macOS is the first developer-host target.
- The provider uses an approved VM technology with a Linux guest matching the
  host architecture.
- Intel macOS remains subject to the existing project platform policy and actual
  test capacity.

### Linux hosts

- Linux x86-64 is the initial CI/server-host target.
- The provider uses hardware virtualization where available and fails visibly
  when the required isolation capability is absent.

### Windows

- Static analysis of Windows artifacts is already initial scope in steps 1 and
  2.
- Windows dynamic execution is planned, not supported initially.
- A future provider must use a Windows guest and address Hyper-V/Windows
  virtualization, NTFS/reparse points, registry/services/tasks/WMI, DPAPI and
  credentials, Windows networking, PE/DLL loading, guest licensing and image
  servicing, telemetry, and Windows-specific evidence collection.
- Windows provider admission requires a separate ADR or explicit revision of
  ADR-0075 plus full security and release gates.

## Users And Jobs

| User | Job |
| --- | --- |
| Founder/evaluator | Observe an unfamiliar MVP without running it on the normal workstation |
| Security reviewer | Inspect what processes, files, and network attempts occurred |
| Developer | Learn whether install/build/test/dev actions violate the approved profile |
| Admission policy owner | Permit only an exact approved action plan and profile |
| Incident responder | Preserve a bounded suspicious run when explicitly authorized |

## Terminology

- **Quarantine profile:** Immutable limits, guest image, mounts, actions,
  network, evidence, retention, and platform policy.
- **Execution plan:** Ordered, externally constructed actions; repository text
  cannot add or alter actions.
- **Guest image:** Pinned, verified, minimal operating-system image used as the
  disposable base.
- **Input snapshot:** Exact content-addressed repository material admitted by
  steps 1 and 2.
- **Behavior report:** Immutable observations and limitations from one run.
- **Escape indicator:** Evidence that the guest attempted or achieved access
  outside the permitted boundary; any credible indicator blocks reuse.

## Initial Execution Profiles

Build slowly through closed profiles:

| Profile | Intended action | Network | Credentials |
| --- | --- | --- | --- |
| `fixture_smoke_v1` | Project-owned synthetic executables only | Denied | None |
| `offline_command_v1` | One explicit executable/argv against an admitted snapshot | Denied | None |
| `offline_build_v1` | One approved build/test action with pre-provisioned guest tools | Denied | None |
| `mirror_build_v1` | Later package restore through an Impresari-controlled read-only mirror gateway | Exact mirror only | Short-lived mirror token held outside guest where feasible |
| `observed_egress_v1` | Later allowlisted external behavior observation | Explicit destinations only | None |

Only `fixture_smoke_v1` may be implemented before the complete isolation
conformance foundation passes. Later profiles require separate activation.

## Critical User Journeys

### Journey 1 — Admit a run

1. Context supplies the exact assessment and snapshot.
2. The deterministic admission service supplies a current
   `isolated_execution_eligible` decision naming one quarantine profile.
3. The user reviews the exact action plan, limits, network state, data retained,
   and limitations.
4. The Runner independently verifies all identities and policy intersections.
5. Any stale, missing, broader, or ambiguous input denies admission.

### Journey 2 — Execute offline

1. The Runner creates a new guest from a verified read-only base image.
2. It copies the exact snapshot into a guest-private writable workspace while
   retaining an immutable baseline for comparison.
3. It installs no host credentials and exposes no host filesystem.
4. It executes only the externally approved action and argv without a host
   shell.
5. It records bounded behavior and kills the guest on completion, timeout,
   cancellation, or violation.

### Journey 3 — Observe a network attempt

1. Guest code attempts DNS or a network connection in an offline profile.
2. The virtual network boundary denies the request.
3. The Runner records destination evidence in a privacy-minimized form allowed
   by the profile.
4. The attempt becomes a behavior finding; the Runner never disables the
   boundary to help the application continue.

### Journey 4 — Collect and destroy

1. The guest stops or is forcibly terminated.
2. The Runner seals process, filesystem-delta, network-attempt, resource,
   policy-violation, and completeness records.
3. Only explicitly permitted artifacts leave the guest after hostile-content
   scanning and size/type validation.
4. The guest disk, memory state, and job-private data are destroyed or placed in
   an explicitly authorized incident-preservation state.
5. Cleanup failure blocks worker reuse.

## Functional Requirements

### A. Admission and authorization

| ID | Requirement |
| --- | --- |
| DQR-FR-001 | Require exact assessment, decision, snapshot, profile, action-plan, guest-image, Runner, and policy identities |
| DQR-FR-002 | Independently verify the admission decision is fresh and permits only the selected profile |
| DQR-FR-003 | Intersect user, consumer, platform, image, and Runner policy; never union capabilities |
| DQR-FR-004 | Reject repository-supplied action, mount, network, credential, tool, image, or retention configuration |
| DQR-FR-005 | Require explicit human approval before the first real-untrusted run and every material profile expansion |

### B. VM lifecycle

| ID | Requirement |
| --- | --- |
| DQR-FR-006 | Use a fresh VM-backed guest for every job |
| DQR-FR-007 | Verify guest image digest, signature/provenance, version, patch state, architecture, and profile compatibility |
| DQR-FR-008 | Use an immutable base plus disposable job overlay or equivalent non-reused storage |
| DQR-FR-009 | Expose no host filesystem except an implementation-controlled transfer mechanism with no ambient host paths |
| DQR-FR-010 | Prevent guest reuse, snapshot rollback to contaminated state, and cross-job shared writable caches initially |
| DQR-FR-011 | Destroy or quarantine all job state after terminal collection and verify cleanup |

### C. Execution

| ID | Requirement |
| --- | --- |
| DQR-FR-012 | Execute only closed action classes and exact argv constructed outside repository content |
| DQR-FR-013 | Never invoke a host shell with repository-controlled strings |
| DQR-FR-014 | Run as a non-administrator guest identity without privilege elevation |
| DQR-FR-015 | Enforce wall time, CPU, memory, process, disk, file, output, and concurrency limits at host and guest layers |
| DQR-FR-016 | Kill the VM on cancellation, lease expiry, policy violation, supervisor loss, or resource breach |
| DQR-FR-017 | Treat package installation, build, test, server, and installer actions as distinct profiles and evidence classes |

### D. Secrets, mounts, and host protection

| ID | Requirement |
| --- | --- |
| DQR-FR-018 | Provide no inherited host environment, home, credentials, SSH agent, browser state, password manager, cloud metadata, or production data |
| DQR-FR-019 | Prohibit Docker/VM control sockets, host devices, host process namespaces, clipboard, shared folders, drag/drop, and arbitrary guest additions |
| DQR-FR-020 | Use synthetic canary secrets and resources only for containment testing |
| DQR-FR-021 | Keep evidence export one-way, bounded, typed, scanned, and explicitly authorized |

### E. Network

| ID | Requirement |
| --- | --- |
| DQR-FR-022 | Deny guest network by default at a host-controlled virtual boundary |
| DQR-FR-023 | Deny loopback-to-host, private/LAN, link-local, multicast, metadata, DNS, proxy, and IPv4/IPv6 bypass paths |
| DQR-FR-024 | Permit later egress only through a named profile, exact gateway/destination policy, and explicit approval |
| DQR-FR-025 | Prevent repository code from changing DNS, proxy, routing, firewall, or allowlist policy outside its disposable guest view |
| DQR-FR-026 | Record denied attempts without storing unnecessary payload content |

### F. Evidence and outcome

| ID | Requirement |
| --- | --- |
| DQR-FR-027 | Record process tree, executable hashes, exit state, timing, resource use, filesystem delta, network attempts, policy violations, and sensor completeness |
| DQR-FR-028 | Bind every report to the exact assessment, decision, snapshot, profile, action plan, guest image, Runner, and sensor versions |
| DQR-FR-029 | Separate observed behavior from inferred suspiciousness and unknown/unobserved behavior |
| DQR-FR-030 | Never return a safe/clean/malware-free verdict |
| DQR-FR-031 | Send behavior evidence back through a bounded untrusted-result boundary |
| DQR-FR-032 | Require a new assessment and human/consumer decision before any trusted-environment use |

## Evidence Model

The behavior report contains:

- admission and execution identities;
- exact start/end/termination state and clock provenance;
- action and executable identities without secret-bearing argv;
- process creation and ancestry;
- guest filesystem additions, modifications, deletions, permissions, and hashes;
- denied and permitted network attempts under the selected privacy policy;
- persistence and privilege observations available from admitted sensors;
- output/artifact inventory and extraction decisions;
- resource totals and limit events;
- sensor health, blind spots, truncations, and unknowns;
- cleanup or incident-preservation result.

Raw stdout/stderr and files are hostile local artifacts, not canonical audit.
They remain bounded, previewable, and separately retained.

## Success Metrics

| Metric | Initial target |
| --- | --- |
| Host credential/home/repository access from guest | Zero successful accesses |
| Network packets leaving an offline profile | Zero |
| Cross-job writable-state reuse | Zero |
| Unapproved action execution | Zero |
| Guest surviving cancellation/timeout | Zero in conformance suite |
| Input snapshot mismatch admitted | Zero |
| Behavior report identity/provenance completeness | 100% |
| Cleanup verified | 100% or job host quarantined |
| Safe/clean verdicts | Zero |
| Existing Context/Analyzer operation when Runner disabled | Unchanged |

## Nonfunctional Requirements

### Security

- VM isolation is the required initial boundary; containers alone do not satisfy
  the quarantine claim.
- The supervisor, image builder, guest agent/sensors, transfer channel, and
  hypervisor interface are part of the trusted computing base and require SBOM,
  provenance, patch, and vulnerability policy.
- No isolation claim exceeds native platform evidence.

### Reliability

- Host restart, Runner crash, guest hang, disk-full, clock change, and partial
  evidence produce a terminal unknown/incomplete state, never inferred success.
- Idempotent job identities prevent duplicate execution after retry.

### Performance

- Cold-boot and execution overhead are measured separately.
- Optimization cannot introduce shared writable guest state, host mounts, or
  weaker cleanup.

### Usability

- Before execution, show exact profile, action, network, credentials, limits,
  retained data, and unsupported behavior in plain language.
- After execution, lead with policy violations, coverage gaps, and next required
  decision rather than a single score.

## Implementation Plan And Gates

### DQR-0 — Provider contract and threat fixtures

- Freeze VM provider, image, action-plan, report, and cleanup contracts.
- Build synthetic escape, process, filesystem, network, resource, crash, and
  evidence fixtures only.

Gate: ADR-0075, threat model, host selection, and independent design review.

### DQR-1 — VM lifecycle skeleton

- Implement verified image boot, disposable overlay, synthetic guest agent,
  cancellation, hard teardown, and cleanup verification.
- No repository input or network.

Gate: repeated contamination, crash/restart, orphan, rollback, and cleanup tests.

### DQR-2 — Offline fixture execution

- Add `fixture_smoke_v1` with project-owned benign and hostile synthetic
  executables.
- Validate canaries, filesystem/process observation, resource enforcement, and
  zero egress.

Gate: macOS and Linux host isolation evidence plus independent security review.

### DQR-3 — Admitted snapshot transfer

- Add exact snapshot copy, guest-private workspace, immutable baseline, and
  output/delta collection.
- Still no real untrusted repository or package installation.

Gate: TOCTOU, transfer, polyglot, archive, output, and source-immutability tests.

### DQR-4 — Offline real-untrusted private pilot

- Enable one closed offline action profile for one owner-controlled repository
  at a time.
- No credentials, external network, production data, or output application.

Gate: explicit per-pilot founder approval, incident readiness, host backup,
kill switch, and post-run review.

### DQR-5 — Mediated dependency access

- Add a read-only package mirror gateway only if offline evidence demonstrates
  the need and the cache/provenance/poisoning design is approved.

Gate: separate egress, dependency, credential, mirror integrity, and privacy
review. This phase is not implied by Step 3 approval.

### DQR-6 — Future Windows dynamic provider

- Design a Windows guest image, provider, sensors, evidence mapping, licensing,
  servicing, and isolation suite.

Gate: new or revised ADR, Windows-specific threat model, native conformance,
independent security review, and explicit founder approval.

## Acceptance Criteria

| ID | Given | When | Then |
| --- | --- | --- | --- |
| DQR-AC-001 | A stale or broader admission decision | A run is requested | Admission is denied before VM creation |
| DQR-AC-002 | An offline profile | Guest code attempts network access | No packet exits and a bounded denial observation is recorded |
| DQR-AC-003 | Seeded host credential canaries | Hostile guest code searches for them | No canary is observable from the guest |
| DQR-AC-004 | Guest code forks, persists, or hangs | Deadline/cancel occurs | The entire VM terminates and cannot affect the next job |
| DQR-AC-005 | Guest modifies its repository copy | Collection runs | Delta is recorded while the original snapshot remains unchanged |
| DQR-AC-006 | Sensor or collection failure | Report is assembled | Completeness is partial/unknown and no permissive result is inferred |
| DQR-AC-007 | A Windows executable | Initial Linux guest profile is requested | Execution is unsupported/denied, not emulated or silently attempted |
| DQR-AC-008 | A completed run | User asks to copy all output to a trusted repository | The operation is unavailable without separate review and authorization |

## Risks And Controls

| Risk | Control |
| --- | --- |
| Hypervisor or guest escape | Patch/provenance policy, minimal devices, defense in depth, kill switch |
| Sandbox-aware or delayed malware | Honest bounded observation and no safe verdict |
| Dependency access reintroduces egress | Offline first; separate mirror phase and gateway |
| Guest output attacks host parser | Bounded transfer and isolated post-scan before preview |
| Shared state contaminates later job | Fresh image overlay and no shared writable caches |
| User mistakes eligibility for approval | Closed language and separate post-run decision |
| Platform differences weaken claims | Per-provider conformance and explicit support matrix |
| Windows behavior is missed | Initial static coverage plus explicit dynamic unsupported state |

## Rollback, Disablement, And Incident Response

- Global and per-provider kill switches prevent new runs.
- Image or Runner revocation blocks admission by exact identity.
- Disabling Quarantine Runner leaves Context and Analyzer Runner available.
- Cleanup failure quarantines the host/provider instance from new work.
- Incident preservation is local, explicit, encrypted where supported, bounded,
  access-controlled, and never automatically uploaded.
- Rollback cannot resume or reuse a previously contaminated guest.

## Approval Boundary

This PRD supplies a staged implementation plan but authorizes no hypervisor use,
guest download, code execution, network access, or real-repository pilot. Every
implementation and activation phase requires its stated evidence and separate
founder approval.
