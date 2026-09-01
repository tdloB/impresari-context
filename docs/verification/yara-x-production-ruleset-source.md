# YARA-X Production Ruleset Source Verification

- Decision: ADR-0106 Option A
- Date: 2026-08-31
- State: `source_candidate_review_required`
- Independent human review: pending

## Frozen identities

- Rule source: `rules/yara-x/production-v1-candidate.yar`
- Rule-source SHA-256: `2c793693e57d6e2f25cf5a38a38033b32afcf05bc56cc6deb088601d140fa9f7`
- Profile: `profiles/v1/yara-x-production-ruleset-v1.json`
- Profile SHA-256: `9dbb28f52510e63e18834f0ece42a807b4ae03a9fff13fa97f954492a4631d62`
- Rules: 3
- Original generated fixtures: 12

## Source-only checks

`scripts/check-yara-x-production-ruleset.rb` verifies exact identities,
ownership and licensing, the closed literal/hex language, declared metadata,
known false positives and blind spots, the four-role fixture matrix, exact
fixture provenance, and the independent-review scope. Its bounded evaluator
checks only the declared literal/hex expectations; it does not load, compile,
or execute YARA-X.

The source receipt and review scope are registered contract fixtures. Negative
fixtures prove that an AI review and a compiled-source overclaim fail schema
validation.

## Non-claims

No compiler or analyzer was executed. No engine, compiled rule, or other
artifact was retained, uploaded, signed, attested, published, or installed. No
repository content, credential, third-party rule, malware sample, or live
network destination entered the corpus. This evidence does not admit the
ruleset or production scanner, authorize repository scanning, open IAR-2, or
claim detection quality, safety, or malware-free status.

## Remaining manual gate

An attributable human independent of the rule authorship, with practical YARA
or malware-analysis experience, must review the exact source and scope
digests, disclose conflicts, disposition every rule, and leave zero open
critical, high, or unknown issues. Any source change invalidates that review.
