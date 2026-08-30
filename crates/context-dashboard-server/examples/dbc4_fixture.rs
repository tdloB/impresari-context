// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Disposable synthetic fixture preparation for the DBC-4 native-browser rehearsal."]

use std::{env, error::Error, ffi::OsString, fmt::Write as _, fs, path::PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt as _;

use context_core::{AuditOutcome, Capability, ResourceBudget, audit_event};
use context_dashboard::{
    BudgetCeilings, BudgetSelector, LocalBudgetPolicyDraft, LocalBudgetRule, PolicyStore,
    compile_policy, project_event,
};
use context_store::{AuditReader, AuditRetention, AuditStore};
use rusqlite::{Connection, params};
use serde::Serialize;
use sha2::{Digest, Sha256};

const FIXED_TIME: &str = "2026-08-30T00:00:00Z";
const FIXED_CUTOFF: &str = "2026-08-01T00:00:00Z";

#[derive(Serialize)]
struct CanaryManifest {
    schema_name: &'static str,
    schema_version: &'static str,
    canaries: Vec<String>,
    hostile_script_marker: &'static str,
}

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    let root = exact_root(&arguments)?;
    let audit_root = root.join("audit-cache");
    let policy_root = root.join("policy-state");
    let canaries = canaries()?;

    let mut audit = AuditStore::open(&audit_root)?;
    let identity = format!("sha256:{}", "a".repeat(64));
    let event = audit_event(
        "evt_dashboard_dbc4_valid",
        "req_dashboard_dbc4_valid",
        FIXED_TIME,
        Some(&identity),
        Some(&identity),
        Capability::ContextBuild,
        AuditOutcome::Limited,
        &identity,
        budget()?,
        7,
        "0.2.0",
    )?;
    audit.append(&event, &AuditRetention::new(FIXED_CUTOFF, 100, 1_048_576)?)?;
    drop(audit);

    insert_rejected_row(&audit_root, &canaries)?;
    let batch = AuditReader::open(&audit_root)?.recent(10)?;
    if batch.events.len() != 1 || batch.unavailable_rows != 1 {
        return Err("fixture audit projection boundary was not exact".into());
    }
    project_event(&batch.events[0])?;
    PolicyStore::apply(&policy_root, policy()?, None, None)?;

    let manifest = CanaryManifest {
        schema_name: "dbc4-private-canary-manifest",
        schema_version: "1.0.0",
        canaries,
        hostile_script_marker: "__impresariDBC4Executed",
    };
    let manifest_path = root.join("private-canaries.json");
    fs::write(&manifest_path, serde_json::to_vec(&manifest)?)?;
    #[cfg(unix)]
    fs::set_permissions(&manifest_path, fs::Permissions::from_mode(0o600))?;

    let digest = hex(&Sha256::digest(fs::read(&manifest_path)?));
    println!(
        "{}",
        serde_json::json!({
            "schema_name": "dbc4-fixture-ready",
            "schema_version": "1.0.0",
            "audit_root": audit_root,
            "policy_root": policy_root,
            "private_manifest_sha256": format!("sha256:{digest}"),
            "valid_rows": "1",
            "withheld_rows": "1"
        })
    );
    Ok(())
}

fn exact_root(arguments: &[OsString]) -> Result<PathBuf, Box<dyn Error>> {
    if arguments.len() != 1 {
        return Err("usage: dbc4_fixture <existing-empty-root-below-system-temp>".into());
    }
    let supplied = PathBuf::from(&arguments[0]);
    if supplied.as_os_str().is_empty() || supplied.is_symlink() || !supplied.is_dir() {
        return Err("fixture root must be an existing real directory".into());
    }
    let root = supplied.canonicalize()?;
    let mut temporary_roots = vec![env::temp_dir().canonicalize()?];
    let conventional = PathBuf::from("/private/tmp");
    if conventional.is_dir() {
        temporary_roots.push(conventional.canonicalize()?);
    }
    let strictly_below_temporary = temporary_roots
        .iter()
        .any(|temporary| root != *temporary && root.starts_with(temporary));
    if !strictly_below_temporary || fs::read_dir(&root)?.next().is_some() {
        return Err(
            "fixture root must be empty and strictly below the system temporary root".into(),
        );
    }
    Ok(root)
}

fn canaries() -> Result<Vec<String>, Box<dyn Error>> {
    let mut random = [0_u8; 16];
    getrandom::fill(&mut random).map_err(|_| "fixture randomness unavailable")?;
    let suffix = hex(&random);
    Ok(vec![
        format!("DBC4_PATH_{suffix}/private/source.rs"),
        format!("DBC4_FILENAME_{suffix}.pem"),
        format!("DBC4_QUERY_{suffix}"),
        format!("DBC4_CONTENT_{suffix}"),
        format!("DBC4_PROMPT_{suffix}"),
        format!("DBC4_CREDENTIAL_{suffix}"),
        format!("DBC4_ENVIRONMENT_{suffix}"),
    ])
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(
        String::with_capacity(bytes.len().saturating_mul(2)),
        |mut output, byte| {
            write!(output, "{byte:02x}").expect("writing to a string cannot fail");
            output
        },
    )
}

fn insert_rejected_row(
    audit_root: &std::path::Path,
    canaries: &[String],
) -> Result<(), Box<dyn Error>> {
    let hostile = format!(
        "<script>globalThis.__impresariDBC4Executed=true</script><img src=x onerror=\"globalThis.__impresariDBC4Executed=true\">{}",
        "X".repeat(8_192)
    );
    let payload = serde_json::to_vec(&serde_json::json!({
        "schema_name": "future-hostile-audit-event",
        "schema_version": "999.0.0",
        "source_path": canaries[0],
        "filename": canaries[1],
        "query": canaries[2],
        "content": canaries[3],
        "prompt": canaries[4],
        "credential": canaries[5],
        "environment": canaries[6],
        "hostile": hostile,
        "unknown": true
    }))?;
    let connection = Connection::open(audit_root.join("audit/audit.sqlite3"))?;
    connection.execute(
        "INSERT INTO audit_events(event_id,occurred_at,workspace_identity,payload) VALUES(?1,?2,NULL,?3)",
        params!["evt_dashboard_dbc4_withheld", FIXED_TIME, payload],
    )?;
    Ok(())
}

fn budget() -> Result<ResourceBudget, Box<dyn Error>> {
    Ok(ResourceBudget::conservative(
        8_192, 8, 16, 512, 32, 4, 5_000, 8_388_608,
    )?)
}

fn policy() -> Result<context_dashboard::LocalBudgetPolicy, Box<dyn Error>> {
    Ok(compile_policy(LocalBudgetPolicyDraft {
        schema_name: "local-budget-policy".into(),
        schema_version: "1.0.0".into(),
        revision: "1".into(),
        created_at: FIXED_TIME.into(),
        expires_at: None,
        rules: vec![LocalBudgetRule {
            rule_id: "local".into(),
            selector: BudgetSelector {
                purpose: None,
                capability: Some(Capability::ContextBuild),
            },
            deny: false,
            ceilings: BudgetCeilings {
                requested: Some("4096".into()),
                ..BudgetCeilings::default()
            },
        }],
    })?)
}
