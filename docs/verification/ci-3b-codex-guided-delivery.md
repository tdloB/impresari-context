# CI-3b Codex guided-delivery verification

- Status: local contract checks passed; authentication preflight safely declined delivery; no L3 admission
- Observed: 2026-08-28
- Governing records: [CI-3b PRD](../product/ci-3b-codex-guided-delivery-prd.md), [CI-3b ARD](../architecture/ci-3b-codex-guided-delivery-ard.md), and [ADR-0055](../decisions/0055-codex-ephemeral-guided-delivery.md)

## Local contract evidence

`context-codex-app-server` prepares only the exact CI-3a Codex intent identity: `codex` / `app_server_ephemeral` / `0.150.0-alpha.8` / `turn_start`. Preview performs no client I/O. Its envelope contains the shared planner's exact canonical packet bytes in base64url form plus an untrusted-evidence notice. The serialized preview omits the internal byte buffer; apply derives the canonical bytes again and rejects any altered packet, envelope, receipt, plan, snapshot, workspace, protocol, or client binding before it creates a child process.

The delivery packet ceiling is 524,288 bytes. This is a client-transport boundary, not a change to normal context-planner budgets; it ensures a base64-expanded envelope stays within the JSON-RPC line limit. The receipt always asserts a read-only sandbox, disabled tool network access, and no added authority.

The tests cover exact envelope bytes, altered serialized previews, required `--apply`, preview identity mismatch, source immutability, authority-request cancellation, and degraded authority receipts. The JSON Schema fixture rejects a receipt that claims enabled network access.

## Current isolated live App Server smoke

The locally installed binary reported `codex-cli 0.150.0-alpha.8` on macOS
aarch64. Generated schemas and official App Server documentation confirmed
that clients must send `initialized` after the initialization response and
that delivery succeeds only when the matching terminal turn status is
`completed`. Both rules now have direct protocol tests.

The isolated runtime returned `account: null` with
`requiresOpenaiAuth: true` from `account/read`. Apply returned `no_delivery` /
`codex_auth_unavailable` before thread creation and packet delivery. The normal
Codex home was logged in, confirming that the isolation boundary—not packet
construction—removed the required authentication. No credentials were
inspected or copied. Source bytes remained unchanged and the temporary runtime
was removed.

This is narrower and more accurate than the earlier timeout. L3 now requires
an explicit, reviewed authentication handoff compatible with an ephemeral
runtime, followed by two successful live records.

## Earlier isolated live App Server smoke (historical)

The locally installed binary reported `codex-cli 0.149.0-alpha.4.1` on macOS aarch64. A one-file temporary workspace was snapshotted and previewed with an explicit `implementation` intent and an 8,192-byte hard planner budget. The reviewed packet was:

- packet ID: `sha256:b805a98b880064140be0b4c5910cb9c8eecc2ddcb14583bd9cfc1947a84bfac0`
- plan ID: `sha256:38e0475de861204319a5cb6df509d16ac1d8da2a2701368b94376a326735d8ed`
- snapshot ID: `sha256:25023dd368dae4b0fcea61b7c872871f0c5ff4802b044e6650c1803b92ef04c6`

An apply invocation required the exact preview artifact, the displayed packet ID, `--apply`, a temporary runtime parent, and the absolute app-server binary. The child initialized and started its isolated lifecycle, but no `turn/completed` notification arrived within the 45-second bound. It returned:

```text
outcome: degraded
reason_code: codex_turn_timeout
approval_requests_declined: 0
client_io_performed: true
ephemeral_thread: true
read_only_sandbox: true
network_access_enabled: false
authority_added: false
```

The temporary source file hash remained `d31ce4cc878ea6bfa7fbb6c608c4ffc2b6739a464acd1ad5f8d188d8169ac46b` and the runtime parent was empty after process exit. No source content, prompt, or model output was retained in this record.

This proves bounded, fail-closed degradation and cleanup—not a successful delivery. Codex remains L1/L2; L3 needs a successful, repeatable turn-complete record under the same version/platform/protocol scope.

## Local commands

```text
cargo test -p context-codex-app-server
cargo test -p context-cli --lib codex_delivery_preview_and_apply_preview_never_start_a_client_process
cargo test -p context-conformance --test schema_conformance
ruby scripts/rehearse-codex-guided-delivery.rb --runs 2
```

## Complete quality gate

`./scripts/check.sh` passed locally on 2026-08-28. The gate includes the
repository and security-boundary policies, workspace tests, schema and
fixture conformance, deterministic identity vectors, SBOM validation,
evaluation checks, shell syntax, Clippy with warnings denied, and doctests.
