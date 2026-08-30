# IAR-1B Linux Rootless User-Manager Rehearsal

- Date: 2026-08-30
- Decision: ADR-0078
- Scope: one transient user unit and the complete original-synthetic composite
- Privilege use: No
- Persistent service: No
- Production admitted: No
- Real analyzer authorized: No

## Prior Preflight Evidence

PR 137 run `33293552482` produced the first bounded rootless host matrix:

- Ubuntu 24.04 x86_64, kernel `6.17.0-1022-azure`: ready;
- Ubuntu 24.04 arm64, kernel `6.17.0-1022-azure`: ready;
- Ubuntu 26.04 x86_64, kernel `7.0.0-1012-azure`: ready;
- Ubuntu 22.04 x86_64, kernel `6.8.0-1064-azure`: insufficient
  delegation because CPU was absent while memory and pids were available.

The Ubuntu 22.04 result is an expected fail-closed support result. It does not
trigger sudo, a system unit, a privileged daemon, or an application-only
fallback.

## Rehearsal Boundary

Only a preflight-ready ephemeral GitHub-hosted runner may enter the rehearsal.
The launcher talks to the already-running per-user service manager and creates
one foreground transient service with `Delegate=cpu memory pids`, `Type=exec`,
wait, pipe, and collect semantics. The command, working directory, environment,
controller list, and synthetic composite script are fixed. The environment PATH
is reduced to `/usr/bin:/bin`.

The transient service executes the existing frozen Linux composite below its
own delegated subtree. That corpus combines atomic placement, Landlock,
seccomp, descriptor closure, zero path-backed writes, CPU/memory/process limits,
bounded output, exact cgroup kill, timeout, crash/relaunch, empty-state cleanup,
and cross-job isolation. No repository-derived input or real analyzer is used.

After completion, the launcher asks only the user manager for the exact unit's
load state. The unit must be `not-found`, proving collection. The receipt does
not record the unit name or a raw cgroup path.

## Closed Result Contract

`linux-rootless-user-manager-rehearsal.schema.json` admits an exact-host
candidate only when preflight was ready, the complete composite passed, and the
transient unit was collected. Deterministic negative states cover preflight
skip, launch failure, composite failure, and cleanup failure. Every state keeps
production, real analyzers, privileged installation, network, credentials,
sudo, system services, and persistent services closed.

Run the source-free contract checks with:

```sh
ruby scripts/check-linux-rootless-user-manager-rehearsal.rb
```

## Next Gate

PR 138 run `33294099301` reproduced the candidate on every preflight-ready
target without sudo, a privileged service, or a persistent unit:

- Ubuntu 24.04 x86_64, kernel `6.17.0-1022-azure`, job `99210693873`:
  `candidate_passed`; transient unit created and collected; composite receipt
  `d9bbcbc55831385b3f56962170622cb2f79dbf8a7237573a2c2d8f712d100c2c`;
- Ubuntu 24.04 arm64, kernel `6.17.0-1022-azure`, job `99210693845`:
  `candidate_passed`; transient unit created and collected; composite receipt
  `7633048a5dfb5f0172efe4dbdf925ba51aa5adfa9b435a273091e3fb3c27731d`;
- Ubuntu 26.04 x86_64, kernel `7.0.0-1012-azure`, job `99210693871`:
  `candidate_passed`; transient unit created and collected; composite receipt
  `0daae8d668a69b7cc4637796a7fab497e44776e86586ae522ba17d6b1da85ffb`;
- Ubuntu 22.04 x86_64, kernel `6.8.0-1064-azure`, job `99210693750`:
  `skipped_preflight`; no unit was attempted or created because CPU delegation
  was unavailable.

These are exact-host rootless synthetic candidates only. Production support
still needs a frozen rootless maintenance/release matrix, clean install,
upgrade, logout/login, rollback, and uninstall lifecycle evidence, plus
independent admission of the selected externally managed profile. IAR-2 and
real analyzers remain closed.
