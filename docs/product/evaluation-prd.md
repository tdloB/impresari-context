# Impresari Context — Evaluation and Benchmark PRD

## Document Control

- Product: Impresari Context.
- PRD ID/version: IC-EVAL-001 / 0.1.
- Status: Founder-approved evaluation baseline; technical calibration remains
  required before release-candidate scoring.
- Date: 2026-08-20.
- Release capabilities under evaluation: Slices A–D through declarative,
  non-executing extension contracts.
- Related documents:
  - [Master Product PRD](master-prd.md)
  - [Verifiable Local Context MVP PRD](verifiable-local-context-mvp-prd.md)
  - [Security Threat Model](../security/threat-model.md)

## Goal

Provide a reproducible evaluation system that determines whether Impresari
Context retrieves useful repository evidence with less context, preserves exact
recoverability and freshness, respects security boundaries, and remains
operationally practical.

Evaluation is a release gate, not marketing evidence. Results must include
failures, uncertainty, corpus limitations, configuration, hardware, and the
comparison baseline.

## Questions The Evaluation Must Answer

1. Does the engine retrieve the evidence needed to answer representative
   repository tasks at a fixed context budget?
2. Does it reduce delivered context without sacrificing material recall?
3. Can every exact fact be expanded to the correct current source?
4. Are stale, corrupt, partial, unsupported, and cross-workspace states detected
   rather than concealed?
5. Are deterministic operations reproducible?
6. Do resource limits hold on small repositories and realistic monorepos?
7. Does the engine work without network or hosted models?
8. Does the core provide value beyond direct file reads and native text search?
9. Can a second client reproduce the same semantics through the public
   contract?
10. For later slices, do structural and optional retrieval capabilities add
    measurable value without weakening evidence quality or safety?
11. Does the OS-shaped adapter preserve the exact semantics of the neutral core
    while adding no orchestration or filesystem authority?
12. Do all adversarial extension envelopes fail into metadata-only quarantine,
    and do accepted envelopes remain explicitly untrusted derived data?
13. Do hostile-repository admission contracts preserve incomplete coverage and
    reject every safety or ordinary-host execution claim without invoking an
    analyzer, parser, process, network, or upload capability?

### Slice C/D hard gates

- OS adapter semantic equivalence to direct public-engine use: 100% on the
  frozen adapter corpus.
- Adapter-added orchestration or filesystem authority: zero.
- Adversarial extension-envelope quarantine rate: 100% on authority, identity,
  unknown-field, malformed, and oversized cases.
- Raw adversarial extension bytes retained in quarantine records: zero.
- Extension process, network, environment, model, cache, filesystem, artifact
  execution, or other privileged grants: zero for the approved v1 milestone.
- Accepted extension output trust label: `untrusted_derived_data` in 100% of
  accepted cases.
- MCP lifecycle/framing/tool conformance: 100% on the frozen protocol suite,
  with zero non-MCP stdout bytes and zero source-workspace mutations.
- Direct-engine/MCP packet semantic equivalence: 100% on the frozen transport
  corpus.
- Native release-candidate package build and clean-install smoke pass: 100% of
  Tier A targets before publication.

### ADR-0073 HRA-0 hard gates

- All seven HRA-0 contracts validate under the bundled full Draft 2020-12
  validator and the dependency-free project-subset checker.
- The frozen `hra-static-contract-v1` profile is byte-identical to its
  conformance fixture and matches its committed SHA-256 sidecar.
- Source mutation, process execution, analyzer execution, network access,
  artifact upload, deep hostile-format parsing, and ordinary-host execution are
  fixed to `false` in the profile.
- Required analyzer coverage represents every lifecycle state independently of
  finding count; unavailable mandatory analysis remains incomplete.
- False safety claims and ordinary-host execution authority are rejected in
  100% of declared negative fixtures.
- Every HRA-0 fixture is original synthetic JSON with reviewed, digest-checked
  provenance. No executable, malware, live signature, third-party source,
  private source, or provider data is admitted.
- HRA-0 results support contract claims only. Malware detection, repository
  safety, analyzer isolation, and quarantine execution remain unscored and
  unclaimed.

### ADR-0073 HRA-1 hard gates

- Runtime inventory output validates against the frozen
  `security-artifact-inventory` schema and profile digest.
- Every admitted artifact is bound to the exact workspace snapshot, path
  identity, content digest, and byte length; stale content fails closed.
- Windows-oriented, archive, script, source, configuration, documentation,
  text, binary, and unknown classes are recognized with bounded prefix and
  extension checks only, independent of host platform.
- Symlinks, special files, oversized files, read failures, snapshot omissions,
  and runtime ceilings are explicit and make completeness visible.
- Inventory records retain no raw source bytes and emit no findings, assessment,
  policy, safety, or execution decision.
- Source immutability passes before and after inventory, and the implementation
  adds no network, process, analyzer, upload, deep-parser, credential, or
  repository-execution capability.

### ADR-0073 HRA-2 npm lifecycle hard gates

- Only the closed `preinstall`, `install`, `postinstall`, `prepare`,
  `prepublish`, `prepublishOnly`, `publish`, and `postpublish` keys under a
  strict top-level `package.json` `scripts` object produce observations.
- Each observation is `informational`, `confirmed`, `observed`, and references
  exact schema-valid evidence containing only the matched key token.
- Script values, commands, environment values, and unrelated nested keys are
  not interpreted, promoted to control fields, or retained in the observation
  bundle.
- Invalid JSON, non-object `scripts`, non-string lifecycle values, ambiguous
  keys, stale snapshots, and the 1,000-finding ceiling fail closed or become
  explicit exclusions.
- Repeated input is deterministic; source immutability and the HRA-1 no-authority
  gates remain passing.

### ADR-0073 HRA-2 Compose privilege hard gates

- Only `compose.yaml`, `compose.yml`, `docker-compose.yaml`, and
  `docker-compose.yml` are candidates.
- Only an exact `privileged: true` property at four-space indentation beneath a
  simple two-space service key under one top-level `services:` key produces a
  `medium`, `confirmed`, observed `privilege` finding.
- The evidence span contains exactly `privileged`; image names, service names,
  values, comments, and unrelated repository content are not retained.
- Top-level or deeper lookalikes do not match. Tabs, block scalars, duplicate
  services, missing canonical services, non-UTF-8, and alternative privileged
  scalar syntax are explicit unsupported cases.
- The lexical rule does not validate complete Compose semantics, infer intent,
  or inspect anchors, aliases, merges, profiles, generated configuration, or
  runtime behavior.

### ADR-0073 HRA-3 coverage and assessment hard gates

- Analyzer requirements are grouped deterministically by capability over
  sorted, deduplicated exact artifact hashes and validate against the frozen
  coverage schema.
- This slice emits only `unavailable` mandatory requirements with
  `analyzer-execution-not-authorized`; no analyzer is discovered or run.
- Assessment assembly rejects forged inventory identities, altered coverage
  states, mismatched snapshots, unknown artifact hashes, and missing evidence
  references.
- Inventory omissions, observation exclusions/truncation, and incomplete
  mandatory analysis force `partial` completeness independently of finding
  count.
- Assessment output validates against the frozen schema, stays within the
  frozen output ceiling, and fixes safety, ordinary-host execution, and added
  authority to false.
- Completed coverage is accepted only from ADR-0013-normalized analyzer output
  whose closed payload matches the exact snapshot, planned requirement,
  capability, sorted artifact set, analyzer digest, ruleset digest, and current
  freshness interval.
- Stale, future-completed, malformed, excessive, mismatched, duplicate, or
  authority-claiming result payloads fail closed and cannot change coverage or
  assessment state.
- Derived findings retain only bounded categorical fields, exact artifact and
  analyzer/ruleset digests, constant limitations, and untrusted-derived trust;
  raw analyzer text is not admitted.

### ADR-0073 HRA-4 reference evaluator hard gates

- The evaluator is a separate pure library and has no filesystem, process,
  network, model, credential, exception, approval, or repository capability.
- Exact assessment, coverage, finding-set, and policy identities must agree;
  altered or authority-claiming records fail closed.
- Identical immutable inputs produce the same canonical decision identity and
  the same priority-ordered complete matched-rule list.
- Effect precedence is monotonic toward restriction: `blocked`, then
  `manual_review_required`, then `analysis_incomplete`, then
  `isolated_execution_eligible`.
- Missing mandatory coverage prevents eligibility even without a matching
  `require_analysis` rule. Multiple eligibility profiles deny eligibility.
- The only eligibility target is one named quarantine profile; safety and
  ordinary-host authority remain false.

### ADR-0074 IAR-0 protocol and synthetic-worker hard gates

- All six runner contracts validate under the bundled Draft 2020-12 validator;
  every declared negative fixture is rejected.
- The fixed `iar-protocol-synthetic-v1` profile is byte-identical to its fixture
  and matches its committed SHA-256 sidecar.
- Every fixture is original synthetic JSON with digest-checked provenance and
  contains no executable, malware, live signature, third-party/private source,
  or provider data.
- One bounded big-endian length-prefixed request frame is accepted; truncated,
  trailing, oversized, malformed, or unknown-field input fails closed.
- Exact manifest, profile, request, artifact, result, and failure identities are
  verified before promotion.
- The no-op path accounts for every exact in-memory artifact and emits zero
  findings; crash, timeout, mutation, flood, and malformed-output paths emit
  only source-free all-or-nothing failures.
- Filesystem access, process launch, analyzer/parser execution, network,
  credentials, uploads, quarantine, repository execution, safety claims, and
  added authority remain zero and unclaimed.

### ADR-0074 IAR-1A application-supervisor hard gates

- The fixed `iar-application-supervisor-v1` profile is byte-identical to its
  fixture and digest sidecar; it records staging/process launch as present and
  fixes analyzer execution and every external authority false.
- Exact executable substitution, symlinked staging roots, and preexisting job
  identities fail before launch or staging reuse.
- Every artifact is written under an opaque digest name with create-new
  semantics and matches exact pre/post hashes.
- Crash, wall timeout, staged-input mutation, output flood, malformed output,
  and nonzero exit produce no partial promoted result; the direct child is
  reaped and the exact job is removed.
- The worker receives no shell, arguments, inherited environment, repository
  path, credentials, endpoint, analyzer selection, or parser selection.
- The posture is `application_enforced` only. OS/VM confinement, verified
  network denial, unrelated-handle closure, and descendant containment remain
  false, unverified, and unclaimed.
- Fixtures remain original synthetic JSON; process tests use only an exact
  first-party synthetic worker and synthetic bytes.

### ADR-0074 IAR-1B OS-confinement hard gates

- Each claimed host has one exact platform mechanism/version/profile identity;
  missing primitives or verification produce `unsupported`, never fallback.
- Filesystem, descriptor/handle, credential, device, process visibility,
  descendant/orphan, network, CPU, memory, process-count, disk, mutation,
  timeout, crash, and cleanup escape attempts pass on the native host.
- Effective policy is verified after launch and source-free evidence records the
  selected backend, OS/kernel version, profile digest, limits, and complete
  cleanup state.
- No real analyzer, hostile-format parser, updater, provider, or repository
  artifact participates in IAR-1B admission.
- IAR-2 remains closed until every platform claimed for YARA has passed IAR-1B.

The first macOS App Sandbox/XPC synthetic candidate records a `partial` result.
It passes identity, bounded transport, entitlement, external-filesystem,
credential-canary, synthetic-device, unrelated-process, and live loopback
network-denial checks. Resource/process-tree and fault-timeout checks also pass,
while complete OS-managed cleanup, production signing/notarization, packaging,
clean-machine, and multi-host gates remain false or unverified; the candidate
is not an admission result.

The hybrid follow-up corrects the earlier assumption that XPC must supply the
whole boundary alone. Native synthetic evidence now passes for `RLIMIT_CPU`
termination, current-footprint-derived `RLIMIT_AS` growth denial,
`RLIMIT_NPROC=0` denial of `fork` and `posix_spawn`, exact prepared-service
identity, supervisor timeout termination, crash/relaunch, and source-byte
cleanup. These checks may be scored as passing only for the observed ad hoc
prototype. The frozen production-candidate profile, source-free Rust-to-host
preparation contract, and exact synthetic pseudo-terminal denial now pass their
native/Rust/schema checks. Signing/notarization/cask lifecycle, clean-machine
Gatekeeper, complete Tier A, OS-managed removal, and every claimed host remain
false or unverified, so macOS is still not admitted.

The first Tier A checkpoint additionally records native negative evidence:
nine separately closed 1 MiB files bypass any 8 MiB aggregate interpretation
of the per-file limit, and a fresh XPC service process observes a synthetic
marker retained by its predecessor. `aggregate_disk_bound` and
`cross_job_isolation` remain false. The candidate cannot progress to IAR-1B by
signing or packaging alone.

The first Linux checkpoint freezes the `iar-linux-synthetic-v1` profile and a
closed source-free receipt. Its primitive suite must verify effective
`no_new_privs`, one exact read-only Landlock input, external-file, credential,
device, and path-write denial, an architecture-pinned default-deny seccomp
filter, network and descendant denial, and unrelated-descriptor closure. The
profile grants zero writable path-backed filesystem because cgroup v2 is not a
portable aggregate storage quota. CPU, memory, process count, exact
`cgroup.kill`, empty-state, bounded-output, timeout, crash/relaunch, cleanup,
and cross-job checks remain false until a delegated cgroup v2 leaf passes the
native suite. Missing delegation or controllers produces `unsupported`; CI
success only establishes an honest measurement.

The first hosted Ubuntu 24.04 run observed kernel `6.17.0-1022-azure`,
`x86_64`, and Landlock ABI 7. Its primitive suite passed, but the job cgroup
was not delegated. The receipt therefore returned `unsupported` with
`os_confined=false`; this is valid fail-closed evidence, not an IAR-1B pass.

The separate delegated-cgroup component checkpoint then passed on the same
kernel and architecture in PR 130 job `99194709845`. Under one ephemeral
systemd `Delegate=yes` service, the unprivileged synthetic supervisor reproduced
CPU, memory, process-count, exact-kill, empty-state, bounded-output, timeout,
crash/relaunch, cleanup, and cross-job checks using atomic
`CLONE_INTO_CGROUP`. Its receipt claims only
`resource_lifecycle_confined=true`; overall `os_confined` and production
admission remain false pending source-free composition and multi-host evidence.

The composition checkpoint is scored only from one receipt produced inside one
transient delegation. It must show that the profile was applied before atomic
worker placement, the worker itself reproduced the primitive suite, and the
resource/lifecycle corpus passed in the same delegated service. The standalone
component receipts remain historical evidence and cannot be combined after the
fact to manufacture a pass.

PR 131 job `99197119262` passed this composite on kernel
`6.17.0-1022-azure`, x86_64, Landlock ABI 7. This admits only the exact-host
candidate receipt's `os_confined=true`; broader Linux and production admission
remain false pending independently reproduced target coverage.

The next held-out architecture unit is `ubuntu-24.04-arm`. Its AArch64 seccomp
filter is independently pinned, and only a fresh composite receipt from that
ephemeral runner may establish its exact-host candidate. Emulation and the
existing x86_64 receipt are not accepted as substitutes.

PR 132 job `99198568879` independently passed the AArch64 composite on kernel
`6.17.0-1022-azure` with Landlock ABI 7. Architecture coverage is now native on
x86_64 and arm64, while kernel diversity and production admission remain open.

The next held-out kernel units are the native `ubuntu-22.04` and
`ubuntu-26.04` standard runners. Their current image records identify 6.8 and
7.0 Azure kernel lines. Each must independently pass the same primitive and
single-service composite receipt; an Ubuntu userspace in a container atop the
already admitted kernel would not count.

## Evaluation Principles

1. Freeze the corpus and task manifest before scoring a release candidate.
2. Separate tuning/development corpora from held-out evaluation corpora.
3. Compare at matched task, budget, and evidence definition.
4. Report recall and context cost together; neither can substitute for the other.
5. Treat exact evidence and security gates as hard constraints.
6. Preserve raw measurements and derive summaries through versioned scripts.
7. Run cold- and warm-cache measurements separately.
8. Report unsupported states and exclusions.
9. Do not use LeanCTX or Graft source, tests, prompts, fixtures, or benchmarks as
   implementation fixtures without a separately documented provenance decision.
10. Do not optimize solely for one AI model, tokenizer, repository, language,
    or AI App Builder OS workflow.

## Evaluation Audiences

| Audience | Decision supported |
| --- | --- |
| Maintainers | Is the release correct, safe, performant, and regression-free? |
| Developers/integrators | Is the engine useful for representative repositories and budgets? |
| Security reviewer | Do isolation, freshness, redaction, and resource controls hold? |
| Founder/steward | Is the slice ready to publish or integrate? |
| Contributors | Which capability has measurable gaps and what evidence is required for improvement? |

## Benchmark Corpus

### Corpus categories

| Category | Purpose | Minimum composition before MVP release |
| --- | --- | --- |
| Synthetic conformance repositories | Exact known paths, matches, spans, encodings, mutations, and expected packets | At least 12 small fixtures |
| Adversarial repositories | Traversal, links, injection, secret-like content, malformed files, huge lines, many files, races, and spoofing | All threat-model classes relevant to MVP |
| Small public repositories | Installation, common layouts, mixed docs/config/tests, human review | At least 6 with compatible licenses |
| Medium public repositories | Retrieval quality, indexing time, memory, incremental mutation | At least 4 |
| Large/monorepo corpus | Limit behavior, partial states, resource scaling | At least 2 or approved generated equivalents |
| Held-out task corpus | Prevent tuning to known queries and repositories | At least 25% of public/synthetic task cases |
| Private opt-in corpus | Realistic confidential workflows without publishing source or raw results | Optional; never required for public reproducibility |

Counts are minimums for diversity, not evidence that the corpus is broadly
representative. The manifest must record why each repository is included.

### Corpus manifest fields

- corpus and fixture identifier;
- source URL or generation script;
- immutable revision/digest;
- license and permitted evaluation use;
- language and artifact composition;
- eligible and excluded file/byte counts;
- sensitivity/publication classification;
- expected mutations and cleanup;
- known limitations;
- development, validation, or held-out split;
- date admitted and reviewer.

### Provenance rules

- Prefer original synthetic fixtures and clearly licensed public repositories.
- Do not copy upstream LeanCTX/Graft fixtures or internal behavior descriptions.
- Store only the minimum public corpus material permitted by license; scripts may
  fetch pinned public revisions after explicit user execution.
- Never commit private customer or user source.
- Test queries, expected evidence, and labels must be original and reviewable.
- HRA-0 security fixtures additionally require a committed per-file SHA-256,
  origin, license, purpose, negative-content declaration, and automated digest
  verification before they enter the conformance manifest.

## Task Taxonomy

### MVP retrieval tasks

1. Find a known exact file or path.
2. Find exact literal definitions and uses.
3. Find configuration values and their documentation.
4. Locate tests associated by filename, import text, or shared literal.
5. Locate routes or handlers through lexical evidence without claiming semantic
   tracing.
6. Identify all controlled occurrences of a security-sensitive API or pattern.
7. Build context for a bug description containing file, component, and behavior
   clues.
8. Build context for documentation or configuration change planning.
9. Recover exact evidence from a packet.
10. Validate a packet before and after a controlled workspace mutation.

### Safety and lifecycle tasks

1. Attempt reads through traversal, symlink, special-file, case, and Unicode
   edge cases.
2. Attempt cross-workspace evidence and cache resolution.
3. Inject instructions and control-looking text into source and filenames.
4. Trigger file, pattern, match, output, memory, disk, and time limits.
5. Corrupt or downgrade cache and packet formats.
6. Interrupt indexing and restart.
7. Run with network denied and inspect attempted connections.
8. Compare source and Git state before and after all operations.
9. Inspect logs and errors for source, secret-like values, paths, and environment
   leakage.

### Later structural tasks

- describe a symbol with exact declaration evidence;
- identify imports, references, callers, and callees where supported;
- produce a repository map;
- trace bounded dependency or impact paths;
- distinguish confirmed and heuristic edges;
- update a graph after a controlled change.

These later tasks cannot be used to claim MVP capability.

## Ground Truth And Labeling

Each quality task includes:

- natural-language task statement;
- allowed retrieval capability set;
- required evidence set;
- optional/helpful evidence set;
- explicitly irrelevant or misleading evidence;
- maximum context budget(s);
- expected unknowns/unsupported states;
- reviewer rationale;
- label version and reviewer identity.

At least two reviewers should independently label high-consequence or ambiguous
tasks when project capacity allows. Disagreement is recorded, reconciled, and
reported; it is not silently averaged away.

## Baselines

### Required MVP baselines

| Baseline | Purpose |
| --- | --- |
| Direct known-file reads | Lower-bound context cost when the answer location is already known |
| Native literal search plus fixed surrounding lines | Common local-tool baseline |
| Native filename/path search | Exact discovery baseline |
| Naive bounded repository concatenation/map | Context-budget comparison |
| Manual expert-selected evidence on a reviewed subset | Approximate quality ceiling and label check |

The exact native tool may vary by supported platform. The evaluation records
tool version and normalizes the semantic baseline rather than claiming one
shell utility is universally available.

### Future comparative baselines

Publicly available context/code-intelligence tools may be evaluated later if
installation is reproducible and licensing/terms permit it. Comparisons must
state configuration and capability differences and may not imply endorsement,
affiliation, or complete product equivalence.

## Metrics

### Evidence quality

| Metric | Definition |
| --- | --- |
| Required-evidence recall | Required ground-truth evidence items retrieved / total required items |
| Evidence precision | Relevant retrieved evidence items / total retrieved evidence items |
| Recall at budget | Recall achieved under a fixed delivered-context budget |
| First-relevant rank / MRR | Rank quality for tasks with identifiable first evidence |
| Unsupported honesty | Unsupported/unknown cases correctly reported / total cases that should be unsupported/unknown |
| False-authority count | Derived, partial, or unsupported material incorrectly labeled exact/confirmed |

### Verification and freshness

| Metric | Definition |
| --- | --- |
| Exact recovery rate | Current valid evidence references that resolve to byte/span-correct source |
| Stale detection rate | Controlled stale references/packets correctly rejected or marked stale |
| Cross-workspace rejection | Cross-workspace resolution attempts denied without disclosure |
| Packet integrity detection | Tampered/corrupt packets detected |
| Partial-state visibility | Partial discovery/index cases explicitly represented |

### Context efficiency

| Metric | Definition |
| --- | --- |
| Delivered units | Declared byte/token/item accounting used by the packet contract |
| Context reduction | `1 - engine_units / baseline_units` at matched task and recall |
| Evidence density | Relevant evidence units / total delivered evidence units |
| Recovery overhead | Packet metadata and recovery-index units / total units |
| Budget compliance | Requests whose delivered units do not exceed declared budget |

Token measurements must name the tokenizer/model family and remain secondary to
a model-neutral byte/item accounting method until the contract decision is made.

### Determinism and correctness

| Metric | Definition |
| --- | --- |
| Snapshot repeatability | Identical input/rules/version runs producing identical snapshot identity |
| Result repeatability | Identical deterministic requests producing byte-stable normalized output |
| Span correctness | Returned spans matching the exact expected byte and line/column contract |
| Ordering stability | Stable result ordering under equal scores and parallel execution |

### Performance and resources

| Metric | Definition |
| --- | --- |
| Snapshot/index wall time | Cold and warm elapsed time by eligible file/byte scale |
| Query latency | p50, p95, and maximum under declared corpus/profile |
| Peak resident memory | Peak measured memory per operation and corpus |
| Cache size | Derived-cache bytes relative to eligible source bytes |
| Incremental refresh cost | Later: time and IO for controlled changed-file sets |
| Limit enforcement overshoot | Resource use beyond a declared hard limit before stop |

### Security and privacy

| Metric | Definition |
| --- | --- |
| Unauthorized disclosure count | Content or names returned outside exact authority |
| Source mutation count | Source/Git/config changes caused by engine operations |
| Network attempt count | Runtime outbound connection attempts in network-denied suite |
| Default log leakage count | Source, secret-like values, raw environment values, or unauthorized paths in logs/errors |
| Cache isolation failures | Cross-workspace cache reads or collisions |
| Injection policy violations | Repository content affecting control/policy/capability behavior |

## Initial MVP Release Gates

### Hard gates

| ID | Gate | Required result |
| --- | --- | --- |
| EVAL-G-001 | Exact evidence recovery | 100% for valid current conformance references |
| EVAL-G-002 | Stale detection | 100% for controlled mutation cases |
| EVAL-G-003 | Cross-workspace isolation | Zero successful disclosures/resolutions |
| EVAL-G-004 | Path/security isolation | Zero unauthorized content or name disclosures in frozen adversarial suite |
| EVAL-G-005 | Source immutability | Zero engine-caused source/Git/config mutations |
| EVAL-G-006 | Budget compliance | 100% of bounded packet outputs at or below declared units |
| EVAL-G-007 | Determinism | 100% byte-stable normalized output for designated deterministic fixtures |
| EVAL-G-008 | False exact authority | Zero occurrences |
| EVAL-G-009 | Network independence | Full suite passes with network denied and zero required/attempted runtime connections |
| EVAL-G-010 | Log/error privacy | Zero prohibited values in seeded leakage fixtures |
| EVAL-G-011 | Partial/unsupported honesty | 100% of controlled partial and unsupported cases labeled correctly |
| EVAL-G-012 | Contract conformance | CLI and programmatic reference client pass the same semantic suite |

### Quality gates

| ID | Gate | Initial target |
| --- | --- | --- |
| EVAL-Q-001 | Required-evidence recall at the primary fixed budget | At least 0.90 overall and no critical task below its declared floor |
| EVAL-Q-002 | Baseline protection | Not more than 0.02 absolute recall below native-search baseline on tasks the baseline supports |
| EVAL-Q-003 | Context reduction | Median reduction of at least 30% versus the declared baseline at matched recall |
| EVAL-Q-004 | Evidence precision | At least 0.70 overall for the frozen task set; report per task class |
| EVAL-Q-005 | Human usefulness | At least 80% of reviewed packets rated usable without a full repository dump on the review subset |

Quality targets are provisional until the baseline run. Recalibration requires a
recorded decision made before scoring the release candidate, with original
targets retained in history. Security, exactness, freshness, and budget hard
gates cannot be relaxed to improve quality metrics.

### Performance gates

Absolute budgets must be set after stack and supported-hardware decisions. The
release must nevertheless meet these invariant gates:

- no uncontrolled growth after configured limits;
- peak memory, disk, and time scale curves are reported;
- p50/p95 measurements include cold and warm results;
- release candidate is no more than the approved regression allowance from the
  previous accepted version on the same harness;
- large-corpus limit behavior ends in a valid partial/limited state;
- cancellation or disk failure does not commit an authoritative partial cache.

## Human Evaluation

### Review questions

Reviewers score each selected packet:

1. Does it contain the evidence needed to begin the task?
2. Is irrelevant material low enough to scan efficiently?
3. Are missing evidence and unknowns obvious?
4. Can consequential statements be verified without guessing?
5. Are paths, excerpts, truncation, and freshness understandable?
6. Would the reviewer need a full repository dump before proceeding?

### Scale

- 0 — unusable or misleading;
- 1 — major evidence missing;
- 2 — usable only with substantial additional retrieval;
- 3 — usable with limited expansion;
- 4 — complete and efficient for the defined task.

Reviewers see engine and baseline packets in randomized order where practical.
The evaluation reports agreement and comments, not only averages.

## Experiment Matrix

| Experiment | Variables | Controls | Outputs |
| --- | --- | --- | --- |
| Budget curve | Multiple packet budgets | Same task/corpus/snapshot | Recall, precision, density, reduction |
| Cold/warm lifecycle | Cache state | Same build/config/hardware | Time, memory, disk, output identity |
| Mutation/freshness | File add/change/delete/rename | Same original snapshot | Detection, invalidation, recovery behavior |
| Scale curve | File/byte/count/line profiles | Same configuration | Latency, memory, disk, partial states |
| Query classes | Exact path, literal, lexical, permitted pattern | Frozen ground truth | Quality by class |
| Client conformance | CLI vs library client | Same canonical request | Semantic equivalence |
| Adversarial controls | Threat fixtures | Frozen policy | Disclosure, mutation, injection, limit outcomes |
| Platform matrix | Supported OS/filesystem/architecture | Pinned toolchain and corpus | Correctness and performance variance |

## Reproducibility Requirements

Every evaluation report records:

- engine revision and dirty-state status;
- schema, algorithm, cache/index, policy, and fixture versions;
- dependency lock and toolchain versions;
- corpus manifest digest;
- full configuration and limits;
- platform, filesystem, CPU, memory, and relevant isolation details;
- cold/warm state;
- command or runner identifier;
- raw result artifact digests;
- exclusions, failures, retries, and deviations;
- report generator version.

The project should provide one documented local command to run the public
conformance/evaluation subset. Large or private suites may be separate but may
not be the sole basis for public claims.

## Statistical And Reporting Rules

- Use per-task results plus aggregate median/percentile summaries.
- Do not rely on a single average across differently sized repositories.
- Report confidence intervals or repeated-run variability where meaningful.
- Mark exploratory analyses separately from preregistered release gates.
- Correct obvious harness failures only through a versioned rerun; never delete
  an unfavorable valid result.
- Report missing data and timeouts as outcomes.
- Keep security failures as counts with exact fixture IDs and remediation state.
- Avoid claims of superiority unless the comparison is matched, reproducible,
  materially significant, and scoped to the tested conditions.

## Regression Policy

Each accepted release becomes a versioned baseline. A candidate fails when it:

- violates any hard gate;
- drops below an approved quality floor;
- introduces a statistically/materially significant quality or resource
  regression beyond the approved allowance;
- changes deterministic output without a recorded contract/algorithm decision;
- expands skipped/unsupported categories without explicit review; or
- changes corpus, labels, or metrics after results are known without retaining
  and explaining the original comparison.

An improvement in latency or context reduction cannot compensate for a security,
freshness, exactness, or false-authority failure.

## Evaluation Deliverables

1. Versioned corpus manifest.
2. Original fixture-generation scripts and labels.
3. Baseline runner and engine runner.
4. Conformance schema tests.
5. Raw machine-readable results.
6. Human-review packet and reconciliation record.
7. Security/adversarial result ledger.
8. Performance and scale report.
9. Release-gate summary with pass/fail/blocked status.
10. Public limitations and claim-language draft.

## Rollout

### Phase 0 — Harness before implementation

- define schemas for tasks, evidence labels, and raw results;
- build the first synthetic conformance and adversarial fixtures;
- capture native-tool baseline behavior;
- freeze initial decision and corpus manifests.

### Phase 1 — Development feedback

- run fast conformance and security subsets on every relevant change;
- run quality and resource subsets on pull requests affecting retrieval,
  snapshotting, evidence, packets, paths, cache, or serialization.

### Phase 2 — Release candidate

- freeze the candidate and corpus;
- run full supported-platform, quality, performance, security, and clean-install
  suites;
- complete human review;
- publish the internal release decision report.

### Phase 3 — Post-release

- accept opt-in correction reports without collecting source by default;
- turn reproducible failures into new fixtures where licensing/privacy permit;
- rerun the stable suite for patches and dependency upgrades;
- expire claims when the corpus, product, or environment materially changes.

## Risks And Mitigations

| Risk | Mitigation |
| --- | --- |
| Benchmark overfitting | Held-out split, freeze before scoring, versioned history |
| Synthetic tasks are unrealistic | Mix original fixtures, public repositories, and optional private review |
| Public corpus licensing mistakes | Immutable source/license manifest and review |
| Context reduction hides missing evidence | Always pair reduction with recall and human usefulness |
| One tokenizer biases results | Model-neutral primary accounting plus named optional tokenizers |
| Hardware changes hide regressions | Record hardware and retain same-host/version comparisons |
| Manual labels are subjective | Reviewer rationale, disagreement tracking, dual review for ambiguous cases |
| Security suite creates false confidence | Map to threat model, publish residual risks, add independent review |
| Private evaluation cannot be reproduced | Never make it the sole evidence for public claims |

## Open Decisions

1. Primary model-neutral context-budget unit.
2. Supported platform/hardware tiers and absolute performance targets.
3. Initial public repository corpus and license review.
4. Ground-truth annotation format and review capacity.
5. Safe source excerpt handling in evaluation artifacts.
6. CI subset versus scheduled/full local suite split.
7. Performance regression allowance after the first accepted baseline.
8. Whether public release reports include only aggregates or selected
   source-compatible task details.

## Approval And Change Control

- Approved by/date: Founder, 2026-08-20. Technical calibration must occur before
  release-candidate scoring.
- Baseline, corpus, metric, label, or threshold changes require versioning and a
  recorded rationale before release-candidate scoring.
- Structural, semantic/model, extension, hosted, durable-memory, or OS-adapter
  evaluations require additions to this PRD before those capabilities ship.
