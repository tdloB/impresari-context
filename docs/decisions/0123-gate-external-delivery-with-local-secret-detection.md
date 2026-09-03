# ADR-0123: Gate External Delivery With Local Secret Detection

- Status: Proposed; documentation only; implementation not authorized
- Date: 2026-09-01
- Decider: Aaron Boldt requested the design records; implementation decision remains open
- Related PRD: [Proactive Secret Egress Protection PRD](../product/proactive-secret-egress-protection-prd.md)
- Architecture: [Proactive Secret Egress Protection ARD](../architecture/proactive-secret-egress-protection-ard.md)

## Context

Impresari Context can construct exact-source packets and deliver explicitly
authorized packets through admitted external client surfaces. Existing
controls minimize logs, preserve sensitivity/redaction metadata, bind consent,
and constrain delivery, but the product explicitly does not claim to be a
secret scanner or DLP system. An authorized exact query may therefore include
a hardcoded repository credential.

Relying on users to request a separate security scan is insufficient for the
external-delivery boundary, especially for novice users. At the same time,
calling a pattern matcher “DLP” would overstate protection, and silently
redacting an immutable exact-source packet would weaken evidence semantics.

## Proposed decision

Adopt a narrowly named **Proactive Secret Egress Protection** capability in two
separately admitted layers:

1. an automatic local deterministic guard on every source-bearing packet sent
   through an Impresari-owned external-delivery adapter; and
2. a later explicit, bounded, read-only inventory of likely hardcoded secrets
   in an authorized current repository snapshot.

The initial egress guard scans the final immutable packet after planning and
before serialization. A complete scan with zero blocking findings creates an
opaque delivery-capable wrapper around the exact packet. A finding,
indeterminate state, failure, stale binding, policy/catalog mismatch,
unsupported input, or incomplete coverage produces a local no-delivery
receipt and no client invocation.

The first implementation blocks rather than redacts. It has no repository-,
prompt-, model-, or client-controlled suppression and no `allow anyway` path.
It performs no network request, credential validation, process execution,
repository mutation, history scan, secret rotation, or external reporting.
Matched values are never retained or emitted.

This capability must be described as best-effort secret detection, not
complete DLP and not a replacement for provider secret scanning, push
protection, a secret manager, credential rotation, or human security review.

## Consequences

### Benefits

- Protection is automatic at the point where disclosure would occur; novice
  users do not need to know that a separate scan exists.
- External delivery fails closed without giving the detector broader workspace,
  provider, process, or credential authority.
- Clean packets retain byte-for-byte planner equivalence and exact packet IDs.
- Findings are auditable by detector, evidence span, catalog, policy, and
  packet identity without reproducing the credential.
- The pure detector can later support a local current-snapshot inventory
  without coupling external delivery to a whole-repository scan.

### Costs and risks

- False positives can block useful delivery because the initial design has no
  bypass. Users must remove/rotate the value, narrow the request, or work
  locally.
- False negatives remain possible; a clean decision is not proof that a packet
  contains no secret.
- Every external adapter becomes dependent on current detector and policy
  compatibility evidence.
- Provider token formats and benign-code corpora require ongoing versioned
  maintenance.
- Scanning adds bounded latency proportional to packet size.
- Inventory scanning adds a separate source-read and resource surface and must
  not be admitted by implication from the egress guard.

## Alternatives considered

### Keep the current disclosure only

Rejected as the recommended future state because documentation does not prevent
an accidental external disclosure and assumes users recognize the risk.

### Depend on GitHub or hosting-provider secret scanning

Rejected as the sole control because not every workspace is hosted on a
supported provider, provider configuration varies, findings may arrive after
delivery, and Impresari must govern its own egress boundary. Provider scanning
remains complementary.

### Silently redact matching text and deliver the rest

Rejected for the initial implementation because modified bytes are no longer
the exact planner packet, partial replacement can leave usable credential
fragments, and the UX could create false confidence. A typed derived-redaction
design requires a separate decision.

### Scan the whole repository before every packet

Rejected because it adds avoidable latency and read scope while failing to
improve the decision about bytes not selected for delivery. Whole-snapshot
inventory remains an explicit separate mode.

### Use an external cloud DLP or secret-validation API

Rejected because it would disclose source or secret candidates to another
service, add network and credential authority, introduce provider availability
into local admission, and conflict with local deterministic operation.

### Reuse YARA-X or the malware analyzer runner

Rejected because malware-signature execution and hardcoded-secret detection
have different threat models, output contracts, release gates, and product
claims. Secret protection should not wait for or inherit analyzer authority.

### Permit an immediate user override

Rejected for the initial design because overrides create a high-risk bypass and
require durable operator ownership, expiry, audit, policy precedence, and
novice-safe UX. A later exception mechanism requires its own ADR.

## Required implementation gates

If implementation is later approved, it must proceed in independently reviewed
increments:

1. closed contracts and original-synthetic fixture provenance;
2. a pure bounded detector with denied side effects;
3. a client-neutral typed egress interlock;
4. independent integration evidence for every admitted external adapter; and
5. a separately admitted explicit current-snapshot inventory.

No increment may claim exhaustive detection. Git-history scanning, derived
redacted delivery, operator exceptions, remote detector updates, additional
sensitive-data classes, or automatic repository remediation each require a
separate decision.

## Current authorization boundary

This ADR records a proposal only. The requested change authorizes creation of
the PRD, ARD, and ADR and nothing else. It does not authorize roadmap edits,
schemas, fixtures, code, tests, detectors, source scanning, packet interception,
client changes, workflows, network access, repository inventory, history
access, suppressions, releases, or protection claims.

## Revisit triggers

Revisit before:

- approving any implementation increment;
- changing block-only behavior or adding redacted delivery;
- adding an override or suppression mechanism;
- scanning outside final packet bytes or the authorized current snapshot;
- adding Git-history, archive, binary, submodule, ignored, or untracked scans;
- adding a remote rules feed, provider API, model, subprocess, or analyzer;
- persisting finding details or exposing them to dashboard/export surfaces;
- claiming general DLP, exhaustive secret detection, or compliance coverage.
