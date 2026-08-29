# HRA-2 npm Lifecycle Rule Corpus

- Scope: the first ADR-0073 HRA-2 execution-surface increment.
- Input: exact current bytes for an HRA-1-inventoried file whose basename is
  exactly `package.json`.
- Parser boundary: strict JSON validation plus a bounded direct-object/key
  scanner; no package manager, shell, interpreter, repository process, or
  network access.

## Closed rules

The only admitted keys are `preinstall`, `install`, `postinstall`, `prepare`,
`prepublish`, `prepublishOnly`, `publish`, and `postpublish`. A key produces an
observation only when it is a direct member of the top-level `scripts` object
and its value is a JSON string.

Each observation is an informational, confirmed `lifecycle_hook` fact. Its
evidence span and excerpt contain exactly the JSON key token. The repository-
controlled value is neither interpreted nor retained in the observation
bundle. The fact therefore means only “this exact lifecycle declaration
exists,” never that the value is safe, malicious, executable on this host, or
authorized to run.

## Explicit unsupported cases

- invalid JSON;
- a non-object root or `scripts` value;
- a non-string admitted lifecycle value;
- escaped, duplicated, or otherwise ambiguous admitted keys; and
- finding-limit truncation.

These cases produce a content-free exclusion or fail stale identity checks;
they never fall back to a heuristic match.

## False-positive review

Tests prove that ordinary script keys, same-named keys outside `scripts`, and
nested metadata do not produce findings. Exact evidence is schema-valid and
recoverable to the current snapshot/hash/span. Script values are absent from
serialized observations. This corpus does not inspect transitive package
behavior, shell semantics, package-manager configuration, dependencies, or
generated manifests.
