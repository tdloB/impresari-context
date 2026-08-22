// SPDX-License-Identifier: Apache-2.0
#![forbid(unsafe_code)]
#![doc = "Child-process helper for abrupt cache restart evaluation."]

use std::{
    env,
    io::{self, Write as _},
    path::Path,
    process, thread,
    time::Duration,
};

use context_store::{CachedArtifact, WorkspaceCache};

const WORKSPACE: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SNAPSHOT: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const POLICY: &str = "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn run(mode: &str, root: &Path) -> Result<(), String> {
    let mut cache = WorkspaceCache::open(root, WORKSPACE).map_err(|error| error.to_string())?;
    match mode {
        "initialize" => {
            cache
                .promote(
                    SNAPSHOT,
                    POLICY,
                    &[CachedArtifact {
                        path_units: "YS5ycw".into(),
                        display_path: "a.rs".into(),
                        content_hash: WORKSPACE.into(),
                        size_bytes: 1,
                        terms: "alpha".into(),
                    }],
                )
                .map_err(|error| error.to_string())?;
        }
        "hold" => {
            println!("READY");
            io::stdout().flush().map_err(|error| error.to_string())?;
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        "verify" => {
            let current = cache
                .current()
                .map_err(|error| error.to_string())?
                .ok_or("missing current generation")?;
            if current.snapshot_id != SNAPSHOT {
                return Err("committed generation changed after restart".into());
            }
        }
        _ => return Err("invalid helper mode".into()),
    }
    Ok(())
}

fn main() {
    let arguments = env::args().collect::<Vec<_>>();
    if arguments.len() != 3 || run(&arguments[1], Path::new(&arguments[2])).is_err() {
        process::exit(1);
    }
}
