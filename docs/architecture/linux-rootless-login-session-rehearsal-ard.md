# Linux Rootless Genuine Login-Session Rehearsal ARD

- Status: Accepted for implementation
- Date: 2026-08-30
- Governing PRD: [Linux Production Lifecycle PRD](../product/linux-isolation-production-lifecycle-prd.md)
- Decision: [ADR-0090](../decisions/0090-linux-rootless-genuine-login-session-rehearsal.md)

## Objective

Supply the missing rootless profile A evidence through two genuine PAM/logind
login sessions for one disposable, non-lingering user. A process restart,
`systemctl --user daemon-reload`, operator capability, or inherited external
delegation cannot substitute for this boundary.

## Test Topology

```text
ephemeral Linux test host controller
      |
      +-- create one temporary non-privileged user
      +-- establish login session 1 through isolated loopback SSH/PAM
      |      package lifecycle + preflight + synthetic rootless corpus
      +-- close session and observe user manager/session termination
      +-- establish login session 2 through the same PAM path
             new session + new user-manager identity + same package identity
             fresh preflight + synthetic rootless corpus + cleanup
      |
      +-- terminate sessions, remove user and isolated SSH state
```

The live job runs only from a protected default-branch or exact manually
selected commit on an ephemeral hosted test machine. Pull requests exercise a
source-free composer with frozen observations and do not receive privilege.

## Bounded Privilege

The host controller may use administrator authority only to create/remove the
temporary user, start/stop one loopback-only isolated SSH daemon using a fixed
configuration, query/terminate that user's logind state, and remove its home.
It may not modify the system SSH service, enable lingering, install a service,
change systemd policy, create an administrator authorization rule, access
credentials, or operate outside the ephemeral test host.

The session user receives no sudo authority. All Impresari package, preflight,
and synthetic confinement work runs as that user.

## Evidence Contract

The closed receipt binds:

- exact host, source, baseline package, and candidate package identities;
- distinct logind session IDs and start times;
- non-lingering state;
- terminated first-session user-manager identity and distinct second-session
  user-manager invocation identity;
- candidate package identity preserved across reentry;
- fresh ready preflight and complete synthetic confinement receipt in each
  session; and
- absence of service units, authorization policy, cgroups, descendants, staged
  source, temporary user, SSH process, and home state after collection.

## Failure Handling

Any missing PAM/logind evidence, retained manager, unchanged invocation
identity, unavailable CPU/memory/pids delegation, non-clean state, or teardown
failure returns `unsupported` or `failed`. It cannot emit a lifecycle candidate
or fall back to the externally managed profile.
