# YARA-X Synthetic Runner Envelope Evidence

- Date: 2026-08-31
- Decision: [ADR-0101](../decisions/0101-prove-synthetic-runner-to-adapter-envelope-before-artifact-admission.md)
- Profile: `yara-x-synthetic-runner-envelope-v1`
- Profile SHA-256: `356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b`
- State: hosted isolated synthetic matrix passed

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

## Hosted Evidence

GitHub Actions run
[`33419412353`](https://github.com/tdloB/impresari-context/actions/runs/33419412353),
job
[`99577842304`](https://github.com/tdloB/impresari-context/actions/runs/33419412353/job/99577842304),
passed on 2026-08-31 against source commit
`b63a2fdeff39e27e9ec3149fe0e8c2300894cadb`. The job used GitHub-hosted
Ubuntu 24.04.4 x86-64, runner `2.337.0`, and runner image
`ubuntu-24.04` version `20260823.283.1`.

The empty-workspace job downloaded the exact public source archive at commit
`9a6bac4f8bda10b4b08ef3429587b9ae7f8bd1ce`, required length `27920771`,
and verified SHA-256
`9dd03466b46fc1a882e39c2ce99c2d6ac0db18431a5c93ad2e24ab40922c0ef2`
before extraction. It built the locked envelope components with Rust `1.98.0`,
ran both closed cases inside the delegated Linux cgroup/Landlock/seccomp
boundary, and emitted only this bounded summary:

```text
YARA-X synthetic runner envelope passed: cases=2 os_confined=true yara_x_executed=false production_admitted=false
```

The separate final step confirmed removal of the runtime directory and absence
of envelope binaries under the checkout target paths. No artifact, cache,
fixture output, repository-derived analyzer input, or receipt was uploaded.

## Non-Claims

YARA-X did not run. No analyzer, repository content, credential, malware,
network destination, production artifact, or admitted ruleset enters this
checkpoint. The passing result establishes only the synthetic transport,
composition, confinement, and cleanup envelope. It does not establish IAR-2,
detection quality, safety, or malware-free status.
