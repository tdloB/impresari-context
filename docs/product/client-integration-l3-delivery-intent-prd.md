# Impresari Context — CI-3a: Explicit Delivery-Intent Contract PRD

- Status: Approved for implementation
- Date: 2026-08-25
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Governing roadmap: [Client Integration Depth Roadmap](client-integration-roadmap.md)
- Dependency: [CI-3 Guided Context Delivery PRD](client-integration-l3-guided-context-delivery-prd.md)
- Architecture requirements: [CI-3a delivery-intent ARD](../architecture/ci-3a-delivery-intent-ard.md)

## Objective

Define and implement the client-neutral, explicit data contract that a future
client lifecycle adapter must submit before Impresari can build and serialize
one deterministic planner packet. It is an in-process reference capability,
not a client hook, installer, or delivery mechanism.

## Scope

- Strictly deserialized intent: adapter contract version, client/scope/version/
  lifecycle identity, explicit consent, request/event identifiers, consumer
  identity, UTC time, supported profile, query, exact planner steps, and hard
  resource budget.
- Validation of fixed supported identity, explicit one-delivery consent,
  bounded identifiers/query/steps, and contract version before engine use.
- A reference delivery result whose serialized packet bytes are exactly the
  shared planner result and whose receipt exposes packet/plan/snapshot/policy
  identity, delivery outcome, and no added authority.
- Deterministic no-delivery results for disabled, unsupported, or malformed
  intents; no adapter may infer a profile, query, lifecycle point, or consent.

## Non-goals

- A native client adapter, lifecycle hook, configuration install, delivery to a
  client process, packet persistence, retry, networking, shell/process access,
  source mutation, client state mutation, or promotion of any integration.

## Acceptance criteria

- Equivalent declared planner steps, request identity, and budget yield packet
  bytes identical to the direct public planner call.
- Unknown fields, incompatible contract, absent consent, unsupported identity,
  invalid profile/query, and malformed budget fail closed without engine work.
- Tests prove byte equivalence, visible identity/receipt data, disabled and
  rejected no-delivery paths, and that the adapter adds no authority.

## Reassessment checkpoint

After release gating CI-3a, reassess official lifecycle surfaces for each
priority client. Only a documented surface with explicit consent and exact
removal may receive a client-specific CI-3b adapter.
