# ADR-0100: Freeze The YARA-X NDJSON Adapter Before Runner Linkage

- Status: Implemented for pure original-synthetic parsing; execution and production remain gated
- Date: 2026-08-31
- Decider: Aaron Boldt through the standing accepted-roadmap directive and ADR-0099 activation gate
- Related: ADR-0013, ADR-0074, ADR-0082, ADR-0095, ADR-0098, ADR-0099

## Context

ADR-0099 proves that the exact narrowed YARA-X v1.20.0 candidate and
Impresari-owned synthetic rules produce the frozen one-line NDJSON shape inside
the Linux candidate boundary. It deliberately does not retain raw output,
implement a production parser, admit an executable or ruleset, connect the
parser to the Analyzer Runner, or scan repository content.

The next lowest-authority checkpoint is to parse hostile vendor-shaped output
as untrusted data without executing the vendor. Combining that work with
runner linkage or production artifact admission would make parser failures,
confinement failures, and supply-chain failures indistinguishable.

## Decision

Freeze `yara-x-ndjson-adapter-v1` as a pure all-or-nothing transformation from
one bounded original-synthetic YARA-X NDJSON record plus exact control metadata
to a path-free, source-free normalized result.

The input must be at most 131,072 bytes and contain exactly one UTF-8 JSON
object followed by one LF. BOMs, CRLF, leading or trailing whitespace, extra
lines, empty input, unknown fields, duplicate fields, non-integer numbers, and
noncanonical marker lengths fail closed. The top-level object contains only
`path` and `rules`; the path must exactly equal the separately supplied staged
path and is never emitted.

Each rule contains only `identifier`, `namespace`, `strings`, and `tags`. Each
string contains only `identifier`, `match`, and `offset`. Identifiers and tags
use the frozen ASCII identifier grammar. Tags and observations are emitted in
canonical order and duplicates fail. The parser derives a positive byte length
only from the exact zero-byte marker ` ... N more bytes`, checks offset plus
length against the separately supplied artifact length, and enforces every
ADR-0098 observation, range, tag, identifier, and total-output ceiling.

The parser receives exact workspace snapshot, manifest, artifact, executable,
ruleset, profile, and completion-time identities as control data. It binds
every normalized observation to those identities and emits no path, source
bytes, matched bytes, raw output, raw error, command, argument, rule source,
network destination, credential, or authority. Empty rules are a successful
complete no-match result; any malformed or partial state produces one stable
source-free error and no partial result.

Implementation is a new pure Rust library with no filesystem, process, network,
environment, clock, or credential capability. Its complete positive and
negative corpus is committed original-synthetic data with provenance and exact
digests. Tests must cover valid match/no-match records, all frozen pattern
shapes, duplicate and unknown fields, framing, UTF-8, path substitution,
marker grammar, integer overflow, range escape, excessive arrays, ordering,
and deterministic normalization.

## Consequences

- Vendor output becomes an explicitly untrusted parser boundary before it can
  enter ADR-0013 normalization.
- Parser correctness can be reviewed independently from OS confinement,
  process launch, and artifact signing.
- The exact staged path is validated but cannot leak into a normalized result.
- No successful parse means YARA-X ran; execution is a separate authenticated
  runner fact.
- The production artifact pipeline, runner linkage, repository-derived input,
  IAR-2, and detection-quality work remain separate gates.

## Alternatives

- Parse NDJSON in the ordinary Context process after runner linkage: rejected
  because transport and parser authority would be introduced together.
- Reuse the legacy ADR-0095 fixture parser unchanged: rejected because its
  engine identity and synthetic result shape predate the exact YARA-X wire
  contract.
- Retain raw NDJSON for later debugging: rejected because the vendor record can
  contain path and matched-data fields outside the admitted contract.
- Normalize partially valid rules: rejected because partial parsing can
  launder missing coverage or malformed output.

## Activation Gate

This ADR authorizes only the pure library, schemas, profile, original-synthetic
fixtures, provenance, and offline tests described above. It does not authorize
downloading or executing YARA-X, invoking a process, reading repository-derived
analyzer input, linking the adapter to the Analyzer Runner, admitting or
signing an executable or ruleset, using network or credentials, or claiming
reproducibility, IAR-2, production, detection quality, safety, or malware-free
status.

After implementation, the next decision must independently choose between the
production artifact pipeline and synthetic runner-to-adapter envelope linkage.
Neither may carry repository-derived bytes until all of its own gates pass.

## Implementation Evidence

The `context-yara-x-adapter` crate implements the frozen transformation as an
in-memory-only library. The profile digest is
`sha256:e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c`.
Closed schemas cover the profile, separate control metadata, and normalized
result. A provenance record binds every committed original-synthetic positive
and negative fixture by exact digest.

Unit tests prove the valid match and no-match paths, deterministic ordering and
identity, closed-field and duplicate rejection, framing and UTF-8 failures,
path mismatch, identifier, marker, and range failures, and a byte-mutation
corpus without panics. Repository checks freeze the dependency list and reject
filesystem, process, network, environment, clock, credential, and embedded-file
capability tokens from production parser code. The result schema fixes every
execution, confinement, production, IAR-2, safety, and authority claim to
false.
