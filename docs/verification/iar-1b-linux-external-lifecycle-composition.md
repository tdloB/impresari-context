# IAR-1B Linux External Lifecycle Composition

- Date: 2026-08-30
- Decision: ADR-0081
- Profile: `externally_managed`
- Production admitted: No
- Real analyzer authorized: No

## Contract Verification

Run:

```sh
ruby scripts/check-linux-external-lifecycle-composition.rb
```

The check reproduces one linked synthetic lifecycle candidate and deterministic
identity, package, external, interruption, and withdrawal failures. It also
holds the single accepted external provisioner launch site and verifies that the
health collector contains no `systemd-run`, `systemctl`, `sudo`, `curl`, or
`wget` authority.

The closed schema admits the source-free health and composition receipts and
rejects a production overclaim. The complete repository gate runs this check.

## Hosted Checkpoint

The release-candidate Linux job now composes the exact candidate package with a
fresh external rehearsal in the same exact-source workflow. The operator may
create only the one accepted temporary delegated service. After collection, the
health collector runs with descriptor 3 closed; all source records and the
composition receipt are retained in the candidate artifact for bounded review.

The first exact hosted composition will be recorded after this change reaches
`main`. Until then, C retains its independently proven package and external
synthetic candidates, but not the composed lifecycle-candidate claim. A remains
partial pending genuine login-session reentry, and production and IAR-2 remain
closed.
