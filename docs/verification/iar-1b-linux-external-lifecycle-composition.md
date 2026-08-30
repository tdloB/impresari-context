# IAR-1B Linux External Lifecycle Composition

- Date: 2026-08-30
- Decision: ADR-0081
- Profile: `externally_managed`
- Production admitted: No
- Real analyzer authorized: No

## Contract Verification

Run:

```sh
ruby scripts/check-linux-external-lifecycle-composition.rb
```

The check reproduces one linked synthetic lifecycle candidate and deterministic
identity, package, external, interruption, and withdrawal failures. It also
holds the single accepted external provisioner launch site and verifies that the
health collector contains no `systemd-run`, `systemctl`, `sudo`, `curl`, or
`wget` authority.

The closed schema admits the source-free health and composition receipts and
rejects a production overclaim. The complete repository gate runs this check.

## Hosted Checkpoint

The release-candidate Linux job now composes the exact candidate package with a
fresh external rehearsal in the same exact-source workflow. The operator may
create only the one accepted temporary delegated service. After collection, the
health collector receives a non-directory sentinel at descriptor 3, proving
that no inherited delegated-directory capability remains without exposing a
raw path. All source records and the composition receipt are retained in the
candidate artifact for bounded review.

## Exact Hosted Evidence

PRs [145](https://github.com/tdloB/impresari-context/pull/145) and
[146](https://github.com/tdloB/impresari-context/pull/146) merged the composition
and the Linux descriptor-portability correction. Release-candidate run
[33300661271](https://github.com/tdloB/impresari-context/actions/runs/33300661271)
then passed on all three packaging targets. Its Linux job `99228064803` ran from
exact source commit `8f8f9adb5d99f373fbd6456564dfa6233c37bc34` on Ubuntu
24.04 x86_64, kernel `6.17.0-1022-azure`.

The retained Linux artifact records:

- candidate archive identity
  `78d1b56552f292ba7da86a0c62ec7a1c840554bd0eec7f1e8a4ec900bece99e3`;
- published v0.1.0 baseline archive identity
  `5b3c71025128e847d8a336f33a6938afaccb105b71a6ec0b30f0fb8c814049b3`;
- C package receipt identity
  `7be2bf0d0742577151b01cde7db1e37ecaa3f893c3fca35b1aba8c884b13489c`;
- fresh external receipt identity
  `e3383329b73ec4132c7aecfe6f381d0ab65ad5d882f67e92871593dfb3a07047`;
- original-synthetic composite identity
  `d9bbcbc55831385b3f56962170622cb2f79dbf8a7237573a2c2d8f712d100c2c`;
- withdrawal receipt identity
  `555d79b84d239a25c72ab6969996b372c3458eb353c53aa7c2a7d05f5522614b`;
- final composition receipt identity
  `0481667521371f3c7db33abfc4b99165fa9b71bd7bc8ed504173f7a89d4ea80b`.

The composition returned `lifecycle_candidate`, linked every receipt to the
same host and exact source, verified exact cgroup kill, timeout, crash/relaunch,
cleanup, capability withdrawal, and clean state, and kept every production,
analyzer, packaging, privileged-installation, and persistent-service authority
claim false. This admits the exact C synthetic lifecycle candidate only. A
remains partial pending genuine login-session reentry, and production and IAR-2
remain closed pending the separate expiring production-support admission.
