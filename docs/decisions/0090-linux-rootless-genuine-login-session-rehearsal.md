# ADR-0090: Prove Rootless Reentry With A Disposable PAM Login User

- Status: Accepted for implementation
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

## Revisit Triggers

Review if the test needs system SSH mutation, lingering, persistent users,
administrator-installed Impresari components, non-loopback networking, or a
non-ephemeral host.
