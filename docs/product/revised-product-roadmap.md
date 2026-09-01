# Impresari Context — Revised Product Roadmap

- Status: Approved
- Date: 2026-08-23
- Owner: Aaron Boldt
- Authority: Founder-approved product roadmap
- Client depth: [Client Integration Depth Roadmap](client-integration-roadmap.md)

This roadmap is the phase-sequencing source of truth. It separates completed
delivery slices from the phase that owns their product outcome. Individual
language and client additions require their own admission evidence and must not
silently broaden the authority boundary.

Release baseline: `v0.1.0` was published on 2026-08-23 UTC from commit
`c77e95ce95b2fde99da2582707d4e4d58a512122`. This roadmap also records later
default-branch work; a completed roadmap item is not, by itself, a claim that
the capability exists in the `v0.1.0` binaries.

| Phase | Outcome | Current status |
| --- | --- | --- |
| 0 | Correct public language/client contract and read-only doctor | Complete |
| 1 | Python and configuration evidence; first-class Codex, Claude Code, and Cursor kits | Complete for recorded client/version/OS scopes: Python, narrow strict-JSON, bounded JSONC, bounded TOML, deliberately bounded YAML, and the three named integrations. Codex is first-class for Codex CLI `0.149.0-alpha.4.1` on macOS aarch64 after isolated user-home installation, malformed-configuration rejection, deterministic App Server lifecycle/packet equivalence, and exact removal evidence. Claude Code is first-class for CLI `2.1.241` on macOS aarch64 after malformed strict-configuration rejection, native isolated local-scope add/get/removal, and bounded temporary-config packet-equivalence evidence. Cursor is first-class for Agent CLI `3.17.8` (`2026.08.11-e8db854`) on macOS aarch64 after isolated project enable/list-tools/disable/removal and guarded Agent-mode packet-equivalence evidence. |
| 2 | Rust and Go structural evidence; broader agent access | Complete for recorded scopes: Rust and Go are complete. GitHub Copilot CLI is first-class L1 with recorded-scope L2 guidance and L4 health for CLI `1.0.80` on macOS aarch64. VS Code Copilot extension host is independently first-class L1 with recorded-scope L2 guidance for VS Code `1.134.0` on macOS arm64. Gemini remains generic because normal-client testing is blocked by its current free-tier service. |
| 3 | Deterministic context planner | Complete (approved initial scope): profile-bound deterministic plans, coverage/omission reporting, exact plan and packet identities, CLI, and MCP support are implemented. Standalone profiles retain explicit structural, change-set, associated-test, and configuration-to-code omissions rather than inferring evidence. |
| 4 | Java, Kotlin, C#, impact evidence, and incremental updates | Complete: bounded Java, Kotlin, C#, structural-impact, declared change-set, caller-declared associated-test, repository orientation, explicit incremental-update, and convention/exemplar evidence are accepted after full hosted CI. |
| 5 | Demand-led language expansion | Complete for the accepted scope: Scala, Elixir, Clojure, Haskell, C, C++, Ruby, PHP, and Swift are admitted with bounded structural evidence after independent hosted acceptance. Additional languages remain demand-gated under ADR-0040. |

## Adoption experience track

The short-install and first-run increment provides a pinned, checksum-verified
macOS/Linux installer and one preview-by-default `quickstart` command for the
recorded managed clients. ADR-0076 now accepts staged macOS work toward one
signed/notarized [CLI-compatible cask](macos-hybrid-xpc-distribution-prd.md),
but publication remains gated on IAR-1B, signing/notarization, clean-machine,
migration, upgrade, rollback, and uninstall evidence. The earlier formula
proposal remains Linux-only and separately gated. Automatic update installation remains a later, separate
future increment with a reviewable
[proposed PRD](automatic-update-installation-prd.md), architecture, and ADR.
It remains unapproved and separately gated on signing-root custody, scheduled
background execution, and live-rehearsal authority so neither adoption feature
can silently expand installer or background authority.

## Observability and budget-control track

A local real-time dashboard and narrowing-only budget-control layer are
accepted under [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md).
The DBC-1 foundation freezes closed policy/decision/snapshot schemas, a pure
field-wise-minimum evaluator, metadata-only audit projection, bounded
aggregates, and a concurrent read-only audit view. DBC-2 adds preview-first
exact-owned policy apply/remove/rollback, optimistic concurrency, one atomic
current/previous state, admission-time reload, actual operation narrowing, and
limited/denied audit outcomes. DBC-3 adds the source-free `dashboard serve`
command, an isolated std-only verified-loopback listener, one-use fragment
bootstrap, a separate memory-only 256-bit API-route capability, bundled
digest-addressed assets, exact Host/Origin/CSRF checks,
preview-receipt-bound policy writes, bounded SSE recovery, and exact
foreground shutdown. DBC-4 completes synthetic native-browser admission with
adversarial source-canary, hostile-string, local-only asset, exact policy
lifecycle, shutdown, and disposable-cleanup evidence. Remote, hosted,
organization, billing, telemetry, and
source-viewing surfaces remain outside the roadmap without a separate founder
decision and external-data boundary.

## Hostile-repository security expansion track

The earlier `New malware feature chat` design has been recovered into three
reviewable increments: an
[evidence-only hostile-repository admission foundation](hostile-repository-admission-prd.md),
a separate [isolated analyzer runner](isolated-analyzer-runner-prd.md), and a
[disposable quarantine runner](disposable-quarantine-runner-prd.md). The first
increment is static-first. ClamAV and YARA are the initial local scanner
direction behind the isolated runner, and any optional online reputation check
is hash-only and explicit. ADR-0073 HRA-0 contract freezing and HRA-1 bounded,
read-only artifact inventory are implemented with closed schemas, a fixed
authority-denying resource profile, explicit exclusions, and original-synthetic
fixture provenance. HRA-2 is complete with closed npm lifecycle and canonical
Compose privileged-service corpora plus exact key-token evidence. HRA-3 is
complete with deterministic unavailable-by-default analyzer coverage planning,
closed synthetic result intake through ADR-0013 normalization, and immutable
assessment assembly. HRA-4 is complete as a separate pure deterministic
evaluator with monotonic restriction, explicit incomplete-analysis handling,
and no exception or ordinary-host authorization input. HRA-5 is complete with
an exact-commit, three-platform candidate build and clean-install rehearsal;
ADR-0073 Step 1 is complete. ADR-0074 IAR-0 and the IAR-1A
application-enforced baseline are complete with closed runner and supervisor
contracts, fixed profiles, reviewed fixture provenance, exact identities,
private synthetic staging, and short-lived worker supervision. The remaining
IAR-1B OS-confinement checkpoint is not complete. None of
these components authorizes scanner execution, repository execution, artifact upload,
threat-intelligence access, or VM/cloud provisioning. Each increment remains
bound to its accepted threat-model, platform, supply-chain, privacy, and
release-evidence scope. OS-specific confinement is next; real analyzers and
Step 3 quarantine remain later, separately evidenced increments.
The initial macOS feasibility inventory also records that the available
`sandbox-exec` command is deprecated and cannot serve as the durable production
boundary merely because it is already used by a network-denied test harness.
A synthetic App Sandbox/private-XPC prototype now records a partial macOS
result: native sandbox identity, bounded IPC, and selected filesystem,
credential, synthetic-device, process, and network denials pass. Hard
resource/process-tree limits, exact-target fault timeout, and bounded
source-byte cleanup also pass, while complete OS-managed container cleanup,
production signing/notarization, packaging, and multi-host evidence remain
open. It does not admit macOS or open IAR-2.
The hybrid resource/lifecycle checkpoint subsequently passed native synthetic
CPU termination, bounded address-space growth, `fork`/`posix_spawn` denial,
exact-target timeout termination, crash/relaunch, and source-byte cleanup. The
candidate combines App Sandbox/private XPC with the Rust supervisor and public
resource limits, and ADR-0076 selects one CLI-compatible Homebrew cask as its
intended release topology. The frozen `iar-macos-xpc-hybrid-v1` profile and
closed source-free Rust-to-host preparation handshake now pass schema, Rust,
and native effective-limit checks. macOS remains at IAR-1A until Developer ID
signing/notarization, cask lifecycle, clean-machine Gatekeeper, the full Tier A
corpus, and every claimed macOS host pass. No privileged daemon, private API,
persistent service, or VM fallback is added.

The first decisive Tier A probes then found material gaps in that exact
topology: multiple individually legal files exceeded any aggregate job-disk
ceiling, and a fresh XPC service process read a synthetic marker retained by a
preceding job. The candidate remains useful defense-in-depth, and ADR-0076
Option C remains the packaging choice, but macOS XPC is not advanced to
Developer ID rehearsal as an IAR-1B backend. The next OS-confinement feasibility
increment moves to Linux while macOS remains IAR-1A.

The Linux increment freezes `iar-linux-synthetic-v1` with read-only staged
input, zero writable path-backed filesystem, bounded output pipes,
`no_new_privs`, version-negotiated Landlock, architecture-pinned default-deny
seccomp, descriptor closure, and a required delegated cgroup v2 leaf. The first
native checkpoint is synthetic-only. On GitHub-hosted Ubuntu 24.04 kernel
`6.17.0-1022-azure`, the primitive suite passed with Landlock ABI 7, but the
job cgroup was not delegated, so the closed receipt returned `unsupported` and
kept `os_confined=false`. IAR-1B remains open until the complete delegated-
cgroup resource and lifecycle corpus passes. A green measurement job is not
itself an admission claim.

The next checkpoint freezes a separate `iar-linux-cgroup-synthetic-v1`
component profile and runs it only inside one CI-created, transient systemd
service with `Delegate=yes`. The unprivileged probe owns only that service's
cgroup subtree, places each synthetic worker atomically with
`CLONE_INTO_CGROUP`, and measures CPU, memory, process count, exact kill and
empty state, bounded output, timeout, crash/relaunch, cleanup, and cross-job
isolation. The receipt keeps overall `os_confined=false` even when this
component passes; admission requires a later source-free composition with the
primitive suite and additional host evidence.

PR 130 job `99194709845` passed that component checkpoint on hosted Ubuntu
24.04, kernel `6.17.0-1022-azure`, x86_64. The next increment is therefore the
source-free composite: one atomically placed worker must reproduce the primitive
and resource/lifecycle boundaries together. A component pass alone does not
advance Linux to IAR-1B.

That composite is now implemented as a replacement for the standalone
delegated-cgroup CI step, preserving the authorization ceiling of one ephemeral
`Delegate=yes` service. It applies the frozen `iar-linux-synthetic-v1` limits
before atomic worker placement, runs the primitive boundary inside that exact
worker, and repeats the source-free resource/lifecycle corpus below the same
delegation. Its exact-host status is determined only by new hosted composite
evidence; the earlier component receipts cannot be combined to produce it.

PR 131 job `99197119262` then passed that single-service composite on hosted
Ubuntu 24.04, kernel `6.17.0-1022-azure`, x86_64, with Landlock ABI 7. The closed
receipt may therefore set `os_confined=true` for that exact observed candidate.
It does not admit Linux broadly, enable a real analyzer, or open IAR-2; the next
roadmap checkpoint is independent kernel and architecture coverage without
weakening any Tier A gate.

The first independent architecture increment targets GitHub's standard
ephemeral `ubuntu-24.04-arm` runner. Its AArch64 seccomp audit identity and
syscall filter are separately pinned, and a dedicated synthetic-only job must
reproduce both the ordinary primitive receipt and the one-service composite.
It cannot inherit the x86_64 result and is admitted only by its own hosted
receipt.

PR 132 job `99198568879` passed that native arm64 checkpoint on kernel
`6.17.0-1022-azure` with Landlock ABI 7. Both x86_64 and arm64 therefore have
independent exact-host composite candidates on this one Azure kernel line.
Broader Linux and IAR-2 remain closed until a materially independent kernel
target passes and the production support scope is frozen.

PR 133 jobs `99200027090` and `99200027056` passed the held-out native
GitHub-hosted Ubuntu 22.04 and 26.04 checkpoints on materially different
`6.8.0-1064-azure`/Landlock ABI 4 and
`7.0.0-1012-azure`/Landlock ABI 8 kernels. Each invocation retained the
one-transient-service ceiling and produced its own source-free composite
receipt. Kernel and architecture diversity are now demonstrated for the
candidate; production admission remains closed until an exact support,
freshness, withdrawal, and release-maintenance contract is frozen.

ADR-0077 now freezes that candidate maintenance contract. The released manifest
admits only the exact Ubuntu 24.04 x86_64 and arm64 evidence as candidate scope;
the Ubuntu 22.04 and 26.04 receipts remain kernel-diversity-only. A source-free
foreground evaluator returns deterministic compatible, stale, changed, missing,
unsupported, and unavailable states and withdraws the candidate claim on every
non-compatible state. It performs no host discovery or execution and always
keeps production and real-analyzer admission false. The next checkpoint is an
explicit production-support decision; IAR-2 remains closed.

ADR-0078 now frames that decision. The recommended first production-feasibility
topology uses only an existing systemd user-manager delegation, with a separate
explicit profile for administrator/orchestrator-provided delegated subtrees.
Automatic sudo/pkexec fallback, a privileged daemon, and an administrator-
installed unit are excluded from the first slice. The founder accepted this
rootless-plus-externally-managed direction on 2026-08-30. The administrator-
provisioned profile remains deferred unless measured unsupported attempts exceed
10% and that profile would recover at least half. The closed topology policy and
source-free evaluator now exercise both selected profiles and deterministically
reject unavailable, unsupported, insufficient, invalid, and privileged-service
paths without host discovery or authority. The next checkpoint is bounded
source-free host preflight on independently pinned targets followed by the full
synthetic confinement corpus; production and IAR-2 remain closed.

The first rootless host slice now implements that bounded preflight. It reads
only fixed Linux kernel, cgroup v2, and existing per-user systemd-manager
metadata; records no raw cgroup path or user identity; and performs no process
launch, D-Bus call, service/cgroup mutation, network access, privilege request,
or repair. Its deterministic receipt distinguishes ready-for-rehearsal,
unavailable, unsupported, insufficient-delegation, and invalid-host states while
keeping `os_confined=false`, production false, and real analyzers closed. The
next rootless checkpoint is the no-sudo foreground transient-user-unit synthetic
corpus on preflight-ready targets. The externally managed profile remains an
independent later checkpoint.

PR 137 run `33293552482` then recorded the first live rootless matrix. Hosted
Ubuntu 24.04 x86_64 and arm64 plus Ubuntu 26.04 x86_64 were ready for rehearsal;
Ubuntu 22.04 x86_64 failed closed because its user manager exposed memory and
pids but not CPU. The no-sudo rehearsal is now implemented for ready targets:
one foreground transient user service receives only CPU/memory/pids delegation,
runs the frozen original-synthetic composite, and must be collected afterward.
The Ubuntu 22.04 path skips without a system unit or privileged fallback. Hosted
PR 138 run `33294099301` then passed the complete rootless synthetic rehearsal
on Ubuntu 24.04 x86_64 and arm64 plus Ubuntu 26.04 x86_64. Each ready target
created and collected one transient user service without sudo, privilege, or
persistence. Ubuntu 22.04 again skipped before launch because CPU delegation
was unavailable. These are exact-host synthetic candidates only; production,
the external profile, and IAR-2 stay closed. The inherited-capability contract
for the selected external profile is now frozen and source-free tested: fixed
descriptor slot 3, directory verification, immediate close-on-exec, no raw
path, and no production authority. The next external checkpoint is bounded live
cgroup revalidation plus the complete original-synthetic corpus in an
operator-provided ephemeral environment. That live checkpoint is now
implemented as a dedicated hosted job with one collected operator service,
fixed descriptor slot 3, bounded revalidation, the complete synthetic corpus,
and explicit descendant cleanup. PR 140 run `33295514984` passed that gate on
Ubuntu 24.04 x86_64, kernel `6.17.0-1022-azure`; the complete composite identity
was `d9bbcbc55831385b3f56962170622cb2f79dbf8a7237573a2c2d8f712d100c2c`.
Both selected profiles now have independent exact-host synthetic candidates.
The next checkpoint freezes the shared Linux install, upgrade, rollback,
login-session, health-withdrawal, and uninstall lifecycle matrix before any
production or IAR-2 claim. ADR-0079 now freezes that source-free contract. The
rootless profile uses logout/login reentry; the external profile uses operator
relaunch; both share exact install, upgrade, rollback, cancellation, crash,
withdrawal, uninstall, and clean-state semantics. The deterministic evaluator
admits only a contract-level lifecycle candidate and fixes production,
packaging, privilege, persistent services, and analyzers false. ADR-0080
implements the independently hosted package-only rehearsal against the
published v0.1.0 Linux baseline and an exact-source release candidate for both
selected profiles. It proves install, replacement, rollback, and removal for A
and C, plus operator relaunch for C. A remains explicitly partial until a real
logout/login boundary is observed. Release-candidate run `33297882070`, Linux
job `99220559617`, passed the exact merged-source rehearsal. C returned
`package_lifecycle_candidate`; A returned `package_lifecycle_partial` with its
logout/login phase still unobserved. The following checkpoint composes the exact
C package receipt with fresh topology, cancellation, crash, and withdrawal
evidence without weakening the accepted A+C authority boundary. A continues on
its separate genuine-login-session evidence path. ADR-0081 now implements that
C composition as one exact-source release-candidate sequence: one accepted
temporary operator service, fresh topology and original-synthetic interruption
and crash evidence, explicit post-collection missing-capability withdrawal, and
closed SHA-linked composition. PRs 145 and 146 merged the composition and its descriptor-
portability correction. Release-candidate run `33300661271`, Linux job
`99228064803`, then passed the complete exact-source C lifecycle sequence from
commit `8f8f9adb5d99f373fbd6456564dfa6233c37bc34`; its final composition identity
is `0481667521371f3c7db33abfc4b99165fa9b71bd7bc8ed504173f7a89d4ea80b`.
C is therefore an exact-host synthetic lifecycle candidate. A remains partial
pending genuine login-session reentry. The next checkpoint is an expiring,
deterministically withdrawn production-support admission for C; production,
real analyzers, privilege, persistence, automatic repair, and IAR-2 remain
closed until that separate gate passes.

ADR-0082 now freezes that C admission gate. It pins the exact hosted target,
fresh lifecycle evidence, source commit, candidate archive, and tracked manifest
identity, but returns `release_pending` because the candidate is newer than the
published v0.1.0 artifact and still reports the same project version. Production
support cannot activate until a new immutable version/tag/archive is published
and bound by a reviewed update; v0.1.0 cannot be reused. Stale, changed, missing,
unsupported, and unavailable states deterministically withdraw the claim. A
remains partial, broad Linux and administrator-installed services remain out of
scope, and IAR-2 stays closed. The next release checkpoint is version selection,
publication of the exact or freshly revalidated source, and post-publication
admission evidence.

ADR-0083 now applies ADR-0017's earlier-review trigger to the proposed v0.2.0
feature release. The exact prepared product baseline, eight required security-
review areas, reviewer independence requirements, finding policy, reviewer
brief, and source-free readiness receipt are frozen. ADR-0084 retains that
release gate while backlogging reviewer engagement until a candidate is
frozen. ADR-0085 records that the first frozen candidate is now immutable
historical evidence rather than the final release candidate because accepted
analyzer-confinement roadmap work continues and no reviewer is currently
available. Roadmap development therefore continues, but
automated or AI-assisted checks cannot satisfy the review and no v0.2.0 tag,
publication, Linux production-support admission, or real-analyzer authorization
may occur first. At candidate freeze, the project must refresh the exact source
scope and reviewer brief, obtain the attributable independent human report,
remediate or disposition findings, and then run the remaining release gates.

The historical candidate-freeze portion is complete. PR 156 froze product source at
`1a9923c0e5d671581f6b7da3bc4248b604971d63`; exact candidate run
`33323269945` passed on macOS arm64, Linux x86-64, and Windows x86-64. The
refreshed historical scope pins the package evidence and release controls but
cannot satisfy a later release after production code changes. The roadmap
sequence is genuine Linux rootless login-session evidence, scheduled
maintenance automation, local-VM macOS confinement, Windows native
confinement, stable VM/cask distribution, verified updates, and platform-gated
YARA admission. A new final candidate and attributable independent human report
follow those intended release contents; no tag or publication is authorized.

ADR-0086 freezes fail-closed scheduled compatibility and maintenance
automation. ADR-0087 replaces only the failed macOS XPC analyzer-execution
topology with a fresh local Linux VM feasibility candidate while retaining one
CLI-compatible cask as the desired user experience. ADR-0088 adds an
independently evidenced Windows LPAC/AppContainer plus Job Object feasibility
track. ADR-0089 selects YARA as the first real analyzer but keeps execution
closed until an exact platform has current production IAR-1B support. ADR-0090
owns the immediate profile-A checkpoint: two genuine PAM/logind sessions for a
temporary non-lingering Linux user, without a privileged product install or
persistent service. Protected run `33341872303`, job `99338854149`, passed from
exact commit `bf2504f78ddb4e709407a0ac5c23d5d0ecc534a6`. Its source-free receipt
identity is `50ceac6df76bf90f40f6e888bb931ac84e5d18acaa7d8a442834adbcbe2538d4`.
Both session and user-manager identities were distinct, the first manager
terminated, package identity was stable, and every cleanup condition passed.
Profile A is therefore an exact-host synthetic lifecycle candidate; production
admission and real analyzers remain closed. ADR-0086 scheduled maintenance is
implemented with bounded metadata observations, deterministic six-state
receipts, exact-owned issue reconciliation, and a monthly no-release candidate
rehearsal. Default-branch runs `33345269371` and `33345318603` passed from
`d196f4cfc0332fd3bcfa6e93ec3bb95f5d8706ff`, created five exact-owned issues,
and proved repeat reconciliation without duplicates. No observed upstream
version was admitted automatically. ADR-0087's first native local-VM checkpoint
then passed the two requirements the XPC topology failed: a hard-capacity
scratch device and cross-job isolation across two fresh guests. Exact read-only
synthetic input, absent guest networking, stop, and per-job removal also passed
on macOS `26.5.1` arm64. The next partial matrix froze a reproducible exact
guest and passed tampered-guest rejection, bounded output-flood rejection,
malformed-result rejection, whole-VM timeout and forked-descendant stop,
early-exit cleanup, controller cancellation, and post-fault recovery. Full
macOS IAR-1B remains closed. The following Rust-supervisor checkpoint then
passed pre-launch controller-digest verification, exact external cancellation,
forced controller kill/reap, exact stale-job removal, and a fresh recovery VM
after each action while preserving the single audited analyzer launch site.
The following resource/canary checkpoint then froze a separate synthetic guest
and passed exact cgroup v2 memory OOM containment, CPU throttling, process-count
accounting, cgroup removal, exact attached-device enumeration, six host-only
canary classes, prohibited host-path absence, host-controller process-
identity absence, byte-exact host-canary retention, and exact job cleanup
through the same Rust launch boundary. Host sleep/interruption, sealed guest
supply-chain and distribution, multi-host evidence, independent review,
production, and real analyzers remain closed.
The next checkpoint installed the macOS will-sleep observer and proved the
shared fail-closed stop, exact cleanup, and fresh recovery path using only a
job-private synthetic interruption trigger. Its contract fixes
`real_host_sleep_observed=false`; genuine sleep/wake, reboot, power loss,
sealed supply chain and distribution, multi-host evidence, independent review,
production, and real analyzers remain closed.
The following offline supply-chain checkpoint then froze the expiring guest
release manifest, exact six-component inventory, SPDX SBOM, license record,
source/build provenance, vulnerability policy, and explicit initial rollback
identity. Both source-only CI validation and exact prepared-artifact validation
passed. Publisher authentication, vulnerability disposition, Developer ID
signing/notarization, one-cask lifecycle, genuine sleep/reboot/power-loss,
multi-host evidence, independent review, production, and real analyzers remain
closed.
The next explicit release-time check then verified Alpine's detached signature
on the exact 3.24.1 aarch64 netboot archive under the fingerprint published on
Alpine's official downloads page. Its embedded kernel and initramfs exactly
match the frozen guest inputs. Upstream publisher authentication is therefore
closed for this candidate; Impresari metadata sealing, vulnerability
disposition, Apple signing/notarization, one-cask lifecycle, multi-host and
disruptive lifecycle evidence, independent review, production, and real
analyzers remain closed.
The following bounded vulnerability review compared the exact authenticated
`6.18.35-0-virt` guest kernel with Alpine 3.24 aarch64 `linux-virt`
`6.18.48-r0`. Because the candidate is thirteen stable patch releases behind
and Alpine's published `linux-lts` secdb record does not establish complete
6.18 advisory coverage, the exact candidate is denied and must be replaced.
The review is dispositioned, but `vulnerability_assessment_complete=false`;
no specific CVE applicability or vulnerability-free claim is made. Current
guest replacement and renewed review now precede release-metadata sealing.
The next checkpoint completed that replacement with the exact authenticated
Alpine `linux-virt-6.18.48-r0` package. Versioned v2 profiles preserve the
historical denial evidence while cross-binding the current kernel, module,
initramfses, controller and Rust supervisor contracts, release records, and
rollback predecessor. All native synthetic matrices passed again. The renewed
review records the candidate as current with no further replacement required,
but complete current `6.18` advisory coverage remains unestablished, so
vulnerability completion, distribution sealing, production, and real analyzers
remain closed.
The following release-metadata checkpoint content-addresses all sixteen active
v2 metadata and public-verification members under one canonical
path/size/SHA-256 set digest. The exact profile and offline receipt bind the
guest manifest, component set, upstream authentication, incomplete
vulnerability assessment, rollback predecessor, and every active runtime
profile. Repository metadata sealing is therefore closed. GitHub publication
attestation, Developer ID signing/notarization, one-cask lifecycle, genuine
sleep/reboot/power-loss, multi-host evidence, complete advisory coverage,
independent review, production, and real analyzers remain closed.
ADR-0092 now freezes the complete intended Windows LPAC/AppContainer, Job
Object, mitigation, resource, staging, output, and cleanup profile. PR 181 run
`33361303368`, job `99393036278`, passed its deliberately smaller native
preflight on Windows Server 2025 build `26100` x86-64 from source
`393bc0b40d57fad0a5cb88cfe22394148f6bf464`. It verified NTFS, the required
API surface, empty Job Object set/query with kill-on-close and no breakaway,
and a unique zero-capability AppContainer create/derive/delete lifecycle. The
validated receipt keeps worker launch, network/path/resource/descendant denial,
complete cleanup, OS confinement, production, and analyzer execution false.
Windows feasibility has therefore advanced without claiming a sandbox; the
suspended synthetic-worker matrix remains the next Windows gate.
ADR-0093 now freezes that next gate. Its digest-bound profile requires a
first-party worker created suspended under a fresh zero-capability LPAC
identity, exact ACL staging with AppContainer profile-storage write removed,
three exact inherited protocol handles, compatible mitigations, child-process
denial, and a pre-resume Job Object assignment. Nineteen source-free scenarios
cover positive input, path/network/registry/handle/process denials, resources,
faults, cleanup, and cross-job isolation. Contract fixtures retain every
measured field false; native execution remains pending. Even a complete first
matrix keeps OS confinement, production, and analyzer execution false until a
later independent-host and lifecycle admission decision.
PR 183 then executed that exact matrix on GitHub-hosted Windows Server 2025
build `26100`. Final run `33366224611`, job `99407325602`, from source
`a402d25cb7d80351f7ff7c875c58849025f5ed8c` returned `unsupported_host` after
`CreateProcessW` failed with error `5` before worker code ran. The validated
receipt kept every denial, confinement, production, analyzer, and authority
claim false, and cleanup completed. This closes the legacy no-admin LPAC path
on that exact host without weakening it or adding privileged host preparation.
ADR-0094 therefore opens a narrower BaseContainer capability-routing
checkpoint. A dedicated Windows 11 arm64 job may read only the product type,
build, filesystem, and trusted System32 `processmodel.dll` export set. It must
return a deterministic unsupported state below build `26600` or when the exact
experimental exports are absent. It cannot invoke the API, launch a worker,
mutate ACLs or Windows features, elevate, install a service, claim IAR-1B, or
admit production or real analyzers. A capability-ready result may authorize
only a separately frozen synthetic-worker rehearsal.
PR 184 run `33367923249`, job `99412379212`, completed that checkpoint from
source `e74be1bda94c6285671120723610fd611a9fdd27`. Windows 11 Enterprise build
`26200` arm64 exposed `processmodel.dll` and both exact experimental exports,
but the frozen minimum is build `26600`; the validated result is therefore
`unsupported_build`. No worker launched and every confinement, production,
analyzer, and authority claim remained false. The Windows roadmap now waits
for an independently available qualifying build rather than weakening the
contract or introducing automatic administrator host preparation.

ADR-0095 completes the next non-executing prerequisite for ADR-0089. It freezes
the digest-bound `yara-adapter-contract-v1` profile, a production-shaped but
original-synthetic result, deterministic normalization, complete artifact
accounting, exact byte-range evidence bindings, and provenance-bound positive
and negative fixtures. The checkpoint introduces no YARA binary, rules,
process launch, parser, repository analyzer input, network, credential access,
IAR-2, production admission, safety claim, or authority. Live YARA activation
still requires an exact production-admitted IAR-1B platform, signed analyzer
and ruleset artifacts, a separately reviewed live-result contract and hosted
evidence, and the applicable independent human security review.

ADR-0096 completes the next metadata-only prerequisite. It selects the exact
official YARA v4.5.8 tag commit and BSD-3-Clause license-file identity, records
that the upstream release has no uploaded assets, and freezes a 30-day
source-selection expiry plus revocation-first state precedence. A future
per-target Impresari executable and project-owned ruleset must remain separate,
content-addressed, reviewed, signed, expiring artifacts with complete build,
SBOM, provenance, reproducibility, license, and rollback evidence. This
checkpoint downloads no source, creates no binary or rule, uses no network at
verification time, and does not change the closed IAR-2 or production posture.

The following build-profile audit found a material upstream transition: the
official YARA v4.5.8 project is now maintenance-focused and directs enhancement
work to stable YARA-X. ADR-0097 selects YARA-X before the first analyzer build.
The exact legacy YARA candidate remains evaluated evidence, not wasted work or
an admitted artifact. The engine-specific parts of ADR-0089/0096 are
superseded, and the ADR-0095 adapter must receive new YARA-X identities. No
engine-specific build, ruleset, live adapter, source download, or execution
may begin until the replacement contracts are frozen.

ADR-0098 freezes those replacement contracts without creating an artifact or
execution path. The official YARA-X v1.20.0 release assets remain metadata-only
candidates because their upstream build uses mutable toolchain/runner labels
and supplies no per-asset signature or SLSA provenance. Production uses a
separately pinned, rebuilt, reviewed, and Impresari-signed artifact. The first
ruleset is module-free and project-owned; the future invocation is one compiled
ruleset plus one staged file with closed arguments and bounded NDJSON that
retains zero matched bytes. The next checkpoint is exact artifact and ruleset
creation/admission, still gated from execution.

ADR-0099 opens only that synthetic compatibility checkpoint. It freezes the
immutable source archive and a minimal module-free Impresari patch, authors an
original-synthetic literal/hex/wide ruleset, and requires a locked Rust build
plus synthetic positive/negative scans inside the existing Linux CI isolation
boundary. The binary, compiled rules, raw results, and receipt remain
ephemeral. Repository inputs, live parsing, signatures, uploads, production
admission, detection claims, and IAR-2 remain closed pending separate review of
the hosted evidence.

Run `33406541396`, job `99535422988`, completed that hosted review input on
2026-08-31. The exact patched v1.20.0 candidate passed the Linux composite,
all five original-synthetic compatibility cases, and mandatory cleanup on
Ubuntu 24.04 x86-64. No artifact was uploaded or admitted. The next roadmap
checkpoint is a separately frozen live NDJSON adapter and production-artifact
pipeline design; repository-derived execution, signatures, publication,
production admission, and IAR-2 remain closed.

ADR-0100 therefore selects the pure parser as the next lowest-authority step.
It freezes a one-record YARA-X NDJSON boundary and permits only offline parsing
of provenance-bound original-synthetic fixtures into a path-free normalized
result. It introduces no analyzer process, repository-derived input, runner
linkage, artifact/ruleset admission, production, or IAR-2 authority. Production
artifact design and synthetic runner-envelope linkage remain separate later
decisions after the parser corpus passes.

The ADR-0100 parser corpus now passes. The content-addressed profile, closed
schemas, pure Rust implementation, deterministic normalized result, and
original-synthetic provenance are implemented without adding any runtime
capability. The next analyzer checkpoint must independently choose either a
production artifact pipeline or a synthetic runner-to-adapter envelope. It may
not introduce repository-derived input or production/IAR-2 claims by
implication.

ADR-0101 chooses and implements the synthetic runner-to-adapter envelope as
that next checkpoint. It uses a dedicated Impresari-owned emitter and already
admitted synthetic isolation to prove bounded process-output capture, exact
identity handoff, parser composition, and cleanup without executing YARA-X.
The source-free contracts, coordinator, closed emitter, runner reuse, and local
non-executing matrix are complete. Run `33419412353`, job `99577842304`, passed
both closed cases and mandatory cleanup in the ephemeral hosted Ubuntu 24.04
boundary. YARA-X did not execute and no artifact or ruleset was admitted.
Production artifact creation and signing remain the following independent
gate.

ADR-0102 implements the next narrow checkpoint: real output from the exact
ephemeral YARA-X v1.20.0 candidate is captured through the single audited
Analyzer Runner site, inside the admitted Linux boundary, and passed in memory
to the ADR-0100 adapter. Only the five generated Impresari synthetic cases are
eligible. Run `33432469614`, job `99620875408`, passed the manual
empty-workspace hosted matrix and mandatory cleanup on Ubuntu 24.04 x86-64.
Executable and ruleset admission, repository scanning, production, IAR-2,
detection quality, and safety remain closed regardless of the compatibility
result. The next analyzer checkpoint is the separately reviewed production
artifact and ruleset admission pipeline; hosted synthetic success does not
activate it by implication.

ADR-0103 now fixes that production-admission architecture before retained
artifact work. The engine bundle, project-owned ruleset bundle, fresh Linux
support receipt, and product release are independently content-addressed and
joined only by a final source-free binding manifest. The ordered next work is:
closed schemas/evaluators, retained candidate engine build, independently
reviewed production ruleset, signing/publication and lifecycle rehearsal, then
a separate activation review. Repository-derived IAR-2 input remains a later
decision. Current authorization also covers the bounded no-upload build and
synthetic compatibility work recorded by ADR-0102 and ADR-0105; no production
artifact, rule, signing identity, upload, or activation is implied.

The first ADR-0103 source-free evaluator is implemented under policy
`sha256:fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c`.
It deterministically reports the current `release_pending` state and fixes
`active` as unreachable while the policy's activation bit is false. Closed
registered candidate schemas now cover the engine bundle, ruleset bundle, and
release binding, including fail-closed negative fixtures. The contract stage is
complete. ADR-0105 investigated the changed executable identities observed
across otherwise matching hosted builds before ADR-0104 retention approval.
That diagnostic remains ephemeral, build-only, and no-upload; its canonical
result is an input to the retained candidate's explicit reproducibility
disposition, not a production claim.

ADR-0104 now implements the approved boundary for that next step: one manually
dispatched, no-secret, Linux x86-64 build in an exact digest-pinned image; one
authenticated, non-release seven-day GitHub Actions artifact containing only
the engine and bounded supply-chain evidence; and a separate non-executing
verifier. Exact-main run `33460329608` retained exactly one candidate and
passed the separate non-executing verifier and both cleanup gates. Signing,
publication, production rules, activation, repository scans, and IAR-2 remain
later independent decisions.

ADR-0105 completed its active pre-retention diagnostic in run `33443483096`.
The ordinary clean builds differed, while two fixed time/path-remapped clean
builds were byte-identical at SHA-256
`a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d`.
This proves only same-job canonical equality; cross-run, cross-host, retained
artifact, and production reproducibility remain unproven. The run compiled no
rules, executed no analyzer, and uploaded no artifact. ADR-0104 later received
its own narrow retention authorization without inheriting any broader claim.

ADR-0106 Option A is implemented as three original, project-owned,
module-free observation rules and twelve generated non-malicious fixtures. The
closed source-only evaluator verifies the permitted literal/hex surface,
per-rule provenance, limitations, and positive, near-miss, benign-collision,
and mutation expectations without invoking YARA-X. The independent
attributable human ruleset review is backlogged while ordinary roadmap work
continues, but remains mandatory before compilation or retention. No analyzer
execution, signing, publication, production, repository-scan, or IAR-2
authority is added.

ADR-0107 returns to the accepted macOS local-VM distribution sequence without
crossing the Apple credential boundary. It freezes one source-free,
CLI-compatible cask contract: one `Impresari Context.app`, one public
`impresari-context` link, a closed embedded role layout, exact ADR-0091 metadata
binding, whole-bundle install/upgrade/rollback, formula-conflict rejection, and
narrow uninstall without `zap`, package scripts, privileged helpers, or
background services. The deterministic receipt keeps assembly, publication
attestation, Developer ID signing, notarization, live cask lifecycle, sealed
distribution, production, macOS IAR-1B, and analyzer execution false. The next
reversible checkpoint is unsigned synthetic-only bundle assembly and exact
layout verification without installation.

ADR-0108 completes that checkpoint. A portable offline checker assembles the
13-entry `Impresari Context.app` tree twice under private temporary roots,
binds the ADR-0107 contract and ADR-0091 metadata seal, verifies the exact
canonical tree digest, keeps every apparent executable as a mode-`0644`
synthetic marker, and removes both roots. No release app, archive, cask,
installation, source-revision binding, signing, notarization, publication, VM,
analyzer, production, or macOS IAR-1B authority is added. The next reversible
checkpoint is to freeze the build and release-identity contract for substituting
real unsigned product and guest candidates; it must not yet sign, install,
publish, launch a VM, or execute an analyzer.

ADR-0109 completes that contract checkpoint. It separates the current contract
baseline from a future candidate source revision, hashes the exact direct
build-control inputs, closes four product build units plus the metadata-sealed
guest role, freezes the future unsigned-candidate schema, and requires exact
Apple/Rust build identities, artifact digests, SBOM, licenses, vulnerability,
reproducibility, guest, compound-identity, and rollback evidence before any
marker can be replaced. No candidate was compiled or retained, and release
identity, bundle assembly, signing, notarization, installation, publication,
VM, analyzer, production, and macOS IAR-1B remain false. The next reversible
checkpoint is a bounded unsigned-candidate build-and-cleanup design; it must
resolve exact build-host identity and artifact custody before implementation
and still must not access Apple credentials, install or publish a cask, launch
a VM, or execute an analyzer.

ADR-0110 completes the product-only build-and-cleanup checkpoint. From exact
revision `aca656771f9286b13fbcc046b133ade62b58da2a`, two independent private
offline builds produced byte-identical arm64 Mach-O outputs for the CLI, MCP
server, structural worker, and Swift VM controller. Exact source, host,
toolchain, artifact, ad-hoc code identity, dynamic-library, SBOM, license,
advisory, reproducibility, and cleanup metadata is retained; every runnable
byte, cache, and raw log was deleted. This is not a complete ADR-0109 release
candidate because no guest was materialized and no app was assembled. The next
reversible checkpoint is an exact authenticated guest-candidate substitution
design that can complete the unsigned release record without yet accessing
Apple credentials, signing, notarizing, installing or publishing a cask,
launching a VM, or executing an analyzer.

ADR-0111 completes that source-free substitution design. The ordinary guest
payload is now closed to exactly two mode-`0644` runtime resources, `Image` and
`impresari-initramfs.gz`, while the standalone init, module, and resource-canary
assets remain explicit build or test intermediates. The contract binds the
ADR-0091 manifest and metadata seal, the controller's exact ordinary asset
names, the publisher-authenticated Alpine APK, exact extraction inputs,
Impresari-owned guest source, Zig target/options, canonical initramfs builder,
and mandatory later private-root cleanup. No package was downloaded, no guest
byte was built or retained, and no release, app, signing, notarization, cask,
VM, analyzer, production, or macOS IAR-1B authority was added. The next
reversible checkpoint is an ephemeral authenticated materialization-and-delete
rehearsal for only those two synthetic guest resources; it must remain separate
from app assembly, Apple credentials, distribution, VM launch, and analyzers.

ADR-0112 completes that ephemeral rehearsal. One exact publisher-authenticated
Alpine package was downloaded into a fresh private root, its APKv2 signature,
signed data hash, package identity, architecture, version, and source commit
were verified, and only the kernel and `virtio_blk` module were selected. Zig
0.16.0 and the frozen project builders reproduced the exact two ADR-0111
payload identities. Neither output was executed, and every downloaded,
extracted, built, cached, and log byte was deleted before the metadata-only
record was retained. App assembly, Apple credentials, signing, notarization,
cask lifecycle, VM launch, analyzers, release identity, production, and macOS
IAR-1B remain gated. The next reversible checkpoint is source-free composition
of the already proven product and guest identities into the frozen unsigned
release-candidate contract before any runnable app assembly or Apple identity
access.

## Parallel Client Integration Depth Track

Codex, Claude Code, Cursor, and GitHub Copilot follow the separate
[Client Integration Depth Roadmap](client-integration-roadmap.md). A client is
first-class only after its L1 managed-connection evidence passes; native
guidance, planner-backed delivery, and deep lifecycle support are separately
admitted L2–L4 claims. This track runs in parallel with Phases 1–4 and does not
block structural-language or deterministic-planner releases.

Gemini CLI remains generic legacy compatibility because Google's consumer path
has moved to Antigravity. Do not invest in additional Gemini-specific depth
until a stable Antigravity surface has been evaluated.

## Phase 0 — Correct the public contract

Correct unsupported structural-language claims; publish discovery, lexical, and
structural evidence levels; publish generic, first-class, experimental, and
unsupported client classifications; and provide a read-only doctor command for
binary, parser worker, cache, workspace isolation, and MCP-handshake checks.

## Phase 1 — Python and configuration evidence

Deliver Python plus JSON, JSONC, TOML, and deliberately bounded YAML structural
evidence. Configuration claims are limited to key/object containment, manifest
fields, safe defined include/reference relationships, and only exact
configuration-to-code references. Deliver first-class tested local connection
kits for Codex, Claude Code, and Cursor.

## Phase 2 — Rust and Go, plus broader agent access

Deliver Rust and Go structural evidence and tested local connection kits for
Gemini CLI, GitHub Copilot CLI, and VS Code Copilot. The language slices and
GitHub Copilot CLI L1/L2/L4 and the distinct VS Code Copilot extension-host
L1/L2 admissions are complete for their recorded scopes; Gemini remains legacy
generic compatibility. Deeper Copilot work is governed by the parallel
client-integration roadmap.

## Phase 3 — Deterministic Context Planner

Deliver a bounded deterministic planner—not agent governance—that consumes a
declared task profile, query, snapshot, policy, budget, and supported evidence.
It emits an explicit retrieval plan, selection reason codes, coverage report,
omitted candidates and budget reasons, and exact packet identity. Initial
profiles are orientation, implementation, bug investigation, change review,
security review, test selection, and configuration change.

## Phase 4 — Enterprise-language expansion and impact evidence

Deliver Java, Kotlin, and C# structural evidence, then strengthen change-set
packets, bounded impact paths, associated-test evidence, orientation packets,
explicit incremental structural updates, and deterministic conventions and
exemplar evidence.

## Phase 5 — Demand-led language expansion

Scala, Elixir, Clojure, Haskell, C, C++, Ruby, PHP, and Swift are admitted with
bounded structural evidence. The founder-approved five-language program in
ADR-0064 completed its independent hosted acceptance gates in order. Evaluate
any other language—including F#, Elm, Dart, and carefully constrained SQL—only
from attributable adopter demand and evaluation evidence.
