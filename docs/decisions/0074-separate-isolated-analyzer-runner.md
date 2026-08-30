# ADR-0074: Separate isolated analyzer runner

- Status: Accepted; IAR-0 protocol and IAR-1A application-enforced synthetic
  supervision implemented; the macOS hybrid IAR-1B candidate is selected for
  continued feasibility but remains partial and unadmitted
- Date: 2026-08-26
- Scope: Static analyzer execution, scanner adapters, Windows hostile-format
  analysis, rule/database updates, and hash-only reputation

## Context

ADR-0013 intentionally introduced extension declarations and hostile-output
normalization without loading code. Hostile-repository admission now needs
ClamAV, YARA, binary and installer parsers, dependency analysis, and
Windows-oriented static analysis. These tools consume attacker-controlled
bytes, often include native code, and can fail or be compromised.

Loading them into the Context process would give scanner vulnerabilities the
same workspace and cache authority as the evidence core. Allowing each scanner
to contact the network would also mix source access, credentials, egress,
provider terms, and result authority.

## Decision

1. Implement static analyzer execution in a separately packaged and released
   Impresari Analyzer Runner, never inside the Context core process.
2. Context authorizes source and supplies a snapshot-bound artifact manifest;
   the Runner stages only selected exact bytes into per-job private storage.
3. Each analyzer runs in its own short-lived worker with a pinned executable,
   ruleset/database, closed request/result contract, cleared environment, no
   original repository path, no credentials, no network, and hard resource
   limits.
4. Complete output is validated all-or-nothing by the Runner and independently
   normalized by Context as untrusted derived data.
5. The analyzer contract is vendor-neutral. YARA and ClamAV are initial
   reference adapters, not mandatory authorities.
6. First-party execution-surface, dependency, binary, and Windows analyzers use
   the same isolation and governance boundary as third-party tools.
7. Windows PowerShell, batch, PE/DLL, Authenticode, MSI, persistence, and build
   analysis is initial static scope on macOS and Linux; artifacts are never
   loaded, installed, registered, or executed.
8. Analyzer/ruleset/database updating is a separate verified process that
   cannot run during a scan and cannot silently roll back or self-activate.
9. Optional external reputation follows Privacy B through a separate gateway
   that accepts only SHA-256, holds provider credentials, and exposes no file
   upload, sample download, provider mutation, or general search operation.
10. Local scanning works without MISP or a paid provider. MISP and commercial
    services remain optional provider adapters.

## Consequences

### Positive

- Scanner compromise is separated from source authorization and exact evidence.
- Open and commercial analyzers can coexist behind one result vocabulary.
- Hash reputation can be disabled without losing local analysis.
- Windows repositories receive useful static coverage before Windows dynamic
  execution exists.
- Analyzer failures become explicit coverage gaps rather than permissive
  fallbacks.

### Costs

- Separate packaging, staging, confinement, update, compatibility, and cleanup
  systems increase engineering and release work.
- Scanner semantics differ and require careful normalization.
- Cross-platform confinement strength will not be identical and must be
  reported honestly.
- Open community provider terms and availability may be insufficient for later
  commercial scale.
- Static analysis still cannot prove runtime behavior or absence of malware.

## Alternatives Considered

### Link scanner libraries into Context

Rejected because it enlarges the core trusted computing base and shares its
workspace/cache authority with hostile parsers.

### Run all analyzers in one shared daemon

Rejected initially because one compromise or contaminated state could cross
analyzer, repository, and job boundaries.

### Choose one proprietary scanner

Rejected as the architecture because it creates lock-in and makes one vendor's
coverage and terms part of the product contract. Commercial adapters remain
possible.

### Build an Impresari antivirus engine

Rejected because signature research, unpackers, threat intelligence, and
malware-family maintenance would create an unbounded security product burden.
Impresari-owned repository-specific rules remain appropriate.

### Give scanner workers direct threat-intelligence access

Rejected because it mixes hostile parsing with credentials and egress and makes
privacy enforcement difficult to verify.

### Require MISP initially

Rejected because it adds server, database, feed-curation, patching, backup, and
administration burden for users who only need local scanning and bounded hash
lookups.

## Verification

- Context works unchanged when the Runner is absent.
- Workers cannot read the source repository, Context cache, home directory,
  credentials, other jobs, or network in the supported confinement matrix.
- Input mutation, crash, timeout, excessive output, and malformed results cause
  complete-result rejection.
- Every result records analyzer and ruleset/database identity and accounts for
  every requested artifact.
- Windows static fixtures pass on macOS and Linux without artifact execution.
- Packet capture proves that reputation requests contain SHA-256 only and use no
  upload/download/mutation endpoints.
- Updater rollback, substitution, expiry, and activation failures fail closed.

## Implementation Sequence

The standing roadmap directive authorizes accepted increments without a
repeated approval ceremony. IAR-0 freezes closed contracts, the fixed resource
profile, provenance-reviewed synthetic fixtures, exact framing, and an
in-memory no-op/fault worker. It adds no scanner installation, subprocess,
filesystem staging, network query, upload, credential access, parser, or
repository execution.

IAR-1A adds private content-addressed staging,
exact executable pinning, a short-lived synthetic subprocess, bounded transport
and wall time, input rehashing, complete-result validation, cleanup, and an
explicit measured posture. It claims only `application_enforced`: OS/VM
confinement, verified network denial, and descendant containment remain false
or unverified, so the IAR-1B OS-confinement checkpoint remains open. Real analyzer
admission, updater/network capability, provider access, and quarantine remain
separate later gates.

IAR-1B begins with independently admitted platform backends. The first macOS
App Sandbox/private-XPC feasibility prototype is synthetic-only and partial: it
demonstrates exact sandbox identity, bounded IPC, and selected native denials,
but does not yet establish hard resource/process-tree controls, rehearsed fault
timeout, complete OS-managed container cleanup, production signing/notarization,
packaging, or multi-host compatibility. It does not change the decision status,
admit macOS, authorize a real analyzer, or open IAR-2. The hybrid follow-up
corrects the assumption that XPC must supply every layer alone. It combines
App Sandbox/private XPC access confinement, irreversible in-service CPU,
address-space-growth, and process-count limits, and Rust-supervisor exact-target
wall-time termination and cleanup. Those native synthetic probes pass locally,
as does denial of an exact synthetic pseudo-terminal device. The frozen
`iar-macos-xpc-hybrid-v1` production-candidate profile is effective in the
service, and the closed source-free Rust-to-host preparation handshake rejects
paths, arguments, environment, credentials, network authority, analyzer
execution, mismatched identity, partial readiness, and premature confinement
claims. The candidate is selected for continued feasibility. Developer ID and
notarization, ADR-0076 cask lifecycle, clean-machine Gatekeeper, the full Tier A
corpus, and multi-host evidence remain mandatory. Privileged launch daemons,
private APIs, persistent services, and VMs remain outside this decision.

The subsequent Tier A checkpoint materially fails the selected macOS topology:
the service can exceed a job-wide temporary-disk ceiling through multiple
individually legal files, and a fresh service process can read a synthetic
marker retained by the preceding job in the shared service container. Signing,
notarization, and cask packaging cannot correct these runtime properties.
macOS therefore remains at IAR-1A; the XPC design is retained as
defense-in-depth and a future reconsideration point, while IAR-1B feasibility
advances independently on Linux.

## Review Triggers

Review or supersede before long-lived/shared workers, repository path access,
worker network access, artifact upload/download, repository-supplied rules,
model-based analysis, privileged analyzers, automatic provider fallback,
multi-tenant hosting, or any claim that scanner output proves safety.
