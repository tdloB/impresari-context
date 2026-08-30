# DBC-3 foreground loopback dashboard delivery

- Status: Implemented locally; hosted acceptance pending
- Decision: [ADR-0072](../decisions/0072-local-metadata-dashboard-and-narrowing-budget-policy.md)
- Scope: One explicit foreground local process; DBC-4 native browser rehearsal remains pending

## Implemented boundary

- `context-dashboard-server` is an isolated, unsafe-code-free crate that owns
  the only ADR-0072 socket authority. It uses the standard library, binds port
  `0` on an exact IPv4 or IPv6 loopback address, verifies the resulting socket,
  and contains no production outbound connection path.
- `dashboard serve <audit-cache-root> <policy-state-root>` requires two existing,
  exact, disjoint roots. It prints one closed `dashboard-ready` record and runs
  in the foreground; it does not launch a browser, daemonize, advertise, or
  survive explicit shutdown.
- A 256-bit fragment capability is exchanged once for an HttpOnly,
  SameSite=Strict process-local cookie. The token is cleared from browser
  history before the exchange and is never placed in an HTTP request target,
  asset, state response, SSE frame, or error body.
- The bounded HTTP/1.1 parser rejects queries, fragments, duplicate or forwarded
  headers, transfer encoding, pipelining, non-ASCII ambiguity, oversized input,
  wrong Host/Origin, missing authentication, and missing write CSRF/content
  headers. Responses carry no-store, no-frame, no-sniff, restrictive CSP,
  same-origin isolation, and permissions-denial headers.
- Bundled HTML, CSS, and JavaScript have the domain-separated release identity
  `sha256:525f0f3be9eed8cc7ec3abe9d7176cac39d500f4d9df7bef8428fd0897115f70`.
  The UI has no remote resources, inline script/style, source map, storage,
  service worker, telemetry, or source-rendering path.
- Policy apply, removal, and rollback remain preview-first. Every actual browser
  mutation supplies the exact deterministic preview receipt ID and the current
  policy identity/revision; recomputation mismatch or stale state fails closed.
- SSE windows are bounded by clients, frames, bytes, age, polling rate, socket
  timeout, and snapshot size. A missing or out-of-window sequence produces a
  bounded `reset_required` snapshot rather than unbounded replay.

## Evidence

- Closed readiness and source-free HTTP-error schemas accept only loopback
  origins and reject remote origins or diagnostic detail.
- Unit/integration tests cover strict parsing, exact/disjoint roots, wrong Host,
  CSP assets, one-use bootstrap and replay rejection, authenticated state,
  bounded initial SSE delivery, absence of the bootstrap secret, exact policy
  preview/apply, malformed JSON response, explicit shutdown, thread join, and
  closed listener cleanup.
- `scripts/check-security-boundaries.sh` preserves the project-wide network ban,
  permits exactly one production dashboard listener bind site, rejects outbound
  connect/resolution APIs in that crate, and continues to reject network-capable
  runtime dependencies.

## Explicit non-claims

DBC-3 does not establish the final native-browser, adversarial-XSS, screenshot,
or source-canary evidence required by DBC-4. It adds no remote mode, daemon,
automatic browser launch, source view, packet view, telemetry, cost estimation,
provider control, or authority to raise any governing budget.
