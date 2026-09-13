// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! What a session was already sent by read substitution.
//!
//! A host that offers the same read twice in one session would be sent the
//! same declarations twice. The server remembers, per session, only the
//! identity of each answer it delivered and the request that delivered it,
//! never the bytes, so no repository content outlives its request (ADR-0135).
//! A repeat whose answer is unchanged is met with a notice naming that request.
//! The notice is an offer like any other answer: the host can ask for the full
//! answer again, or read the file itself.

use std::collections::BTreeMap;

use context_core::json_contract_identity;
use context_engine::read_substitution::ReadSubstitution;
use serde::Serialize;
use serde_json::Value;

/// Answers remembered per session. Recording stops at the bound instead of
/// evicting, so memory does not grow with a session and nothing remembered is
/// forgotten while the session lasts. Past the bound, answers are sent in full.
const MAX_REMEMBERED_ANSWERS: usize = 4_096;

/// A delivered answer's identity, and the request that delivered it.
struct Remembered {
    identity: String,
    request_id: String,
}

/// Per-session memory of delivered read substitutions.
#[derive(Default)]
pub(crate) struct ReadMemory {
    sessions: BTreeMap<String, BTreeMap<(String, Option<String>), Remembered>>,
}

/// Everything a host acts on in an answer, without its bytes: each span's range
/// and content hash already commit to its text.
#[derive(Serialize)]
struct AnswerIdentity<'a> {
    display_path: &'a str,
    requested_symbol: Option<&'a str>,
    spans: Vec<(u64, u64, &'a str)>,
    omitted_spans: u64,
    unknowns: &'a [String],
}

/// Sent instead of an answer this session already holds unchanged.
#[derive(Serialize)]
struct RepeatNotice<'a> {
    schema_name: &'static str,
    schema_version: &'static str,
    display_path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_symbol: Option<&'a str>,
    unchanged_since_request: &'a str,
    answer_identity: &'a str,
    withheld_bytes: u64,
    whole_file_bytes: u64,
    to_receive_it_again: &'static str,
    authority_added: bool,
}

impl ReadMemory {
    /// Forget everything a closed session was sent.
    pub(crate) fn forget_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    /// Answer a read offer: the full substitution, or a notice when this
    /// session was already sent the same answer and `repeat` is not set.
    pub(crate) fn answer(
        &mut self,
        session_id: &str,
        request_id: &str,
        repeat: bool,
        substitution: ReadSubstitution,
    ) -> Result<Value, &'static str> {
        let identity = json_contract_identity(
            "read-substitution-answer",
            &AnswerIdentity {
                display_path: &substitution.display_path,
                requested_symbol: substitution.requested_symbol.as_deref(),
                spans: substitution
                    .spans
                    .iter()
                    .map(|span| (span.start_byte, span.end_byte, span.content_sha256.as_str()))
                    .collect(),
                omitted_spans: substitution.omitted_spans,
                unknowns: &substitution.unknowns,
            },
        )
        .map_err(|_| "read substitution failed")?;
        let key = (
            substitution.display_path.clone(),
            substitution.requested_symbol.clone(),
        );
        let session = self.sessions.entry(session_id.to_owned()).or_default();
        if !repeat
            && let Some(earlier) = session.get(&key)
            && earlier.identity == identity
        {
            return serde_json::to_value(RepeatNotice {
                schema_name: "read-substitution-repeat",
                schema_version: "1.0.0",
                display_path: &substitution.display_path,
                requested_symbol: substitution.requested_symbol.as_deref(),
                unchanged_since_request: &earlier.request_id,
                answer_identity: &identity,
                withheld_bytes: substitution.returned_bytes,
                whole_file_bytes: substitution.whole_file_bytes,
                to_receive_it_again: "call again with repeat set to true, or read the file",
                authority_added: false,
            })
            .map_err(|_| "read substitution failed");
        }
        if session.len() < MAX_REMEMBERED_ANSWERS || session.contains_key(&key) {
            session.insert(
                key,
                Remembered {
                    identity,
                    request_id: request_id.to_owned(),
                },
            );
        }
        serde_json::to_value(substitution).map_err(|_| "read substitution failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use context_engine::read_substitution::SubstitutedSpan;

    fn substitution(path: &str, content_sha256: &str) -> ReadSubstitution {
        ReadSubstitution {
            schema_name: "read-substitution".into(),
            schema_version: "1.0.0".into(),
            display_path: path.into(),
            requested_symbol: Some("authenticate".into()),
            spans: vec![SubstitutedSpan {
                name: Some("authenticate".into()),
                start_byte: 0,
                end_byte: 30,
                content_sha256: content_sha256.into(),
                text: "fn authenticate() { audit(); }".into(),
            }],
            whole_file_bytes: 45,
            returned_bytes: 30,
            omitted_spans: 0,
            unknowns: Vec::new(),
        }
    }

    #[test]
    fn a_repeated_unchanged_answer_names_the_request_that_delivered_it() {
        let mut memory = ReadMemory::default();
        let first = memory
            .answer(
                "session_a",
                "req_first",
                false,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("first");
        assert_eq!(first["spans"][0]["text"], "fn authenticate() { audit(); }");
        let second = memory
            .answer(
                "session_a",
                "req_second",
                false,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("second");
        assert_eq!(second["schema_name"], "read-substitution-repeat");
        assert_eq!(second["unchanged_since_request"], "req_first");
        assert_eq!(second["withheld_bytes"], 30);
        assert_eq!(second["requested_symbol"], "authenticate");
        assert!(second.get("spans").is_none(), "no bytes are sent again");
        assert_eq!(second["authority_added"], false);
    }

    #[test]
    fn a_repeat_request_a_changed_source_or_another_session_gets_the_full_answer() {
        let mut memory = ReadMemory::default();
        memory
            .answer(
                "session_a",
                "req_first",
                false,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("first");
        let asked = memory
            .answer(
                "session_a",
                "req_again",
                true,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("repeat");
        assert!(
            asked.get("spans").is_some(),
            "repeat asks for the full answer"
        );
        let changed = memory
            .answer(
                "session_a",
                "req_changed",
                false,
                substitution("auth.rs", "sha256:bb"),
            )
            .expect("changed");
        assert!(
            changed.get("spans").is_some(),
            "a changed source is sent in full"
        );
        let other = memory
            .answer(
                "session_b",
                "req_other",
                false,
                substitution("auth.rs", "sha256:bb"),
            )
            .expect("other");
        assert!(
            other.get("spans").is_some(),
            "another session was never sent it"
        );
        let again = memory
            .answer(
                "session_a",
                "req_after",
                false,
                substitution("auth.rs", "sha256:bb"),
            )
            .expect("again");
        assert_eq!(
            again["unchanged_since_request"], "req_changed",
            "the session now holds the changed answer"
        );
    }

    #[test]
    fn a_closed_session_is_forgotten_and_memory_stops_at_its_bound() {
        let mut memory = ReadMemory::default();
        memory
            .answer(
                "session_a",
                "req_first",
                false,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("first");
        memory.forget_session("session_a");
        let reopened = memory
            .answer(
                "session_a",
                "req_reopened",
                false,
                substitution("auth.rs", "sha256:aa"),
            )
            .expect("reopened");
        assert!(
            reopened.get("spans").is_some(),
            "a closed session is forgotten"
        );

        let mut bounded = ReadMemory::default();
        for index in 0..MAX_REMEMBERED_ANSWERS {
            bounded
                .answer(
                    "session_a",
                    "req_fill",
                    false,
                    substitution(&format!("f{index}.rs"), "sha256:aa"),
                )
                .expect("fill");
        }
        bounded
            .answer(
                "session_a",
                "req_over",
                false,
                substitution("over.rs", "sha256:aa"),
            )
            .expect("over");
        let over = bounded
            .answer(
                "session_a",
                "req_over_again",
                false,
                substitution("over.rs", "sha256:aa"),
            )
            .expect("over again");
        assert!(
            over.get("spans").is_some(),
            "past the bound nothing new is remembered"
        );
        let early = bounded
            .answer(
                "session_a",
                "req_early",
                false,
                substitution("f0.rs", "sha256:aa"),
            )
            .expect("early");
        assert_eq!(
            early["schema_name"], "read-substitution-repeat",
            "nothing remembered is evicted"
        );
    }
}
