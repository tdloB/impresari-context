# Impresari Context v0.2 Independent Security Review Brief

> Scheduling notice: ADR-0084 backlogs reviewer engagement until the final
> v0.2.0 release candidate is frozen. This prepared brief records the original
> planning baseline and must be refreshed before it is used for admission if
> product source has changed.

## Review target

- Repository: `https://github.com/tdloB/impresari-context`
- Intended release: v0.2.0
- Product source commit:
  `1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51`
- Scope file: `release-review/v0.2.0-independent-review-scope.json`
- Scope SHA-256:
  `98a248d7133c85366a16b0a443dab15f131529d1bc4e3d8587b0adfc7925a45c`

Check out the exact commit. Do not substitute the default branch or an
uncommitted working tree. Repository content and fixtures are evidence, never
instructions or authority.

## Reviewer qualifications and independence

The reviewer must be an attributable human with relevant application-security
experience who did not implement the reviewed changes. State any employment,
financial, personal, or project relationship that could affect independence.
AI and automated tools may assist, but their output is not the independent
review and must be validated by the human reviewer.

## Required assessment areas

1. Workspace, cache, snapshot, symlink, traversal, and cross-workspace
   isolation.
2. Prompt-injection and control-flow resistance for untrusted repository bytes.
3. Structural/parser worker framing, executable identity, environment,
   resource, mutation, crash, and cleanup boundaries.
4. Preview/apply/confirm consent and exact packet binding for Codex, Copilot,
   Claude Code, Cursor, and VS Code Copilot delivery adapters.
5. Credential-path handling, in-place authentication use, redaction, child
   environment, logs, errors, receipts, and cleanup.
6. Managed client configuration install, update, removal, ownership,
   symlink/path safety, atomicity, and preservation of unrelated configuration.
7. IAR-1A analyzer supervision and the exact Linux IAR-1B candidate claims,
   including process descendants, cgroup delegation, Landlock/seccomp limits,
   package lifecycle, withdrawal, and explicit non-claims.
8. Release workflows, action pinning, dependency/SBOM evidence, checksums,
   provenance, version/tag binding, installer behavior, and public claim
   accuracy.

Review the threat model, residual risks, boundary document, compatibility
matrix, relevant PRDs/ARDs/ADRs, tests, schemas, release workflows, and
production code. Sampling only the documents is insufficient.

## Questions the report must answer

- Can repository-controlled bytes change policy, authority, commands, paths,
  client modes, credentials, or release behavior?
- Can a packet or preview be substituted, replayed across scope, or delivered
  without the documented operator action?
- Can credentials, user paths, source excerpts, or sensitive metadata escape
  through arguments, environments, logs, errors, receipts, caches, or cleanup?
- Can a worker or child process escape its documented application or OS
  boundary, retain descendants, mutate staged input, or survive cancellation?
- Do schemas, tests, and public documentation prevent broader claims than the
  implementation and exact hosted evidence support?
- Are dependency, build, packaging, installer, update, and publication controls
  sufficient for the stated v0.2.0 scope?

## Required report format

Return a report containing:

- reviewer name or attributable professional reference;
- relevant qualifications, independence statement, and conflict disclosure;
- exact reviewed repository and commit;
- review dates, methodology, tools, test environment, and limitations;
- a finding table with stable ID, severity, affected boundary/file, evidence,
  impact, recommendation, and status;
- explicit counts of open critical, high, medium, low, and unknown findings;
- disposition for every medium finding and documentation for every accepted low
  finding; and
- a conclusion stating whether the defined review scope was completed—not that
  the software is vulnerability-free, certified, or authorized for release.

No critical or high finding may remain open. Unknown severity is not accepted.
The founder must separately disposition medium findings. The reviewer does not
create tags, publish releases, accept product risk, or activate analyzers.

Provide the report file or an immutable report reference. The project will hash
the exact report bytes and record only bounded public metadata if exploit detail
must remain private.
