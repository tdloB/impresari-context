# Local MCP and Release-Candidate Traceability

Date: 2026-08-22

Status: implemented and locally gated. Native hosted candidate rehearsal is
required before these three targets can be recorded as release evidence.

## Local MCP

| Requirement | Evidence |
| --- | --- |
| Local stdio only | `context-mcp` uses injected `BufRead`/`Write`; production security-boundary checks reject network APIs and network-capable dependencies. |
| Protocol lifecycle | Initialization, initialized notification, ping, tool listing, tool calls, duplicate IDs, malformed JSON, batches, invalid fields, and oversized lines have direct tests. |
| Bounded transport | Input is consumed incrementally, retained bytes are capped at 1 MiB, and one process retains at most 10,000 request IDs. |
| Stdout integrity | The transport test parses every emitted line as JSON; clean-launch rehearsal also parses every response. Diagnostics use stderr. |
| Authority neutrality | Launch arguments fix workspace, cache, consumer, and role. Tool results state that no orchestration or filesystem authority was added. |
| Semantic equivalence | The frozen evaluation sends the same plan to the direct engine and MCP and requires identical packets. |
| Untrusted source and immutability | The equivalence fixture includes instruction-like hostile repository text and verifies exact source bytes remain unchanged. |

## Release candidates

| Requirement | Evidence |
| --- | --- |
| Exact source | Manual workflow requires and verifies a full 40-character commit SHA. |
| Least privilege | Workflow permissions are `contents: read`; checkout credentials are not persisted and no publishing/signing credentials are present. |
| Native targets | Candidate matrix covers macOS ARM64, Linux x86-64, and Windows x86-64 MSVC. |
| Package contents | Each archive contains the CLI, structural worker, MCP process, notices, security/support documents when present, and the SPDX SBOM. |
| Integrity | Per-file SHA-256 values and sizes are recorded in `MANIFEST.json`; each archive has a separate SHA-256 file. |
| Clean rehearsal | Rehearsal extracts into a fresh temporary root, validates the manifest and archive digest, runs the CLI, and performs a real MCP initialize/tools exchange. |
| No publication | Workflow output is a seven-day candidate artifact only. It does not create a tag, GitHub release, package publication, or signature. |

## Explicitly deferred

- Loading or launching third-party extension artifacts.
- Granting extensions filesystem, process, network, environment, model, or
  cache access.
- Remote MCP/HTTP transports or durable multi-client sessions.
- Public tags, releases, package publication, or signer identity claims.

The first two items require a new founder review, ADR, and threat-model update
before implementation.
