# HRA-2 Compose Privilege Rule Corpus

- Scope: the second ADR-0073 HRA-2 execution-surface increment.
- Candidate basenames: `compose.yaml`, `compose.yml`, `docker-compose.yaml`, and
  `docker-compose.yml` only.
- Parser boundary: bounded UTF-8 line inspection; no general YAML or Compose
  semantic evaluation, process, container engine, analyzer, or network access.

## Admitted observation

One finding is emitted only for the exact line `privileged: true` at four-space
indentation while a simple two-space service mapping is active beneath exactly
one top-level `services:` key. The finding is a `medium`, `confirmed`, observed
`privilege` fact. Its evidence contains exactly the `privileged` key token.

This fact means only that the canonical declaration exists. It does not prove
that the Compose document is complete, that a profile selects the service, that
the container will run, or that the declaration is malicious or safe.

## Explicit unsupported cases

Tabs, non-UTF-8 bytes, YAML block scalars, duplicate top-level services keys,
missing canonical services layout, and any alternative `privileged:` scalar
syntax are excluded. Anchors, aliases, merges, flow mappings, extension fields,
generated overrides, nested Compose features, and full YAML semantics are not
interpreted.

## False-positive review

Tests prove that top-level lookalikes, deeper label-like text, comments,
`privileged: false`, block-scalar text, and noncanonical documents do not emit
the admitted finding. Unsupported ambiguity is visible instead of falling back
to substring matching.
