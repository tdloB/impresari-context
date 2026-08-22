// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Replaceable, workspace-isolated `SQLite` cache persistence."]

use std::{
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
    time::Duration,
};

use context_core::{AuditEvent, validate_audit_event, validate_utc_timestamp};
use rusqlite::{Connection, OpenFlags, TransactionBehavior, limits::Limit, params};

const CACHE_SCHEMA_VERSION: &str = "1.0.0";
const AUDIT_SCHEMA_VERSION: &str = "1.0.0";

/// Stable cache failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheErrorCode {
    /// Cache-root selection is unsafe.
    InvalidCacheRoot,
    /// Another writer owns the workspace namespace.
    WriterBusy,
    /// Stored metadata has a different workspace or schema identity.
    IncompatibleCache,
    /// `SQLite` or generation integrity failed.
    CorruptCache,
    /// A cache resource ceiling was exceeded.
    ResourceLimit,
    /// A filesystem or database operation failed.
    StorageFailure,
}

/// Safe cache error that omits local paths and SQL text.
#[derive(Debug)]
pub struct CacheError {
    code: CacheErrorCode,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CacheError {
    fn new(code: CacheErrorCode) -> Self {
        Self { code, source: None }
    }

    fn storage(error: impl Error + Send + Sync + 'static) -> Self {
        Self {
            code: CacheErrorCode::StorageFailure,
            source: Some(Box::new(error)),
        }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> CacheErrorCode {
        self.code
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            CacheErrorCode::InvalidCacheRoot => "invalid cache root",
            CacheErrorCode::WriterBusy => "cache writer is busy",
            CacheErrorCode::IncompatibleCache => "incompatible cache",
            CacheErrorCode::CorruptCache => "corrupt cache",
            CacheErrorCode::ResourceLimit => "cache resource limit exceeded",
            CacheErrorCode::StorageFailure => "cache operation failed",
        })
    }
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

/// Derived artifact metadata; source bytes are never persisted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CachedArtifact {
    /// Canonical base64url native path units.
    pub path_units: String,
    /// Diagnostic-only relative rendering.
    pub display_path: String,
    /// SHA-256 of authoritative source bytes.
    pub content_hash: String,
    /// Exact source byte length.
    pub size_bytes: u64,
    /// Bounded normalized lexical terms for contentless candidate retrieval.
    pub terms: String,
}

/// Metadata for one promoted cache generation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheGeneration {
    /// Monotonic database-local identifier.
    pub generation_id: i64,
    /// Source snapshot identity.
    pub snapshot_id: String,
    /// Discovery-policy identity.
    pub discovery_policy: String,
    /// Number of derived artifact rows.
    pub artifact_count: u64,
}

/// Exclusive writer over one exact workspace cache namespace.
#[derive(Debug)]
pub struct WorkspaceCache {
    connection: Connection,
    _writer_lock: File,
    workspace_identity: String,
    namespace: PathBuf,
}

/// Explicit bounded retention applied on every audit append.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditRetention {
    /// Delete records strictly older than this UTC timestamp.
    pub cutoff_utc: String,
    /// Maximum retained records after an append.
    pub max_events: u64,
    /// Maximum physical audit database bytes after an append.
    pub max_bytes: u64,
}

impl AuditRetention {
    /// Creates a validated retention policy.
    ///
    /// # Errors
    ///
    /// Fails for malformed UTC timestamps or limits outside the local profile.
    pub fn new(cutoff_utc: &str, max_events: u64, max_bytes: u64) -> Result<Self, CacheError> {
        if validate_utc_timestamp(cutoff_utc).is_err()
            || max_events == 0
            || max_events > 1_000_000
            || !(1_048_576..=104_857_600).contains(&max_bytes)
        {
            return Err(CacheError::new(CacheErrorCode::ResourceLimit));
        }
        Ok(Self {
            cutoff_utc: cutoff_utc.into(),
            max_events,
            max_bytes,
        })
    }
}

/// Result of one transactional audit append and retention pass.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditAppendResult {
    /// Whether the appended event survived retention.
    pub retained: bool,
    /// Total retained event count.
    pub retained_events: u64,
    /// Physical database size after retention.
    pub database_bytes: u64,
}

/// Exclusive writer for the separate metadata-only audit database.
#[derive(Debug)]
pub struct AuditStore {
    connection: Connection,
    _writer_lock: File,
    database_path: PathBuf,
}

impl AuditStore {
    /// Opens the audit database below a validated cache root.
    ///
    /// # Errors
    ///
    /// Fails closed for unsafe roots, symlinks, active writers, incompatible
    /// metadata, corruption, or storage errors.
    pub fn open(cache_root: &Path) -> Result<Self, CacheError> {
        let root = prepare_cache_root(cache_root)?;
        let audit = root.join("audit");
        if audit.try_exists().map_err(CacheError::storage)? {
            reject_symlink(&audit)?;
        }
        fs::create_dir_all(&audit).map_err(CacheError::storage)?;
        set_private_permissions(&audit)?;
        let audit = audit.canonicalize().map_err(CacheError::storage)?;
        if audit.parent() != Some(root.as_path()) {
            return Err(CacheError::new(CacheErrorCode::InvalidCacheRoot));
        }
        let lock_path = audit.join("lock");
        let writer_lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(CacheError::storage)?;
        set_private_file_permissions(&lock_path)?;
        writer_lock.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => CacheError::new(CacheErrorCode::WriterBusy),
            std::fs::TryLockError::Error(error) => CacheError::storage(error),
        })?;
        let database_path = audit.join("audit.sqlite3");
        let connection = Connection::open_with_flags(
            &database_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(CacheError::storage)?;
        set_private_file_permissions(&database_path)?;
        configure(&connection)?;
        initialize_audit(&connection)?;
        verify_integrity(&connection)?;
        Ok(Self {
            connection,
            _writer_lock: writer_lock,
            database_path,
        })
    }

    /// Transactionally appends a validated metadata event and applies age,
    /// count, and physical-byte retention.
    ///
    /// # Errors
    ///
    /// Fails for an invalid event, duplicate identity, policy limit, or storage
    /// failure. No source/query/path fields exist in the accepted type.
    pub fn append(
        &mut self,
        event: &AuditEvent,
        retention: &AuditRetention,
    ) -> Result<AuditAppendResult, CacheError> {
        validate_audit_event(event)
            .map_err(|_| CacheError::new(CacheErrorCode::IncompatibleCache))?;
        let payload = serde_json::to_vec(event).map_err(CacheError::storage)?;
        if payload.len() > 65_536 {
            return Err(CacheError::new(CacheErrorCode::ResourceLimit));
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(CacheError::storage)?;
        transaction
            .execute(
                "DELETE FROM audit_events WHERE occurred_at < ?1",
                [&retention.cutoff_utc],
            )
            .map_err(CacheError::storage)?;
        transaction
            .execute(
                "INSERT INTO audit_events(event_id,occurred_at,workspace_identity,payload) VALUES(?1,?2,?3,?4)",
                params![event.event_id, event.occurred_at, event.workspace_identity, payload],
            )
            .map_err(CacheError::storage)?;
        let maximum = i64::try_from(retention.max_events)
            .map_err(|_| CacheError::new(CacheErrorCode::ResourceLimit))?;
        transaction
            .execute(
                "DELETE FROM audit_events WHERE sequence IN (
                   SELECT sequence FROM audit_events ORDER BY occurred_at DESC,event_id DESC LIMIT -1 OFFSET ?1
                 )",
                [maximum],
            )
            .map_err(CacheError::storage)?;
        transaction.commit().map_err(CacheError::storage)?;
        self.enforce_audit_bytes(retention.max_bytes)?;
        let retained = self
            .connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM audit_events WHERE event_id=?1)",
                [&event.event_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(CacheError::storage)?;
        Ok(AuditAppendResult {
            retained,
            retained_events: self.event_count()?,
            database_bytes: database_len(&self.database_path)?,
        })
    }

    /// Returns at most `limit` newest events in deterministic order.
    ///
    /// # Errors
    ///
    /// Fails for an invalid limit, corrupt payload, or storage error.
    pub fn recent(&self, limit: u64) -> Result<Vec<AuditEvent>, CacheError> {
        if limit == 0 || limit > 10_000 {
            return Err(CacheError::new(CacheErrorCode::ResourceLimit));
        }
        let limit =
            i64::try_from(limit).map_err(|_| CacheError::new(CacheErrorCode::ResourceLimit))?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT payload FROM audit_events ORDER BY occurred_at DESC,event_id DESC LIMIT ?1",
            )
            .map_err(CacheError::storage)?;
        statement
            .query_map([limit], |row| row.get::<_, Vec<u8>>(0))
            .map_err(CacheError::storage)?
            .map(|row| {
                let bytes = row.map_err(CacheError::storage)?;
                let event: AuditEvent =
                    serde_json::from_slice(&bytes).map_err(CacheError::storage)?;
                validate_audit_event(&event)
                    .map_err(|_| CacheError::new(CacheErrorCode::CorruptCache))?;
                Ok(event)
            })
            .collect()
    }

    /// Deletes records for one exact opaque workspace identity.
    ///
    /// # Errors
    ///
    /// Fails for an invalid identity or storage error.
    pub fn purge_workspace(&mut self, workspace_identity: &str) -> Result<u64, CacheError> {
        validate_identity(workspace_identity)?;
        let removed = self
            .connection
            .execute(
                "DELETE FROM audit_events WHERE workspace_identity=?1",
                [workspace_identity],
            )
            .map_err(CacheError::storage)?;
        Ok(u64::try_from(removed).unwrap_or(u64::MAX))
    }

    fn event_count(&self) -> Result<u64, CacheError> {
        let count = self
            .connection
            .query_row("SELECT COUNT(*) FROM audit_events", [], |row| {
                row.get::<_, i64>(0)
            })
            .map_err(CacheError::storage)?;
        u64::try_from(count).map_err(|_| CacheError::new(CacheErrorCode::CorruptCache))
    }

    fn enforce_audit_bytes(&mut self, maximum: u64) -> Result<(), CacheError> {
        while database_len(&self.database_path)? > maximum {
            let removed = self
                .connection
                .execute(
                    "DELETE FROM audit_events WHERE sequence IN (
                       SELECT sequence FROM audit_events ORDER BY occurred_at,event_id LIMIT 100
                     )",
                    [],
                )
                .map_err(CacheError::storage)?;
            if removed == 0 {
                return Err(CacheError::new(CacheErrorCode::ResourceLimit));
            }
        }
        Ok(())
    }
}

impl WorkspaceCache {
    /// Opens or initializes an isolated cache namespace.
    ///
    /// # Errors
    ///
    /// Fails for unsafe roots, invalid identity, active writers, incompatible
    /// metadata, corruption, or storage errors.
    pub fn open(cache_root: &Path, workspace_identity: &str) -> Result<Self, CacheError> {
        validate_identity(workspace_identity)?;
        let root = prepare_cache_root(cache_root)?;
        let workspaces = root.join("workspaces");
        if workspaces.try_exists().map_err(CacheError::storage)? {
            reject_symlink(&workspaces)?;
        }
        let namespace = workspaces.join(workspace_identity);
        if namespace.try_exists().map_err(CacheError::storage)? {
            reject_symlink(&namespace)?;
        }
        fs::create_dir_all(&namespace).map_err(CacheError::storage)?;
        set_private_permissions(&namespace)?;
        let namespace = namespace.canonicalize().map_err(CacheError::storage)?;
        if !namespace.starts_with(&root) {
            return Err(CacheError::new(CacheErrorCode::InvalidCacheRoot));
        }
        let lock_path = namespace.join("lock");
        let writer_lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(CacheError::storage)?;
        set_private_file_permissions(&lock_path)?;
        writer_lock.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => CacheError::new(CacheErrorCode::WriterBusy),
            std::fs::TryLockError::Error(error) => CacheError::storage(error),
        })?;
        let database_path = namespace.join("index.sqlite3");
        let connection = Connection::open_with_flags(
            &database_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(CacheError::storage)?;
        set_private_file_permissions(&database_path)?;
        configure(&connection)?;
        initialize(&connection, workspace_identity)?;
        verify_integrity(&connection)?;
        Ok(Self {
            connection,
            _writer_lock: writer_lock,
            workspace_identity: workspace_identity.to_owned(),
            namespace,
        })
    }

    /// Returns this namespace's opaque workspace identity.
    #[must_use]
    pub fn workspace_identity(&self) -> &str {
        &self.workspace_identity
    }

    /// Returns the resolved namespace for diagnostics and exact purge planning.
    #[must_use]
    pub fn namespace(&self) -> &Path {
        &self.namespace
    }

    /// Atomically creates and promotes one complete derived generation.
    ///
    /// # Errors
    ///
    /// Fails for invalid inputs, limits, or transactional/integrity errors. The
    /// prior current generation remains current if the transaction fails.
    pub fn promote(
        &mut self,
        snapshot_id: &str,
        discovery_policy: &str,
        artifacts: &[CachedArtifact],
    ) -> Result<CacheGeneration, CacheError> {
        validate_identity(snapshot_id)?;
        validate_identity(discovery_policy)?;
        if artifacts.len() > 1_000_000 {
            return Err(CacheError::new(CacheErrorCode::ResourceLimit));
        }
        for artifact in artifacts {
            validate_artifact(artifact)?;
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(CacheError::storage)?;
        transaction
            .execute(
                "INSERT INTO generations(snapshot_id,discovery_policy,state,artifact_count) VALUES(?1,?2,'staging',?3)",
                params![snapshot_id, discovery_policy, artifacts.len().to_string()],
            )
            .map_err(CacheError::storage)?;
        let generation_id = transaction.last_insert_rowid();
        {
            let mut insert = transaction.prepare(
                "INSERT INTO artifacts(generation_id,path_units,display_path,content_hash,size_bytes) VALUES(?1,?2,?3,?4,?5)",
            ).map_err(CacheError::storage)?;
            for artifact in artifacts {
                insert
                    .execute(params![
                        generation_id,
                        artifact.path_units,
                        artifact.display_path,
                        artifact.content_hash,
                        artifact.size_bytes.to_string()
                    ])
                    .map_err(CacheError::storage)?;
                let search_id = transaction
                    .query_row(
                        "INSERT INTO artifact_search_keys(generation_id,path_units) VALUES(?1,?2) RETURNING search_id",
                        params![generation_id, artifact.path_units],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(CacheError::storage)?;
                transaction
                    .execute(
                        "INSERT INTO artifact_terms(rowid,terms) VALUES(?1,?2)",
                        params![search_id, artifact.terms],
                    )
                    .map_err(CacheError::storage)?;
            }
        }
        let inserted: i64 = transaction
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE generation_id=?1",
                [generation_id],
                |row| row.get(0),
            )
            .map_err(CacheError::storage)?;
        if inserted != i64::try_from(artifacts.len()).unwrap_or(i64::MAX) {
            return Err(CacheError::new(CacheErrorCode::CorruptCache));
        }
        transaction
            .execute(
                "UPDATE generations SET state='retired' WHERE state='current'",
                [],
            )
            .map_err(CacheError::storage)?;
        transaction
            .execute(
                "UPDATE generations SET state='current' WHERE generation_id=?1 AND state='staging'",
                [generation_id],
            )
            .map_err(CacheError::storage)?;
        transaction.commit().map_err(CacheError::storage)?;
        Ok(CacheGeneration {
            generation_id,
            snapshot_id: snapshot_id.to_owned(),
            discovery_policy: discovery_policy.to_owned(),
            artifact_count: u64::try_from(inserted)
                .map_err(|_| CacheError::new(CacheErrorCode::CorruptCache))?,
        })
    }

    /// Reads the last complete promoted generation.
    ///
    /// # Errors
    ///
    /// Fails when stored metadata cannot be queried or parsed.
    pub fn current(&self) -> Result<Option<CacheGeneration>, CacheError> {
        let mut statement = self.connection.prepare(
            "SELECT generation_id,snapshot_id,discovery_policy,artifact_count FROM generations WHERE state='current'",
        ).map_err(CacheError::storage)?;
        let mut rows = statement.query([]).map_err(CacheError::storage)?;
        let Some(row) = rows.next().map_err(CacheError::storage)? else {
            return Ok(None);
        };
        let count: String = row.get(3).map_err(CacheError::storage)?;
        let result = CacheGeneration {
            generation_id: row.get(0).map_err(CacheError::storage)?,
            snapshot_id: row.get(1).map_err(CacheError::storage)?,
            discovery_policy: row.get(2).map_err(CacheError::storage)?,
            artifact_count: count.parse().map_err(CacheError::storage)?,
        };
        if rows.next().map_err(CacheError::storage)?.is_some() {
            return Err(CacheError::new(CacheErrorCode::CorruptCache));
        }
        Ok(Some(result))
    }

    /// Reports bundled `SQLite` version and compile options.
    ///
    /// # Errors
    ///
    /// Fails when diagnostic metadata cannot be queried.
    pub fn sqlite_diagnostics(&self) -> Result<(String, Vec<String>), CacheError> {
        let version = self
            .connection
            .query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .map_err(CacheError::storage)?;
        let mut statement = self
            .connection
            .prepare("PRAGMA compile_options")
            .map_err(CacheError::storage)?;
        let options = statement
            .query_map([], |row| row.get(0))
            .map_err(CacheError::storage)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(CacheError::storage)?;
        Ok((version, options))
    }

    /// Returns bounded path-identity candidates for validated lexical terms.
    ///
    /// # Errors
    ///
    /// Fails for invalid terms, excessive limits, missing current generation,
    /// or database errors. Raw FTS syntax is never accepted.
    pub fn lexical_candidates(
        &self,
        terms: &[String],
        max_candidates: u64,
    ) -> Result<Vec<String>, CacheError> {
        if terms.is_empty()
            || terms.len() > 16
            || max_candidates == 0
            || max_candidates > 10_000
            || terms.iter().any(|term| {
                term.is_empty()
                    || term.len() > 64
                    || !term.bytes().all(|byte| {
                        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                    })
            })
        {
            return Err(CacheError::new(CacheErrorCode::ResourceLimit));
        }
        let query = terms.join(" AND ");
        let limit = i64::try_from(max_candidates)
            .map_err(|_| CacheError::new(CacheErrorCode::ResourceLimit))?;
        let mut statement = self
            .connection
            .prepare(
                "SELECT keys.path_units FROM artifact_terms
                 JOIN artifact_search_keys AS keys ON keys.search_id=artifact_terms.rowid
                 JOIN generations AS generation ON generation.generation_id=keys.generation_id
                 WHERE artifact_terms MATCH ?1 AND generation.state='current'
                 ORDER BY bm25(artifact_terms), keys.path_units LIMIT ?2",
            )
            .map_err(CacheError::storage)?;
        statement
            .query_map(params![query, limit], |row| row.get(0))
            .map_err(CacheError::storage)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(CacheError::storage)
    }
}

/// Purges one exact workspace cache namespace after acquiring its writer lock.
///
/// # Errors
///
/// Fails for unsafe roots/identities, symlinked namespaces, an active writer,
/// or a filesystem error. It never removes the cache root or source workspace.
pub fn purge_workspace(cache_root: &Path, workspace_identity: &str) -> Result<bool, CacheError> {
    validate_identity(workspace_identity)?;
    let root = prepare_cache_root(cache_root)?;
    let namespace = root.join("workspaces").join(workspace_identity);
    if !namespace.try_exists().map_err(CacheError::storage)? {
        return Ok(false);
    }
    reject_symlink(&namespace)?;
    let resolved = namespace.canonicalize().map_err(CacheError::storage)?;
    if resolved.parent().and_then(Path::parent) != Some(root.as_path()) {
        return Err(CacheError::new(CacheErrorCode::InvalidCacheRoot));
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(resolved.join("lock"))
        .map_err(CacheError::storage)?;
    lock.try_lock().map_err(|error| match error {
        std::fs::TryLockError::WouldBlock => CacheError::new(CacheErrorCode::WriterBusy),
        std::fs::TryLockError::Error(error) => CacheError::storage(error),
    })?;
    fs::remove_dir_all(resolved).map_err(CacheError::storage)?;
    Ok(true)
}

fn configure(connection: &Connection) -> Result<(), CacheError> {
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(CacheError::storage)?;
    connection
        .set_limit(Limit::SQLITE_LIMIT_ATTACHED, 0)
        .map_err(CacheError::storage)?;
    connection
        .set_limit(Limit::SQLITE_LIMIT_LENGTH, 16 * 1024 * 1024)
        .map_err(CacheError::storage)?;
    connection
        .set_limit(Limit::SQLITE_LIMIT_SQL_LENGTH, 256 * 1024)
        .map_err(CacheError::storage)?;
    connection
        .execute_batch(
            "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
         PRAGMA trusted_schema=OFF; PRAGMA temp_store=MEMORY;",
        )
        .map_err(CacheError::storage)
}

fn initialize(connection: &Connection, workspace_identity: &str) -> Result<(), CacheError> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS cache_metadata(singleton INTEGER PRIMARY KEY CHECK(singleton=1),schema_version TEXT NOT NULL,workspace_identity TEXT NOT NULL);
         CREATE TABLE IF NOT EXISTS generations(generation_id INTEGER PRIMARY KEY,snapshot_id TEXT NOT NULL UNIQUE,discovery_policy TEXT NOT NULL,state TEXT NOT NULL CHECK(state IN('staging','current','retired')),artifact_count TEXT NOT NULL);
         CREATE UNIQUE INDEX IF NOT EXISTS one_current_generation ON generations(state) WHERE state='current';
         CREATE TABLE IF NOT EXISTS artifacts(generation_id INTEGER NOT NULL REFERENCES generations(generation_id) ON DELETE CASCADE,path_units TEXT NOT NULL,display_path TEXT NOT NULL,content_hash TEXT NOT NULL,size_bytes TEXT NOT NULL,PRIMARY KEY(generation_id,path_units)) WITHOUT ROWID;
         CREATE TABLE IF NOT EXISTS artifact_search_keys(search_id INTEGER PRIMARY KEY,generation_id INTEGER NOT NULL REFERENCES generations(generation_id) ON DELETE CASCADE,path_units TEXT NOT NULL,UNIQUE(generation_id,path_units));
         CREATE VIRTUAL TABLE IF NOT EXISTS artifact_terms USING fts5(terms,content='');",
    ).map_err(CacheError::storage)?;
    connection
        .execute(
            "INSERT OR IGNORE INTO cache_metadata VALUES(1,?1,?2)",
            params![CACHE_SCHEMA_VERSION, workspace_identity],
        )
        .map_err(CacheError::storage)?;
    let stored: (String, String) = connection
        .query_row(
            "SELECT schema_version,workspace_identity FROM cache_metadata WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(CacheError::storage)?;
    if stored.0 != CACHE_SCHEMA_VERSION || stored.1 != workspace_identity {
        return Err(CacheError::new(CacheErrorCode::IncompatibleCache));
    }
    Ok(())
}

fn initialize_audit(connection: &Connection) -> Result<(), CacheError> {
    connection
        .execute_batch(
            "PRAGMA auto_vacuum=FULL;
             CREATE TABLE IF NOT EXISTS audit_metadata(
               singleton INTEGER PRIMARY KEY CHECK(singleton=1),schema_version TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS audit_events(
               sequence INTEGER PRIMARY KEY,
               event_id TEXT NOT NULL UNIQUE,
               occurred_at TEXT NOT NULL,
               workspace_identity TEXT,
               payload BLOB NOT NULL CHECK(length(payload)<=65536)
             );
             CREATE INDEX IF NOT EXISTS audit_by_time ON audit_events(occurred_at,event_id);
             CREATE INDEX IF NOT EXISTS audit_by_workspace ON audit_events(workspace_identity,occurred_at,event_id);",
        )
        .map_err(CacheError::storage)?;
    connection
        .execute(
            "INSERT OR IGNORE INTO audit_metadata VALUES(1,?1)",
            [AUDIT_SCHEMA_VERSION],
        )
        .map_err(CacheError::storage)?;
    let version: String = connection
        .query_row(
            "SELECT schema_version FROM audit_metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .map_err(CacheError::storage)?;
    if version == AUDIT_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(CacheError::new(CacheErrorCode::IncompatibleCache))
    }
}

fn verify_integrity(connection: &Connection) -> Result<(), CacheError> {
    let verdict: String = connection
        .query_row("PRAGMA integrity_check(1)", [], |row| row.get(0))
        .map_err(CacheError::storage)?;
    if verdict == "ok" {
        Ok(())
    } else {
        Err(CacheError::new(CacheErrorCode::CorruptCache))
    }
}

fn prepare_cache_root(root: &Path) -> Result<PathBuf, CacheError> {
    if root.as_os_str().is_empty() || root.parent().is_none() {
        return Err(CacheError::new(CacheErrorCode::InvalidCacheRoot));
    }
    if root.try_exists().map_err(CacheError::storage)? {
        reject_symlink(root)?;
    }
    fs::create_dir_all(root).map_err(CacheError::storage)?;
    set_private_permissions(root)?;
    let resolved = root.canonicalize().map_err(CacheError::storage)?;
    if resolved.parent().is_none()
        || std::env::var_os("HOME").is_some_and(|home| resolved == Path::new(&home))
    {
        return Err(CacheError::new(CacheErrorCode::InvalidCacheRoot));
    }
    Ok(resolved)
}

fn reject_symlink(path: &Path) -> Result<(), CacheError> {
    if fs::symlink_metadata(path)
        .map_err(CacheError::storage)?
        .file_type()
        .is_symlink()
    {
        Err(CacheError::new(CacheErrorCode::InvalidCacheRoot))
    } else {
        Ok(())
    }
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), CacheError> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(CacheError::storage)
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<(), CacheError> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(CacheError::storage)
}

#[cfg(windows)]
fn set_private_permissions(_path: &Path) -> Result<(), CacheError> {
    Ok(())
}

#[cfg(windows)]
fn set_private_file_permissions(_path: &Path) -> Result<(), CacheError> {
    Ok(())
}

fn database_len(path: &Path) -> Result<u64, CacheError> {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .map_err(CacheError::storage)
}

fn validate_identity(value: &str) -> Result<(), CacheError> {
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid {
        Ok(())
    } else {
        Err(CacheError::new(CacheErrorCode::IncompatibleCache))
    }
}

fn validate_artifact(artifact: &CachedArtifact) -> Result<(), CacheError> {
    validate_identity(&artifact.content_hash)?;
    if artifact.path_units.is_empty()
        || artifact.path_units.len() > 87_382
        || artifact.display_path.is_empty()
        || artifact.display_path.len() > 32_768
        || artifact.display_path.contains('\0')
        || artifact.terms.len() > 16 * 1024 * 1024
        || artifact.terms.contains('\0')
    {
        Err(CacheError::new(CacheErrorCode::ResourceLimit))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_core::{AuditOutcome, Capability, ResourceBudget, audit_event};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
    const A: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const C: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    struct TestRoot(PathBuf);
    impl TestRoot {
        fn new() -> Self {
            let n = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path =
                std::env::temp_dir().join(format!("impresari-cache-{}-{n}", std::process::id()));
            fs::create_dir(&path).expect("create test root");
            Self(path)
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn artifact(name: &str, hash: &str) -> CachedArtifact {
        CachedArtifact {
            path_units: name.into(),
            display_path: format!("{name}.rs"),
            content_hash: hash.into(),
            size_bytes: 12,
            terms: format!("{name} rust example"),
        }
    }

    fn event(id: &str, workspace: &str, occurred_at: &str) -> AuditEvent {
        audit_event(
            id,
            "req_12345678",
            occurred_at,
            Some(workspace),
            Some(B),
            Capability::CodeSearch,
            AuditOutcome::Allowed,
            C,
            ResourceBudget::conservative(4096, 10, 10, 128, 100, 32, 30_000, 536_870_912)
                .expect("budget"),
            12,
            "0.0.0",
        )
        .expect("event")
    }

    #[test]
    fn promotion_replaces_current_atomically() {
        let root = TestRoot::new();
        let mut cache = WorkspaceCache::open(&root.0, A).expect("open");
        assert!(cache.current().expect("current").is_none());
        let first = cache.promote(B, C, &[artifact("YQ", A)]).expect("first");
        assert_eq!(cache.current().expect("current"), Some(first));
        let second = cache.promote(C, B, &[artifact("Yg", B)]).expect("second");
        assert_eq!(cache.current().expect("current"), Some(second));
        let duplicate = vec![artifact("same", A), artifact("same", B)];
        assert!(cache.promote(A, B, &duplicate).is_err());
        assert_eq!(
            cache
                .current()
                .expect("current after failure")
                .expect("present")
                .snapshot_id,
            C
        );
    }

    #[test]
    fn tampered_and_incompatible_cache_state_fails_closed() {
        let root = TestRoot::new();
        let cache = WorkspaceCache::open(&root.0, A).expect("open");
        let database = cache.namespace().join("index.sqlite3");
        cache
            .connection
            .execute(
                "UPDATE cache_metadata SET schema_version='forged' WHERE singleton=1",
                [],
            )
            .expect("tamper metadata");
        drop(cache);
        assert_eq!(
            WorkspaceCache::open(&root.0, A)
                .expect_err("incompatible metadata")
                .code(),
            CacheErrorCode::IncompatibleCache
        );

        fs::write(&database, b"not a sqlite database").expect("corrupt database");
        let error = WorkspaceCache::open(&root.0, A).expect_err("corrupt database");
        assert!(matches!(
            error.code(),
            CacheErrorCode::CorruptCache | CacheErrorCode::StorageFailure
        ));
        assert_eq!(error.to_string(), "cache operation failed");
        assert!(
            !error
                .to_string()
                .contains(database.to_string_lossy().as_ref())
        );
    }

    #[cfg(unix)]
    #[test]
    fn cache_and_lock_permissions_are_private() {
        use std::os::unix::fs::PermissionsExt as _;

        let root = TestRoot::new();
        let cache = WorkspaceCache::open(&root.0, A).expect("open");
        assert_eq!(
            fs::metadata(&root.0)
                .expect("root mode")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(cache.namespace())
                .expect("namespace mode")
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(cache.namespace().join("lock"))
                .expect("lock mode")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(cache.namespace().join("index.sqlite3"))
                .expect("database mode")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }

    #[cfg(unix)]
    #[test]
    fn permission_loss_fails_safely_and_recovers_after_operator_restore() {
        use std::os::unix::fs::PermissionsExt as _;

        let root = TestRoot::new();
        let cache = WorkspaceCache::open(&root.0, A).expect("open");
        let database = cache.namespace().join("index.sqlite3");
        drop(cache);
        fs::set_permissions(&database, fs::Permissions::from_mode(0o000)).expect("remove access");
        let denied = WorkspaceCache::open(&root.0, A);
        fs::set_permissions(&database, fs::Permissions::from_mode(0o600)).expect("restore access");
        if let Err(error) = denied {
            assert_eq!(error.code(), CacheErrorCode::StorageFailure);
            assert_eq!(error.to_string(), "cache operation failed");
        }
        WorkspaceCache::open(&root.0, A).expect("recover after restore");
    }

    #[test]
    fn writer_lock_and_namespaces_are_isolated() {
        let root = TestRoot::new();
        let first = WorkspaceCache::open(&root.0, A).expect("first");
        assert_eq!(
            WorkspaceCache::open(&root.0, A).expect_err("busy").code(),
            CacheErrorCode::WriterBusy
        );
        let other = WorkspaceCache::open(&root.0, B).expect("other");
        assert_ne!(first.namespace(), other.namespace());
    }

    #[test]
    fn bundled_sqlite_has_fts5_and_safe_pragmas() {
        let root = TestRoot::new();
        let cache = WorkspaceCache::open(&root.0, A).expect("open");
        let (_, options) = cache.sqlite_diagnostics().expect("diagnostics");
        assert!(options.iter().any(|option| option == "ENABLE_FTS5"));
        let journal: String = cache
            .connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal");
        let synchronous: i64 = cache
            .connection
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .expect("sync");
        let foreign_keys: i64 = cache
            .connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("fk");
        assert_eq!(
            (journal.as_str(), synchronous, foreign_keys),
            ("delete", 2, 1)
        );
    }

    #[test]
    fn lexical_candidates_are_current_bounded_and_compiled() {
        let root = TestRoot::new();
        let mut cache = WorkspaceCache::open(&root.0, A).expect("open");
        cache
            .promote(B, C, &[artifact("alpha", A), artifact("beta", B)])
            .expect("promote");
        assert_eq!(
            cache
                .lexical_candidates(&["alpha".to_owned()], 10)
                .expect("candidates"),
            vec!["alpha"]
        );
        assert_eq!(
            cache
                .lexical_candidates(&["alpha OR beta".to_owned()], 10)
                .expect_err("raw syntax must fail")
                .code(),
            CacheErrorCode::ResourceLimit
        );
    }

    #[test]
    fn purge_is_exact_and_refuses_active_writer() {
        let root = TestRoot::new();
        let cache = WorkspaceCache::open(&root.0, A).expect("open");
        assert_eq!(
            purge_workspace(&root.0, A)
                .expect_err("active writer")
                .code(),
            CacheErrorCode::WriterBusy
        );
        let namespace = cache.namespace().to_owned();
        drop(cache);
        assert!(purge_workspace(&root.0, A).expect("purge"));
        assert!(!namespace.exists());
        assert!(root.0.exists());
        assert!(!purge_workspace(&root.0, A).expect("idempotent"));
    }

    #[test]
    fn audit_is_separate_metadata_only_and_retention_is_deterministic() {
        let root = TestRoot::new();
        let mut audit = AuditStore::open(&root.0).expect("audit");
        let retention =
            AuditRetention::new("2026-08-20T00:00:00Z", 2, 1_048_576).expect("retention");
        audit
            .append(
                &event("evt_00000001", A, "2026-08-19T23:59:59Z"),
                &retention,
            )
            .expect("first");
        audit
            .append(
                &event("evt_00000002", B, "2026-08-20T02:00:00Z"),
                &retention,
            )
            .expect("second");
        let result = audit
            .append(
                &event("evt_00000003", A, "2026-08-20T03:00:00Z"),
                &retention,
            )
            .expect("third");
        assert_eq!(result.retained_events, 2);
        assert!(result.database_bytes <= retention.max_bytes);
        assert_eq!(
            audit
                .recent(10)
                .expect("recent")
                .iter()
                .map(|event| event.event_id.as_str())
                .collect::<Vec<_>>(),
            vec!["evt_00000003", "evt_00000002"]
        );
        assert!(!root.0.join("workspaces").exists());
        let bytes = fs::read(root.0.join("audit/audit.sqlite3")).expect("database");
        assert!(
            !bytes
                .windows(b"sample query".len())
                .any(|part| part == b"sample query")
        );
    }

    #[test]
    fn audit_workspace_purge_is_exact_and_writer_is_exclusive() {
        let root = TestRoot::new();
        let mut audit = AuditStore::open(&root.0).expect("audit");
        assert_eq!(
            AuditStore::open(&root.0).expect_err("exclusive").code(),
            CacheErrorCode::WriterBusy
        );
        let retention =
            AuditRetention::new("2026-08-20T00:00:00Z", 10, 1_048_576).expect("retention");
        audit
            .append(
                &event("evt_00000001", A, "2026-08-20T01:00:00Z"),
                &retention,
            )
            .expect("a");
        audit
            .append(
                &event("evt_00000002", B, "2026-08-20T02:00:00Z"),
                &retention,
            )
            .expect("b");
        assert_eq!(audit.purge_workspace(A).expect("purge a"), 1);
        let remaining = audit.recent(10).expect("remaining");
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].workspace_identity.as_deref(), Some(B));
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_cache_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let root = TestRoot::new();
        let target = root.0.join("target");
        fs::create_dir(&target).expect("target");
        let alias = root.0.join("alias");
        symlink(&target, &alias).expect("symlink");
        assert_eq!(
            WorkspaceCache::open(&alias, A)
                .expect_err("reject symlink")
                .code(),
            CacheErrorCode::InvalidCacheRoot
        );
    }
}
