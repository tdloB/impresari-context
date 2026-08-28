//! Shared, provider-neutral primitives for production evaluation adapters.

#![forbid(unsafe_code)]

use crate::agent_eval::{EvidenceCitation, Usage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Maximum bytes returned by one repository read tool call.
pub const MAX_TOOL_READ_BYTES: usize = 65_536;
const MAX_SOURCE_FILES: usize = 10_000;

/// Model-produced answer format. Digests are deliberately absent because the
/// trusted adapter derives them from the frozen source.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAnswer {
    /// Final answer text scored in bounded process memory.
    pub answer: String,
    /// Source ranges selected by the model.
    pub evidence: Vec<ModelEvidenceRange>,
}

/// One model-selected source range without a model-controlled digest.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelEvidenceRange {
    /// Allow-listed repository-relative path.
    pub path: String,
    /// Inclusive first line.
    pub line_start: u32,
    /// Inclusive final line.
    pub line_end: u32,
}

/// Result returned by the repository read tool.
#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryRead {
    /// Canonical repository-relative path.
    pub path: String,
    /// Inclusive first line.
    pub line_start: u32,
    /// Inclusive final line.
    pub line_end: u32,
    /// Exact requested text, joined with `\n` between source lines.
    pub content: String,
    /// Adapter-derived SHA-256 of `content`.
    pub sha256: String,
}

/// Read and tool counters observed by the trusted tool dispatcher.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ToolCounters {
    /// All repository-tool invocations, including invalid requests.
    pub tool_calls: u64,
    /// Successful repository file reads.
    pub repository_file_reads: u64,
    /// Successful reads of a path previously read in this arm.
    pub repeated_repository_file_reads: u64,
}

/// Per-arm repository tool dispatcher with a frozen source allowlist.
pub struct RepositoryToolBoundary {
    root: PathBuf,
    allowed: BTreeSet<String>,
    successfully_read: BTreeSet<String>,
    counters: ToolCounters,
}

impl RepositoryToolBoundary {
    /// Creates a dispatcher after verifying the root and every allow-listed
    /// source path. Symlinks and non-regular files fail closed.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid root, unsafe path, symlink, duplicate,
    /// or non-regular file.
    pub fn new(root: &Path, source_files: &[String]) -> Result<Self, String> {
        let root = root
            .canonicalize()
            .map_err(|error| format!("canonicalize repository root: {error}"))?;
        if !root.is_dir() || source_files.is_empty() {
            return Err("repository root and source allowlist are required".to_owned());
        }
        let mut allowed = BTreeSet::new();
        for relative in source_files {
            if !allowed.insert(relative.clone()) {
                return Err(format!("duplicate source path {relative:?}"));
            }
            let _ = resolve_regular_file(&root, relative)?;
        }
        Ok(Self {
            root,
            allowed,
            successfully_read: BTreeSet::new(),
            counters: ToolCounters::default(),
        })
    }

    /// Returns the exact sorted source allowlist and counts one tool call.
    pub fn list_files(&mut self) -> Vec<String> {
        self.counters.tool_calls = self.counters.tool_calls.saturating_add(1);
        self.allowed.iter().cloned().collect()
    }

    /// Counts a malformed or unknown repository-tool invocation that was
    /// rejected before a concrete operation could run.
    pub fn record_rejected_tool_call(&mut self) {
        self.counters.tool_calls = self.counters.tool_calls.saturating_add(1);
    }

    /// Reads an inclusive line range through the measured tool boundary.
    ///
    /// # Errors
    ///
    /// Returns an error for a non-allow-listed path, invalid range, oversized
    /// response, changed file type, or filesystem failure.
    pub fn read_file(
        &mut self,
        relative: &str,
        line_start: u32,
        line_end: u32,
    ) -> Result<RepositoryRead, String> {
        self.counters.tool_calls = self.counters.tool_calls.saturating_add(1);
        let read = self.read_range(relative, line_start, line_end)?;
        self.counters.repository_file_reads = self.counters.repository_file_reads.saturating_add(1);
        if !self.successfully_read.insert(relative.to_owned()) {
            self.counters.repeated_repository_file_reads = self
                .counters
                .repeated_repository_file_reads
                .saturating_add(1);
        }
        Ok(read)
    }

    fn read_range(
        &self,
        relative: &str,
        line_start: u32,
        line_end: u32,
    ) -> Result<RepositoryRead, String> {
        if !self.allowed.contains(relative) {
            return Err(format!("repository path {relative:?} is not allow-listed"));
        }
        if line_start == 0 || line_end < line_start {
            return Err("repository line range is invalid".to_owned());
        }
        let path = resolve_regular_file(&self.root, relative)?;
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("read repository path {relative:?}: {error}"))?;
        let lines = text.lines().collect::<Vec<_>>();
        let start = usize::try_from(line_start - 1)
            .map_err(|_| "repository line range is invalid".to_owned())?;
        let end =
            usize::try_from(line_end).map_err(|_| "repository line range is invalid".to_owned())?;
        if start >= lines.len() || end > lines.len() {
            return Err(format!("repository line range exceeds {relative:?}"));
        }
        let content = lines[start..end].join("\n");
        if content.len() > MAX_TOOL_READ_BYTES {
            return Err(format!(
                "repository read exceeds {MAX_TOOL_READ_BYTES} bytes"
            ));
        }
        Ok(RepositoryRead {
            path: relative.to_owned(),
            line_start,
            line_end,
            sha256: hash_bytes(content.as_bytes()),
            content,
        })
    }

    /// Converts model-selected source ranges into adapter-derived citations.
    /// Each citation is independently verified with the same allowlist and
    /// path checks as a model tool read. Adapter-internal verification is not
    /// included in model tool/read counters.
    ///
    /// # Errors
    ///
    /// Returns an error when any citation is unsafe or invalid.
    pub fn derive_citations(
        &mut self,
        ranges: &[ModelEvidenceRange],
    ) -> Result<Vec<EvidenceCitation>, String> {
        if ranges.len() > 32 {
            return Err("model evidence must contain at most 32 ranges".to_owned());
        }
        ranges
            .iter()
            .map(|range| {
                let read = self.read_range(&range.path, range.line_start, range.line_end)?;
                Ok(EvidenceCitation {
                    path: read.path,
                    line_start: read.line_start,
                    line_end: read.line_end,
                    sha256: read.sha256,
                })
            })
            .collect()
    }

    /// Returns current trusted counters.
    #[must_use]
    pub const fn counters(&self) -> ToolCounters {
        self.counters
    }

    /// Adds trusted tool-boundary counters to provider token usage.
    #[must_use]
    pub fn apply_counters(&self, mut usage: Usage) -> Usage {
        usage.tool_calls = self.counters.tool_calls;
        usage.repository_file_reads = self.counters.repository_file_reads;
        usage.repeated_repository_file_reads = self.counters.repeated_repository_file_reads;
        usage
    }
}

/// Computes the harness-compatible fingerprint over exact relative paths and
/// source bytes after applying the same regular-file and symlink checks.
///
/// # Errors
///
/// Returns an error for unsafe, duplicate, missing, symlinked, or non-regular
/// source files.
pub fn source_fingerprint(root: &Path, source_files: &[String]) -> Result<String, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize repository root: {error}"))?;
    let mut files = source_files.to_vec();
    files.sort();
    if files.is_empty() || files.len() > MAX_SOURCE_FILES {
        return Err("source allowlist size is invalid".to_owned());
    }
    let mut previous: Option<&str> = None;
    let mut hasher = Sha256::new();
    for relative in &files {
        if previous == Some(relative) {
            return Err(format!("duplicate source path {relative:?}"));
        }
        previous = Some(relative);
        let path = resolve_regular_file(&root, relative)?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(&path)
                .map_err(|error| format!("read repository path {relative:?}: {error}"))?,
        );
        hasher.update([0]);
    }
    Ok(hex_digest(hasher.finalize()))
}

/// Copies only allow-listed regular files into an empty isolated source root.
/// This prevents the packet engine from discovering unlisted repository data.
///
/// # Errors
///
/// Returns an error if the destination is not empty or a source/destination
/// path is unsafe, symlinked, missing, or cannot be copied.
pub fn materialize_source(
    root: &Path,
    source_files: &[String],
    destination: &Path,
) -> Result<(), String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("canonicalize repository root: {error}"))?;
    fs::create_dir_all(destination)
        .map_err(|error| format!("create isolated source root: {error}"))?;
    if fs::read_dir(destination)
        .map_err(|error| format!("inspect isolated source root: {error}"))?
        .next()
        .is_some()
    {
        return Err("isolated source root must be empty".to_owned());
    }
    for relative in source_files {
        let source = resolve_regular_file(&root, relative)?;
        let target = destination.join(relative);
        let parent = target
            .parent()
            .ok_or_else(|| format!("source path {relative:?} has no parent"))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("create isolated source directory: {error}"))?;
        fs::copy(&source, &target)
            .map_err(|error| format!("copy source path {relative:?}: {error}"))?;
    }
    Ok(())
}

fn resolve_regular_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if !path.is_relative()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe repository path {relative:?}"));
    }
    let mut current = root.to_path_buf();
    for component in path.components() {
        let Component::Normal(part) = component else {
            return Err(format!("unsafe repository path {relative:?}"));
        };
        current.push(part);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|error| format!("inspect repository path {relative:?}: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("symlink is not allowed in {relative:?}"));
        }
    }
    let metadata = fs::metadata(&current)
        .map_err(|error| format!("inspect repository path {relative:?}: {error}"))?;
    if !metadata.is_file() {
        return Err(format!(
            "repository path {relative:?} is not a regular file"
        ));
    }
    Ok(current)
}

fn hash_bytes(bytes: &[u8]) -> String {
    hex_digest(Sha256::digest(bytes))
}

fn hex_digest(digest: impl IntoIterator<Item = u8>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::from("sha256:");
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DIRECTORY_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock after epoch")
                .as_nanos();
            let sequence = TEST_DIRECTORY_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-production-adapter-{}-{nonce}-{sequence}",
                std::process::id(),
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn counts_only_model_reads_and_not_adapter_derived_citations() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("one.rs"), "first\nsecond\n").expect("write source");
        let mut tools =
            RepositoryToolBoundary::new(&directory.0, &["one.rs".into()]).expect("create tools");
        assert_eq!(tools.list_files(), vec!["one.rs"]);
        let first = tools.read_file("one.rs", 1, 1).expect("first read");
        assert_eq!(first.content, "first");
        let citations = tools
            .derive_citations(&[ModelEvidenceRange {
                path: "one.rs".into(),
                line_start: 2,
                line_end: 2,
            }])
            .expect("derive citation");
        assert_eq!(citations[0].sha256, hash_bytes(b"second"));
        assert_eq!(
            tools.counters(),
            ToolCounters {
                tool_calls: 2,
                repository_file_reads: 1,
                repeated_repository_file_reads: 0,
            }
        );
    }

    #[test]
    fn rejects_escape_and_invalid_range_without_counting_successful_reads() {
        let directory = TestDirectory::new();
        fs::write(directory.0.join("one.rs"), "one\n").expect("write source");
        let mut tools =
            RepositoryToolBoundary::new(&directory.0, &["one.rs".into()]).expect("create tools");
        assert!(tools.read_file("../one.rs", 1, 1).is_err());
        assert!(tools.read_file("one.rs", 2, 2).is_err());
        assert_eq!(tools.counters().tool_calls, 2);
        assert_eq!(tools.counters().repository_file_reads, 0);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_allowlist_entry() {
        use std::os::unix::fs::symlink;

        let directory = TestDirectory::new();
        fs::write(directory.0.join("real.rs"), "one\n").expect("write source");
        symlink("real.rs", directory.0.join("linked.rs")).expect("create symlink");
        assert!(RepositoryToolBoundary::new(&directory.0, &["linked.rs".into()]).is_err());
    }
}
