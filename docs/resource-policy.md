# Initial Resource Policy

`profiles/v1/conservative-local-v1.json` is the sole accepted pre-release
resource profile. Its SHA-256 file digest is published beside it and is the
`policy_profile` value used by v1 requests and decisions.

The values are conservative safety ceilings, not performance claims. They use
one portable baseline across macOS, Linux, and Windows until evaluation supports
platform-specific profiles. Defaults bound ordinary local work; absolute maxima
limit output amplification, filesystem traversal, oversized inputs, elapsed
work, memory, cache growth, and audit retention independently.

Callers may request values from the inclusive minimum through maximum. Missing
values receive the default. Values below the minimum, above the maximum, not
expressible as canonical unsigned decimal strings, or internally inconsistent
are rejected before workspace enumeration or content access. Only the engine
owner may select or replace the active profile. Changing any byte creates a new
fingerprint and requires contract, security, L05 test, and evaluation review.

The first benchmark campaign must test these limits on each Tier A platform.
Until then, the profile is an implementation gate and defensive baseline; it
must not be represented as evidence that every maximum is attainable on every
machine.
