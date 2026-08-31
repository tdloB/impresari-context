// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Pure bounded normalization of one original-synthetic YARA-X NDJSON record."]
#![allow(clippy::struct_excessive_bools)]

use std::{error::Error, fmt, fmt::Write as _};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Frozen adapter profile identifier.
pub const PROFILE_ID: &str = "yara-x-ndjson-adapter-v1";
/// Frozen adapter profile digest. The repository checker binds this to the profile bytes.
pub const PROFILE_DIGEST: &str =
    "sha256:e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c";

const MAX_INPUT_BYTES: usize = 131_072;
const MAX_PATH_BYTES: usize = 4_096;
const MAX_RULES: usize = 256;
const MAX_OBSERVATIONS: usize = 256;
const MAX_TAGS_PER_OBSERVATION: usize = 32;
const MAX_RANGES_PER_OBSERVATION: usize = 32;
const MAX_TOTAL_RANGES: usize = 8_192;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_NORMALIZED_OUTPUT_BYTES: usize = 2_097_152;
const RESULT_ID_DOMAIN: &[u8] = b"impresari-context/yara-x-normalized-result/v1\0";

/// Stable source-free parser failure categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterErrorCode {
    /// A frozen size or count ceiling was exceeded.
    ResourceLimit,
    /// The one-record LF framing contract was violated.
    Framing,
    /// The record was not valid UTF-8.
    Utf8,
    /// JSON was malformed or did not match the closed vendor shape.
    JsonContract,
    /// Separately supplied control metadata was invalid.
    InvalidControl,
    /// The vendor path did not equal the separately supplied staged path.
    PathMismatch,
    /// A rule, namespace, tag, or string identifier was invalid.
    InvalidIdentifier,
    /// Canonically unique observations, tags, or ranges contained a duplicate.
    Duplicate,
    /// The matched-data field did not contain the exact zero-byte length marker.
    MatchMarker,
    /// A derived byte range overflowed or escaped the declared artifact.
    Range,
    /// Deterministic result serialization failed.
    Serialization,
}

/// A content-free adapter failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterError(AdapterErrorCode);

impl AdapterError {
    const fn new(code: AdapterErrorCode) -> Self {
        Self(code)
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn code(self) -> AdapterErrorCode {
        self.0
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.0 {
            AdapterErrorCode::ResourceLimit => "YARA-X adapter resource limit exceeded",
            AdapterErrorCode::Framing => "invalid YARA-X NDJSON framing",
            AdapterErrorCode::Utf8 => "invalid YARA-X NDJSON encoding",
            AdapterErrorCode::JsonContract => "invalid YARA-X NDJSON contract",
            AdapterErrorCode::InvalidControl => "invalid YARA-X adapter control metadata",
            AdapterErrorCode::PathMismatch => "YARA-X staged path mismatch",
            AdapterErrorCode::InvalidIdentifier => "invalid YARA-X identifier",
            AdapterErrorCode::Duplicate => "duplicate YARA-X observation data",
            AdapterErrorCode::MatchMarker => "invalid YARA-X zero-byte match marker",
            AdapterErrorCode::Range => "invalid YARA-X evidence range",
            AdapterErrorCode::Serialization => "YARA-X result serialization failed",
        })
    }
}

impl Error for AdapterError {}

/// Exact source-free metadata supplied separately from untrusted vendor output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterControl {
    /// Frozen adapter profile identifier.
    pub profile_id: String,
    /// Exact frozen adapter profile digest.
    pub profile_digest: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact analyzer execution manifest identity.
    pub manifest_id: String,
    /// Exact staged artifact identity.
    pub artifact_hash: String,
    /// Exact staged artifact size in bytes.
    pub artifact_bytes: u64,
    /// Exact staged path expected in the vendor record. It is never emitted.
    pub expected_staged_path: String,
    /// Exact YARA-X executable identity.
    pub executable_digest: String,
    /// Exact compiled ruleset identity.
    pub ruleset_digest: String,
    /// Caller-supplied completion time in canonical UTC form.
    pub completed_at: String,
}

/// One source-free byte range derived from the exact zero-byte vendor marker.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRange {
    /// YARA string identifier.
    pub string_identifier: String,
    /// Decimal byte offset.
    pub offset: String,
    /// Decimal positive byte length.
    pub length: String,
}

/// One path-free, source-free normalized rule observation.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedObservation {
    /// Exact staged artifact identity.
    pub artifact_hash: String,
    /// Vendor namespace.
    pub namespace: String,
    /// Vendor rule identifier.
    pub rule_identifier: String,
    /// Canonically ordered tags.
    pub tags: Vec<String>,
    /// Canonically ordered evidence ranges.
    pub ranges: Vec<EvidenceRange>,
    /// Stable trust classification.
    pub classification: String,
    /// Frozen normalization method.
    pub method: String,
    /// Stable trust statement.
    pub trust: String,
    /// Closed limitations.
    pub limitations: Vec<String>,
    /// Always false.
    pub authority_added: bool,
}

/// Deterministic, source-free result for one original-synthetic artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedResult {
    /// Contract name.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Domain-separated exact result identity.
    pub result_id: String,
    /// Frozen adapter profile identifier.
    pub profile_id: String,
    /// Exact adapter profile digest.
    pub profile_digest: String,
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: String,
    /// Exact analyzer execution manifest identity.
    pub manifest_id: String,
    /// Frozen analyzer identifier.
    pub analyzer_id: String,
    /// Exact staged artifact identity.
    pub artifact_hash: String,
    /// Exact staged artifact size as canonical decimal text.
    pub artifact_bytes: String,
    /// Exact YARA-X executable identity.
    pub executable_digest: String,
    /// Exact compiled ruleset identity.
    pub ruleset_digest: String,
    /// Caller-supplied canonical UTC completion time.
    pub completed_at: String,
    /// Canonically ordered observations.
    pub observations: Vec<NormalizedObservation>,
    /// True only because the complete one-record parse succeeded.
    pub complete_accounting: bool,
    /// Closed limitations.
    pub limitations: Vec<String>,
    /// Frozen fixture provenance.
    pub result_origin: String,
    /// Always false.
    pub raw_output_retained: bool,
    /// Always false.
    pub source_bytes_retained: bool,
    /// Always false.
    pub matched_bytes_retained: bool,
    /// Always false.
    pub path_emitted: bool,
    /// Always false; parsing cannot prove execution.
    pub analyzer_executed: bool,
    /// Always false; parsing cannot prove confinement.
    pub os_confined: bool,
    /// Always false.
    pub production_admitted: bool,
    /// Always false.
    pub iar_2_admitted: bool,
    /// Always false.
    pub safety_claimed: bool,
    /// Always false.
    pub authority_added: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VendorRecord {
    path: String,
    rules: Vec<VendorRule>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VendorRule {
    identifier: String,
    namespace: String,
    strings: Vec<VendorString>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VendorString {
    identifier: String,
    #[serde(rename = "match")]
    matched_marker: String,
    offset: u64,
}

#[derive(Serialize)]
struct ResultIdentity<'a> {
    profile_id: &'a str,
    profile_digest: &'a str,
    workspace_snapshot: &'a str,
    manifest_id: &'a str,
    analyzer_id: &'a str,
    artifact_hash: &'a str,
    artifact_bytes: &'a str,
    executable_digest: &'a str,
    ruleset_digest: &'a str,
    completed_at: &'a str,
    observations: &'a [NormalizedObservation],
    complete_accounting: bool,
    result_origin: &'a str,
}

/// Normalizes one exact bounded YARA-X NDJSON record.
///
/// The function performs no I/O and returns no partial result on error.
///
/// # Errors
///
/// Returns a stable source-free [`AdapterError`] when control metadata,
/// framing, the closed vendor shape, identifiers, markers, ranges, ordering,
/// or a frozen resource ceiling is invalid.
pub fn normalize(raw: &[u8], control: &AdapterControl) -> Result<NormalizedResult, AdapterError> {
    validate_control(control)?;
    let body = validate_framing(raw)?;
    let text = std::str::from_utf8(body).map_err(|_| AdapterError::new(AdapterErrorCode::Utf8))?;
    let record: VendorRecord = serde_json::from_str(text)
        .map_err(|_| AdapterError::new(AdapterErrorCode::JsonContract))?;

    if record.path != control.expected_staged_path {
        return Err(AdapterError::new(AdapterErrorCode::PathMismatch));
    }
    if record.rules.len() > MAX_RULES {
        return Err(AdapterError::new(AdapterErrorCode::ResourceLimit));
    }

    let mut total_ranges = 0usize;
    let mut observations = Vec::with_capacity(record.rules.len());
    for rule in record.rules {
        validate_identifier(&rule.identifier, false)?;
        validate_identifier(&rule.namespace, false)?;
        if rule.tags.len() > MAX_TAGS_PER_OBSERVATION
            || rule.strings.len() > MAX_RANGES_PER_OBSERVATION
        {
            return Err(AdapterError::new(AdapterErrorCode::ResourceLimit));
        }
        if rule.strings.is_empty() {
            return Err(AdapterError::new(AdapterErrorCode::JsonContract));
        }

        let mut tags = rule.tags;
        for tag in &tags {
            validate_identifier(tag, false)?;
        }
        tags.sort_unstable();
        if has_adjacent_duplicate(&tags) {
            return Err(AdapterError::new(AdapterErrorCode::Duplicate));
        }

        let mut ranges = Vec::with_capacity(rule.strings.len());
        for vendor_string in rule.strings {
            validate_identifier(&vendor_string.identifier, true)?;
            let length = parse_match_length(&vendor_string.matched_marker)?;
            let end = vendor_string
                .offset
                .checked_add(length)
                .ok_or_else(|| AdapterError::new(AdapterErrorCode::Range))?;
            if end > control.artifact_bytes {
                return Err(AdapterError::new(AdapterErrorCode::Range));
            }
            ranges.push(EvidenceRange {
                string_identifier: vendor_string.identifier,
                offset: vendor_string.offset.to_string(),
                length: length.to_string(),
            });
        }
        ranges.sort_unstable_by(|left, right| {
            decimal_value(&left.offset)
                .cmp(&decimal_value(&right.offset))
                .then_with(|| left.string_identifier.cmp(&right.string_identifier))
                .then_with(|| decimal_value(&left.length).cmp(&decimal_value(&right.length)))
        });
        if has_adjacent_duplicate(&ranges) {
            return Err(AdapterError::new(AdapterErrorCode::Duplicate));
        }
        total_ranges = total_ranges
            .checked_add(ranges.len())
            .ok_or_else(|| AdapterError::new(AdapterErrorCode::ResourceLimit))?;
        if total_ranges > MAX_TOTAL_RANGES {
            return Err(AdapterError::new(AdapterErrorCode::ResourceLimit));
        }

        observations.push(NormalizedObservation {
            artifact_hash: control.artifact_hash.clone(),
            namespace: rule.namespace,
            rule_identifier: rule.identifier,
            tags,
            ranges,
            classification: "untrusted_derived_data".to_owned(),
            method: "yara_x_ndjson_rule_observation_v1".to_owned(),
            trust: "untrusted_derived_data".to_owned(),
            limitations: vec![
                "original-synthetic-fixture".to_owned(),
                "rule-match-is-not-a-safety-verdict".to_owned(),
            ],
            authority_added: false,
        });
    }

    if observations.len() > MAX_OBSERVATIONS {
        return Err(AdapterError::new(AdapterErrorCode::ResourceLimit));
    }
    observations.sort_unstable();
    if observations.windows(2).any(|pair| {
        pair[0].namespace == pair[1].namespace && pair[0].rule_identifier == pair[1].rule_identifier
    }) {
        return Err(AdapterError::new(AdapterErrorCode::Duplicate));
    }

    build_result(control, observations)
}

fn validate_control(control: &AdapterControl) -> Result<(), AdapterError> {
    if control.profile_id != PROFILE_ID || control.profile_digest != PROFILE_DIGEST {
        return Err(AdapterError::new(AdapterErrorCode::InvalidControl));
    }
    for identity in [
        &control.workspace_snapshot,
        &control.manifest_id,
        &control.artifact_hash,
        &control.executable_digest,
        &control.ruleset_digest,
    ] {
        if !valid_sha256(identity) {
            return Err(AdapterError::new(AdapterErrorCode::InvalidControl));
        }
    }
    let path = control.expected_staged_path.as_bytes();
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path
            .iter()
            .any(|byte| *byte == 0 || byte.is_ascii_control())
        || context_core::validate_utc_timestamp(&control.completed_at).is_err()
    {
        return Err(AdapterError::new(AdapterErrorCode::InvalidControl));
    }
    Ok(())
}

fn validate_framing(raw: &[u8]) -> Result<&[u8], AdapterError> {
    if raw.is_empty() || raw.len() > MAX_INPUT_BYTES || raw.last() != Some(&b'\n') {
        return Err(AdapterError::new(AdapterErrorCode::Framing));
    }
    let body = &raw[..raw.len() - 1];
    if body.is_empty()
        || body.starts_with(&[0xef, 0xbb, 0xbf])
        || body.contains(&b'\n')
        || body.contains(&b'\r')
        || body.first().is_some_and(u8::is_ascii_whitespace)
        || body.last().is_some_and(u8::is_ascii_whitespace)
    {
        return Err(AdapterError::new(AdapterErrorCode::Framing));
    }
    Ok(body)
}

fn validate_identifier(value: &str, string_identifier: bool) -> Result<(), AdapterError> {
    let candidate = if string_identifier {
        value
            .strip_prefix('$')
            .ok_or_else(|| AdapterError::new(AdapterErrorCode::InvalidIdentifier))?
    } else {
        value
    };
    let bytes = candidate.as_bytes();
    if bytes.is_empty()
        || bytes.len() > MAX_IDENTIFIER_BYTES
        || !(bytes[0].is_ascii_alphabetic() || bytes[0] == b'_')
        || !bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
    {
        return Err(AdapterError::new(AdapterErrorCode::InvalidIdentifier));
    }
    Ok(())
}

fn parse_match_length(marker: &str) -> Result<u64, AdapterError> {
    let decimal = marker
        .strip_prefix(" ... ")
        .and_then(|value| value.strip_suffix(" more bytes"))
        .ok_or_else(|| AdapterError::new(AdapterErrorCode::MatchMarker))?;
    if decimal.is_empty()
        || !decimal.bytes().all(|byte| byte.is_ascii_digit())
        || (decimal.len() > 1 && decimal.starts_with('0'))
    {
        return Err(AdapterError::new(AdapterErrorCode::MatchMarker));
    }
    let length = decimal
        .parse::<u64>()
        .map_err(|_| AdapterError::new(AdapterErrorCode::MatchMarker))?;
    if length == 0 {
        return Err(AdapterError::new(AdapterErrorCode::MatchMarker));
    }
    Ok(length)
}

fn build_result(
    control: &AdapterControl,
    observations: Vec<NormalizedObservation>,
) -> Result<NormalizedResult, AdapterError> {
    let artifact_bytes = control.artifact_bytes.to_string();
    let identity = ResultIdentity {
        profile_id: PROFILE_ID,
        profile_digest: PROFILE_DIGEST,
        workspace_snapshot: &control.workspace_snapshot,
        manifest_id: &control.manifest_id,
        analyzer_id: "impresari.yara-x",
        artifact_hash: &control.artifact_hash,
        artifact_bytes: &artifact_bytes,
        executable_digest: &control.executable_digest,
        ruleset_digest: &control.ruleset_digest,
        completed_at: &control.completed_at,
        observations: &observations,
        complete_accounting: true,
        result_origin: "original_synthetic_fixture",
    };
    let serialized = serde_json::to_vec(&identity)
        .map_err(|_| AdapterError::new(AdapterErrorCode::Serialization))?;
    let mut hasher = Sha256::new();
    hasher.update(RESULT_ID_DOMAIN);
    hasher.update(serialized);
    let mut result_id = String::with_capacity(71);
    result_id.push_str("sha256:");
    for byte in hasher.finalize() {
        write!(&mut result_id, "{byte:02x}")
            .map_err(|_| AdapterError::new(AdapterErrorCode::Serialization))?;
    }

    let result = NormalizedResult {
        schema_name: "yara-x-normalized-result".to_owned(),
        schema_version: "1.0.0".to_owned(),
        result_id,
        profile_id: PROFILE_ID.to_owned(),
        profile_digest: PROFILE_DIGEST.to_owned(),
        workspace_snapshot: control.workspace_snapshot.clone(),
        manifest_id: control.manifest_id.clone(),
        analyzer_id: "impresari.yara-x".to_owned(),
        artifact_hash: control.artifact_hash.clone(),
        artifact_bytes,
        executable_digest: control.executable_digest.clone(),
        ruleset_digest: control.ruleset_digest.clone(),
        completed_at: control.completed_at.clone(),
        observations,
        complete_accounting: true,
        limitations: vec![
            "adapter-only".to_owned(),
            "no-live-analyzer".to_owned(),
            "not-a-safety-verdict".to_owned(),
            "original-synthetic-fixture".to_owned(),
        ],
        result_origin: "original_synthetic_fixture".to_owned(),
        raw_output_retained: false,
        source_bytes_retained: false,
        matched_bytes_retained: false,
        path_emitted: false,
        analyzer_executed: false,
        os_confined: false,
        production_admitted: false,
        iar_2_admitted: false,
        safety_claimed: false,
        authority_added: false,
    };
    if serde_json::to_vec(&result)
        .map_err(|_| AdapterError::new(AdapterErrorCode::Serialization))?
        .len()
        > MAX_NORMALIZED_OUTPUT_BYTES
    {
        return Err(AdapterError::new(AdapterErrorCode::ResourceLimit));
    }
    Ok(result)
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn decimal_value(value: &str) -> u64 {
    value.parse().expect("internally generated decimal")
}

fn has_adjacent_duplicate<T: PartialEq>(values: &[T]) -> bool {
    values.windows(2).any(|pair| pair[0] == pair[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &[u8] = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"SyntheticMarker\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$marker\",\"match\":\" ... 12 more bytes\",\"offset\":8}],\"tags\":[\"synthetic\",\"contract\"]}]}\n";
    const NO_MATCH: &[u8] = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n";

    fn control() -> AdapterControl {
        AdapterControl {
            profile_id: PROFILE_ID.to_owned(),
            profile_digest: PROFILE_DIGEST.to_owned(),
            workspace_snapshot: format!("sha256:{}", "d".repeat(64)),
            manifest_id: format!("sha256:{}", "a".repeat(64)),
            artifact_hash: format!("sha256:{}", "1".repeat(64)),
            artifact_bytes: 64,
            expected_staged_path: "/staged/artifact.bin".to_owned(),
            executable_digest: format!("sha256:{}", "b".repeat(64)),
            ruleset_digest: format!("sha256:{}", "c".repeat(64)),
            completed_at: "2026-08-31T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn normalizes_match_without_path_or_source() {
        let result = normalize(VALID, &control()).expect("valid record");
        assert_eq!(result.observations.len(), 1);
        assert_eq!(result.observations[0].tags, ["contract", "synthetic"]);
        assert_eq!(result.observations[0].ranges[0].offset, "8");
        assert_eq!(result.observations[0].ranges[0].length, "12");
        let serialized = serde_json::to_string(&result).expect("serialize result");
        assert!(!serialized.contains("/staged"));
        assert!(!serialized.contains("more bytes"));
        assert!(!serialized.contains("matched_marker"));
        let expected: NormalizedResult = serde_json::from_str(include_str!(
            "../../../tests/conformance/v1/valid/yara-x-normalized-result.json"
        ))
        .expect("valid committed result");
        assert_eq!(result, expected);
    }

    #[test]
    fn accepts_complete_no_match() {
        let result = normalize(NO_MATCH, &control()).expect("valid no-match record");
        assert!(result.observations.is_empty());
        assert!(result.complete_accounting);
    }

    #[test]
    fn result_identity_is_deterministic_across_vendor_order() {
        let reversed = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"SyntheticMarker\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$z\",\"match\":\" ... 2 more bytes\",\"offset\":30},{\"identifier\":\"$marker\",\"match\":\" ... 12 more bytes\",\"offset\":8}],\"tags\":[\"synthetic\",\"contract\"]}]}\n";
        let canonical = b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"SyntheticMarker\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$marker\",\"match\":\" ... 12 more bytes\",\"offset\":8},{\"identifier\":\"$z\",\"match\":\" ... 2 more bytes\",\"offset\":30}],\"tags\":[\"contract\",\"synthetic\"]}]}\n";
        assert_eq!(
            normalize(reversed, &control()).expect("reversed"),
            normalize(canonical, &control()).expect("canonical")
        );
    }

    #[test]
    fn rejects_closed_contract_violations() {
        let cases: &[(&[u8], AdapterErrorCode)] = &[
            (b"{\"path\":\"/other\",\"rules\":[]}\n", AdapterErrorCode::PathMismatch),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[],\"extra\":false}\n", AdapterErrorCode::JsonContract),
            (b"{\"path\":\"/staged/artifact.bin\",\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n", AdapterErrorCode::JsonContract),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n{}\n", AdapterErrorCode::Framing),
            (b" {\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n", AdapterErrorCode::Framing),
            (&[0xff, b'\n'], AdapterErrorCode::Utf8),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                normalize(raw, &control()).expect_err("must fail").code(),
                *expected
            );
        }
    }

    #[test]
    fn rejects_marker_identifier_duplicate_and_range_failures() {
        let cases: &[(&[u8], AdapterErrorCode)] = &[
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"bad-id\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}],\"tags\":[]}]}\n", AdapterErrorCode::InvalidIdentifier),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}],\"tags\":[\"same\",\"same\"]}]}\n", AdapterErrorCode::Duplicate),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\"bytes\",\"offset\":0}],\"tags\":[]}]}\n", AdapterErrorCode::MatchMarker),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 02 more bytes\",\"offset\":0}],\"tags\":[]}]}\n", AdapterErrorCode::MatchMarker),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 2 more bytes\",\"offset\":63}],\"tags\":[]}]}\n", AdapterErrorCode::Range),
        ];
        for (raw, expected) in cases {
            assert_eq!(
                normalize(raw, &control()).expect_err("must fail").code(),
                *expected
            );
        }
    }

    #[test]
    fn malformed_mutations_never_panic() {
        let mut mutations = Vec::new();
        for index in 0..VALID.len() {
            let mut candidate = VALID.to_vec();
            candidate[index] ^= 0xff;
            mutations.push(candidate);
        }
        for candidate in mutations {
            let _ = normalize(&candidate, &control());
        }
    }

    #[test]
    fn rejects_invalid_control_without_echoing_content() {
        let mut invalid = control();
        invalid.profile_digest = format!("sha256:{}", "0".repeat(64));
        let error = normalize(VALID, &invalid).expect_err("profile mismatch");
        assert_eq!(error.code(), AdapterErrorCode::InvalidControl);
        assert!(!error.to_string().contains("sha256:"));

        let mut invalid = control();
        invalid.completed_at = "not-a-time".to_owned();
        assert_eq!(
            normalize(VALID, &invalid).expect_err("invalid time").code(),
            AdapterErrorCode::InvalidControl
        );

        let mut invalid = control();
        invalid.expected_staged_path = "/staged/\0artifact".to_owned();
        assert_eq!(
            normalize(VALID, &invalid).expect_err("invalid path").code(),
            AdapterErrorCode::InvalidControl
        );
    }

    #[test]
    fn rejects_all_frozen_framing_failures() {
        let mut oversized = vec![b'a'; MAX_INPUT_BYTES];
        oversized.push(b'\n');
        let cases: Vec<Vec<u8>> = vec![
            b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}".to_vec(),
            b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}\r\n".to_vec(),
            [
                &[0xef, 0xbb, 0xbf][..],
                b"{\"path\":\"/staged/artifact.bin\",\"rules\":[]}\n",
            ]
            .concat(),
            oversized,
        ];
        for raw in cases {
            assert_eq!(
                normalize(&raw, &control()).expect_err("must fail").code(),
                AdapterErrorCode::Framing
            );
        }
    }

    #[test]
    fn rejects_overflow_duplicate_rules_ranges_and_empty_evidence() {
        let cases: &[(&[u8], AdapterErrorCode)] = &[
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[],\"tags\":[]}]}\n", AdapterErrorCode::JsonContract),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 18446744073709551616 more bytes\",\"offset\":0}],\"tags\":[]}]}\n", AdapterErrorCode::MatchMarker),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 2 more bytes\",\"offset\":18446744073709551615}],\"tags\":[]}]}\n", AdapterErrorCode::Range),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0},{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}],\"tags\":[]}]}\n", AdapterErrorCode::Duplicate),
            (b"{\"path\":\"/staged/artifact.bin\",\"rules\":[{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}],\"tags\":[]},{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$b\",\"match\":\" ... 1 more bytes\",\"offset\":1}],\"tags\":[]}]}\n", AdapterErrorCode::Duplicate),
        ];
        let mut large_control = control();
        large_control.artifact_bytes = u64::MAX;
        for (raw, expected) in cases {
            assert_eq!(
                normalize(raw, &large_control)
                    .expect_err("must fail")
                    .code(),
                *expected
            );
        }
    }

    #[test]
    fn rejects_frozen_collection_ceilings() {
        let one_rule = "{\"identifier\":\"Rule_{index}\",\"namespace\":\"impresari\",\"strings\":[{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}],\"tags\":[]}";
        let rules = (0..=MAX_RULES)
            .map(|index| one_rule.replace("{index}", &index.to_string()))
            .collect::<Vec<_>>()
            .join(",");
        let raw = format!("{{\"path\":\"/staged/artifact.bin\",\"rules\":[{rules}]}}\n");
        assert_eq!(
            normalize(raw.as_bytes(), &control())
                .expect_err("too many rules")
                .code(),
            AdapterErrorCode::ResourceLimit
        );

        let tags = (0..=MAX_TAGS_PER_OBSERVATION)
            .map(|index| format!("\"tag_{index}\""))
            .collect::<Vec<_>>()
            .join(",");
        let raw = format!(
            "{{\"path\":\"/staged/artifact.bin\",\"rules\":[{{\"identifier\":\"Rule\",\"namespace\":\"impresari\",\"strings\":[{{\"identifier\":\"$a\",\"match\":\" ... 1 more bytes\",\"offset\":0}}],\"tags\":[{tags}]}}]}}\n"
        );
        assert_eq!(
            normalize(raw.as_bytes(), &control())
                .expect_err("too many tags")
                .code(),
            AdapterErrorCode::ResourceLimit
        );
    }
}
