# CI-2 VS Code Copilot native-guidance PRD

- Status: Approved implementation increment; L2 admission pending live evidence
- Date: 2026-08-27
- Authority: Founder-approved client-integration roadmap and autonomous delivery directive
- Parent records: [CI-2 native guidance PRD](client-integration-l2-native-guidance-prd.md), [VS Code Copilot L1 record](../verification/ci-1-vscode-copilot-admission.md), and [Client Integration Depth Roadmap](client-integration-roadmap.md)

## Objective

Make the already-approved, opt-in Copilot project instruction practical for
VS Code Copilot Agent Chat when a user asks for bounded Impresari Context
evidence. The result must help a conversational client choose a valid,
minimal context-packet path without introducing automatic delivery, expanded
authority, or a claim that repeating a prompt repeats the same tool calls.

## Problem evidence

The recorded VS Code L1 smoke discovered and started the local server and
completed bounded session open/close. Its conversational `context_build`
attempt did not satisfy the strict request schema, so no packet was delivered
and Copilot read the local probe file instead. That is not a transport or
permission failure. It shows that the owned guidance and live tool descriptions
need a clearer canonical request path.

The first v3 live attempt then proved that VS Code Copilot `1.134.0` rejects a
tool definition containing top-level `oneOf` before evaluating any proposed
arguments. The supported client-schema subset and the server's strict runtime
request grammar are therefore separate compatibility layers.

## Scope

- Revise the owned GitHub Copilot project instruction from v2 to v3 with an
  explicit, schema-backed four-tool lifecycle.
- Explain the two exclusive `context_build` forms: direct bounded `steps` for
  exact file/term evidence, or one supported `profile` plus `query` for
  planner-backed evidence.
- Require use of the current live input schema for identifiers, operation time,
  full hard budget, and policy fingerprint; the static instruction must not
  copy mutable protocol values.
- Require a clear no-packet result when a packet call fails. Ordinary local
  analysis may continue, but it must not be presented as Impresari packet
  evidence.
- Enrich the live `context_build` tool description with the same canonical
  shape, without changing engine policy, budget accounting, transport, or
  authority.
- Advertise a flat, closed, Copilot-compatible input schema with canonical
  decimal-string budgets while retaining exact exclusive-form checks in the
  server runtime.
- Add a disposable VS Code extension-host rehearsal that installs, validates,
  inspects, and exactly removes both the L1 `.vscode/mcp.json` entry and the
  v3 owned project instruction.
- Revalidate the existing GitHub Copilot CLI L2 artifact at v3 in its own
  isolated CLI scope.

## Non-goals

- No auto-start, auto-trust, auto-approval, prompt interception, hidden prompt
  injection, packet prefetch, background process, provider proxy, source edit,
  shell execution, network service, user-profile change, or Agent Host support.
- No simplified budget preset or server-side implicit budget. Every packet
  remains bounded by caller-provided, schema-validated budget values.
- No L3 packet delivery, packet equivalence claim for conversational VS Code,
  or deterministic tool-selection claim.

## Acceptance criteria

1. The v3 artifact is exact-owned, static, bounded, installable only through
   explicit apply, and v2 remains recognized only for safe removal.
2. The live `context_build` schema describes one canonical direct-evidence
   request form and one canonical profile request form; it never suggests that
   a model may omit a required budget or use both forms together.
   It contains none of the `oneOf`, `anyOf`, `allOf`, or `not` keywords rejected
   by the recorded VS Code Copilot validator.
3. A disposable runner has a source-free preview, rejects non-`/private/tmp`
   roots and symlinks, validates the fixed configuration and guidance artifact,
   and removes only exact owned files after explicit operator confirmations.
4. The recorded VS Code `1.134.0` macOS arm64 L2 smoke shows the v3 guidance
   is active, a packet is built and resolved in the same bounded session, and
   cleanup preserves the source fixture byte-for-byte.
5. The GitHub Copilot CLI v3 guidance smoke and the complete local/hosted
   quality gates pass before public L2 promotion.

## Reassessment checkpoint

After the VS Code L2 run, reassess the Master PRD, CI-2 PRD, client-integration
roadmap, compatibility matrix, and this PRD. If a client still cannot form a
valid request using its supported guidance surface, record that limitation and
do not introduce implicit defaults or automatic delivery to compensate.
