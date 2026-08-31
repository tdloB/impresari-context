# YARA Analyzer Admission ARD

- Status: ADR-0102 live synthetic composition implemented; hosted matrix, production artifacts, and IAR-2 remain gated
- Date: 2026-08-31
- Governing PRD: [YARA Analyzer Admission PRD](../product/yara-analyzer-admission-prd.md)
- Decision: [ADR-0089](../decisions/0089-yara-first-real-analyzer-admission.md), [ADR-0099](../decisions/0099-build-yara-x-synthetic-compatibility-candidate.md), [ADR-0100](../decisions/0100-freeze-yara-x-ndjson-adapter-before-runner-linkage.md), [ADR-0101](../decisions/0101-prove-synthetic-runner-to-adapter-envelope-before-artifact-admission.md), [ADR-0102](../decisions/0102-compose-real-yara-x-synthetic-output-with-frozen-adapter.md)

## Architecture

```text
HRA analyzer plan + exact artifact manifest
                    |
                    v
production-admitted IAR-1B worker
  pinned YARA + pinned project rules + read-only staged artifacts
                    |
                    v
bounded vendor result -> closed adapter -> ADR-0013 normalization
                    |
                    v
immutable assessment + explicit coverage/limitations
```

Context never parses raw YARA output or invokes YARA. The Runner validates the
complete vendor output against a narrow adapter schema, then Context treats the
adapter envelope as untrusted derived data and normalizes it independently.

## Executable And Ruleset Supply Chain

- Pin source repository, source revision, build environment, target, compiler,
  dependencies, license, artifact digest, SBOM, and provenance.
- Build with only required modules; reject unapproved dynamic libraries and
  repository-provided modules.
- Compile project rules in a separate no-source release job.
- Root metadata defines current and previous admitted rulesets, expiry, and
  rollback prevention. No worker possesses signing or update credentials.

## Request And Result Boundary

- Request names only exact content IDs from the HRA plan, analyzer/profile
  identity, ruleset identity, and fixed budgets.
- Result accounts for every requested content ID and contains only bounded
  normalized rule identifiers, namespaces, tags, strings/offsets where
  permitted, and diagnostic reason codes.
- Raw stdout/stderr and unmatched file bytes are never retained by Context.

## Verification

- Unit fixtures cover parsing, ordering, duplicate matches, Unicode, offsets,
  excessive strings, malformed output, unknown rule, and incomplete coverage.
- Fault workers cover substitution, crash, timeout, fork, memory, output, and
  ruleset mismatch under every claimed platform backend.
- Release rehearsal proves clean install, update, rollback rejection, expiry,
  removal, and no network or source leakage.

## ADR-0095 Contract Boundary

The first adapter contract is deliberately synthetic-only. Its input mirrors
the minimum future result shape but admits only original-synthetic fixture
records. A deterministic offline checker verifies complete artifact accounting,
canonical rule observations, bounded byte ranges, exact digest identities, and
constant non-authority claims. It neither parses raw YARA output nor adds a
process, analyzer, ruleset, source, network, credential, or platform admission
path. Live results require a new reviewed contract and the ADR-0089 activation
gate.

## ADR-0096 Supply-Chain Boundary

The upstream source, executable build, and ruleset are three independent
identities. Version 1 selects only the official YARA v4.5.8 tag commit and
license-file metadata. Because that release has no uploaded GitHub assets, the
profile rejects any upstream binary and requires a future Impresari-owned,
per-target reproducible build with an exact archive digest, dependency closure,
SBOM, provenance, signature, vulnerability/license review, expiry, and
revocation record.

The production ruleset remains absent. Its later admission requires a separate
project-owned source and compiled-artifact identity, human review, license,
signature, expiry, and rollback record. Repository rules, includes, external
paths, custom modules, in-job updates, network retrieval, and worker-held update
credentials are structurally false. The offline checker verifies metadata and
fail-closed states only; it cannot download, build, sign, load, or execute an
artifact.

## ADR-0097 YARA-X Direction

YARA-X was selected before its engine-specific build architecture was frozen.
ADRs 0098–0100 now provide new engine/profile identities, documented
rule-compatibility constraints, and a bounded NDJSON adapter. They may not
reuse legacy YARA's executable, compiled-rules, module, or result-parser
identities. The common artifact, ruleset, confinement, accounting, expiry,
revocation, and non-safety requirements remain unchanged.

## ADR-0098 Closed YARA-X Boundary

The first YARA-X execution surface is one independently signed `yr` artifact,
one signed project-owned compiled ruleset, and one private staged regular file.
The exact argument vector selects compiled rules, NDJSON, namespace/tags,
zero-byte string rendering, disabled console logs and mmap, one thread, and
fixed file, match, engine-time, and output ceilings. A private empty `HOME`
prevents operator configuration from entering the scan contract.

The parser accepts one closed NDJSON object. It validates the exact staged path
but emits no path, retains no raw output or matched bytes, and derives range
length only from the v1.20.0 zero-byte marker. Imports/modules, includes,
external variables, regex/base64/XOR patterns, repository rules, recursive or
list scans, module data, relaxed syntax, ignored invalid rules, and arbitrary
arguments are closed. Any future expansion requires a versioned ADR and
compatibility/security evidence.

## ADR-0099 Synthetic Build And Compatibility Architecture

```text
immutable v1.20.0 archive + exact Impresari patch + Cargo.lock
                              |
                  ephemeral no-secret build host
                              |
          narrowed static yr + compiled synthetic rules
                              |
     fresh delegated cgroup + Landlock + seccomp per scan
                              |
     generated synthetic input -> bounded NDJSON assertion
                              |
          digest-only ephemeral receipt -> mandatory cleanup
```

The patch is data, not a new build tool. It disables default modules and
parallel compilation, then updates only `crossbeam-epoch` and `memmap2` to
compatible fixed releases. A pre-build gate hashes the pristine source files,
patch, and patched manifests/lock; an exact feature-tree gate proves RSA,
X.509, spin, Wasmtime WASI, and cap-std absent from the selected CLI graph.
RustSec ignores are enumerated by advisory ID and are valid only while those
reachability assertions pass.

Rule compilation consumes only the committed Impresari-owned synthetic rule
source. Scan inputs are generated in the ephemeral job and never come from the
checkout. Each scan uses the frozen ADR-0098 argument vector and a private
empty `HOME`. The launcher atomically places the process into a fresh cgroup,
installs the read-only staged-job Landlock policy and architecture-pinned
seccomp policy, bounds tasks, memory, CPU, elapsed time, and output, and
verifies an empty removed cgroup after completion. The existing composite CI
script owns the one transient delegated service; the compatibility path cannot
create another privileged launch site.

Build output, compiled rules, raw NDJSON, and runtime receipts remain under
ephemeral temporary/target paths and are deleted after validation. CI may
print only bounded digests, state, and claim-denial metadata. No upload action,
signature, release asset, cache publication, production process launch, or
ordinary host integration is introduced.

Run `33406541396`, job `99535422988`, passed this architecture on the exact
hosted candidate, including the separate mandatory cleanup step. Its executable
identity is per-run evidence and is not an admitted or reproducible artifact.

## ADR-0100 Pure Parser Boundary

```text
synthetic vendor-shaped NDJSON + exact control identities
                         |
             pure closed Rust parser
                         |
       path-free source-free normalized result
```

The parser owns no path or byte acquisition. The caller supplies the expected
staged path and artifact length as control data; the parser validates both and
emits neither. Exact structs with unknown-field denial cover the top-level,
rule, and string objects. A framing pass rejects BOM, CR, extra LF, surrounding
whitespace, invalid UTF-8, and over-limit input before JSON decoding.

The marker parser accepts only the canonical positive-decimal zero-byte form.
Checked integer addition prevents range overflow and escape. Canonicalization
sorts tags, rules, and ranges after rejecting duplicates, then hashes the exact
identity-bound normalized representation. Any error discards all intermediate
state and returns only a stable source-free category.

The library has no runner or analyzer dependency and cannot assert execution,
confinement, production, IAR-2, or safety. ADR-0101 now freezes synthetic
envelope composition; the production artifact pipeline remains a later
architectural decision.

The implemented crate is `context-yara-x-adapter`. It depends only on the core
UTC validator and locked Serde, JSON, and SHA-256 libraries. Its profile,
control, and result schemas are registry members; its original-synthetic corpus
is content-addressed by one closed provenance record. The parser returns a
single stable content-free error code or a complete normalized result, never a
partial result.

## ADR-0101 Synthetic Envelope Architecture

```text
closed case id -> pinned synthetic emitter -> bounded in-memory stdout
                                                |
                                  exact envelope validation
                                                |
                                      ADR-0100 pure parser
                                                |
                                source-free composition receipt
```

The emitter and coordinator are test-only components. The emitter embeds two
reviewed original-synthetic records and accepts only their closed case IDs. The
coordinator, not the parser, owns process termination, output ceilings,
confinement receipt validation, and cleanup. It passes captured bytes to the
parser only after exact length/digest and envelope checks, then discards them.

The composition receipt binds the synthetic job and normalized result but does
not copy the path or output. It records synthetic-emitter execution separately
from analyzer execution. No existing IAR-0 result is reinterpreted, and no
production runner API is opened by this checkpoint.

The implemented coordinator is `context-yara-x-envelope`. It reuses the one
`Command::new` site in `context-analyzer-runner`; the existing C launcher adds
one closed `--synthetic-envelope` mode and places only the emitter in the fresh
job cgroup. Exact preflight bytes are written by the launcher parent only after
a successful emitter exit, so any emitter stderr makes the capture fail. The
coordinator removes the exact job and empty cgroup before composing the receipt.

Run `33419412353`, job `99577842304`, passed both closed cases on the admitted
Ubuntu 24.04 synthetic boundary and passed its separate cleanup assertion. The
result proves this synthetic runner-to-adapter architecture only. The receipt
correctly records `yara_x_executed=false` and `production_admitted=false`.

## ADR-0102 Real-Engine Synthetic Composition

```text
pinned source-built yr + compiled Impresari synthetic rules + generated case
                                  |
                    single audited Runner launch site
                                  |
              fresh cgroup + Landlock + seccomp launcher
                                  |
                    bounded stdout held in memory
                                  |
                       ADR-0100 pure adapter
                                  |
          source-free execution receipt + path-free result
```

The runner verifies canonical regular files and exact SHA-256 identities for
the launcher, executable, compiled rules, and artifact before launch. The
launcher receives only the frozen argument vector and reports the exact
confinement preflight. The coordinator requires process success, bounded
output, exact preflight, exact expected rule identifiers, complete parser
accounting, and removal of the job directory and empty cgroup before emitting
the outer receipt.

This is test-only composition, not a production runner surface. The pure
adapter continues to state that it cannot prove execution; the outer
domain-separated receipt records real YARA-X execution and confinement. It
fixes artifact/ruleset admission, production, IAR-2, detection, safety, and
authority claims to false.
