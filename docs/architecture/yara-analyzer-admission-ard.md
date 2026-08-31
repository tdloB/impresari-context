# YARA Analyzer Admission ARD

- Status: Legacy adapter/source contracts superseded by ADR-0097; YARA-X replacement contracts pending; execution gated on IAR-1B
- Date: 2026-08-30
- Governing PRD: [YARA Analyzer Admission PRD](../product/yara-analyzer-admission-prd.md)
- Decision: [ADR-0089](../decisions/0089-yara-first-real-analyzer-admission.md)

## Architecture

```text
HRA analyzer plan + exact artifact manifest
                    |
                    v
production-admitted IAR-1B worker
  pinned YARA + pinned project rules + read-only staged artifacts
                    |
                    v
bounded vendor result -> closed adapter -> ADR-0013 normalization
                    |
                    v
immutable assessment + explicit coverage/limitations
```

Context never parses raw YARA output or invokes YARA. The Runner validates the
complete vendor output against a narrow adapter schema, then Context treats the
adapter envelope as untrusted derived data and normalizes it independently.

## Executable And Ruleset Supply Chain

- Pin source repository, source revision, build environment, target, compiler,
  dependencies, license, artifact digest, SBOM, and provenance.
- Build with only required modules; reject unapproved dynamic libraries and
  repository-provided modules.
- Compile project rules in a separate no-source release job.
- Root metadata defines current and previous admitted rulesets, expiry, and
  rollback prevention. No worker possesses signing or update credentials.

## Request And Result Boundary

- Request names only exact content IDs from the HRA plan, analyzer/profile
  identity, ruleset identity, and fixed budgets.
- Result accounts for every requested content ID and contains only bounded
  normalized rule identifiers, namespaces, tags, strings/offsets where
  permitted, and diagnostic reason codes.
- Raw stdout/stderr and unmatched file bytes are never retained by Context.

## Verification

- Unit fixtures cover parsing, ordering, duplicate matches, Unicode, offsets,
  excessive strings, malformed output, unknown rule, and incomplete coverage.
- Fault workers cover substitution, crash, timeout, fork, memory, output, and
  ruleset mismatch under every claimed platform backend.
- Release rehearsal proves clean install, update, rollback rejection, expiry,
  removal, and no network or source leakage.

## ADR-0095 Contract Boundary

The first adapter contract is deliberately synthetic-only. Its input mirrors
the minimum future result shape but admits only original-synthetic fixture
records. A deterministic offline checker verifies complete artifact accounting,
canonical rule observations, bounded byte ranges, exact digest identities, and
constant non-authority claims. It neither parses raw YARA output nor adds a
process, analyzer, ruleset, source, network, credential, or platform admission
path. Live results require a new reviewed contract and the ADR-0089 activation
gate.

## ADR-0096 Supply-Chain Boundary

The upstream source, executable build, and ruleset are three independent
identities. Version 1 selects only the official YARA v4.5.8 tag commit and
license-file metadata. Because that release has no uploaded GitHub assets, the
profile rejects any upstream binary and requires a future Impresari-owned,
per-target reproducible build with an exact archive digest, dependency closure,
SBOM, provenance, signature, vulnerability/license review, expiry, and
revocation record.

The production ruleset remains absent. Its later admission requires a separate
project-owned source and compiled-artifact identity, human review, license,
signature, expiry, and rollback record. Repository rules, includes, external
paths, custom modules, in-job updates, network retrieval, and worker-held update
credentials are structurally false. The offline checker verifies metadata and
fail-closed states only; it cannot download, build, sign, load, or execute an
artifact.

## ADR-0097 YARA-X Direction

YARA-X is selected, but no engine-specific build architecture is frozen yet.
The replacement requires new engine/profile identities, documented
rule-compatibility constraints, and a bounded JSON/NDJSON adapter. It may not
reuse legacy YARA's executable, compiled-rules, module, or result-parser
identity. The common artifact, ruleset, confinement, accounting, expiry,
revocation, and non-safety requirements remain unchanged.
