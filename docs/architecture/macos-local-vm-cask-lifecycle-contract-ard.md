# macOS Local-VM Cask Lifecycle Contract Architecture

- Status: Accepted for source-free implementation; live distribution remains gated
- Date: 2026-09-01
- Governing PRD: [macOS Local-VM Cask Lifecycle Contract PRD](../product/macos-local-vm-cask-lifecycle-prd.md)
- Decision: [ADR-0107](../decisions/0107-freeze-macos-local-vm-cask-lifecycle-before-signing.md)

## Context

ADR-0087 replaced the failed XPC analyzer-execution topology with one fresh
local Linux VM per analyzer job while retaining ADR-0076's selected Option C
user experience: one Homebrew cask with CLI compatibility. ADR-0091 sealed the
active guest metadata graph. The next reversible step is to define what the
cask would own and how its lifecycle must behave before producing distributable
bytes.

## Architecture

```text
ADR-0091 metadata seal
          |
          v
source-free cask package contract
  |       |       |       |
 layout  ownership lifecycle nonclaims
          |
          v
deterministic offline receipt
          |
          +-- no app assembly
          +-- no signing/notarization
          +-- no Homebrew execution
          +-- no VM/analyzer execution
```

The package contract is repository metadata, not a cask or distributable
artifact. It fixes the intended app-relative destinations and ownership
semantics while leaving every release-time digest unresolved until an exact
bundle is assembled under a later decision.

## Closed bundle topology

The future archive contains exactly one `Impresari Context.app`. Its contract
allows these embedded roles:

| App-relative destination | Role |
| --- | --- |
| `Contents/MacOS/impresari-context` | CLI and Rust supervisor entry point |
| `Contents/Helpers/impresari-context-mcp` | Local stdio MCP server |
| `Contents/Helpers/impresari-context-structural-worker` | Existing isolated structural worker |
| `Contents/Helpers/impresari-context-vm-controller` | macOS Virtualization framework controller |
| `Contents/Resources/macos-vm/guest/` | Exact future guest payload root |
| `Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json` | ADR-0091 metadata seal copy |

The Homebrew cask owns the app bundle and one `impresari-context` binary link
to the embedded CLI. The contract does not create links for internal helpers.

## Lifecycle state machine

```text
absent -> installed -> upgraded
   ^         |            |
   |         +-> removed <-+
   |
previous accepted version --explicit rollback--> installed
```

- Install is an all-or-nothing placement of one exact bundle and one CLI link.
- Upgrade replaces the whole bundle; it cannot replace individual helpers or
  guest assets.
- Rollback selects one previously accepted whole bundle through Homebrew's
  operator-controlled lifecycle; it is not automatic.
- Migration rejects a coexisting formula/cask ownership conflict before any
  mutation.
- Uninstall removes only the two cask-owned paths. No `zap` contract exists.

## Security boundaries

- No privileged helper, LaunchDaemon, LaunchAgent, login item, background
  service, postflight, uninstall script, arbitrary shell, or package hook.
- No network, credential, workspace, cache, client-home, or repository-source
  access during contract evaluation.
- No mutable URL, release discovery, checksum lookup, signature claim, or
  analyzer payload is represented by this checkpoint.
- The guest payload root is a closed future destination, not proof that current
  synthetic guest bytes were packaged.

## Deterministic evaluator

The evaluator reads only exact repository metadata. It verifies:

1. profile and package-contract identities;
2. the ADR-0091 seal and metadata-set binding;
3. the closed path/role inventory;
4. one app, one CLI link, and whole-bundle version alignment;
5. exact install, upgrade, rollback, migration, and uninstall semantics;
6. absence of privileged, scripted, automatic-update, client-mutation, and
   analyzer authority; and
7. fail-closed false claims for every later distribution gate.

## Alternatives

- **Assemble an unsigned app immediately:** rejected for this checkpoint because
  the layout and lifecycle would harden without a closed reviewable contract.
- **Sign and notarize immediately:** rejected because it crosses a manual
  credential and publication-preparation boundary before source-free policy is
  fixed.
- **Return to formula-plus-helper packaging:** rejected by the founder-selected
  one-cask Option C topology and its split-version/uninstall burden.

## Sequencing

After this contract passes, the next non-production checkpoint may assemble one
unsigned, synthetic-only app bundle and verify byte layout without installing
it. Live signing, notarization, Homebrew lifecycle, publication, production,
and analyzer activation remain later independent decisions.
