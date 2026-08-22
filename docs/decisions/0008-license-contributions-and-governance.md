# ADR-0008: License, contributions, and project governance

- Status: Accepted; public-name and legal-steward gates resolved
- Date: 2026-08-20
- Scope: Source, documentation, contributions, releases, upstream attribution,
  project decisions, and security governance

## Context

The project is intended to be genuinely open source, broadly usable by
individuals and commercial products, and independently implemented from LeanCTX
and Graft. It needs a predictable outbound license, a lightweight way for
contributors to certify provenance, transparent decision records, and controls
for security-sensitive changes.

BoldtHaus Studio, LLC is the legal project steward and initial copyright holder.
The founder confirmed Impresari Context as the public project name on
2026-08-22. A
software license does not resolve ownership, trademark, patent, attribution, or
public-name questions by itself.

## Decision

### Outbound license

License original project software and documentation under the **Apache License,
Version 2.0**, SPDX identifier `Apache-2.0`, subject to confirming the initial
copyright/steward record before public repository creation.

- Include the unmodified official Apache 2.0 license text as `LICENSE`.
- Use SPDX identifiers in source and generated artifacts where appropriate.
- Do not add field-of-use, non-compete, “ethical source,” hosted-service, or
  source-available restrictions and still describe the result as Apache-2.0.
- Dependencies and incorporated third-party material retain their own compatible
  licenses and notices.
- The project license grants no permission to use project names or logos as
  trademarks beyond applicable law and any separately published trademark
  policy.

Apache-2.0 is OSI-approved and includes an express patent-license framework,
which is valuable for developer infrastructure and commercial adoption. This
document is a project policy decision, not legal advice; counsel may require
implementation details or an updated decision before public release.

### Copyright and inbound contributions

- Contributors retain copyright in their original contributions unless a
  separate written assignment is deliberately adopted later.
- Contributions are accepted under the same Apache-2.0 outbound license.
- Require Developer Certificate of Origin 1.1 sign-off on commits/patches through
  `Signed-off-by` trailers.
- Do not require a Contributor License Agreement initially.
- A contributor must identify copied, adapted, generated, employer-owned, or
  otherwise third-party-derived material and its license/provenance.
- AI assistance does not remove the contributor's DCO responsibility or the
  project's provenance review.

Changing from DCO to a CLA, requiring assignment, or changing the license is a
material governance decision requiring a new ADR and contributor impact review.

### Initial steward and maintainer model

- Aaron Boldt is the initial project lead and maintainer. BoldtHaus Studio, LLC
  is the legal steward and initial copyright holder.
- `GOVERNANCE.md` will define roles, appointment/removal, decision authority,
  conflicts, inactivity, and succession.
- `MAINTAINERS.md` will identify current maintainers and security/release
  responsibilities without publishing unnecessary personal data.
- Maintainer status is earned through sustained reviewed contribution and trust;
  it is not automatically granted by employment, sponsorship, or volume.
- The project will document material sponsorship or employment conflicts that
  affect a decision.

### Decision process

- Normal changes use public pull requests, passing tests, provenance checks, and
  maintainer review.
- Material architecture, trust-boundary, public-contract, storage-identity,
  security, extension, model, network, governance, or license changes require an
  ADR.
- The project prefers documented consensus. When consensus cannot be reached,
  the project lead decides within the published scope and records the rationale
  and opposing evidence.
- Before a second maintainer exists, the founder may merge ordinary changes after
  CI and a recorded self-review. Public release still requires all independent
  security/evaluation gates promised by the PRDs; solo governance cannot waive
  them silently.
- Once two qualified maintainers exist, changes to authorization, evidence
  identity, canonicalization, parser/extension execution, release credentials,
  security policy, and governance require review by a second maintainer.

### Repository protection and releases

Before public release:

- protect the default branch against direct unreviewed pushes where the hosting
  platform supports it;
- require CI, DCO/provenance, formatting/lint, test, security, and license checks;
- pin third-party CI actions and toolchains;
- publish versioned changelogs and support policy;
- create release tags through the documented maintainer process;
- produce checksums, dependency/SBOM evidence, and build provenance appropriate
  to the distribution method;
- keep package/release credentials least-privileged, separate from development,
  and recoverable/revocable;
- never enable automatic self-update in the engine.

The exact signing and reproducible-build mechanism requires a release-engineering
decision before the first binary release.

### Required community files

The local scaffold will include:

- `LICENSE` — exact Apache License 2.0 text;
- `NOTICE` — project notices only when required; do not place informal influence
  acknowledgments here if they could imply incorporated upstream code;
- `ACKNOWLEDGMENTS.md` — LeanCTX, Graft, and other architectural influences with
  clear non-affiliation/no-code-incorporation language;
- `CONTRIBUTING.md` — workflow, DCO, tests, provenance, generated/AI-assisted
  contribution expectations, and conduct;
- `GOVERNANCE.md` and `MAINTAINERS.md`;
- `SECURITY.md` — private reporting, supported versions, response practice, and
  disclosure expectations;
- `CODE_OF_CONDUCT.md` — an established open-source code of conduct selected
  without material modification, with real enforcement contact/process;
- `THIRD_PARTY_LICENSES` or generated equivalent plus dependency inventory;
- issue and pull-request templates that collect reproduction/provenance without
  soliciting private source or secrets.

### LeanCTX and Graft attribution

- Credit LeanCTX, Yves Gugger, and contributors, and Graft/Nanonets contributors
  in `ACKNOWLEDGMENTS.md` and the architecture provenance document.
- Describe the project as independently implemented and “informed by” or
  “inspired by” publicly demonstrated capabilities.
- Do not imply endorsement, affiliation, a fork, merger, formal successor, or
  clean-room process.
- Do not copy or mechanically translate upstream source, tests, fixtures,
  prompts, documentation, naming, or visual assets.
- Any future source reuse triggers the documented source-reuse gate before merge,
  including immutable revision, path mapping, license, notices, modifications,
  compatibility, and maintainer approval.
- Courtesy outreach is encouraged when the architecture and public materials are
  reviewable; it is not a substitute for license compliance or a request for
  implied endorsement.

### Security governance

- Security reports use a private channel published in `SECURITY.md`.
- The project records severity, affected versions, remediation, advisory,
  release, and disclosure decisions without exposing reporter or private source
  unnecessarily.
- A maintainer cannot waive an MVP hard security gate without a public ADR,
  founder/steward approval, explicit residual-risk disclosure, and any required
  independent review; critical authorization/data-isolation failures remain
  non-waivable for release.
- No remote kill switch or silent update is permitted.

### Public-name and owner gate

Before a public repository or package is created under **Impresari Context**:

1. confirm the legal project steward/copyright record;
2. complete the existing naming/counsel gate for the public name;
3. approve trademark and attribution language;
4. verify that `LICENSE`, copyright notices, package metadata, security contact,
   and governance contacts are accurate;
5. confirm that no private AI App Builder OS content or credentials are present.

The public-name portion of this gate was confirmed by the founder on 2026-08-22.
This project record documents that product decision but is not legal advice or a
trademark registration.

## Rationale

Apache-2.0 supports broad open-source and commercial use while providing explicit
license and patent terms. A DCO is a lightweight provenance certification that
fits contributor-retained copyright and avoids imposing a CLA before the project
has an organization that needs one. ADR-based governance preserves the evidence
and security boundaries established by the product documents.

## Consequences

### Positive

- Clear OSI-approved permissive license for users and companies.
- Contributor provenance without initial copyright assignment.
- Upstream inspiration is credited transparently without implying code reuse.
- Material decisions and security exceptions remain reviewable.
- Governance can grow from a founder-led project without pretending a community
  structure already exists.

### Costs

- DCO sign-off adds contribution friction.
- Contributor-retained copyright can make future relicensing difficult, which is
  intentional protection against unilateral license changes.
- Founder-led early governance has a bus-factor and independent-review gap.
- Apache notice, dependency, patent, and trademark questions still need careful
  maintenance and may require counsel.

## Alternatives Considered

### MIT license

Simpler, but rejected because Apache-2.0 provides more explicit patent and notice
terms for infrastructure likely to receive commercial use.

### Dual Apache-2.0/MIT

Common in the Rust ecosystem but not necessary for the initial project and adds
ambiguity about notices and contribution expectations. It can be reconsidered
only with a concrete adoption need and legal review.

### GPL/AGPL copyleft

Rejected for the initial goal of broad embedding in different clients and
commercial products. This is a strategic choice, not a judgment about copyleft.

### CLA or copyright assignment

Deferred because no legal foundation/company need has been established and the
DCO plus inbound=outbound license is adequate for initial provenance.

### Benevolent dictator language without succession rules

Rejected because authority, inactivity, conflicts, and promotion should be
documented even in a founder-led project.

## Verification

- Automated SPDX/license and dependency inventory checks.
- DCO sign-off check on contributions.
- Provenance checklist in pull requests and code review.
- Release checklist validates license, notices, third-party inventory, security,
  governance, owner, naming, and private-content boundaries.
- Periodic maintainer/security contact and supported-version review.

## Official References

- [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [Apache licensing information](https://www.apache.org/licenses/)
- [OSI approved licenses](https://opensource.org/licenses)
- [Developer Certificate of Origin 1.1](https://developercertificate.org/)

## Review Triggers

Review when the legal steward changes, a CLA/assignment is proposed, relicensing
is requested, a foundation is formed, commercial hosted terms are proposed, a
trademark policy is ready, a material upstream source import is requested, or
the maintainer/community structure outgrows founder-led governance.
