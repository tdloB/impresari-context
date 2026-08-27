# CI-3b Codex Guided Delivery — Architecture Requirements and Design

- Status: Implemented; L3 admission pending successful lifecycle evidence
- Date: 2026-08-26
- Governing product record: [CI-3b Codex PRD](../product/ci-3b-codex-guided-delivery-prd.md)
- Governing decision: [ADR-0055](../decisions/0055-codex-ephemeral-guided-delivery.md)

## Flow

```text
explicit intent -> CI-3a snapshot/consent/planner validation -> exact preview
                                                               |
                                                               v
                      operator saves/reviews artifact -> --apply + expected packet ID
                                                               |
                                                               v
     rederive canonical packet bytes -> verify all bindings -> isolated App Server child
                                                               |
                                                               v
       initialize -> ephemeral read-only thread -> read-only/no-network turn -> receipt
                                                               |
                                                               v
                 deny any authority request; terminate child; delete runtime directory
```

The saved preview is operator-controlled evidence, not server-side state. The
apply command never reopens the workspace, planner, or cache; it consumes only
the preview artifact, a caller-owned runtime parent, an absolute Codex binary,
and an expected packet ID. This avoids duplicate audit events and ensures the
packet that crosses the boundary is the reviewed packet.

## Invariants

1. The adapter permits only one exact Codex client/scope/version/lifecycle
   tuple. Other identities yield CI-3a no-delivery before planning.
2. Canonical bytes are regenerated from the serialized `ContextPacket`; packet
   ID, SHA-256, base64url encoding, input text, plan ID, snapshot, workspace,
   and receipt fields must all agree before `Command::spawn`.
3. The packet ceiling is 512 KiB. It limits base64-expanded user input below
   the JSON-RPC reader's 1 MiB line limit and bounds artifact parsing.
4. The child uses no shell, no source root, no cache root, no persistent
   `CODEX_HOME`, no inherited credentials/environment beyond `HOME` and
   `PATH`, and no captured model content.
5. The App Server thread is ephemeral; its sandbox is read-only with tool
   network access disabled. Each incoming authority request is actively
   cancelled or denied, then the session fails closed.
6. Temporary directories are direct children of a verified non-symlink runtime
   parent and are removed on every return path. The adapter never deletes a
   caller-provided directory.

## Degradation policy

`no_delivery` means the packet did not enter a compatible client lifecycle
surface. `degraded` means a process may have received the bounded packet but
the session did not reach the completed lifecycle point; it exposes only a
stable reason and count of denied authority requests. Neither result retries,
falls back to a hook, or changes client configuration.

## Verification

- Unit tests simulate delivery, identity mismatch, authority requests, and
  serialized preview alteration.
- CLI tests prove preview and un-applied delivery do not start a client.
- The live smoke uses an isolated temporary source workspace, cache, preview
  artifact, and runtime parent. It records only packet/plan/snapshot identities
  and receipt metadata; no source content, prompt, or model output is retained.
