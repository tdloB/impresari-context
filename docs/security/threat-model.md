# Impresari Context — Security Threat Model

## Document Control

- Document ID/version: IC-SEC-001 / 0.1.
- Status: Founder-approved security design baseline; implementation evidence
  remains required. ADR-0017 governs independent review for `v0.1.0`; its
  earlier-review trigger is active for the proposed v0.2.0 scope under ADR-0083;
  ADR-0084 defers engagement until candidate freeze without waiving the gate.
- Date: 2026-08-20.
- Initial scope: Slice A — Verifiable Local Context MVP plus the authorized
  Slice B structural-worker boundary in ADR-0010.
- Future scope represented but not authorized: consumer adapters, general
  extensions, model access, network access, and hosted deployment.
- Current separately authorized scope includes consent-gated external-client
  delivery adapters and application/OS isolation candidates. Their exact
  boundaries and non-claims are reviewed in the v0.2.0 independent-review brief.
- Parent requirements:
  - [Master Product PRD](../product/master-prd.md)
  - [Verifiable Local Context MVP PRD](../product/verifiable-local-context-mvp-prd.md)
  - [System boundaries](../boundaries.md)

## Purpose

Define the assets, trust zones, adversaries, misuse cases, security invariants,
required controls, verification methods, and residual risks for a local engine
that reads untrusted software repositories and emits source-derived evidence.

This document is a design and verification contract. It does not claim that the
future implementation is secure, sandboxed, independently audited, or suitable
for sensitive code merely because the specified controls are implemented.

## Security Objectives

1. Read only data inside the exact authorized workspace scope.
2. Never let repository content alter control flow, policy, or capabilities.
3. Prevent source-derived data from crossing workspace or consumer boundaries.
4. Make stale, corrupt, partial, unsupported, and derived information visible.
5. Preserve exact evidence integrity and provenance.
6. Deny network, process execution, source mutation, extension access, and
   durable-memory promotion unless separately designed and authorized.
7. Limit CPU, memory, storage, time, traversal, and output consumption.
8. Avoid exposing source, secrets, sensitive paths, queries, or environment
   values through logs, errors, caches, exports, or diagnostics.
9. Make releases and dependencies reproducible and attributable.
10. Fail closed when authorization, redaction, integrity, or capability state is
    ambiguous.

## Security Non-Objectives

The MVP does not promise to:

- protect source from a malicious user who already has equivalent local OS
  access;
- make an untrusted host operating system trustworthy;
- prevent an authorized user from intentionally exporting authorized content;
- detect every secret, vulnerability, license issue, or malicious source file;
- securely execute repository code, build scripts, parsers that require code
  execution, or arbitrary third-party extensions;
- provide a hardened multi-tenant boundary;
- provide cryptographic non-repudiation or remote attestation;
- replace operating-system permissions, disk encryption, endpoint security, or
  organizational access controls.

## System Model

```text
Authorized user / consumer
        |
        v
CLI or in-process adapter
        |
        v
Capability and policy gateway
        |
        +-----------> metadata-first local audit
        |
        v
Workspace controller ---- read-only ----> Untrusted source workspace
        |
        v
Snapshot / lexical index ---- read-write -> Isolated derived cache
        |
        v
Evidence normalizer and packet builder
        |
        +-----------> Explicit local export root
```

There is no outbound network, arbitrary repository process, extension host,
model provider, or hosted control plane. Slice B adds one pinned,
capability-reduced parser worker that receives bounded source bytes and no
workspace path or ambient execution authority.

## Assets

| Asset | Security need |
| --- | --- |
| Source code and documentation | Confidentiality and no mutation |
| File and directory names | Confidentiality; names may themselves be sensitive |
| Git and workspace metadata | Confidentiality and integrity |
| Workspace authorization scope | Integrity and least privilege |
| Snapshot and evidence identities | Integrity, freshness, and non-confusion |
| Context packets and exports | Confidentiality, integrity, provenance, and scope |
| Cache/index | Confidentiality, isolation, integrity, and safe deletion |
| Policy decisions and caller identity | Integrity and auditability |
| Audit records | Integrity, minimization, and safe retention |
| Local environment and credentials | Confidentiality; inaccessible by default |
| Release artifacts and dependencies | Integrity, authenticity, provenance, and availability |
| User trust in observed versus derived results | Correct labeling and visible uncertainty |

## Trust Zones

### Z1 — Engine control plane

Request validation, policy evaluation, capability dispatch, budget enforcement,
and audit metadata. Trusted only when running an approved, pinned build with
approved configuration.

### Z2 — Source workspace

Always untrusted. File content, paths, filenames, symlinks, Git metadata,
configuration, generated files, and comments may be malicious, deceptive,
sensitive, malformed, oversized, or changed concurrently.

### Z3 — Derived cache

Sensitive and replaceable. It may contain source-derived content and indexes.
It cannot be trusted without integrity and workspace/snapshot validation.

### Z4 — Consumer interface

Supplies purpose, caller identity, workspace request, and budgets. It may be
misconfigured or malicious and cannot override core safety invariants.

### Z5 — Export destination

Explicitly authorized local destination. Exported packets remain sensitive and
do not carry new workspace or action authority.

### Z6 — Structural parser worker and future extensions

Untrusted by default. The authorized structural worker is governed by ADR-0010,
receives bounded source bytes through a single framed request, and returns
hostile output that is validated all-or-nothing. General extensions remain
future work and require separate filesystem, network, environment, output,
integrity, and lifecycle controls.

### Z7 — Future external providers

Denied. Later model/network access requires destination allowlists, data
classification, redaction, retention disclosure, consent/approval, audit, and
failure design.

## Adversaries And Failure Sources

| Actor/source | Capability or behavior |
| --- | --- |
| Malicious repository author | Controls repository files, names, symlinks, metadata, size, encoding, and instructions |
| Compromised dependency | Executes within engine privileges or corrupts parsing, hashing, serialization, or update behavior |
| Malicious or careless local caller | Requests broad paths, complex patterns, oversized budgets, cross-workspace handles, or unsafe exports |
| Concurrent local process | Changes paths or content between validation and read |
| Compromised cache or local account | Alters index, packet, configuration, or audit files |
| Future malicious extension/parser | Attempts undeclared reads, execution, network, environment access, output spoofing, or persistence |
| Accidental developer error | Produces incomplete discovery, wrong spans, stale evidence, secret logging, or permissive defaults |
| Resource exhaustion input | Uses huge files, many files, long lines, Unicode/path edge cases, complex patterns, cycles, or rapid mutation |
| Supply-chain attacker | Publishes a malicious dependency, toolchain, build action, package, or release artifact |

## Security Invariants

| ID | Invariant | MVP gate |
| --- | --- | --- |
| SEC-INV-001 | Authorization occurs after canonical resolution and before enumeration/content read | Block |
| SEC-INV-002 | The core never writes to the source workspace | Block |
| SEC-INV-003 | Repository content is data and cannot alter policy, tool permissions, or execution | Block |
| SEC-INV-004 | Exact evidence resolves only against its authorized matching workspace snapshot | Block |
| SEC-INV-005 | Cache is never the sole authority for exact source | Block |
| SEC-INV-006 | Snapshot mismatch, corruption, and partial indexing are visible | Block |
| SEC-INV-007 | Network, arbitrary repository process execution, general extensions, and telemetry are absent/denied; only the pinned ADR-0010 parser worker may be launched | Block |
| SEC-INV-008 | Source content and secrets are excluded from logs by default | Block |
| SEC-INV-009 | Every operation enforces resource and output budgets | Block |
| SEC-INV-010 | One workspace cannot resolve another workspace's handles or cache | Block |
| SEC-INV-011 | Derived output cannot claim exact-source authority without a verified source hash and span | Block |
| SEC-INV-012 | Consumer-supplied purpose, role, prompt, or content cannot broaden capabilities | Block |
| SEC-INV-013 | Exports require an explicit allowed destination and preserve sensitivity metadata | Block |
| SEC-INV-014 | Optional/future model or extension failure cannot destroy exact evidence recovery | Future block |
| SEC-INV-015 | Durable knowledge cannot be promoted without consumer-defined approval | Future block |

## Data Classification

| Class | Examples | Default handling |
| --- | --- | --- |
| Control metadata | Request IDs, versions, timing, outcome codes | May enter minimized audit |
| Workspace metadata | Canonical identity, relative paths, revision, file metadata | Sensitive; minimize and scope |
| Source-derived content | Code, documentation, excerpts, index terms | Confidential; local only; no default logs |
| Secret-like content | Tokens, keys, credentials, private URLs, personal data | Restricted; redact/omit under policy; never log |
| Packet/export | Evidence plus metadata and possible excerpts | Inherit highest contained classification |
| Configuration/environment | Paths, policy, environment keys/values | Keys minimized; values denied unless explicitly required |
| Derived claim | Deterministic or future model output | Labeled derived; linked to evidence; never elevated silently |

The engine may provide configurable detection and redaction assistance, but it
must not claim that automated detection finds every secret or sensitive datum.

## Threat And Control Register

### Filesystem authorization and isolation

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-001 | Parent/path traversal | `..`, mixed separators, prefixes, alternate roots, or normalization escapes approved root | Explicit root, canonical resolution, component-aware containment, platform fixtures | Unit + adversarial | OS-specific path semantics |
| SEC-T-002 | Symlink escape | In-root link points outside root | Resolve link target before authorization; deny or separately authorize; never follow by default across root | Adversarial | Concurrent swaps |
| SEC-T-003 | TOCTOU/path swap | Path changes after check and before read | Revalidate at read, prefer descriptor-relative/no-follow APIs where supported, verify identity before/after read, fail on mismatch | Race harness | Host/filesystem limitations |
| SEC-T-004 | Special-device access | FIFO, socket, device, procfs-like or pseudo-file causes block/leak | Permit regular files/directories only by default; nonblocking metadata check; deny special types | Fixture suite | Platform-specific types |
| SEC-T-005 | Hard-link/alias confusion | Authorized path aliases content with unexpected identity | Record file identity where supported; never infer broader authorization; document hard-link limits | Platform tests | Same-filesystem aliases may be indistinguishable |
| SEC-T-006 | Case/Unicode collision | Canonically equivalent or case-folded paths confuse cache/evidence | Define platform-aware path identity, Unicode policy, collision detection, no lossy normalization | Cross-platform fixtures | Filesystem behavior differences |
| SEC-T-007 | Nested repository/submodule escape | Git metadata or submodule points outside root | Treat Git metadata as untrusted; filesystem scope remains authoritative; explicit nested/submodule policy | Integration tests | Incomplete Git metadata |
| SEC-T-008 | Unsafe export destination | Traversal or symlink writes packet outside allowed export root | Explicit export root; canonical parent checks; safe create; no overwrite by default; revalidation | Adversarial | Local user can move exported file later |

### Repository-content and prompt-injection threats

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-009 | Repository prompt injection | Comments/docs say to ignore policy, expose secrets, or run commands | Typed request/control channel; content labeled untrusted; no policy parsing from source; no execution/network capability | Injection corpus | Downstream consumer may misuse evidence |
| SEC-T-010 | Output/control spoofing | Source mimics packet fields, tool messages, ANSI sequences, or delimiters | Structured serialization; escaping; terminal-safe rendering; source never merged into control fields | Fuzz + golden tests | Human may still be socially engineered |
| SEC-T-011 | False exact authority | Derived text claims it is verified source | Exact evidence kind requires independently computed hash/span; immutable typed fields | Schema/property tests | Semantic truth still needs review |
| SEC-T-012 | Secret exfiltration via query/log | Query, match, error, or debug event includes sensitive content | Metadata-first logging; source/query omission; redacted diagnostics; log tests | Log inspection | Local authorized output can contain source by design |
| SEC-T-013 | Malicious encoding/parser input | Invalid UTF, bidi controls, nulls, long lines, malformed files | Bounded decoding; raw-byte identity; explicit unsupported/binary status; safe rendering | Fuzz corpus | Display ambiguity in third-party clients |

### Integrity, freshness, and cache threats

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-014 | Cache poisoning | Modified index returns false matches or cross-project content | Workspace/snapshot namespacing, schema/version checks, integrity validation, source confirmation, rebuild path | Tamper suite | Local account with full access can also alter executable/config |
| SEC-T-015 | Stale evidence substitution | Old reference resolves against new content at same path | Bind to snapshot and content hash; verify before expansion; explicit stale state | Mutation suite | Files may change immediately after verification |
| SEC-T-016 | Partial index presented as complete | Limits/errors skip files silently | Record discovery totals/skips and completeness state; packet carries partial/unknown fields | Fault injection | Unobservable filesystem races |
| SEC-T-017 | Hash/canonicalization ambiguity | Different inputs share normalized identity or cross-platform rules diverge | Approved modern hash; domain separation; documented byte/canonicalization contract; version fingerprints | Conformance fixtures | Cryptographic collision remains theoretical residual |
| SEC-T-018 | Cross-workspace handle confusion | Handle from A resolves in B | Opaque IDs bind workspace + snapshot + evidence; policy recheck on every resolution | Isolation suite | Caller may intentionally copy source outside engine |
| SEC-T-019 | Packet tampering | Serialized fields or excerpts changed | Canonical packet identity/integrity check; reject corrupt; do not auto-repair | Tamper tests | No signer/non-repudiation in MVP |
| SEC-T-020 | Rollback/downgrade confusion | Old cache/schema interpreted by new engine | Compatibility matrix, schema version, explicit migration/rebuild, no implicit downgrade | Upgrade tests | User can run old vulnerable binary |

### Resource exhaustion and availability

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-021 | File-count/size exhaustion | Millions of files, huge files, long lines, generated trees | File/count/byte/depth limits, ignore rules, partial state, streaming reads | Stress suite | Legitimate monorepos may require tuning |
| SEC-T-022 | Pattern complexity | Catastrophic regex/backtracking or huge match set | Safe engine or restricted syntax, compiled-size limits, time/match/output budgets | Adversarial patterns | Some useful patterns may be unsupported |
| SEC-T-023 | Memory amplification | Index/excerpt/serialization expands far beyond input/budget | Streaming/bounded structures, size accounting, allocation limits, backpressure | Profiling + stress | Runtime allocator behavior |
| SEC-T-024 | Disk exhaustion | Cache, audit, temp, or export grows unbounded | Quotas, preflight space checks, atomic writes, retention/purge, no source-root cache | Fault injection | Host may exhaust disk externally |
| SEC-T-025 | Rapid mutation/livelock | Repository changes continuously during snapshot | Bounded retries, consistency check, partial/stale result, never infinite stabilization | Mutation stress | Busy workspaces may not yield current snapshot |
| SEC-T-026 | Cancellation corruption | Termination leaves authoritative partial cache | Temp/staging area, atomic commit, validity marker, cleanup/rebuild | Kill/restart tests | Abrupt power loss filesystem semantics |
| SEC-T-026A | Native parser compromise or crash | Malformed source exploits grammar/runtime, floods output, or spoofs graph facts | Short-lived worker; pinned executable/grammars; cleared environment; empty CWD; bounded framing/time/output; full response and span/provenance validation; no partial promotion | Worker protocol, crash, timeout, flood, malformed-frame, identity, and native-platform confinement tests | Application-enforced controls are not a complete OS sandbox; native C remains in the worker TCB |

### Interfaces, policy, and local data exposure

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-027 | CLI argument/terminal injection | Filenames/queries cause shell or terminal behavior | No shell interpolation; argv APIs; escape control characters; structured output | CLI adversarial tests | User may paste unsafe output elsewhere |
| SEC-T-028 | Overbroad default root | Empty/current/home/root path grants excessive access | Require explicit path; deny broad roots unless exact policy permits; clear confirmation outside library | Unit + UX test | Authorized user may deliberately approve broad root later |
| SEC-T-029 | Policy confused deputy | Consumer-supplied role/purpose grants more access | Opaque authenticated/local caller context; policy decision independent of prompt; capability allowlist | Policy matrix | MVP local identity is not strong authentication |
| SEC-T-030 | Error leakage | Absolute paths, sibling names, snippets, env values, stack traces exposed | Safe error envelope; authorized relative paths; diagnostics opt-in and local | Snapshot/error tests | Debug builds may be mishandled |
| SEC-T-031 | Permissive cache/export permissions | Other local users read derived source | Restrictive creation mode, documented local-user assumptions, permission checks where supported | Platform tests | ACL/backup/admin access outside engine |
| SEC-T-032 | Environment leakage | Engine reads or records unrelated environment/secrets | Explicit minimal configuration keys; no general enumeration; never record values | Instrumented tests | Runtime/toolchain may read environment internally |
| SEC-T-033 | Undocumented telemetry/network | Dependency makes outbound request | Network-denied full suite, dependency review, no update checker, build-time/runtime distinction | Network monitor | Host/runtime behavior outside packaged binary |

### Supply chain and release

| ID | Threat | Attack/failure path | Required controls | Verification | Residual risk |
| --- | --- | --- | --- | --- | --- |
| SEC-T-034 | Malicious dependency | Package executes, exfiltrates, or corrupts output | Minimize dependencies, lock, review critical crates/packages, SBOM, license/security scan, update policy | CI evidence | Zero-day/maintainer compromise |
| SEC-T-035 | Build/release compromise | CI, action, toolchain, or package account publishes altered binary | Pin CI actions/toolchains, least privilege, protected releases, reproducible/provenance plan, signing decision | Release rehearsal | CI/platform compromise |
| SEC-T-036 | Provenance/license contamination | Upstream code copied without compliance | Contribution rules, review checklist, dependency notices, original fixtures, source-reuse gate | Review + inventory | Contributor misrepresentation |
| SEC-T-037 | Unsafe automatic update | Binary/extensions update during sensitive work | No self-update; explicit user-controlled upgrade; version surfaced | Network/behavior test | Package manager policy outside engine |
| SEC-T-038 | Extension declaration/output spoofing | Untrusted manifest or output claims a trusted identity, exact evidence, control authority, or excessive payload | Closed manifest/output schemas; exact local digest pins; zero privileged grants; pre-parse length bound; identity matching; untrusted-derived label; metadata-only quarantine | Contract, pinning, spoofing, authority, unknown-field, and size tests | Digest pin is not publisher authentication; downstream consumers can still misuse untrusted payload text |
| SEC-T-039 | MCP framing/lifecycle abuse | Client sends oversized, malformed, batched, duplicated, out-of-order, or hostile JSON-RPC messages | Local stdio only; bounded newline framing; strict schemas; lifecycle state machine; unique request IDs; fixed tools; stdout purity; fail-closed errors | Protocol and adversarial process tests | Launching host controls inherited environment and process identity |
| SEC-T-040 | MCP authority confusion | Tool arguments or repository text attempt to select roots, grant access, orchestrate agents, or trigger execution | Roots fixed at process launch; thin delegation; constant no-authority results; no sampling/elicitation/network/execution; source treated as data | Equivalence, hostile-text, immutability, and capability tests | Authorized client may disclose returned evidence after receipt |
| SEC-T-041 | Release candidate substitution | Artifact is built from unexpected source, altered after build, or mislabeled as published/trusted | Exact source SHA; locked build; package manifest; checksums; SBOM; pinned CI actions; read-only workflow permissions; manual publication | Native package/rehearsal workflow | CI platform compromise; SHA-256 is integrity, not publisher authentication |
| SEC-T-042 | Hostile artifact classification spoofing | Extension, magic prefix, path, or declared platform attempts to trigger unsafe parsing or hide an execution surface | Regular-file boundary; bounded prefix only; extension/magic disagreement is explicit; closed artifact classes; no archive traversal or deep hostile-format parser | HRA schema, malformed-contract, resource-profile, cross-platform synthetic fixtures, and HRA-1 runtime inventory tests | Static classification can be incomplete or ambiguous and is not malware detection |
| SEC-T-043 | Coverage laundering | Zero findings, unavailable analyzers, or stale results are presented as complete or safe | Coverage is canonical and separate from findings; closed lifecycle states; incomplete mandatory analysis cannot yield a safety claim | HRA coverage and assessment fixtures; later deterministic truth tables | An authorized consumer can ignore the assessment outside Context |
| SEC-T-044 | Repository admission authority escalation | Repository text, analyzer output, policy data, or a model attempts to authorize host execution or claim safety | Closed policy fields and four decision states; `safety_claimed`, `ordinary_host_execution_authorized`, and `authority_added` fixed false; exceptions owned by an external authorized human | Invalid safety-claim and host-authority fixtures; schema conformance | Future quarantine eligibility still requires separate runtime and human gates |
| SEC-T-045 | Execution-surface rule confusion | A lifecycle-like key outside the admitted object, malformed JSON, duplicate/escaped syntax, or an untrusted command value attempts to create or alter an observation | Closed filename/key corpus; strict JSON validation; direct top-level object/key recovery; key-token-only evidence; values never interpreted; unsupported syntax explicit | HRA-2 false-positive, schema, stale-source, non-retention, and immutability tests | A declared lifecycle hook is an execution surface, not evidence of malicious intent |
| SEC-T-046 | YAML lexical context spoofing | A comment, block scalar, label, top-level key, alternate scalar, tab, alias, or ambiguous Compose layout resembles a privileged service declaration | Four exact basenames; canonical services/service/property indentation; exact `privileged: true`; key-token-only evidence; unsupported constructs fail explicit | Compose positive, lookalike, alternate-value, block-scalar, and noncanonical-layout tests | The deliberately limited lexical rule does not validate full Compose semantics |
| SEC-T-047 | Coverage laundering or assessment substitution | Forged inventory fields, altered coverage states, zero findings, mismatched evidence, or stale/spoofed analyzer envelopes attempt to produce a complete or permissive assessment | Recompute inventory and coverage identities; exact snapshot/artifact/finding/evidence/provenance binding; ADR-0013 normalization; closed categorical payload; mandatory unavailable or stale coverage forces partial; all authority and safety fields false | HRA-3 deterministic, schema, forgery, laundering, freshness, mismatch, missing-analysis, and output-bound tests | A separately authorized caller could fabricate normalized objects in memory; the public workflow must preserve the ADR-0013 intake sequence |
| SEC-T-048 | Policy substitution or exception smuggling | Altered policy fields, reordered rules, fabricated findings, conflicting eligibility profiles, or an injected exception attempts to produce a more permissive decision | Recompute exact policy and evidence-record identities; closed deserialization; canonical rule order; fixed restrictive-effect precedence; mandatory-coverage override; no exception input; one quarantine profile; all safety and authority fields false | HRA-4 truth-table, repeatability, schema, identity, monotonic-denial, missing-analysis, conflicting-profile, unknown-field, and authority tests | An external consumer can ignore the reference decision; final risk acceptance remains outside Context |
| SEC-T-049 | Analyzer protocol or partial-result confusion | A malformed, oversized, trailing, identity-mismatched, repository-supplied, faulted, or authority-claiming runner message attempts to promote analysis | Closed schemas; fixed profile digest; exact domain-separated identities; bounded single-message framing; exact artifact accounting; source-free all-or-nothing failures; zero findings in IAR-0 | IAR-0 schema, provenance, framing, identity, mutation, crash, timeout, flood, malformed-output, and authority tests | IAR-0 is an in-memory model and does not establish OS process confinement |
| SEC-T-050 | Synthetic supervisor staging or isolation overclaim | A symlink, stale job, executable substitution, staged-byte mutation, crash, hang, output flood, malformed frame, or misleading confinement flag attempts to escape or overstate IAR-1A | Exact worker digest; fresh private opaque staging; pre/post hashes; cleared environment; private CWD; bounded pipes/deadline; direct-child reap; complete validation/cleanup; closed posture fixed to application-only | IAR-1A pin, collision, symlink, mutation, crash, timeout, flood, malformed-output, cleanup, schema, profile, and provenance tests | Portable controls do not prove network denial, unrelated-handle closure, descendant containment, or OS/VM isolation against a compromised worker; IAR-1B remains pending |
| SEC-T-051 | macOS XPC sandbox or lifecycle overclaim | An entitlement mistake, broad grant, XPC substitution, network path, unrelated-resource access, retained cross-job source, aggregate disk exhaustion, unbounded reply, wrong-PID termination, or incomplete resource/process-tree control is presented as a production sandbox | Closed synthetic request/receipt; ad hoc nested signing; effective entitlement inspection; no network entitlement; external file, credential, synthetic pseudo-terminal device, unrelated-process, and live loopback denials; frozen authority-denying resource profile; source-free Rust-to-host launch handshake; irreversible CPU/address-space/process-count/per-file-size/descriptor limits; exact prepared-service identity before timeout termination; bounded request/reply; crash/relaunch and source-byte cleanup; all admission and authority fields fixed false | Native synthetic hybrid App Sandbox/XPC check plus closed schemas, valid profile/request/preparation/partial fixtures, invalid authority and overclaim fixtures, provenance hashes, Rust contract tests, and platform-interface review; decisive aggregate-disk and cross-job persistence probes | App Sandbox/private XPC has now demonstrated aggregate-disk and cross-job-container isolation failures under this topology. macOS remains at IAR-1A and is not admitted at IAR-1B; signing or packaging cannot repair the runtime gap |
| SEC-T-052 | Linux primitive, cgroup, composition, or support-scope overclaim | Kernel API presence, a partial probe, separate component passes, stale evidence, host drift, or a finite exact-host corpus is presented as complete confinement or broad Linux support | Closed profiles/receipts; `no_new_privs`; read-only staged input; architecture-pinned seccomp; descriptor closure; zero writable path-backed filesystem; one transient delegated subtree; atomic placement; exact lifecycle verification; exact expiring candidate manifest; deterministic withdrawal; production and analyzer admission fixed false | Native synthetic primitive/component/composite checks; x86_64/arm64 plus held-out 6.8/7.0 corpus; exact binding checks; six-state source-free health evaluator; schema overclaim fixtures; frozen provenance | CI sudo is limited to transient-service creation. Candidate evidence is not broad Linux or production support, and no candidate evidence authorizes real analyzer execution |
| SEC-T-053 | Local dashboard disclosure or budget-authority expansion | Malformed audit rows, source-bearing values, ambiguous selectors, stale policy state, crash points, symlink replacement, hostile browser input, or a loopback attacker attempts to expose repository data or raise a governing limit | Closed metadata-only projection; malformed-row withholding; enumerated selectors; canonical identity; deterministic deny precedence; field-wise minimum across every governing layer; expired-policy withdrawal; exact owner marker; private distinct state root; mutation lock; one atomic canonical current/previous state; optimistic identity/revision checks; admission-time reload; isolated std-only loopback listener; verified ephemeral bind; independent one-use fragment and memory-only API-route capabilities; exact Host/Origin/CSRF checks; no cookies or browser storage; bundled CSP-constrained assets; exact preview receipt required for writes; bounded SSE; no outbound socket path | DBC-1 schema, identity, duplicate/unknown-field, source-field, malformed-row, projection, aggregate, deny, expiry, determinism, and field-monotonicity tests; DBC-2 preview/apply/remove/rollback, stale-write, modified/symlink state, live reload, actual operation narrowing, and limited/denied audit tests; DBC-3 strict parser, loopback, bootstrap replay, headers, source-free error, stream, exact preview/write, and shutdown tests; DBC-4 native-browser source-canary, hostile-row/string, loopback-only asset, exact policy lifecycle, screenshot, shutdown, and disposable-cleanup evidence | Local browser extensions and browser/runtime defects remain outside process control. The API-route capability is visible to the local operator's browser process and developer tools for the life of the foreground session |
| SEC-T-054 | Guest vulnerability-review laundering | A stale guest, incomplete provider advisory feed, or metadata-only review is presented as current, vulnerability-free, fully assessed, or production-admitted | Exact manifest/auth/SBOM/provider-snapshot bindings; explicit current-version comparison; incomplete-coverage disclosure; mandatory deny-and-replace disposition; production, analyzer, and authority fields fixed false | Offline static checker; record/profile/receipt schemas; valid fixtures; invalid complete-coverage and production overclaim | Provider data may omit advisories and exact configuration applicability is not established; replacement must undergo a fresh review |

### Future-scope threats

| ID | Trigger | Required new analysis before authorization |
| --- | --- | --- |
| SEC-F-001 | Additional structural parser adapters | Repeat parser memory-safety, malformed syntax, native-library, grammar-provenance, isolation, and false-edge review per grammar |
| SEC-F-002 | Extension host | Publisher trust, signatures/digests, sandbox enforcement, capability grants, update/revocation, quarantine |
| SEC-F-003 | Remote MCP/HTTP transport beyond the accepted local dashboard | Authentication, origin, session confusion, confused deputy, remote exposure, rate limits, protocol injection, identity, tenancy, and retention |
| SEC-F-004 | Model/semantic retrieval | Data egress, prompt injection, retention, provider policy, consent, cost, nondeterminism, poisoning |
| SEC-F-005 | Hosted/multi-tenant mode | Tenant isolation, identity, secrets, encryption, regional/privacy obligations, abuse, availability, incident response |
| SEC-F-006 | Durable memory | Approval, poisoning, retention, deletion, scope bleed, provenance, expiration, correction |
| SEC-F-007 | Source mutation or execution | Command isolation, patch authority, approval, rollback, repository integrity, supply-chain consequences |
| SEC-F-008 | Isolated analyzer runner | Process sandbox, parser/scanner supply chain, signature updates, output normalization, egress, credentials, licensing, retention, and crash containment under ADR-0074 |
| SEC-F-009 | Disposable quarantine runner | VM provider, image provenance, host/guest boundary, networking, credentials, artifact transfer, observation, destruction proof, and Windows deferral under ADR-0075 |

## Security Requirements

### MVP-blocking controls

| ID | Requirement |
| --- | --- |
| SEC-REQ-001 | Implement component-aware canonical root containment and revalidation at read/export time |
| SEC-REQ-002 | Permit regular files only by default and define platform behavior for links and special objects |
| SEC-REQ-003 | Make source workspace access mechanically read-only in design and verify it through before/after state comparison |
| SEC-REQ-004 | Bind cache, snapshots, evidence, and packets to exact workspace identities and versions |
| SEC-REQ-005 | Enforce configurable hard ceilings for discovery, query, memory, time, output, cache, audit, and export |
| SEC-REQ-006 | Use structured data/control separation and terminal-safe output |
| SEC-REQ-007 | Keep network, telemetry, process execution, extension loading, and self-update absent/denied |
| SEC-REQ-008 | Exclude source, raw query content, secret-like values, and environment values from default logs/errors |
| SEC-REQ-009 | Treat cache and packet corruption as failure or rebuild, never as trusted input |
| SEC-REQ-010 | Run adversarial, fuzz, property, mutation, kill/restart, permission, clean-install, and network-denied tests |
| SEC-REQ-011 | Produce an SBOM/dependency inventory and complete license/provenance review for release candidates |
| SEC-REQ-012 | Document local-user, filesystem, backup, and host-compromise residual risks |
| SEC-REQ-013 | Launch structural parsing only through the pinned ADR-0010 worker and validate complete output before graph promotion |
| SEC-REQ-014 | Keep general extension loading and privileged grants absent while enforcing closed digest-pinned declarations and metadata-only quarantine for submitted output |
| SEC-REQ-015 | Keep MCP local to bounded stdio, enforce lifecycle/framing/tool schemas, and grant no roots, network, model, execution, or orchestration authority |
| SEC-REQ-016 | Build release candidates from exact commits with locked inputs, checksums, SBOM, native clean-install rehearsal, and no automatic publication |
| SEC-REQ-017 | Keep hostile-repository admission evidence-only: closed schemas, fixed resource profile, canonical incomplete coverage, no safety claim, and no mutation, process, analyzer, network, upload, deep-parser, or ordinary-host authority |
| SEC-REQ-018 | Keep execution-surface observations to reviewed deterministic rules over safely admitted formats, exact evidence, explicit unsupported syntax, and uninterpreted repository-controlled values |
| SEC-REQ-019 | Keep reference admission evaluation pure and separate, require exact immutable inputs and policy digest, apply monotonic restriction, reject exception smuggling, and never authorize ordinary-host execution |
| SEC-REQ-020 | Require closed bounded analyzer-runner contracts, exact manifest/profile/artifact identities, single-message framing, complete accounting, and all-or-nothing authority-neutral failure before any process-backed runner work |
| SEC-REQ-021 | Report analyzer-supervisor controls and limitations exactly; never infer OS/VM confinement or verified network/descendant denial from application-enforced staging, environment clearing, bounded transport, direct-child lifecycle, and cleanup |

## Verification Strategy

| Method | Coverage |
| --- | --- |
| Unit and property tests | Path containment, identities, budgets, schema, deterministic ordering |
| Filesystem adversarial suite | Traversal, links, races, special files, permissions, case, Unicode, nested repositories |
| Repository injection corpus | Prompt/control spoofing, ANSI/bidi, malicious filenames, false authority claims |
| Fuzzing | Paths, decoders, pattern/query parser, packet parser, evidence spans, cache reader |
| Mutation suite | Freshness, stale evidence, partial snapshots, rapid workspace changes |
| Tamper and fault injection | Cache/packet corruption, disk full, permission loss, cancellation, abrupt restart |
| Isolation suite | Cross-workspace handles, cache roots, exports, concurrent sessions |
| Resource stress | File count/size, long lines, match explosion, memory/disk/time ceilings |
| Network-denied execution | Detect hidden telemetry, update checks, dependency network behavior |
| Source mutation check | Byte/metadata/Git comparison before and after operations |
| Supply-chain review | Locks, SBOM, licenses, advisories, pinned CI/toolchain, artifact provenance |
| HRA-0 contract conformance | Full Draft 2020-12 validation, fixed-profile digest, original-synthetic fixture provenance, false-safety rejection, and ordinary-host-authority rejection |
| HRA-1 inventory conformance | Snapshot/hash/length binding, bounded static classification, explicit exclusions, source immutability, schema-valid output, and symlink non-following |
| HRA-2 npm lifecycle conformance | Closed lifecycle-key corpus, exact key-token evidence, false-positive rejection, unsupported syntax, stale-snapshot failure, value non-retention, and source immutability |
| HRA-2 Compose privilege conformance | Exact basename/layout/value rule, key-token evidence, top-level/nested lookalike rejection, explicit unsupported YAML constructs, and no semantic or runtime claim |
| HRA-3 coverage and assessment conformance | Deterministic capability grouping, exact artifact binding, unavailable mandatory analysis visibility, closed synthetic analyzer envelope, ADR-0013 normalized provenance, stale/mismatch/authority rejection, immutable identities, schema-valid output, and constant no-safety/no-authority fields |
| HRA-4 reference evaluator conformance | Pure separate library, exact immutable-input and policy-digest validation, complete matched reasons, monotonic restriction, missing-analysis denial, exception-input rejection, one quarantine profile, and constant no-safety/no-host-authority fields |
| Independent review | Threat model and release candidate reviewed by someone other than primary implementer when feasible |

## Security Release Gates

The MVP cannot be released publicly unless the following technical gates pass:

1. All `Block` invariants have passing evidence.
2. Every MVP-blocking security requirement maps to implementation and tests.
3. Zero unauthorized content or path-name disclosures occur in the frozen
   adversarial suite.
4. Zero source-workspace mutations occur across the full test suite.
5. Cross-workspace cache, handle, evidence, and export tests pass.
6. Stale, corrupt, incompatible, and partial states fail visibly.
7. Resource ceilings hold under stress without uncontrolled host impact.
8. The network-denied suite observes no required or attempted runtime network
   access.
9. Default logs and errors pass source/secret leakage inspection.
10. Critical/high known dependency vulnerabilities are absent or have a
    documented, time-bounded, approved exception.
11. Installation and uninstallation do not modify global shell, editor, Git, or
    model-provider configuration.
12. Residual risks and unsupported configurations are published.

## Incident And Vulnerability Handling Requirements

Before public release the project must publish:

- a private vulnerability reporting channel;
- supported versions and response expectations;
- a severity and embargo process;
- a method to revoke or replace compromised releases;
- a security-advisory and changelog practice;
- cache/export cleanup guidance when confidentiality is affected;
- instructions that avoid posting sensitive proof-of-concept source publicly.

No automated remote kill switch or silent update is permitted.

## Residual Risk Summary

Even after MVP controls pass:

- an authorized local user can request and export authorized source;
- a compromised host or same-privilege account can usually access the same data
  or alter the executable;
- filesystem races cannot be eliminated equally on every platform;
- automated sensitivity detection can miss secrets or over-redact useful text;
- correctness of lexical relevance does not establish semantic correctness;
- downstream clients may mishandle valid evidence or ignore trust labels;
- dependencies and build systems retain supply-chain risk;
- local cache and exports may be captured by backups or administrator tools.

These risks must be communicated rather than hidden behind a general claim of
“secure” or “sandboxed.”

## Open Security Decisions

1. Supported platform filesystems and exact no-follow/descriptor APIs.
2. Primary memory-safe implementation language and unsafe-code policy.
3. Hash algorithm, domain separation, and canonical serialization.
4. Cache encryption position and local-key assumptions.
5. Cache/export file permission and ACL behavior by platform.
6. Safe pattern-search engine or restricted query syntax.
7. Secret/sensitivity classifier scope and default policy.
8. Audit integrity, retention, and rotation design.
9. Release signing, provenance, and reproducible-build target.
10. Security contact and maintainer response capacity.

## Approval And Change Control

- Approved by/date: Founder, 2026-08-20, as the design baseline. ADR-0017 makes
  independent review an assurance target before `v1.0.0`, or an earlier
  mandatory gate when a qualifying trust-boundary expansion occurs; it is not a
  mandatory `v0.1.0` gate.
- Any addition of network, model access, process execution, native parser code,
  extension loading, hosted deployment, durable memory, source mutation, remote
  transport, or multi-tenant behavior requires a threat-model revision before
  implementation.
