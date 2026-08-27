# ADR-0051: Codex user-home managed connection

- Status: Accepted for implementation
- Date: 2026-08-26
- Scope: Codex L1 connection configuration and admission evidence

## Context

Codex CLI `0.149.0-alpha.4.1` and its App Server loaded a local stdio MCP
entry from the active Codex home configuration, while an observed trusted
repository `.codex/config.toml` entry was not loaded by that runtime. Treating
the repository file as a supported Codex connection scope would therefore
misrepresent the client and leave the published kit unexercised.

The product still needs an auditable managed connection, but it must not
discover or silently modify a person's actual Codex home.

## Decision

Codex L1 connections use an explicit **user-level** TOML target: the active
`$CODEX_HOME/config.toml` or a caller-named equivalent. The public native
setup command is `codex mcp add`; the Impresari kit may inspect, validate,
preview, install, update, or remove only a caller-named TOML file under the
shared exact-ownership contract.

The real-client admission rehearsal uses an explicit empty `CODEX_HOME` under
`/private/tmp`. It must:

- reject malformed client TOML before valid installation;
- apply and validate the exact versioned entry;
- confirm Codex recognizes that entry;
- exercise the direct App Server four-tool lifecycle and packet equivalence;
- remove the exact entry and confirm Codex no longer recognizes it; and
- leave the source workspace unchanged.

The baseline one-use App Server test in ADR-0028 remains configuration-free.

## Constraints

- No default user home discovery, automated real-home write, project-trust
  operation, sign-in, approval grant, environment forwarding, remote MCP, or
  conversational-model tool-selection test is allowed.
- A temporary Codex home can contain client-owned runtime state after the
  App Server runs; the required cleanup is exact Impresari entry removal, not
  deletion of that external client state.
- First-class scope is restricted to the observed Codex CLI version and OS/
  architecture until additional evidence is released.

## Consequences

The public Codex kit matches the configuration surface that the observed
client actually loads and preserves the existing preview/ownership boundary.
The product does not offer project-local Codex MCP configuration. Client
upgrades that change home/configuration behavior require revalidation and may
demote the published classification.

## References

- [CI-1 managed connections PRD](../product/client-integration-l1-managed-connections-prd.md)
- [Codex connection-kit record](../verification/phase-1-codex-connection-kit.md)
- [ADR-0018: client compatibility contract](0018-first-class-client-integration-and-compatibility-contract.md)
- [ADR-0028: deterministic Codex transport](0028-codex-deterministic-mcp-tool-conformance.md)
