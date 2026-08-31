# YARA-X NDJSON Adapter Evidence

- Date: 2026-08-31
- Decision: [ADR-0100](../decisions/0100-freeze-yara-x-ndjson-adapter-before-runner-linkage.md)
- Profile: `yara-x-ndjson-adapter-v1`
- Profile SHA-256: `e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c`
- Scope: pure offline original-synthetic parser only

## Implemented Boundary

`context-yara-x-adapter` accepts one in-memory record of at most 131,072 bytes
and separately supplied source-free control metadata. It requires exactly one
UTF-8 JSON object followed by LF, denies unknown and duplicate fields, validates
the exact staged path without emitting it, accepts only frozen ASCII identifiers
and the exact positive ` ... N more bytes` marker, checks every derived range,
and emits observations in canonical order.

The deterministic result binds the workspace snapshot, manifest, artifact,
artifact length, executable, compiled ruleset, profile, and completion time.
It contains no staged path, source bytes, matched bytes, raw output, raw error,
command, arguments, rule source, network destination, credential, or added
authority. Empty rules are a complete no-match result. Every failure is one
stable source-free category with no partial result.

## Evidence

- Closed registry schemas:
  `yara-x-ndjson-adapter-profile`, `yara-x-ndjson-adapter-control`, and
  `yara-x-normalized-result`.
- Exact profile sidecar and byte-identical valid profile fixture.
- Reviewed provenance with exact SHA-256 for every original-synthetic JSON and
  NDJSON fixture.
- Unit coverage for match/no-match, path and source omission, deterministic
  ordering and identity, duplicate and unknown fields, LF framing, UTF-8, path
  substitution, identifier grammar, marker grammar, checked range escape, and
  byte mutations without panic.
- Repository guard freezes the four-dependency surface and rejects filesystem,
  process, network, environment, clock, credential, and embedded-file tokens
  from production parser code.

## Non-Claims

This evidence does not show that YARA-X executed or was OS-confined. The parser
is not linked to the Analyzer Runner, does not consume repository-derived input,
and admits no executable or ruleset. Production, IAR-2, detection quality,
safety, and malware-free claims remain false. The earlier hosted compatibility
run remains separate evidence and cannot be relabeled as parser execution.
