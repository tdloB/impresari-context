// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Full Draft 2020-12 validation of the published conformance manifest."]

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::PathBuf,
};

use jsonschema::Registry;
use serde_json::Value;
use sha2::{Digest, Sha256};

use context_core::{
    AuditOutcome, Capability, ContextPacket, EvidenceArtifact, EvidenceExcerpt, EvidenceExtraction,
    EvidencePath, EvidenceRecord, EvidenceSpan, PacketDraft, PolicySubject, ResourceBudget,
    audit_event, build_packet, decide, packet_bytes, packet_validation_result,
};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root should exist during tests")
}

fn lowercase_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().fold(String::new(), |mut hex, byte| {
        write!(hex, "{byte:02x}").expect("writing to a string cannot fail");
        hex
    })
}

fn read_json(path: &PathBuf) -> Value {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn conformance_packet(snapshot: &str, other_hash: &str) -> ContextPacket {
    let evidence = EvidenceRecord {
        schema_name: "evidence".into(),
        schema_version: "1.0.0".into(),
        evidence_id: other_hash.into(),
        workspace_snapshot: snapshot.into(),
        artifact: EvidenceArtifact {
            path: EvidencePath {
                display_path: "a.rs".into(),
                platform_family: "unix".into(),
                unit_encoding: "unix_bytes".into(),
                relative_units_base64url: "YS5ycw".into(),
            },
            content_hash: other_hash.into(),
            file_kind: "regular_file".into(),
            decoding: "utf8".into(),
        },
        span: EvidenceSpan {
            start_byte: "0".into(),
            end_byte: "1".into(),
        },
        excerpt: EvidenceExcerpt {
            encoding: "base64url".into(),
            bytes_base64url: "eA".into(),
            match_start_byte: "0".into(),
            match_end_byte: "1".into(),
        },
        kind: "exact_source".into(),
        extraction: EvidenceExtraction {
            method: "literal_search".into(),
            version: "1.0.0".into(),
        },
        confidence: "confirmed".into(),
        trust: "untrusted_workspace_content".into(),
        freshness: "current".into(),
        sensitivity: Some("normal".into()),
    };
    build_packet(PacketDraft {
        workspace_identity: other_hash.into(),
        workspace_snapshot: snapshot.into(),
        request_id: "req_12345678".into(),
        purpose: "conformance".into(),
        created_at: "2026-08-21T00:00:00Z".into(),
        policy_decision: other_hash.into(),
        budget: ResourceBudget::conservative(4096, 10, 10, 128, 100, 32, 30_000, 536_870_912)
            .expect("budget"),
        evidence: vec![evidence],
        assumptions: Vec::new(),
        conflicts: Vec::new(),
        unknowns: Vec::new(),
        redactions: Vec::new(),
    })
    .expect("build packet")
}

#[test]
fn declared_fixture_verdicts_match_draft_2020_12() {
    let root = repository_root();
    let schema_root = root.join("schemas/v1");
    let fixture_root = root.join("tests/conformance/v1");
    let registry_document = read_json(&schema_root.join("registry.json"));
    let mut schemas = BTreeMap::new();
    let mut registry = Registry::new();

    for entry in registry_document["schemas"]
        .as_array()
        .expect("registry schemas should be an array")
    {
        let path = entry["path"]
            .as_str()
            .expect("schema path should be a string");
        if !path.ends_with(".schema.json") {
            continue;
        }
        let schema = read_json(&schema_root.join(path));
        let id = schema["$id"].as_str().expect("schema should declare $id");
        registry = registry
            .add(id, schema.clone())
            .unwrap_or_else(|error| panic!("failed to register {path}: {error}"));
        schemas.insert(path.to_owned(), schema);
    }
    let registry = registry
        .prepare()
        .expect("schema registry should prepare offline");
    let manifest = read_json(&fixture_root.join("manifest.json"));

    for case in manifest["cases"]
        .as_array()
        .expect("manifest cases should be an array")
    {
        let fixture = case["fixture"]
            .as_str()
            .expect("fixture path should be a string");
        let schema_ref = case["schema"]
            .as_str()
            .expect("schema reference should be a string");
        let expected = case["valid"].as_bool().expect("valid should be a boolean");
        let (schema_path, fragment) = schema_ref.split_once('#').unwrap_or((schema_ref, ""));
        let schema = schemas
            .get(schema_path)
            .expect("manifest schema should be registered");
        let wrapper = if fragment.is_empty() {
            schema.clone()
        } else {
            let id = schema["$id"].as_str().expect("schema should declare $id");
            serde_json::json!({"$ref": format!("{id}#{fragment}")})
        };
        let validator = jsonschema::draft202012::options()
            .with_registry(&registry)
            .should_validate_formats(true)
            .build(&wrapper)
            .unwrap_or_else(|error| panic!("failed to compile {schema_ref}: {error}"));
        let instance = read_json(&fixture_root.join(fixture));
        let actual = validator.is_valid(&instance);
        assert_eq!(actual, expected, "unexpected schema verdict for {fixture}");
    }
}

#[test]
fn identity_vectors_reproduce_exact_preimages_and_digests() {
    let root = repository_root();
    let document = read_json(&root.join("tests/conformance/v1/identity-vectors.json"));
    let vectors = document["vectors"]
        .as_array()
        .expect("vectors should be an array");

    assert_eq!(
        vectors.len(),
        10,
        "every initial object kind needs one vector"
    );
    for vector in vectors {
        let object_kind = vector["object_kind"].as_str().expect("object kind");
        let payload = vector["canonical_payload"]
            .as_str()
            .expect("canonical payload");
        let preimage = ["impresari-context", object_kind, "1.0.0", payload].join("\0");
        assert_eq!(lowercase_hex(preimage.as_bytes()), vector["preimage_hex"]);
        let digest = format!(
            "sha256:{}",
            lowercase_hex(Sha256::digest(preimage.as_bytes()))
        );
        assert_eq!(
            digest, vector["digest"],
            "digest mismatch for {object_kind}"
        );
    }
}

#[test]
fn resource_profile_is_valid_ordered_and_fingerprinted() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/conservative-local-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("resource profile should be readable");
    let profile: Value = serde_json::from_slice(&profile_bytes).expect("resource profile JSON");
    let schema = read_json(&root.join("schemas/v1/resource-policy-profile.schema.json"));
    let mut registry = Registry::new();
    let common = read_json(&root.join("schemas/v1/common.schema.json"));
    let common_id = common["$id"].as_str().expect("common schema id").to_owned();
    registry = registry
        .add(common_id, common)
        .expect("register common schema");
    let registry = registry.prepare().expect("prepare profile registry");
    let validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .build(&schema)
        .expect("compile resource profile schema");
    assert!(
        validator.is_valid(&profile),
        "resource profile must satisfy its schema"
    );

    for (name, range) in profile["limits"].as_object().expect("profile limits") {
        let parse = |field: &str| {
            range[field]
                .as_str()
                .unwrap_or_else(|| panic!("{name}.{field} must be a string"))
                .parse::<u64>()
                .unwrap_or_else(|error| panic!("{name}.{field}: {error}"))
        };
        let (minimum, default, maximum) = (parse("minimum"), parse("default"), parse("maximum"));
        assert!(
            minimum <= default && default <= maximum,
            "unordered range: {name}"
        );
    }

    let sidecar = fs::read_to_string(root.join("profiles/v1/conservative-local-v1.sha256"))
        .expect("profile digest sidecar");
    let expected = sidecar.split_whitespace().next().expect("sidecar digest");
    assert_eq!(lowercase_hex(Sha256::digest(profile_bytes)), expected);
}

#[test]
fn resource_policy_boundary_vectors_match_the_profile() {
    let root = repository_root();
    let profile = read_json(&root.join("profiles/v1/conservative-local-v1.json"));
    let vectors = read_json(&root.join("tests/conformance/v1/resource-policy-boundaries.json"));

    for case in vectors["cases"].as_array().expect("boundary cases") {
        let name = case["limit"].as_str().expect("limit name");
        let value = case["value"]
            .as_str()
            .expect("limit value")
            .parse::<u64>()
            .expect("u64 value");
        let range = &profile["limits"][name];
        let minimum = range["minimum"]
            .as_str()
            .expect("minimum")
            .parse::<u64>()
            .expect("u64 minimum");
        let maximum = range["maximum"]
            .as_str()
            .expect("maximum")
            .parse::<u64>()
            .expect("u64 maximum");
        let actual = if value < minimum {
            "below_minimum"
        } else if value > maximum {
            "above_maximum"
        } else {
            "allowed"
        };
        assert_eq!(
            actual, case["verdict"],
            "boundary verdict mismatch for {name}={value}"
        );
    }
}

#[test]
fn hostile_repository_contract_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/hra-static-contract-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("HRA profile should be readable");
    let sidecar = fs::read_to_string(root.join("profiles/v1/hra-static-contract-v1.sha256"))
        .expect("HRA profile digest sidecar");
    let expected = sidecar
        .split_whitespace()
        .next()
        .expect("HRA sidecar digest");
    assert_eq!(lowercase_hex(Sha256::digest(&profile_bytes)), expected);

    let fixture_profile =
        fs::read(root.join("tests/conformance/v1/valid/hra-static-contract-profile.json"))
            .expect("HRA profile fixture");
    assert_eq!(
        profile_bytes, fixture_profile,
        "profile fixture must be exact"
    );

    let provenance =
        read_json(&root.join("tests/conformance/v1/hostile-repository-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(
            provenance[prohibited], false,
            "prohibited provenance: {prohibited}"
        );
    }
    for case in provenance["cases"].as_array().expect("provenance cases") {
        assert_eq!(case["origin"], "original_synthetic");
        assert_eq!(case["license"], "Apache-2.0");
        let relative = case["path"].as_str().expect("fixture path");
        assert!(!relative.starts_with('/') && !relative.contains(".."));
        let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
            .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
        assert_eq!(
            lowercase_hex(Sha256::digest(bytes)),
            case["sha256"].as_str().expect("fixture digest"),
            "fixture provenance digest mismatch for {relative}"
        );
    }

    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("provenance cases")
        .iter()
        .map(|case| case["path"].as_str().expect("provenance path"))
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let hra_schemas = [
        "security-artifact-inventory.schema.json",
        "security-finding.schema.json",
        "analyzer-coverage.schema.json",
        "analyzer-result-envelope.schema.json",
        "repository-security-assessment.schema.json",
        "repository-admission-policy.schema.json",
        "repository-admission-decision.schema.json",
        "hostile-repository-resource-profile.schema.json",
    ];
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            hra_schemas
                .iter()
                .any(|schema| case["schema"].as_str().expect("schema").starts_with(schema))
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        provenance_paths, declared_paths,
        "every HRA conformance fixture needs reviewed provenance"
    );
}

#[test]
fn analyzer_runner_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-protocol-synthetic-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("IAR-0 profile should be readable");
    let sidecar = fs::read_to_string(root.join("profiles/v1/iar-protocol-synthetic-v1.sha256"))
        .expect("IAR-0 profile digest sidecar");
    let expected = sidecar
        .split_whitespace()
        .next()
        .expect("IAR-0 sidecar digest");
    assert_eq!(lowercase_hex(Sha256::digest(&profile_bytes)), expected);
    assert_eq!(
        profile_bytes,
        fs::read(root.join("tests/conformance/v1/valid/iar-protocol-synthetic-profile.json"))
            .expect("IAR-0 profile fixture"),
        "IAR-0 profile fixture must be exact"
    );

    let provenance =
        read_json(&root.join("tests/conformance/v1/analyzer-runner-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(
            provenance[prohibited], false,
            "prohibited provenance: {prohibited}"
        );
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest"),
                "fixture provenance digest mismatch for {relative}"
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let iar_schemas = [
        "analyzer-runner-capability.schema.json",
        "analyzer-execution-manifest.schema.json",
        "analyzer-runner-request.schema.json",
        "analyzer-runner-result.schema.json",
        "analyzer-runner-failure.schema.json",
        "analyzer-runner-resource-profile.schema.json",
    ];
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            iar_schemas
                .iter()
                .any(|schema| case["schema"].as_str().expect("schema").starts_with(schema))
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        provenance_paths, declared_paths,
        "every IAR-0 conformance fixture needs reviewed provenance"
    );
}

#[test]
fn analyzer_supervisor_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-application-supervisor-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("IAR-1 profile");
    let sidecar = fs::read_to_string(root.join("profiles/v1/iar-application-supervisor-v1.sha256"))
        .expect("IAR-1 profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(root.join("tests/conformance/v1/valid/iar-application-supervisor-profile.json"))
            .expect("IAR-1 profile fixture")
    );

    let provenance =
        read_json(&root.join("tests/conformance/v1/analyzer-supervisor-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema"),
                "analyzer-supervisor-resource-profile.schema.json"
                    | "analyzer-supervisor-confinement.schema.json"
                    | "analyzer-supervisor-audit.schema.json"
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_xpc_contract_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-xpc-hybrid-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS XPC profile");
    let sidecar = fs::read_to_string(root.join("profiles/v1/iar-macos-xpc-hybrid-v1.sha256"))
        .expect("macOS XPC profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(root.join("tests/conformance/v1/valid/iar-macos-xpc-hybrid-profile.json"))
            .expect("macOS XPC profile fixture")
    );
    let provenance =
        read_json(&root.join("tests/conformance/v1/macos-xpc-sandbox-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS XPC provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-xpc-sandbox-feasibility.schema.json"
                        | "macos-xpc-resource-profile.schema.json"
                        | "macos-xpc-launch.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_local_vm_contract_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-local-vm-feasibility-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS local VM profile");
    let sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-macos-local-vm-feasibility-v1.sha256"))
            .expect("macOS local VM profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(
            root.join("tests/conformance/v1/valid/iar-macos-local-vm-feasibility-profile.json"),
        )
        .expect("macOS local VM profile fixture")
    );
    let provenance =
        read_json(&root.join("tests/conformance/v1/macos-local-vm-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS local VM provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-local-vm-resource-profile.schema.json"
                        | "macos-local-vm-feasibility.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_local_vm_matrix_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-local-vm-synthetic-matrix-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS local VM matrix profile");
    let sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-macos-local-vm-synthetic-matrix-v1.sha256"))
            .expect("macOS local VM matrix profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(
            root.join(
                "tests/conformance/v1/valid/iar-macos-local-vm-synthetic-matrix-profile.json",
            )
        )
        .expect("macOS local VM matrix profile fixture")
    );
    let provenance =
        read_json(&root.join("tests/conformance/v1/macos-local-vm-matrix-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS local VM matrix provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-local-vm-matrix-resource-profile.schema.json"
                        | "macos-local-vm-matrix-job.schema.json"
                        | "macos-local-vm-matrix-failure.schema.json"
                        | "macos-local-vm-synthetic-matrix.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_local_vm_supervisor_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-local-vm-supervisor-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS local VM supervisor profile");
    let sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-macos-local-vm-supervisor-v1.sha256"))
            .expect("macOS local VM supervisor profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(
            root.join("tests/conformance/v1/valid/iar-macos-local-vm-supervisor-profile.json")
        )
        .expect("macOS local VM supervisor profile fixture")
    );
    let provenance = read_json(
        &root.join("tests/conformance/v1/macos-local-vm-supervisor-fixture-provenance.json"),
    );
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS local VM supervisor provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-local-vm-supervisor-resource-profile.schema.json"
                        | "macos-local-vm-supervisor-lifecycle.schema.json"
                        | "macos-local-vm-supervisor-matrix.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_local_vm_resource_canary_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-local-vm-resource-canary-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS local VM resource/canary profile");
    let sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-macos-local-vm-resource-canary-v1.sha256"))
            .expect("macOS local VM resource/canary profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(
            root.join(
                "tests/conformance/v1/valid/iar-macos-local-vm-resource-canary-profile.json",
            )
        )
        .expect("macOS local VM resource/canary profile fixture")
    );
    let provenance = read_json(
        &root.join("tests/conformance/v1/macos-local-vm-resource-canary-fixture-provenance.json"),
    );
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS local VM resource/canary provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-local-vm-resource-canary-profile.schema.json"
                        | "macos-local-vm-resource-canary-receipt.schema.json"
                        | "macos-local-vm-resource-canary-failure.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn macos_local_vm_interruption_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-macos-local-vm-interruption-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("macOS local VM interruption profile");
    let sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-macos-local-vm-interruption-v1.sha256"))
            .expect("macOS local VM interruption profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(
            root.join("tests/conformance/v1/valid/iar-macos-local-vm-interruption-profile.json")
        )
        .expect("macOS local VM interruption profile fixture")
    );
    let provenance = read_json(
        &root.join("tests/conformance/v1/macos-local-vm-interruption-fixture-provenance.json"),
    );
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("macOS local VM interruption provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "macos-local-vm-interruption-profile.schema.json"
                        | "macos-local-vm-host-interruption.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn linux_isolation_contract_profile_and_fixture_provenance_are_frozen() {
    let root = repository_root();
    let profile_path = root.join("profiles/v1/iar-linux-synthetic-v1.json");
    let profile_bytes = fs::read(&profile_path).expect("Linux isolation profile");
    let sidecar = fs::read_to_string(root.join("profiles/v1/iar-linux-synthetic-v1.sha256"))
        .expect("Linux isolation profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&profile_bytes)),
        sidecar.split_whitespace().next().expect("profile digest")
    );
    assert_eq!(
        profile_bytes,
        fs::read(root.join("tests/conformance/v1/valid/iar-linux-synthetic-profile.json"))
            .expect("Linux isolation profile fixture")
    );
    let cgroup_profile_path = root.join("profiles/v1/iar-linux-cgroup-synthetic-v1.json");
    let cgroup_profile_bytes = fs::read(&cgroup_profile_path).expect("Linux cgroup profile");
    let cgroup_sidecar =
        fs::read_to_string(root.join("profiles/v1/iar-linux-cgroup-synthetic-v1.sha256"))
            .expect("Linux cgroup profile digest sidecar");
    assert_eq!(
        lowercase_hex(Sha256::digest(&cgroup_profile_bytes)),
        cgroup_sidecar
            .split_whitespace()
            .next()
            .expect("cgroup profile digest")
    );
    assert_eq!(
        cgroup_profile_bytes,
        fs::read(root.join("tests/conformance/v1/valid/iar-linux-cgroup-synthetic-profile.json"),)
            .expect("Linux cgroup profile fixture")
    );
    let provenance =
        read_json(&root.join("tests/conformance/v1/linux-isolation-fixture-provenance.json"));
    assert_eq!(
        provenance["review_status"],
        "approved_original_synthetic_only"
    );
    for prohibited in [
        "contains_executable_artifacts",
        "contains_malware_or_live_signatures",
        "contains_third_party_source",
        "contains_private_or_customer_source",
        "network_or_provider_data_used",
    ] {
        assert_eq!(provenance[prohibited], false);
    }
    let provenance_paths = provenance["cases"]
        .as_array()
        .expect("Linux isolation provenance cases")
        .iter()
        .map(|case| {
            assert_eq!(case["origin"], "original_synthetic");
            assert_eq!(case["license"], "Apache-2.0");
            let relative = case["path"].as_str().expect("fixture path");
            assert!(!relative.starts_with('/') && !relative.contains(".."));
            let bytes = fs::read(root.join("tests/conformance/v1").join(relative))
                .unwrap_or_else(|error| panic!("provenance fixture {relative}: {error}"));
            assert_eq!(
                lowercase_hex(Sha256::digest(bytes)),
                case["sha256"].as_str().expect("fixture digest")
            );
            relative
        })
        .collect::<BTreeSet<_>>();
    let conformance = read_json(&root.join("tests/conformance/v1/manifest.json"));
    let declared_paths = conformance["cases"]
        .as_array()
        .expect("conformance cases")
        .iter()
        .filter(|case| {
            matches!(
                case["schema"].as_str().expect("schema").split('#').next(),
                Some(
                    "linux-isolation-resource-profile.schema.json"
                        | "linux-isolation-feasibility.schema.json"
                        | "linux-isolation-maintenance.schema.json"
                        | "linux-isolation-production-topology.schema.json"
                        | "linux-rootless-host-preflight.schema.json"
                        | "linux-rootless-user-manager-rehearsal.schema.json"
                        | "linux-external-delegation-capability.schema.json"
                        | "linux-external-delegation-live-rehearsal.schema.json"
                        | "linux-isolation-production-lifecycle.schema.json"
                        | "linux-isolation-package-lifecycle-rehearsal.schema.json"
                        | "linux-cgroup-resource-profile.schema.json"
                        | "linux-cgroup-feasibility.schema.json"
                )
            )
        })
        .map(|case| case["fixture"].as_str().expect("fixture"))
        .collect::<BTreeSet<_>>();
    assert_eq!(provenance_paths, declared_paths);
}

#[test]
fn rust_packet_output_satisfies_the_published_schema() {
    let root = repository_root();
    let schema_root = root.join("schemas/v1");
    let registry_document = read_json(&schema_root.join("registry.json"));
    let mut registry = Registry::new();
    for entry in registry_document["schemas"].as_array().expect("schemas") {
        let path = entry["path"].as_str().expect("schema path");
        if path.ends_with(".schema.json") {
            let schema = read_json(&schema_root.join(path));
            let id = schema["$id"].as_str().expect("schema id").to_owned();
            registry = registry.add(id, schema).expect("register schema");
        }
    }
    let registry = registry.prepare().expect("prepare registry");
    let schema = read_json(&schema_root.join("context-packet.schema.json"));
    let validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .should_validate_formats(true)
        .build(&schema)
        .expect("packet validator");
    let hash_a = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let hash_b = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let packet = conformance_packet(hash_a, hash_b);
    let value: Value = serde_json::from_slice(&packet_bytes(&packet).expect("canonical packet"))
        .expect("packet JSON");
    validator.validate(&value).unwrap_or_else(|error| {
        panic!("Rust packet must satisfy context-packet.schema.json: {error}")
    });

    let validation =
        packet_validation_result(&packet, true, Some(hash_a), true, "2026-08-21T00:00:01Z")
            .expect("validation result");
    let validation_schema = read_json(&schema_root.join("packet-validation.schema.json"));
    let validation_validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .should_validate_formats(true)
        .build(&validation_schema)
        .expect("validation-result validator");
    let validation_value = serde_json::to_value(validation).expect("validation JSON");
    validation_validator
        .validate(&validation_value)
        .unwrap_or_else(|error| {
            panic!("Rust result must satisfy packet-validation.schema.json: {error}")
        });

    let audit = audit_event(
        "evt_12345678",
        "req_12345678",
        "2026-08-21T00:00:02Z",
        Some(hash_a),
        Some(hash_a),
        Capability::ContextValidate,
        AuditOutcome::Allowed,
        hash_b,
        packet.budget.clone(),
        3,
        "0.1.0",
    )
    .expect("audit event");
    let audit_schema = read_json(&schema_root.join("audit-event.schema.json"));
    let audit_validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .should_validate_formats(true)
        .build(&audit_schema)
        .expect("audit validator");
    audit_validator
        .validate(&serde_json::to_value(audit).expect("audit JSON"))
        .unwrap_or_else(|error| panic!("Rust event must satisfy audit-event.schema.json: {error}"));

    let decision = decide(
        "req_12345678",
        &PolicySubject {
            caller_id: "caller_12345678".into(),
            role: "local_user".into(),
            purpose: "conformance".into(),
        },
        Some(hash_a),
        Capability::ContextValidate,
        Some(packet.budget),
        "2026-08-21T00:00:03Z",
    )
    .expect("policy decision");
    let decision_schema = read_json(&schema_root.join("policy-decision.schema.json"));
    let decision_validator = jsonschema::draft202012::options()
        .with_registry(&registry)
        .should_validate_formats(true)
        .build(&decision_schema)
        .expect("decision validator");
    decision_validator
        .validate(&serde_json::to_value(decision).expect("decision JSON"))
        .unwrap_or_else(|error| {
            panic!("Rust decision must satisfy policy-decision.schema.json: {error}")
        });
}
