# Windows BaseContainer Capability Routing Architecture

- Status: Accepted for no-worker contract implementation
- Date: 2026-08-31
- Governing PRD: [Windows BaseContainer Capability Routing PRD](../product/windows-basecontainer-capability-routing-prd.md)
- Decision: [ADR-0094](../decisions/0094-prefer-windows-basecontainer-without-automatic-host-preparation.md)

## Boundary

```text
digest-bound source-free profile
             |
             v
read-only Windows 11 arm64 probe
  +-- product type / build / architecture / filesystem
  +-- trusted System32 processmodel.dll presence
  +-- exact experimental export presence
             |
             v
closed capability receipt -> deterministic checker
             |
             +-- ready for later synthetic rehearsal
             +-- unsupported (family/filesystem/build/API)
```

The probe observes capability only. It does not call either sandbox export,
create an AppContainer profile, launch a process, create a Job Object, mutate a
host object, or request authority.

## Routing Contract

The evaluator uses this exact order:

1. a non-workstation Windows product returns `unsupported_host_family`;
2. a non-NTFS workspace volume returns `unsupported_filesystem`;
3. a build below `26600` returns `unsupported_build`;
4. an absent trusted module or either absent export returns
   `unsupported_api_absent`;
5. only the complete conjunction returns `ready_for_basecontainer_rehearsal`.

The version threshold is a minimum observation gate, not a compatibility
claim. Export presence is also not an enforcement claim. Effective confinement
can be established only by a later full synthetic matrix.

## Authority Boundary

- The only loaded code is the Microsoft inbox module resolved with
  `LOAD_LIBRARY_SEARCH_SYSTEM32` for export inspection.
- The receipt excludes machine name, user identity, paths, environment values,
  SIDs, credentials, repository content, and arbitrary OS metadata.
- No administrator helper, UAC prompt, scheduled task, service, package
  activation, drive-root/null-device ACL preparation, firewall rule, or
  optional-feature enablement exists in this slice.
- The build-26100 legacy LPAC implementation remains historical feasibility
  evidence and may continue to report unsupported; it is not an automatic
  fallback from BaseContainer.

## Later Synthetic Composition

A later BaseContainer worker design must be new rather than assumed equivalent
to ADR-0093. The experimental API forbids inherited handles and owns part of the
Job Object/UI boundary, so protocol transport and pre-resume resource
composition require fresh contracts. The later matrix must still prove the
same Tier A filesystem, registry, credential, device, network, process,
resource, fault, cleanup, and cross-job denials.

## Failure Behavior

Unexpected FFI, environment, or filesystem-observation failures exit without
an admissible receipt. Expected platform incompatibility emits a valid
unsupported receipt. Neither path enables a fallback or changes the host.

## Current Platform References

- [Microsoft Create Process in Sandbox APIs](https://learn.microsoft.com/en-us/windows/win32/secauthz/createprocessinsandbox)
- [Microsoft AppContainer/LPAC launch guidance](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)
- [GitHub-hosted runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- [Microsoft MXC host-preparation documentation](https://github.com/microsoft/mxc/blob/main/docs/host-prep.md)
- [Microsoft MXC Windows policy-support matrix](https://github.com/microsoft/mxc/blob/main/docs/process-container/os-version-support.md)

These sources inform feasibility only. Their claims do not substitute for an
Impresari exact-host receipt or admission decision.
