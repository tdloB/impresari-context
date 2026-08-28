#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

production_sources=$(find crates -path '*/src/*.rs' -type f -print)
network_sites=$(grep -n -E 'std::net::|TcpStream|UdpSocket|reqwest|ureq|hyper::|tonic::|enable_load_extension' $production_sources || true)
unexpected_network_sites=$(printf '%s\n' "$network_sites" |
    grep -v '^crates/context-evaluation/src/provider_adapter\.rs:' |
    grep -v '^crates/context-evaluation/src/provider_adapter/\(openai\|anthropic\)\.rs:' || true)
if [ -n "$unexpected_network_sites" ]; then
    printf '%s\n' "$unexpected_network_sites" >&2
    printf 'forbidden network or extension-loading surface in production code\n' >&2
    exit 1
fi

# ADR-0060 permits one HTTPS client dependency only in the developer evaluation
# crate. Provider endpoints remain constants in the two reviewed translations.
network_manifest_sites=$(grep -n -E '^[[:space:]]*(reqwest|ureq|hyper|tonic|curl|openssl)[[:space:]]*=' crates/*/Cargo.toml || true)
if [ "$(printf '%s\n' "$network_manifest_sites" | sed '/^$/d' | wc -l | tr -d ' ')" -ne 1 ] ||
   ! printf '%s\n' "$network_manifest_sites" | grep -q '^crates/context-evaluation/Cargo.toml:.*reqwest'; then
    printf '%s\n' "$network_manifest_sites" >&2
    printf 'unexpected direct network-capable runtime dependency\n' >&2
    exit 1
fi

# ADR-0010 permits one fixed argv-based launch site for the pinned structural
# worker. ADR-0055 adds one separate fixed argv-based Codex App Server launch
# site for an explicit, ephemeral, authority-denying delivery attempt.
# ADR-0059 separately permits one explicit-consent, argv-only launch site in
# the developer evaluation harness. No other production module may acquire
# child-process authority.
process_sites=$(grep -n -E 'std::process::Command|Command::new' $production_sources || true)
expected_structural_site='crates/context-structural/src/lib.rs:'
expected_codex_site='crates/context-codex-app-server/src/lib.rs:'
expected_evaluation_site='crates/context-evaluation/src/agent_eval.rs:'
if [ "$(printf '%s\n' "$process_sites" | sed '/^$/d' | wc -l | tr -d ' ')" -ne 3 ] ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_structural_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_codex_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_evaluation_site"; then
    printf '%s\n' "$process_sites" >&2
    printf 'unexpected child-process authority outside ADR-0010, ADR-0055, and ADR-0059 launch sites\n' >&2
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
