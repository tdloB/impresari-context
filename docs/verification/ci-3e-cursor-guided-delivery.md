# CI-3e Cursor guided-delivery verification

- Status: admitted for the recorded scope
- Date: 2026-08-29
- Client scope: Cursor Agent CLI `2026.08.25-3e8eec8`, macOS aarch64,
  ask-mode sandboxed print with stream JSON
- Governing records: [CI-3e PRD](../product/ci-3e-cursor-guided-delivery-prd.md),
  [CI-3e ARD](../architecture/ci-3e-cursor-guided-delivery-ard.md), and
  [ADR-0062](../decisions/0062-cursor-ask-mode-programmatic-guided-delivery.md)

## Deterministic evidence

The adapter verifies exact preview bindings before I/O, runs in an empty
disposable workspace, carries Cursor's silent authentication-status result,
requires exact prompt and cwd events, rejects every tool call, requires one
successful result, retains no model output, and removes its runtime.

Unit tests cover packet bytes, preview alteration, no-I/O mismatch, exact event
acceptance, prompt/cwd drift, tool execution, authority denial, authentication
evidence propagation, and external-path separation.

## Live evidence

Two founder-authorized synthetic rehearsals completed on 2026-08-29. Cursor
used the existing authenticated user home in place. `--trust` applied only to
each newly created empty disposable runtime. Neither run exposed the source
workspace, copied or deleted credential state, inherited a provider API-key
variable, added authority, or executed a tool. Both runs observed one
successful terminal result, preserved the source hash, and left the runtime
parent empty after cleanup.

| Run | Packet | Plan | Snapshot |
| --- | --- | --- | --- |
| 1 | `sha256:2e327dc1ce927c35ef55f58a8b4b7e39526bc6a218c7b2755c6dc40003385897` | `sha256:040fc0528e6e36ae62e8aa327ad5d40999b007b1fbdbb1f6976611924e6ad9d2` | `sha256:9c8592cfe7215a5894be2751972fddd3987c1edbdc6239943e74f3650d7071de` |
| 2 | `sha256:c84733f04cbf4dc575cc44f2f6ba1b9887749978b31fa93efed46a019c0e4fa5` | `sha256:4e5322d433ea8aa73dcda0b50ef5cb37f30f72df1c23d70c5347bac5a814dd7a` | `sha256:afffeec749cda12fb1a1f71e7ccf903c879813feb8f28bc838ebb660d5ad7fd6` |

Both runs recorded source SHA-256
`0c327c4bcb0f06ab595264a0efc26d1f78ce4802020c20e1e16857810087efc2`,
`source_immutable: true`, `runtime_clean: true`,
`authentication_status_verified: true`, `terminal_result_observed: true`, and
`tool_executions_observed: 0` on `universal.arm64e-darwin25`.

This admits only Cursor Agent `2026.08.25-3e8eec8` on the recorded macOS
scope and invocation. It does not extend the claim to interactive sessions,
source workspaces, other versions, or other Cursor surfaces.
