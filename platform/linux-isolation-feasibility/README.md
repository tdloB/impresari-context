# Linux IAR-1B feasibility probe

This directory contains the source-free, synthetic-only ADR-0074 Linux
feasibility probe. It is a test harness, not a production launcher and not an
analyzer.

The first checkpoint verifies host capabilities and a narrow `x86_64`
primitive composition:

- `no_new_privs` is effective;
- Landlock exposes only one read-only synthetic job directory;
- a default-deny, architecture-pinned seccomp filter denies network and
  descendant creation;
- unrelated inherited descriptors are closed; and
- no path-backed filesystem writes are allowed, giving this profile a hard
  aggregate writable-filesystem budget of zero.

CPU, memory, process-count, exact kill, empty-state, timeout, crash, cleanup,
and cross-job tests remain false until a delegated cgroup v2 leaf is available
and the complete resource/lifecycle suite passes. A host without the required
ABI, controllers, delegation, or verifier reports `unsupported`; it never
falls back to an IAR-1B claim.

The probe never reads repository content, executes an analyzer, contacts a
network provider, accesses credentials, or adds host authority.
