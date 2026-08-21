# ADR-0009: Path and structured identity encoding

- Status: Accepted for contract implementation
- Date: 2026-08-21
- Scope: Native paths, workspace identity, canonical JSON values, domain-separated hashes, and conformance vectors

## Context

ADR-0005 selects raw-byte hashing, RFC 8785 canonical JSON, SHA-256, and lossless
platform-aware paths, but intentionally left several byte-level choices to the
implementation contract. Those choices affect permanent public identities and
must be decided before schemas or code rather than inferred independently by
Rust and consumer adapters.

## Decision

### Relative path identity

Every public artifact path has a safe `display_path` plus this identity payload:

```json
{
  "platform_family": "unix",
  "unit_encoding": "unix_bytes",
  "relative_units_base64url": "c3JjL2xpYi5ycw"
}
```

- Base64url uses the RFC 4648 URL-and-filename-safe alphabet with **no padding**.
- Decoders reject padding, non-canonical encodings, invalid alphabet characters,
  and any value that does not round-trip byte-for-byte.
- Unix `unit_encoding` is `unix_bytes`; its decoded value is the exact relative
  native path bytes with `/` separators.
- Windows `unit_encoding` is `windows_utf16le`; its decoded value is the exact
  sequence of native UTF-16 code units, each encoded as two little-endian bytes,
  with `\` separators. Unpaired surrogate code units are preserved and never
  repaired through lossy Unicode conversion.
- Empty paths, absolute/rooted paths, NUL units, `.` components, `..` components,
  alternate Windows separators, drive-relative forms, UNC/device prefixes, and
  trailing separators are invalid artifact identities.
- A decoded identity is authorized only after reconstructing a native relative
  path, joining it beneath the already-authorized root, and performing the live
  containment/link/object checks. Encoding is identity, not authorization.
- `display_path` is derived through an escaping renderer, is never parsed back,
  and never participates as the sole identity.

### Workspace identity

The MVP workspace identity is local and location-bound. It is the structured
SHA-256 identity of:

- platform family;
- the lossless native units of the fully resolved authorized absolute root;
- the applicable path-contract version.

The absolute root units are hash input but are never emitted in a packet, audit
row, or public handle. Moving a workspace creates a new workspace identity.
Opening the same root through an alias must resolve to the same canonical root or
fail visibly; cache identity never substitutes for a live authorization check.
Filesystem object IDs may be recorded for race/alias checks but are not the sole
persistent workspace identity because their portability and stability vary.

### Canonical JSON value profile

Objects hashed through RFC 8785 use the I-JSON-compatible value domain required
by JCS:

- duplicate keys, lone Unicode surrogates in JSON strings, non-finite numbers,
  and values that cannot be represented by the schema are rejected;
- JSON integer fields are limited to the inclusive safe range
  `[-9007199254740991, 9007199254740991]`;
- hashes, byte offsets, sizes, counters, budgets, and timestamps that could exceed
  or ambiguously cross that range use schema-constrained strings;
- unsigned decimal strings use `0` or a non-zero digit followed by digits, with
  no sign, whitespace, exponent, decimal point, or leading zero;
- negative zero is not permitted by an integer schema used for identity.

### Domain-separated envelope

The exact structured-identity preimage is:

```text
UTF8("impresari-context") || 0x00 ||
ASCII(object_kind)        || 0x00 ||
ASCII(schema_version)     || 0x00 ||
JCS_UTF8(validated_payload)
```

`object_kind` and `schema_version` are restricted to ASCII
`[a-z0-9][a-z0-9._-]{0,63}`, which excludes NUL. The initial object-kind registry
is:

- `workspace-root`;
- `workspace-snapshot`;
- `discovery-policy`;
- `artifact-record`;
- `evidence`;
- `claim`;
- `context-packet`;
- `policy-decision`;
- `index-generation`;
- `handoff-export`.

Adding or changing an object kind is a contract change. Digests are rendered as
`sha256:` followed by exactly 64 lowercase hexadecimal characters.

### Schema compatibility

- Schema versions are ASCII semantic versions without build metadata.
- Readers reject unknown major versions before canonicalization or policy use.
- A reader may accept a newer minor version only when the schema explicitly
  permits the added fields, all security-relevant unknown fields are rejected,
  and canonicalization includes every accepted field.
- Writers emit one exact version. They never downgrade by silently deleting
  evidence, policy, conflict, unknown, freshness, or budget fields.
- Each identity-bearing schema names its object kind and whether the complete
  object or a named identity projection is hashed. Identity projections are
  themselves versioned schemas and cannot be assembled ad hoc.

## Conformance Gate

Before identity implementation merges, original project fixtures must publish:

- Unix UTF-8 and non-UTF-8 path units;
- Windows BMP, supplementary-pair, and unpaired-surrogate units;
- slash, backslash, dot, root, drive, UNC/device, NUL, empty, and collision rejection cases;
- padded, non-canonical, and malformed base64url rejection cases;
- JCS ordering, escaping, safe-integer boundary, negative-zero, duplicate-key,
  lone-surrogate, and invalid-number cases;
- one full preimage byte sequence and expected SHA-256 digest for every initial
  object kind;
- cross-language reproduction by the Rust reference and at least one independent
  implementation before a public contract is declared stable.

Fixtures contain synthetic paths and cannot include private workspace names.

## Rationale

Native units preserve paths that are not valid Unicode. Unpadded canonical
base64url is broadly implementable. UTF-16LE makes Windows code-unit bytes
unambiguous across languages. Location-bound opaque workspace identities avoid
publishing absolute paths while making moves and aliases explicit. A constrained
JCS value profile prevents cross-language number and string divergence.

## Consequences

### Positive

- Schema and implementation teams share exact bytes rather than prose guesses.
- Non-Unicode native paths remain representable without lossy display strings.
- Hash preimages and compatibility behavior are independently reproducible.
- Workspace handles do not disclose absolute roots.

### Costs

- Moving a workspace rebuilds its cache and invalidates prior handles.
- Windows adapters must preserve raw UTF-16 code units.
- Public contracts need both ergonomic display paths and opaque identity data.
- Conformance fixtures are mandatory before useful implementation work.

## Alternatives Considered

### UTF-8 paths only

Rejected because supported filesystems can contain names that do not round-trip
through Unicode strings.

### Case-folded or Unicode-normalized identity

Rejected because normalization is filesystem-dependent and can merge distinct
native names or create aliases the OS does not recognize.

### Stable random workspace IDs

Deferred because they require authoritative durable registration state and can
silently rebind after source movement or cache copying.

### File IDs as persistent workspace identity

Rejected as the sole identity because availability and stability differ across
platforms, filesystems, copies, backups, and mounts.

## Verification

- All Conformance Gate fixtures pass on every Tier A platform where applicable.
- Decode/encode is byte-for-byte idempotent for valid native-unit fixtures.
- Every invalid/ambiguous form fails before filesystem access.
- No packet, audit record, error, or diagnostic-safe mode exposes workspace-root units.
- Cross-workspace and moved-workspace handles fail closed.

## Official References

- [RFC 4648 — Base-N Encodings](https://www.rfc-editor.org/rfc/rfc4648)
- [RFC 8785 — JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785)
- [RFC 7493 — I-JSON Message Format](https://www.rfc-editor.org/rfc/rfc7493)

## Review Triggers

Review if a Tier A platform cannot round-trip the selected native units, a
location-independent workspace identity becomes a product requirement, JCS
interoperability fails, or a public binary contract replaces JSON identity.
