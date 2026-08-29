# Isolated Analyzer Runner — Architecture Requirements Document

## Document Control

- Product: Impresari Analyzer Runner for Impresari Context.
- ARD ID/version: IC-IAR-ARD-001 / 0.1.
- Status: Proposed; architecture and implementation planning only.
- Date: 2026-08-26.
- Sequence: Security expansion step 2 of 3.
- Related records:
  - [Isolated Analyzer Runner PRD](../product/isolated-analyzer-runner-prd.md)
  - [ADR-0074](../decisions/0074-separate-isolated-analyzer-runner.md)
  - [Hostile Repository Admission ARD](hostile-repository-admission-ard.md)
  - [ADR-0010](../decisions/0010-structural-worker-protocol-and-isolation.md)
  - [ADR-0013](../decisions/0013-extension-contracts-without-code-loading.md)

## Architecture Objective

Provide a vendor-neutral, locally controlled execution boundary for static
security analyzers while preserving the Context core's no-code-loading and
no-network guarantees. A compromised analyzer must be contained to a disposable
job area and must be unable to turn its findings into authority.

## Governing Architecture Decisions

### AD-IAR-001 — Separate release and process

The Runner is a separate binary, package, process, SBOM, update channel, and
failure domain. Context never links scanner libraries or starts analyzer code
inside its policy process.

### AD-IAR-002 — Broker artifacts, not repositories

Context authorizes and hashes source. The Runner receives an immutable manifest
and content-addressed artifacts staged into a job-private area. Workers never
receive the original repository or Context cache path.

### AD-IAR-003 — One analyzer, one worker boundary

Analyzers run separately with individually declared capabilities and limits. A
failure or compromise in one cannot access another analyzer's job state or
cause another analyzer to run.

### AD-IAR-004 — Network is a separate gateway

Analyzer workers are always offline. Optional SHA-256 reputation is performed
by a narrow provider gateway that cannot read staged files and cannot upload or
download artifacts.

### AD-IAR-005 — Open-first, vendor-neutral portfolio

The contract is provider-neutral. YARA and ClamAV are reference adapters, not
canonical authorities. MISP and paid services are optional later adapters.

### AD-IAR-006 — No partial promotion

The supervisor validates the complete result before forwarding it. Crash,
timeout, output flood, mutation, identity mismatch, or malformed output discards
the complete analyzer result.

## System Context

```text
Impresari Context
  assessment plan + artifact manifest
              |
              v
Analyzer Runner control plane
  request validation / policy / staging / supervision
      |              |                  |
      v              v                  v
 YARA worker     ClamAV worker     Windows/static workers
 no network      no network        no network
      |              |                  |
      +--------------+------------------+
                     v
          result validator/normalizer
                     |
                     v
              Impresari Context

Separate optional path:
SHA-256 list -> Reputation Gateway -> one allowed provider
                    no artifact access
```

## Deployment Units

| Unit | Contents | May access |
| --- | --- | --- |
| Context core | Existing evidence and assessment engine | Authorized source read, Context cache |
| Runner supervisor | Policy, staging, worker launch, result validation | Job manifest, staged objects, Runner state |
| Analyzer worker | One pinned analyzer and its read-only rules/data | Its job input and bounded output/temp only |
| Rule/database updater | Download, verify, stage, activate, rollback | Exact update endpoints and update store only |
| Reputation gateway | Provider adapter, credential handle, cache | SHA-256 requests, provider endpoint, response cache |
| Admission evaluator | Pure assessment policy | Assessment and policy bytes only |

The updater and reputation gateway never share a process with an analyzer
worker. The Runner supervisor never receives provider credential values.

## Trust Zones

### IAR-Z1 — Context control plane

Higher authority for source authorization and exact evidence. It treats all
Runner output as untrusted derived data.

### IAR-Z2 — Runner supervisor

Trusted only for local admission, staging, process control, and output
validation. It has no source-provider credentials or general network.

### IAR-Z3 — Job staging store

Sensitive, ephemeral, and scoped to one request/snapshot/analyzer. Contains
only selected artifact bytes and safe opaque identities.

### IAR-Z4 — Analyzer workers

Hostile and potentially compromised. They have no ambient authority and can
write only bounded job-private output/temp data.

### IAR-Z5 — Update channel and store

Privileged supply-chain boundary. Updates require exact origins, integrity,
provenance, license, compatibility, anti-rollback, and staged activation.

### IAR-Z6 — Reputation provider

External and untrusted for availability, result quality, retention, and output.
It observes queried hashes and the gateway's service identity/network metadata.

## Request Protocol

The Runner request is a single closed, length-bounded message containing:

- schema/protocol versions and unique request ID;
- Context engine, workspace, snapshot, assessment-plan, and policy identities;
- requested analyzer capability IDs;
- analyzer/ruleset/database pins;
- content-addressed artifact descriptors and exact sizes;
- platform target classifications;
- hard resource and output budgets;
- cancellation deadline and operation time;
- an opaque return correlation ID.

It contains no absolute repository path, source-provider credential, external
provider credential, shell command, repository-supplied analyzer path, or
network destination.

## Artifact Transfer And Staging

Preferred transfer modes are selected per platform and artifact size:

1. Bounded framed bytes for small artifacts.
2. Supervisor-created content-addressed objects in a private transfer store.
3. Read-only inherited handles where platform semantics and conformance evidence
   are stronger than path-based transfer.

The supervisor:

- creates a unique owner-only job directory outside repository/cache roots;
- writes artifacts under opaque names, not repository-controlled paths;
- verifies exact size and SHA-256 before and after each worker;
- prevents links, devices, sockets, mount points, and alternate path aliases;
- gives each analyzer a separate view;
- never reconstructs untrusted nested directory layouts unless an analyzer
  contract explicitly needs a bounded logical map;
- deletes staged bytes after the retention gate, documenting secure-deletion
  limitations.

## Analyzer Manifest V2 Requirements

ADR-0013 remains the Context-side non-executing contract. The Runner adds a
separate execution manifest that includes:

- stable ID, version, publisher statement, artifact digest, signature or
  provenance evidence, and revocation state;
- executable identity and exact argv template controlled by the Runner;
- supported host platforms, target formats, and capability IDs;
- accepted input media, maximum input count/bytes, and output schema;
- filesystem, temp, process, environment, library, and OS-confinement needs;
- explicit network requirement, which must be `none` for scanner workers;
- ruleset/database compatibility and freshness requirements;
- determinism class and expected nondeterministic fields;
- license, SBOM, advisory, update, rollback, and support status;
- worker-specific threat model and conformance fixture IDs.

An inspected repository cannot supply or alter a manifest, analyzer executable,
argv template, ruleset, database, or limit.

## Worker Launch And Confinement

The launcher must:

- resolve an absolute, policy-pinned executable and verify its digest;
- start without a shell;
- clear the environment and supply only declared non-secret values;
- use a job-private empty home, current, temp, input, and output layout;
- close unrelated file descriptors/handles and prevent inheritance;
- run as a non-administrator identity with no elevation capability;
- enforce process-tree, CPU, memory, wall-time, disk, file-count, descriptor,
  recursion, input, and output limits;
- deny network, IPC outside the job, host devices, debugger/ptrace access, and
  unrelated process visibility where the platform supports enforcement;
- kill the entire tree and invalidate output on any violation.

Application controls are the portable baseline. Platform reports distinguish
`application_enforced`, `os_confined`, and `vm_confined`; no report may imply
equivalent sandbox strength without evidence.

## Result Protocol And Normalization

A worker returns exactly one bounded result envelope with:

- request, analyzer, executable, ruleset/database, and capability identities;
- input artifact hashes and per-artifact status;
- findings with typed fields and opaque evidence offsets where applicable;
- scanned/skipped/failed counters and limit events;
- tool exit/status semantics normalized by the adapter;
- elapsed/resource observations;
- explicit completeness and limitations;
- no raw source, absolute paths, environment, stack trace, or control command.

The supervisor validates the complete envelope, stable ordering, duplicates,
limits, identities, and artifact accounting. Context then applies its own
independent normalization and exact-evidence binding. Neither layer trusts a
scanner's statement that content is exact, safe, clean, or authorized.

## Reference Adapter Architecture

### YARA

- Compile only a locally approved digest-pinned rule bundle before a job.
- Disable repository-supplied includes and rules.
- Admit modules explicitly; PE and other native modules receive their own
  dependency and malformed-input review.
- Set scan timeout, stack, match, string, module, and output limits.
- Record rule namespace, rule ID, tags, metadata, and ruleset digest as
  untrusted derived data.

### ClamAV

- Prefer a short-lived standalone scan process for initial isolation and
  reproducibility rather than a shared daemon.
- Load a pinned database set from an immutable approved directory.
- Record engine/database versions and freshness.
- Set archive, recursion, file, scan-size, PCRE, temp, and output limits.
- Treat `FOUND`, clean/no-detection, error, skipped, and limit states distinctly.
- Keep database updating outside the job and never enable sample submission.

### First-party analyzers

First-party status does not bypass the worker boundary. Every Windows, binary,
dependency, and execution-surface analyzer uses the same manifest, staging,
limit, result, and revocation contracts.

## Windows Static Analyzer Architecture

### PowerShell and batch

Parsing occurs without invocation or expression evaluation. The worker emits
syntax/token/AST facts and bounded capability signals. Dynamic strings,
reflection, encoded content, environment dependence, and parse recovery become
explicit uncertainty.

### PE and DLL

Workers parse bounded PE/COFF structures and expose selected headers, sections,
imports/exports, resources, overlay, debug, CLR, and signing metadata. They do
not load the image, resolve DLLs, run entrypoints, register components, or trust
declared timestamps and version strings.

### Authenticode

The adapter separates:

- signature container presence;
- cryptographic verification result under a named trust store/time policy;
- signer and timestamp metadata;
- reputation evidence, if separately queried;
- the invariant that a valid signature is not a benignness verdict.

### MSI

The worker treats MSI as hostile structured data. It inspects bounded tables,
streams, files, sequences, conditions, and custom actions without invoking
Windows Installer or extracted payloads. Embedded binaries/scripts are hashed
and routed as new artifacts subject to expansion limits.

### Persistence and build surfaces

Static rules correlate service, scheduled-task, registry, startup, WMI,
MSBuild, NuGet, packaging, and signing declarations with exact evidence. They
report capabilities and ambiguity rather than inferred attacker intent.

## Rule And Database Update Architecture

The updater is disabled during active scans and follows:

```text
fetch from exact allowlisted origin
  -> verify TLS/origin
  -> verify signature/digest/provenance/license
  -> scan/parse in update staging
  -> compatibility and rollback checks
  -> conformance smoke
  -> atomic activation by immutable ID
```

The active set never changes during a job. Failed updates retain the last known
non-revoked compatible set. A minimum-freshness policy may stop new scans while
still permitting status, evidence inspection, rollback, and disablement.

## Privacy B Reputation Gateway

The gateway API accepts only:

- provider adapter ID/version;
- SHA-256 list within a fixed maximum;
- purpose and local policy decision ID;
- timeout and response-size bounds.

Before egress it rejects every request containing additional artifact metadata.
It uses exact provider endpoints, TLS verification, redirect denial or strict
same-origin handling, proxy controls, DNS/IP/private-network protections, rate
limits, and a provider-specific credential handle.

Normalized response fields include provider, query time, dataset/version when
available, match state, provider classification, confidence limitations,
fresh-until, response digest, and retention/terms policy version. Raw responses
remain local and bounded or are discarded.

The gateway does not expose submission, download, commenting, search-by-name,
or general provider-query operations.

## Storage Architecture

Separate stores are required for:

- immutable approved analyzer artifacts;
- immutable approved ruleset/database versions;
- ephemeral job staging and bounded raw outputs;
- normalized result cache keyed by snapshot/artifact/analyzer/ruleset/policy;
- reputation cache keyed by provider/hash/dataset/policy;
- metadata-first audit and revocation records.

No store is located inside a source workspace. Provider credentials use the OS
secure credential facility where available and never enter result or audit
records.

## Failure Semantics

| Event | Required behavior |
| --- | --- |
| Worker crash/timeout/kill | Discard result; mark failed |
| Input mutation | Kill worker; discard; security event |
| Analyzer output mismatch | Quarantine metadata only |
| Stale/revoked analyzer or rules | Do not start; unavailable/stale |
| Update failure | Retain last non-revoked version or stop new scans |
| Reputation provider failure | Unknown; no provider fallback unless separately enabled |
| Reputation cache corruption | Delete/rebuild; never accept |
| Cleanup failure | Quarantine job area; block reuse; surface operator action |

## Threat Register

| Threat | Control |
| --- | --- |
| Scanner parser compromise | Separate worker, minimal staged input, OS confinement, no network/secrets |
| Malicious rule bundle | Review/pin/limit/rollback and separate update boundary |
| Archive/decompression bomb | Preflight and scanner-specific expansion ceilings |
| Analyzer fork/orphan | Process group/job object/cgroup and kill-tree verification |
| Repository analyzer substitution | No repository discovery; exact policy pins |
| Result spoofing | Closed protocol, identities, hashes, complete validation |
| Hash privacy disclosure | Explicit provider opt-in and SHA-256-only gateway |
| Provider credential theft | Gateway-only secure handle; no worker/Context access |
| Supply-chain compromise | SBOM, provenance, signatures/digests, revocation, staged update |
| Cross-job contamination | Per-job stores and sequential contamination tests |

## Observability And Audit

Record source-free metadata:

- job, snapshot, analyzer, ruleset/database, policy, and result IDs;
- start/end, status, resource totals, file counts, skips, limits, and failure
  classes;
- confinement level and platform evidence version;
- updater activation/rejection/revocation;
- provider adapter, number of hashes, cache hits, response class, and latency,
  but not the hashes themselves by default;
- cleanup completion.

Audit cannot authorize a scan or stage transition.

## Conformance And Evaluation

### Protocol

- Closed request/result schemas, framing, identity, duplicate, version, trailing,
  malformed, and output-limit cases.

### Isolation

- Filesystem escape, handle inheritance, home/credential canaries, socket/DNS,
  process visibility, child/orphan, privilege, debugger, temp, mutation, and
  cross-job tests on every supported host.

### Analyzer

- Known benign/malicious test standards where licensing permits, original
  synthetic rules, malformed and resource-hostile files, disagreement, false
  positives, false negatives, skip accounting, and database/rule freshness.

### Windows

- Cross-host fixtures for PowerShell, batch, PE/DLL, Authenticode, MSI,
  persistence, MSBuild, NuGet, and ambiguous/polyglot artifacts.

### Privacy

- Packet-capture proof of SHA-256-only egress; no upload/download endpoints;
  redirect/proxy/DNS/private-network denial; credential/log canaries; provider
  timeout and rate-limit behavior.

## Implementation Sequence And Release Gates

Follow IAR-0 through IAR-6 in the PRD. No phase may combine the updater,
reputation gateway, supervisor, and scanner into one process merely for
convenience. Each analyzer capability requires its own admission evidence and
may be released or revoked independently.

Before implementation:

1. Step 1 contracts and ADR-0073 must be accepted.
2. ADR-0074 and the expanded threat model must be accepted.
3. Exact host confinement evidence requirements must be approved.
4. Analyzer/ruleset license and distribution feasibility must be confirmed.
5. Provider terms and privacy disclosures must be current and approved.
6. Founder authorization must name the exact implementation phase.

## Rollback And Incident Isolation

- Global Runner kill switch stops new jobs and terminates active jobs.
- Per-analyzer/ruleset/provider revocation is supported.
- A compromised adapter cannot enable a replacement provider automatically.
- Existing Context remains usable without the Runner.
- Old normalized results retain provenance but become revoked/stale as policy
  requires.
- Incident response can preserve a bounded job image only under explicit local
  authorization; default cleanup continues otherwise.

## External Reference Baseline

These sources inform planning; exact versions, licenses, terms, and behavior
must be reverified before implementation or activation:

- [ClamAV usage and scanner/update separation](https://docs.clamav.net/manual/Usage)
- [ClamAV signature formats and signed databases](https://docs.clamav.net/manual/Signatures.html)
- [YARA documentation and rule/module model](https://yara.readthedocs.io/en/latest/)
- [Microsoft PE/COFF format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
- [Microsoft Windows Installer custom actions](https://learn.microsoft.com/en-us/windows/win32/msi/about-custom-actions)
- [CIRCL Hashlookup](https://www.circl.lu/services/hashlookup/)
- [MalwareBazaar community API](https://bazaar.abuse.ch/api/)
- [MISP](https://www.misp-project.org/)
- [VirusTotal API overview](https://docs.virustotal.com/docs/api-overview)

## Architecture Exit Criteria

Step 2 is ready for a real-untrusted-repository pilot only after all reference
adapters, Windows capabilities, updater, Privacy B gateway, Tier A confinement,
cleanup, disagreement, and failure-mode gates pass and an independent security
review and founder activation are recorded. This ARD does not authorize that
pilot or any implementation.
