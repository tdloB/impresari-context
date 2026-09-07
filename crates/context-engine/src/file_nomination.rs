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

/// Largest number of package-proximate files admitted beyond the nominated set.
///
/// Measured on twenty-two astropy tasks, every reference file that nomination
/// missed sat in a package that already had a nominated file — five of five. A
/// change is rarely confined to the file whose name the task happens to use.
///
/// This is a guard against a pathological directory, not a selector. Reach
/// admits a package in path order, and path order is not relevance order, so a
/// ceiling that truncates a package chooses arbitrarily; it is set high enough
/// to take an ordinary package whole.
pub const MAX_REACH_FILES: usize = 32;

/// Why one file was nominated.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NominationRank {
    /// The task named this exact path and the snapshot contains it.
    ExactTaskPath,
    /// The file declares an identifier the task named.
    ///
    /// A declaration identifies a file; a mention does not. `Header` occurs in
    /// hundreds of astropy files and is declared in one.
    DeclarationMatch,
    /// The file contains identifiers the task named.
    IdentifierMatch,
    /// The file shares a package with the best-ranked nominated file.
    ///
    /// Ranked strictly last: proximity is the weakest ground here, and a file
    /// the task actually names must never be displaced by a neighbour.
    PackageReach,
}

impl NominationRank {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ExactTaskPath => "exact_task_path",
            Self::DeclarationMatch => "task_identifier_declared",
            Self::IdentifierMatch => "task_identifier_match",
            Self::PackageReach => "package_proximate_reach",
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
/// `tracked_paths` is the snapshot's file inventory. `admitted_paths` is the
/// subset this product can parse, from which reach draws package neighbours.
/// `identifier_matches` maps a portable path to the distinct task identifiers
/// that file contains, and `declaration_matches` to the ones it declares; the
/// caller supplies both from whatever index it already holds, so nomination
/// stays a pure ranking decision.
#[must_use]
pub fn nominate_files(
    task_paths: &[String],
    task_identifiers: &[String],
    tracked_paths: &BTreeSet<String>,
    admitted_paths: &BTreeSet<String>,
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

    // Then files by how strongly they answer the task, counting a declaration
    // as worth several mentions rather than as an overriding tier. A file that
    // declares one task identifier is not automatically a better answer than
    // one that mentions four: ranked as a tier, a task about `Table` anchored
    // on `io/fits/column.py`, which declares `Column`, over `table/table.py`,
    // which declares `Table` and mentions most of the rest.
    let ranked = rank_by_evidence(
        declaration_matches,
        identifier_matches,
        &admitted_identifiers,
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

    // A change is rarely confined to the file the task names. Neighbours of the
    // best-ranked nominee are admitted after every directly nominated file, so
    // proximity can add coverage but can never displace evidence.
    let reached = reach_by_package(&files, admitted_paths, &mut seen);
    let reach_count = reached.len();
    files.extend(reached);

    let mut unknowns = Vec::new();
    // Coverage is always partial by construction; say so every time.
    unknowns.push("structural_scope_limited_to_nominated_files".to_owned());
    if considered_files > u64::try_from(files.len()).unwrap_or(u64::MAX) {
        unknowns.push("nomination_ceiling_reached".to_owned());
    }
    if reach_count > 0 {
        // A consumer reading a map must be able to tell a file the task pointed
        // at from a neighbour admitted on proximity alone.
        unknowns.push("nomination_includes_package_reach".to_owned());
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

/// Parseable files sharing the best-ranked nominee's package, in path order.
///
/// The anchor is one package, not every nominated package: a nomination of
/// sixteen files can span a dozen directories, and admitting all of them is not
/// proximity, it is a second repository-wide pass.
///
/// Path order is deliberate and its weakness is deliberate too. There is no
/// relevance signal among neighbours — that is what makes them neighbours rather
/// than matches — so ordering claims nothing, and the ceiling exists to bound a
/// pathological directory rather than to choose within a reasonable one.
fn reach_by_package(
    nominated: &[NominatedFile],
    admitted_paths: &BTreeSet<String>,
    seen: &mut BTreeSet<&str>,
) -> Vec<NominatedFile> {
    let Some(anchor) = nominated.first() else {
        return Vec::new();
    };
    let Some(package) = package_of(&anchor.display_path) else {
        return Vec::new();
    };
    admitted_paths
        .iter()
        .filter(|path| package_of(path) == Some(package))
        .filter(|path| !seen.contains(path.as_str()))
        .take(MAX_REACH_FILES)
        .map(|path| NominatedFile {
            display_path: path.clone(),
            reason_code: NominationRank::PackageReach.reason_code().to_owned(),
            matched_identifiers: 0,
        })
        .collect()
}

/// The immediate parent directory of a portable path.
///
/// A repository-root file has no package, and treating the root as one would
/// make every top-level file a neighbour of every other.
fn package_of(path: &str) -> Option<&str> {
    path.rfind('/').map(|cut| &path[..cut])
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
}

/// Files carrying at least one admitted identifier, best first.
///
/// Ties break by path, so a snapshot yields an identical nomination.
fn rank_by_evidence<'a>(
    declaration_matches: &'a BTreeMap<String, BTreeSet<String>>,
    identifier_matches: &'a BTreeMap<String, BTreeSet<String>>,
    admitted: &BTreeSet<&str>,
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
            })
        })
        .collect();
    ranked.sort_by(|left, right| {
        evidence_score(right)
            .cmp(&evidence_score(left))
            .then_with(|| left.path.cmp(right.path))
    });
    ranked
}

fn evidence_score(candidate: &RankedCandidate<'_>) -> u64 {
    candidate
        .declared
        .saturating_mul(DECLARATION_WEIGHT)
        .saturating_add(candidate.mentioned)
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
    /// No file is admitted for reach, so a test sees ranking alone.
    fn no_reach() -> BTreeSet<String> {
        BTreeSet::new()
    }

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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
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
        let nomination = nominate_files(
            &[],
            &["alpha".to_owned()],
            &tracked,
            &no_reach(),
            &none(),
            &matched,
        );
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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
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
            &no_reach(),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &matches(&[("io/fits/header.py", &["Header"])]),
        );
        assert_eq!(nomination.files.len(), 1);
        assert_eq!(nomination.files[0].reason_code, "task_identifier_declared");
        // One file is one candidate, however many grounds it answers on.
        assert_eq!(nomination.considered_files, 1);
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
            &no_reach(),
            &matches(&[("io/fits/header.py", &["Header"])]),
            &BTreeMap::new(),
        );
        assert_eq!(nomination.admitted_identifiers, vec!["Header", "Card"]);
    }

    #[test]
    fn nomination_is_deterministic_for_identical_inputs() {
        let tracked = set(&["a.py", "b.py"]);
        let matched = matches(&[("a.py", &["alpha"]), ("b.py", &["alpha"])]);
        let first = nominate_files(
            &[],
            &["alpha".to_owned()],
            &tracked,
            &no_reach(),
            &none(),
            &matched,
        );
        let second = nominate_files(
            &[],
            &["alpha".to_owned()],
            &tracked,
            &no_reach(),
            &none(),
            &matched,
        );
        assert!(first == second);
    }

    #[test]
    fn reach_admits_package_neighbours_of_the_best_ranked_file() {
        // Measured on twenty-two astropy tasks, every reference file nomination
        // missed sat in a package that already had a nominated file — five of
        // five. A change is rarely confined to the file the task names.
        let tracked = set(&[
            "astropy/coordinates/builtin_frames/icrs.py",
            "astropy/coordinates/builtin_frames/itrs.py",
            "astropy/coordinates/builtin_frames/__init__.py",
            "astropy/table/table.py",
        ]);
        let nomination = nominate_files(
            &[],
            &["ICRS".to_owned()],
            &tracked,
            &tracked,
            &matches(&[("astropy/coordinates/builtin_frames/icrs.py", &["ICRS"])]),
            &none(),
        );
        let paths: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert_eq!(
            paths[0], "astropy/coordinates/builtin_frames/icrs.py",
            "the file the task pointed at ranks first"
        );
        assert!(
            paths.contains(&"astropy/coordinates/builtin_frames/itrs.py"),
            "a package neighbour is reached"
        );
        assert!(
            !paths.contains(&"astropy/table/table.py"),
            "a file in another package is not a neighbour"
        );
        assert!(
            nomination
                .unknowns
                .contains(&"nomination_includes_package_reach".to_owned()),
            "a consumer must be able to tell proximity from evidence"
        );
    }

    #[test]
    fn reach_never_displaces_a_file_the_task_named() {
        // Proximity is the weakest ground admitted. A neighbour that pushed out
        // a file carrying task evidence would trade a signal for a guess.
        // The anchor is the best-ranked file's package, so the exact task path
        // sits in `pkg` and its neighbours are what reach can offer.
        let tracked = set(&["pkg/a.py", "pkg/b.py", "pkg/c.py", "pkg/z.py"]);
        let nomination = nominate_files(
            &["pkg/z.py".to_owned()],
            &["Thing".to_owned()],
            &tracked,
            &tracked,
            &matches(&[("pkg/a.py", &["Thing"])]),
            &none(),
        );
        let evidence: Vec<&str> = nomination
            .files
            .iter()
            .filter(|file| file.reason_code != "package_proximate_reach")
            .map(|file| file.display_path.as_str())
            .collect();
        assert!(
            evidence.contains(&"pkg/z.py"),
            "an exact task path survives"
        );
        assert!(evidence.contains(&"pkg/a.py"), "a declaring file survives");
        let first_reach = nomination
            .files
            .iter()
            .position(|file| file.reason_code == "package_proximate_reach");
        let last_evidence = nomination
            .files
            .iter()
            .rposition(|file| file.reason_code != "package_proximate_reach");
        let first = first_reach.expect("the fixture must actually reach neighbours");
        let last = last_evidence.expect("and must carry evidence files");
        assert!(first > last, "every reach file ranks after every nominee");
    }

    #[test]
    fn reach_draws_only_from_files_this_product_can_parse() {
        // A neighbour that cannot be parsed adds a path to the scope and no
        // structure, which is cost without coverage.
        let tracked = set(&["pkg/a.py", "pkg/b.py", "pkg/notes.md", "pkg/data.bin"]);
        let admitted = set(&["pkg/a.py", "pkg/b.py"]);
        let nomination = nominate_files(
            &[],
            &["Thing".to_owned()],
            &tracked,
            &admitted,
            &matches(&[("pkg/a.py", &["Thing"])]),
            &none(),
        );
        let paths: Vec<&str> = nomination
            .files
            .iter()
            .map(|file| file.display_path.as_str())
            .collect();
        assert!(
            paths.contains(&"pkg/b.py"),
            "a parseable neighbour is reached, so this fixture is not vacuous"
        );
        assert!(!paths.contains(&"pkg/notes.md"), "prose is not structure");
        assert!(!paths.contains(&"pkg/data.bin"), "nor is a binary");
    }

    #[test]
    fn reach_is_bounded_and_a_root_file_has_no_package() {
        let mut wide: Vec<String> = (0..MAX_REACH_FILES + 20)
            .map(|index| format!("pkg/file{index:03}.py"))
            .collect();
        wide.push("pkg/anchor.py".to_owned());
        let tracked: BTreeSet<String> = wide.into_iter().collect();
        let nomination = nominate_files(
            &[],
            &["Thing".to_owned()],
            &tracked,
            &tracked,
            &matches(&[("pkg/anchor.py", &["Thing"])]),
            &none(),
        );
        let reached = nomination
            .files
            .iter()
            .filter(|file| file.reason_code == "package_proximate_reach")
            .count();
        assert_eq!(
            reached, MAX_REACH_FILES,
            "a pathological directory is bounded"
        );

        // A repository-root file has no package; treating the root as one would
        // make every top-level file a neighbour of every other.
        let root = set(&["setup.py", "conftest.py"]);
        let rooted = nominate_files(
            &[],
            &["Thing".to_owned()],
            &root,
            &root,
            &matches(&[("setup.py", &["Thing"])]),
            &none(),
        );
        assert_eq!(
            rooted.files.len(),
            1,
            "the repository root is not a package"
        );
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
