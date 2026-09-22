# ADR-0166: Name the Holder When a Writer Lock Is Busy

- Status: Accepted
- Date: 2026-09-21
- Amends: [ADR-0006](0006-local-cache-and-storage.md), whose writer lock already
  admits diagnostic metadata that cannot authorize breaking a live lock

## Context

`Quality / ubuntu-24.04 / Rust 1.96.0` failed once on PR #318, a Dependabot bump
of a workflow action that touches no Rust:

```text
hostile_repository_text_remains_untrusted_and_never_enters_audit_or_errors
panicked at crates/context-engine/tests/security_adversarial.rs:171:44
audit: CacheError { code: WriterBusy, source: None }
```

The test drops the engine and reopens the audit store. `WriterBusy` says the
store's exclusive lock on `audit/lock` was still held. It says nothing about by
whom, and that is the whole question: a handle this process still holds is a
leak or a double open in our own code, while another process is the lock doing
its job.

Reading the code does not settle it. `AuditStore` holds its lock as a `File`
with no `Drop` impl, so closing the file releases it; the engine owns the store
by value; neither crate spawns a thread or a process; `WorkspaceCache` locks a
different path; and each test root carries the process id and an atomic
sequence, so two tests cannot share one.

Nor does repetition. The same job passed on main that day and on a plain re-run.
A probe then ran the same binary 100 times on that Linux job — 50 at default
parallelism, 50 single-threaded, each confirming it executed — with no failure,
beside 50 local runs across 1.96.0 and 1.98.0. One failure in 150 clean
executions puts the rate under about 1%.

So the next occurrence has to explain itself, because we cannot summon it.

## Decision

1. **A `WriterBusy` error carries who holds the lock**, as its error source:
   this process, another process, or unknown.
2. **On Linux the holder comes from `/proc/locks`**, matched by the lock file's
   device and inode, with the recorded pid compared against this process.
   Elsewhere the answer is unknown, which is the honest value rather than a
   guess.
3. **A line that does not parse is skipped, not interpreted.** A wrong answer
   here would send a reader after the wrong bug, which is worse than no answer.
4. **Nothing else changes.** The category stays `WriterBusy`, its message stays
   "cache writer is busy", and the holder never authorizes breaking a live lock,
   exactly as ADR-0006 requires.
5. **Source-free.** The detail is a category. No path, no identifier, no
   contents.

## Consequences

- **The next occurrence decides it.** "Held by this process" means our own
  handle outlived what closed it, and points at the engine's drop. "Held by
  another process" means the environment, and points at the runner. The failure
  text reaches CI logs unchanged in shape, since the detail rides in the error's
  source.
- **No behaviour is masked.** A bounded retry would have hidden this failure
  whether or not it addressed the cause, and would have turned a fail-closed
  lock into a stall. This leaves the failure exactly as loud as it was.
- **The three lock sites report alike:** the audit store, the workspace cache,
  and the workspace deletion path all answer the same question the same way.
- **Linux only, for now.** macOS and Windows expose no equivalent table, so
  they answer unknown. The failure has only been seen on Linux.

## Alternatives considered

**Retry the lock briefly before failing.** Rejected. It makes the symptom
disappear without evidence about the cause, and if the cause is a double open in
our code it converts a correct, loud failure into a silent stall.

**Record the holding pid in the error.** Rejected as unnecessary: the decisive
bit is whether the holder is this process, and a pid in an error message invites
a reader to act on a number that may already be stale.

**Write a pid file beside the lock.** Rejected by ADR-0006 already: metadata may
aid diagnostics but is not authority, and a stale pid file is worse than no
file. The kernel's own table needs no upkeep.

**Leave it and wait.** Rejected. The failure is rare enough that it will recur
long after the context is gone, and the report as it stands does not say enough
to act on.
