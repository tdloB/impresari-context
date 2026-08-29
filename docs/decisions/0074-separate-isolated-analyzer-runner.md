# ADR-0074: Separate isolated analyzer runner

- Status: Proposed; implementation not authorized
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

## Implementation Gate

This ADR authorizes no scanner installation, process execution, network query,
or code change. Implementation requires accepted step 1 contracts, a threat
model update, exact analyzer/dependency/license decisions, platform confinement
plans, provider-terms review, and explicit founder authorization per phase.

## Review Triggers

Review or supersede before long-lived/shared workers, repository path access,
worker network access, artifact upload/download, repository-supplied rules,
model-based analysis, privileged analyzers, automatic provider fallback,
multi-tenant hosting, or any claim that scanner output proves safety.
