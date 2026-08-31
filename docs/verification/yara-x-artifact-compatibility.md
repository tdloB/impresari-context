# YARA-X Artifact Compatibility Evidence

- Date: 2026-08-31
- Decision: ADR-0099
- Engine: YARA-X v1.20.0
- Input: Impresari-owned original synthetic only
- Result: Candidate passed
- Production admitted: No
- IAR-2 admitted: No

## Frozen Inputs

The manual workflow at commit
`f73803d2e37cc261a414c4b23ec52ce316df7968` downloaded the exact public
Impresari source root `9a69e1e0d3fff58676ef91f33b8dd9f6b8330ae7`
without checking out the repository. That archive was 27,883,271 bytes with
SHA-256
`ad141c2749ab363207f15830a48bb259b2338795843da11659962841325fea1b`.

The build then enforced these existing ADR-0099 identities:

- upstream YARA-X commit:
  `60ad06971467029e77967e59d580cbbe85a1474d`;
- upstream archive SHA-256:
  `8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee`;
- Impresari patch SHA-256:
  `b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd`;
- patched CLI manifest SHA-256:
  `a141a064f49eedc1d2bd079e95f1ce187d7d9fba845f6e801ed7c44eaa378402`;
- compatibility profile SHA-256:
  `ea2abe8460a1faab60b4ab2d854e48bdd45f1998106cd5e62229153155d254a8`;
- rule-source SHA-256:
  `5379d03476eebf9c06379ad8d791d5ff1879c331300869d3eaf54c0e578c812b`.

The build used Rust `1.93.0`, locked dependencies, the frozen feature graph,
the `release-lto` profile, static CRT flags, and the Pulley interpreter feature.

## Hosted Evidence

[GitHub Actions run 33406541396](https://github.com/tdloB/impresari-context/actions/runs/33406541396),
job `99535422988`, completed successfully in 8 minutes 42 seconds on:

- GitHub runner image `ubuntu-24.04`, version `20260823.283.1`;
- kernel `6.17.0-1022-azure`;
- architecture `x86_64`;
- Landlock ABI `7`.

The Linux composite IAR-1B feasibility gate returned `candidate_passed` before
the analyzer corpus ran. All five frozen synthetic cases passed. The bounded
compatibility line recorded:

- executable SHA-256:
  `f238098b1351303ad53cd240ffe1b591f4a0d7f625ac26ba9d22a7ac1ab3b718`;
- compiled-rules SHA-256:
  `010ea0e190fa5bf8f07fa08b6cb594ad154fa352fa53931e9eb85e1bf5847f35`;
- cases: `5`;
- result: `candidate_passed`;
- production: `false`;
- IAR-2: `false`.

The separate `always()` cleanup step also passed after restoring owner-only
write permission inside the three exact disposable roots and removing them.
The workflow uploaded no source, executable, compiled rules, raw output, or
receipt artifact.

## Interpretation

This run proves compatibility only for the exact hosted environment and frozen
synthetic corpus. The executable digest is a per-run evidence identity, not a
reproducible or admitted release artifact. The mutable hosted build image still
prevents a production reproducibility claim.

No repository content was scanned. No credential was read or conveyed to the
build or analyzer. No live adapter, signature, release asset, ruleset admission,
detection-quality claim, malware-free claim, production support, or IAR-2
authority follows from this result.

The next checkpoint may freeze and implement a synthetic-only live NDJSON
adapter contract and design a separate production artifact pipeline. Actual
repository-derived input and production analyzer execution remain separately
gated.
