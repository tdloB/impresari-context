# ADR-0045: Owned Native-Guidance Artifact Lifecycle

- Status: Accepted
- Date: 2026-08-25
- Deciders: Impresari Context maintainers
- Related: [CI-2a PRD](../product/client-integration-l2-owned-guidance-lifecycle-prd.md), [ADR-0041](0041-native-agent-guidance-artifacts.md)

## Context

CI-2 provides static artifacts, but a template alone does not give users a
safe, repeatable opt-in lifecycle. Existing project instructions are security-
and collaboration-sensitive: generic text merging would allow accidental
overwrites, ambiguous ownership, and a broad configuration-editing surface.

## Decision

Implement a client-neutral `client guidance` CLI surface over four fixed,
released project artifact paths. It derives a target from a caller-named project
root and client identifier; it does not search for client configuration or
choose a user/global scope. The lifecycle only creates an absent target after
an explicit `--apply`, validates exact template bytes, and removes only an
exactly owned target.

The target parents must already exist. The implementation will not create
`.claude`, `.cursor`, or `.github` directories and will never overwrite or
merge an existing file. Static templates remain canonical release artifacts;
the CLI embeds the same fixed content contract and exposes a digest receipt.

## Consequences

- Install succeeds only for projects with the appropriate pre-existing native
  directory structure (Codex's root `AGENTS.md` needs no subdirectory).
- Existing instructions require a user-managed, separately reviewed approach;
  Impresari will not edit them.
- The narrow lifecycle is auditable and reversible but does not itself prove a
  client uses the artifact or promote any client to L2.

## Alternatives rejected

- Append markers to existing instruction files: cannot preserve unrelated text
  or reliably distinguish ownership.
- Auto-create native client directories: unnecessarily changes project state
  and blurs whether a client surface was intentionally enabled.
- Global/client-home installation: exceeds CI-2's project-scoped consent model.
