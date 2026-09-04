// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Bounded task-identifier index (IC-TII-129).
//!
//! Nomination needs to know which files contain the identifiers a task names.
//! Answering that by searching per identifier costs thousands of repository
//! reads each. This index answers it from memory, built once during
//! preparation.
//!
//! It holds identifiers and portable paths only — never source bytes, byte
//! ranges, or extracted content — so it answers membership rather than becoming
//! a second retrieval path around exact-source provenance.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Largest number of files one index will hold.
pub const MAX_INDEXED_FILES: usize = 20_000;
/// Largest number of distinct identifiers retained for one file.
///
/// Bounds a generated or vendored file so it cannot dominate the index.
pub const MAX_IDENTIFIERS_PER_FILE: usize = 512;
/// Largest identifier length retained, matching the task-signal ceiling.
const MAX_IDENTIFIER_BYTES: usize = 256;

/// Closed failure category for one index lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierIndexErrorCode {
    /// The index belongs to a different workspace snapshot.
    SnapshotMismatch,
}

impl IdentifierIndexErrorCode {
    /// Stable source-free category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SnapshotMismatch => "snapshot_mismatch",
        }
    }
}

impl std::fmt::Display for IdentifierIndexErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::error::Error for IdentifierIndexErrorCode {}

/// Snapshot-bound map from admitted file to the identifiers it contains.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TaskIdentifierIndex {
    /// Exact snapshot identity this index answers for.
    pub workspace_snapshot: String,
    /// Portable display path to the identifiers that file contains.
    pub files: BTreeMap<String, BTreeSet<String>>,
    /// Explicit record of every bound that bit.
    pub unknowns: Vec<String>,
}

/// Accumulates an index one admitted file at a time.
pub struct TaskIdentifierIndexBuilder {
    index: TaskIdentifierIndex,
}

impl TaskIdentifierIndexBuilder {
    /// Start an index bound to one exact workspace snapshot.
    #[must_use]
    pub fn new(workspace_snapshot: &str) -> Self {
        Self {
            index: TaskIdentifierIndex {
                workspace_snapshot: workspace_snapshot.to_owned(),
                files: BTreeMap::new(),
                unknowns: Vec::new(),
            },
        }
    }

    /// Admit one file's identifiers.
    ///
    /// `bytes` is read once here and never retained. Only identifiers the task
    /// planner would also admit are kept, so the two sides cannot drift.
    pub fn admit(&mut self, display_path: &str, bytes: &[u8]) {
        if self.index.files.len() >= MAX_INDEXED_FILES {
            push_unknown(
                &mut self.index.unknowns,
                "identifier_index_file_limit_reached",
            );
            return;
        }
        let Ok(text) = std::str::from_utf8(bytes) else {
            // A file that is not UTF-8 contributes no identifiers, explicitly.
            push_unknown(
                &mut self.index.unknowns,
                "identifier_index_undecodable_file",
            );
            return;
        };
        let mut identifiers = BTreeSet::new();
        let mut truncated = false;
        for token in split_identifier_tokens(text) {
            if identifiers.len() >= MAX_IDENTIFIERS_PER_FILE {
                truncated = true;
                break;
            }
            if token.len() > MAX_IDENTIFIER_BYTES {
                continue;
            }
            if crate::is_code_identifier_signal(token) {
                identifiers.insert(token.to_owned());
            }
            // A dotted access carries the member the graph actually names, the
            // same way task signals treat `ts.remove_column`.
            if let Some((_, member)) = token.rsplit_once('.')
                && member.len() <= MAX_IDENTIFIER_BYTES
                && crate::is_code_identifier_signal(member)
                && identifiers.len() < MAX_IDENTIFIERS_PER_FILE
            {
                identifiers.insert(member.to_owned());
            }
        }
        if truncated {
            push_unknown(
                &mut self.index.unknowns,
                "identifier_index_file_identifier_limit_reached",
            );
        }
        if !identifiers.is_empty() {
            self.index
                .files
                .insert(display_path.to_owned(), identifiers);
        }
    }

    /// Finish the index.
    #[must_use]
    pub fn finish(mut self) -> TaskIdentifierIndex {
        self.index.unknowns.sort();
        self.index.unknowns.dedup();
        self.index
    }
}

impl TaskIdentifierIndex {
    /// Files containing every named identifier, in the shape nomination wants.
    ///
    /// The lookup takes no workspace handle, so it cannot read the repository.
    ///
    /// # Errors
    /// Returns a closed category when the index belongs to a different
    /// snapshot; a stale index must refuse rather than answer plausibly.
    pub fn identifier_matches(
        &self,
        workspace_snapshot: &str,
        identifiers: &[String],
    ) -> Result<BTreeMap<String, BTreeSet<String>>, IdentifierIndexErrorCode> {
        if workspace_snapshot != self.workspace_snapshot {
            return Err(IdentifierIndexErrorCode::SnapshotMismatch);
        }
        let wanted: BTreeSet<&str> = identifiers.iter().map(String::as_str).collect();
        let mut matches: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (path, held) in &self.files {
            let found: BTreeSet<String> = held
                .iter()
                .filter(|identifier| wanted.contains(identifier.as_str()))
                .cloned()
                .collect();
            if !found.is_empty() {
                matches.insert(path.clone(), found);
            }
        }
        Ok(matches)
    }

    /// Portable paths the index admitted, for the bound snapshot.
    ///
    /// Package-proximate reach (IC-PPNR-130) needs the set of files this
    /// product can parse at all. A sibling it cannot read is a wasted scope
    /// slot, and the admitted set is exactly that answer without a
    /// per-language table living in nomination.
    ///
    /// # Errors
    ///
    /// Returns [`IdentifierIndexErrorCode::SnapshotMismatch`] when the caller
    /// asks for a snapshot this index was not built from.
    pub fn admitted_paths(
        &self,
        workspace_snapshot: &str,
    ) -> Result<BTreeSet<String>, IdentifierIndexErrorCode> {
        if workspace_snapshot != self.workspace_snapshot {
            return Err(IdentifierIndexErrorCode::SnapshotMismatch);
        }
        Ok(self.files.keys().cloned().collect())
    }

    /// Files indexed.
    #[must_use]
    pub fn indexed_files(&self) -> usize {
        self.files.len()
    }
}

fn push_unknown(unknowns: &mut Vec<String>, reason: &str) {
    if !unknowns.iter().any(|value| value == reason) {
        unknowns.push(reason.to_owned());
    }
}

/// Split source text the way task signals split task text, so an identifier the
/// planner can name is an identifier the index can hold.
fn split_identifier_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| {
        !(character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ':'))
    })
    .filter(|token| !token.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNAPSHOT: &str =
        "sha256:1111111111111111111111111111111111111111111111111111111111111111";
    const OTHER: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";

    fn index_of(files: &[(&str, &str)]) -> TaskIdentifierIndex {
        let mut builder = TaskIdentifierIndexBuilder::new(SNAPSHOT);
        for (path, body) in files {
            builder.admit(path, body.as_bytes());
        }
        builder.finish()
    }

    #[test]
    fn the_index_holds_what_a_task_could_name() {
        // The astropy case: the report names `_required_columns`, and the file
        // that matters declares it.
        let index = index_of(&[
            (
                "astropy/timeseries/core.py",
                "class BaseTimeSeries(QTable):\n    _required_columns = None\n    def _check_required_columns(self):\n        pass\n",
            ),
            (
                "astropy/timeseries/sampled.py",
                "class TimeSeries(BaseTimeSeries):\n    def add_column(self):\n        pass\n",
            ),
        ]);
        let matches = index
            .identifier_matches(SNAPSHOT, &["_required_columns".to_owned()])
            .expect("matches");
        assert!(matches.contains_key("astropy/timeseries/core.py"));
        assert!(!matches.contains_key("astropy/timeseries/sampled.py"));

        // CamelCase is code-shaped, so a class name is findable too.
        let classes = index
            .identifier_matches(SNAPSHOT, &["BaseTimeSeries".to_owned()])
            .expect("matches");
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn prose_shaped_words_are_never_indexed() {
        // The index admits only what the task planner would admit. A bare
        // lowercase word is indistinguishable from prose by shape.
        let index = index_of(&[("a.py", "the value of result is here\n")]);
        for word in ["the", "value", "result", "here"] {
            let matches = index
                .identifier_matches(SNAPSHOT, &[(*word).to_owned()])
                .expect("matches");
            assert!(matches.is_empty(), "{word} must not be indexed");
        }
    }

    #[test]
    fn a_dotted_access_indexes_the_member_the_graph_names() {
        let index = index_of(&[("a.py", "ts.remove_column('flux')\n")]);
        let matches = index
            .identifier_matches(SNAPSHOT, &["remove_column".to_owned()])
            .expect("matches");
        assert!(matches.contains_key("a.py"));
    }

    #[test]
    fn a_stale_index_refuses_rather_than_answering() {
        let index = index_of(&[("a.py", "def alpha_beta():\n    pass\n")]);
        assert!(matches!(
            index.identifier_matches(OTHER, &["alpha_beta".to_owned()]),
            Err(IdentifierIndexErrorCode::SnapshotMismatch)
        ));
    }

    #[test]
    fn per_file_identifier_bound_is_enforced_and_recorded() {
        let body = (0..MAX_IDENTIFIERS_PER_FILE * 2)
            .map(|index| format!("name_{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let index = index_of(&[("big.py", &body)]);
        assert_eq!(
            index.files["big.py"].len(),
            MAX_IDENTIFIERS_PER_FILE,
            "bound must hold"
        );
        assert!(
            index
                .unknowns
                .contains(&"identifier_index_file_identifier_limit_reached".to_owned())
        );
    }

    #[test]
    fn an_undecodable_file_is_recorded_not_silently_skipped() {
        let mut builder = TaskIdentifierIndexBuilder::new(SNAPSHOT);
        builder.admit("binary.bin", &[0xff, 0xfe, 0x00]);
        let index = builder.finish();
        assert!(index.files.is_empty());
        assert!(
            index
                .unknowns
                .contains(&"identifier_index_undecodable_file".to_owned())
        );
    }

    #[test]
    fn an_oversized_token_is_dropped_without_dropping_the_file() {
        let long = "a_".to_owned() + &"z".repeat(MAX_IDENTIFIER_BYTES);
        let body = format!("{long} keeper_name\n");
        let index = index_of(&[("a.py", &body)]);
        let held = &index.files["a.py"];
        assert!(held.contains("keeper_name"));
        assert!(held.iter().all(|value| value.len() <= MAX_IDENTIFIER_BYTES));
    }

    #[test]
    fn building_twice_over_identical_input_yields_an_identical_index() {
        let files = [
            ("a.py", "def alpha_beta():\n    pass\n"),
            ("b.py", "GammaDelta = 1\n"),
        ];
        assert_eq!(index_of(&files), index_of(&files));
    }

    #[test]
    fn a_lookup_performs_no_repository_read() {
        // Structural, not incidental: the lookup signature takes no workspace,
        // no path, and no reader, so it cannot reach the filesystem.
        let source = include_str!("identifier_index.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        let lookup = shipped
            .split_once("pub fn identifier_matches(")
            .expect("lookup")
            .1
            .split_once("\n    }")
            .expect("lookup body")
            .0;
        for forbidden in ["fs::", "read_exact", "File", "Path"] {
            assert!(
                !lookup.contains(forbidden),
                "lookup must not reach {forbidden}"
            );
        }
    }

    #[test]
    fn module_retains_no_source_and_reaches_no_oracle_or_execution() {
        let source = include_str!("identifier_index.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        for forbidden in [
            "reference_patch",
            "FAIL_TO_PASS",
            "test_patch",
            "Command",
            "spawn",
            "excerpt",
            "start_byte",
        ] {
            assert!(!shipped.contains(forbidden), "must not reach {forbidden}");
        }
    }
}
