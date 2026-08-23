# Changelog

All notable changes to Impresari Context are documented here. The project uses
[Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-08-22

### Added

- A local-first Rust engine that builds compact, task-specific context packets
  from an exact repository snapshot.
- Verifiable source evidence, deterministic packet identity, explicit budget
  accounting, and visible completeness, exclusion, truncation, and staleness
  reporting.
- Read-only workspace discovery with protections for path traversal, symlink
  escape, hostile repository content, cache separation, and source mutation.
- Lexical retrieval for eligible UTF-8 files and structural analysis for
  TypeScript, TSX, JavaScript, JSX, Python, and recognized strict-JSON
  configuration manifests. Other languages and JSON data files receive no
  structural support claim in this release.
- A command-line interface, neutral Rust library surface, reference client,
  consumer adapter contract, and local-only MCP server over standard I/O.
- Cross-platform CI, adversarial and conformance suites, release-candidate
  packaging, SBOM generation, dependency auditing, CodeQL, secret scanning,
  OpenSSF Scorecard, and bounded fuzz testing.

### Security

Security review status: This release has not undergone an independent
third-party security audit. It has passed the project's documented automated,
AI-assisted internal, native-platform, dependency, static-analysis,
secret-scanning, fuzzing, packaging, checksum, and provenance checks. These
controls reduce risk but are not a substitute for independent review.

- Repository content is always treated as untrusted data and receives no
  policy, execution, network, or orchestration authority.
- The initial release does not load executable extensions, grant privileged
  extension capabilities, provide remote MCP transport, or claim cryptographic
  signer identity for context packets.
- No publicly known vulnerability in Impresari Context is fixed by this first
  release.

[0.1.0]: https://github.com/tdloB/impresari-context/releases/tag/v0.1.0
