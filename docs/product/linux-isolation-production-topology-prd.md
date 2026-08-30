# Linux IAR-1B Production Topology PRD

- Status: Proposed; founder decision required
- Date: 2026-08-30
- Owner: Aaron Boldt
- Decision: ADR-0078
- Parent: ADR-0074 and ADR-0077

## Problem

The admitted Linux candidate evidence uses a CI-created transient system
service solely to establish a delegated cgroup v2 subtree. The synthetic
supervisor and worker are unprivileged after that boundary exists. A normal
installation still needs a defined way to obtain the same delegation without
silently running Impresari as root, adding a privileged daemon, or treating
`sudo` as a fallback.

Candidate evidence is therefore not yet production support. IAR-2 must remain
closed until the installation topology is selected, implemented, and passes
the same source-free Tier A and lifecycle corpus on every claimed target.

## User Outcome

An admitted Linux installation either obtains one exact, contained delegated
subtree through an approved platform path or reports `unsupported` before
staging or launching a worker. The user is never surprised by a password
prompt, privilege escalation, persistent background service, or weaker
application-only fallback.

## Decision Options

### Option A — Existing systemd user-manager delegation (recommended first)

Use only an already-running per-user systemd manager whose parent has delegated
the required cgroup v2 controllers. Impresari creates a foreground transient
user service/scope below that existing boundary and manages only its subtree.

- No root, sudo, package-installed policy, or persistent Impresari service.
- Smallest authority and simplest uninstall story.
- Availability varies by distribution, login/session setup, containers, and
  whether CPU, memory, and pids controllers reach the user manager.
- Missing user manager, controller, or effective delegation is `unsupported`.

### Option B — Opt-in administrator-provisioned delegation

Install a root-owned declarative systemd unit and narrowly scoped authorization
policy that creates a transient delegated subtree for the invoking user. The
analyzer supervisor remains unprivileged below it.

- Can cover headless and locked-down systems where Option A is unavailable.
- Adds privileged installation, policy review, upgrade/removal duties, and a
  new local authorization surface.
- Must not become a long-running daemon or a general command launcher.
- Requires a separate explicit founder decision and dedicated threat model.

### Option C — Externally managed delegated subtree

An administrator or container/orchestration platform supplies an already
delegated subtree through a closed launch contract. Impresari validates and
uses it but never creates or broadens it.

- Appropriate for enterprise or CI environments with existing cgroup policy.
- Preserves least privilege inside Impresari.
- Requires operator integration and is not a general desktop quickstart.

### Rejected As Initial Defaults

- Automatic `sudo` or `pkexec` fallback.
- A privileged or always-running Impresari daemon.
- Direct writes into a systemd-owned cgroup outside a delegated unit.
- Treating Docker/Podman availability as equivalent evidence.
- Falling back to IAR-1A while reporting IAR-1B or analyzer readiness.

## Recommendation

Select Option A as the first production-feasibility candidate and Option C as
the explicit externally managed profile. Keep Option B as a separately approved
future expansion only if real user demand shows Option A/C coverage is
insufficient. This preserves a rootless default and makes unsupported systems
honest rather than privileged by surprise.

## Acceptance Gates

- Exact supported systemd, cgroup v2, kernel, architecture, distribution, and
  package identities are frozen.
- A source-free preflight verifies the delegated root, controller availability,
  single-writer ownership, containment, and cleanup before any worker launch.
- The complete primitive, resource, fault, cleanup, and cross-job corpus passes
  below the selected topology on every claimed target.
- Install, upgrade, rollback, logout/login, cancellation, crash, and uninstall
  leave no stale service, policy, cgroup, or source bytes.
- Missing or changed prerequisites withdraw the production claim.
- No real analyzer or repository content participates in topology admission.

## Decision Needed

Approve the rootless-first Option A plus explicit Option C direction, or
authorize the broader Option B administrator-installed boundary for the first
production slice. This choice materially changes installation authority and
cannot be inferred from candidate CI authorization.
