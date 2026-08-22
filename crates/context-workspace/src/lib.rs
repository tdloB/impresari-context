// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Workspace authorization, discovery, snapshots, and exact reads."]

use std::{
    error::Error,
    ffi::OsString,
    fmt,
    io::{self, Read},
    path::{Component, Path, PathBuf},
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

#[cfg(windows)]
fn native_units(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt as _;
    path.as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect()
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
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    struct TestRoot(PathBuf);

    impl TestRoot {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("impresari-context-{nonce}"));
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
        fs::write(root.0.join("src/lib.rs"), b"pub fn answer() -> u8 { 42 }\n")
            .expect("write fixture");
        let workspace = AuthorizedWorkspace::open(&root.0).expect("authorize root");
        let identity =
            PathIdentity::from_relative_path(Path::new("src/lib.rs")).expect("path identity");
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
}
