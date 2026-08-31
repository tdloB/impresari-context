#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

production_sources=$(find crates -path '*/src/*.rs' -type f -print)
non_dashboard_sources=$(find crates -path '*/src/*.rs' -type f ! -path 'crates/context-dashboard-server/src/*' -print)
if grep -n -E 'std::net::|TcpStream|UdpSocket|reqwest|ureq|hyper::|tonic::|enable_load_extension' $non_dashboard_sources; then
    printf 'forbidden network or extension-loading surface in production code\n' >&2
    exit 1
fi

# ADR-0072 permits one std-only loopback listener inside the isolated dashboard
# server crate. Strip cfg(test) modules before checking so test clients do not
# count as product authority. Production code may accept connected streams but
# may not initiate an outbound connection or resolve a remote address.
dashboard_outbound_sites=$(find crates/context-dashboard-server/src -name '*.rs' -type f -exec awk '
    /^#\[cfg\(test\)\]/{nextfile}
    /TcpStream::connect|connect_timeout|UdpSocket|ToSocketAddrs|lookup_host|reqwest|ureq|hyper::|tonic::/{print FILENAME ":" FNR ":" $0}
' {} +)
if [ -n "$dashboard_outbound_sites" ]; then
    printf '%s\n' "$dashboard_outbound_sites" >&2
    printf 'unexpected outbound network authority in ADR-0072 dashboard server\n' >&2
    exit 1
fi
dashboard_listener_sites=$(find crates/context-dashboard-server/src -name '*.rs' -type f -exec awk '
    /^#\[cfg\(test\)\]/{nextfile}
    /TcpListener::bind/{print FILENAME ":" FNR ":" $0}
' {} +)
if [ "$(printf '%s\n' "$dashboard_listener_sites" | sed '/^$/d' | wc -l | tr -d ' ')" -ne 1 ]; then
    printf '%s\n' "$dashboard_listener_sites" >&2
    printf 'ADR-0072 requires exactly one production loopback listener bind site\n' >&2
    exit 1
fi

# ADR-0010 permits one fixed argv-based launch site for the pinned structural
# worker. ADR-0055 adds one separate fixed argv-based Codex App Server launch
# site for an explicit, ephemeral, authority-denying delivery attempt. ADR-0060
# adds the equivalent exact-version, zero-tool Copilot prompt launch site.
# ADR-0061 adds Claude Code's safe-mode, zero-tool print launch site. ADR-0062
# adds Cursor Agent's ask-mode, sandboxed, empty-workspace print launch site.
# ADR-0063 adds VS Code's Ask-mode chat CLI launcher in an empty disposable cwd.
# ADR-0074 adds one exact-pinned, application-enforced analyzer-runner launch
# site with private staging, bounded transport, and no analyzer authority.
# ADR-0087 reuses that same site for the exact synthetic macOS VM controller;
# it does not add a second child-process launch surface.
# No other production module may acquire child-process authority.
process_sites=$(grep -n -E 'std::process::Command|Command::new' $production_sources || true)
expected_structural_site='crates/context-structural/src/lib.rs:'
expected_codex_site='crates/context-codex-app-server/src/lib.rs:'
expected_copilot_site='crates/context-copilot-cli/src/lib.rs:'
expected_claude_site='crates/context-claude-code/src/lib.rs:'
expected_cursor_site='crates/context-cursor-agent/src/lib.rs:'
expected_vscode_site='crates/context-vscode-copilot/src/lib.rs:'
expected_analyzer_site='crates/context-analyzer-runner/src/lib.rs:'
if [ "$(printf '%s\n' "$process_sites" | sed '/^$/d' | wc -l | tr -d ' ')" -ne 7 ] ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_structural_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_codex_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_copilot_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_claude_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_cursor_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_vscode_site" ||
   ! printf '%s\n' "$process_sites" | grep -q "^$expected_analyzer_site"; then
    printf '%s\n' "$process_sites" >&2
    printf 'unexpected production child-process authority outside ADR-0010, ADR-0055, ADR-0060, ADR-0061, ADR-0062, ADR-0063, and ADR-0074 launch sites\n' >&2
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
