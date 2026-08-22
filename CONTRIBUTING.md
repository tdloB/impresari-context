# Contributing

Thank you for helping improve Impresari Context. Use a fork or topic branch and
submit changes through a pull request. Do not include vulnerability details in
a public contribution; follow `SECURITY.md` instead.

## Development checks

Run `./scripts/check.sh` before submitting a change. If dependency manifests or
the lockfile change, also run `./scripts/audit-dependencies.sh`. Changes must
preserve the security boundaries and acceptance requirements in `docs/`.

Major new functionality and changes to observable behavior must include
suitable automated tests. Tests should cover the normal path and applicable
boundary, failure, and security behavior. If automated tests are not practical
or applicable, explain why in the pull request and obtain maintainer approval
for the documented exception before merge.

## Developer Certificate of Origin

Every contribution submitted after the public contribution process begins must
include a `Signed-off-by` trailer certifying the Developer Certificate of Origin
1.1. Sign commits with:

```text
git commit -s
```

Read the DCO at <https://developercertificate.org/>. Sign-off is a provenance
certification, not a copyright assignment.

## Provenance

Identify copied, adapted, generated, employer-owned, or otherwise third-party
material. AI assistance does not transfer responsibility: contributors must
have the right to submit every contribution and must review it themselves.
Do not copy or mechanically translate LeanCTX or Graft material.

Material changes to architecture, trust boundaries, public contracts, storage
identity, security, extensions, models, networking, governance, or licensing
require an ADR.
