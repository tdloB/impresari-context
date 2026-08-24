#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$repo_root"

./scripts/check-repository.sh
./scripts/check-security-boundaries.sh
ruby ./scripts/check-contracts.rb
ruby ./scripts/check-identity-vectors.rb
ruby ./scripts/check-path-vectors.rb
ruby ./scripts/check-jcs-vectors.rb
ruby ./scripts/check-semantic-vectors.rb
ruby ./scripts/check-sbom.rb
ruby ./scripts/check-evaluation.rb
ruby ./scripts/check-scale-evaluation.rb
ruby ./scripts/check-abrupt-restart.rb
ruby -c ./scripts/rehearse-codex-app-server.rb
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo metadata --locked --no-deps --format-version 1 >/dev/null
