# ADR-0098: Freeze The YARA-X Contract-Only Admission Profile

- Status: Accepted for contract-only implementation
- Date: 2026-08-31
- Decider: Aaron Boldt through ADR-0097 and the standing accepted-roadmap directive
- Related: ADR-0074, ADR-0082, ADR-0095, ADR-0096, ADR-0097

## Context

ADR-0097 selects YARA-X as the first real analyzer but admits no artifact,
ruleset, parser, or execution. The official v1.20.0 release publishes five CLI
archives and one Windows C-API archive with GitHub-recorded SHA-256 digests.
Its pinned release workflow builds the CLI with `release-lto` and static CRT
flags, but selects the mutable Rust `stable` toolchain and mutable
`ubuntu-latest`, `macos-latest`, and `windows-latest` runner labels. The release
provides no per-asset signature or SLSA provenance.

The `yr scan` CLI supports compiled rules, JSON/NDJSON output, internal timeout,
one-thread operation, match limits, and disabled memory mapping. Its v1.20.0
NDJSON serializer can expose an offset plus a byte-count marker while retaining
zero matched bytes when `--print-strings=0` is used.

## Decision

Freeze `yara-x-contract-v1` as a source, artifact-strategy, rule/module,
invocation, output, resource, compatibility, and revocation contract.

Use YARA-X v1.20.0 at commit
`60ad06971467029e77967e59d580cbbe85a1474d`. Record all six official uploaded
assets and their GitHub digests as public metadata. Exclude the C-API asset and
treat every CLI asset as a candidate only. Production artifacts must instead
be rebuilt per target from a separately digest-pinned source archive in a
locked environment, receive dependency closure, SBOM, provenance,
reproducibility disposition, vulnerability/license review, and an Impresari
signature. This checkpoint does not download that archive or any asset.

The first ruleset must be project-owned, separately content-addressed,
reviewed, licensed, compiled, signed, expiring, and revocable. Version 1 permits
only literal text and hex patterns with the frozen small modifier/condition
surface. It permits no imports/modules, includes, external variables, regular
expressions, base64, XOR, custom metadata input, repository rules, or in-job
updates. Every rule must pass the frozen YARA-X compatibility corpus; invalid
rules cannot be ignored and relaxed regular-expression behavior is forbidden.

The eventual worker may invoke only the exact `yr scan` argument template in
the profile against one compiled ruleset and one exact private staged regular
file. It must use an empty private `HOME`, one thread, no memory mapping, five
seconds of engine timeout inside the stricter external runner deadline, and
bounded match, file, and output limits. It must accept exactly one bounded UTF-8
NDJSON object, validate the staged path and closed fields, derive byte length
only from the zero-byte marker, discard the marker and raw output, and emit
only digest-bound normalized ranges. Unknown, duplicate, malformed, truncated,
extra-line, raw-byte, XOR/plaintext, path, or accounting states fail closed.

## Consequences

- Official release digests are precise evidence but not Impresari artifact
  admission.
- The closed invocation prevents repository content from selecting arguments,
  rules, modules, configuration, directories, or network behavior.
- Zero-byte string rendering preserves offsets and lengths without retaining
  matching repository bytes.
- YARA-X API and rule differences cannot silently inherit legacy YARA
  identities or fixtures.
- Linux execution remains closed until an exact published production IAR-1B
  profile, admitted artifact/ruleset, live adapter contract, hosted evidence,
  and applicable independent review all pass. macOS and Windows remain
  independently closed.

## Alternatives

- Admit official binaries from GitHub digests alone: rejected because digest
  metadata does not supply per-asset signing, locked builds, provenance, or
  reproducibility.
- Link the YARA-X library into the ordinary process: rejected because hostile
  parsing must remain inside an independently admitted analyzer boundary.
- Retain matched strings in JSON: rejected because offsets and lengths can be
  recovered without retaining repository bytes.
- Permit modules in the first ruleset: deferred because modules add deep
  hostile-format parsers and need separate review.
- Compile repository rules at runtime: rejected because hostile repositories
  cannot define analyzer behavior.

## Activation Gate

This ADR adds no artifact or execution authority. Revisit before downloading a
source/archive asset for a build, creating or signing an executable or ruleset,
implementing the live parser, running a compatibility corpus through YARA-X,
executing `yr`, accepting repository-derived analyzer input, using network or
credentials, or claiming IAR-2, production, platform support, detection
quality, safety, or malware-free status.

## Official Sources

- [YARA-X v1.20.0 release](https://github.com/VirusTotal/yara-x/releases/tag/v1.20.0)
- [Pinned release workflow](https://github.com/VirusTotal/yara-x/blob/v1.20.0/.github/workflows/release.yaml)
- [Pinned workspace manifest](https://github.com/VirusTotal/yara-x/blob/v1.20.0/Cargo.toml)
- [CLI commands](https://virustotal.github.io/yara-x/docs/cli/commands/)
- [Configuration file](https://virustotal.github.io/yara-x/docs/cli/config-file/)
- [YARA-X rule differences](https://virustotal.github.io/yara-x/docs/writing_rules/differences-with-yara/)
- [YARA-X modules](https://virustotal.github.io/yara-x/docs/modules/)
