# CI-3f VS Code Copilot guided-delivery verification

- Status: admitted for VS Code `1.134.0` on macOS arm64
- Date: 2026-08-29
- Candidate scope: VS Code `1.134.0`, macOS arm64, Ask-mode chat CLI
- Governing records: [CI-3f PRD](../product/ci-3f-vscode-copilot-guided-delivery-prd.md),
  [CI-3f ARD](../architecture/ci-3f-vscode-copilot-guided-delivery-ard.md), and
  [ADR-0063](../decisions/0063-vscode-chat-cli-guided-delivery.md)

## Deterministic evidence

The adapter binds exact CI-3a intent, planner packet, prompt, version, and
lifecycle identities before any VS Code I/O. Apply uses a newly created empty
cwd, the exact Ask/new-window/bounded-prompt/stdin-context command, a minimal
cleared environment, bounded discarded launcher output, no source path, and exact
runtime cleanup. Launcher success produces only `confirmation_required`.

Confirmation accepts only the pending receipt and identical expected plus
operator-observed packet IDs. It records that model responses and tool
execution are not machine-observable and that provider delivery was not
inferred. No model output or credential content is retained.

## Live evidence

An initial authorized launch established that stdin without a positional prompt
only stages context and does not submit a turn; it is excluded as no delivery.

Two corrected authorized synthetic launches passed with exact visible packet-ID
acknowledgments:

| Run | Packet | Plan | Workspace snapshot |
| --- | --- | --- | --- |
| 1 | `sha256:89f04cefa0254ee9c4ed444caa814fb01698839d413f7e87a561f2c327781333` | `sha256:00aa59f3a31a9ff2bb4d4ac7d093315bea267c7a0a477d6b404b7f803e8894fb` | `sha256:8cfcfb7ed77a7520a3ff01fd111afa337011bf6c21719a14a059af354e2ed03e` |
| 2 | `sha256:763b658c949e1b10d3dce2188bd819a66b96a47ffd4d5ae9e29b1c8f8725d306` | `sha256:767e61f651141b17244286cc737cfcdac9a74f595b0b7ecbac16b8e31cb88c72` | `sha256:4d95e2281a6fe275cda550f7b637f2fe87e59915abf9be1a54a744887a84e5a9` |

Both runs preserved source digest
`0c327c4bcb0f06ab595264a0efc26d1f78ce4802020c20e1e16857810087efc2`,
removed the disposable runtime, exposed no source workspace, inferred no
provider delivery from launcher exit, inspected/copied/deleted no credential
state, and added no authority. Delivery was finalized only from the exact
operator-observed acknowledgment.
