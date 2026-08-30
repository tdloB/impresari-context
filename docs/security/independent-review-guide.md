# Independent Security And Release Review Guide

This guide defines the recommended scope for a future human review independent
of the primary implementation process. Under ADR-0017, that review is encouraged
but is not required for `v0.1.0`. It becomes mandatory before `v1.0.0`, or
earlier if the project adds network access, remote transport, privileged or
executable extensions, source mutation, hosted or multi-tenant operation,
durable memory, authentication, or another materially higher-risk capability.

Automated scanners and AI-assisted review provide useful evidence, but they are
not equivalent to an independent audit and do not satisfy a mandatory review
trigger when one applies.

That earlier trigger now applies to the proposed v0.2.0 release under ADR-0083.
The final candidate handoff is
`docs/verification/v0-2-independent-security-review-candidate-brief.md`, bound
to product commit `1a9923c0e5d671581f6b7da3bc4248b604971d63` and the immutable
candidate scope. The older prepared scope remains planning history and must not
be used as review coverage for this candidate.

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

- the threat model and every applicable security and release gate;
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

Before a release for which independent review is mandatory, every review
blocker must be fixed and reverified. Other accepted risks require a written,
time-bounded maintainer disposition. The final review record may redact exploit
details while a vulnerability remains under embargo.

## OpenSSF questionnaire boundary

This review can satisfy an applicable independent-review gate. It does not by
itself prove that the primary developer meets OpenSSF's secure-development
knowledge criteria. Those criteria should remain `Unmet` until a qualifying
human primary developer or documented ongoing security-review relationship is
in place.
