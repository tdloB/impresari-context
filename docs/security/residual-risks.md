# Local Security Residual Risks

Status: required Slice A disclosure; not a claim that the product is released.

- The engine trusts the operating system and the account running it. An
  administrator, compromised process with the same account, debugger, kernel,
  or hostile endpoint tooling can inspect memory, change files between checks,
  or bypass local controls.
- Cache and export permissions reduce ordinary cross-user disclosure but do not
  defeat administrator access, inherited ACLs, snapshots, backups, filesystem
  history, crash dumps, or storage forensics. Platform-native ACL verification
  remains part of the release matrix.
- Source access is logically and mechanically read-only within the runtime, but
  the host does not provide an immutable mount. Concurrent external mutation
  may produce a stale or partial result and sufficiently adversarial races
  cannot be eliminated without stronger host isolation.
- Exact evidence necessarily returns authorized source excerpts. Callers must
  treat packet data as sensitive and repository-controlled, even when it looks
  like instructions, terminal control data, or trusted metadata.
- Secret-like data is excluded from default audit and safe errors, but an
  authorized exact query may intentionally retrieve it. This engine is not a
  secret scanner, DLP system, malware sandbox, or authorization provider.
- Network and process execution are absent from production code and the macOS
  suite passes with networking denied. Native Linux and Windows denial evidence
  is still required before release.
- SQLite cache corruption fails closed, but disk, memory, library, and hardware
  faults can still make the local service unavailable. Derived cache may be
  deleted and rebuilt; audit retention and backup policy belong to the operator.
- Unicode, case sensitivity, links, mount behavior, and file identity differ by
  filesystem and platform. Unsupported or ambiguous objects are skipped or
  rejected, which may reduce completeness rather than silently grant authority.
- Dependencies are locked, inventoried, and scanned, but maintainer compromise,
  malicious build infrastructure, and unknown vulnerabilities remain possible.
  Release provenance is required for `v0.1.0`. Independent review remains a
  deferred assurance target before `v1.0.0`, or earlier after a qualifying
  trust-boundary expansion. This release has not undergone an independent
  third-party security audit.
