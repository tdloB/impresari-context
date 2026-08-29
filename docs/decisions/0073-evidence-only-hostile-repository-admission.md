# ADR-0073: Evidence-only hostile-repository admission

- Status: Accepted; Step 1 complete (HRA-0 through HRA-5)
- Date: 2026-08-26
- Scope: Security artifact inventory, assessment, coverage, and deterministic
  stage-eligibility contracts

## Context

Impresari Context already treats a repository as hostile, reads it through a
bounded capability boundary, and prevents repository text from granting
execution or network authority. It does not currently identify a complete set
of security-relevant execution surfaces, account for external analyzer
coverage, or produce a repository admission assessment.

Adding a traditional malware scanner directly to the core would enlarge the
trusted computing base, introduce hostile native/file-format parsers, require
signature lifecycle and network design, and encourage users to interpret a
zero-detection result as proof of safety. The existing extension contract in
ADR-0013 deliberately supports declarations and bounded hostile-output intake
without loading extension code.

The product needs a slow first step that makes later malware analysis possible
without crossing the current execution boundary.

## Decision

Add an evidence-only hostile-repository admission foundation with these durable
rules:

1. The Context core may inventory security-relevant artifact classes and narrow
   execution-surface observations through existing read-only access.
2. Every observed finding is snapshot-bound and recoverable to exact evidence;
   binary inventory is bound to exact artifact hashes.
3. Required analysis and coverage state are canonical and independent from the
   number of findings.
4. The core may normalize separately produced, digest-bound analyzer output as
   untrusted derived data but may not load or execute analyzer artifacts.
5. Deep PE, DLL, MSI, PowerShell, malware, dependency, archive, or other hostile
   parsing not separately admitted to the core runs only in the future Isolated
   Analyzer Runner.
6. A pure deterministic evaluator may map an immutable assessment and immutable
   consumer policy to `blocked`, `manual_review_required`,
   `analysis_incomplete`, or `isolated_execution_eligible`.
7. The evaluator is logically outside the evidence core and has no filesystem,
   network, process, model, credential, exception, or approval authority.
8. `isolated_execution_eligible` can target only an approved Disposable
   Quarantine Runner profile. It never authorizes ordinary-host execution.
9. The contracts contain no `clean`, `safe`, `trusted`, `approved`, or
   `malware_free` state.
10. Windows-oriented static artifact recognition is initial scope on macOS and
    Linux, while deep hostile-format parsing remains isolated.

## Consequences

### Positive

- The product gains useful repository-security evidence before accepting the
  risk of executing third-party scanners.
- Scanner, policy, and quarantine components can evolve behind stable snapshot,
  finding, coverage, and decision contracts.
- Windows repositories are not deferred merely because initial hosts are macOS
  and Linux.
- Missing coverage cannot be hidden by a successful or zero-detection scan.
- The trusted Context core remains read-only and non-executing.

### Costs

- Step 1 cannot detect known malware without separately supplied analyzer data.
- Users may receive `analysis_incomplete` frequently until step 2 exists.
- Security-specific schemas, fixtures, rules, and evaluation increase the
  maintenance surface.
- A separate policy evaluator adds a boundary that must remain semantically
  aligned with assessment schemas.

## Alternatives Considered

### Load ClamAV or YARA in the Context process

Rejected because hostile parsing and scanner dependencies would share the
core's workspace authority and fault domain.

### Treat malware scanning as the complete admission decision

Rejected because signatures do not cover install hooks, CI misuse, privileged
containers, credential access, prompt injection, novel malware, or missing
analysis.

### Let an AI model score repository risk

Rejected as an authority mechanism because the decision must be reproducible,
explainable, resistant to repository prompt injection, and fail closed.

### Defer all Windows support

Rejected because cross-platform static identification can be designed now, and
deferral would permit unsafe POSIX assumptions to harden in public contracts.

### Allow the core to decide final risk acceptance

Rejected because final acceptance belongs to the consumer and authorized
human under the existing product boundary.

## Verification

- No repository process or analyzer artifact is started by the Context core.
- Network-denied and source-immutability suites remain passing.
- Every observed finding resolves to exact matching snapshot evidence.
- Stale, mismatched, malformed, excessive, or authority-claiming analyzer
  envelopes are rejected or quarantined.
- Coverage remains incomplete when mandatory analysis is absent or failed.
- Decision truth tables are deterministic and monotonic toward restriction.
- Windows artifact recognition passes on macOS, Linux, and Windows fixtures
  without invoking Windows execution facilities.
- No schema or human rendering presents a safety or malware-free claim.

## Implementation Gate

The founder authorized HRA-0 and then HRA-1 on 2026-08-29. HRA-1 adds only a
bounded, read-only artifact inventory and explicit exclusions under the frozen
HRA-0 contracts and resource profile. It adds no findings, policy decisions,
analyzer execution, network access, uploads, deep hostile-format parsing, or
repository execution.

The standing roadmap directive and the founder's confirmed HRA-2 boundary admit
narrow, deterministic observations for safely admitted formats. The first
implemented corpora recognize exact npm lifecycle keys under a strict
`package.json` top-level `scripts` object and exact `privileged: true` service
keys in a deliberately canonical Compose layout. Values remain uninterpreted
and unretained. HRA-3 deterministically plans unavailable mandatory coverage,
accepts only closed synthetic result payloads after ADR-0013 zero-capability
normalization, and assembles immutable assessments. It does not discover or run
analyzers. HRA-4 evaluates immutable records in a separate pure library with no
I/O, exception, approval, or execution authority; restrictive effects dominate
and eligibility can name only a quarantine profile. ADR-0074 and ADR-0075
remain proposed and independently gated.

HRA-5 passed the exact-commit Tier A release-candidate matrix at commit
`12a46c1b9d934830450019470c3a74c9a1b47bf8` in GitHub Actions run 33266846683.
All candidate artifacts were temporary and no release was published. ADR-0073
Step 1 is complete; ADR-0074 implementation still requires explicit founder
approval.

## Review Triggers

Review or supersede this decision before:

- loading or executing any analyzer artifact;
- adding network or provider credentials to the Context core;
- uploading a file or source-derived metadata;
- admitting a deep hostile-format parser into the core;
- allowing a decision to authorize ordinary-host execution;
- adding AI/model output to admission policy evaluation;
- changing final risk acceptance ownership; or
- claiming that an assessment proves a repository is safe.
