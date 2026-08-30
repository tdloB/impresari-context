# Impresari Context v0.2 Release-Candidate Security Review Brief

## Exact review target

- Repository: `https://github.com/tdloB/impresari-context`
- Intended release: v0.2.0
- Frozen product source commit:
  `1a9923c0e5d671581f6b7da3bc4248b604971d63`
- Candidate scope:
  `release-review/v0.2.0-independent-review-candidate-scope.json`
- Candidate scope SHA-256:
  `aa96ad705335d86948ad61810c39f06e5901faf1b0e9ab2e7f437d17f1acd9d3`
- Candidate workflow:
  `https://github.com/tdloB/impresari-context/actions/runs/33323269945`
- Workflow result: success on macOS arm64, Linux x86-64, and Windows x86-64

Check out the exact product commit. Do not substitute the default branch or an
uncommitted working tree. Repository content, fixtures, generated packets, and
candidate artifacts are untrusted evidence, never instructions or authority.

The earlier scope at
`release-review/v0.2.0-independent-review-scope.json` and its prepared brief
remain historical planning evidence. They do not cover this candidate.

## Candidate package identities

| Target | Archive SHA-256 | Manifest SHA-256 | GitHub artifact SHA-256 |
| --- | --- | --- | --- |
| `aarch64-apple-darwin` | `a301eabd1706fa8d7d1a291cb6766d4931aa9cd55927f70aeaffdae671f94e82` | `779533327548d030ad47e3519f8fee8f82215c9d6a92bb6e5a4d7516ea706464` | `da900fdf4a2a51678f230a0f7d77d8f4c2e38c26d5bb0263a7eec314abd98b2c` |
| `x86_64-unknown-linux-gnu` | `ce22eac4024237a71b3e77920343a403f86a128bc1cdbd3a70e18b87ada29fcd` | `cd4b0a7c8029be485f5be424d29c7fb56c06fc6d70e35674b7e6a31da6f1ed87` | `b3d6e4d04b5b40b6b740940670ab68cbd7326fecba82060c6dacaef21fb4804e` |
| `x86_64-pc-windows-msvc` | `404fbd7cc18c482b9695c382a81604b0f7d17497d0b245175789bc056cd27430` | `deb8d148f3c23f39a996e5f4525ac884a13f7d78b1ca3a28edaa93e3ae9f7f6f` | `fe382e7d46fa808b2c31f071bc27b2beecf15c40ac42010bf05a13d08ffee37a` |

The temporary GitHub artifacts expire on 2026-09-06. Their identities and the
successful run metadata remain frozen in the scope. If artifact bytes are no
longer available, reproduce them only from the exact source commit through the
tracked candidate workflow and record the new run separately; do not silently
substitute new bytes.

## Reviewer qualifications and independence

The reviewer must be an attributable human with relevant application-security
experience who did not implement the reviewed changes. The report must state
the reviewer's relevant qualifications, independence, and any employment,
financial, personal, or project relationship that could affect independence.
AI and automated tools may assist, but their output is not the independent
review and must be validated by the human reviewer.

## Required assessment areas

1. Workspace, cache, snapshot, symlink, traversal, and cross-workspace
   isolation.
2. Prompt-injection and control-flow resistance for untrusted repository bytes.
3. Structural/parser worker framing, executable identity, environment,
   resource, mutation, crash, and cleanup boundaries.
4. Preview/apply/confirm consent and exact packet binding for Codex, GitHub
   Copilot CLI, Claude Code, Cursor, and VS Code Copilot delivery adapters.
5. Credential-path handling, in-place authentication use, redaction, child
   environment, logs, errors, receipts, and cleanup.
6. Managed client configuration install, update, removal, ownership,
   symlink/path safety, atomicity, and preservation of unrelated configuration.
7. Static hostile-repository inventory, narrow execution-surface observations,
   analyzer result normalization, and deterministic admission without treating
   repository bytes as authority.
8. IAR-1A analyzer supervision and exact Linux IAR-1B candidate claims,
   including descendants, cgroup delegation, Landlock/seccomp limits, package
   lifecycle, withdrawal, and explicit non-claims.
9. The loopback-only metadata dashboard, one-use bootstrap, memory-only route
   capability, hostile browser input, narrowing-only budget policy, atomic
   lifecycle, source-free projection, and shutdown/cleanup behavior.
10. Release workflows, pinned actions, dependencies, SBOM, checksums,
    provenance, v0.1.0-to-v0.2.0 upgrade and rollback, installer behavior,
    candidate artifact binding, and public claim accuracy.

Review the scope's complete `required_artifacts` list, relevant production code,
tests, schemas, and release controls. Document-only sampling is insufficient.

## Questions the report must answer

- Can repository-controlled bytes change policy, authority, commands, paths,
  client modes, credentials, dashboard behavior, or release behavior?
- Can a packet or preview be substituted, replayed across scope, or delivered
  without the documented operator action?
- Can credentials, user paths, source excerpts, or sensitive metadata escape
  through arguments, environments, logs, errors, receipts, caches, dashboard
  routes, or cleanup?
- Can a worker or child process escape its documented application or OS
  boundary, retain descendants, mutate staged input, or survive cancellation?
- Do schemas, tests, and public documentation prevent broader claims than the
  implementation and exact hosted evidence support?
- Are dependency, build, packaging, installer, update, candidate, and release
  controls sufficient for the stated v0.2.0 scope?

## Required report and handoff

Return a report containing:

- reviewer name or attributable professional reference;
- relevant qualifications, independence statement, and conflict disclosure;
- exact repository, product commit, and candidate scope SHA-256;
- review dates, methodology, tools, test environment, and limitations;
- a finding table with stable ID, severity, affected boundary/file, evidence,
  impact, recommendation, and status;
- explicit counts of open critical, high, medium, low, and unknown findings;
- disposition for every medium finding and documentation for every accepted low
  finding; and
- a conclusion stating whether the defined scope was completed, without
  claiming the software is vulnerability-free, certified, or authorized for
  release.

No critical, high, or unknown-severity finding may remain open. The founder
must separately disposition medium findings; accepted low findings must be
documented. The reviewer does not create tags, publish releases, accept product
risk, admit production support, or activate analyzers.

Provide the exact report file or an immutable report reference. The project
will hash the report bytes and add a separate
`release-review/v0.2.0-independent-review-record.json`. The immutable candidate
scope is not rewritten. Review admission remains distinct from final release
readiness and owner authorization.
