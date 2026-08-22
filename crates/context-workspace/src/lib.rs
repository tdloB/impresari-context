// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Workspace authorization, discovery, snapshots, and exact reads."]

use std::{
    collections::BTreeMap,
    error::Error,
    ffi::{OsStr, OsString},
    fmt,
    io::{self, Read},
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use cap_std::{ambient_authority, fs::Dir};
use sha2::{Digest, Sha256};

const IDENTITY_NAMESPACE: &str = "impresari-context";
const CONTRACT_VERSION: &str = "1.0.0";

/// Stable machine-readable workspace failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceErrorCode {
    /// The requested root or artifact does not exist.
    PathNotFound,
    /// The requested root is not a directory.
    RootNotDirectory,
    /// A path identity is malformed or incompatible with this platform.
    InvalidPathIdentity,
    /// A path is absolute, traversing, or otherwise outside the relative contract.
    PathOutsideRoot,
    /// The path resolves through a symbolic link.
    SymlinkRejected,
    /// The target is not a regular file.
    UnsupportedObject,
    /// The configured file-size ceiling was exceeded.
    ResourceLimit,
    /// The filesystem changed during a guarded read.
    ChangedDuringRead,
    /// A local filesystem operation failed without a safe public detail.
    IoFailure,
}

/// A safe workspace error that never embeds an ambient absolute path.
#[derive(Debug)]
pub struct WorkspaceError {
    code: WorkspaceErrorCode,
    source: Option<io::Error>,
}

impl WorkspaceError {
    fn new(code: WorkspaceErrorCode) -> Self {
        Self { code, source: None }
    }

    fn io(code: WorkspaceErrorCode, source: io::Error) -> Self {
        Self {
            code,
            source: Some(source),
        }
    }

    /// Returns the stable error category.
    #[must_use]
    pub const fn code(&self) -> WorkspaceErrorCode {
        self.code
    }
}

impl fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            WorkspaceErrorCode::PathNotFound => "path not found",
            WorkspaceErrorCode::RootNotDirectory => "workspace root is not a directory",
            WorkspaceErrorCode::InvalidPathIdentity => "invalid path identity",
            WorkspaceErrorCode::PathOutsideRoot => "path is outside the authorized root",
            WorkspaceErrorCode::SymlinkRejected => "symbolic links are not permitted",
            WorkspaceErrorCode::UnsupportedObject => "unsupported filesystem object",
            WorkspaceErrorCode::ResourceLimit => "resource limit exceeded",
            WorkspaceErrorCode::ChangedDuringRead => "file changed during read",
            WorkspaceErrorCode::IoFailure => "filesystem operation failed",
        })
    }
}

impl Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|source| source as &dyn Error)
    }
}

/// Lossless, platform-qualified identity for a relative artifact path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathIdentity {
    /// A diagnostic-only rendering that is never parsed for authorization.
    pub display_path: String,
    /// Native path family (`unix` or `windows`).
    pub platform_family: &'static str,
    /// Native unit encoding used by `relative_units_base64url`.
    pub unit_encoding: &'static str,
    /// Canonical unpadded base64url native units.
    pub relative_units_base64url: String,
}

impl PathIdentity {
    /// Encodes a validated native relative path.
    ///
    /// # Errors
    ///
    /// Returns an error when the path is empty, absolute, or contains a
    /// traversal, prefix, root, or non-normal component.
    pub fn from_relative_path(path: &Path) -> Result<Self, WorkspaceError> {
        validate_relative_components(path)?;
        let bytes = native_units(path);
        validate_native_units(&bytes)?;
        Ok(Self {
            display_path: escaped_display(path),
            platform_family: platform_family(),
            unit_encoding: unit_encoding(),
            relative_units_base64url: URL_SAFE_NO_PAD.encode(bytes),
        })
    }

    /// Decodes and revalidates the native relative path on this platform.
    ///
    /// # Errors
    ///
    /// Returns an error for a platform/encoding mismatch, malformed or
    /// non-canonical base64url, or invalid decoded native path units.
    pub fn to_relative_path(&self) -> Result<PathBuf, WorkspaceError> {
        if self.platform_family != platform_family() || self.unit_encoding != unit_encoding() {
            return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
        }
        let decoded = URL_SAFE_NO_PAD
            .decode(&self.relative_units_base64url)
            .map_err(|_| WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity))?;
        if decoded.is_empty() || URL_SAFE_NO_PAD.encode(&decoded) != self.relative_units_base64url {
            return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
        }
        validate_native_units(&decoded)?;
        let path = path_from_native_units(&decoded)?;
        validate_relative_components(&path)?;
        Ok(path)
    }
}

/// Immutable bytes returned by a guarded exact read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactRead {
    /// Validated path identity used for the operation.
    pub path: PathIdentity,
    /// SHA-256 of the exact returned bytes.
    pub content_hash: String,
    /// Exact file bytes; no decoding or newline rewriting is performed.
    pub bytes: Vec<u8>,
}

/// Bounded optional repository metadata discovered without executing Git.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepositoryMetadata {
    /// Detached or symbolic HEAD object identity when safely resolved.
    pub revision: Option<String>,
    /// `unknown` for detected Git metadata or `not_applicable` otherwise.
    pub working_tree: &'static str,
}

/// Hard discovery limits selected by the active resource policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DiscoveryPolicy {
    /// Maximum number of eligible regular files.
    pub max_files: u64,
    /// Maximum bytes across eligible files.
    pub max_total_bytes: u64,
    /// Maximum bytes for any one file.
    pub max_file_bytes: u64,
    /// Maximum recursive directory depth beneath the root.
    pub max_depth: u64,
}

impl DiscoveryPolicy {
    /// Creates a validated discovery policy.
    ///
    /// # Errors
    ///
    /// Returns an error when any hard limit is zero or the per-file maximum is
    /// greater than the total-byte maximum.
    pub fn new(
        max_files: u64,
        max_total_bytes: u64,
        max_file_bytes: u64,
        max_depth: u64,
    ) -> Result<Self, WorkspaceError> {
        if max_files == 0
            || max_total_bytes == 0
            || max_file_bytes == 0
            || max_depth == 0
            || max_file_bytes > max_total_bytes
        {
            return Err(WorkspaceError::new(WorkspaceErrorCode::ResourceLimit));
        }
        Ok(Self {
            max_files,
            max_total_bytes,
            max_file_bytes,
            max_depth,
        })
    }

    fn identity(self) -> String {
        let payload = format!(
            "{{\"max_depth\":\"{}\",\"max_file_bytes\":\"{}\",\"max_files\":\"{}\",\"max_total_bytes\":\"{}\"}}",
            self.max_depth, self.max_file_bytes, self.max_files, self.max_total_bytes
        );
        structured_digest("discovery-policy", payload.as_bytes())
    }
}

/// One immutable regular-file record in a workspace snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactRecord {
    /// Lossless relative path identity.
    pub path: PathIdentity,
    /// SHA-256 of exact file bytes.
    pub content_hash: String,
    /// Exact byte length represented as an integer in the Rust API.
    pub size_bytes: u64,
}

/// Reasons discovery omitted filesystem objects.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SkipReason {
    /// A built-in cache/VCS/build directory was excluded.
    PolicyExcluded,
    /// The per-file ceiling was exceeded.
    Oversized,
    /// A symlink was encountered and not followed.
    Symlink,
    /// A socket, device, FIFO, or other non-regular object was encountered.
    SpecialFile,
    /// A file, byte, or depth ceiling stopped discovery.
    LimitReached,
    /// Metadata or content could not be read safely.
    ReadFailed,
    /// A file changed during its guarded read.
    ChangedDuringRead,
}

/// Deterministic content snapshot of eligible workspace files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSnapshot {
    /// Opaque workspace identity.
    pub workspace_identity: String,
    /// Domain-separated identity of this exact snapshot.
    pub snapshot_id: String,
    /// Discovery-policy identity applied to the snapshot.
    pub discovery_policy: String,
    /// Eligible artifacts sorted by native path units.
    pub artifacts: Vec<ArtifactRecord>,
    /// Aggregate exact bytes across eligible artifacts.
    pub eligible_bytes: u64,
    /// Omission counts by stable reason.
    pub skipped: BTreeMap<SkipReason, u64>,
    /// True only when no eligibility-affecting omission occurred.
    pub complete: bool,
    /// Optional safely resolved Git HEAD object identity.
    pub repository_revision: Option<String>,
    /// Git working-tree state; currently `unknown` or `not_applicable`.
    pub working_tree: &'static str,
}

/// An explicitly authorized, read-only directory capability.
#[derive(Debug)]
pub struct AuthorizedWorkspace {
    root: Dir,
    workspace_identity: String,
}

impl AuthorizedWorkspace {
    /// Opens one explicitly supplied root and converts it to a directory capability.
    ///
    /// # Errors
    ///
    /// Returns an error when the root cannot be resolved, is not a directory,
    /// or cannot be opened as a capability.
    pub fn open(root: &Path) -> Result<Self, WorkspaceError> {
        let resolved = std::fs::canonicalize(root).map_err(map_not_found)?;
        if !resolved
            .metadata()
            .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?
            .is_dir()
        {
            return Err(WorkspaceError::new(WorkspaceErrorCode::RootNotDirectory));
        }
        let identity = workspace_identity(&resolved);
        let root = Dir::open_ambient_dir(&resolved, ambient_authority())
            .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?;
        Ok(Self {
            root,
            workspace_identity: identity,
        })
    }

    /// Returns the opaque, location-bound workspace identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.workspace_identity
    }

    /// Reads bounded Git HEAD metadata relative to the workspace capability.
    ///
    /// External `gitdir` files, links, malformed refs, missing objects, and
    /// unsupported layouts are never followed and produce an unknown state.
    #[must_use]
    pub fn repository_metadata(&self) -> RepositoryMetadata {
        let metadata = match self.root.symlink_metadata(".git") {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return RepositoryMetadata {
                    revision: None,
                    working_tree: "not_applicable",
                };
            }
            Err(_) => {
                return RepositoryMetadata {
                    revision: None,
                    working_tree: "unknown",
                };
            }
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return RepositoryMetadata {
                revision: None,
                working_tree: "unknown",
            };
        }
        RepositoryMetadata {
            revision: self.git_head_revision(),
            working_tree: "unknown",
        }
    }

    fn git_head_revision(&self) -> Option<String> {
        let head_path = PathIdentity::from_relative_path(&Path::new(".git").join("HEAD")).ok()?;
        let head = self.read_exact(&head_path, 4096).ok()?;
        let value = std::str::from_utf8(&head.bytes).ok()?.trim();
        if let Some(reference) = value.strip_prefix("ref: ") {
            let relative = git_metadata_path(reference)?;
            let identity = PathIdentity::from_relative_path(&relative).ok()?;
            if let Ok(reference_bytes) = self.read_exact(&identity, 4096) {
                return normalized_git_object(
                    std::str::from_utf8(&reference_bytes.bytes).ok()?.trim(),
                );
            }
            return self.packed_git_reference(reference);
        }
        normalized_git_object(value)
    }

    fn packed_git_reference(&self, reference: &str) -> Option<String> {
        let path = PathIdentity::from_relative_path(&Path::new(".git").join("packed-refs")).ok()?;
        let packed = self.read_exact(&path, 1_048_576).ok()?;
        let text = std::str::from_utf8(&packed.bytes).ok()?;
        text.lines().find_map(|line| {
            let (object, name) = line.split_once(' ')?;
            if name == reference {
                normalized_git_object(object)
            } else {
                None
            }
        })
    }

    /// Checks whether an ambient directory resolves to this authorized root
    /// without disclosing the root path.
    ///
    /// # Errors
    ///
    /// Returns an error when the candidate cannot be resolved as a directory.
    pub fn is_same_root(&self, candidate: &Path) -> Result<bool, WorkspaceError> {
        let resolved = std::fs::canonicalize(candidate).map_err(map_not_found)?;
        if !resolved
            .metadata()
            .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?
            .is_dir()
        {
            return Err(WorkspaceError::new(WorkspaceErrorCode::RootNotDirectory));
        }
        Ok(workspace_identity(&resolved) == self.workspace_identity)
    }

    /// Reads a regular file relative to the held capability under a hard byte ceiling.
    ///
    /// # Errors
    ///
    /// Returns an error if identity validation or capability-relative access
    /// fails, a symlink or non-file is encountered, the byte limit is exceeded,
    /// or the file changes during the guarded read.
    pub fn read_exact(
        &self,
        identity: &PathIdentity,
        max_bytes: u64,
    ) -> Result<ExactRead, WorkspaceError> {
        let relative = identity.to_relative_path()?;
        reject_symlink_components(&self.root, &relative)?;
        let canonical = self.root.canonicalize(&relative).map_err(map_not_found)?;
        if canonical != relative {
            return Err(WorkspaceError::new(WorkspaceErrorCode::SymlinkRejected));
        }
        let before = self
            .root
            .symlink_metadata(&relative)
            .map_err(map_not_found)?;
        if before.file_type().is_symlink() {
            return Err(WorkspaceError::new(WorkspaceErrorCode::SymlinkRejected));
        }
        if !before.is_file() {
            return Err(WorkspaceError::new(WorkspaceErrorCode::UnsupportedObject));
        }
        if before.len() > max_bytes {
            return Err(WorkspaceError::new(WorkspaceErrorCode::ResourceLimit));
        }

        let mut file = self.root.open(&relative).map_err(map_not_found)?;
        let opened = file
            .metadata()
            .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?;
        if !opened.is_file() {
            return Err(WorkspaceError::new(WorkspaceErrorCode::UnsupportedObject));
        }
        let capacity = usize::try_from(opened.len().min(max_bytes))
            .map_err(|_| WorkspaceError::new(WorkspaceErrorCode::ResourceLimit))?;
        let mut bytes = Vec::with_capacity(capacity);
        file.by_ref()
            .take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
            return Err(WorkspaceError::new(WorkspaceErrorCode::ResourceLimit));
        }
        if opened.len() != u64::try_from(bytes.len()).unwrap_or(u64::MAX) {
            return Err(WorkspaceError::new(WorkspaceErrorCode::ChangedDuringRead));
        }

        Ok(ExactRead {
            path: identity.clone(),
            content_hash: sha256_digest(&bytes),
            bytes,
        })
    }

    /// Discovers and hashes eligible regular files under deterministic hard limits.
    ///
    /// # Errors
    ///
    /// Returns an error only when the authorized root itself cannot be read.
    /// Individual hostile or unreadable entries are recorded as omissions.
    pub fn snapshot(&self, policy: DiscoveryPolicy) -> Result<WorkspaceSnapshot, WorkspaceError> {
        self.snapshot_bounded(policy, Duration::from_mins(5))
    }

    /// Discovers a snapshot under an additional monotonic elapsed-time ceiling.
    ///
    /// # Errors
    ///
    /// Returns an error for a zero/excessive deadline or when the authorized
    /// root itself cannot be read. Deadline exhaustion produces a partial
    /// snapshot with an explicit limit omission.
    pub fn snapshot_bounded(
        &self,
        policy: DiscoveryPolicy,
        max_elapsed: Duration,
    ) -> Result<WorkspaceSnapshot, WorkspaceError> {
        if max_elapsed.is_zero() || max_elapsed > Duration::from_mins(5) {
            return Err(WorkspaceError::new(WorkspaceErrorCode::ResourceLimit));
        }
        let mut state = DiscoveryState::new(self, policy, max_elapsed);
        discover_directory(&self.root, Path::new(""), 0, &mut state)?;
        state.artifacts.sort_by(|left, right| {
            left.path
                .relative_units_base64url
                .cmp(&right.path.relative_units_base64url)
        });
        let policy_id = policy.identity();
        let snapshot_id = snapshot_identity(
            &self.workspace_identity,
            &policy_id,
            &state.artifacts,
            &state.skipped,
        );
        let complete = state
            .skipped
            .keys()
            .all(|reason| !skip_affects_completeness(*reason));
        let repository = self.repository_metadata();
        Ok(WorkspaceSnapshot {
            workspace_identity: self.workspace_identity.clone(),
            snapshot_id,
            discovery_policy: policy_id,
            artifacts: state.artifacts,
            eligible_bytes: state.eligible_bytes,
            complete,
            skipped: state.skipped,
            repository_revision: repository.revision,
            working_tree: repository.working_tree,
        })
    }
}

struct DiscoveryState<'workspace> {
    workspace: &'workspace AuthorizedWorkspace,
    policy: DiscoveryPolicy,
    artifacts: Vec<ArtifactRecord>,
    eligible_bytes: u64,
    skipped: BTreeMap<SkipReason, u64>,
    started: Instant,
    max_elapsed: Duration,
    timed_out: bool,
}

impl<'workspace> DiscoveryState<'workspace> {
    fn new(
        workspace: &'workspace AuthorizedWorkspace,
        policy: DiscoveryPolicy,
        max_elapsed: Duration,
    ) -> Self {
        Self {
            workspace,
            policy,
            artifacts: Vec::new(),
            eligible_bytes: 0,
            skipped: BTreeMap::new(),
            started: Instant::now(),
            max_elapsed,
            timed_out: false,
        }
    }

    fn skip(&mut self, reason: SkipReason) {
        *self.skipped.entry(reason).or_default() += 1;
    }
}

fn discover_directory(
    directory: &Dir,
    relative_directory: &Path,
    depth: u64,
    state: &mut DiscoveryState<'_>,
) -> Result<(), WorkspaceError> {
    let entries = directory
        .entries()
        .map_err(|error| WorkspaceError::io(WorkspaceErrorCode::IoFailure, error))?;
    let mut entries_ok = Vec::new();
    for entry in entries {
        if state.started.elapsed() >= state.max_elapsed {
            state.skip(SkipReason::LimitReached);
            state.timed_out = true;
            break;
        }
        match entry {
            Ok(entry) => entries_ok.push(entry),
            Err(_) => state.skip(SkipReason::ReadFailed),
        }
    }
    let mut entries = entries_ok;
    entries.sort_by(|left, right| {
        native_os_units(&left.file_name()).cmp(&native_os_units(&right.file_name()))
    });

    for entry in entries {
        let name = entry.file_name();
        if excluded_name(&name) {
            state.skip(SkipReason::PolicyExcluded);
            continue;
        }
        let relative = relative_directory.join(&name);
        let Ok(file_type) = entry.file_type() else {
            state.skip(SkipReason::ReadFailed);
            continue;
        };
        if file_type.is_symlink() {
            state.skip(SkipReason::Symlink);
            continue;
        }
        if file_type.is_dir() {
            if depth >= state.policy.max_depth {
                state.skip(SkipReason::LimitReached);
                continue;
            }
            match entry.open_dir() {
                Ok(child) => discover_directory(&child, &relative, depth + 1, state)?,
                Err(_) => state.skip(SkipReason::ReadFailed),
            }
            if state.timed_out {
                break;
            }
            continue;
        }
        if !file_type.is_file() {
            state.skip(SkipReason::SpecialFile);
            continue;
        }
        if u64::try_from(state.artifacts.len()).unwrap_or(u64::MAX) >= state.policy.max_files {
            state.skip(SkipReason::LimitReached);
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            state.skip(SkipReason::ReadFailed);
            continue;
        };
        if metadata.len() > state.policy.max_file_bytes {
            state.skip(SkipReason::Oversized);
            continue;
        }
        if state
            .eligible_bytes
            .checked_add(metadata.len())
            .is_none_or(|total| total > state.policy.max_total_bytes)
        {
            state.skip(SkipReason::LimitReached);
            continue;
        }
        let Ok(identity) = PathIdentity::from_relative_path(&relative) else {
            state.skip(SkipReason::ReadFailed);
            continue;
        };
        match state
            .workspace
            .read_exact(&identity, state.policy.max_file_bytes)
        {
            Ok(exact) => {
                let size_bytes = u64::try_from(exact.bytes.len()).unwrap_or(u64::MAX);
                state.eligible_bytes += size_bytes;
                state.artifacts.push(ArtifactRecord {
                    path: identity,
                    content_hash: exact.content_hash,
                    size_bytes,
                });
            }
            Err(error) if error.code() == WorkspaceErrorCode::ChangedDuringRead => {
                state.skip(SkipReason::ChangedDuringRead);
            }
            Err(_) => state.skip(SkipReason::ReadFailed),
        }
    }
    Ok(())
}

fn normalized_git_object(value: &str) -> Option<String> {
    if matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Some(value.to_ascii_lowercase())
    } else {
        None
    }
}

fn git_metadata_path(reference: &str) -> Option<PathBuf> {
    if reference.is_empty() || reference.contains(['\\', ':', '\0']) {
        return None;
    }
    let mut path = PathBuf::from(".git");
    for component in reference.split('/') {
        if component.is_empty() || matches!(component, "." | "..") {
            return None;
        }
        path.push(component);
    }
    Some(path)
}

fn snapshot_identity(
    workspace_identity: &str,
    policy_identity: &str,
    artifacts: &[ArtifactRecord],
    skipped: &BTreeMap<SkipReason, u64>,
) -> String {
    let artifacts_json = artifacts
        .iter()
        .map(|artifact| {
            format!(
                "{{\"content_hash\":\"{}\",\"path_units\":\"{}\",\"size_bytes\":\"{}\"}}",
                artifact.content_hash, artifact.path.relative_units_base64url, artifact.size_bytes
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let skipped_json = skipped
        .iter()
        .map(|(reason, count)| format!("\"{}\":\"{count}\"", skip_reason_name(*reason)))
        .collect::<Vec<_>>()
        .join(",");
    let payload = format!(
        "{{\"artifacts\":[{artifacts_json}],\"discovery_policy\":\"{policy_identity}\",\"skipped\":{{{skipped_json}}},\"workspace_identity\":\"{workspace_identity}\"}}"
    );
    structured_digest("workspace-snapshot", payload.as_bytes())
}

const fn skip_reason_name(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::PolicyExcluded => "policy_excluded",
        SkipReason::Oversized => "oversized",
        SkipReason::Symlink => "symlink",
        SkipReason::SpecialFile => "special_file",
        SkipReason::LimitReached => "limit_reached",
        SkipReason::ReadFailed => "read_failed",
        SkipReason::ChangedDuringRead => "changed_during_read",
    }
}

const fn skip_affects_completeness(reason: SkipReason) -> bool {
    matches!(
        reason,
        SkipReason::Oversized
            | SkipReason::LimitReached
            | SkipReason::ReadFailed
            | SkipReason::ChangedDuringRead
    )
}

fn excluded_name(name: &OsStr) -> bool {
    [".git", ".impresari-context", "target"]
        .iter()
        .any(|excluded| name == OsStr::new(excluded))
}

fn reject_symlink_components(root: &Dir, relative: &Path) -> Result<(), WorkspaceError> {
    let mut prefix = PathBuf::new();
    for component in relative.components() {
        prefix.push(component.as_os_str());
        let metadata = root.symlink_metadata(&prefix).map_err(map_not_found)?;
        if metadata.file_type().is_symlink() {
            return Err(WorkspaceError::new(WorkspaceErrorCode::SymlinkRejected));
        }
    }
    Ok(())
}

fn map_not_found(error: io::Error) -> WorkspaceError {
    let code = if error.kind() == io::ErrorKind::NotFound {
        WorkspaceErrorCode::PathNotFound
    } else {
        WorkspaceErrorCode::IoFailure
    };
    WorkspaceError::io(code, error)
}

fn validate_relative_components(path: &Path) -> Result<(), WorkspaceError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(WorkspaceError::new(WorkspaceErrorCode::PathOutsideRoot));
    }
    if path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
    {
        Ok(())
    } else {
        Err(WorkspaceError::new(WorkspaceErrorCode::PathOutsideRoot))
    }
}

#[cfg(unix)]
fn validate_native_units(bytes: &[u8]) -> Result<(), WorkspaceError> {
    let invalid = bytes.is_empty()
        || bytes.contains(&0)
        || bytes.first() == Some(&b'/')
        || bytes.last() == Some(&b'/')
        || bytes
            .split(|byte| *byte == b'/')
            .any(|component| component.is_empty() || component == b"." || component == b"..");
    if invalid {
        Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity))
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn validate_native_units(bytes: &[u8]) -> Result<(), WorkspaceError> {
    if bytes.is_empty() || bytes.len() % 2 != 0 {
        return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
    }
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    let separator = b'\\' as u16;
    let invalid = units.contains(&0)
        || units.contains(&(b'/' as u16))
        || units.first() == Some(&separator)
        || units.last() == Some(&separator)
        || (units.len() >= 2 && units[1] == b':' as u16)
        || units.split(|unit| *unit == separator).any(|component| {
            component.is_empty()
                || component == [b'.' as u16]
                || component == [b'.' as u16, b'.' as u16]
        });
    if invalid {
        Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity))
    } else {
        Ok(())
    }
}

fn structured_digest(kind: &str, payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(IDENTITY_NAMESPACE.as_bytes());
    hasher.update([0]);
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(CONTRACT_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(payload);
    digest_label(hasher.finalize())
}

fn sha256_digest(bytes: &[u8]) -> String {
    digest_label(Sha256::digest(bytes))
}

fn digest_label(bytes: impl AsRef<[u8]>) -> String {
    let mut label = String::with_capacity(71);
    label.push_str("sha256:");
    for byte in bytes.as_ref() {
        use fmt::Write as _;
        write!(label, "{byte:02x}").expect("writing to a string cannot fail");
    }
    label
}

fn workspace_identity(resolved: &Path) -> String {
    let encoded = URL_SAFE_NO_PAD.encode(native_units(resolved));
    let payload = format!(
        "{{\"path_contract_version\":\"{CONTRACT_VERSION}\",\"platform_family\":\"{}\",\"resolved_root_units_base64url\":\"{encoded}\"}}",
        platform_family()
    );
    structured_digest("workspace-root", payload.as_bytes())
}

#[cfg(unix)]
fn native_units(path: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    path.as_os_str().as_bytes().to_vec()
}

#[cfg(unix)]
fn native_os_units(value: &OsStr) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt as _;
    value.as_bytes().to_vec()
}

#[cfg(windows)]
fn native_units(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt as _;
    path.as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect()
}

#[cfg(windows)]
fn native_os_units(value: &OsStr) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt as _;
    value.encode_wide().flat_map(u16::to_le_bytes).collect()
}

#[cfg(unix)]
fn path_from_native_units(bytes: &[u8]) -> Result<PathBuf, WorkspaceError> {
    use std::os::unix::ffi::OsStringExt as _;
    if bytes.contains(&0) || bytes.first() == Some(&b'/') || bytes.last() == Some(&b'/') {
        return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
    }
    Ok(PathBuf::from(OsString::from_vec(bytes.to_vec())))
}

#[cfg(windows)]
fn path_from_native_units(bytes: &[u8]) -> Result<PathBuf, WorkspaceError> {
    use std::os::windows::ffi::OsStringExt as _;
    if bytes.len() % 2 != 0 {
        return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
    }
    let units = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();
    if units.contains(&0) || units.contains(&(b'/' as u16)) {
        return Err(WorkspaceError::new(WorkspaceErrorCode::InvalidPathIdentity));
    }
    Ok(PathBuf::from(OsString::from_wide(&units)))
}

#[cfg(unix)]
const fn platform_family() -> &'static str {
    "unix"
}

#[cfg(windows)]
const fn platform_family() -> &'static str {
    "windows"
}

#[cfg(unix)]
const fn unit_encoding() -> &'static str {
    "unix_bytes"
}

#[cfg(windows)]
const fn unit_encoding() -> &'static str {
    "windows_utf16le"
}

fn escaped_display(path: &Path) -> String {
    path.as_os_str().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    struct TestRoot(PathBuf);

    static NEXT_TEST_ROOT: AtomicU64 = AtomicU64::new(0);

    impl TestRoot {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let sequence = NEXT_TEST_ROOT.fetch_add(1, Ordering::Relaxed);
            let process = std::process::id();
            let path = std::env::temp_dir()
                .join(format!("impresari-context-{process}-{nonce}-{sequence}"));
            fs::create_dir(&path).expect("create isolated test root");
            Self(path)
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn exact_read_is_relative_bounded_and_hashed() {
        let root = TestRoot::new();
        fs::create_dir(root.0.join("src")).expect("create source directory");
        let source_path = Path::new("src").join("lib.rs");
        fs::write(root.0.join(&source_path), b"pub fn answer() -> u8 { 42 }\n")
            .expect("write fixture");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let identity = PathIdentity::from_relative_path(&source_path).expect("path identity");
        let exact = workspace
            .read_exact(&identity, 1024)
            .expect("bounded exact read");

        assert_eq!(exact.bytes, b"pub fn answer() -> u8 { 42 }\n");
        assert_eq!(exact.path, identity);
        assert_eq!(exact.content_hash.len(), 71);
        assert!(workspace.identity().starts_with("sha256:"));
    }

    #[test]
    fn traversal_and_oversized_reads_fail_closed() {
        let root = TestRoot::new();
        fs::write(root.0.join("small.txt"), b"12345").expect("write fixture");
        assert_eq!(
            PathIdentity::from_relative_path(Path::new("../outside"))
                .expect_err("traversal must fail")
                .code(),
            WorkspaceErrorCode::PathOutsideRoot
        );
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let identity =
            PathIdentity::from_relative_path(Path::new("small.txt")).expect("path identity");
        assert_eq!(
            workspace
                .read_exact(&identity, 4)
                .expect_err("oversized read must fail")
                .code(),
            WorkspaceErrorCode::ResourceLimit
        );
    }

    #[cfg(unix)]
    #[test]
    fn noncanonical_native_separators_are_rejected() {
        use std::os::unix::ffi::OsStringExt as _;

        for bytes in [b"a//b".to_vec(), b"a/".to_vec()] {
            let path = PathBuf::from(OsString::from_vec(bytes));
            assert_eq!(
                PathIdentity::from_relative_path(&path)
                    .expect_err("noncanonical separators must fail")
                    .code(),
                WorkspaceErrorCode::InvalidPathIdentity
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlink_is_rejected_even_when_target_is_inside_root() {
        use std::os::unix::fs::symlink;

        let root = TestRoot::new();
        fs::write(root.0.join("target.txt"), b"inside").expect("write fixture");
        symlink("target.txt", root.0.join("link.txt")).expect("create symlink");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let identity =
            PathIdentity::from_relative_path(Path::new("link.txt")).expect("path identity");
        assert_eq!(
            workspace
                .read_exact(&identity, 1024)
                .expect_err("symlink must fail")
                .code(),
            WorkspaceErrorCode::SymlinkRejected
        );
    }

    #[cfg(unix)]
    #[test]
    fn intermediate_symlink_is_rejected_with_stable_code() {
        use std::os::unix::fs::symlink;

        let root = TestRoot::new();
        fs::create_dir(root.0.join("actual")).expect("create actual directory");
        fs::write(root.0.join("actual/file.txt"), b"inside").expect("write fixture");
        symlink("actual", root.0.join("alias")).expect("create symlink");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let identity =
            PathIdentity::from_relative_path(Path::new("alias/file.txt")).expect("path identity");
        assert_eq!(
            workspace
                .read_exact(&identity, 1024)
                .expect_err("intermediate symlink must fail")
                .code(),
            WorkspaceErrorCode::SymlinkRejected
        );
    }

    #[test]
    fn snapshots_are_deterministic_and_content_sensitive() {
        let root = TestRoot::new();
        fs::create_dir(root.0.join("src")).expect("create source directory");
        fs::write(root.0.join("src/b.rs"), b"b").expect("write b");
        fs::write(root.0.join("src/a.rs"), b"a").expect("write a");
        fs::create_dir(root.0.join("target")).expect("create excluded directory");
        fs::write(root.0.join("target/generated"), b"ignored").expect("write excluded file");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let policy = DiscoveryPolicy::new(10, 1024, 128, 8).expect("valid policy");

        let first = workspace.snapshot(policy).expect("first snapshot");
        let repeated = workspace.snapshot(policy).expect("repeat snapshot");
        assert_eq!(first.snapshot_id, repeated.snapshot_id);
        assert_eq!(first.artifacts.len(), 2);
        assert!(first.complete);
        assert_eq!(first.skipped.get(&SkipReason::PolicyExcluded), Some(&1));

        fs::write(root.0.join("src/a.rs"), b"changed").expect("change source");
        let changed = workspace.snapshot(policy).expect("changed snapshot");
        assert_ne!(first.snapshot_id, changed.snapshot_id);
    }

    #[test]
    fn snapshot_limits_produce_explicit_partial_state() {
        let root = TestRoot::new();
        fs::write(root.0.join("a.txt"), b"a").expect("write a");
        fs::write(root.0.join("b.txt"), b"b").expect("write b");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let policy = DiscoveryPolicy::new(1, 1024, 128, 8).expect("valid policy");
        let snapshot = workspace.snapshot(policy).expect("limited snapshot");

        assert_eq!(snapshot.artifacts.len(), 1);
        assert!(!snapshot.complete);
        assert_eq!(snapshot.skipped.get(&SkipReason::LimitReached), Some(&1));
    }

    #[test]
    fn git_metadata_is_bounded_capability_relative_and_never_executes_git() {
        const REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
        let branch = TestRoot::new();
        let git = branch.0.join(".git");
        fs::create_dir_all(git.join("refs").join("heads")).expect("refs");
        fs::write(git.join("HEAD"), b"ref: refs/heads/main\n").expect("head");
        fs::write(
            git.join("refs").join("heads").join("main"),
            format!("{REVISION}\n"),
        )
        .expect("branch ref");
        let metadata = AuthorizedWorkspace::open(&branch.0)
            .expect("workspace")
            .repository_metadata();
        assert_eq!(metadata.revision.as_deref(), Some(REVISION));
        assert_eq!(metadata.working_tree, "unknown");

        fs::write(git.join("HEAD"), format!("{REVISION}\n")).expect("detached head");
        assert_eq!(
            AuthorizedWorkspace::open(&branch.0)
                .expect("workspace")
                .repository_metadata()
                .revision
                .as_deref(),
            Some(REVISION)
        );

        let packed = TestRoot::new();
        fs::create_dir(packed.0.join(".git")).expect("git directory");
        fs::write(
            packed.0.join(".git").join("HEAD"),
            b"ref: refs/heads/release\n",
        )
        .expect("head");
        fs::write(
            packed.0.join(".git").join("packed-refs"),
            format!("# pack-refs\n{REVISION} refs/heads/release\n"),
        )
        .expect("packed refs");
        assert_eq!(
            AuthorizedWorkspace::open(&packed.0)
                .expect("workspace")
                .repository_metadata()
                .revision
                .as_deref(),
            Some(REVISION)
        );

        let external = TestRoot::new();
        fs::write(
            external.0.join(".git"),
            b"gitdir: /outside/not-authorized\n",
        )
        .expect("gitdir file");
        let metadata = AuthorizedWorkspace::open(&external.0)
            .expect("workspace")
            .repository_metadata();
        assert_eq!(metadata.revision, None);
        assert_eq!(metadata.working_tree, "unknown");

        let plain = TestRoot::new();
        assert_eq!(
            AuthorizedWorkspace::open(&plain.0)
                .expect("workspace")
                .repository_metadata()
                .working_tree,
            "not_applicable"
        );
    }
}
