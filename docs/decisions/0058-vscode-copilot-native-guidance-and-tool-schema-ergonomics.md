# ADR-0058: VS Code Copilot native guidance and tool-schema ergonomics

- Status: Accepted; L2 recorded for VS Code `1.134.0` on macOS arm64
- Date: 2026-08-27
- Scope: VS Code Copilot extension-host native guidance

## Context

VS Code Copilot L1 admission established a safe extension-host MCP connection.
During the live smoke, Copilot opened and closed a bounded session but formed
an invalid `context_build` request and then read the local fixture directly.
The local read was not an Impresari packet and must not be treated as one.

A second live attempt with v3 guidance exposed a separate client-validator
constraint before any build arguments reached the MCP server. VS Code Copilot
`1.134.0` rejected the tool definition with `object has unsupported top-level
schema keyword 'oneOf'`. Supplying an exact request in the prompt could not
bypass that definition-level rejection.

The existing owned Copilot v2 instruction correctly avoided copying mutable
protocol values, but it did not make the two legal packet forms sufficiently
explicit for a conversational client. The transport already exposes the
required live schema and no engine-policy gap was identified.

## Decision

Publish a v3 exact-owned Copilot instruction that describes one canonical
direct-evidence path (`steps`) and one canonical planner path (`profile` plus
`query`), their lifecycle order, and the no-packet failure boundary. Enhance
the live `context_build` description and field descriptions so clients can
obtain the dynamic required values from the MCP schema itself.

Maintain v2 as a removal-only legacy artifact. Do not broaden `context_build`,
default budgets, or make any configuration/approval decision for a client.
Publish the input schema as a flat closed object using the Copilot-supported
schema subset and canonical decimal-string budget fields. Keep mutual
exclusivity authoritative in the existing server-side deserialization and
dispatch match, which rejects mixed, incomplete, or unknown request shapes
before engine work. Previously accepted non-negative integer budget values
remain normalized by the server but are not the canonical advertised form.

## Consequences

- VS Code L2 earned a distinct recorded-scope admission through its own
  disposable guidance-and-packet smoke; L1 evidence alone did not imply it.
- GitHub Copilot CLI's prior v2 L2 record was revalidated with v3 in its own
  isolated client scope before the current-template claim was promoted.
- Clients receive clearer portable guidance without a provider-specific proxy
  or automatic retrieval.
- Conversational selection remains non-deterministic and requires revalidation
  when the client, custom-instruction behavior, MCP schema, or platform changes.

## References

- [CI-2 VS Code Copilot native-guidance PRD](../product/ci-2-vscode-copilot-native-guidance-prd.md)
- [CI-2 native-guidance ARD](../architecture/ci-2-vscode-copilot-native-guidance-ard.md)
- [VS Code custom instructions](https://code.visualstudio.com/docs/agent-customization/custom-instructions)
- [ADR-0041](0041-native-agent-guidance-artifacts.md)
- [ADR-0045](0045-owned-native-guidance-artifact-lifecycle.md)
