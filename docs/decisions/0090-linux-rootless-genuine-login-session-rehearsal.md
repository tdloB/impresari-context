# ADR-0090: Prove Rootless Reentry With A Disposable PAM Login User

- Status: Accepted; exact-host candidate evidence recorded
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

ADR-0080 left profile A partial because a hosted package process cannot prove
logout/login behavior. The missing property is not that a command can restart;
it is that a fresh real login receives a correctly delegated user manager and
preserves package identity without privileged installation or persistent
service state.

## Decision

Use one temporary non-lingering user and two loopback SSH sessions mediated by
PAM/logind on an ephemeral Linux test host. Run the existing bounded preflight
and original-synthetic rootless corpus as that unprivileged user in each
session. Require the first session and user manager to terminate before the
second login, then prove clean removal.

Privilege is restricted to ephemeral test setup and teardown; Impresari gains
no sudo path, service, policy, lingering state, or production installer change.

## Consequences

- The observation represents a genuine session boundary rather than a process
  restart.
- The live workflow has a narrow administrator setup surface that must never
  run on pull-request code or a persistent host.
- Hosted images without the required PAM/logind behavior report unsupported.
- A pass completes only profile A's package/reentry evidence; production and
  real analyzers remain separately gated.

## Alternatives

- Restart `systemd --user`: rejected because it is not logout/login evidence.
- Enable lingering: rejected because it intentionally prevents the lifecycle
  transition being measured.
- Use the runner account: rejected because terminating its user manager could
  terminate the build agent and contaminate evidence.
- Install a permanent self-hosted runner or service: rejected because the
  evidence can be collected on an ephemeral host.

## Evidence

Protected workflow run
[`33341872303`](https://github.com/tdloB/impresari-context/actions/runs/33341872303),
job `99338854149`, passed from exact `main` commit
`bf2504f78ddb4e709407a0ac5c23d5d0ecc534a6` on GitHub-hosted Ubuntu 24.04
x86-64. The source-free receipt SHA-256 is
`50ceac6df76bf90f40f6e888bb931ac84e5d18acaa7d8a442834adbcbe2538d4`;
GitHub recorded artifact digest
`sha256:7986c3ebd64e5871e4823898699b1dfa54221ca0d02593700b01bdd222149c8b`.

The receipt records two distinct hashed PAM/logind session identities, two
distinct hashed user-manager invocation identities, first-manager termination,
stable package identity, both original-synthetic rehearsals passing, and every
cleanup condition true. Its result is only `login_session_candidate`:
production admission, real analyzers, privileged installation, and persistent
services remain false.

## Revisit Triggers

Review if the test needs system SSH mutation, lingering, persistent users,
administrator-installed Impresari components, non-loopback networking, or a
non-ephemeral host.
