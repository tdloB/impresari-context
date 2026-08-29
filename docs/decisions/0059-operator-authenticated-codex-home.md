# ADR-0059: Operator-Authenticated Codex Home Boundary

- Status: Accepted
- Date: 2026-08-28
- Deciders: Impresari Context maintainer and founder
- Related: [ADR-0055](0055-codex-ephemeral-guided-delivery.md),
  [CI-3b PRD](../product/ci-3b-codex-guided-delivery-prd.md), and
  [CI-3b ARD](../architecture/ci-3b-codex-guided-delivery-ard.md)

## Decision

Allow explicit Codex guided delivery to use one operator-supplied, dedicated,
already-authenticated `CODEX_HOME`. The path must be absolute, canonical, a
real directory rather than a symlink, and separate from the disposable runtime
parent. The apply command names it explicitly after preview and still requires
`--apply` plus the expected packet identity.

Impresari Context never discovers the user's ordinary Codex home, reads or
copies credential files itself, exports credential state, initiates refreshes,
or deletes the supplied home. Codex owns its supported managed-ChatGPT token
lifecycle inside that dedicated directory. Impresari confirms only the
source-free `account/read` result before creating an ephemeral thread.

## Consequences

- Authentication becomes an explicit operator-owned prerequisite instead of
  an implicit inherited capability.
- Codex may maintain its own authentication state in the dedicated home; that
  state is outside Impresari's owned cleanup boundary.
- The disposable current directory remains separate and is deleted after each
  attempt. The source workspace and cache are never passed to App Server.
- A missing, unauthenticated, symlinked, overlapping, or unsupported home
  fails before packet delivery.
- L3 admission still requires two successful live completions with source
  immutability, runtime cleanup, no granted authority, and bounded receipts.
