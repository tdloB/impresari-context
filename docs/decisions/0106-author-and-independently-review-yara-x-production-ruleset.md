# ADR-0106: Author And Independently Review The First YARA-X Production Ruleset

- Status: Approved; Option A source package implemented; independent human ruleset review backlogged but required before compilation
- Date: 2026-08-31
- Related PRD: [YARA Analyzer Admission PRD](../product/yara-analyzer-admission-prd.md)
- Related architecture: [YARA Analyzer Admission ARD](../architecture/yara-analyzer-admission-ard.md)
- Related decisions: ADR-0073, ADR-0074, ADR-0085, ADR-0098, ADR-0100, ADR-0102, ADR-0103, ADR-0104

## Context

ADR-0104 produced one short-lived, authenticated-reader YARA-X engine
candidate and verified its exact contents without executing it. ADR-0103
requires the engine and production rules to remain separate so either can be
reviewed, updated, expired, rolled back, or revoked without silently changing
the other.

The existing YARA-X rules are synthetic compatibility fixtures. They prove a
narrow parser and execution contract, but they are not security policy and
cannot be renamed or promoted into a production ruleset. A production rule can
create false positives, false reassurance, licensing obligations, and an
ongoing response burden. Its intended detection, source, limits, tests, and
human review therefore need their own frozen boundary before rule authorship.

This decision is deliberately separate from the deferred whole-product
independent security review in ADR-0085. The same qualified reviewer may later
perform both reviews, but each scope, evidence record, and admission result
must remain explicit.

## Proposed Decision

Choose a small, original, project-owned version-1 ruleset. Do not import a
community feed, vendor collection, repository-supplied rule, or real malware
sample for the first release.

### Claim and purpose

- The ruleset reports only exact, named static indicators that each rule is
  designed and tested to recognize.
- Every match remains an observation requiring interpretation. A match does
  not prove malware, and no match does not prove safety or malware-free status.
- Each rule must declare one stable identifier, purpose, severity-independent
  category, supported artifact kinds, exact positive conditions, known
  false-positive conditions, known blind spots, and owner.
- Version 1 targets high-confidence, explainable indicators. Breadth, heuristic
  scoring, family attribution, behavior prediction, and comprehensive malware
  coverage are out of scope.

### Source and language boundary

- Store original Impresari-owned source separately from synthetic
  compatibility rules and bind it to a source manifest and SHA-256 identity.
- Permit only the ADR-0098 module-free literal and hexadecimal condition
  surface.
- Continue to prohibit imports, includes, external variables, regular
  expressions, base64 and XOR modifiers, repository-provided rules, network
  retrieval, and compilation inside an analyzer job.
- Reject duplicated identifiers, unbounded strings, private or global policy
  coupling, undeclared metadata, and syntax outside the closed profile.
- Record copyright, license, author, review state, and change rationale for
  every rule. Third-party rule text or signatures are not eligible for this
  candidate.

### Test corpus

- Use only original, generated, non-malicious byte fixtures with exact
  provenance and licenses.
- Require at least one exact positive, one near-miss negative, one benign
  collision challenge, and one bounded mutation case per rule.
- Keep the compatibility corpus distinct from the production-rule corpus so a
  parser smoke cannot be represented as detection validation.
- Run source linting and source-free contract evaluation before any compiler or
  engine is invoked.
- A later separately authorized build may compile the exact reviewed source
  outside scan jobs and may run only the approved original-synthetic corpus
  inside an admitted isolation boundary. This proposal does not authorize that
  build or execution.

### Independent ruleset review

Before a compiled ruleset candidate can be represented as review-complete, an
attributable human who did not author the rules must:

- disclose conflicts and demonstrate practical YARA or malware-analysis
  experience;
- review the exact source digest, source manifest, rule-by-rule objectives,
  prohibited-surface report, and generated corpus;
- assess false-positive risk, blind spots, misleading names or metadata,
  licensing and ownership, and whether each fixture actually supports its
  stated claim;
- record approve, revise, or reject for every rule and identify every open
  critical, high, or unknown issue; and
- deliver a signed or hash-addressed attributable report bound to the exact
  ruleset and review-scope digests.

Any open critical, high, unknown, ownership, or prohibited-surface issue keeps
the ruleset unavailable. AI assistance may help prepare evidence but may not
serve as the independent human reviewer.

### Change and lifecycle boundary

- Any rule-source change creates a new source identity and invalidates the
  prior compiled identity, compatibility evidence, and review result.
- Expiry, revocation, rollback floor, signing, and publication remain part of
  later ADR-0103 stages. They are not satisfied by source review.
- A rule can be withdrawn independently of the engine. Removed or revoked
  rules remain visible in bounded historical evidence but cannot enter a new
  bundle.

## Options Considered

### Option A — Original minimal project ruleset (recommended)

This gives Impresari complete ownership and an explainable first claim while
keeping licensing, parser, review, false-positive, and update risk bounded. It
starts narrower than mature community collections and requires deliberate
future expansion.

### Option B — Curated third-party or community rules

This could add coverage faster, but it imports licensing, provenance,
maintenance, false-positive, and supply-chain obligations before the first
production lifecycle is proven. Defer it to a later feed-admission ADR.

### Option C — User- or repository-supplied rules

This is incompatible with the current trust boundary because mutable input
would become executable analyzer policy. It remains prohibited.

### Option D — Ship the engine without a production ruleset

This preserves the candidate work but does not deliver a usable scanner. It is
the fail-closed state while this decision or its review is pending.

## Consequences

- The first production claim will be intentionally narrow and auditable.
- A reviewer evaluates a small ruleset and generated corpus rather than the
  entire product or unknown third-party signatures.
- Detection breadth will initially lag large community feeds. Impresari must
  say that plainly and must not market absence of matches as safety.
- Rule maintenance becomes a permanent security and release responsibility.
- The engine candidate remains useful supply-chain evidence while the ruleset
  is pending, but no analyzer product is activated.

## Approval And Activation Gate

Founder approval of this ADR authorized contract and source authorship for the
original minimal ruleset and its generated test corpus. It did not
authorize compiler or analyzer execution, artifact upload, signing,
attestation, publication, installation, repository scanning, credentials,
production admission, IAR-2, third-party rules, or real malware samples.

Independent review is a genuine manual gate. Until an eligible reviewer binds
an acceptable report to the exact source and scope identities, the ruleset
must remain `missing_evidence` or otherwise unavailable. A later decision must
separately authorize compilation and candidate retention after the review
contract and source are frozen.

The founder deferred reviewer engagement on 2026-08-31. Ordinary roadmap work
may continue, but the deferral grants no authority to compile, retain, sign,
publish, activate, or scan with these rules. The exact review scope must be
refreshed if its source identity changes, and an acceptable independent report
remains mandatory before the compilation gate can open.

## Option A Implementation Evidence

The approved source-only implementation freezes three original
Impresari-owned observation rules and twelve generated, non-malicious fixtures.
Each rule has one positive, near-miss, benign-collision, and bounded-mutation
fixture. The closed checker parses only the permitted literal/hex subset and
evaluates those declared fixture expectations without invoking YARA-X.

- Source: `rules/yara-x/production-v1-candidate.yar`
- Source SHA-256: `2c793693e57d6e2f25cf5a38a38033b32afcf05bc56cc6deb088601d140fa9f7`
- Profile SHA-256: `9dbb28f52510e63e18834f0ece42a807b4ae03a9fff13fa97f954492a4631d62`
- Rules: `3`
- Generated fixtures: `12`
- State: `source_candidate_review_required`

The implementation compiles or executes no rules, retains or uploads no
artifact, scans no repository, and adds no production, detection-quality,
safety, malware-free, IAR-2, signing, publication, or activation claim. The
exact review scope remains a manual gate and cannot be satisfied by AI review.
