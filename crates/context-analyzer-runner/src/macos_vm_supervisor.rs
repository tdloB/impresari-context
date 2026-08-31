// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Source-free synthetic macOS local-VM lifecycle supervisor for ADR-0087."]

use std::{env, path::PathBuf, time::Duration};

use context_analyzer_runner::{MacOsVmSupervisorAction, MacOsVmSyntheticSupervisor};

fn run() -> Result<Vec<u8>, ()> {
    let mut arguments = env::args_os().skip(1);
    let controller = PathBuf::from(arguments.next().ok_or(())?);
    let asset_root = PathBuf::from(arguments.next().ok_or(())?);
    let expected_controller_digest = arguments.next().ok_or(())?.into_string().map_err(|_| ())?;
    let job_id = arguments.next().ok_or(())?.into_string().map_err(|_| ())?;
    let action = arguments.next().ok_or(())?.into_string().map_err(|_| ())?;
    if arguments.next().is_some() {
        return Err(());
    }
    let action = MacOsVmSupervisorAction::from_name(&action).ok_or(())?;
    let receipt = MacOsVmSyntheticSupervisor {
        controller,
        expected_controller_digest,
        asset_root,
        timeout: Duration::from_secs(10),
    }
    .execute(&job_id, action)
    .map_err(|_| ())?;
    serde_json_canonicalizer::to_vec(&receipt).map_err(|_| ())
}

fn main() {
    match run() {
        Ok(bytes) => println!("{}", String::from_utf8_lossy(&bytes)),
        Err(()) => std::process::exit(1),
    }
}
