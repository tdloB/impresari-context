# CI-3c GitHub Copilot CLI Guided Delivery — Architecture Requirements and Design

- Status: Approved design; implementation and live admission pending
- Date: 2026-08-28
- Governing product record: [CI-3c PRD](../product/ci-3c-copilot-cli-guided-delivery-prd.md)
- Governing decision: [ADR-0060](../decisions/0060-copilot-programmatic-prompt-guided-delivery.md)

## Flow

```text
explicit CI-3a intent -> deterministic packet preview -> operator review
                                                     |
                                                     v
                         --apply + expected packet ID + authenticated home
                                                     |
                                                     v
        rederive canonical bytes -> verify bindings -> disposable empty runtime
                                                     |
                                                     v
 Copilot programmatic prompt, zero tools/MCP/source/remote -> terminal JSON event
                                                     |
                                                     v
                    bounded receipt -> terminate -> delete disposable runtime
```

## Invariants

1. Only `github_copilot_cli` / `programmatic_prompt` / `1.0.80` /
   `prompt_start` reaches planning or client I/O.
2. Apply regenerates canonical packet bytes and verifies the envelope,
   preparation receipt, packet, plan, workspace, and snapshot identities.
3. The child command uses direct argument passing, never a shell. It fixes
   `--disable-builtin-mcps`, `--no-remote`, `--no-remote-export`,
   `--no-auto-update`, `--no-custom-instructions`, `--no-ask-user`,
   `--disallow-temp-dir`, an empty `--available-tools`, bounded credits, JSON
   output, and the exact envelope as `--prompt`.
4. The current directory is a new empty runtime. No source workspace or cache
   path is passed to Copilot. The cleared child environment exposes only the
   minimum process variables and the explicit authenticated `COPILOT_HOME`.
5. Provider network remains available because it is intrinsic to the admitted
   hosted Copilot lifecycle. Tool and URL surfaces are absent; Impresari does
   not grant a network-capable tool.
6. Any tool execution event, malformed event, timeout, or incompatible version
   fails closed. Model text is neither parsed as evidence nor retained.
7. The authenticated home must be real, canonical, non-symlinked, and disjoint
   from the runtime. The runtime is deleted; the caller's home never is.

## Degradation policy

`no_delivery` means the prompt did not enter the exact admitted client surface.
`degraded` means the process started but terminal acceptance was not proven.
There is no retry, interactive fallback, MCP fallback, or source-read fallback.

## Verification

Unit tests use a deterministic fake Copilot process and assert the complete
argument/environment boundary. Live admission uses two independent disposable
runtimes and one dedicated operator-authenticated home, retaining only packet,
plan, snapshot, version, platform, outcome, cleanup, and authority metadata.
