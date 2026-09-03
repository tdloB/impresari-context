// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
//! Cache-stable context prefix identity (IC-CSCP-127).
//!
//! Every provider re-sends conversation history each turn, so a context packet
//! delivered once is billed many times. Providers already solve this for a
//! stable prefix. This module makes the prefix's stability *checkable*: it
//! derives a key from exactly the inputs that determine the stable bytes, so a
//! client can cache confidently and detect staleness.
//!
//! Impresari performs no caching and issues no provider request. It states that
//! the bytes are worth caching; the client decides.

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

/// Schema discriminator for the stability declaration.
pub const CACHE_PREFIX_SCHEMA_NAME: &str = "impresari_context_cache_prefix";
/// Schema version for the stability declaration.
pub const CACHE_PREFIX_SCHEMA_VERSION: &str = "1.0";

/// Domain separator, so a prefix key can never collide with another digest the
/// product emits.
const CACHE_PREFIX_DOMAIN: &[u8] = b"impresari-context\0cache-stable-prefix\0v1\0";

/// Fields that change per request and must stay outside the cached prefix.
///
/// Named explicitly so a client can place them after the prefix rather than
/// discovering the hard way that its cache never hits.
pub const VOLATILE_FIELDS: [&str; 6] = [
    "request_id",
    "event_id",
    "occurred_at",
    "elapsed_ms",
    "receipt",
    "per_call",
];

/// The identities that determine the stable bytes.
///
/// Every field is already safe in a source-free record. Task *text*, source
/// bytes, paths, queries, and excerpts are deliberately absent: a cache key
/// travels further than a packet — into client logs, provider telemetry, and
/// support channels — so it stays control metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextPrefixInputs<'a> {
    /// Exact workspace snapshot identity.
    pub workspace_snapshot: &'a str,
    /// Exact task identity, not task text.
    pub task_identity: &'a str,
    /// Exact resource-budget identity.
    pub budget_identity: &'a str,
    /// Exact policy-decision identity.
    pub policy_identity: &'a str,
    /// Exact product build identity.
    pub product_identity: &'a str,
}

/// A client-facing statement that a prefix is safe to cache under a key.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPrefixStability {
    /// Schema discriminator.
    pub schema_name: String,
    /// Schema version.
    pub schema_version: String,
    /// Key over exactly the inputs that determine the stable bytes.
    pub cache_key: String,
    /// Whether the prefix is declared stable under that key.
    pub stable: bool,
    /// Fields a client must keep after the prefix.
    pub volatile_fields: Vec<String>,
}

/// Closed failure category for one stability declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CachePrefixErrorCode {
    /// An identity is not an exact `sha256:` digest.
    InvalidIdentity,
}

impl CachePrefixErrorCode {
    /// Stable source-free category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidIdentity => "invalid_identity",
        }
    }
}

impl std::fmt::Display for CachePrefixErrorCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl std::error::Error for CachePrefixErrorCode {}

/// Declare a context prefix cacheable under a derived key.
///
/// Changing any determining input changes the key; changing a volatile field
/// cannot, because no volatile field is an input.
///
/// # Errors
/// Returns a closed category when any identity is not an exact digest.
pub fn declare_context_prefix_stability(
    inputs: &ContextPrefixInputs<'_>,
) -> Result<ContextPrefixStability, CachePrefixErrorCode> {
    let ordered = [
        inputs.workspace_snapshot,
        inputs.task_identity,
        inputs.budget_identity,
        inputs.policy_identity,
        inputs.product_identity,
    ];
    if !ordered.iter().all(|identity| is_exact_digest(identity)) {
        return Err(CachePrefixErrorCode::InvalidIdentity);
    }
    let mut hasher = Sha256::new();
    hasher.update(CACHE_PREFIX_DOMAIN);
    for identity in ordered {
        // Length-prefixed so no two field arrangements can collide.
        hasher.update(
            u64::try_from(identity.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        hasher.update(identity.as_bytes());
    }
    let mut cache_key = String::from("sha256:");
    for byte in hasher.finalize() {
        use std::fmt::Write as _;
        write!(cache_key, "{byte:02x}").expect("string write");
    }
    Ok(ContextPrefixStability {
        schema_name: CACHE_PREFIX_SCHEMA_NAME.to_owned(),
        schema_version: CACHE_PREFIX_SCHEMA_VERSION.to_owned(),
        cache_key,
        stable: true,
        volatile_fields: VOLATILE_FIELDS
            .iter()
            .map(|field| (*field).to_owned())
            .collect(),
    })
}

fn is_exact_digest(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        format!("sha256:{}", byte.to_string().repeat(64))
    }

    fn inputs<'a>(
        snapshot: &'a str,
        task: &'a str,
        budget: &'a str,
        policy: &'a str,
        product: &'a str,
    ) -> ContextPrefixInputs<'a> {
        ContextPrefixInputs {
            workspace_snapshot: snapshot,
            task_identity: task,
            budget_identity: budget,
            policy_identity: policy,
            product_identity: product,
        }
    }

    #[test]
    fn identical_inputs_yield_an_identical_key() {
        let snapshot = digest('1');
        let task = digest('2');
        let budget = digest('3');
        let policy = digest('4');
        let product = digest('5');
        let first =
            declare_context_prefix_stability(&inputs(&snapshot, &task, &budget, &policy, &product))
                .expect("key");
        let second =
            declare_context_prefix_stability(&inputs(&snapshot, &task, &budget, &policy, &product))
                .expect("key");
        assert_eq!(first.cache_key, second.cache_key);
        assert!(first.stable);
    }

    #[test]
    fn every_determining_input_changes_the_key() {
        let base = [
            digest('1'),
            digest('2'),
            digest('3'),
            digest('4'),
            digest('5'),
        ];
        let reference = declare_context_prefix_stability(&inputs(
            &base[0], &base[1], &base[2], &base[3], &base[4],
        ))
        .expect("key")
        .cache_key;
        for index in 0..base.len() {
            let mut changed = base.clone();
            changed[index] = digest('a');
            let key = declare_context_prefix_stability(&inputs(
                &changed[0],
                &changed[1],
                &changed[2],
                &changed[3],
                &changed[4],
            ))
            .expect("key")
            .cache_key;
            assert_ne!(key, reference, "input {index} must change the key");
        }
    }

    #[test]
    fn field_order_cannot_collide() {
        // Length-prefixing means a value moving between fields changes the key.
        let first = digest('1');
        let second = digest('2');
        let filler = digest('3');
        let straight =
            declare_context_prefix_stability(&inputs(&first, &second, &filler, &filler, &filler))
                .expect("key")
                .cache_key;
        let swapped =
            declare_context_prefix_stability(&inputs(&second, &first, &filler, &filler, &filler))
                .expect("key")
                .cache_key;
        assert_ne!(straight, swapped);
    }

    #[test]
    fn the_key_is_exactly_a_digest_and_carries_no_content() {
        let snapshot = digest('1');
        let task = digest('2');
        let budget = digest('3');
        let policy = digest('4');
        let product = digest('5');
        let declared =
            declare_context_prefix_stability(&inputs(&snapshot, &task, &budget, &policy, &product))
                .expect("key");
        assert!(is_exact_digest(&declared.cache_key));
        assert_eq!(declared.cache_key.len(), 71);
    }

    #[test]
    fn volatile_fields_are_declared_and_are_not_inputs() {
        let snapshot = digest('1');
        let task = digest('2');
        let budget = digest('3');
        let policy = digest('4');
        let product = digest('5');
        let declared =
            declare_context_prefix_stability(&inputs(&snapshot, &task, &budget, &policy, &product))
                .expect("key");
        for field in VOLATILE_FIELDS {
            assert!(declared.volatile_fields.iter().any(|value| value == field));
        }
        // The struct that determines the key has exactly the determining
        // fields, so a volatile value has no way to reach it.
        assert_eq!(declared.volatile_fields.len(), VOLATILE_FIELDS.len());
    }

    #[test]
    fn a_non_digest_identity_fails_closed() {
        let good = digest('1');
        for bad in ["", "sha256:short", "notadigest", "sha256:zz"] {
            assert!(matches!(
                declare_context_prefix_stability(&inputs(bad, &good, &good, &good, &good)),
                Err(CachePrefixErrorCode::InvalidIdentity)
            ));
        }
    }

    #[test]
    fn module_performs_no_caching_and_no_provider_work() {
        let source = include_str!("cache_prefix.rs");
        let shipped = source
            .split_once("#[cfg(test)]")
            .expect("test module marker")
            .0;
        for forbidden in [
            "Command",
            "spawn",
            "OPENAI",
            "ANTHROPIC",
            "cache_control",
            "fs::",
        ] {
            assert!(!shipped.contains(forbidden), "must not reach {forbidden}");
        }
    }
}
