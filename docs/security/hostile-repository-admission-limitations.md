# Hostile-Repository Admission Step 1 Limitations

- Status: default-branch Step 1 release-readiness disclosure.
- Applies to: ADR-0073 HRA-0 through HRA-4.
- Does not apply retroactively to: published `v0.1.0` artifacts.

## What Step 1 does

Step 1 inventories a bounded set of security-relevant artifacts without
executing them, emits two deliberately narrow exact-evidence observation
corpora, plans required analyzer coverage, accepts only closed synthetic
analyzer results after the existing zero-capability normalization boundary,
assembles immutable assessments, and offers a separate pure deterministic
reference evaluator.

All repository content and analyzer-derived records remain untrusted data.
Outputs are snapshot- and digest-bound, missing analysis stays visible, and the
reference evaluator can return only `blocked`, `manual_review_required`,
`analysis_incomplete`, or `isolated_execution_eligible`.

## What Step 1 does not do

Step 1 includes no:

- malware scanner, ClamAV engine, YARA engine, or signature/database lifecycle;
- analyzer discovery, installation, loading, process launch, or execution;
- network lookup, reputation-provider query, telemetry, or artifact upload;
- deep PE, DLL, MSI, PowerShell, archive, or other hostile-format parser;
- sandbox, container, VM, quarantine runner, repository execution, or behavior
  observation;
- AI/model risk scoring, exception grant, approval, final risk acceptance, or
  ordinary-host execution authorization; or
- claim that a repository is clean, safe, trusted, approved, or malware-free.

`isolated_execution_eligible` is only a reproducible policy classification that
names one prospective disposable-quarantine profile. No such runner exists in
Step 1, and the classification does not create, authorize, or invoke one.

## Interpretation rules

- Zero findings does not establish complete coverage or safety.
- `complete` means the frozen admitted inputs and mandatory coverage records are
  complete; it does not mean the repository is safe.
- `unknown`, exclusions, unsupported syntax, stale data, and unavailable or
  failed mandatory analysis remain first-class restrictive evidence.
- Static artifact and execution-surface observations identify syntax or format
  facts only. They do not establish intent, exploitability, or runtime behavior.
- A consumer may adopt a stricter policy. Final risk acceptance belongs to an
  authorized human outside Impresari Context.

## Step 2 boundary

ADR-0074 analyzer execution and ADR-0075 quarantine execution remain proposed,
separately gated scopes. Step 2 may not begin merely because Step 1 is ready;
it requires the explicit founder approval named by the HRA-5 gate plus the
applicable threat-model, privacy, platform, supply-chain, and independent-review
requirements.
