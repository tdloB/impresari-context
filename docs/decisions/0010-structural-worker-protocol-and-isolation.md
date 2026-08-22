# ADR-0010: Structural worker protocol and isolation

- Status: Accepted for Slice B implementation
- Date: 2026-08-22
- Scope: TypeScript/JavaScript structural parsing workers

## Context

ADR-0004 requires native parser and grammar code to run outside the policy
process. The worker must receive source bytes rather than workspace paths, must
not inherit ambient authority, and must return only fully validated structural
facts. A parser crash, malformed frame, timeout, or partial response must not
corrupt the control plane or create graph authority.

## Decision

### Process model

The control process starts one short-lived worker for a bounded batch. The
worker is not a daemon and receives no workspace path, cache path, credential,
or caller policy. The control process reads authorized source through the
existing capability boundary and sends only the minimum source bytes and
lossless relative-path identity needed to label results.

The worker executable identity is verified before use against an explicit
expected SHA-256 digest or the digest recorded by the invoking distribution.
Grammar, parser, resolver, protocol, and graph-contract versions are included
in every successful response and every promoted fact.

### Framing and serialization

- Standard input and output use unsigned 32-bit big-endian length-prefixed JSON
  frames: four length bytes followed by exactly that many UTF-8 JSON bytes.
- Protocol version `1.0.0` permits exactly one request and one response per
  process. EOF, trailing bytes, duplicate JSON fields, unknown fields, invalid
  UTF-8, invalid JSON, or a second frame are failures.
- A request frame is limited to 8 MiB. A response frame is limited to 16 MiB.
- Source is base64url without padding. Raw source hashing remains authoritative.
- Canonical graph identities use the existing RFC 8785 and domain-separated
  SHA-256 contract after the control process validates the complete response.
- Standard error is diagnostic-only, capped at 16 KiB, discarded by default,
  and never included in public errors or audit records.

### Request contract

Every request contains:

- schema name and protocol version;
- opaque request identifier;
- language identifier;
- lossless relative path identity;
- content SHA-256 and base64url source bytes;
- requested fact classes;
- maximum facts, nesting depth, and response bytes;
- parser, grammar, resolver, and graph-contract version expectations.

It contains no absolute path, environment value, network destination, command,
workspace handle, cache handle, or policy decision.

### Response contract

A success response echoes the request and content identities and contains a
complete ordered fact array. Every fact includes its class, stable local key,
source byte span, extraction method, confidence, and resolver provenance.
Unsupported and unresolved relationships are explicit records or warnings;
they are not omitted in a way that implies completeness.

An error response contains only a stable code, retryability, and bounded safe
message. A worker never returns source excerpts, absolute paths, stack traces,
environment values, or operating-system error text.

The control process validates the entire response, rechecks spans against the
authorized content, rejects duplicate keys and limit violations, and promotes
nothing if any field is invalid. Partial worker output is never authoritative.

### Capability reduction

For every Tier A platform, the launcher:

- clears the environment and supplies only a fixed locale marker when the
  platform requires one;
- uses a non-workspace empty current directory;
- closes or prevents inheritance of unrelated handles and file descriptors;
- pipes only standard input, output, and bounded diagnostic error;
- supplies no repository, cache, home, temporary-directory, credential,
  network, model, shell, or command capability;
- kills the whole worker process on cancellation, timeout, protocol failure, or
  output-limit breach.

The worker contains no filesystem or network feature. Application design and
capability omission are the portable baseline; stronger OS confinement is
defense in depth and must be reported per platform rather than implied.

### Resource enforcement

The control process enforces request/input/output/fact/time ceilings on every
platform. Worker-side counters enforce syntax depth, node visits, fact count,
and allocation estimates. Wall time defaults to 5 seconds and may only be
narrowed by policy.

- Linux release packaging adds a documented seccomp/namespace profile when
  available.
- macOS release packaging adds a documented sandbox profile when available.
- Windows release packaging uses a Job Object for process-tree termination and
  applicable memory/CPU ceilings.

Until native enforcement is demonstrated on a platform, reports must say
`application_enforced`; they must not claim a complete OS sandbox.

### Failure and restart behavior

Timeout, crash, signal/abnormal termination, malformed output, version or
identity mismatch, excess output, and partial output discard the complete
batch. One bounded retry is permitted only for a fresh worker and only when the
caller budget still permits it. Repeated failure opens a per-parser circuit for
the session and returns structural state as unavailable while exact lexical
evidence remains usable.

Parser failures never trigger repository execution, grammar download,
self-update, fallback to an unpinned parser, or promotion of heuristic text as
confirmed structure.

## Consequences

The protocol adds process and validation overhead, but it prevents parser code
from sharing the control plane's workspace authority. Short-lived workers and
all-or-nothing batches simplify crash recovery and evidence reasoning. Richer
incremental worker sessions may be proposed later without weakening this v1
contract.

## Verification

- Golden request/response and canonical graph vectors.
- Duplicate/unknown field, malformed length, truncated frame, trailing byte,
  invalid UTF-8/JSON/base64, wrong hash/version, and oversized frame tests.
- Crash, panic, timeout, cancellation, output flood, deep syntax, long token,
  fact-limit, and repeated-failure tests.
- Capability tests demonstrating no workspace path or ambient environment is
  sent to the worker and no worker output bypasses validation.
- Native Tier A confinement and resource evidence recorded separately from
  application-enforced controls.

## Review triggers

Review this decision before long-lived workers, dynamic grammar loading,
repository filesystem access, worker network access, model access, Windows
handle inheritance changes, or claims of stronger OS sandboxing.
