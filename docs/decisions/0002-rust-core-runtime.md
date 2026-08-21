# ADR-0002: Rust core runtime

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: Core engine, CLI, local library, and first-party worker processes

## Context

The engine reads hostile repositories, enforces path and resource boundaries,
hashes large file sets, maintains derived indexes, and exposes a stable local
library and CLI. Memory safety, predictable native distribution, explicit error
handling, concurrency control, and low runtime overhead matter more than rapid UI
iteration.

The core must not require Node.js, Python, a JVM, a hosted model, or an external
service. Consumer adapters may use their ecosystem's native language, but the
public capability semantics need one authoritative implementation.

## Decision

Implement the core engine and CLI in **stable Rust using the Rust 2024 edition**.

### Toolchain policy

- Use stable Rust only for released code. Nightly-only language or Cargo
  features are prohibited from the normal build.
- Pin the development and release toolchain in `rust-toolchain.toml` and record
  it in evaluation/release evidence.
- Declare `rust-version` in published Cargo packages.
- At initial release, the minimum supported Rust version (MSRV) will be the
  greater of Rust 1.85, which introduced Rust 2024 edition support, or the
  oldest of the three stable releases tested by project CI.
- Raising MSRV requires release notes, dependency evidence, and a normal minor
  release unless a security fix requires faster movement.

### Workspace shape

Use a Cargo workspace with narrow crates rather than one binary or a premature
microservice split. The expected initial boundaries are:

- `context-core`: protocol-independent types, policies, budgets, evidence, and
  packet contracts;
- `context-workspace`: canonical workspace authorization, discovery, snapshots,
  and exact reads;
- `context-store`: replaceable cache/index persistence;
- `context-retrieval`: bounded deterministic MVP search and packaging;
- `context-cli`: human and structured local interface;
- `context-conformance`: fixtures and contract test helpers.

Names are internal placeholders until package naming passes the public-name
gate. Crate boundaries may be consolidated if they create cyclic or superficial
abstractions, but capability and trust boundaries must remain visible.

### Runtime policy

- Prefer synchronous, bounded operations for the MVP. Do not adopt an async
  runtime without measured concurrency or transport requirements.
- Use bounded worker pools only where evaluation demonstrates benefit.
- Cancellation must occur at explicit safe points and cannot commit a partial
  authoritative cache.
- No daemon, background service, automatic updater, or resident telemetry
  process is part of the MVP.

### Unsafe and native-code policy

- First-party Rust crates use `#![forbid(unsafe_code)]` by default.
- A first-party crate that requires unsafe code needs a dedicated ADR, a minimal
  encapsulated surface, documented invariants, targeted tests, and security
  review.
- Native dependencies such as SQLite or the later Tree-sitter runtime are
  permitted only through reviewed, pinned bindings. Their existence does not
  weaken the safe-code rule for first-party crates.
- Dynamic library/plugin loading is prohibited in the MVP.

### API policy

- Rust library APIs are the first in-process reference API, not the permanent
  cross-language ABI.
- Public serialized contracts remain language-neutral and versioned.
- No stable C ABI is promised in early releases.
- Errors use typed codes and safe details; panics must not be part of normal
  input or policy handling.

## Rationale

Rust provides memory safety without a garbage-collected runtime, strong typed
contracts, mature cross-platform native tooling, and a good fit for bounded
filesystem/index work. The Rust project publishes and tests Tier 1 host tools,
Cargo supports an explicit `rust-version`, and Rust 2024 is the current edition.

This choice does not claim that Rust removes logic, dependency, FFI, filesystem,
or supply-chain vulnerabilities. The threat model and evaluation gates remain
mandatory.

## Consequences

### Positive

- Strong memory and type safety in the core authorization/evidence path.
- Single native binary and library distribution without a language runtime.
- Explicit resource ownership and predictable local operation.
- Good support for fuzzing, property tests, structured errors, and portable CLI
  tooling.
- The later Rust Tree-sitter binding is officially documented by Tree-sitter.

### Costs

- Higher contributor learning curve than TypeScript or Python.
- Native dependencies require per-target build and security attention.
- Cross-language consumers need serialized or transport adapters.
- Compile times and dependency feature graphs require active management.

## Alternatives Considered

### TypeScript/Node.js

Rejected for the authoritative core because runtime installation, memory/resource
control, module supply chain, and native filesystem/parser boundaries would be
less self-contained. TypeScript remains appropriate for the AI App Builder OS
adapter.

### Go

Credible and simpler to distribute, but rejected because Rust provides a
stronger fit for memory-sensitive indexing, explicit ownership, safe low-level
filesystem work, and later embedded/native parser boundaries.

### Python

Rejected for the core because of runtime distribution, performance variability,
native-extension dependence, and weaker enforcement of the central typed
contracts. Python may receive a client SDK later.

### Mixed-language core from the start

Rejected because it creates multiple build systems and duplicated semantics
before the core contract is stable.

## Verification

- CI builds with the pinned stable toolchain and the declared MSRV.
- `cargo fmt --check`, `cargo clippy` with warnings denied for project code, unit
  tests, documentation tests, conformance tests, and security tests are required.
- Dependency inventory, advisories, license checks, and unused-feature review
  are release evidence.
- A repository check fails if first-party unsafe code appears without an
  approved exception.

## Official References

- [Rust platform support](https://doc.rust-lang.org/rustc/platform-support.html)
- [Cargo `rust-version`](https://doc.rust-lang.org/cargo/reference/rust-version.html)
- [Rust editions](https://doc.rust-lang.org/edition-guide/editions/)
- [Rust 2024 edition RFC](https://rust-lang.github.io/rfcs/3501-edition-2024.html)

## Review Triggers

Review this decision if Rust prevents two unrelated consumers from using the
core, the required native interfaces force broad unsafe code, supported targets
cannot be distributed reliably, or measured implementation cost materially
outweighs the security/performance benefits.
