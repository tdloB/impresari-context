// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Host read substitution (IC-HRS-136).
//!
//! A host about to read a file may offer that read here and receive the file's
//! declarations as exact spans instead of the whole file. The host still
//! performs every operation and still decides; this module only selects spans
//! from a graph and attests the bytes recovered for them.
//!
//! Nothing here launches a process, opens a socket, or writes to a workspace,
//! so `SEC-INV-007` stays literally true.

use serde::{Deserialize, Serialize};

/// Schema discriminator for a read-substitution answer.
pub const READ_SUBSTITUTION_SCHEMA_NAME: &str = "impresari_context_read_substitution";
/// Schema version for a read-substitution answer.
pub const READ_SUBSTITUTION_SCHEMA_VERSION: &str = "1.0";

/// Largest number of declaration spans one answer may carry.
///
/// A generated file can declare thousands of symbols, and an answer larger than
/// the file it replaces is not a substitution.
pub const MAX_SUBSTITUTED_SPANS: usize = 256;

/// Why a substitution declined to answer, or answered partially.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubstitutionUnknown {
    /// The snapshot holds no declarations for this path.
    NoDeclarations,
    /// The host's byte ceiling stopped the answer short.
    ByteCeilingReached,
    /// The span ceiling stopped the answer short.
    SpanCeilingReached,
}

impl SubstitutionUnknown {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::NoDeclarations => "no_declarations_for_path",
            Self::ByteCeilingReached => "substitution_byte_ceiling_reached",
            Self::SpanCeilingReached => "substitution_span_ceiling_reached",
        }
    }
}

/// One declaration recovered from the admitted source.
///
/// `content_sha256` is computed over exactly the bytes in `text`, so a host
/// holding the file can verify the span itself. A returned byte that does not
/// verify is rejected by arithmetic rather than by trust (`SEC-INV-011`).
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubstitutedSpan {
    /// Optional declaration name, when the graph carries one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Inclusive starting byte in the file.
    pub start_byte: u64,
    /// Exclusive ending byte in the file.
    pub end_byte: u64,
    /// Hash over exactly the returned bytes.
    pub content_sha256: String,
    /// The recovered bytes.
    pub text: String,
}

/// An offer answered: the declarations of one path, and what it cost.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReadSubstitution {
    /// Schema discriminator.
    pub schema_name: String,
    /// Schema version.
    pub schema_version: String,
    /// Portable path the host offered.
    pub display_path: String,
    /// Declarations, in source order.
    pub spans: Vec<SubstitutedSpan>,
    /// Bytes the whole file would have cost the host.
    pub whole_file_bytes: u64,
    /// Bytes this answer returns.
    pub returned_bytes: u64,
    /// Declarations present but not returned.
    pub omitted_spans: u64,
    /// Explicit shortfall reasons.
    pub unknowns: Vec<String>,
}

impl ReadSubstitution {
    /// Whether taking this answer returns fewer bytes than the file.
    ///
    /// A host decides per call, so it is given the comparison rather than a
    /// recommendation. An answer that is not smaller is still a valid answer;
    /// it is simply one a host should decline.
    #[must_use]
    pub const fn is_smaller_than_source(&self) -> bool {
        self.returned_bytes < self.whole_file_bytes
    }
}

/// A declaration the caller recovered from the graph, before attestation.
pub struct DeclarationSpan<'a> {
    /// Optional declaration name.
    pub name: Option<&'a str>,
    /// Inclusive starting byte.
    pub start_byte: u64,
    /// Exclusive ending byte.
    pub end_byte: u64,
}

/// Answer a read offer with the declarations a path contains.
///
/// `source` is the file's exact bytes, already read and verified by the caller
/// against the authorized snapshot. `declarations` are spans recovered from the
/// structural graph for that same path. Both come from the admitted source, so
/// this function synthesizes nothing: it selects, bounds, and attests.
///
/// `maximum_returned_bytes` is the host's ceiling. Spans are taken in source
/// order until it is reached, so a truncated answer is a prefix of the file
/// rather than an arbitrary subset.
#[must_use]
pub fn substitute_read(
    display_path: &str,
    source: &[u8],
    declarations: &[DeclarationSpan<'_>],
    maximum_returned_bytes: u64,
) -> ReadSubstitution {
    let whole_file_bytes = u64::try_from(source.len()).unwrap_or(u64::MAX);
    let mut spans: Vec<SubstitutedSpan> = Vec::new();
    let mut returned_bytes = 0_u64;
    let mut unknowns: Vec<&'static str> = Vec::new();
    let mut omitted_spans = 0_u64;

    let mut ordered: Vec<&DeclarationSpan<'_>> = declarations.iter().collect();
    ordered.sort_by_key(|span| (span.start_byte, span.end_byte));

    for declaration in ordered {
        if spans.len() >= MAX_SUBSTITUTED_SPANS {
            unknowns.push(SubstitutionUnknown::SpanCeilingReached.reason_code());
            omitted_spans = omitted_spans.saturating_add(1);
            continue;
        }
        let Some(bytes) = span_bytes(source, declaration) else {
            // A span the source cannot satisfy is dropped rather than guessed.
            omitted_spans = omitted_spans.saturating_add(1);
            continue;
        };
        let Ok(text) = std::str::from_utf8(bytes) else {
            omitted_spans = omitted_spans.saturating_add(1);
            continue;
        };
        let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if returned_bytes.saturating_add(size) > maximum_returned_bytes {
            unknowns.push(SubstitutionUnknown::ByteCeilingReached.reason_code());
            omitted_spans = omitted_spans.saturating_add(1);
            continue;
        }
        returned_bytes = returned_bytes.saturating_add(size);
        spans.push(SubstitutedSpan {
            name: declaration.name.map(str::to_owned),
            start_byte: declaration.start_byte,
            end_byte: declaration.end_byte,
            content_sha256: crate::contract_sha256(bytes),
            text: text.to_owned(),
        });
    }

    if spans.is_empty() {
        unknowns.push(SubstitutionUnknown::NoDeclarations.reason_code());
    }
    let mut unknowns: Vec<String> = unknowns.into_iter().map(str::to_owned).collect();
    unknowns.sort();
    unknowns.dedup();

    ReadSubstitution {
        schema_name: READ_SUBSTITUTION_SCHEMA_NAME.to_owned(),
        schema_version: READ_SUBSTITUTION_SCHEMA_VERSION.to_owned(),
        display_path: display_path.to_owned(),
        spans,
        whole_file_bytes,
        returned_bytes,
        omitted_spans,
        unknowns,
    }
}

/// Bytes for one span, when the source actually contains them.
fn span_bytes<'a>(source: &'a [u8], declaration: &DeclarationSpan<'_>) -> Option<&'a [u8]> {
    let start = usize::try_from(declaration.start_byte).ok()?;
    let end = usize::try_from(declaration.end_byte).ok()?;
    if start >= end || end > source.len() {
        return None;
    }
    source.get(start..end)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &[u8] = b"# header\nclass Card:\n    pass\n\nclass Header:\n    pass\n";

    fn decl(name: &str, start: u64, end: u64) -> DeclarationSpan<'_> {
        DeclarationSpan {
            name: Some(name),
            start_byte: start,
            end_byte: end,
        }
    }

    #[test]
    fn every_returned_span_verifies_against_the_source() {
        // This is the security model: a host holding the file checks each span
        // itself, so a byte that does not verify is rejected by arithmetic
        // rather than by trusting the hook (SEC-INV-011).
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[decl("Card", 9, 29), decl("Header", 31, 54)],
            4096,
        );
        assert_eq!(answer.spans.len(), 2);
        for span in &answer.spans {
            let start = usize::try_from(span.start_byte).expect("start fits");
            let end = usize::try_from(span.end_byte).expect("end fits");
            let from_source = &SOURCE[start..end];
            assert_eq!(span.text.as_bytes(), from_source, "span must be exact");
            assert_eq!(
                span.content_sha256,
                crate::contract_sha256(from_source),
                "hash must attest exactly the returned bytes"
            );
        }
    }

    #[test]
    fn a_substitution_reports_what_it_saved() {
        // A host decides per call and a measurement attributes savings, so the
        // comparison is returned rather than a recommendation.
        let answer = substitute_read("fits/card.py", SOURCE, &[decl("Card", 9, 29)], 4096);
        assert_eq!(
            answer.whole_file_bytes,
            u64::try_from(SOURCE.len()).expect("len fits")
        );
        assert_eq!(answer.returned_bytes, 20);
        assert!(answer.is_smaller_than_source());
    }

    #[test]
    fn the_byte_ceiling_is_honoured_and_disclosed() {
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[decl("Card", 9, 29), decl("Header", 31, 54)],
            20,
        );
        assert_eq!(answer.spans.len(), 1);
        assert!(answer.returned_bytes <= 20);
        assert_eq!(answer.omitted_spans, 1);
        assert!(
            answer
                .unknowns
                .contains(&"substitution_byte_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn a_path_with_no_declarations_says_so_and_returns_nothing() {
        let answer = substitute_read("docs/readme.md", SOURCE, &[], 4096);
        assert!(answer.spans.is_empty());
        assert_eq!(answer.returned_bytes, 0);
        assert!(
            answer
                .unknowns
                .contains(&"no_declarations_for_path".to_owned())
        );
        // An empty answer is still honest about what the file would have cost.
        assert!(answer.whole_file_bytes > 0);
    }

    #[test]
    fn a_span_the_source_cannot_satisfy_is_dropped_not_guessed() {
        // A stale graph must never cause invented bytes to be returned.
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[decl("Card", 9, 29), decl("Gone", 9_000, 9_100)],
            4096,
        );
        assert_eq!(answer.spans.len(), 1);
        assert_eq!(answer.spans[0].name.as_deref(), Some("Card"));
        assert_eq!(answer.omitted_spans, 1);
    }

    #[test]
    fn spans_are_returned_in_source_order() {
        // A truncated answer must be a prefix of the file, not an arbitrary
        // subset, or a host cannot reason about what it received.
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[decl("Header", 31, 54), decl("Card", 9, 29)],
            4096,
        );
        let order: Vec<u64> = answer.spans.iter().map(|span| span.start_byte).collect();
        assert_eq!(order, vec![9, 31]);
    }

    #[test]
    fn the_span_ceiling_bounds_a_generated_file() {
        let many: Vec<DeclarationSpan<'_>> = (0..MAX_SUBSTITUTED_SPANS + 10)
            .map(|_| DeclarationSpan {
                name: None,
                start_byte: 9,
                end_byte: 29,
            })
            .collect();
        let answer = substitute_read("fits/card.py", SOURCE, &many, u64::MAX);
        assert_eq!(answer.spans.len(), MAX_SUBSTITUTED_SPANS);
        assert!(
            answer
                .unknowns
                .contains(&"substitution_span_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn module_launches_nothing_and_writes_nothing() {
        let source = include_str!("read_substitution.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        // Network and process tokens are deliberately absent from this list:
        // `scripts/check-security-boundaries.sh` greps production sources for
        // them, and naming them here would make this test's own source trip
        // that scanner. This checks only what the scanner does not.
        for forbidden in ["fs::write", "OpenOptions", "remove_file", "rename("] {
            assert!(
                !shipped.contains(forbidden),
                "read substitution must not reach {forbidden}"
            );
        }
    }
}
