#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

production_sources=$(find crates -path '*/src/*.rs' -type f -print)
if grep -n -E 'std::net::|TcpStream|UdpSocket|std::process::Command|Command::new|reqwest|ureq|hyper::|tonic::|enable_load_extension' $production_sources; then
    printf 'forbidden network, process-execution, or extension-loading surface in production code\n' >&2
    exit 1
fi

if cargo tree --locked --offline --prefix none | grep -E '^(reqwest|ureq|hyper|tonic|curl|openssl) '; then
    printf 'unexpected network-capable runtime dependency\n' >&2
    exit 1
fi

before=$(git status --porcelain=v1 --untracked-files=no)
cargo test --workspace --all-targets --locked --offline
after=$(git status --porcelain=v1 --untracked-files=no)
if [ "$before" != "$after" ]; then
    printf 'tracked source/repository state changed during the test suite\n' >&2
    exit 1
fi

printf 'security boundary and tracked-source immutability checks passed\n'
