// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Bounded deterministic lexical retrieval with exact source evidence."]

use std::{
    error::Error,
    fmt,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use context_core::{
    EvidenceArtifact, EvidenceExcerpt, EvidenceExtraction, EvidencePath, EvidenceRecord,
    EvidenceSpan,
};
use context_store::{CacheError, CachedArtifact, WorkspaceCache};
use context_workspace::{AuthorizedWorkspace, PathIdentity, WorkspaceError, WorkspaceSnapshot};
use sha2::{Digest, Sha256};

/// Stable retrieval failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalErrorCode {
    /// Query or budget input is invalid.
    InvalidInput,
    /// Current source no longer matches the selected snapshot.
    StaleState,
    /// A hard resource limit prevented safe execution.
    ResourceLimit,
    /// Workspace evidence could not be read safely.
    EvidenceUnavailable,
    /// Derived cache access failed.
    CacheFailure,
}

/// Safe retrieval error.
#[derive(Debug)]
pub struct RetrievalError {
    code: RetrievalErrorCode,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl RetrievalError {
    fn new(code: RetrievalErrorCode) -> Self {
        Self { code, source: None }
    }
    fn source(code: RetrievalErrorCode, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            code,
            source: Some(Box::new(source)),
        }
    }
    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(&self) -> RetrievalErrorCode {
        self.code
    }
}

impl fmt::Display for RetrievalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.code {
            RetrievalErrorCode::InvalidInput => "invalid retrieval input",
            RetrievalErrorCode::StaleState => "workspace snapshot is stale",
            RetrievalErrorCode::ResourceLimit => "retrieval resource limit exceeded",
            RetrievalErrorCode::EvidenceUnavailable => "evidence unavailable",
            RetrievalErrorCode::CacheFailure => "derived cache unavailable",
        })
    }
}

impl Error for RetrievalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}

/// Independent hard limits for one search.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchBudget {
    /// Maximum files whose source may be verified.
    pub max_files: u64,
    /// Maximum exact matches returned.
    pub max_matches: u64,
    /// Maximum excerpt bytes per evidence item.
    pub max_excerpt_bytes: u64,
    /// Wall-clock ceiling.
    pub max_elapsed: Duration,
}

impl SearchBudget {
    /// Creates a validated search budget.
    ///
    /// # Errors
    ///
    /// Fails when any limit is zero or exceeds the accepted resource profile.
    pub fn new(
        max_files: u64,
        max_matches: u64,
        max_excerpt_bytes: u64,
        max_elapsed: Duration,
    ) -> Result<Self, RetrievalError> {
        if max_files == 0
            || max_files > 1_000_000
            || max_matches == 0
            || max_matches > 10_000
            || max_excerpt_bytes == 0
            || max_excerpt_bytes > 65_536
            || max_elapsed.is_zero()
            || max_elapsed > Duration::from_mins(5)
        {
            return Err(RetrievalError::new(RetrievalErrorCode::ResourceLimit));
        }
        Ok(Self {
            max_files,
            max_matches,
            max_excerpt_bytes,
            max_elapsed,
        })
    }
}

/// Exact byte-authoritative evidence from current source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    /// Domain-separated evidence identity.
    pub evidence_id: String,
    /// Snapshot against which source was verified.
    pub workspace_snapshot: String,
    /// Lossless artifact path identity.
    pub path: PathIdentity,
    /// Verified source content hash.
    pub content_hash: String,
    /// Inclusive start byte.
    pub start_byte: u64,
    /// Exclusive end byte.
    pub end_byte: u64,
    /// Exact bounded bytes around the match.
    pub excerpt: Vec<u8>,
    /// Offset of the match start within `excerpt`.
    pub match_start_in_excerpt: u64,
    /// Offset of the match end within `excerpt`.
    pub match_end_in_excerpt: u64,
    /// Source decoding classification.
    pub decoding: &'static str,
    /// Extraction method that produced the match.
    pub extraction_method: &'static str,
}

/// Deterministic bounded search result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    /// Exact verified matches in stable path/span order.
    pub matches: Vec<Evidence>,
    /// Whether an independent hard limit omitted possible results.
    pub truncated: bool,
    /// Stable reasons for truncation.
    pub truncation_reasons: Vec<&'static str>,
}

/// Builds and atomically promotes the contentless lexical candidate generation.
///
/// # Errors
///
/// Fails if source differs from the snapshot, cannot be read, term extraction
/// exceeds limits, or the cache transaction fails.
pub fn build_lexical_generation(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    cache: &mut WorkspaceCache,
) -> Result<(), RetrievalError> {
    if workspace.identity() != snapshot.workspace_identity
        || cache.workspace_identity() != snapshot.workspace_identity
    {
        return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
    }
    let mut cached = Vec::with_capacity(snapshot.artifacts.len());
    for artifact in &snapshot.artifacts {
        let exact = workspace
            .read_exact(&artifact.path, artifact.size_bytes.max(1))
            .map_err(map_workspace)?;
        if exact.content_hash != artifact.content_hash
            || exact.bytes.len() as u64 != artifact.size_bytes
        {
            return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
        }
        cached.push(CachedArtifact {
            path_units: artifact.path.relative_units_base64url.clone(),
            display_path: artifact.path.display_path.clone(),
            content_hash: artifact.content_hash.clone(),
            size_bytes: artifact.size_bytes,
            terms: lexical_document(&exact.bytes)?,
        });
    }
    cache
        .promote(&snapshot.snapshot_id, &snapshot.discovery_policy, &cached)
        .map_err(map_cache)?;
    Ok(())
}

/// Searches exact literal bytes across the snapshot.
///
/// # Errors
///
/// Fails for an empty/oversized query, stale source, unsafe reads, or a timeout.
pub fn search_literal(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    needle: &[u8],
    budget: SearchBudget,
) -> Result<SearchResult, RetrievalError> {
    if needle.is_empty() || needle.len() > 8192 {
        return Err(RetrievalError::new(RetrievalErrorCode::InvalidInput));
    }
    search_candidates(
        workspace,
        snapshot,
        &snapshot
            .artifacts
            .iter()
            .map(|a| a.path.relative_units_base64url.clone())
            .collect::<Vec<_>>(),
        &[needle.to_vec()],
        false,
        "literal_search",
        budget,
    )
}

/// Searches normalized lexical terms using FTS5 candidates and exact source verification.
///
/// # Errors
///
/// Fails for an invalid query, stale source, unsafe reads, cache failure, or timeout.
pub fn search_lexical(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    cache: &WorkspaceCache,
    query: &str,
    budget: SearchBudget,
) -> Result<SearchResult, RetrievalError> {
    let mut terms = lexical_terms(query.as_bytes());
    terms.sort();
    terms.dedup();
    if terms.is_empty() || terms.len() > 16 {
        return Err(RetrievalError::new(RetrievalErrorCode::InvalidInput));
    }
    let candidates = cache
        .lexical_candidates(&terms, budget.max_files)
        .map_err(map_cache)?;
    let current = cache.current().map_err(map_cache)?;
    if current
        .as_ref()
        .map(|generation| generation.snapshot_id.as_str())
        != Some(snapshot.snapshot_id.as_str())
    {
        return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
    }
    let needles = terms
        .iter()
        .map(|term| term.as_bytes().to_vec())
        .collect::<Vec<_>>();
    search_candidates(
        workspace,
        snapshot,
        &candidates,
        &needles,
        true,
        "lexical_search",
        budget,
    )
}

/// Converts verified in-memory evidence to the public byte-safe wire contract.
#[must_use]
pub fn evidence_record(evidence: &Evidence) -> EvidenceRecord {
    EvidenceRecord {
        schema_name: "evidence".into(),
        schema_version: "1.0.0".into(),
        evidence_id: evidence.evidence_id.clone(),
        workspace_snapshot: evidence.workspace_snapshot.clone(),
        artifact: EvidenceArtifact {
            path: EvidencePath {
                display_path: evidence.path.display_path.clone(),
                platform_family: evidence.path.platform_family.into(),
                unit_encoding: evidence.path.unit_encoding.into(),
                relative_units_base64url: evidence.path.relative_units_base64url.clone(),
            },
            content_hash: evidence.content_hash.clone(),
            file_kind: "regular_file".into(),
            decoding: evidence.decoding.into(),
        },
        span: EvidenceSpan {
            start_byte: evidence.start_byte.to_string(),
            end_byte: evidence.end_byte.to_string(),
        },
        excerpt: EvidenceExcerpt {
            encoding: "base64url".into(),
            bytes_base64url: URL_SAFE_NO_PAD.encode(&evidence.excerpt),
            match_start_byte: evidence.match_start_in_excerpt.to_string(),
            match_end_byte: evidence.match_end_in_excerpt.to_string(),
        },
        kind: "exact_source".into(),
        extraction: EvidenceExtraction {
            method: evidence.extraction_method.into(),
            version: "1.0.0".into(),
        },
        confidence: "confirmed".into(),
        trust: "untrusted_workspace_content".into(),
        freshness: "current".into(),
        sensitivity: Some("normal".into()),
    }
}

/// Expands evidence around its exact span under a separate hard byte ceiling.
///
/// # Errors
///
/// Fails when the evidence/snapshot binding is invalid, current source differs,
/// or the requested match itself cannot fit within the expansion ceiling.
pub fn expand_evidence(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    evidence: &Evidence,
    before_bytes: u64,
    after_bytes: u64,
    max_bytes: u64,
) -> Result<Evidence, RetrievalError> {
    if max_bytes == 0 || max_bytes > 65_536 || evidence.workspace_snapshot != snapshot.snapshot_id {
        return Err(RetrievalError::new(RetrievalErrorCode::ResourceLimit));
    }
    let artifact = snapshot
        .artifacts
        .iter()
        .find(|artifact| {
            artifact.path.relative_units_base64url == evidence.path.relative_units_base64url
                && artifact.content_hash == evidence.content_hash
        })
        .ok_or_else(|| RetrievalError::new(RetrievalErrorCode::StaleState))?;
    let exact = workspace
        .read_exact(&artifact.path, artifact.size_bytes.max(1))
        .map_err(map_workspace)?;
    if exact.content_hash != artifact.content_hash {
        return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
    }
    let start = usize::try_from(evidence.start_byte)
        .map_err(|_| RetrievalError::new(RetrievalErrorCode::InvalidInput))?;
    let end = usize::try_from(evidence.end_byte)
        .map_err(|_| RetrievalError::new(RetrievalErrorCode::InvalidInput))?;
    let max = usize::try_from(max_bytes).unwrap_or(usize::MAX);
    if start > end || end > exact.bytes.len() || end - start > max {
        return Err(RetrievalError::new(RetrievalErrorCode::ResourceLimit));
    }
    let before = usize::try_from(before_bytes)
        .unwrap_or(usize::MAX)
        .min(max - (end - start));
    let excerpt_start = start.saturating_sub(before);
    let remaining = max - (end - excerpt_start);
    let after = usize::try_from(after_bytes)
        .unwrap_or(usize::MAX)
        .min(remaining);
    let excerpt_end = end.saturating_add(after).min(exact.bytes.len());
    let mut expanded = evidence.clone();
    expanded.excerpt = exact.bytes[excerpt_start..excerpt_end].to_vec();
    expanded.match_start_in_excerpt = u64::try_from(start - excerpt_start).unwrap_or(u64::MAX);
    expanded.match_end_in_excerpt = u64::try_from(end - excerpt_start).unwrap_or(u64::MAX);
    Ok(expanded)
}

fn search_candidates(
    workspace: &AuthorizedWorkspace,
    snapshot: &WorkspaceSnapshot,
    candidates: &[String],
    needles: &[Vec<u8>],
    ascii_case_insensitive: bool,
    extraction_method: &'static str,
    budget: SearchBudget,
) -> Result<SearchResult, RetrievalError> {
    if workspace.identity() != snapshot.workspace_identity {
        return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
    }
    if needles.is_empty()
        || needles.iter().any(|needle| {
            u64::try_from(needle.len()).unwrap_or(u64::MAX) > budget.max_excerpt_bytes
        })
    {
        return Err(RetrievalError::new(RetrievalErrorCode::ResourceLimit));
    }
    let started = Instant::now();
    let mut evidence = Vec::new();
    let mut reasons = Vec::new();
    let file_limit = usize::try_from(budget.max_files).unwrap_or(usize::MAX);
    for path_units in candidates.iter().take(file_limit) {
        if started.elapsed() > budget.max_elapsed {
            reasons.push("elapsed_limit");
            break;
        }
        let artifact = snapshot
            .artifacts
            .iter()
            .find(|artifact| artifact.path.relative_units_base64url == *path_units)
            .ok_or_else(|| RetrievalError::new(RetrievalErrorCode::StaleState))?;
        let exact = workspace
            .read_exact(&artifact.path, artifact.size_bytes.max(1))
            .map_err(map_workspace)?;
        if exact.content_hash != artifact.content_hash {
            return Err(RetrievalError::new(RetrievalErrorCode::StaleState));
        }
        for needle in needles {
            let remaining = budget.max_matches.saturating_sub(evidence.len() as u64);
            let remaining_usize = usize::try_from(remaining).unwrap_or(usize::MAX);
            let starts = find_bounded(
                &exact.bytes,
                needle,
                ascii_case_insensitive,
                remaining_usize.saturating_add(1),
            );
            if starts.is_empty() {
                return Err(RetrievalError::new(RetrievalErrorCode::CacheFailure));
            }
            if starts.len() > remaining_usize {
                reasons.push("match_limit");
            }
            for start in starts.into_iter().take(remaining_usize) {
                evidence.push(make_evidence(
                    snapshot,
                    artifact,
                    &exact.bytes,
                    start,
                    start + needle.len(),
                    budget.max_excerpt_bytes,
                    extraction_method,
                ));
            }
            if reasons.contains(&"match_limit") {
                break;
            }
        }
        if reasons.contains(&"match_limit") {
            break;
        }
    }
    if candidates.len() as u64 > budget.max_files {
        reasons.push("file_limit");
    }
    evidence.sort_by(|left, right| {
        left.path
            .relative_units_base64url
            .cmp(&right.path.relative_units_base64url)
            .then(left.start_byte.cmp(&right.start_byte))
    });
    reasons.sort_unstable();
    reasons.dedup();
    Ok(SearchResult {
        truncated: !reasons.is_empty(),
        truncation_reasons: reasons,
        matches: evidence,
    })
}

fn make_evidence(
    snapshot: &WorkspaceSnapshot,
    artifact: &context_workspace::ArtifactRecord,
    bytes: &[u8],
    start: usize,
    end: usize,
    max_excerpt: u64,
    extraction_method: &'static str,
) -> Evidence {
    let cap = usize::try_from(max_excerpt).unwrap_or(usize::MAX);
    let wanted = end - start;
    let padding = cap.saturating_sub(wanted) / 2;
    let excerpt_start = start.saturating_sub(padding);
    let excerpt_end = bytes.len().min(excerpt_start.saturating_add(cap).max(end));
    let payload = format!(
        "{{\"content_hash\":\"{}\",\"end_byte\":\"{end}\",\"path_units\":\"{}\",\"snapshot\":\"{}\",\"start_byte\":\"{start}\"}}",
        artifact.content_hash, artifact.path.relative_units_base64url, snapshot.snapshot_id
    );
    Evidence {
        evidence_id: structured_digest("evidence", payload.as_bytes()),
        workspace_snapshot: snapshot.snapshot_id.clone(),
        path: artifact.path.clone(),
        content_hash: artifact.content_hash.clone(),
        start_byte: start as u64,
        end_byte: end as u64,
        excerpt: bytes[excerpt_start..excerpt_end].to_vec(),
        match_start_in_excerpt: (start - excerpt_start) as u64,
        match_end_in_excerpt: (end - excerpt_start) as u64,
        decoding: if std::str::from_utf8(bytes).is_ok() {
            "utf8"
        } else {
            "unsupported"
        },
        extraction_method,
    }
}

fn find_bounded(
    haystack: &[u8],
    needle: &[u8],
    ascii_case_insensitive: bool,
    limit: usize,
) -> Vec<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return Vec::new();
    }
    (0..=haystack.len() - needle.len())
        .filter(|start| {
            let candidate = &haystack[*start..*start + needle.len()];
            if ascii_case_insensitive {
                candidate.eq_ignore_ascii_case(needle)
            } else {
                candidate == needle
            }
        })
        .take(limit)
        .collect()
}

fn lexical_document(bytes: &[u8]) -> Result<String, RetrievalError> {
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(RetrievalError::new(RetrievalErrorCode::ResourceLimit));
    }
    let mut terms = lexical_terms(bytes);
    terms.sort();
    terms.dedup();
    Ok(terms.join(" "))
}

fn lexical_terms(bytes: &[u8]) -> Vec<String> {
    bytes
        .split(|byte| !(byte.is_ascii_alphanumeric() || *byte == b'_'))
        .filter(|term| !term.is_empty() && term.len() <= 64)
        .map(|term| {
            term.iter()
                .map(u8::to_ascii_lowercase)
                .map(char::from)
                .collect()
        })
        .collect()
}

fn structured_digest(kind: &str, payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"impresari-context");
    hasher.update([0]);
    hasher.update(kind.as_bytes());
    hasher.update([0]);
    hasher.update(b"1.0.0");
    hasher.update([0]);
    hasher.update(payload);
    let mut label = String::from("sha256:");
    for byte in hasher.finalize() {
        use fmt::Write as _;
        write!(label, "{byte:02x}").expect("string write");
    }
    label
}

fn map_workspace(error: WorkspaceError) -> RetrievalError {
    RetrievalError::source(RetrievalErrorCode::EvidenceUnavailable, error)
}
fn map_cache(error: CacheError) -> RetrievalError {
    RetrievalError::source(RetrievalErrorCode::CacheFailure, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_workspace::DiscoveryPolicy;
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);
    struct TestRoot(PathBuf);
    impl TestRoot {
        fn new(label: &str) -> Self {
            let n = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "impresari-retrieval-{label}-{}-{n}",
                std::process::id()
            ));
            fs::create_dir(&path).expect("create root");
            Self(path)
        }
    }
    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn setup() -> (
        TestRoot,
        TestRoot,
        AuthorizedWorkspace,
        WorkspaceSnapshot,
        WorkspaceCache,
    ) {
        let source = TestRoot::new("source");
        let cache_root = TestRoot::new("cache");
        fs::write(source.0.join("sample.txt"), b"Alpha alpha beta\n").expect("source");
        let workspace = AuthorizedWorkspace::open(&source.0).expect("workspace");
        let policy = DiscoveryPolicy::new(100, 1024 * 1024, 1024, 8).expect("policy");
        let snapshot = workspace.snapshot(policy).expect("snapshot");
        let cache = WorkspaceCache::open(&cache_root.0, workspace.identity()).expect("cache");
        (source, cache_root, workspace, snapshot, cache)
    }

    #[test]
    fn literal_search_returns_exact_overlapping_spans_under_limits() {
        let (_source, _cache_root, workspace, snapshot, _cache) = setup();
        let budget = SearchBudget::new(10, 10, 32, Duration::from_secs(1)).expect("budget");
        let result = search_literal(&workspace, &snapshot, b"alpha", budget).expect("search");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(
            (result.matches[0].start_byte, result.matches[0].end_byte),
            (6, 11)
        );
        assert!(!result.truncated);
    }

    #[test]
    fn lexical_candidates_are_case_normalized_then_source_verified() {
        let (_source, _cache_root, workspace, snapshot, mut cache) = setup();
        build_lexical_generation(&workspace, &snapshot, &mut cache).expect("index");
        let budget = SearchBudget::new(10, 10, 32, Duration::from_secs(1)).expect("budget");
        let result =
            search_lexical(&workspace, &snapshot, &cache, "ALPHA beta", budget).expect("search");
        assert_eq!(
            result
                .matches
                .iter()
                .map(|item| item.start_byte)
                .collect::<Vec<_>>(),
            vec![0, 6, 12]
        );
    }

    #[test]
    fn match_limit_is_explicit_and_deterministic() {
        let (_source, _cache_root, workspace, snapshot, _cache) = setup();
        let budget = SearchBudget::new(10, 1, 32, Duration::from_secs(1)).expect("budget");
        let result = search_literal(&workspace, &snapshot, b"a", budget).expect("search");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.truncation_reasons, vec!["match_limit"]);
    }

    #[test]
    fn expansion_rechecks_source_and_emits_byte_safe_wire_evidence() {
        let (_source, _cache_root, workspace, snapshot, _cache) = setup();
        let budget = SearchBudget::new(10, 10, 5, Duration::from_secs(1)).expect("budget");
        let original = search_literal(&workspace, &snapshot, b"beta", budget)
            .expect("search")
            .matches
            .remove(0);
        let expanded = expand_evidence(&workspace, &snapshot, &original, 6, 1, 11).expect("expand");
        assert_eq!(expanded.excerpt, b"alpha beta\n");
        assert_eq!(
            (
                expanded.match_start_in_excerpt,
                expanded.match_end_in_excerpt
            ),
            (6, 10)
        );

        let wire = evidence_record(&expanded);
        assert_eq!(wire.schema_name, "evidence");
        assert_eq!(wire.span.start_byte, "12");
        assert_eq!(wire.span.end_byte, "16");
        assert_eq!(wire.excerpt.bytes_base64url, "YWxwaGEgYmV0YQo");
    }

    #[test]
    fn expansion_rejects_changed_source_and_an_undersized_ceiling() {
        let (source, _cache_root, workspace, snapshot, _cache) = setup();
        let budget = SearchBudget::new(10, 10, 5, Duration::from_secs(1)).expect("budget");
        let original = search_literal(&workspace, &snapshot, b"beta", budget)
            .expect("search")
            .matches
            .remove(0);
        assert_eq!(
            expand_evidence(&workspace, &snapshot, &original, 0, 0, 3)
                .expect_err("ceiling")
                .code(),
            RetrievalErrorCode::ResourceLimit
        );
        fs::write(source.0.join("sample.txt"), b"changed\n").expect("mutate fixture");
        assert_eq!(
            expand_evidence(&workspace, &snapshot, &original, 1, 1, 8)
                .expect_err("stale")
                .code(),
            RetrievalErrorCode::StaleState
        );
    }
}
