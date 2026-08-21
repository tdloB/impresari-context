# ADR-0006: Local cache and storage design

- Status: Accepted for implementation baseline
- Date: 2026-08-20
- Scope: MVP discovery metadata, lexical index, packet records, local audit, and
  future structural index persistence

## Context

The engine needs transactional derived storage, deterministic schema/version
checks, bounded lexical retrieval, crash recovery, and project isolation. The
cache contains sensitive source-derived data but can never become the source of
truth. The MVP has one local workspace per operation/session, no multi-tenant
service, and no network filesystem requirement.

The storage choice must work in one native binary on all Tier A platforms and
must not require a separate server.

## Decision

Use **SQLite as the embedded derived metadata/index store**, with one isolated
cache namespace per canonical workspace identity.

### Database packaging

- Link a reviewed, pinned SQLite build into the application rather than relying
  on an unknown system SQLite.
- Enable only required features, including FTS5 for bounded lexical candidate
  retrieval.
- Disable runtime loadable extensions.
- Record SQLite build/version and compile options in diagnostic and evaluation
  metadata.
- The implementation baseline must use a currently supported SQLite release
  containing relevant security/correctness fixes. If WAL is ever enabled, it
  must not use versions affected by SQLite's documented WAL-reset bug; version
  floors belong in the dependency policy and lockfile, not only this prose.

### Cache layout

```text
configured-cache-root/
  workspaces/
    <workspace-identity>/
      manifest.json
      index.sqlite3
      lock
      staging/
  audit/
    audit.sqlite3
  exports/                 # only when explicitly configured
```

- Source workspaces never contain engine cache, audit, or temporary files.
- Cache-root selection follows OS conventions or an explicit configuration;
  empty, home, filesystem-root, or unresolved destructive targets are denied.
- Workspace cache directories use restrictive supported permissions and never
  share database files across workspace identities.
- Temporary/staging files remain inside the exact workspace cache namespace.

### SQLite journal and concurrency policy

Use SQLite's default rollback-journal family for the MVP, with a project-selected
durability setting validated by fault tests. Do **not** enable WAL by default.

Rationale:

- the MVP has a single bounded writer and does not need WAL's reader/writer
  concurrency;
- rollback journaling avoids persistent `-wal`/`-shm` lifecycle and network-
  filesystem assumptions;
- SQLite documents WAL tradeoffs and same-host shared-memory requirements;
- cache writes can use staging plus atomic generation promotion.

One process holds the workspace writer lock during index generation/promotion.
Concurrent readers may use the last complete generation. They never observe a
partially built generation as current.

### Schema and generations

- Every database contains schema version, engine compatibility, workspace
  identity, snapshot identity, discovery-policy fingerprint, and creation state.
- Build a new or changed index in a staging generation and commit/promote only
  after integrity and completeness checks.
- A current-generation pointer/manifest is replaced atomically where supported.
- Incompatible, corrupt, cancelled, or partial generations are never promoted.
- Derived-cache schema upgrades rebuild by default in the MVP. In-place
  migrations require evidence that preserving the cache is materially valuable.
- Cache absence or deletion causes rebuild, not loss of authoritative source.

### Lexical index

- Use SQLite FTS5 as a candidate index with a project-selected built-in tokenizer
  and a restricted, project-owned query compiler.
- Prefer a contentless FTS5 design or equivalent that does not make SQLite the
  retrievable source-content store.
- Do not expose raw FTS5 query syntax directly to callers.
- Exact literal/pattern spans are verified by bounded reads of the matching
  current source content; FTS ranking/snippets are never exact evidence.
- Ranking inputs, tokenizer configuration, tie-breaking, and FTS/SQLite versions
  are recorded for determinism and evaluation.
- If FTS5 cannot meet exactness or deterministic ranking gates, replace the
  lexical candidate implementation behind the same public contract.

### Source-derived content policy

- Store the minimum derived data required for retrieval.
- Do not store a second authoritative copy of whole source files.
- Terms, paths, offsets, metadata, or graph facts remain sensitive even when raw
  source is absent.
- Evidence expansion always verifies the authorized source workspace and
  content hash.
- Packet serialization may contain excerpts only when explicitly built/exported
  under policy; packet records are not silently added to durable cache.

### Local audit

- Store metadata-first audit events separately from replaceable workspace
  indexes so a cache rebuild does not erase the local decision trail.
- The MVP audit database is local, bounded, rotated/retained by explicit policy,
  and contains no source/query/secret content by default.
- Audit integrity is transactional but is not represented as cryptographic
  non-repudiation. A hash-chain or signed audit design requires a later decision.

### Purge and recovery

- Purge targets one resolved workspace identity or one explicit audit-retention
  range.
- Broad recursive variables, home/root targets, and unresolved symlinks are
  prohibited.
- Purge never touches the source workspace.
- Crash, disk-full, corruption, and cancellation tests prove that the last valid
  generation remains readable or the cache fails visibly and can rebuild.

## Rationale

SQLite provides an embedded transactional store and FTS5 full-text indexing in
a portable, serverless format. A per-workspace derived database preserves
isolation and simple deletion. Rollback journaling matches the MVP's single-
writer lifecycle and avoids taking on WAL behavior before concurrency requires
it.

## Consequences

### Positive

- One bundled local dependency across Tier A platforms.
- Transactional generation metadata and crash behavior.
- Efficient bounded lexical candidate search without a server.
- Cache namespaces can be inspected, versioned, rebuilt, and purged.
- Later structural tables can coexist without changing public packet schemas.

### Costs

- SQLite/FTS5 is native code in the trusted dependency inventory.
- Token indexes still contain sensitive source-derived information.
- Contentless FTS requires re-reading source for exact spans/snippets.
- Per-workspace databases duplicate schema overhead.
- Project must manage locks, SQLite limits, integrity checks, and version fixes.

## Alternatives Considered

### Content-addressed flat files only

Useful for immutable blobs but rejected as the sole store because transactional
metadata, bounded querying, schema checks, and crash-safe generation promotion
would need substantial custom machinery.

### RocksDB/LMDB or another embedded KV store

Deferred. They can be fast but add native/platform complexity and do not provide
SQLite's relational constraints and built-in FTS5 advantages for the MVP.

### External search service

Rejected because it violates local-first, no-service, no-network, and simple
distribution goals.

### SQLite WAL by default

Rejected for MVP because expected concurrency does not justify the additional
shared-memory/checkpoint/file-lifecycle behavior. It may be reconsidered with
measured multi-process workloads and a current patched SQLite.

### Store complete source in SQLite

Rejected because the source workspace must remain authoritative and minimizing
duplicated sensitive content reduces recovery and privacy risk.

## Verification

- SQLite compile-option and extension-loading tests.
- Cross-workspace isolation and permission tests on every Tier A platform.
- FTS candidate-versus-exact-source verification tests.
- Corrupt DB, wrong workspace, wrong snapshot, wrong version, disk-full,
  cancellation, kill/restart, and atomic-promotion fault tests.
- Cache-size, build-time, and query evaluation across corpus scale.
- Purge dry-run/target validation and source-immutability tests.

## Official References

- [SQLite database file format](https://www.sqlite.org/fileformat.html)
- [SQLite FTS5](https://www.sqlite.org/fts5.html)
- [SQLite write-ahead logging and tradeoffs](https://www.sqlite.org/wal.html)

## Review Triggers

Review if multi-process concurrency needs WAL, FTS5 cannot meet quality or
determinism gates, cache size exceeds accepted limits, encryption-at-rest becomes
a product requirement, a hosted/multi-tenant mode is approved, or structural
graph scale materially exceeds SQLite's evaluated envelope.
