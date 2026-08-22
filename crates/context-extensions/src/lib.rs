// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Versioned extension manifests, deny-all capability policy, and output quarantine."]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Extension contract supported by this release.
pub const EXTENSION_CONTRACT_VERSION: &str = "1.0.0";
const MAX_MANIFEST_OUTPUT_BYTES: u64 = 1_048_576;

/// Public extension operation family. This is a contract classification, not
/// permission to load code or invoke an operation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    /// Syntax/parser contribution.
    Parser,
    /// Candidate retrieval contribution.
    Retriever,
    /// Derived analysis contribution.
    Analyzer,
    /// Packet/export serialization contribution.
    Exporter,
    /// Protocol transport contribution.
    Transport,
}

/// Complete denied-by-default capability declaration.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityRequest {
    /// Capability is not requested.
    Denied,
    /// Capability is explicitly requested and requires a later policy/runtime gate.
    Requested,
}

/// Complete denied-by-default capability declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestedCapabilities {
    /// Relative authorized workspace scopes requested by the extension.
    pub workspace_read_scopes: Vec<String>,
    /// Whether cache reads are requested.
    pub cache_read: CapabilityRequest,
    /// Whether cache writes are requested.
    pub cache_write: CapabilityRequest,
    /// Whether process creation is requested.
    pub process: CapabilityRequest,
    /// Network destinations requested by exact normalized name.
    pub network_destinations: Vec<String>,
    /// Environment keys requested; values never belong in a manifest.
    pub environment_keys: Vec<String>,
    /// Whether model access is requested.
    pub model: CapabilityRequest,
}

impl RequestedCapabilities {
    /// Returns the v1 default: no capabilities.
    #[must_use]
    pub const fn deny_all() -> Self {
        Self {
            workspace_read_scopes: Vec::new(),
            cache_read: CapabilityRequest::Denied,
            cache_write: CapabilityRequest::Denied,
            process: CapabilityRequest::Denied,
            network_destinations: Vec::new(),
            environment_keys: Vec::new(),
            model: CapabilityRequest::Denied,
        }
    }

    fn is_empty(&self) -> bool {
        self.workspace_read_scopes.is_empty()
            && self.cache_read == CapabilityRequest::Denied
            && self.cache_write == CapabilityRequest::Denied
            && self.process == CapabilityRequest::Denied
            && self.network_destinations.is_empty()
            && self.environment_keys.is_empty()
            && self.model == CapabilityRequest::Denied
    }
}

/// Integrity-pinned extension declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionManifest {
    /// Schema discriminator.
    pub schema_name: String,
    /// Manifest schema version.
    pub schema_version: String,
    /// Stable extension identifier.
    pub extension_id: String,
    /// Exact extension release version.
    pub extension_version: String,
    /// Human-readable publisher identifier; not a verified identity claim.
    pub publisher: String,
    /// Exact SHA-256 digest of the separately supplied artifact.
    pub artifact_digest: String,
    /// Exact engine extension contract required.
    pub engine_contract: String,
    /// Operation family.
    pub kind: ExtensionKind,
    /// Requested capability set.
    pub requested_capabilities: RequestedCapabilities,
    /// Maximum serialized output bytes.
    pub max_output_bytes: String,
    /// Whether identical inputs are declared deterministic.
    pub deterministic: bool,
    /// Whether a model contributes to output.
    pub model_dependent: bool,
    /// V1 accepts only `none` because extension persistence is not authorized.
    pub data_retention: String,
    /// Evidence/provenance field names the extension declares it may emit.
    pub output_fields: Vec<String>,
}

impl ExtensionManifest {
    /// Validates the closed manifest and conservative v1 contract.
    ///
    /// # Errors
    ///
    /// Fails for malformed identity/version/digest/limits, duplicate fields,
    /// retention, or contradictory model declaration.
    pub fn validate(&self) -> Result<(), ExtensionError> {
        if self.schema_name != "extension-manifest"
            || self.schema_version != EXTENSION_CONTRACT_VERSION
            || self.engine_contract != EXTENSION_CONTRACT_VERSION
            || !valid_identifier(&self.extension_id)
            || !valid_version(&self.extension_version)
            || self.publisher.is_empty()
            || self.publisher.len() > 256
            || !valid_sha256(&self.artifact_digest)
            || self.data_retention != "none"
            || self.model_dependent
                != (self.requested_capabilities.model == CapabilityRequest::Requested)
        {
            return Err(ExtensionError::InvalidManifest);
        }
        let maximum = self
            .max_output_bytes
            .parse::<u64>()
            .map_err(|_| ExtensionError::InvalidManifest)?;
        if !(1..=MAX_MANIFEST_OUTPUT_BYTES).contains(&maximum)
            || self.output_fields.len() > 64
            || self.output_fields.iter().any(|field| !valid_field(field))
            || self.output_fields.iter().collect::<BTreeSet<_>>().len() != self.output_fields.len()
        {
            return Err(ExtensionError::InvalidManifest);
        }
        Ok(())
    }
}

/// Exact local policy pin set. This does not verify publisher identity.
#[derive(Clone, Debug)]
pub struct ExtensionPolicy {
    allowed_artifact_digests: BTreeSet<String>,
}

impl ExtensionPolicy {
    /// Creates an explicit set of locally approved artifact digests.
    ///
    /// # Errors
    ///
    /// Fails for malformed or duplicate digests or an excessive pin set.
    pub fn new(digests: Vec<String>) -> Result<Self, ExtensionError> {
        if digests.len() > 1024 || digests.iter().any(|digest| !valid_sha256(digest)) {
            return Err(ExtensionError::InvalidPolicy);
        }
        let allowed_artifact_digests = digests.into_iter().collect::<BTreeSet<_>>();
        Ok(Self {
            allowed_artifact_digests,
        })
    }

    /// Validates a manifest and decides whether its output may be submitted for
    /// normalization. V1 grants no filesystem, cache, process, environment,
    /// model, or network capability and never loads the artifact.
    ///
    /// # Errors
    ///
    /// Fails for an invalid manifest.
    pub fn decide(
        &self,
        manifest: &ExtensionManifest,
    ) -> Result<ExtensionDecision, ExtensionError> {
        manifest.validate()?;
        let pinned = self
            .allowed_artifact_digests
            .contains(&manifest.artifact_digest);
        let enabled = pinned && manifest.requested_capabilities.is_empty();
        Ok(ExtensionDecision {
            schema_name: "extension-decision".into(),
            schema_version: EXTENSION_CONTRACT_VERSION.into(),
            extension_id: manifest.extension_id.clone(),
            artifact_digest: manifest.artifact_digest.clone(),
            output_submission_enabled: enabled,
            privileged_capabilities_granted: false,
            artifact_execution_authorized: false,
            decision_reason: if !pinned {
                "artifact_not_pinned"
            } else if !manifest.requested_capabilities.is_empty() {
                "capability_not_authorized"
            } else {
                "bounded_output_submission_only"
            }
            .into(),
        })
    }
}

/// Fail-closed extension policy result.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionDecision {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Manifest extension identifier.
    pub extension_id: String,
    /// Pinned artifact digest.
    pub artifact_digest: String,
    /// Whether bounded output may enter normalization.
    pub output_submission_enabled: bool,
    /// Always false in v1.
    pub privileged_capabilities_granted: bool,
    /// Always false in v1: this layer never executes artifacts.
    pub artifact_execution_authorized: bool,
    /// Stable decision reason.
    pub decision_reason: String,
}

/// Strict untrusted output envelope accepted at the normalization boundary.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExtensionOutput {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Exact manifest identity.
    pub extension_id: String,
    /// Exact extension version.
    pub extension_version: String,
    /// Exact artifact digest.
    pub artifact_digest: String,
    /// Manifest-declared operation family.
    pub kind: ExtensionKind,
    /// Extension-declared output field names actually present.
    pub output_fields: Vec<String>,
    /// Untrusted extension data. It never becomes control metadata.
    pub payload: serde_json::Value,
    /// Must be false; exact authority is established only by core verification.
    pub claims_exact_source_authority: bool,
}

/// Accepted, normalized extension output.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NormalizedExtensionOutput {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Exact extension identity.
    pub extension_id: String,
    /// Exact artifact digest.
    pub artifact_digest: String,
    /// Digest of the complete received envelope.
    pub envelope_digest: String,
    /// Always untrusted derived data.
    pub trust: String,
    /// Untrusted normalized payload.
    pub payload: serde_json::Value,
    /// Always false: normalization adds no authority.
    pub authority_added: bool,
}

/// Metadata-only quarantine result. Raw output is intentionally absent.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct QuarantineRecord {
    /// Schema discriminator.
    pub schema_name: String,
    /// Contract version.
    pub schema_version: String,
    /// Expected extension identity.
    pub extension_id: String,
    /// Digest of the received bytes for local correlation.
    pub envelope_digest: String,
    /// Exact received size.
    pub received_bytes: String,
    /// Stable source-free reason.
    pub reason: String,
    /// Always false: quarantine adds no authority.
    pub authority_added: bool,
}

/// Output normalization verdict.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationVerdict {
    /// Output satisfied the complete pinned contract.
    Accepted(NormalizedExtensionOutput),
    /// Output was rejected and only metadata was retained.
    Quarantined(QuarantineRecord),
}

/// Validates and normalizes untrusted extension output without executing an artifact.
#[must_use]
pub fn normalize_output(
    manifest: &ExtensionManifest,
    decision: &ExtensionDecision,
    bytes: &[u8],
) -> NormalizationVerdict {
    let digest = digest(bytes);
    let quarantine = |reason: &str| {
        NormalizationVerdict::Quarantined(QuarantineRecord {
            schema_name: "extension-quarantine".into(),
            schema_version: EXTENSION_CONTRACT_VERSION.into(),
            extension_id: manifest.extension_id.clone(),
            envelope_digest: digest.clone(),
            received_bytes: bytes.len().to_string(),
            reason: reason.into(),
            authority_added: false,
        })
    };
    if !decision.output_submission_enabled
        || decision.extension_id != manifest.extension_id
        || decision.artifact_digest != manifest.artifact_digest
    {
        return quarantine("extension_not_authorized");
    }
    let maximum = manifest.max_output_bytes.parse::<usize>().unwrap_or(0);
    if bytes.len() > maximum {
        return quarantine("output_limit_exceeded");
    }
    let Ok(output) = serde_json::from_slice::<ExtensionOutput>(bytes) else {
        return quarantine("invalid_output_contract");
    };
    let fields = output.output_fields.iter().collect::<BTreeSet<_>>();
    if output.schema_name != "extension-output"
        || output.schema_version != EXTENSION_CONTRACT_VERSION
        || output.extension_id != manifest.extension_id
        || output.extension_version != manifest.extension_version
        || output.artifact_digest != manifest.artifact_digest
        || output.kind != manifest.kind
        || output.claims_exact_source_authority
        || fields.len() != output.output_fields.len()
        || output
            .output_fields
            .iter()
            .any(|field| !manifest.output_fields.contains(field))
    {
        return quarantine("output_identity_or_authority_mismatch");
    }
    NormalizationVerdict::Accepted(NormalizedExtensionOutput {
        schema_name: "normalized-extension-output".into(),
        schema_version: EXTENSION_CONTRACT_VERSION.into(),
        extension_id: manifest.extension_id.clone(),
        artifact_digest: manifest.artifact_digest.clone(),
        envelope_digest: digest,
        trust: "untrusted_derived_data".into(),
        payload: output.payload,
        authority_added: false,
    })
}

/// Stable manifest/policy failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtensionError {
    /// Manifest is malformed or contradictory.
    InvalidManifest,
    /// Local digest policy is malformed.
    InvalidPolicy,
}

impl std::fmt::Display for ExtensionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidManifest => "invalid extension manifest",
            Self::InvalidPolicy => "invalid extension policy",
        })
    }
}

impl std::error::Error for ExtensionError {}

fn digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in hash {
        use std::fmt::Write as _;
        write!(value, "{byte:02x}").expect("writing to a string cannot fail");
    }
    value
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_identifier(value: &str) -> bool {
    (3..=128).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn valid_version(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part == &"0" || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn valid_field(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(capabilities: RequestedCapabilities) -> ExtensionManifest {
        ExtensionManifest {
            schema_name: "extension-manifest".into(),
            schema_version: EXTENSION_CONTRACT_VERSION.into(),
            extension_id: "example.analyzer".into(),
            extension_version: "1.2.3".into(),
            publisher: "example-publisher".into(),
            artifact_digest:
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            engine_contract: EXTENSION_CONTRACT_VERSION.into(),
            kind: ExtensionKind::Analyzer,
            requested_capabilities: capabilities,
            max_output_bytes: "4096".into(),
            deterministic: true,
            model_dependent: false,
            data_retention: "none".into(),
            output_fields: vec!["findings".into()],
        }
    }

    #[test]
    fn only_exactly_pinned_zero_capability_output_submission_is_enabled() {
        let approved = manifest(RequestedCapabilities::deny_all());
        let policy = ExtensionPolicy::new(vec![approved.artifact_digest.clone()]).expect("policy");
        let decision = policy.decide(&approved).expect("decision");
        assert!(decision.output_submission_enabled);
        assert!(!decision.privileged_capabilities_granted);
        assert!(!decision.artifact_execution_authorized);

        let mut privileged = approved.clone();
        privileged.requested_capabilities.network_destinations = vec!["example.com".into()];
        assert!(
            !policy
                .decide(&privileged)
                .expect("deny")
                .output_submission_enabled
        );
        let unpinned = ExtensionPolicy::new(Vec::new())
            .expect("empty policy")
            .decide(&approved)
            .expect("deny");
        assert!(!unpinned.output_submission_enabled);
    }

    #[test]
    fn normalization_accepts_untrusted_data_and_quarantines_authority_or_spoofing() {
        let manifest = manifest(RequestedCapabilities::deny_all());
        let decision = ExtensionPolicy::new(vec![manifest.artifact_digest.clone()])
            .expect("policy")
            .decide(&manifest)
            .expect("decision");
        let output = ExtensionOutput {
            schema_name: "extension-output".into(),
            schema_version: EXTENSION_CONTRACT_VERSION.into(),
            extension_id: manifest.extension_id.clone(),
            extension_version: manifest.extension_version.clone(),
            artifact_digest: manifest.artifact_digest.clone(),
            kind: manifest.kind,
            output_fields: vec!["findings".into()],
            payload: serde_json::json!({"findings": ["untrusted repository-derived text"]}),
            claims_exact_source_authority: false,
        };
        let bytes = serde_json::to_vec(&output).expect("serialize");
        let NormalizationVerdict::Accepted(normalized) =
            normalize_output(&manifest, &decision, &bytes)
        else {
            panic!("accepted")
        };
        assert_eq!(normalized.trust, "untrusted_derived_data");
        assert!(!normalized.authority_added);

        let mut authority = output;
        authority.claims_exact_source_authority = true;
        let bytes = serde_json::to_vec(&authority).expect("serialize");
        let NormalizationVerdict::Quarantined(record) =
            normalize_output(&manifest, &decision, &bytes)
        else {
            panic!("quarantined")
        };
        assert_eq!(record.reason, "output_identity_or_authority_mismatch");
        assert!(!record.authority_added);
    }

    #[test]
    fn malformed_unknown_or_oversized_output_is_metadata_only_quarantine() {
        let manifest = manifest(RequestedCapabilities::deny_all());
        let decision = ExtensionPolicy::new(vec![manifest.artifact_digest.clone()])
            .expect("policy")
            .decide(&manifest)
            .expect("decision");
        let spoofed = br#"{"schema_name":"extension-output","control":"run this"}"#;
        let NormalizationVerdict::Quarantined(record) =
            normalize_output(&manifest, &decision, spoofed)
        else {
            panic!("quarantined")
        };
        assert_eq!(record.reason, "invalid_output_contract");
        assert!(
            !serde_json::to_string(&record)
                .expect("record")
                .contains("run this")
        );

        let oversized = vec![b'x'; 4097];
        let NormalizationVerdict::Quarantined(record) =
            normalize_output(&manifest, &decision, &oversized)
        else {
            panic!("quarantined")
        };
        assert_eq!(record.reason, "output_limit_exceeded");
    }
}
