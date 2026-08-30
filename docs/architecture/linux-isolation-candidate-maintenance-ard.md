# Linux IAR-1B Candidate Maintenance ARD

- Status: Accepted
- Date: 2026-08-30
- Decision: ADR-0077

## Architecture

`linux-isolation/linux-iar-1b-candidate-v1.json` is the released claim source.
It binds exact target identities and evidence to immutable SHA-256 identities.
`scripts/linux-isolation-maintenance.rb` is a foreground, source-free pure
assessment surface:

```text
released manifest + explicit observation
                    |
                    v
        closed manifest validation
                    |
                    v
 exact target/class/freshness comparison
                    |
                    v
 authority-denying candidate health receipt
```

The evaluator neither discovers nor verifies the live host. Collection and any
new synthetic rehearsal stay outside this contract and require their own
bounded workflow. This separation prevents a read-only health query from
acquiring process, service, privilege, or network authority.

## Deterministic State Order

1. Reject malformed or authority-expanding manifests.
2. Unknown target: `unsupported`.
3. Diversity-only target: `unsupported`.
4. Candidate unavailable: `unavailable`.
5. Exact evidence unavailable: `missing_evidence`.
6. Freshness expired: `stale_evidence`.
7. Any runner label/image, OS release, kernel, architecture, or Landlock ABI
   mismatch: `changed`.
8. Otherwise: `compatible_candidate`.

Only the final state sets `candidate_claim_active=true`. Production and real
analyzer admission remain false in every state.

## Integrity And Withdrawal

Repository verification recomputes the profile, probe, composite-check, and
evidence fixture digests from the current tree. Any mismatch fails CI. Runtime
receipts bind the manifest digest and selected evidence digest. Expiry,
missing evidence, target unavailability, host identity drift, or bound-artifact
drift withdraws the candidate claim rather than falling back.

## Resource Profile

The evaluator is a short foreground Ruby process over small bounded JSON. It
spawns no children and writes no persistent state. The existing
`iar-linux-synthetic-v1` resource profile remains the synthetic worker profile;
this maintenance surface does not run that worker.
