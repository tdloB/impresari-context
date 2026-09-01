# ADR-0087: Use A Fresh Local VM For macOS Analyzer Confinement

- Status: Accepted; release-metadata sealing checkpoint passed
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

## Implementation Checkpoint

On 2026-08-30, two consecutive fresh local Linux VMs passed the hard-capacity
scratch-disk and cross-job canary requirements that the earlier XPC topology
failed. The same run proved exact read-only synthetic input, no configured or
guest-visible non-loopback network device, exact profile/asset binding, VM
stop, and per-job removal on macOS `26.5.1` arm64. The result is recorded in
[IAR-1B macOS local-VM feasibility](../verification/iar-1b-macos-local-vm-feasibility.md).

This is not full IAR-1B admission. The remaining synthetic escape,
descendant/resource, lifecycle, malformed-result, supply-chain, multi-host,
distribution, and independent-review gates remain mandatory.

The next checkpoint passed exact guest pinning and reproducible construction,
malformed-result and bounded-output rejection, whole-VM timeout and synthetic
descendant stop, early-exit handling, controller cancellation, cleanup, and
post-fault recovery. The resulting receipt still lists every unproven gate and
cannot claim confinement, production, or analyzer execution.

The third checkpoint connected the Rust supervisor through its existing single
audited process-launch site. Exact external cancellation and forced-controller
termination both reaped the controller, removed all exact job state, and
completed a new recovery VM job. Guest resource pressure, the complete host-
canary corpus, supply-chain, multi-host, distribution, and independent-review
gates remain mandatory before IAR-1B admission.

The fourth checkpoint used a separately frozen synthetic guest and the same
audited Rust launch site. Its exact guest cgroup v2 leaf contained memory
pressure, throttled CPU pressure, bounded the child count, and was removed.
Six host-only canary classes remained absent from the exact attached devices,
prohibited host paths and the host controller process identity were absent in
the guest, the host corpus remained byte-exact, and all job state was removed.
Host interruption, sealed supply-chain/distribution, multi-host, and
independent-review gates remain mandatory before IAR-1B admission.

The fifth checkpoint installed the macOS will-sleep observer and routed it and
an exact job-private synthetic trigger through one shared fail-closed VM-stop
handler. The automated trigger stopped the VM, reaped the controller, removed
all exact job state, and completed a fresh recovery VM through the same audited
Rust launch site. Its closed receipt requires `real_host_sleep_observed=false`.
Genuine host sleep/wake, reboot, power-loss, sealed supply-chain/distribution,
multi-host, and independent-review gates remain mandatory before admission.

The sixth checkpoint freezes an expiring synthetic guest release manifest,
complete component inventory, SPDX SBOM, license and provenance records,
vulnerability policy, and explicit initial rollback identity. Its offline gate
passed against both repository metadata and every prepared guest component.
Because the upstream publisher has not been authenticated, vulnerabilities
have not been dispositioned, and no Developer ID signature or notarized bundle
exists, its receipt requires `sealed_distribution=false` and
`production_admitted=false`. This implements ADR-0087's existing supply-chain
design without making a new topology decision.

The seventh checkpoint verified the exact Alpine 3.24.1 aarch64 netboot
archive's detached OpenPGP signature under fingerprint
`0482D84022F52DF1C4E7CD43293ACD0907D9495A`, as published on Alpine's official
downloads page. The two embedded guest inputs exactly match the frozen
manifest. This closes upstream publisher authentication only. The archive is
not committed and no runtime network is added; release-metadata sealing,
vulnerability disposition, Developer ID signing/notarization, production, and
analyzer execution remain closed.

The eighth checkpoint performs the bounded vulnerability review against exact
Alpine provider snapshots. It finds the authenticated `6.18.35-0-virt`
candidate thirteen stable patch releases behind `linux-virt` `6.18.48-r0`,
while the published `linux-lts` secdb entry does not establish complete 6.18
advisory coverage. The exact candidate is denied and replacement is required.
This is a fail-closed application of the existing supply-chain decision, not a
new topology decision: review completion does not imply
`vulnerability_assessment_complete`, production admission, or CVE
applicability. See the [vulnerability disposition](../verification/iar-1b-macos-local-vm-vulnerability-disposition.md).

The ninth checkpoint replaces the denied guest with the exact current Alpine
`linux-virt-6.18.48-r0` package. A versioned v2 identity chain preserves all v1
evidence, authenticates the package and index through the APK signing key from
the OpenPGP-authenticated netboot archive, cross-binds the derived guest and
release records, and repeats every native synthetic matrix successfully. The
candidate is current and does not require another replacement at this
checkpoint. Complete advisory coverage, sealed Impresari distribution,
production admission, and analyzer execution remain closed. See the
[current guest replacement](../verification/iar-1b-macos-local-vm-current-guest-replacement.md).

The tenth checkpoint content-addresses all sixteen active v2 metadata and
public-verification members as one exact path/size/SHA-256 inventory. A closed
profile binds the seal, and the deterministic offline receipt cross-binds the
guest manifest, component set, upstream authentication, vulnerability
assessment, and rollback predecessor. This closes repository release-metadata
sealing only. GitHub publication attestation, Developer ID signing,
notarization, cask lifecycle, sealed distribution, production, and analyzer
execution remain false. See the
[release-metadata sealing record](../verification/iar-1b-macos-local-vm-release-metadata-sealing.md).

ADR-0107 adds the eleventh checkpoint: a source-free contract for the retained
one-cask, CLI-compatible distribution direction. It fixes exact component roles,
Homebrew ownership, whole-bundle lifecycle, migration conflict rejection, and
narrow uninstall without creating a cask or app bundle. Every signing,
notarization, live lifecycle, distribution, production, IAR-1B, and analyzer
claim remains false. See the
[cask lifecycle contract record](../verification/iar-1b-macos-local-vm-cask-lifecycle-contract.md).
