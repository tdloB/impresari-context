# ADR-0097: Reconsider The First Analyzer Engine Selection

- Status: Accepted; Option A selected
- Date: 2026-08-31
- Decider: Aaron Boldt
- Related: ADR-0089, ADR-0095, ADR-0096

## Context

ADR-0089 selected YARA as the first real analyzer while execution remained
gated. ADR-0095 froze a synthetic rule-observation contract, and ADR-0096
selected the official YARA v4.5.8 source candidate without downloading,
building, signing, or admitting an artifact.

During the post-ADR-0096 build-profile audit, the official YARA v4.5.8 README
was found to describe YARA as being in maintenance mode and to direct future
enhancement work toward YARA-X. The official YARA-X project describes itself as
the Rust rewrite intended to replace YARA, reports stable production use, and
publishes current multi-platform release assets with GitHub-recorded SHA-256
digests.

This is a material upstream transition. Freezing a build, module, ruleset, and
live-output contract before reconsidering the engine could create avoidable
migration work or place the first production analyzer on the maintenance-only
line.

## Choice Considered

The founder considered exactly two directions before another engine-specific
build or execution record could be accepted.

### Option A — Supersede YARA With YARA-X

- Supersede ADR-0089's engine choice and ADR-0096's source candidate.
- Preserve ADR-0095's evidence-binding principles but version the adapter,
  analyzer identity, and fixture contracts for YARA-X.
- Pin an exact YARA-X release, tag commit, per-platform artifact/source build,
  CLI/API surface, rule-language subset, module policy, and structured-output
  schema.
- Re-run rule compatibility, malformed-input, false-positive, resource,
  supply-chain, confinement, hosted-evidence, and independent-review gates.

### Option B — Retain Legacy YARA

- Keep ADR-0089 and ADR-0096 current.
- Freeze the exact v4.5.8 per-target build, disabled process-scanning profile,
  module subset, compiled-rules invocation, and bounded text-output parser.
- Record the maintenance-only upstream posture and a future migration trigger.
- Continue every existing artifact, confinement, hosted-evidence, and
  independent-review gate.

## Evidence Relevant To The Choice

| Topic | YARA v4.5.8 | YARA-X v1.20.0 |
| --- | --- | --- |
| Upstream posture | Maintained for bug fixes and minor features; enhancements focus on YARA-X | Described by upstream as mature, stable, production-used, and intended to replace YARA |
| Implementation | C | Rust |
| Current official release observed | v4.5.8, commit `84b0e3cc0e42f8f8e6b84d19c97ec3ac6ff8aee8`, 2026-07-28 | v1.20.0, commit `60ad06971467029e77967e59d580cbbe85a1474d`, 2026-08-24 |
| Uploaded release assets | None | macOS arm64/x86_64, Linux arm64/x86_64, Windows x86_64, and Windows C API assets with GitHub-recorded SHA-256 digests |
| Machine-readable CLI result | Legacy text output requires a bounded parser | `json` and `ndjson` scan output documented |
| Process scanning | Present upstream and must be disabled at build time | Not implemented, matching the static-only initial scope |
| Compatibility cost | Existing ADRs name this engine | APIs are not drop-in compatible; documented rule-language differences require a new compatibility corpus |

The table records upstream facts and architecture consequences. It does not
admit either engine, artifact, or rule.

## Decision

Select Option A: supersede legacy YARA with YARA-X as the first real-analyzer
engine. ADR-0089's engine choice and ADR-0096's v4.5.8 source candidate are
superseded. ADR-0095's evidence-binding and non-authority principles remain
requirements, but its engine, adapter, compiled-rule, fixture, and result
identities may not be reused as YARA-X identities.

The next checkpoint must freeze the exact YARA-X release/source and
per-platform artifact strategy, build and dependency closure, CLI/API surface,
rule-language and module subset, structured-output schema, ruleset lifecycle,
and compatibility corpus. That checkpoint is contract-only unless separately
authorized and admitted.

Until those replacement contracts are accepted:

- no source archive may be fetched for a build;
- no analyzer executable or ruleset may be built, signed, or admitted;
- no live-output parser or adapter may be activated;
- no analyzer may execute; and
- IAR-2 and production admission remain closed.

## Official Sources

- [YARA v4.5.8 README](https://github.com/VirusTotal/yara/blob/v4.5.8/README.md)
- [YARA v4.5.8 release](https://github.com/VirusTotal/yara/releases/tag/v4.5.8)
- [YARA v4.5.8 command-line contract](https://github.com/VirusTotal/yara/blob/v4.5.8/docs/commandline.rst)
- [YARA-X v1.20.0 README](https://github.com/VirusTotal/yara-x/blob/v1.20.0/README.md)
- [YARA-X v1.20.0 release](https://github.com/VirusTotal/yara-x/releases/tag/v1.20.0)
- [YARA-X versus YARA](https://virustotal.github.io/yara-x/docs/intro/yara-x-vs-yara/)
- [YARA-X rule differences](https://virustotal.github.io/yara-x/docs/writing_rules/differences-with-yara/)
- [YARA-X CLI commands](https://virustotal.github.io/yara-x/docs/cli/commands/)

## Resolution

Aaron Boldt explicitly approved the recommended YARA-X switch on 2026-08-31.
This decision selects an engine direction only. It does not admit a source
archive, release asset, executable, ruleset, live parser, analyzer execution,
IAR-2, production support, or safety claim.
