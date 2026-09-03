# Impresari Context — Proactive Secret Egress Protection PRD

- Status: Proposed; documentation only; implementation not authorized
- Date: 2026-09-01
- Owner: Aaron Boldt
- Related roadmap: [Revised Product Roadmap](revised-product-roadmap.md)
- Architecture: [Proactive Secret Egress Protection ARD](../architecture/proactive-secret-egress-protection-ard.md)
- Decision: [ADR-0123](../decisions/0123-gate-external-delivery-with-local-secret-detection.md)

## Product statement

Impresari Context should prevent likely hardcoded repository secrets from
silently entering context sent to an external model or client service. The
first protection is automatic at the outbound delivery boundary. A later,
explicit inventory mode should help an operator find likely secrets in the
authorized current repository snapshot.

This is deliberately narrower than general data loss prevention (DLP). It does
not promise to identify every secret, personal datum, confidential business
record, regulated datum, or policy violation.

## Problem

The current engine minimizes logs, preserves sensitivity and redaction fields,
and bounds external client delivery, but it is not a secret scanner or DLP
system. An authorized exact-source request can therefore retrieve a hardcoded
credential. If that packet is delivered to an external LLM client, ordinary
authorization and packet bounds do not by themselves prevent credential
disclosure.

Novice users may not know that they should run a separate security scan. The
highest-risk protection must therefore be part of the delivery path rather
than an optional instruction that users need to remember.

## Goals

1. Automatically inspect every source-bearing packet immediately before an
   Impresari-owned adapter delivers it to an external service.
2. Fail closed when a blocking secret match or an indeterminate scan prevents
   a clean decision.
3. Keep detected secret bytes out of receipts, logs, caches, dashboards,
   exports, diagnostics, and external requests.
4. Preserve exact packet identity and evidence semantics; the first release
   blocks delivery instead of silently rewriting source excerpts.
5. Provide a separate, explicit, bounded, read-only inventory of likely
   hardcoded secrets in the authorized current snapshot.
6. Make detector, policy, scan, and decision identities reproducible and
   auditable without retaining matched values.
7. Give users clear local remediation guidance without mutating, validating,
   rotating, revoking, or uploading credentials.

## Non-goals

- General-purpose enterprise DLP, data classification, compliance discovery,
  PII detection, legal hold, content moderation, or endpoint protection.
- A guarantee that every secret or sensitive value will be detected.
- Credential validation against GitHub, cloud providers, databases, or any
  other network service.
- Automatic repository edits, secret deletion, rotation, revocation, commit
  rewriting, or issue creation.
- Trusting source files, prompts, comments, or repository configuration to
  disable protection.
- Scanning Git history, untracked files, ignored files, submodules, external
  mounts, build outputs, archives, or unsupported binary formats in the first
  implementation.
- Reusing YARA-X, malware analyzers, or the isolated analyzer runner as a
  substitute for a purpose-built secret detector.
- Inspecting client responses or becoming a proxy for all client/model traffic.

## Users and outcomes

| User | Desired outcome |
| --- | --- |
| Novice developer | A likely credential cannot be sent externally merely because the developer did not know to request a secret scan. |
| Experienced developer | A blocked delivery identifies the affected local artifact and detector class without reproducing the secret. |
| Security reviewer | Detector, policy, packet, decision, and fixture identities make the result reproducible and its limitations explicit. |
| Operator | A bounded local inventory reports likely current-snapshot secrets without network access or repository mutation. |

## Product modes

### Mode 1: automatic outbound guard

The guard runs for every source-bearing packet handled by an Impresari-owned
external-delivery adapter. It scans only the already-authorized immutable
packet bytes. A clean decision permits delivery of those exact bytes. A
blocking or indeterminate decision produces a local no-delivery receipt.

The initial mode has no `allow anyway` action. The user may remove or rotate
the credential, narrow the request so the affected evidence is not selected,
or keep the work local. Any later exception mechanism requires its own decision
record and must not be controlled by repository content or model output.

### Mode 2: explicit current-snapshot inventory

An explicit local command scans ordinary text artifacts already admitted to an
authorized current snapshot, subject to independent file, byte, time, finding,
and output budgets. It reports metadata-only findings locally and never sends
source or findings to a provider.

Inventory mode is not required to make Mode 1 protective. It is a discovery
and remediation aid, not an external-delivery prerequisite.

### Deferred mode: Git-history inventory

History scanning is excluded initially because it expands the object set,
resource profile, identity model, retention questions, and remediation UX. It
requires a separate PRD/ARD/ADR checkpoint before implementation.

## Functional requirements

| ID | Requirement |
| --- | --- |
| PSEP-FR-001 | Every Impresari-owned external-delivery serializer that can carry source must accept only a packet accompanied by a current clean DLP decision. |
| PSEP-FR-002 | The outbound guard must inspect the exact immutable bytes selected for delivery, after planning and before client serialization or invocation. |
| PSEP-FR-003 | Detection must be local, deterministic, versioned, content-addressed, and free of network, subprocess, model, credential-store, and environment-enumeration authority. |
| PSEP-FR-004 | A blocking finding, scan failure, stale decision, detector/policy mismatch, unsupported encoding, exceeded scan budget, or incomplete coverage must produce no external delivery. |
| PSEP-FR-005 | A clean decision must preserve the original packet bytes and packet ID exactly. The first implementation must not redact or rewrite source in place. |
| PSEP-FR-006 | Findings and receipts must never contain matched secret bytes, surrounding source, a reversible encoding, or a hash derived only from the secret value. |
| PSEP-FR-007 | A finding may contain detector class, severity, confidence, current snapshot and artifact identities, relative path for local display, exact span, and a value-independent finding ID. |
| PSEP-FR-008 | Repository files, prompts, model output, client output, comments, and task text must not suppress, downgrade, or bypass a finding. |
| PSEP-FR-009 | The initial detector catalog must use high-confidence structural and contextual rules. Entropy alone must not create a clean decision or a standalone finding. |
| PSEP-FR-010 | Private-key material, recognized provider-token forms, credential-bearing connection strings, and credential-name-plus-value structures must have original-synthetic positive, near-miss, mutation, and benign-collision fixtures. |
| PSEP-FR-011 | The inventory command must be explicit, read-only, current-snapshot-bound, previewable, locally rendered, and independently budgeted. |
| PSEP-FR-012 | Inventory findings must not alter packet admission, repository state, policy, cache authority, or external-delivery settings. |
| PSEP-FR-013 | Logs, dashboard events, and audit records may contain only bounded metadata such as detector version, decision, counts, coverage, duration, and reason codes. |
| PSEP-FR-014 | Detection must never test whether a credential works, contact its issuer, or infer account ownership. |
| PSEP-FR-015 | User-facing output and public claims must state that detection is best-effort and not a replacement for provider secret scanning, push protection, a secret manager, credential rotation, or security review. |

## Detection and decision model

The initial catalog should distinguish:

- `private_key_material`: a bounded recognized private-key envelope;
- `provider_credential`: a versioned provider-specific prefix/shape;
- `credential_assignment`: a credential-bearing name plus a bounded value
  shape in a safely supported textual form;
- `credential_uri`: a recognized URI form containing user-secret material;
- `indeterminate`: bytes or scan state for which the guard cannot prove the
  required coverage.

All four positive classes block external delivery. `indeterminate` also blocks
delivery but is not reported as a confirmed secret. Entropy may strengthen a
contextual detector but cannot independently classify ordinary hashes, UUIDs,
content identities, checksums, or generated test data as credentials.

## User experience

A blocked external delivery should say, in substance:

> Delivery blocked: the packet contains one or more likely secrets, or the
> secret scan could not establish complete coverage. No packet was sent.

The local detail view may show relative artifact path, line/column span,
detector class, confidence, reason code, and remediation choices. It must not
show the matched value or source context. External clients receive nothing
from the blocked packet, including the finding report.

Inventory output should separate likely-secret findings from indeterminate or
unsupported artifacts and show explicit coverage and omission totals.

## Privacy and security requirements

- Source and matched values exist only in bounded process memory for the scan
  and follow existing packet/session lifetime rules.
- No matched byte enters persistent cache, audit storage, diagnostic bundles,
  screenshots, fixture output, or CI artifacts.
- Test fixtures use only obvious, nonfunctional, Impresari-owned synthetic
  values reserved for testing.
- The detector catalog is product-owned. Workspace content cannot add or
  remove detectors.
- A catalog or policy update is a release-governed security change with exact
  compatibility and regression evidence.
- The egress decision is bound to the exact packet, detector catalog, policy,
  engine version, and scan result; it cannot be reused for another packet.
- Source mutation between scan and delivery is structurally irrelevant because
  the guard scans immutable packet bytes, not a path that is reopened later.

## Performance and resource requirements

- Outbound scanning is linear in packet bytes and remains inside the packet's
  existing hard byte and time ceilings, with its own smaller fail-closed limit.
- The inventory command has explicit maxima for files, bytes per file, total
  bytes, findings, duration, and output bytes.
- Parallelism is bounded and deterministic output ordering is independent of
  scheduling.
- An exceeded limit yields explicit incomplete coverage; it never silently
  returns `clean`.

## Acceptance criteria

1. A source-free contract and closed schemas define detector catalogs,
   policies, findings, scan results, egress decisions, and metadata-only
   receipts before production integration.
2. Original-synthetic corpora cover every detector class, mutations, partial
   prefixes, escaped forms, multiline boundaries, Unicode confusables,
   generated hashes, UUIDs, test placeholders, malformed encodings, and budget
   boundaries.
3. Property tests prove that only an exact, current `clean` decision can create
   a delivery-capable packet wrapper.
4. Integration tests prove every admitted external adapter refuses raw,
   blocked, stale, mismatched, indeterminate, and incomplete packets.
5. Canary tests prove matched values never enter receipts, logs, cache,
   dashboard state, diagnostics, process arguments, environment, or external
   client input.
6. Packet-equivalence tests prove a clean packet delivered through the guard
   is byte-identical to the direct planner packet.
7. Inventory tests prove authorization, snapshot identity, path containment,
   budgets, deterministic ordering, unsupported-object reporting, and complete
   source-workspace immutability.
8. Real-client rehearsals use only synthetic nonfunctional credentials and
   independently prove no invocation occurs for a blocked packet.
9. Documentation and conformance language explicitly preserve false-negative
   and false-positive residual risk.

## Proposed delivery sequence

1. **PSEP-0 — contracts only:** closed schemas, detector taxonomy, policy,
   reason codes, receipts, limits, and original-synthetic provenance.
2. **PSEP-1 — pure detector:** local deterministic scanning over supplied
   bounded bytes, with no packet or client integration.
3. **PSEP-2 — outbound interlock:** delivery-capable wrapper and fail-closed
   integration with the client-neutral delivery boundary.
4. **PSEP-3 — client evidence:** independent synthetic blocked/clean evidence
   for every admitted external-delivery adapter and version scope.
5. **PSEP-4 — repository inventory:** explicit bounded current-snapshot scan
   and metadata-only local report.
6. **Later decision:** operator-owned exceptions, expanded data classes, or Git
   history only after separate risk and product review.

## Documentation-only boundary

This PRD does not authorize schemas, fixtures, detectors, scanning, source
access, packet interception, client changes, network access, repository
inventory, history access, suppressions, code, tests, workflows, releases, or
product claims. Those require a separately approved implementation increment.
