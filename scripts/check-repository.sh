#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

required_files='Cargo.toml
Cargo.lock
rust-toolchain.toml
LICENSE
ACKNOWLEDGMENTS.md
CONTRIBUTING.md
GOVERNANCE.md
MAINTAINERS.md
SECURITY.md
CODE_OF_CONDUCT.md'

printf '%s\n' "$required_files" | while IFS= read -r path; do
    if [ ! -f "$path" ]; then
        printf 'missing required file: %s\n' "$path" >&2
        exit 1
    fi
done

if [ ! -f docs/decisions/0017-v0.1-release-assurance-policy.md ]; then
    printf 'missing v0.1 release assurance policy ADR\n' >&2
    exit 1
fi

ruby ./scripts/check-release-assurance-policy.rb

if find crates -type f -name '*.rs' -exec grep -L '#!\[forbid(unsafe_code)\]' {} \; | grep -q .; then
    printf 'every first-party Rust crate root must forbid unsafe code\n' >&2
    exit 1
fi

if grep -R -n -E 'context-engine-oss|BEGIN (RSA|OPENSSH|EC|DSA) PRIVATE KEY' \
    --exclude-dir=.git --exclude-dir=target --exclude='check-repository.sh' .; then
    printf 'repository contains a forbidden placeholder path or private-key marker\n' >&2
    exit 1
fi

printf 'repository policy checks passed\n'
