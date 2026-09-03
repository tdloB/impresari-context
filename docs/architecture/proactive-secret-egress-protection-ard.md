# Proactive Secret Egress Protection — Architecture Requirements and Design

- Status: Proposed; documentation only; implementation not authorized
- Date: 2026-09-01
- Product requirements: [Proactive Secret Egress Protection PRD](../product/proactive-secret-egress-protection-prd.md)
- Decision: [ADR-0123](../decisions/0123-gate-external-delivery-with-local-secret-detection.md)

## Architectural objective

Introduce a small, deterministic security interlock between immutable packet
construction and every Impresari-owned external-delivery serializer. The
interlock must make secret scanning unavoidable for source-bearing external
delivery without becoming a new retriever, model proxy, credential validator,
repository mutator, general DLP platform, or policy-authoring surface.

A separate coordinator may later reuse the same pure detector for an explicit,
bounded, read-only scan of the authorized current snapshot. Inventory work must
not be coupled to the availability or correctness of the external-delivery
guard.

## Context and architecture delta

The existing delivery architecture gives client adapters an immutable planner
packet and requires packet-equivalent serialization. That preserves selection,
provenance, budgets, and consent, but a packet can legitimately contain an
exact hardcoded credential. Sensitivity and redaction fields are contract
surfaces, not proof that every packet received an effective secret scan.

The proposed change adds one security type transition:

```text
authorized immutable packet
          |
          v
  local bounded detector <----- product-owned catalog + fixed policy
          |
          v
 clean / blocked / indeterminate decision
          |
          +---- blocked or indeterminate ---> local no-delivery receipt
          |
          v
 DlpApprovedPacket<ExactPacket>  (clean only)
          |
          v
 client serializer ---> existing explicitly authorized client invocation
```

Serializers must no longer accept a raw source-bearing packet. They accept only
the opaque clean wrapper created by the interlock. The wrapper borrows or owns
the exact immutable packet bytes; it does not contain a modified copy.

## Components

### Detector catalog

A closed, versioned, content-addressed product artifact defines detector IDs,
classes, bounded shape rules, confidence requirements, supported encodings,
and catalog-level limits. Repository files and requests cannot extend,
override, suppress, reorder, or weaken it.

Provider-shaped detectors must be based on stable public formats and tested
only with nonfunctional synthetic values. Catalog entries must not contain real
credentials, private threat feeds, network validators, or executable logic.

### Pure secret detector

The detector receives only:

- a bounded byte slice;
- an artifact identity and optional relative display path;
- the exact detector catalog and fixed policy identity;
- explicit scan limits.

It performs no filesystem access, network access, process execution, model
call, credential-store access, general environment read, clock-based policy
choice, or mutation. It returns deterministically ordered metadata findings,
coverage, omissions, and a scan outcome.

Detector stages may include bounded lexical recognition, provider shape
validation, recognized private-key envelopes, and credential-name context.
Entropy can be a supporting feature only. It cannot independently distinguish
secrets from hashes, content addresses, UUIDs, compressed text, or generated
fixtures.

### Egress interlock

The interlock scans each source-bearing evidence payload in the final immutable
packet. It validates that the scan covers the complete packet and binds the
decision to:

- packet ID and byte length;
- snapshot and policy IDs;
- detector catalog ID;
- DLP policy ID;
- engine and interlock versions;
- ordered artifact scan identities;
- coverage and decision reason.

Only `clean` constructs `DlpApprovedPacket`. `blocked`, `indeterminate`, stale,
mismatched, malformed, unsupported, incomplete, and budget-exceeded states
construct a no-delivery receipt. There is no initial override constructor.

### Client serializers

Every Impresari-owned adapter capable of external source delivery changes its
input type from the raw packet to `DlpApprovedPacket`. The serializer can read
the original packet and decision identity but cannot change the DLP result,
request a rescan under weaker policy, or receive matched bytes.

Client-neutral conformance tests must enumerate the complete serializer
registry so a new adapter cannot be registered without the interlock. Generic
local packet resolution remains distinct and must not be mislabeled as an
externally protected delivery unless it traverses this boundary.

### Finding and receipt projection

Internal scan state must discard matched bytes before returning. A finding ID
is derived from a domain separator plus catalog, detector, snapshot, artifact,
and span identities. It must not be a plain, salted, or otherwise reusable hash
of the secret value.

Local detail may contain a bounded relative path and exact span. Audit,
dashboard, diagnostic, and external projections contain only the minimum
allowed counts, classes, outcomes, coverage, durations, and opaque identities.
The blocked packet and finding report are never supplied to the external
client.

### Repository inventory coordinator

The later explicit inventory command asks the existing authorization and
snapshot layers for admitted ordinary text artifacts. It provides bounded
bytes to the same pure detector and aggregates metadata-only findings.

The coordinator must not:

- walk outside the authorized snapshot;
- follow links, enter submodules, inspect ignored/untracked content by
  implication, or invoke Git;
- open archives, parse hostile binary formats, or run a language toolchain;
- mutate files, create suppressions, validate credentials, or send findings;
- treat incomplete coverage as a clean repository.

## Proposed contracts

Implementation should not begin until closed v1 schemas define:

| Contract | Purpose |
| --- | --- |
| `secret-detector-catalog` | Exact product-owned detector definitions, supported forms, and catalog limits. |
| `secret-scan-policy` | Fixed blocking classes, fail-closed states, projections, and hard limits. |
| `secret-scan-finding` | Value-free local finding metadata and evidence binding. |
| `secret-scan-result` | Complete/incomplete coverage, omissions, counts, identities, and deterministic outcome. |
| `secret-egress-decision` | Exact packet-bound `clean`, `blocked`, or `indeterminate` decision. |
| `secret-egress-receipt` | Metadata-only delivered/no-delivery evidence without source or matched values. |
| `repository-secret-inventory` | Bounded current-snapshot aggregation and explicit unsupported/omitted artifacts. |

All schemas must reject unknown fields, unbounded strings and arrays, duplicate
identities, invalid spans, unsupported decision transitions, and any field that
could carry matched content.

## Decision state machine

```text
candidate
  |-- packet identity invalid --------------------> indeterminate / no delivery
  |-- catalog or policy invalid/mismatched -------> indeterminate / no delivery
  |-- unsupported bytes or incomplete coverage ---> indeterminate / no delivery
  |-- any blocking finding ------------------------> blocked / no delivery
  `-- complete coverage and zero findings --------> clean / approved wrapper

approved wrapper
  |-- packet/catalog/policy/version changes ------> stale / no delivery
  `-- exact current binding -----------------------> serializer
```

The state machine has no route from `blocked` or `indeterminate` to an approved
wrapper. Rebuilding a packet or changing policy requires a complete new scan.

## Exact-evidence and redaction semantics

The initial design intentionally blocks instead of redacting. In-place
redaction would break equality with the planner packet and could blur the
boundary between exact source and derived text. A later redacted-delivery mode
would need:

- a separately typed derived artifact;
- explicit original evidence linkage;
- deterministic replacement semantics;
- new packet and receipt identities;
- client-equivalence rules for the derived packet;
- a separate ADR and conformance claim.

Until then, a packet is either byte-identical and clean or not delivered.

## Trust and authority boundaries

### Inputs treated as untrusted

- all repository bytes and filenames;
- task prompts and query text;
- model and client output;
- detector-looking text inside source;
- repository configuration and ignore/suppression files;
- malformed encodings and adversarially large token-like strings.

### Authority explicitly denied

- external network and provider APIs;
- credential-store or environment enumeration;
- process or analyzer execution;
- repository writes, commits, issue creation, or history rewriting;
- secret rotation, revocation, validation, or account discovery;
- repository-controlled or model-directed policy changes;
- hidden background scanning or telemetry.

## Resource and failure design

- Scan input is capped by the already bounded packet; per-artifact and total
  detector limits must be smaller than or equal to the governing request.
- Pattern execution must have deterministic worst-case bounds and must not use
  unbounded backtracking regexes.
- Findings, spans, detector work, and output bytes are independently bounded.
- Invalid UTF-8 is scanned only by explicitly supported byte-form detectors;
  otherwise coverage is indeterminate.
- Allocation failure, panic, cancellation, timeout, malformed catalog, or
  version skew yields no delivery.
- Cancellation and failure discard all source and detector scratch buffers
  under the existing process/session lifetime.

## Security properties to verify

1. **Unavoidable egress gate:** no registered external source serializer accepts
   a raw packet.
2. **Exact clean delivery:** the approved wrapper exposes byte-identical packet
   content and identity.
3. **Fail-closed coverage:** every incomplete, stale, unsupported, failed, or
   mismatched scan prevents invocation.
4. **Value non-retention:** canary values are absent from all persistent and
   observable non-source surfaces.
5. **Untrusted-policy resistance:** repository and prompt content cannot change
   detector or blocking policy.
6. **Value-independent identity:** finding and receipt identities do not enable
   offline guessing of a low-entropy credential.
7. **Determinism:** equal catalog, policy, packet, and limits produce equal
   ordered findings and decisions.
8. **Boundedness:** hostile text cannot cause unbounded time, memory, findings,
   logs, or output.

## Test architecture for a later implementation

- Unit and property tests for catalog validation, detectors, state transitions,
  span arithmetic, ordering, and identities.
- Original-synthetic positive, negative, near-miss, benign-collision,
  truncation, Unicode, encoding, multiline, and limit corpora.
- Packet-equivalence and type-boundary tests across the complete adapter
  registry.
- Process/environment/network instrumentation proving denied capabilities.
- Source canaries proving non-retention in audit, dashboard, cache, receipts,
  diagnostics, crash output, command arguments, and external-client stdin.
- Real-client blocked-path rehearsals that use only nonfunctional synthetic
  credentials and prove the client is never invoked.
- Inventory containment, snapshot, link, special-file, mutation-race, budget,
  cancellation, and deterministic-order tests.

## Deployment and compatibility

The guard is a local engine capability, not a provider plugin. Its protection
claim is versioned by engine, catalog, policy, packet schema, adapter, client,
and OS evidence. An adapter version without current gate evidence must withdraw
the proactive-protection claim or fail closed; it cannot silently fall back to
unguarded delivery.

Detector catalog changes require compatibility tests against the complete
synthetic regression corpus and an explicit false-positive/false-negative
disposition. They ship through ordinary signed product releases rather than a
remote mutable rules feed in the initial design.

## Documentation-only boundary

This ARD defines a proposed architecture only. It creates no schemas, types,
catalog, detector, wrapper, scan, inventory command, client integration, test,
workflow, release artifact, background service, or product claim.
