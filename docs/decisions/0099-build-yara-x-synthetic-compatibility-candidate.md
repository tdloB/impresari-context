# ADR-0099: Build A Module-Free YARA-X Synthetic Compatibility Candidate

- Status: Accepted for bounded synthetic implementation
- Date: 2026-08-31
- Decider: Aaron Boldt through explicit checkpoint authorization
- Related: ADR-0074, ADR-0077, ADR-0082, ADR-0089, ADR-0097, ADR-0098

## Context

ADR-0098 freezes YARA-X v1.20.0 at commit
`60ad06971467029e77967e59d580cbbe85a1474d`, but deliberately contains no
source archive, executable, rule, or live compatibility evidence. The next
roadmap checkpoint must learn whether an Impresari-owned module-free ruleset
compiles and produces the frozen bounded NDJSON behavior before any production
artifact, live parser, repository input, or IAR-2 decision is considered.

The immutable commit archive has SHA-256
`8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee`.
The stock CLI enables every default module and parallel compilation. Those
features conflict with the frozen no-module surface, expand the optional
cryptographic and hostile-format dependency graph, and add uncontrolled build
threads. The pinned lockfile also contains vulnerable `crossbeam-epoch 0.9.18`
and unsound `memmap2 0.9.10` releases with compatible patched versions.

RustSec reports three other package-level advisories that remain in the full
workspace lockfile. Their affected code is absent from the selected feature
graph: RSA is optional and module-only; the Wasmtime filesystem issue is in
`wasmtime-wasi`/`cap-std`, neither of which is present; and the multi-engine
Wasmtime issue requires two engines while YARA-X creates one. The upstream
YARA-X maintainer independently records the same single-engine disposition for
the latter issue.

## Decision

Create `yara-x-artifact-compatibility-v1` as an expiring, Linux x86-64,
synthetic-only candidate.

Acquire only the immutable commit archive. Apply one content-addressed patch
that disables YARA-X default modules, removes parallel compilation from the
CLI, and makes only two lockfile updates:

- `crossbeam-epoch 0.9.18` to `0.9.20`;
- `memmap2 0.9.10` to `0.9.11`.

Build `yara-x-cli` with Rust `1.93.0`, Cargo `--frozen --locked`, the
`release-lto` profile, static CRT flags, and the Pulley interpreter feature.
The build may fetch only the exact source archive and locked crates on an
ephemeral GitHub-hosted runner without user or repository credentials. Source,
dependencies, executable, compiled rules, output, and receipts are not
uploaded or retained.

Create one Apache-2.0 Impresari-owned ruleset containing literal, hex, and wide
synthetic sentinel rules only. Compile it outside the scan job from exact
reviewed source. Run positive, negative, and bounded failure fixtures only on
generated synthetic bytes. Every `yr scan` process must start atomically in a
fresh delegated cgroup, receive read/execute access only to its staged job,
have no writable filesystem or network, close unrelated descriptors, use a
closed environment and argument vector, and be killed and cleaned as one
cgroup. Reuse the existing single transient CI delegation launch site; add no
new `sudo systemd-run` location.

The candidate may record build, executable, compiled-rules, result, and hosted
isolation evidence. It cannot admit or sign the executable or ruleset, scan
repository content, implement the production parser, claim detection quality,
open IAR-2, or activate production support.

## Dependency Dispositions

- `RUSTSEC-2023-0071`: ignored only for this candidate after an exact feature
  graph proves `rsa` absent. Any graph change fails closed.
- `RUSTSEC-2026-0222`: ignored only while the one-engine YARA-X invariant and
  upstream disposition remain current.
- `RUSTSEC-2026-0269`: ignored only while `wasmtime-wasi` and `cap-std` remain
  absent from the selected graph.
- `RUSTSEC-2025-0141`: retained as an informational, unmaintained dependency
  limitation; no production admission is possible without a later decision.
- yanked `spin 0.9.8`: permitted in the full workspace lock only while absent
  from the selected graph.

Any new advisory, graph reachability, source/patch/lock drift, unexpected
network-capable dependency, output-contract drift, isolation failure, expiry,
or cleanup failure withdraws the candidate.

## Consequences

- The test answers whether the exact narrowed engine and original-synthetic
  rules behave as the frozen contract expects.
- The separately pinned patch makes the candidate an Impresari build of the
  exact upstream source, not an unmodified upstream binary.
- A mutable GitHub runner image prevents a reproducible or production artifact
  claim even though source, patch, toolchain, and dependency resolution are
  locked.
- Artifact signing, SBOM/provenance publication, reproducibility, the live
  adapter, independent review, production IAR-1B publication, and IAR-2 remain
  later gates.

## Alternatives

- Build the stock default-module CLI: rejected because it contradicts the
  frozen module-free contract and expands the reachable attack surface.
- Upgrade YARA-X or Wasmtime beyond v1.20.0: deferred because it changes the
  selected engine source rather than applying bounded compatible fixes.
- Suppress every lockfile advisory without feature analysis: rejected.
- Run the compatibility corpus on the ordinary host: rejected.
- Upload the binary or receipt as a CI artifact: rejected because this
  checkpoint is evidence-only and not distribution admission.

## Activation Gate

The next decision must separately review the hosted evidence and determine
whether to implement the live NDJSON adapter and production artifact pipeline.
Repository-derived scan input, signatures, publication, production admission,
IAR-2, detection, safety, and malware-free claims remain closed.

## Sources

- [YARA-X v1.20.0](https://github.com/VirusTotal/yara-x/releases/tag/v1.20.0)
- [YARA-X issue 726 Wasmtime disposition](https://github.com/VirusTotal/yara-x/issues/726)
- [Wasmtime multi-engine advisory](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-hgjw-h833-99q9)
- [Wasmtime WASI filesystem advisory](https://github.com/bytecodealliance/wasmtime/security/advisories/GHSA-vqjp-4c8c-hfgg)
