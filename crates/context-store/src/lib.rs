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

use rusqlite::{Connection, OpenFlags, TransactionBehavior, limits::Limit, params};

const CACHE_SCHEMA_VERSION: &str = "1.0.0";

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
        let writer_lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(namespace.join("lock"))
            .map_err(CacheError::storage)?;
        writer_lock.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => CacheError::new(CacheErrorCode::WriterBusy),
            std::fs::TryLockError::Error(error) => CacheError::storage(error),
        })?;
        let connection = Connection::open_with_flags(
            namespace.join("index.sqlite3"),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(CacheError::storage)?;
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

#[cfg(windows)]
fn set_private_permissions(_path: &Path) -> Result<(), CacheError> {
    Ok(())
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
    {
        Err(CacheError::new(CacheErrorCode::ResourceLimit))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        }
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
