#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

production_sources=$(find crates -path '*/src/*.rs' -type f -print)
if grep -n -E 'std::net::|TcpStream|UdpSocket|reqwest|ureq|hyper::|tonic::|enable_load_extension' $production_sources; then
    printf 'forbidden network or extension-loading surface in production code\n' >&2
    exit 1
fi

# ADR-0010 permits one fixed argv-based launch site for the pinned structural
# worker. No other production module may acquire child-process authority.
process_sites=$(grep -n -E 'std::process::Command|Command::new' $production_sources || true)
expected_site='crates/context-structural/src/lib.rs:'
if [ "$(printf '%s\n' "$process_sites" | sed '/^$/d' | wc -l | tr -d ' ')" -ne 1 ] ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_site"; then
    printf '%s\n' "$process_sites" >&2
    printf 'unexpected production child-process authority outside the single ADR-0010 launcher\n' >&2
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
