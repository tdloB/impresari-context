# Windows Native Feasibility Contract Architecture

- Status: Accepted for synthetic implementation; worker confinement remains gated
- Date: 2026-08-31
- Governing PRD: [Windows native feasibility contract PRD](../product/windows-native-feasibility-contract-prd.md)
- Governing decision: [ADR-0092](../decisions/0092-freeze-windows-native-feasibility-contract.md)

## Boundary

```text
digest-bound target profile
          |
          v
reviewed standalone native probe -- fresh GitHub-hosted Windows 2025 VM
          |
          +-- observe exact build, x86-64, NTFS, required APIs
          +-- configure/query one empty unnamed Job Object
          +-- create/derive/delete one zero-capability AppContainer profile
          v
bounded receipt with every worker/confinement claim false
```

The probe is feasibility evidence, not the future broker. It receives no path,
command, arguments, environment, credentials, source bytes, or network
authority. It creates no child process. Its only persistent-API call creates a
unique per-user AppContainer profile, which must be deleted before success.

## Frozen target profile

The later worker boundary requires:

- one fresh LPAC identity using AppContainer security capabilities with no
  capability SID and the all-application-packages opt-out attribute;
- a suspended, fixed first-party worker with closed inherited handles;
- an unnamed Job Object with kill-on-close, one active process, no breakaway,
  CPU, process-memory, job-memory, and wall-time supervision;
- dynamic-code, extension-point, font, image-load, child-process, and Win32k/UI
  mitigation policy where compatible with the fixed headless worker;
- one exact read-only staged input ACL, zero writable path-backed storage, and
  bounded inherited result pipes;
- termination, zero-descendant verification, handle closure, staging removal,
  profile deletion, and cross-job canary verification in that order.

This increment records the target but verifies only API, empty Job Object, and
profile lifecycle availability.

## Native safety mechanics

- The probe refuses non-Windows, non-x86-64, non-NTFS, non-GitHub-hosted, or
  non-ephemeral workflow contexts.
- The AppContainer name is fixed-prefix plus process and monotonic identities,
  limited to the documented character and length contract.
- Capability count is zero.
- Created and independently derived SIDs must compare equal; both allocations
  are released with `FreeSid`.
- A cleanup guard retries profile deletion on every post-create failure.
- The Job Object never receives a process. Set/query must return the exact
  kill-on-close and active-process flags, with breakaway absent.
- The receipt contains no profile name, SID, user path, repository path, or
  environment value.

## Failure behavior

Any API failure, HRESULT failure, identity mismatch, unsupported filesystem,
unexpected Job Object flag, profile-cleanup uncertainty, malformed receipt, or
contract drift exits unsuccessfully and emits no admissible receipt. The fresh
host is then destroyed by the CI provider; no fallback is attempted.

## Verification

- Cross-platform schema and exact-profile checks run without native execution.
- The Windows-only job compiles the reviewed source with pinned Rust 1.98 and
  warnings denied, runs it once, and validates its JSON receipt.
- A fixture with `os_confined=true` is invalid by construction.
- Full repository, security-boundary, conformance, and hosted quality checks
  remain required.

## Current platform references

- [Microsoft AppContainer profile creation](https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-createappcontainerprofile)
- [Microsoft AppContainer profile deletion](https://learn.microsoft.com/en-us/windows/win32/api/userenv/nf-userenv-deleteappcontainerprofile)
- [Microsoft Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
- [Microsoft AppContainer/LPAC launch attributes](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)
- [GitHub-hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
