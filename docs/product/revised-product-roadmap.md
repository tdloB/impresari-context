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
version was admitted automatically. The next implementation checkpoint is the
ADR-0087 local-VM macOS confinement feasibility candidate.

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
