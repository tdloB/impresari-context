# Independent Security And Release Review Guide

Impresari Context requires a human review independent of the primary
implementation process before publishing `v0.1.0`. Automated scanners and
AI-assisted review provide useful evidence, but they do not satisfy this gate.

## Reviewer qualification

The reviewer should be able to demonstrate practical experience reviewing
secure software, Rust or systems software, local trust boundaries, or software
supply-chain controls. A paid certification is not required. The reviewer must
not have been the primary author of the implementation under review.

The maintainer records the reviewer's name or stable professional identity,
relevant experience, review date, source commit, scope, limitations, and
disposition of findings. Private contact information does not need to be
published.

## Required scope

The review must cover at least:

- the threat model and every public-release security gate;
- workspace containment, path and symlink handling, and source immutability;
- treatment of repository content as untrusted data;
- packet evidence integrity, completeness accounting, and stale-state failure;
- cache, export, session-handle, extension, and MCP boundaries;
- dependency, CI, SBOM, packaging, checksum, attestation, and release controls;
- public documentation for installation, interfaces, security reporting,
  supported versions, residual risks, and release verification.

## Evidence to provide

The reviewer receives the exact proposed release commit, this repository's
verification records, the output of `./scripts/check.sh`, the native release
candidate run, CodeQL and OpenSSF results, and the packaged archives. Findings
must identify severity, evidence, recommended disposition, and whether they
block release.

Before release, every blocker must be fixed and reverified. Other accepted
risks require a written, time-bounded maintainer disposition. The final review
record may redact exploit details while a vulnerability remains under embargo.

## OpenSSF questionnaire boundary

This review can support the project's independent-release gate. It does not by
itself prove that the primary developer meets OpenSSF's secure-development
knowledge criteria. Those criteria should remain `Unmet` until a qualifying
human primary developer or documented ongoing security-review relationship is
in place.
