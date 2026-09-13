# ADR-0157: Tell a Client When to Use Each Tool at Startup

- Status: Accepted
- Date: 2026-09-12
- Related PRD: [Startup Usage Guidance](../product/startup-usage-guidance-prd.md)
- Architecture: [Startup Usage Guidance](../architecture/startup-usage-guidance-ard.md)
- Follows: [ADR-0154](0154-look-through-local-variables-when-building-a-map.md)

## Context

An MCP server's `initialize` response may carry `instructions`, which a client
can keep in its model's context for the whole session. Impresari's said only
that tool results carry no authority. It named none of its nine tools. It did
not say that a map entry can be followed, or an excerpt widened, before a whole
file is read.

An agent that does not know those tools reads files natively. Impresari's
evidence is then added to that reading instead of replacing it, which is the
opposite of what the product is for.

## Decision

1. `instructions` says how a session starts (`context_session_open`, then
   `context_build` with a profile and the task text as `query`) and what each
   other tool is for. It names every advertised tool in 911 characters, under a
   bound of 1,000.
2. It keeps the statement that results add no orchestration, approval,
   execution, or filesystem authority, and adds that the client remains free to
   read files directly. It asks the client to prefer nothing and gates nothing.
3. A test holds the text to the bound, to naming every tool `tools/list`
   advertises, and to both statements.

## Consequences

Offline, this shows only that the guidance is delivered, bounded, and names
what the server offers. Whether an agent then follows the map instead of reading
whole files is what a graded run measures, and that run is still owed.

A client that keeps `instructions` in every prompt pays 911 characters for it.

A tool added later without being named here fails the test, so the guidance
cannot silently fall behind the tool list.

## Alternatives considered

**Rely on per-client guidance templates.** Kept alongside. A template such as
the Copilot instructions file is installed per client. `instructions` reaches
every MCP client with nothing installed.

**Longer guidance with worked examples.** Rejected. A client may repeat it in
every prompt, and the tool schemas already carry examples.

**Tell the agent to use Impresari instead of reading files.** Rejected. That
would steer the agent's execution, which stays with the host and its user.
