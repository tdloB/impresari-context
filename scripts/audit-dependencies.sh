#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

cargo audit --deny warnings
cargo audit --deny warnings --file fuzz/Cargo.lock
cargo deny check advisories bans licenses sources
cargo tree --workspace --locked --duplicates
