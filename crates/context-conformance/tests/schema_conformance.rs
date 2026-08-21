// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Full Draft 2020-12 validation of the published conformance manifest."]

use std::{collections::BTreeMap, fs, path::PathBuf};

use jsonschema::Registry;
use serde_json::Value;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root should exist during tests")
}

fn read_json(path: &PathBuf) -> Value {
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
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
