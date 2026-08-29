# Disposable Quarantine Runner — Architecture Requirements Document

## Document Control

- Product: Impresari Quarantine Runner.
- ARD ID/version: IC-DQR-ARD-001 / 0.1.
- Status: Proposed; architecture and implementation planning only.
- Date: 2026-08-26.
- Sequence: Security expansion step 3 of 3.
- Related records:
  - [Disposable Quarantine Runner PRD](../product/disposable-quarantine-runner-prd.md)
  - [ADR-0075](../decisions/0075-disposable-vm-quarantine-execution.md)
  - [Hostile Repository Admission ARD](hostile-repository-admission-ard.md)
  - [Isolated Analyzer Runner ARD](isolated-analyzer-runner-ard.md)
  - [Security Threat Model](../security/threat-model.md)

## Architecture Objective

Create a provider-neutral VM-backed dynamic-analysis system that executes only
an exact admitted repository snapshot and approved action plan, exposes no
ambient host authority, denies network and credentials by default, collects
bounded behavior evidence, and destroys the execution environment after every
job.

## Governing Architecture Decisions

### AD-DQR-001 — VM-backed boundary

The initial quarantine claim requires a virtual machine boundary. Containers
may be used inside a guest for packaging but cannot replace the VM boundary or
receive host Docker/VM control sockets.

### AD-DQR-002 — Provider-neutral control plane

The supervisor targets a closed `QuarantineProvider` interface. macOS and Linux
implementations may use different native virtualization systems, but expose the
same image, device, lifecycle, evidence, and cleanup semantics.

### AD-DQR-003 — Fresh environment per job

Every job starts from a verified immutable image and fresh writable overlay.
There are no shared writable dependency caches, long-lived guest agents, or
reused workspaces in the initial design.

### AD-DQR-004 — External action plan

Repository content may be executed as data/code inside the guest, but cannot
define the control-plane action, host executable, VM configuration, mount,
network, credential, retention, or evidence policy. The action plan is typed,
versioned, and approved before boot.

### AD-DQR-005 — Offline first

The initial profile presents no guest network device where feasible or routes
it only to a host-controlled deny/observation boundary. Dependency and observed
egress are later profiles with separate authorization.

### AD-DQR-006 — Observation is not assurance

A run produces bounded behavior evidence and unknowns. Lack of detection or
visible malicious behavior cannot become a safe/clean verdict or ordinary-host
execution approval.

### AD-DQR-007 — Windows dynamic execution is a future provider

Initial macOS/Linux hosts run supported Linux guests. Windows repository static
analysis is initial scope elsewhere; Windows dynamic behavior requires a future
Windows guest/provider admission and cannot be inferred from Linux execution.

## System Context

```text
Context assessment + Analyzer coverage
                 |
                 v
Deterministic admission service
  isolated_execution_eligible(profile)
                 |
                 v
Quarantine Runner control plane
  admission / image / action / lifecycle / evidence / cleanup
       |                 |                    |
       v                 v                    v
Image store       QuarantineProvider      Evidence spool
verified bases     macOS | Linux           hostile/local
                         |
                         v
                  Disposable VM
                  guest-private source copy
                  no host secrets
                  network denied
                         |
                         v
                  bounded behavior report
```

## Deployment Units

| Unit | Responsibility | Authority |
| --- | --- | --- |
| Quarantine supervisor | Admission, profile, provider, lifecycle, cancellation, evidence sealing | Exact approved job only |
| Image builder/importer | Produce or admit minimal guest images | Update sources and image store, never active job |
| Image store | Immutable images, manifests, provenance, revocation | Read-only to job supervisor |
| Platform provider | Create/configure/start/stop/delete VMs | Native hypervisor API only |
| Guest bootstrap agent | Verify job, stage workspace, launch closed action, collect local observations | Guest-only |
| Host network boundary | Deny or mediate profile egress | Virtual interface/gateway only |
| Evidence collector | Receive bounded authenticated guest events and artifacts | Job spool only |
| Post-run result normalizer | Validate report and forward derived evidence | No execution authority |

## Trust Zones

### DQR-Z1 — Admission inputs

Assessment, decision, policy, profile, and action plan are trusted only after
schema, identity, signature/digest, freshness, and audience verification.

### DQR-Z2 — Quarantine control plane

Trusted for one bounded job. It must not parse raw repository content beyond
the approved transfer contract or inherit unrelated credentials.

### DQR-Z3 — Hypervisor/platform provider

Part of the trusted computing base. It may be vulnerable or misconfigured and
requires exact supported host/version evidence.

### DQR-Z4 — Guest image and bootstrap

Trusted only as a pinned, verified, patched image. The running guest becomes
hostile once repository code starts.

### DQR-Z5 — Repository workspace in guest

Fully hostile and writable inside the guest. It cannot map to the original host
workspace.

### DQR-Z6 — Virtual network

Denied by default and enforced outside guest control. Later gateways remain
untrusted external interfaces with explicit destination and data policy.

### DQR-Z7 — Evidence spool and preview

Sensitive and hostile. Guest output, filenames, logs, screenshots, and files
may attack viewers or parsers and require bounded isolated processing.

## Quarantine Provider Interface

A provider implementation must expose closed operations:

```text
capabilities()
validate_host(profile)
verify_image(image_manifest)
create_job_vm(job_manifest)
attach_input(opaque_transfer)
start(job_epoch)
observe()
cancel(job_epoch)
force_stop(job_epoch)
collect_bounded_evidence()
destroy(job_epoch)
verify_destroyed(job_epoch)
```

It cannot accept arbitrary hypervisor flags, host paths, device names, network
configuration, shell strings, or repository-derived settings. Unsupported
profile fields fail before VM creation.

## Canonical Job Manifest

The signed/digest-bound job manifest includes:

- job, workspace, snapshot, assessment, admission decision, and policy IDs;
- exact Runner and provider versions;
- quarantine profile ID/version/digest;
- guest image ID/digest/provenance/patch epoch;
- architecture and supported guest platform;
- ordered closed action plan and exact tool identities;
- input content manifest and transfer digest;
- virtual CPU, memory, disk, process, file, output, time, and concurrency limits;
- network mode and exact gateway/destination policy identity;
- credential mode, which is `none` initially;
- evidence sensor set, fields, redaction, bounds, and retention;
- lease, cancellation, incident-preservation, and cleanup policy;
- user/consumer approval reference and expiry.

Repository bytes cannot modify the manifest after admission. Any mismatch
creates a new job and approval requirement.

## Image Architecture

### Base image

Each image is minimal, immutable, architecture-specific, and identified by a
cryptographic digest plus provenance. It includes only:

- guest OS and required security updates;
- the fixed bootstrap/observation agent;
- exact pre-approved runtime/tool versions for named profiles;
- time, filesystem, process, and network observation support;
- no user credentials, package-manager tokens, SSH keys, cloud agents, shared
  folders, GUI integration, clipboard, host mounts, or auto-update.

### Build and update

Images are built or imported outside active jobs. Admission requires source,
license, SBOM, vulnerability state, signature/provenance, configuration, tool
inventory, patch epoch, supported host/provider matrix, and reproducible or
attestable build evidence.

Activation is atomic and immutable. Revoked images cannot start new jobs.
Running jobs never update in place.

### Writable state

Each job uses a fresh overlay or equivalent disposable disk. Guest memory,
overlay, swap, logs, and transient keys are unique to the job. Initial profiles
use no cross-job writable cache.

## Input Transfer Architecture

1. Context exports an exact no-overwrite snapshot bundle into a dedicated
   transfer area, not the source repository.
2. The supervisor validates manifest, file count, sizes, types, and complete
   content hash set.
3. The provider transfers the bundle through a narrow mechanism that cannot
   expose arbitrary host paths.
4. The guest verifies the complete bundle before materialization.
5. The guest creates a private writable working copy and records its baseline.
6. Host and guest identities are compared before action start.

Transfer errors destroy the guest. The original source is never mounted,
including read-only, in the initial design; a copy avoids live TOCTOU and mount
confusion at the cost of additional I/O.

## Action And Process Architecture

An action class defines:

- stable ID/version and purpose;
- exact guest executable identity and argv schema;
- allowed working directory under the guest workspace;
- input/output file classes;
- environment allowlist with non-secret fixed values;
- network and credential mode;
- resource ceilings and expected descendants;
- observation requirements;
- success/failure semantics;
- unsupported behavior and cleanup.

The guest agent starts the executable without a shell, under a non-admin
identity and process group. If the intended repository action inherently starts
shells or interpreters, that behavior is contained and observed inside the
guest; it still cannot alter the host-side action plan.

## Resource And Lifecycle State Machine

Canonical states:

```text
requested -> admitted -> image_verified -> vm_created -> input_verified
-> running -> collecting -> sealed -> destroying -> destroyed

Any active state -> cancelling -> force_stopping -> collecting_partial
-> destroying -> destroyed

Any cleanup failure -> provider_quarantined
```

Transitions are durable and idempotent. A retry with the same job/epoch cannot
start a second VM after execution began. Host restart recovery stops or destroys
unknown VMs before accepting new work.

Host and guest enforcement cover:

- wall deadline and bounded lease;
- virtual CPU and memory;
- guest process count and ancestry;
- overlay and evidence disk use;
- file/output counts and sizes;
- network packets/flows;
- evidence event rate;
- concurrent jobs.

## Network Architecture

### Offline profile

Preferred configuration exposes no guest network interface. If observation
requires a virtual interface, all traffic terminates at a host-controlled deny
boundary with no forwarding. Tests cover raw IP, DNS, IPv4/IPv6, ICMP, UDP/TCP,
QUIC, multicast, link-local, private ranges, host gateway, metadata, proxy,
tunneling, and virtual-device bypasses relevant to the provider.

### Future mirror profile

A package mirror gateway may expose only pinned package ecosystems and exact
repository endpoints. It separates download from the guest where feasible,
verifies integrity/provenance, blocks arbitrary URLs and scripts-as-packages,
records every object, and provides no reusable credential to the guest.

### Future observed-egress profile

Requires a new policy decision naming exact destinations, protocols, DNS,
redirect, payload logging/redaction, retention, and stop conditions. It cannot
permit LAN, host, metadata, or arbitrary Internet access.

## Secret And Identity Architecture

Initial jobs receive no real secret. The supervisor clears its environment and
does not inherit provider/cloud/Git/browser/password-manager credentials. Guest
image and bootstrap contain no shared credentials.

Per-job local authentication between supervisor and guest uses an ephemeral
channel identity established before hostile execution and scoped to evidence
submission only. It cannot request new capabilities, change policy, access host
files, or survive job destruction.

Synthetic canaries may test isolation. A canary observation is a critical
security failure, not ordinary evidence.

## Behavior Observation Architecture

Sensors should capture, within platform capability and explicit limits:

- process start/exit, parentage, executable/script identity, and privilege;
- filesystem create/write/delete/rename/permission activity inside the guest;
- service, task, cron, startup, and other persistence attempts visible in the
  guest;
- network attempts and policy verdicts;
- resource use and limit violations;
- kernel/security events available without adding excessive guest attack
  surface;
- guest agent/sensor health and gaps.

The system reports what was observed, not all possible behavior. Repository
stdout, logs, and created files remain hostile artifacts. Natural-language
summaries cannot replace structured events.

## Evidence Collection And Egress

The guest seals a canonical behavior record through the ephemeral channel.
Host validation checks schema, job/epoch, event sequence, timestamps, bounds,
sensor identities, and completeness.

Optional artifact extraction requires an exact allowlist by media/type/count/
size and a second hostile-artifact scan. No executable is copied to a trusted
workspace automatically. Preview renderers must themselves use isolated,
bounded paths appropriate to the artifact type.

Evidence stores are job-scoped with separate retention for:

- canonical metadata/report;
- bounded stdout/stderr;
- filesystem delta manifest;
- selected hostile artifacts;
- incident-preserved VM state, normally absent.

## Cleanup And Destruction

Normal terminal cleanup:

1. Stop the action and descendants.
2. Stop the guest and virtual devices.
3. Seal permitted evidence.
4. Detach and destroy overlay, memory/swap, ephemeral identity, and transfer
   copies.
5. Verify no VM/process/device/lock remains.
6. Record cleanup result.

Secure deletion cannot be guaranteed on snapshots, SSDs, backups, or host
forensics. Encryption with per-job ephemeral keys may reduce remanence where
supported. Cleanup failure quarantines the provider instance from new jobs.

## Host Provider Requirements

### macOS

- Initial target: vendor-supported Apple-silicon macOS.
- Candidate implementation may use Apple's Virtualization framework, but exact
  selection remains an implementation ADR/detail.
- Validate entitlements, architecture-matched Linux images, device minimization,
  network configuration, lifecycle, termination, and host filesystem behavior.

### Linux

- Initial target: x86-64 Linux with hardware virtualization available.
- Candidate implementation may use KVM through an approved supervisor, but no
  arbitrary QEMU/libvirt flag passthrough is allowed.
- Validate device permissions, cgroups, namespaces around the supervisor,
  virtual networking, image paths, and process cleanup.

### Future Windows

- Preserve the provider interface for Hyper-V/Host Compute or another admitted
  native boundary.
- Require a Windows guest to claim Windows dynamic behavior.
- Add Windows image licensing/servicing, NTFS/reparse, registry/service/task/WMI,
  DPAPI, Defender/AMSI interaction, network, event logging, and evidence mapping.
- Do not emulate or silently execute Windows binaries under a Linux profile.

## Failure Semantics

| Failure | Required behavior |
| --- | --- |
| Admission/policy mismatch | Deny before image/VM creation |
| Image verification failure | Revoke/quarantine image; no boot |
| Input mismatch | Destroy guest; no execution |
| Guest agent handshake failure | Destroy; report unavailable |
| Sensor failure | Stop if mandatory; otherwise partial per exact profile |
| Network policy violation | Deny, record, optionally terminate per profile |
| Resource/lease/cancel event | Force-stop complete VM and collect partial evidence |
| Supervisor/host restart | Reconcile and destroy unknown active state before new jobs |
| Evidence validation failure | Quarantine metadata/raw spool; no canonical promotion |
| Cleanup failure | Quarantine provider instance |

## Threat Register

| Threat | Mandatory control |
| --- | --- |
| VM/hypervisor escape | Minimal devices, patched pinned TCB, native hardening, kill switch |
| Host mount/socket exposure | Copy-based input; no shared folders/control sockets/devices |
| Credential theft | No credentials; cleared supervisor; synthetic canaries |
| Network exfiltration | No interface or host deny boundary; bypass suite |
| Sandbox evasion/delay | Bounded observation, multiple profiles later, no safe verdict |
| Guest-to-host parser attack | Bounded authenticated evidence and isolated artifact processing |
| Image supply-chain compromise | Attested builds, SBOM, signing/digests, revocation |
| Cross-job contamination | Fresh overlay/memory/identity; cleanup verification |
| Duplicate/replayed job | Durable job epoch and idempotent state machine |
| Repository action injection | External closed action plan; no host shell strings |
| Evidence spoofing | Ephemeral authenticated channel and complete host validation |
| Privilege escalation in guest | Non-admin start, minimal image; still contained as hostile guest behavior |

## Observability And Audit

Source-free control audit includes admission, approvals, profile/image/provider
identities, state transitions, resource totals, termination reason, sensor
completeness, evidence IDs, cleanup, revocation, and incident preservation.

It excludes raw source, secret values, unrestricted command output, unrelated
host inventory, and raw packet payloads by default. Missing audit can only stop
or restrict work; it cannot widen permission.

## Conformance And Evaluation

### Provider lifecycle

- Image substitution/revocation, boot failure, repeated start, crash/restart,
  cancel, force-stop, orphan VM/device/process, overlay reuse, and cleanup.

### Isolation

- Host paths, home, environment, credentials, agents, sockets, clipboard,
  devices, LAN/metadata, process visibility, hypervisor controls, and seeded
  canaries.

### Network

- No-interface proof or packet capture; DNS/IP/protocol/redirect/proxy/private
  address bypass corpus; future gateway allow/deny exactness.

### Resource and adversarial behavior

- Fork/process, CPU, memory, disk, file, output, event, decompression, sleep,
  daemonization, shutdown, reboot, and sensor-flood cases.

### Evidence

- Sequence loss/reorder/duplicate, time drift, spoofing, malformed output,
  hostile filenames/content, collection truncation, and exact snapshot/profile
  binding.

### Cross-platform

- Same semantic job fixtures on macOS and Linux providers with differences
  explicitly recorded. Windows remains unsupported until its own complete
  matrix passes.

## Implementation Sequence And Security Gates

Follow DQR-0 through DQR-6 in the PRD. No real untrusted source enters the
Runner until synthetic isolation, lifecycle, evidence, cleanup, incident, and
independent review gates pass. Network and credentials remain absent from the
initial private pilot.

Before implementation:

1. Steps 1 and 2 must be implemented, evaluated, and accepted.
2. ADR-0075 and the dynamic-execution threat model must be accepted.
3. Exact macOS and Linux provider choices must have implementation records.
4. Image source/licensing/update/provenance policy must be approved.
5. Incident and host recovery plans must exist.
6. Founder authorization must name the exact phase and permitted fixtures.

## Rollback And Failure-Domain Preservation

- Disable Quarantine Runner globally or by provider/profile/image/action class.
- Revocation stops new jobs and active jobs at the next mandatory control point.
- Removal leaves Context and static Analyzer Runner usable.
- No rollback reuses a contaminated overlay or resumes an unknown job.
- Provider compromise does not change Context assessment or Analyzer authority.
- A future Windows provider can be omitted or disabled independently.

## External Reference Baseline

These are candidate platform interfaces, not implementation selections. Exact
host versions, APIs, licensing, image sources, and security behavior must be
reverified before a provider decision:

- [Apple Virtualization framework Linux VM guidance](https://developer.apple.com/documentation/virtualization/creating-and-running-a-linux-virtual-machine)
- [Linux KVM API documentation](https://docs.kernel.org/virt/kvm/api.html)
- [Microsoft Hyper-V APIs](https://learn.microsoft.com/en-us/virtualization/api/)
- [Microsoft Hyper-V specification](https://learn.microsoft.com/en-us/virtualization/hyper-v-on-windows/)

## Architecture Exit Criteria

Step 3 is ready for an owner-controlled real-untrusted offline pilot only after
macOS and Linux provider conformance, image provenance, zero-egress proof,
credential canaries, resource/process containment, evidence integrity, cleanup,
kill switch, incident response, independent security review, and explicit
founder activation are recorded. This ARD authorizes none of those operations.
