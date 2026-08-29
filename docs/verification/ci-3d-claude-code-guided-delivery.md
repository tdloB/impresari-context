# CI-3d Claude Code guided-delivery verification

- Status: Passed; exact-scope L3 admitted
- Date: 2026-08-29
- Client scope: Claude Code `2.1.241`, macOS aarch64, `safe_mode_print`

## Deterministic evidence

- The workspace compiles with the new `context-claude-code` crate.
- Workspace tests and Clippy pass with the lockfile and offline dependency set.
- Contract validation passes with 28 registered schemas and 28 fixtures.
- Security-boundary validation recognizes exactly four reviewed production
  process-launch sites and confirms tracked-source immutability.
- Adapter tests prove canonical packet preservation, serialized-preview
  rehydration, expected-packet mismatch refusal, runtime/home separation,
  exact prompt-event validation, empty tool/MCP initialization, tool-use
  degradation, and authority-free receipts.
- The installed client reports exactly `2.1.241 (Claude Code)` and exposes the
  required safe-mode, print, empty-tools, slash-disable, no-persistence,
  streaming JSON, and exact user-message replay flags.

## Live admission evidence

The operator explicitly authorized sending bounded synthetic workspace-derived
packets to Anthropic through the existing Claude authentication in place. On
2026-08-29, `scripts/rehearse-claude-guided-delivery.rb` completed two runs:

| Run | Packet | Plan | Snapshot | Outcome |
| --- | --- | --- | --- | --- |
| 1 | `sha256:92797a726073f5c4abfd840fa5df08ecf2b32975acab106bfae82b88804913fd` | `sha256:1498632eddd347feb422f1d39dc3e1fb7125d3d7ca1cc01f9978a9a323a852c1` | `sha256:c90273934c02b90c76575ba849f2f99daa8bc044f30f65867e46acd2bd4f4e09` | `delivered` |
| 2 | `sha256:6694632c7fe84c71b6be3ec9634493a0f8a34acaa21b6d849b0c26c963bf3e97` | `sha256:5310c49ded8a0d8ca737f89981c6cdf685869d766bda38d81c165adb2083f9eb` | `sha256:33a89185e8de6db70effe91ad9442cfc7ac20db6fef39e7d00839de81130734d` | `delivered` |

Both records prove exact prompt acknowledgment, one successful terminal result,
empty tool and MCP initialization, zero tool execution, provider-network
disclosure, no source-workspace exposure, immutable source SHA-256
`0c327c4bcb0f06ab595264a0efc26d1f78ce4802020c20e1e16857810087efc2`,
clean runtime removal, existing authentication inherited only by the Claude
child, and no credential-state copy or deletion.

This evidence admits only Claude Code `2.1.241`, macOS aarch64, and the
`safe_mode_print` lifecycle. It does not admit interactive sessions, hooks,
other platforms, or later versions.

## Upstream surface references

- [Claude Code programmatic mode](https://code.claude.com/docs/en/headless)
- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage)
- [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
