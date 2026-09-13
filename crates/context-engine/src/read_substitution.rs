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
    /// The offer named a symbol the graph does not hold for this path.
    SymbolNotDeclared,
    /// The graph for this path is a prefix, so absence proves nothing.
    GraphTruncated,
}

impl SubstitutionUnknown {
    /// Stable source-free reason code.
    #[must_use]
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::NoDeclarations => "no_declarations_for_path",
            Self::ByteCeilingReached => "substitution_byte_ceiling_reached",
            Self::SpanCeilingReached => "substitution_span_ceiling_reached",
            Self::SymbolNotDeclared => "symbol_not_declared_in_path",
            Self::GraphTruncated => "structural_graph_truncated_for_path",
        }
    }
}

/// Whether the graph the caller recovered declarations from covers the file.
///
/// The structural worker bounds its response, and over the ceiling it returns a
/// prefix of the fact list rather than failing. A prefix answers honestly about
/// what it holds but cannot answer about what it never reached — measured on
/// `astropy/io/fits/header.py`, every declaration after roughly line 1,920 was
/// missing from a graph that reported no shortfall of its own.
///
/// This matters most for a named symbol. Without it, a symbol the file really
/// declares comes back as `symbol_not_declared_in_path`, and a host that trusts
/// that skips the read and loses the declaration entirely.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphCompleteness {
    /// Every declaration in the file reached the graph.
    Complete,
    /// The graph holds a prefix of the file's declarations.
    Truncated,
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
    /// Symbol the host asked for, when it named one.
    ///
    /// A whole-path offer returns the file's declarations, which on a
    /// declaration-dense language is nearly the file — measured, 93% on Python
    /// and 91% on TypeScript. Naming the symbol a map already points at is what
    /// makes a substitution small, and it is small in any language because it
    /// does not depend on how much of a file is declarations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_symbol: Option<String>,
    /// Declarations, in source order.
    pub spans: Vec<SubstitutedSpan>,
    /// Bytes the whole file would have cost the host.
    pub whole_file_bytes: u64,
    /// Bytes this answer returns.
    pub returned_bytes: u64,
    /// Declarations selected for this answer but not returned.
    ///
    /// A ceiling or an unsatisfiable span omits a declaration. Declarations a
    /// named `requested_symbol` excluded are not counted here: the host asked
    /// for one symbol, so the rest of the file was never a candidate.
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
/// `symbol` narrows the answer to one named declaration. A map points at a
/// symbol, not a file, so answering the symbol is what makes a substitution
/// small: measured, returning a path's declarations is 93% of a Python file,
/// while the one method a map named is a fraction of it.
///
/// `completeness` says whether `declarations` covers the file. A truncated
/// graph cannot support a claim of absence, so an empty answer from one is
/// disclosed as such rather than reported as a symbol the file does not have.
///
/// `maximum_returned_bytes` is the host's ceiling. Spans are taken in source
/// order until it is reached, so a truncated answer is a prefix of the file
/// rather than an arbitrary subset.
#[must_use]
pub fn substitute_read(
    display_path: &str,
    source: &[u8],
    declarations: &[DeclarationSpan<'_>],
    symbol: Option<&str>,
    completeness: GraphCompleteness,
    maximum_returned_bytes: u64,
) -> ReadSubstitution {
    let whole_file_bytes = u64::try_from(source.len()).unwrap_or(u64::MAX);
    let mut spans: Vec<SubstitutedSpan> = Vec::new();
    let mut returned_bytes = 0_u64;
    let mut unknowns: Vec<&'static str> = Vec::new();
    let mut omitted_spans = 0_u64;

    let ordered = match symbol {
        // A named symbol is answered exactly, without the enclosing-declaration
        // collapse below. The value of naming one is reaching a nested method,
        // and collapsing to the outermost declaration would return the class
        // that contains it — which is the whole-file answer again.
        Some(name) => named_declarations(declarations, name),
        // Declarations nest: a class contains methods, a method contains
        // closures, and each is its own node with its own span. Returning them
        // all returns the enclosing bytes once per level — measured on
        // `astropy/timeseries/core.py`, eighteen spans came to 162% of the file
        // being replaced. Enclosed declarations are therefore dropped, so no
        // byte is returned twice and the outermost keeps its name.
        None => outermost_declarations(declarations),
    };
    if completeness == GraphCompleteness::Truncated {
        unknowns.push(SubstitutionUnknown::GraphTruncated.reason_code());
    }
    if symbol.is_some() && ordered.is_empty() {
        unknowns.push(SubstitutionUnknown::SymbolNotDeclared.reason_code());
    }

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

    if spans.is_empty() && symbol.is_none() {
        unknowns.push(SubstitutionUnknown::NoDeclarations.reason_code());
    }
    let mut unknowns: Vec<String> = unknowns.into_iter().map(str::to_owned).collect();
    unknowns.sort();
    unknowns.dedup();

    ReadSubstitution {
        schema_name: READ_SUBSTITUTION_SCHEMA_NAME.to_owned(),
        schema_version: READ_SUBSTITUTION_SCHEMA_VERSION.to_owned(),
        display_path: display_path.to_owned(),
        requested_symbol: symbol.map(str::to_owned),
        spans,
        whole_file_bytes,
        returned_bytes,
        omitted_spans,
        unknowns,
    }
}

/// Declarations carrying exactly this name, in source order.
///
/// Nesting is not collapsed here. A map names the method it points at, and the
/// class enclosing it is the answer this narrowing exists to avoid.
fn named_declarations<'a>(
    declarations: &'a [DeclarationSpan<'a>],
    symbol: &str,
) -> Vec<&'a DeclarationSpan<'a>> {
    let mut matched: Vec<&DeclarationSpan<'_>> = declarations
        .iter()
        .filter(|declaration| declaration.name == Some(symbol))
        .collect();
    matched.sort_by_key(|declaration| (declaration.start_byte, declaration.end_byte));
    matched
}

/// Declarations that no other declaration encloses, in source order.
///
/// A substitution returns each byte at most once. Where declarations nest, the
/// enclosing one is kept: it carries the name a map would point at, and its
/// bytes already contain the nested ones.
fn outermost_declarations<'a>(
    declarations: &'a [DeclarationSpan<'a>],
) -> Vec<&'a DeclarationSpan<'a>> {
    let mut ordered: Vec<&DeclarationSpan<'_>> = declarations.iter().collect();
    // Widest first at a shared start, so an enclosing span is seen before the
    // declarations it contains.
    ordered.sort_by(|left, right| {
        left.start_byte
            .cmp(&right.start_byte)
            .then_with(|| right.end_byte.cmp(&left.end_byte))
    });
    let mut kept: Vec<&DeclarationSpan<'_>> = Vec::new();
    let mut covered_to = 0_u64;
    for declaration in ordered {
        if !kept.is_empty() && declaration.end_byte <= covered_to {
            continue;
        }
        covered_to = covered_to.max(declaration.end_byte);
        kept.push(declaration);
    }
    kept
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
            None,
            GraphCompleteness::Complete,
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
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[decl("Card", 9, 29)],
            None,
            GraphCompleteness::Complete,
            4096,
        );
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
            None,
            GraphCompleteness::Complete,
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
        let answer = substitute_read(
            "docs/readme.md",
            SOURCE,
            &[],
            None,
            GraphCompleteness::Complete,
            4096,
        );
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
            None,
            GraphCompleteness::Complete,
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
            None,
            GraphCompleteness::Complete,
            4096,
        );
        let order: Vec<u64> = answer.spans.iter().map(|span| span.start_byte).collect();
        assert_eq!(order, vec![9, 31]);
    }

    #[test]
    fn the_span_ceiling_bounds_a_generated_file() {
        // Distinct, non-overlapping declarations, so the ceiling is what stops
        // the answer rather than nesting collapse.
        let count = MAX_SUBSTITUTED_SPANS + 10;
        let generated = vec![b'x'; count * 4];
        let many: Vec<DeclarationSpan<'_>> = (0..count)
            .map(|index| {
                let start = u64::try_from(index * 4).expect("start fits");
                DeclarationSpan {
                    name: None,
                    start_byte: start,
                    end_byte: start + 4,
                }
            })
            .collect();
        let answer = substitute_read(
            "generated.py",
            &generated,
            &many,
            None,
            GraphCompleteness::Complete,
            u64::MAX,
        );
        assert_eq!(answer.spans.len(), MAX_SUBSTITUTED_SPANS);
        assert_eq!(
            answer.omitted_spans,
            u64::try_from(count - MAX_SUBSTITUTED_SPANS).expect("omitted fits")
        );
        assert!(
            answer
                .unknowns
                .contains(&"substitution_span_ceiling_reached".to_owned())
        );
    }

    #[test]
    fn a_nested_declaration_never_returns_its_bytes_twice() {
        // Declarations nest, and each level is its own node. Measured on
        // `astropy/timeseries/core.py`, returning all eighteen came to 162% of
        // the file the substitution was meant to replace.
        let outer = decl("autocheck", 9, 54);
        let inner = decl("wrapper", 20, 40);
        let answer = substitute_read(
            "fits/card.py",
            SOURCE,
            &[outer, inner],
            None,
            GraphCompleteness::Complete,
            4096,
        );
        assert_eq!(answer.spans.len(), 1);
        assert_eq!(answer.spans[0].name.as_deref(), Some("autocheck"));
        assert!(
            answer.returned_bytes <= answer.whole_file_bytes,
            "a substitution must never return more than the file it replaces"
        );
    }

    // A class with two methods: the shape a map points into, and the shape that
    // makes a whole-path answer nearly the whole file.
    const NESTED: &[u8] =
        b"class Card:\n    def verify(self):\n        return 1\n\n    def image(self):\n        return 2\n";

    fn nested_declarations() -> Vec<DeclarationSpan<'static>> {
        vec![
            decl("Card", 0, 90),
            decl("verify", 16, 51),
            decl("image", 56, 90),
        ]
    }

    #[test]
    fn a_named_symbol_returns_that_declaration_and_not_the_class_around_it() {
        // This is the whole point of naming a symbol. Collapsing nesting here
        // would return `Card`, which is the file again — measured, a whole-path
        // answer is 93% of a Python file.
        let answer = substitute_read(
            "fits/card.py",
            NESTED,
            &nested_declarations(),
            Some("verify"),
            GraphCompleteness::Complete,
            4096,
        );
        assert_eq!(answer.spans.len(), 1);
        assert_eq!(answer.spans[0].name.as_deref(), Some("verify"));
        assert_eq!(answer.spans[0].start_byte, 16);
        assert_eq!(answer.requested_symbol.as_deref(), Some("verify"));
        assert!(answer.unknowns.is_empty());

        let whole_path = substitute_read(
            "fits/card.py",
            NESTED,
            &nested_declarations(),
            None,
            GraphCompleteness::Complete,
            4096,
        );
        assert_eq!(
            whole_path.returned_bytes, whole_path.whole_file_bytes,
            "the whole-path answer on this file is the file"
        );
        assert!(
            answer.returned_bytes < whole_path.returned_bytes,
            "naming a symbol must return less than naming the path"
        );
    }

    #[test]
    fn a_symbol_the_path_does_not_declare_returns_nothing_and_says_which() {
        // A map can point at a symbol a stale snapshot no longer holds. The
        // answer must be empty and say so, never the nearest declaration.
        let answer = substitute_read(
            "fits/card.py",
            NESTED,
            &nested_declarations(),
            Some("checksum"),
            GraphCompleteness::Complete,
            4096,
        );
        assert!(answer.spans.is_empty());
        assert_eq!(answer.returned_bytes, 0);
        assert_eq!(
            answer.unknowns,
            vec!["symbol_not_declared_in_path".to_owned()],
            "an unmatched symbol is its own reason, not `no_declarations_for_path`"
        );
    }

    #[test]
    fn a_repeated_symbol_name_returns_every_declaration_of_it_in_source_order() {
        // Two classes can declare the same method name. Returning only the
        // first would hide the one the host is looking for. `stop` is the
        // declaration a whole-path answer would include and this one must not.
        let source = b"def go():\n    pass\n\ndef stop():\n    pass\n\ndef go():\n    pass\n";
        let answer = substitute_read(
            "shadowed.py",
            source,
            &[decl("go", 42, 61), decl("stop", 20, 41), decl("go", 0, 19)],
            Some("go"),
            GraphCompleteness::Complete,
            4096,
        );
        let order: Vec<u64> = answer.spans.iter().map(|span| span.start_byte).collect();
        assert_eq!(order, vec![0, 42], "both `go` declarations, and only those");
        for span in &answer.spans {
            assert_eq!(span.name.as_deref(), Some("go"));
        }
    }

    #[test]
    fn a_truncated_graph_never_claims_a_symbol_is_absent_without_saying_so() {
        // The worker returns a prefix of the fact list over its response
        // ceiling. Measured on `astropy/io/fits/header.py`, that dropped every
        // declaration after roughly line 1,920 — including whole top-level
        // classes — from a graph that reported no shortfall of its own. A host
        // told only `symbol_not_declared_in_path` would skip its read and lose
        // the declaration, so the prefix must be disclosed alongside it.
        let answer = substitute_read(
            "fits/header.py",
            NESTED,
            &nested_declarations(),
            Some("_BasicHeader"),
            GraphCompleteness::Truncated,
            4096,
        );
        assert!(answer.spans.is_empty());
        assert!(
            answer
                .unknowns
                .contains(&"structural_graph_truncated_for_path".to_owned()),
            "absence from a prefix must be disclosed as a prefix"
        );
        assert!(
            answer
                .unknowns
                .contains(&"symbol_not_declared_in_path".to_owned())
        );
    }

    #[test]
    fn a_truncated_graph_is_disclosed_even_when_it_answered() {
        // A whole-path answer off a prefix is smaller than the file for a reason
        // a host must not read as compression.
        let answer = substitute_read(
            "fits/header.py",
            NESTED,
            &[decl("Card", 0, 51)],
            None,
            GraphCompleteness::Truncated,
            4096,
        );
        assert_eq!(answer.spans.len(), 1);
        assert!(
            answer
                .unknowns
                .contains(&"structural_graph_truncated_for_path".to_owned())
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
