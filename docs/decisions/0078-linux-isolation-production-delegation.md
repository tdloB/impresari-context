# ADR-0078: Select The Linux Production Delegation Topology

- Status: Proposed; founder decision required
- Date: 2026-08-30
- Decider: Aaron Boldt

## Context

ADR-0077 maintains current exact-host Linux IAR-1B candidate evidence, but the
CI setup uses one root-created transient `Delegate=yes` service. Production
support needs a way to obtain that parent boundary. Choosing a rootless user
manager, an administrator-installed policy, or an externally managed subtree
changes installation authority, support coverage, packaging, and maintenance.

## Proposed Decision

Use existing systemd user-manager delegation as the first production-
feasibility candidate. Admit an externally managed delegated subtree only as a
separate explicit profile. Do not add automatic sudo/pkexec fallback, a
privileged daemon, or an administrator-installed unit in the first slice.

Systems lacking an effective user-manager delegation remain `unsupported`.
They must not fall back to IAR-1A while reporting IAR-1B or analyzer readiness.
An administrator-provisioned profile may be reconsidered through a separate
founder-approved security and packaging decision if documented demand warrants
the larger authority surface.

## Rationale

The recommended topology minimizes ambient authority, preserves a simple
install/uninstall boundary, and aligns with kernel and systemd delegation's
single-writer model. It intentionally favors an honest narrow support matrix
over surprise privilege escalation.

## Consequences

- Initial production coverage will be narrower than “all Linux.”
- A source-free user-manager feasibility matrix is required before admission.
- Headless, containerized, or policy-restricted hosts may need Option C or
  remain unsupported.
- No real analyzer is authorized; IAR-2 remains closed until the chosen
  topology passes all admission and release gates.

## Alternatives

- Administrator-provisioned system service/policy: broader coverage but adds a
  privileged install and maintenance surface.
- External delegation only: safest for managed environments but poor general
  first-run experience.
- Rootless container engine: adds a large dependency and cannot substitute for
  exact Impresari confinement evidence.
- Privileged daemon or automatic elevation: rejected as excessive authority.

## Approval Effect

Accepting this ADR authorizes only source-free feasibility contracts and
synthetic testing for Option A and the closed Option C interface. It does not
authorize a real analyzer, repository content as analyzer input, network or
credential access, privileged installation, persistent services, package
publication, or IAR-2 admission.
