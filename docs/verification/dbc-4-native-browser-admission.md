# DBC-4 native-browser dashboard admission

- Date: 2026-08-30
- Platform: macOS 26.5.1 arm64
- Browser surface: Codex In-app Browser (`iab`), production desktop build
- Scope: Synthetic audit metadata, hostile rejected row, disposable policy state
- Result: Admitted for the foreground loopback dashboard boundary

## Bound artifact identities

- Bundled dashboard assets:
  `sha256:e3b34610ddf1b3290b09fe14c49bc5632f6f6ef0ddbd7e2a04827e8143a6f249`
- Private synthetic-canary manifest:
  `sha256:6e33ed8df4a51a5e84671f0d69bd891ead2a7418bfe5360ef9f217b724a3634f`
- Final post-shutdown viewport screenshot:
  `sha256:6266db8f52a41269a5aa0b92b44a3792be8652d593923008d7333f400109d366`

The screenshot was inspected in the live browser and is hash-recorded rather
than retained in the repository. It contained no `DBC4_` source-canary marker.

## Rehearsal fixture

`crates/context-dashboard-server/examples/dbc4_fixture.rs` created one valid
canonical audit event plus one deliberately incompatible future-schema row.
The rejected row contained distinct synthetic path, filename, query, content,
prompt, credential, and environment canaries, two HTML/script payloads, an
unknown field, and an oversized inert string. The fixture self-verified exactly
one readable event, one unavailable row, and one valid dashboard projection
before the production CLI started.

`scripts/rehearse-dashboard-native-browser.rb` then launched the real
`impresari-context dashboard serve` foreground command, passed one additional
synthetic environment canary, waited for browser shutdown, scanned process
output and policy state for every exact canary, and removed the entire
disposable root.

## Native-browser observations

- The fragment capability was cleared from the top-level URL before exchange.
- The one-use exchange returned a separate random 256-bit API-route capability
  retained only in bundled-page memory. No cookie, browser storage, query
  parameter, stable API route, or readiness field carried that authority.
- The browser connected, rendered exactly one metadata record, and reported
  exactly one unavailable hostile row without rendering its raw bytes.
- No source-canary marker appeared in visible text, policy state, process
  output, policy files, or the inspected screenshot.
- The protocol regression test also inserted a malformed row with a fixed
  source canary and proved the raw state response counted it unavailable without
  containing the canary bytes.
- Neither the hostile audit-row script nor the hostile policy-draft script
  executed. The latter returned only the closed `policy_unavailable` category.
- The observed asset/request inventory contained only `127.0.0.1` URLs: the
  bundled CSS and JavaScript, bootstrap, state, apply, and remove surfaces. No
  font, image, video, CDN, analytics, telemetry, or external request appeared.
- Apply preview performed no write. Returning its exact receipt applied
  revision `2`; remove preview performed no write; returning that exact receipt
  removed the current policy and retained the declared previous identity.
- UI shutdown ended the foreground session with `Local session ended`, no
  browser warning/error log, successful process exit, and complete disposable
  fixture removal.

The final host receipt reported:

```json
{
  "status": "passed",
  "schema_name": "dbc4-native-browser-host-receipt",
  "schema_version": "1.0.0",
  "asset_sha256": "sha256:e3b34610ddf1b3290b09fe14c49bc5632f6f6ef0ddbd7e2a04827e8143a6f249",
  "private_manifest_sha256": "sha256:6e33ed8df4a51a5e84671f0d69bd891ead2a7418bfe5360ef9f217b724a3634f",
  "source_canaries_absent_from_process_output": true,
  "source_canaries_absent_from_policy_state": true,
  "dashboard_exit_success": true,
  "external_network_required": false,
  "source_workspace_used": false,
  "disposable_fixture_removed": true
}
```

## Defects found and closed by DBC-4

The native run caught four issues that protocol-only tests had not established:

1. Same-origin browser GET/EventSource requests may omit `Origin`; exact
   mismatched origins remain rejected, while state-changing requests still
   require the exact origin.
2. The first synthetic event used a noncanonical resource-policy label and was
   correctly withheld; the fixture now constructs and self-verifies a canonical
   `ResourceBudget`.
3. Cookies are not scoped by loopback port and showed collision/interoperability
   risk across local services and concurrent sessions. Dashboard authority is
   now a separate memory-only random API-route capability, with cookie support
   removed from the server.
4. The initial client transition double-prefixed legacy `/api` suffixes; the
   final observed inventory proves exact capability-relative routes.

## Claim boundary

DBC-4 admits the optional foreground local dashboard for browser-rehearsed,
metadata-only observability and narrowing-only budget control. It does not add
remote reachability, source/packet viewing, background execution, telemetry,
provider-cost authority, automatic browser launch, or a compatibility claim
with another product. Browser extensions, browser-process compromise, and
developer-tool access remain outside the server's control.
