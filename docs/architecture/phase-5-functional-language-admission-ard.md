# Phase 5 Functional-Language Admission ARD

## Scope

Founder demand admits Scala, Elixir, Clojure, and Haskell under ADR-0040's
per-language contract. The existing isolated worker is retained; no protocol,
storage, service, or authority expansion occurs.

## Security posture

Each grammar is exact-version pinned, compiled only into the capability-reduced
worker, and rejected on identity mismatch. Resolver rules emit only directly
tested syntax facts. Parser errors stay explicit; all compiler, package,
macro, type, build, and runtime semantics fail closed by omission.

## Verification

Focused language fixtures, negative-boundary tests, independent manifest
inventory assertion, dependency policy, regenerated SBOM, complete local gate,
and hosted Tier-A matrix are required before a public supported claim.
