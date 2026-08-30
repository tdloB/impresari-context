# ADR-0087: Use A Fresh Local VM For macOS Analyzer Confinement

- Status: Accepted for synthetic feasibility
- Date: 2026-08-30
- Decider: Aaron Boldt
- Supersedes: ADR-0076's XPC analyzer-execution topology only

## Context

The selected App Sandbox/private-XPC candidate passed many access and resource
denials but failed aggregate temporary-storage enforcement and cross-job state
isolation. Signing, notarization, or ordinary cleanup cannot convert those
runtime failures into OS-enforced per-job confinement.

## Decision

Evaluate one fresh, local, headless Linux VM per macOS analyzer job through
Apple's public Virtualization framework. The VM runs on the user's Mac; no
repository content is uploaded or processed by a hosted service. Synthetic
feasibility precedes packaging and real-analyzer admission.

Preserve ADR-0076's intended one-cask, CLI-compatible user experience. Retain
the XPC prototype as historical and defense-in-depth evidence, not an IAR-1B
execution claim.

## Consequences

- Per-job virtual storage can have a hard capacity and be destroyed wholesale.
- A guest kernel adds defense between a compromised analyzer and macOS.
- Distribution gains a guest-image supply chain, larger artifacts, VM startup
  latency, memory use, patching, and architecture-specific compatibility work.
- Unsupported Macs fail closed; no native process fallback is allowed.

## Alternatives

- Continue XPC unchanged: rejected by the Tier A failures.
- Add a privileged daemon or per-job host user: rejected because it expands
  installation authority and persistent privileged surface.
- Host VMs in the cloud: rejected because it introduces source upload,
  retention, tenancy, billing, and external-service boundaries.
- Use a VM for all Context operations: rejected; only the hostile analyzer
  worker needs this boundary.

## Revisit Triggers

Review before guest networking, shared directories, interactive login,
repository execution, cloud fallback, real analyzers, or broader host support.
