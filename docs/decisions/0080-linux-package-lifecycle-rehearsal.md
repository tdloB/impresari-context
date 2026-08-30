# ADR-0080: Rehearse A+C Package Replacement Without Faking Login Reentry

- Status: Accepted
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

ADR-0079 froze a complete lifecycle contract for the selected rootless (A) and
externally managed (C) Linux profiles. The current release tooling proves only
a clean extraction and launch. It does not prove replacement, rollback,
profile-specific reentry, or removal, and a fresh process on a hosted runner is
not evidence of a genuine logout/login boundary.

## Decision

Add an independently hosted, package-only rehearsal that uses the published
v0.1.0 Linux archive as the checksum-verified baseline and the exact workflow
source commit as the separately checksummed candidate. In a disposable prefix,
verify the three-binary package scope through clean install, candidate
replacement, baseline rollback, and uninstall.

For profile C, verify operator relaunch by starting the rolled-back CLI as a new
foreground process and requiring its exact machine-readable safe usage failure
(`error-envelope`, `invalid_input`, exit status 1). For profile A,
record `logout_login` as `not_observed` and require a genuine fresh login
session. Never substitute a process restart or user-unit restart for that gate.

The rehearsal itself performs no network access. The release-candidate workflow
downloads the two public v0.1.0 files before invoking it. It uses no credentials,
privilege, service manager, repository source as analyzer input, persistent
installation path, or real analyzer.

## Consequences

- Both A and C obtain exact package install, replacement, rollback, and removal
  evidence on a hosted Linux runner.
- C may obtain a package-lifecycle candidate with operator relaunch.
- A remains package-lifecycle partial until a real logout/login reentry is
  independently observed.
- Cancellation, crash recovery, health withdrawal, and topology revalidation
  remain separate evidence and are not inferred from package operations.
- Full lifecycle, production, real analyzers, privilege, and persistent services
  remain closed.

## Alternatives

- Treat a new process as logout/login: rejected because it does not cross the
  systemd-logind session lifecycle.
- Install a root-owned service to manufacture reentry: rejected because it
  violates the accepted A+C topology.
- Skip rollback because both archives use version 0.1.0: rejected because exact
  archive, manifest, source-commit, and binary identities distinguish the
  baseline from the candidate independently of the semantic version.

## Acceptance Effect

This decision authorizes a source-free package collector, strict receipt schema,
synthetic conformance fixtures, and a release-candidate workflow rehearsal in a
disposable GitHub-hosted Linux job. It does not admit a production platform or
authorize publication, login-session mutation, privileged installation,
service installation, real analyzers, or IAR-2.
