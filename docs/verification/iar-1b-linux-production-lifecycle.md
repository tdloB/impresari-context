# IAR-1B Linux Production-Lifecycle Contract Verification

- Date: 2026-08-30
- Decision: ADR-0079
- Scope: source-free lifecycle contract only
- Production admitted: No
- Real analyzer authorized: No

## Bound Contract

`linux-isolation/linux-iar-1b-production-lifecycle-v1.json` fixes the selected
A+C profiles, package scope, canonical phase sequences, clean-state invariant,
health-withdrawal behavior, and authority ceiling. The conformance fixture is
byte-identical to the released policy.

## Deterministic Verification

`ruby scripts/check-linux-isolation-production-lifecycle.rb` verifies:

- a complete rootless lifecycle candidate;
- a complete external lifecycle candidate with operator relaunch;
- incomplete evidence;
- ordinary lifecycle failure;
- failed claim withdrawal; and
- invalid operation evidence.

Every receipt denies production, release packaging, real analyzers, privileged
installation, persistent services, host discovery, execution, network,
credentials, service mutation, background monitoring, and automatic repair.
Malformed policy and schema-level authority overclaims fail closed.

## Claim Boundary

The fixtures are original synthetic evidence and do not represent a live
installation. This checkpoint establishes the exact state machine that future
hosted package rehearsals must satisfy. It does not itself admit Linux,
publish a package, or open IAR-2.
