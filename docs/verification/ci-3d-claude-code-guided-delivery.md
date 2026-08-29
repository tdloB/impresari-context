# CI-3d Claude Code guided-delivery verification

- Status: Deterministic implementation verified; live admission pending
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
  required safe-mode, print, empty-tools, slash-disable, no-persistence, and
  streaming JSON flags.

## Live admission gate

Run `scripts/rehearse-claude-guided-delivery.rb --user-home <authenticated-home>`
only after the operator explicitly authorizes sending bounded
workspace-derived packet content to Anthropic's Claude service. Admission
requires two successful records, exact packet/plan/snapshot bindings, empty
tool and MCP inventories, zero tool executions, immutable source, runtime
cleanup, and no credential copying or deletion.

Until those records exist, Claude Code remains L1/L2 and CI-3d remains an
implemented but unadmitted L3 path.

## Upstream surface references

- [Claude Code programmatic mode](https://code.claude.com/docs/en/headless)
- [Claude Code CLI reference](https://code.claude.com/docs/en/cli-usage)
- [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- [Claude Code hooks](https://code.claude.com/docs/en/hooks)
