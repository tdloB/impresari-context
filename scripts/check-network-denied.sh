#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

platform=$(uname -s)
case "$platform" in
    Darwin)
        sandbox-exec -p '(version 1)(allow default)(deny network*)' \
            env CARGO_NET_OFFLINE=true cargo test --workspace --all-targets --locked --offline
        ;;
    Linux)
        if ! command -v unshare >/dev/null 2>&1; then
            printf 'unshare is required for the Linux network-denied gate\n' >&2
            exit 1
        fi
        unshare --net env CARGO_NET_OFFLINE=true \
            cargo test --workspace --all-targets --locked --offline
        ;;
    *)
        printf 'no approved network-denial harness for %s\n' "$platform" >&2
        exit 1
        ;;
esac

printf 'network-denied full test suite passed on %s\n' "$platform"
