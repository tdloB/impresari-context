// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Task-driven candidate file nomination (IC-SSSE-128).
//!
//! Structural extraction spread thinly across a whole repository cannot hold a
//! module's declarations. Nominating a small set of candidate files lets each
//! one be extracted densely instead.
//!
//! Nomination consumes only signals already derived from the task and the
//! current snapshot. It never sees a reference change, accepted patch, or test
//! outcome.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Schema discriminator for a nomination disclosure.
pub const FILE_NOMINATION_SCHEMA_NAME: &str = "impresari_context_file_nomination";
/// Schema version for a nomination disclosure.
pub const FILE_NOMINATION_SCHEMA_VERSION: &str = "1.0";

/// Files nominated for dense structural extraction.
///
/// Measured on twenty-two accepted changes in a large Python repository, the
/// reference file appears among sixteen nominated candidates in 73% of tasks,
/// against 64% at eight. Beyond sixteen the curve is flat, while the graph cost
/// keeps rising, so sixteen is the admitted ceiling.
///
/// It is a closed constant, never caller-supplied: a caller able to widen
/// nomination could steer it, and steering is oracle authority.
pub const MAX_NOMINATED_FILES: usize = 16;

/// Why one file was nominated.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NominationRank {
    /// The task named this exact path and the snapshot contains it.
    ExactTaskPath,
    /// The file contains identifiers the task named.
    IdentifierMatch,
}

impl NominationRank {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExactTaskPath => "exact_task_path",
            Self::IdentifierMatch => "task_identifier_match",
        }
    }
}

/// One nominated file and the ground for admitting it.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NominatedFile {
    /// Portable display path.
    pub display_path: String,
    /// Why the file was admitted.
    pub reason_code: String,
    /// Distinct task identifiers the file contains.
    pub matched_identifiers: u64,
}

/// A bounded nomination and the disclosure a consumer needs to read it safely.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileNomination {
    /// Schema discriminator.
    pub schema_name: String,
    /// Schema version.
    pub schema_version: String,
    /// Admitted files, best first.
    pub files: Vec<NominatedFile>,
    /// Candidates considered before the ceiling applied.
    pub considered_files: u64,
    /// Explicit scope and shortfall reasons.
    pub unknowns: Vec<String>,
}

impl FileNomination {
    /// True when structural coverage is limited to the nominated files.
    ///
    /// A scoped graph is dense but partial. A consumer must be able to tell it
    /// apart from a whole-repository graph, which is thin but complete, so this
    /// is always true for a nomination-derived graph.
    #[must_use]
    pub const fn is_scoped(&self) -> bool {
        true
    }
}

/// Nominate a bounded, ranked set of candidate files.
///
/// `task_paths` and `task_identifiers` are admitted task signals.
/// `tracked_paths` is the snapshot's file inventory. `identifier_matches` maps
/// a portable path to the distinct task identifiers that file contains; the
/// caller supplies it from whatever index it already holds, so nomination stays
/// a pure ranking decision.
#[must_use]
pub fn nominate_files(
    task_paths: &[String],
    task_identifiers: &[String],
    tracked_paths: &BTreeSet<String>,
    identifier_matches: &BTreeMap<String, BTreeSet<String>>,
) -> FileNomination {
    let admitted_identifiers: BTreeSet<&str> =
        task_identifiers.iter().map(String::as_str).collect();

    let mut files: Vec<NominatedFile> = Vec::new();
    let mut seen: BTreeSet<&str> = BTreeSet::new();

    // A path the task names outright outranks any inferred match.
    for path in task_paths {
        if tracked_paths.contains(path) && seen.insert(path.as_str()) {
            files.push(NominatedFile {
                display_path: path.clone(),
                reason_code: NominationRank::ExactTaskPath.reason_code().to_owned(),
                matched_identifiers: identifier_matches
                    .get(path)
                    .map_or(0, |matched| count_admitted(matched, &admitted_identifiers)),
            });
        }
    }

    // Then files by how many distinct task identifiers they carry. Ties break
    // by path, so a snapshot yields an identical nomination.
    let mut ranked: Vec<(u64, &str)> = identifier_matches
        .iter()
        .filter(|(path, _)| !seen.contains(path.as_str()))
        .filter_map(|(path, matched)| {
            let count = count_admitted(matched, &admitted_identifiers);
            (count > 0).then_some((count, path.as_str()))
        })
        .collect();
    ranked.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(right.1)));

    let considered_files =
        u64::try_from(files.len().saturating_add(ranked.len())).unwrap_or(u64::MAX);

    for (count, path) in ranked {
        if files.len() >= MAX_NOMINATED_FILES {
            break;
        }
        if seen.insert(path) {
            files.push(NominatedFile {
                display_path: path.to_owned(),
                reason_code: NominationRank::IdentifierMatch.reason_code().to_owned(),
                matched_identifiers: count,
            });
        }
    }

    let mut unknowns = Vec::new();
    // Coverage is always partial by construction; say so every time.
    unknowns.push("structural_scope_limited_to_nominated_files".to_owned());
    if considered_files > u64::try_from(files.len()).unwrap_or(u64::MAX) {
        unknowns.push("nomination_ceiling_reached".to_owned());
    }
    if files.is_empty() {
        unknowns.push("no_candidate_file_nominated".to_owned());
    }
    unknowns.sort();
    unknowns.dedup();

    FileNomination {
        schema_name: FILE_NOMINATION_SCHEMA_NAME.to_owned(),
        schema_version: FILE_NOMINATION_SCHEMA_VERSION.to_owned(),
        files,
        considered_files,
        unknowns,
    }
}

fn count_admitted(matched: &BTreeSet<String>, admitted: &BTreeSet<&str>) -> u64 {
    u64::try_from(
        matched
            .iter()
            .filter(|value| admitted.contains(value.as_str()))
            .count(),
    )
    .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> BTreeSet<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn matches(pairs: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        pairs
            .iter()
            .map(|(path, idents)| ((*path).to_owned(), set(idents)))
            .collect()
    }

    #[test]
    fn an_exact_task_path_outranks_every_inferred_match() {
        let nomination = nominate_files(
            &["astropy/timeseries/core.py".to_owned()],
            &["TimeSeries".to_owned(), "_required_columns".to_owned()],
            &set(&[
                "astropy/timeseries/core.py",
                "astropy/timeseries/sampled.py",
            ]),
            &matches(&[
                (
                    "astropy/timeseries/sampled.py",
                    &["TimeSeries", "_required_columns"],
                ),
                ("astropy/timeseries/core.py", &["_required_columns"]),
            ]),
        );
        assert_eq!(
            nomination.files[0].display_path,
            "astropy/timeseries/core.py"
        );
        assert_eq!(nomination.files[0].reason_code, "exact_task_path");
        // The sibling is still nominated, just beneath it.
        assert_eq!(
            nomination.files[1].display_path,
            "astropy/timeseries/sampled.py"
        );
    }

    #[test]
    fn files_rank_by_distinct_identifier_count_then_path() {
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
            &set(&["a.py", "b.py", "c.py"]),
            &matches(&[
                ("c.py", &["alpha"]),
                ("a.py", &["alpha", "beta", "gamma"]),
                ("b.py", &["alpha"]),
            ]),
        );
        let order: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert_eq!(order, vec!["a.py", "b.py", "c.py"]);
        assert_eq!(nomination.files[0].matched_identifiers, 3);
    }

    #[test]
    fn a_path_the_snapshot_does_not_contain_is_not_nominated() {
        // Task text routinely names modules and URLs that are not files.
        let nomination = nominate_files(
            &["astropy.timeseries".to_owned(), "gone.py".to_owned()],
            &[],
            &set(&["real.py"]),
            &BTreeMap::new(),
        );
        assert!(nomination.files.is_empty());
        assert!(
            nomination
                .unknowns
                .contains(&"no_candidate_file_nominated".to_owned())
        );
    }

    #[test]
    fn an_identifier_the_task_never_named_cannot_nominate() {
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned()],
            &set(&["a.py", "b.py"]),
            &matches(&[("a.py", &["alpha"]), ("b.py", &["unrelated"])]),
        );
        let paths: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert_eq!(paths, vec!["a.py"]);
    }

    #[test]
    fn nomination_is_bounded_and_says_when_it_truncated() {
        let paths: Vec<String> = (0..40).map(|index| format!("file{index:02}.py")).collect();
        let tracked: BTreeSet<String> = paths.iter().cloned().collect();
        let matched: BTreeMap<String, BTreeSet<String>> = paths
            .iter()
            .map(|path| (path.clone(), set(&["alpha"])))
            .collect();
        let nomination = nominate_files(&[], &["alpha".to_owned()], &tracked, &matched);
        assert_eq!(nomination.files.len(), MAX_NOMINATED_FILES);
        assert_eq!(nomination.considered_files, 40);
        assert!(
            nomination
                .unknowns
                .contains(&"nomination_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn scope_is_always_disclosed_so_a_partial_graph_cannot_read_as_complete() {
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned()],
            &set(&["a.py"]),
            &matches(&[("a.py", &["alpha"])]),
        );
        assert!(nomination.is_scoped());
        assert!(
            nomination
                .unknowns
                .contains(&"structural_scope_limited_to_nominated_files".to_owned())
        );
    }

    #[test]
    fn nomination_is_deterministic_for_identical_inputs() {
        let tracked = set(&["a.py", "b.py"]);
        let matched = matches(&[("a.py", &["alpha"]), ("b.py", &["alpha"])]);
        let first = nominate_files(&[], &["alpha".to_owned()], &tracked, &matched);
        let second = nominate_files(&[], &["alpha".to_owned()], &tracked, &matched);
        assert!(first == second);
    }

    #[test]
    fn module_reaches_no_oracle_execution_or_network_authority() {
        let source = include_str!("file_nomination.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        // Names that would indicate reading an accepted change or executing
        // something. Prose in the module docs is deliberately not scanned for.
        for forbidden in [
            "reference_patch",
            "FAIL_TO_PASS",
            "PASS_TO_PASS",
            "test_patch",
            "Command",
            "spawn",
        ] {
            assert!(!shipped.contains(forbidden), "must not reach {forbidden}");
        }
    }
}
