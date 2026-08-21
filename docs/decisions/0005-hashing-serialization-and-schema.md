# ADR-0005: Hashing, serialization, and schema contracts

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: Workspace snapshots, evidence, packets, policy decisions, cache keys,
  audit references, and public structured interfaces

## Context

Evidence must bind to exact source, packets must detect modification, and
deterministic operations must reproduce identities across clients. Ordinary JSON
object ordering is insufficient for hashing, paths are not universally valid
Unicode, and line/column conventions can silently produce incorrect evidence.

The format must remain easy for non-Rust consumers to validate without making a
binary implementation detail the public contract.

## Decision

### Public serialization

- Use UTF-8 JSON for public requests, responses, packets, policies, validation
  results, audit exports, and conformance fixtures.
- Define schemas using JSON Schema Draft 2020-12.
- Every top-level object includes `schema_name` and semantic `schema_version`.
- Reject duplicate object keys, invalid UTF-8, non-finite numbers, unexpected
  type coercions, and schema-invalid required fields.
- Represent identifiers, hashes, timestamps, large counters, and byte offsets
  as constrained strings or schema-bounded integers that retain cross-language
  precision. Do not rely on arbitrary JSON floating-point values.
- Timestamps use RFC 3339 UTC strings with an explicit `Z` and documented
  precision.

### Canonical JSON

Use the JSON Canonicalization Scheme in **RFC 8785 (JCS)** whenever a JSON
object contributes to an identity or integrity hash.

- Objects are schema-validated before canonicalization.
- Hashing uses canonical UTF-8 bytes, not a language runtime's normal serializer
  output.
- Canonical bytes are an internal/integrity representation; human-formatted JSON
  may use whitespace but must canonicalize to the same content.
- A conformance corpus covers Unicode, escaping, ordering, numbers, and rejection
  cases across implementations.

### Hash algorithm and domain separation

Use **SHA-256** for source content, workspace snapshots, indexes, evidence,
claims, packets, policies, and exported artifact digests.

Each structured identity hashes a domain-separated envelope:

```text
"impresari-context" NUL
object-kind NUL
schema-version NUL
canonical-payload-bytes
```

Identifiers use an explicit form such as `sha256:<lowercase-hex>` and never rely
on a bare digest whose algorithm or object kind is unknown. The exact envelope
bytes and object-kind registry will be published as conformance fixtures before
implementation.

Raw file content hashes cover the exact raw bytes, including BOM and line-ending
bytes. Text decoding does not change content identity.

### Snapshot identity

A workspace snapshot identity covers at minimum:

- canonical workspace identity without publishing unauthorized absolute paths;
- sorted eligible artifact records containing lossless relative path identity,
  file kind, content hash, and policy-relevant metadata;
- discovery/ignore policy fingerprint;
- hash/path contract version;
- engine and relevant index/resolver versions;
- declared partial/skipped state where it affects completeness.

Time of creation, performance measurements, and request IDs do not alter the
content snapshot identity; they belong to a snapshot record around that identity.

### Path representation

Paths have separate display and identity forms.

- Evidence and packets use workspace-relative paths only.
- `display_path` is a safely escaped human representation and is not used as the
  sole identity.
- `path_identity` records platform family, unit encoding, and a lossless
  base64url representation of the relative native path units when necessary.
- Unix path identity uses raw path bytes.
- Windows path identity uses the lossless native path-unit representation chosen
  in the implementation contract; it must not be produced by lossy UTF-8
  conversion or case folding.
- Paths are not Unicode-normalized for identity. Collisions or ambiguous aliases
  fail visibly.
- The path identity contract receives dedicated cross-platform fixtures before
  implementation.

### Source spans

- Authoritative spans are zero-based, half-open raw byte offsets:
  `[start_byte, end_byte)` in the exact content identified by `content_hash`.
- Derived human locations are one-based `start_line`, `start_column`, `end_line`,
  and `end_column`, with columns counted in Unicode scalar values after the
  declared supported decoding.
- A span cannot be exact unless the raw byte offsets, content hash, and decoding
  status are valid.
- Files unsupported for decoding may have file-level exact evidence but no text
  span/excerpt claim.
- Newline and BOM behavior is fixed in conformance fixtures.

### Versioning and compatibility

- Schemas use semantic versions. A breaking field/meaning change increments the
  major schema version.
- Readers reject unknown major versions. Minor additions are accepted only where
  schemas explicitly allow them and semantics remain safe.
- Snapshot/packet IDs include the applicable schema/algorithm contract, so a
  material canonicalization change creates a new identity.
- Cache formats may differ internally but cannot change public meanings.

## Rationale

SHA-256 is widely implemented, interoperable, and already used in the
architecture examples. RFC 8785 supplies a published JSON canonicalization
scheme, while JSON Schema 2020-12 provides language-neutral structural
validation. Raw-byte source hashing and byte-authoritative spans avoid newline,
Unicode, and decoder ambiguity.

## Consequences

### Positive

- Non-Rust clients can validate schemas and reproduce hashes.
- Exact evidence remains bound to raw content rather than display text.
- Packet tampering and stale substitution are detectable.
- Human formatting does not alter structured identity.
- Cross-platform path limitations are explicit rather than hidden by lossy
  strings.

### Costs

- JCS implementation and cross-language number rules require conformance tests.
- Lossless native path representation is more complex than a single path string.
- JSON is more verbose than binary formats.
- SHA-256 may be slower than newer non-cryptographic/content hashes, though
  performance must be measured before considering a change.

## Alternatives Considered

### BLAKE3

Attractive for speed and tree hashing, but rejected for the initial public
contract because SHA-256 has broader native interoperability and simpler
third-party verification. BLAKE3 may be evaluated as an internal accelerator
only if it cannot create identity divergence.

### Normal JSON serialization without canonicalization

Rejected because member ordering, number formatting, and escaping can produce
different hashes for the same logical object.

### CBOR or Protocol Buffers as the only public format

Deferred. Both can be efficient, but JSON/JSON Schema is easier for initial CLI,
SDK, MCP, and audit interoperability. A binary transport may be added later
without changing canonical semantics.

### Git object IDs as source identity

Rejected because the engine must support dirty working trees and non-Git
workspaces, and Git's object model does not include all engine discovery/policy
inputs.

### Line/column-only evidence

Rejected because line endings, encodings, and concurrent edits can make it
ambiguous. Byte ranges plus content hash are authoritative.

## Verification

- Publish canonicalization and digest golden vectors.
- Property-test parse/canonicalize/hash stability and rejection behavior.
- Cross-check selected vectors with an independent RFC 8785 implementation.
- Test UTF-8, BOM, CRLF/LF, combining characters, bidi controls, invalid UTF-8,
  long paths, case collisions, and native non-Unicode paths.
- Mutating one covered field or source byte must change the applicable identity.
- Pretty-printing alone must not change canonical identity.

## Official References

- [RFC 8785 — JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785)
- [JSON Schema Draft 2020-12](https://json-schema.org/draft/2020-12)
- [JSON Schema specification](https://json-schema.org/specification)

## Review Triggers

Review if interoperability tests cannot reproduce JCS identities, JSON size
materially prevents budget goals, SHA-256 is no longer suitable, public clients
require streaming/binary contracts, or supported platform path semantics cannot
be represented losslessly.
