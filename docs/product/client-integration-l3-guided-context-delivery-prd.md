# Impresari Context — CI-3: Planner-Backed Guided Context Delivery PRD

- Status: Approved for implementation
- Date: 2026-08-24
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: Phase 3 deterministic planner; CI-1 and CI-2 are independently admitted per client.
- Architecture requirements: [CI-3 delivery ARD](../architecture/ci-3-guided-context-delivery-ard.md)

## Objective

Where an official client lifecycle surface safely permits it, offer an explicit,
opt-in delivery of a deterministic Impresari Context packet at a defined task
stage. The user must see the profile, budget, packet identity, reason codes,
coverage, omissions, and a no-delivery fallback before any packet reaches the
client.

## Scope

- A client-neutral delivery intent containing a named task profile, query,
  snapshot identity, policy profile, and hard evidence budget.
- Deterministic planner output only: exact retrieval plan, selected evidence,
  reason codes, coverage/omission report, and packet identity.
- Thin client lifecycle adapters only for documented, stable extension points;
  each adapter has an explicit version/OS/scope record and a disabled default.
- Preview, validate, explicit enable/disable, inspect, and exact owned-adapter
  removal. A client that has no safe lifecycle point receives no adapter.
- Packet-equivalence, redaction, budget, source-immutability, degraded-mode,
  and no-hidden-delivery test evidence per client/scope.

## Non-goals

- Background indexing, prompt interception or rewriting, proxying AI providers,
  remote relay services, persistent memory, automatic source edits, shell
  hooks, broad workspace trust, model selection, or agent orchestration.
- Delivery inferred from an unbounded conversation, repository content, or a
  model-selected profile. Conversational models may request a packet, but never
  silently choose an automatic-delivery policy.

## Required flow

1. The user explicitly enables a named client/scope adapter after reviewing its
   target and bounded delivery policy.
2. The adapter receives only an explicit task intent or declines delivery when
   the lifecycle surface does not safely provide one.
3. The deterministic planner creates a snapshot-bound packet under the chosen
   profile and hard budget.
4. The adapter delivers the exact serialized packet or a stable unavailable/
   omitted notice; it records identity, reason, and whether delivery occurred.
5. Disable/removal restores the client’s prior state without modifying source,
   unrelated settings, trust, approvals, or other extensions.

## Acceptance criteria

- Delivery is disabled by default and cannot occur without explicit owned
  adapter installation plus user-selected profile and budget.
- The delivered bytes equal the independently resolved planner packet for the
  same snapshot and request; packet ID, policy ID, plan ID, coverage, and
  omission reasons are observable.
- The adapter rejects stale snapshots, unknown profiles, missing user consent,
  malformed lifecycle input, budgets that exceed policy, and unsupported
  client/version/scope combinations.
- Tests prove no delivery, a bounded successful delivery, redaction, budget
  exhaustion, stale-snapshot rejection, client failure/degradation, exact
  removal, and source-workspace immutability.
- L3 does not promote a client to L1 or L2; client classifications remain
  independent and evidence-backed.

## Security posture

The only data crossing a client boundary is the already-authorized, bounded,
visible packet. No raw workspace read, credential, shell output, model prompt,
or hidden state is exported by an adapter. Adapters do not network, execute
repository code, or retain packets after their packet/session lifetime.

## Reassessment checkpoint

After each proposed lifecycle adapter, reassess client stability, the
compatibility matrix, planner invariants, and the master roadmap. Remove or
defer an adapter when a client cannot preserve explicit consent, packet
equivalence, or exact removal.
