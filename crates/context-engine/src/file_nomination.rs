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
///
/// 1.1 adds the `task_module_path` reason and ranks inferred matches by how
/// rare each task identifier is in the repository.
pub const FILE_NOMINATION_SCHEMA_VERSION: &str = "1.1";

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
    /// The task named a dotted module that denotes exactly one tracked file.
    ///
    /// Task text names modules the way code imports them, so `ascii.rst` is
    /// `astropy/io/ascii/rst.py`. The name is the author's rather than inferred
    /// from a mention, so it ranks with the paths the task writes out.
    TaskModulePath,
    /// The file declares an identifier the task named.
    ///
    /// A declaration identifies a file; a mention does not. `Header` occurs in
    /// hundreds of astropy files and is declared in one.
    DeclarationMatch,
    /// The file contains identifiers the task named.
    IdentifierMatch,
}

impl NominationRank {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExactTaskPath => "exact_task_path",
            Self::TaskModulePath => "task_module_path",
            Self::DeclarationMatch => "task_identifier_declared",
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
    /// Identifiers admitted from the task, in the order they were seen.
    ///
    /// Disclosed because admission is no longer a pure function of a token's
    /// shape: a name can be admitted because the repository declares it
    /// (IC-DAN-131), and a later stage must be able to use the same answer
    /// rather than re-deriving a different one.
    pub admitted_identifiers: Vec<String>,
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
/// a portable path to the distinct task identifiers that file contains, and
/// `declaration_matches` to the ones it declares; the caller supplies both from
/// whatever index it already holds, so nomination stays a pure ranking
/// decision.
#[must_use]
pub fn nominate_files(
    task_paths: &[String],
    task_identifiers: &[String],
    tracked_paths: &BTreeSet<String>,
    declaration_matches: &BTreeMap<String, BTreeSet<String>>,
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

    // A dotted name the snapshot holds no path for is usually a module, written
    // the way code imports it. `astropy-14182` writes `format="ascii.rst"`, and
    // the file it denotes was never nominated: no identifier the task named
    // pointed at it.
    for path in task_paths {
        if files.len() >= MAX_NOMINATED_FILES {
            break;
        }
        if let Some(module) = resolve_module_path(path, tracked_paths)
            && seen.insert(module)
        {
            files.push(NominatedFile {
                display_path: module.to_owned(),
                reason_code: NominationRank::TaskModulePath.reason_code().to_owned(),
                matched_identifiers: identifier_matches
                    .get(module)
                    .map_or(0, |matched| count_admitted(matched, &admitted_identifiers)),
            });
        }
    }

    // Then files by how strongly they answer the task, counting a declaration
    // as worth several mentions rather than as an overriding tier. A file that
    // declares one task identifier is not automatically a better answer than
    // one that mentions four: ranked as a tier, a task about `Table` anchored
    // on `io/fits/column.py`, which declares `Column`, over `table/table.py`,
    // which declares `Table` and mentions most of the rest.
    let weights = rarity_weights(
        &admitted_identifiers,
        declaration_matches,
        identifier_matches,
        tracked_paths.len(),
    );
    let ranked = rank_by_evidence(
        declaration_matches,
        identifier_matches,
        &admitted_identifiers,
        &weights,
        &seen,
    );

    let considered_files =
        u64::try_from(files.len().saturating_add(ranked.len())).unwrap_or(u64::MAX);

    for candidate in ranked {
        if files.len() >= MAX_NOMINATED_FILES {
            break;
        }
        if seen.insert(candidate.path) {
            files.push(NominatedFile {
                display_path: candidate.path.to_owned(),
                reason_code: candidate.rank.reason_code().to_owned(),
                matched_identifiers: candidate.declared.saturating_add(candidate.mentioned),
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
        admitted_identifiers: task_identifiers.to_vec(),
        unknowns,
    }
}

/// How much one declaration outweighs one mention.
///
/// `Header` occurs in hundreds of astropy files and is declared in one, so a
/// declaration has to count for a great deal more. It must not count for
/// everything: ranked as an absolute tier, a file declaring a single task
/// identifier displaced one declaring the central type *and* mentioning most of
/// the rest, and five tasks lost their reference file that way.
const DECLARATION_WEIGHT: u64 = 3;

/// One ranked candidate and the ground that ranked it.
struct RankedCandidate<'a> {
    path: &'a str,
    rank: NominationRank,
    declared: u64,
    mentioned: u64,
    score: u64,
}

/// How much one task identifier tells nomination about a file.
///
/// The weight is the number of times the repository halves before it reaches
/// the files holding the name, plus one. Every name used to count the same, so
/// a file mentioning four common names outranked the one file declaring the
/// rare name a task is about: on `astropy-13398` the file declaring `ITRS`,
/// held by one file, ranked eighteenth behind files mentioning
/// `frame_transform_graph` and `matrix_utilities`, held by thirty and twenty.
/// A declaration still counts three mentions of the same name. The logarithm
/// is an integer, so the ranking is identical on every platform.
fn rarity_weights<'a>(
    admitted: &BTreeSet<&'a str>,
    declaration_matches: &BTreeMap<String, BTreeSet<String>>,
    identifier_matches: &BTreeMap<String, BTreeSet<String>>,
    repository_files: usize,
) -> BTreeMap<&'a str, u64> {
    let holders = |matches: &BTreeMap<String, BTreeSet<String>>, name: &str| {
        matches.values().filter(|held| held.contains(name)).count()
    };
    let files = u64::try_from(repository_files).unwrap_or(u64::MAX).max(1);
    admitted
        .iter()
        .map(|&name| {
            let holding = holders(identifier_matches, name).max(holders(declaration_matches, name));
            let holding = u64::try_from(holding).unwrap_or(u64::MAX).max(1);
            (name, u64::from((files / holding).max(1).ilog2()) + 1)
        })
        .collect()
}

/// The summed weight of the task identifiers one file holds.
fn weigh(matched: Option<&BTreeSet<String>>, weights: &BTreeMap<&str, u64>) -> u64 {
    matched.map_or(0, |matched| {
        matched
            .iter()
            .filter_map(|name| weights.get(name.as_str()))
            .fold(0, |total, weight| total.saturating_add(*weight))
    })
}

/// The one tracked file a dotted module name denotes, if exactly one does.
///
/// The name is read as a path suffix with the file extension removed, so
/// `ascii.rst` denotes `astropy/io/ascii/rst.py`. A name that is also a
/// directory denotes a package, whose files are many, and a name several paths
/// end in denotes none of them. The directory rule is what keeps `io.registry`
/// from resolving to `docs/io/registry.rst` while `astropy/io/registry` exists.
fn resolve_module_path<'a>(token: &str, tracked_paths: &'a BTreeSet<String>) -> Option<&'a str> {
    if !token.contains('.') || token.contains('/') || tracked_paths.contains(token) {
        return None;
    }
    let suffix = token.replace('.', "/");
    let directory = format!("{suffix}/");
    let nested_directory = format!("/{suffix}/");
    let nested_file = format!("/{suffix}");
    let mut found = None;
    for path in tracked_paths {
        if path.starts_with(&directory) || path.contains(&nested_directory) {
            return None;
        }
        let stem = match path.rsplit_once('.') {
            Some((stem, extension)) if !extension.contains('/') => stem,
            _ => path.as_str(),
        };
        if stem == suffix || stem.ends_with(&nested_file) {
            if found.is_some() {
                return None;
            }
            found = Some(path.as_str());
        }
    }
    found
}

/// Files carrying at least one admitted identifier, best first.
///
/// Ties break by path, so a snapshot yields an identical nomination.
fn rank_by_evidence<'a>(
    declaration_matches: &'a BTreeMap<String, BTreeSet<String>>,
    identifier_matches: &'a BTreeMap<String, BTreeSet<String>>,
    admitted: &BTreeSet<&str>,
    weights: &BTreeMap<&str, u64>,
    seen: &BTreeSet<&str>,
) -> Vec<RankedCandidate<'a>> {
    let mut ranked: Vec<RankedCandidate<'a>> = declaration_matches
        .keys()
        .chain(identifier_matches.keys())
        .collect::<BTreeSet<&String>>()
        .into_iter()
        .filter(|path| !seen.contains(path.as_str()))
        .filter_map(|path| {
            let declared = declaration_matches
                .get(path)
                .map_or(0, |matched| count_admitted(matched, admitted));
            let mentioned = identifier_matches
                .get(path)
                .map_or(0, |matched| count_admitted(matched, admitted));
            if declared == 0 && mentioned == 0 {
                return None;
            }
            Some(RankedCandidate {
                path: path.as_str(),
                rank: if declared > 0 {
                    NominationRank::DeclarationMatch
                } else {
                    NominationRank::IdentifierMatch
                },
                declared,
                mentioned,
                score: weigh(declaration_matches.get(path), weights)
                    .saturating_mul(DECLARATION_WEIGHT)
                    .saturating_add(weigh(identifier_matches.get(path), weights)),
            })
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.path.cmp(right.path))
    });
    ranked
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

    /// No declarations recorded, so these cases exercise mention ranking.
    fn none() -> BTreeMap<String, BTreeSet<String>> {
        BTreeMap::new()
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
    fn the_nomination_constants_are_pinned() {
        // A declaration weighted at three was chosen by measurement: ranked as
        // an absolute tier it gained six reference files and lost five, and
        // weighted it gained six and lost three. Sixteen nominated files is the
        // measured knee. Neither should move without that showing up here.
        assert_eq!(MAX_NOMINATED_FILES, 16);
        assert_eq!(DECLARATION_WEIGHT, 3);
    }

    #[test]
    fn a_file_declaring_a_task_identifier_outranks_one_that_mentions_it() {
        // `Header` occurs in hundreds of astropy files and is declared in one.
        // Counting a mention like a definition picks the wrong anchor, and
        // every later stage inherits that choice.
        let nomination = nominate_files(
            &[],
            &["Header".to_owned()],
            &set(&["io/fits/header.py", "io/fits/connect.py", "table/table.py"]),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &matches(&[
                ("io/fits/connect.py", &["Header"]),
                ("table/table.py", &["Header"]),
            ]),
        );
        let order: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert_eq!(order[0], "io/fits/header.py");
        assert_eq!(nomination.files[0].reason_code, "task_identifier_declared");
        assert_eq!(nomination.files[1].reason_code, "task_identifier_match");
    }

    #[test]
    fn many_mentions_can_outweigh_a_single_unrelated_declaration() {
        // A declaration is strong evidence, not overriding evidence. Ranked as
        // an absolute tier, a task about `Table` anchored on the file declaring
        // `Column` over the file declaring `Table` and mentioning the rest.
        let nomination = nominate_files(
            &[],
            &[
                "Table".to_owned(),
                "Column".to_owned(),
                "Row".to_owned(),
                "TableColumns".to_owned(),
            ],
            &set(&["table/table.py", "io/fits/column.py"]),
            &matches(&[
                ("table/table.py", &["Table"]),
                ("io/fits/column.py", &["Column"]),
            ]),
            &matches(&[(
                "table/table.py",
                &["Table", "Column", "Row", "TableColumns"],
            )]),
        );
        assert_eq!(nomination.files[0].display_path, "table/table.py");
    }

    #[test]
    fn a_declaration_still_outweighs_a_lone_mention() {
        let nomination = nominate_files(
            &[],
            &["Header".to_owned()],
            &set(&["io/fits/header.py", "io/fits/connect.py"]),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &matches(&[("io/fits/connect.py", &["Header"])]),
        );
        assert_eq!(nomination.files[0].display_path, "io/fits/header.py");
        assert_eq!(nomination.files[0].reason_code, "task_identifier_declared");
    }

    #[test]
    fn an_exact_task_path_still_outranks_a_declaration() {
        // A path the task writes out is the strongest ground there is.
        let nomination = nominate_files(
            &["table/table.py".to_owned()],
            &["Header".to_owned()],
            &set(&["io/fits/header.py", "table/table.py"]),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &matches(&[("table/table.py", &["Header"])]),
        );
        assert_eq!(nomination.files[0].display_path, "table/table.py");
        assert_eq!(nomination.files[0].reason_code, "exact_task_path");
        assert_eq!(nomination.files[1].display_path, "io/fits/header.py");
    }

    #[test]
    fn a_declaring_file_is_never_nominated_twice() {
        // A file usually both declares and mentions a name; the stronger ground
        // wins and the file appears once.
        let nomination = nominate_files(
            &[],
            &["Header".to_owned()],
            &set(&["io/fits/header.py"]),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &matches(&[("io/fits/header.py", &["Header"])]),
        );
        assert_eq!(nomination.files.len(), 1);
        assert_eq!(nomination.files[0].reason_code, "task_identifier_declared");
        // One file is one candidate, however many grounds it answers on.
        assert_eq!(nomination.considered_files, 1);
    }

    #[test]
    fn a_rare_declared_name_outranks_files_mentioning_common_ones() {
        // `astropy-13398`: the file declaring `ITRS`, held by one file, ranked
        // eighteenth behind files mentioning names held by twenty or thirty.
        let common = [
            "frame_transform_graph",
            "matrix_utilities",
            "rotation_matrix",
            "matrix_transpose",
        ];
        let mut identifiers = vec!["ITRS".to_owned()];
        identifiers.extend(common.iter().map(|name| (*name).to_owned()));
        let mentioning: Vec<String> = (0..30)
            .map(|index| format!("frames/f{index:02}.py"))
            .collect();
        let mut tracked: BTreeSet<String> = mentioning.iter().cloned().collect();
        tracked.extend((30..40).map(|index| format!("frames/f{index:02}.py")));
        tracked.insert("frames/itrs.py".to_owned());
        let mentions: BTreeMap<String, BTreeSet<String>> = mentioning
            .iter()
            .map(|path| (path.clone(), set(&common)))
            .collect();
        let nomination = nominate_files(
            &[],
            &identifiers,
            &tracked,
            &matches(&[("frames/itrs.py", &["ITRS"])]),
            &mentions,
        );
        assert_eq!(nomination.files[0].display_path, "frames/itrs.py");
        assert_eq!(nomination.files[0].reason_code, "task_identifier_declared");
    }

    #[test]
    fn a_dotted_module_name_nominates_the_file_it_denotes() {
        // `astropy-14182` writes `format="ascii.rst"`: a module, not a path.
        let nomination = nominate_files(
            &["ascii.rst".to_owned()],
            &[],
            &set(&["astropy/io/ascii/rst.py", "astropy/io/ascii/core.py"]),
            &none(),
            &none(),
        );
        assert_eq!(nomination.files.len(), 1);
        assert_eq!(nomination.files[0].display_path, "astropy/io/ascii/rst.py");
        assert_eq!(nomination.files[0].reason_code, "task_module_path");
    }

    #[test]
    fn a_dotted_name_for_a_package_or_several_files_nominates_none() {
        // `io.registry` is the package `astropy/io/registry`, not the page
        // `docs/io/registry.rst`, and `x.util` ends two paths and names neither.
        let nomination = nominate_files(
            &["io.registry".to_owned(), "x.util".to_owned()],
            &[],
            &set(&[
                "astropy/io/registry/core.py",
                "docs/io/registry.rst",
                "a/x/util.py",
                "b/x/util.py",
            ]),
            &none(),
            &none(),
        );
        assert!(nomination.files.is_empty());
        assert!(
            nomination
                .unknowns
                .contains(&"no_candidate_file_nominated".to_owned())
        );
    }

    #[test]
    fn nomination_discloses_the_identifiers_it_admitted() {
        // Admission is no longer a function of shape alone, so a later stage
        // must be able to reuse the answer rather than re-deriving a different
        // one.
        let nomination = nominate_files(
            &[],
            &["Header".to_owned(), "Card".to_owned()],
            &set(&["io/fits/header.py"]),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &BTreeMap::new(),
        );
        assert_eq!(nomination.admitted_identifiers, vec!["Header", "Card"]);
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
