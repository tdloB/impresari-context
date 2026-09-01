# Progressive Structural Disclosure — Architecture Requirements and Design

- Status: Implemented and verified by the frozen provider-free MCP mechanics gate.
- Date: 2026-09-01.
- Governing PRD:
  [Progressive Structural Disclosure PRD](../product/progressive-structural-disclosure-prd.md).
- Governing decision:
  [ADR-0121](../decisions/0121-use-bounded-progressive-structural-disclosure.md).

## Architecture outcome

```text
trusted startup: ordinary | eager_structural | progressive_structural
                                  │
             current snapshot + digest-bound structural graph
                                  │
                    identical MCP tool definitions
                                  │
                       identical context_build input
                                  │
        ordinary packet | eager packet | compact disclosure map
                                              │
                              process-local owned handles
                                  ┌───────────┴───────────┐
                                  ▼                       ▼
                         bounded graph lookup      exact evidence expand
                                  │                       │
                                  └───────────┬───────────┘
                                              ▼
                          cumulative ledger + closed receipt
```

Progressive disclosure is a delivery layer over existing canonical evidence.
It neither replaces the structural graph nor creates a second retrieval
engine. The engine owns selection and recovery; MCP owns only strict transport
validation and process-local session routing.

## Trusted mode selection

Delivery mode is fixed at process launch. Tool arguments, repository text, MCP
metadata, and the independent evaluator cannot change it. `ordinary` remains
the default. Both structural modes require the complete worker tuple and one
prepared snapshot-bound graph. Invalid combinations fail before readiness.

Every mode advertises `context_build`, `context_disclosure_lookup`, and
`context_evidence_expand`. In a mode where a capability is unavailable, the
tool returns a closed `unavailable_in_delivery_mode` result without inspecting
source. Tool names, descriptions, and schemas remain equal.

## Progressive build response

Progressive `context_build` requires an open `session_id`. The engine performs
the existing profile plan and structural seed selection, but splits selection
from exact-source packaging. It emits:

- the plan identity and explicit coverage/omissions;
- one compact map identity;
- a deterministic ordered list of bounded disclosure items;
- graph, snapshot, policy, and budget identities;
- one cumulative disclosure receipt.

Each item has a handle plus minimal source-derived labels needed for a consumer
to decide whether to look up or expand it. Labels are exact parser facts or
authorized relative display paths, never generated prose. Item order follows
the existing planner priority and graph order. Truncation is explicit.

The session stores the validated internal graph/evidence descriptors behind
the public handles. It does not persist tasks, source excerpts, maps, handles,
or ledgers to disk. Closing the session or process destroys their authority.

## Handle identities and ownership

Map, item, and evidence handles use separate domain-separated SHA-256
identities. Their canonical preimages include the workspace and snapshot,
graph, plan and policy, descriptor kind, path/content/span or graph node/edge,
selection reason, and contract version as applicable. They exclude session,
consumer, timing, and request/event IDs so deterministic inputs retain equal
identities.

Identity is not authority. Resolution additionally requires the same process,
consumer, open session, configured workspace, current snapshot, prepared graph,
and admitted policy. Handles are opaque inputs: clients cannot provide paths,
spans, nodes, edges, graph JSON, worker details, or source hashes to broaden
them.

## Lookup

`context_disclosure_lookup` accepts:

- `session_id`;
- one owned `map_id` or `item_handle`;
- a closed relation-kind list or `all_admitted`;
- decimal-string item/depth/byte limits that can only narrow the session
  ceilings.

The engine resolves the stored descriptor, calls existing bounded structural
query primitives, filters results through current snapshot and evidence
availability, creates new session-owned handles, and returns compact items. It
does not accept a start-node oracle or arbitrary graph. Repeated identical
lookups return the same result identity while still consuming the declared
cumulative operation and response-byte budgets.

## Exact expansion

`context_evidence_expand` accepts one owned evidence handle and narrowing-only
before/after/max-byte values. The engine reconstructs the stored validated
`EvidenceRecord`, reauthorizes `EvidenceExpand`, rechecks workspace, snapshot,
path, content hash, serialized match bytes, span, and evidence identity, then
uses the existing exact expansion implementation. The response is the existing
exact evidence contract plus a disclosure receipt.

No graph label, map item, or stored descriptor is itself exact source evidence.
Only successful revalidation and recovery may produce an exact excerpt.

## Cumulative ledger

Each progressive session owns one monotonic ledger with configured ceilings
and consumed/remaining values for:

- maps;
- lookups;
- expansions;
- returned items;
- exact source bytes;
- serialized response bytes;
- repository reads;
- repeated repository reads;
- elapsed milliseconds.

The ledger performs checked arithmetic and reserves the entire worst-case
operation before source recovery or serialization. If the reservation would
cross a ceiling, the call returns a source-free exhausted result and consumes
no partial reservation. After success, independently observed actual values
replace the reservation. Accounting corruption or overflow closes progressive
delivery for that session.

Repository reads use the existing workspace observer. Response bytes use the
exact canonical serialized tool-result payload before framing. Time is
diagnostic and non-deterministic; every other receipt identity field is
canonical.

## Receipts

Every operation returns a closed `progressive-disclosure-receipt` containing:

- mode and operation;
- map/item/evidence result identity where applicable;
- workspace, snapshot, graph, policy, and session-reference identities;
- per-call and cumulative consumption;
- remaining ceilings;
- `ready`, `partial`, `exhausted`, `stale`, or `unavailable` state;
- explicit omissions/truncation categories;
- repository read and repeated-read deltas;
- no paths, source text, task text, environment values, cache roots, secrets,
  provider data, or generated summaries.

Measured elapsed time is included in the receipt but excluded from its
deterministic identity.

## Failure and security behavior

- Unknown/duplicate fields, noncanonical decimals, malformed handles,
  unsupported relationships, and expanding limits fail before engine work.
- Foreign consumer/session/workspace/snapshot/graph/content and closed-session
  handles fail without disclosing whether another resource exists.
- Changed source returns stale; it never retargets the handle to current bytes.
- Partial graphs and unresolved edges remain explicit in maps and lookup
  receipts.
- Repository strings are escaped data and cannot select modes, tools, budgets,
  policies, paths, or follow-up operations.
- No model, network, command, environment, source write, durable memory, or
  background authority is added.

## Provider-free fitness gate

Freeze a cross-language corpus with ordinary, eager, and progressive process
manifests. Run each from a fresh source copy and cache, plus a separately named
warm-cache arm. A deterministic script may expand the handles required to
recover the eager evidence set. The gate checks:

1. byte-identical tool definitions and `context_build` inputs;
2. equal source fingerprints and unchanged source trees;
3. initial and cumulative response bytes;
4. graph preparation, lookup, expansion, and total elapsed time separately;
5. reads and repeated reads per operation and cumulatively;
6. exact eager/progressive evidence equivalence after full scripted expansion;
7. explicit omissions, exhaustion, partial graphs, and unsupported languages;
8. deterministic identities across repeated equal runs.

Passing this gate proves mechanics only. It does not predict how many handles a
model will expand or whether tokens, cost, latency, or correctness improve.

## Migration and rollback

The new mode is startup-disabled. Existing calls remain ordinary. Eager
structural mode remains the control and recovery path. Rollback removes the
progressive startup selection and leaves snapshots, graphs, cache entries,
packets, evidence, sessions, and worker protocols intact.

## Revisit triggers

Create a separate decision before adding durable sessions, cross-process
handles, generated summaries, embeddings, request proxies, automatic refresh,
server-initiated behavior, remote MCP, or model-specific disclosure policies.
Revisit descriptor contents and ceilings if the provider-free gate shows
initial byte growth, anchor loss, read amplification, or expansion/eager
evidence mismatch.
