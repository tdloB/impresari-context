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

/// Files admitted by package proximity beyond those the task names (IC-PPNR-130).
///
/// Fifteen of the sixteen reference files map recall misses are files the task
/// report never names, so no ranking over task signals reaches them. Measured
/// against those sixteen, following import edges out of a nominated file
/// reaches one; files in the same package reach ten.
///
/// Expanding from the top-ranked nomination admits about ten siblings on
/// average. Twelve bounds the tail — a large flat directory cannot flood a
/// scope whose fact allowance is shared — without biting in the common case.
///
/// It is a closed constant for the same reason [`MAX_NOMINATED_FILES`] is: a
/// caller able to widen reach could walk the scope toward a file it already
/// wanted, which is oracle authority arriving through a configuration field.
pub const MAX_REACH_FILES: usize = 12;

/// Reason code marking a file admitted by reach rather than by task signal.
pub const REACH_REASON_CODE: &str = "package_proximate_reach";

/// Why one file was nominated.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NominationRank {
    /// The task named this exact path and the snapshot contains it.
    ExactTaskPath,
    /// The file contains identifiers the task named.
    IdentifierMatch,
    /// The file sits in the same package as the best-ranked nomination.
    ///
    /// Ordered last deliberately: a file the task named outranks one the
    /// product inferred.
    PackageProximateReach,
}

impl NominationRank {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExactTaskPath => "exact_task_path",
            Self::IdentifierMatch => "task_identifier_match",
            Self::PackageProximateReach => REACH_REASON_CODE,
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
    /// Directly nominated candidates considered before the ceiling applied.
    ///
    /// Counts the files task signals reached. Reach-expanded files are not
    /// candidates in this sense — they are admitted after the ranking closes —
    /// so they are excluded here and disclosed by their own reason code.
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
/// `tracked_paths` is the snapshot's file inventory. `admitted_paths` is the
/// subset this product can parse, from which reach draws siblings.
/// `identifier_matches` maps a portable path to the distinct task identifiers
/// that file contains; the caller supplies it from whatever index it already
/// holds, so nomination stays a pure ranking decision.
#[must_use]
pub fn nominate_files(
    task_paths: &[String],
    task_identifiers: &[String],
    tracked_paths: &BTreeSet<String>,
    admitted_paths: &BTreeSet<String>,
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

    // Reach runs after ranking has closed and after the truncation verdict is
    // recorded, so it can neither promote a file above a direct nomination nor
    // inflate the file count into hiding that direct candidates were dropped.
    let (reach, reach_truncated) = package_proximate_reach(
        files.first(),
        admitted_paths,
        &seen,
        identifier_matches,
        &admitted_identifiers,
    );
    if !reach.is_empty() {
        unknowns.push("structural_scope_includes_reach_expanded_files".to_owned());
    }
    if reach_truncated {
        unknowns.push("reach_ceiling_reached".to_owned());
    }
    files.extend(reach);

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

/// Admit the anchor's package siblings, best first, bounded.
///
/// The anchor is the single highest-ranked direct nomination. Two anchors were
/// measured and reach one further reference file for three more admitted files;
/// three anchors reach none for ten more. The scoped fact allowance is divided
/// across the scope, so those extra files thin every file already in it.
fn package_proximate_reach(
    anchor: Option<&NominatedFile>,
    admitted_paths: &BTreeSet<String>,
    nominated: &BTreeSet<&str>,
    identifier_matches: &BTreeMap<String, BTreeSet<String>>,
    admitted_identifiers: &BTreeSet<&str>,
) -> (Vec<NominatedFile>, bool) {
    let Some(anchor) = anchor else {
        // No direct nomination is no anchor. Reach infers from a candidate; it
        // does not invent one.
        return (Vec::new(), false);
    };
    let package = package_of(&anchor.display_path);

    let mut siblings: Vec<(u64, &str)> = admitted_paths
        .iter()
        .map(String::as_str)
        .filter(|path| package_of(path) == package && !nominated.contains(path))
        .map(|path| {
            let matched = identifier_matches
                .get(path)
                .map_or(0, |held| count_admitted(held, admitted_identifiers));
            (matched, path)
        })
        .collect();
    // A sibling carrying task identifiers was a direct candidate that fell
    // below the nomination ceiling. Re-admit those first; ties break by path so
    // truncation stays deterministic.
    siblings.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(right.1)));

    let truncated = siblings.len() > MAX_REACH_FILES;
    let files = siblings
        .into_iter()
        .take(MAX_REACH_FILES)
        .map(|(matched, path)| NominatedFile {
            display_path: path.to_owned(),
            reason_code: NominationRank::PackageProximateReach
                .reason_code()
                .to_owned(),
            matched_identifiers: matched,
        })
        .collect();
    (files, truncated)
}

/// A package is the immediate parent directory of a portable path.
///
/// Deliberately crude. A language-aware definition needs a resolver per
/// admitted language, and the measurement says the crude rule already captures
/// the effect.
fn package_of(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(parent, _)| parent)
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

    /// No admitted file inventory, so no reach: these cases test ranking.
    fn none() -> BTreeSet<String> {
        BTreeSet::new()
    }

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
            &none(),
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
            &none(),
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
            &none(),
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
            &none(),
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
        let nomination = nominate_files(&[], &["alpha".to_owned()], &tracked, &none(), &matched);
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
            &none(),
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
    fn reach_admits_package_siblings_of_the_best_nomination() {
        let nomination = nominate_files(
            &["astropy/io/fits/card.py".to_owned()],
            &["Card".to_owned()],
            &set(&["astropy/io/fits/card.py"]),
            &set(&[
                "astropy/io/fits/card.py",
                "astropy/io/fits/header.py",
                "astropy/io/ascii/html.py",
            ]),
            &matches(&[("astropy/io/fits/card.py", &["Card"])]),
        );
        let paths: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        // The sibling is reached; the file in a different package is not.
        assert_eq!(
            paths,
            vec!["astropy/io/fits/card.py", "astropy/io/fits/header.py"]
        );
        assert_eq!(nomination.files[1].reason_code, "package_proximate_reach");
        assert!(
            nomination
                .unknowns
                .contains(&"structural_scope_includes_reach_expanded_files".to_owned())
        );
    }

    #[test]
    fn reach_never_precedes_a_directly_nominated_file() {
        // Extraction spends a shared allowance in order. A file the task named
        // must never lose budget to one the product inferred.
        let mut admitted = set(&["pkg/a.py", "pkg/b.py"]);
        admitted.extend((0..20).map(|index| format!("pkg/reach{index:02}.py")));
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned()],
            &admitted,
            &admitted,
            &matches(&[("pkg/a.py", &["alpha"]), ("pkg/b.py", &["alpha"])]),
        );
        let direct = nomination
            .files
            .iter()
            .filter(|file| file.reason_code != "package_proximate_reach")
            .count();
        assert_eq!(direct, 2);
        assert!(
            nomination.files[..direct]
                .iter()
                .all(|file| file.reason_code != "package_proximate_reach")
        );
        assert!(
            nomination.files[direct..]
                .iter()
                .all(|file| file.reason_code == "package_proximate_reach")
        );
    }

    #[test]
    fn reach_is_bounded_and_says_when_it_truncated() {
        let mut admitted = set(&["pkg/anchor.py"]);
        admitted.extend((0..30).map(|index| format!("pkg/sib{index:02}.py")));
        let nomination = nominate_files(
            &["pkg/anchor.py".to_owned()],
            &[],
            &set(&["pkg/anchor.py"]),
            &admitted,
            &BTreeMap::new(),
        );
        assert_eq!(nomination.files.len(), 1 + MAX_REACH_FILES);
        assert!(
            nomination
                .unknowns
                .contains(&"reach_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn a_sibling_below_the_nomination_ceiling_is_re_admitted_before_an_unmatched_one() {
        // Twenty matched files against a ceiling of sixteen the anchor also
        // occupies, so `m15` onward are dropped from direct nomination.
        let mut admitted = set(&["pkg/anchor.py", "pkg/aaa_unmatched.py"]);
        admitted.extend((0..20).map(|index| format!("pkg/m{index:02}.py")));
        let mut matched: BTreeMap<String, BTreeSet<String>> = (0..20)
            .map(|index| (format!("pkg/m{index:02}.py"), set(&["alpha"])))
            .collect();
        matched.insert("pkg/anchor.py".to_owned(), set(&["alpha", "beta"]));

        let nomination = nominate_files(
            &[],
            &["alpha".to_owned(), "beta".to_owned()],
            &admitted,
            &admitted,
            &matched,
        );
        let reach: Vec<&str> = nomination
            .files
            .iter()
            .filter(|file| file.reason_code == "package_proximate_reach")
            .map(|file| file.display_path.as_str())
            .collect();
        // Alphabetically last, but it carries a task identifier the ceiling cut,
        // so reach re-admits it ahead of a sibling that carries none.
        assert_eq!(reach[0], "pkg/m15.py");
        assert!(reach.contains(&"pkg/aaa_unmatched.py"));
    }

    #[test]
    fn no_direct_nomination_means_no_reach() {
        // Reach infers from a candidate. With none, it invents nothing.
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned()],
            &set(&["pkg/a.py"]),
            &set(&["pkg/a.py", "pkg/b.py"]),
            &BTreeMap::new(),
        );
        assert!(nomination.files.is_empty());
        assert!(
            !nomination
                .unknowns
                .contains(&"structural_scope_includes_reach_expanded_files".to_owned())
        );
    }

    #[test]
    fn reach_cannot_hide_that_direct_candidates_were_dropped() {
        // Reach grows the file count. The truncation verdict is taken before it
        // runs, so a wider result still discloses the dropped candidates.
        let paths: Vec<String> = (0..40)
            .map(|index| format!("pkg/file{index:02}.py"))
            .collect();
        let tracked: BTreeSet<String> = paths.iter().cloned().collect();
        let matched: BTreeMap<String, BTreeSet<String>> = paths
            .iter()
            .map(|path| (path.clone(), set(&["alpha"])))
            .collect();
        let nomination = nominate_files(&[], &["alpha".to_owned()], &tracked, &tracked, &matched);
        assert_eq!(nomination.considered_files, 40);
        assert!(nomination.files.len() > MAX_NOMINATED_FILES);
        assert!(
            nomination
                .unknowns
                .contains(&"nomination_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn reach_admits_only_files_the_product_can_parse() {
        // A sibling the snapshot tracks but the index never admitted is a file
        // structural extraction would skip; spending a scope slot on it wastes
        // the shared allowance.
        let nomination = nominate_files(
            &["pkg/anchor.py".to_owned()],
            &[],
            &set(&["pkg/anchor.py", "pkg/notes.rst", "pkg/sibling.py"]),
            &set(&["pkg/anchor.py", "pkg/sibling.py"]),
            &BTreeMap::new(),
        );
        let paths: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert_eq!(paths, vec!["pkg/anchor.py", "pkg/sibling.py"]);
    }

    #[test]
    fn nomination_is_deterministic_for_identical_inputs() {
        let tracked = set(&["a.py", "b.py"]);
        let matched = matches(&[("a.py", &["alpha"]), ("b.py", &["alpha"])]);
        let first = nominate_files(&[], &["alpha".to_owned()], &tracked, &none(), &matched);
        let second = nominate_files(&[], &["alpha".to_owned()], &tracked, &none(), &matched);
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
