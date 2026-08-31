# YARA-X Synthetic Runner Envelope Evidence

- Date: 2026-08-31
- Decision: [ADR-0101](../decisions/0101-prove-synthetic-runner-to-adapter-envelope-before-artifact-admission.md)
- Profile: `yara-x-synthetic-runner-envelope-v1`
- Profile SHA-256: `356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b`
- State: implementation complete; hosted isolated matrix pending

## Implemented Boundary

The Impresari-owned emitter embeds exactly one valid-match and one
valid-no-match YARA-X-shaped record. It accepts only the two closed case IDs and
has no file, repository, rule, network, credential, or environment-derived
input. The coordinator captures at most 131,072 stdout bytes, requires exact
empty emitter stderr, binds both executable identities, validates the expected
output length and digest, and passes the bytes directly to the pure ADR-0100
parser without writing raw output to storage.

The existing Linux launcher places only the emitter in a fresh delegated
cgroup and applies the already admitted Landlock/seccomp boundary. The
coordinator returns a receipt only after the exact job directory and empty
cgroup leaf are removed. Failures return one source-free category and no
partial normalized result.

## Local Evidence

- Three pure unit tests cover both complete cases, capture mutation, missing
  cleanup, unknown cases, and authority overclaim.
- Closed registry schemas cover profile, control, and receipt.
- Provenance binds all original-synthetic fixtures by exact digest.
- The repository checker freezes profile, limits, cases, nonclaims, emitter
  capability tokens, runner launch-site count, and the closed launcher mode.
- Local tests deliberately do not execute the emitter because macOS is not an
  admitted backend for this checkpoint.

## Non-Claims

YARA-X did not run. No analyzer, repository content, credential, malware,
network destination, production artifact, or admitted ruleset enters this
checkpoint. The result does not establish IAR-2, detection quality, safety, or
malware-free status. Those claims remain false even after the hosted synthetic
matrix passes.
