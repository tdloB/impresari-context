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
ruby ./scripts/check-client-guidance-templates.rb
ruby ./scripts/check-client-lifecycle.rb
ruby ./scripts/check-claude-client-lifecycle.rb
sh -n ./scripts/install.sh
./scripts/install.sh --help >/dev/null
if ./scripts/install.sh --version latest >/dev/null 2>&1; then
    printf 'installer accepted an unpinned latest version\n' >&2
    exit 1
fi
ruby -c ./scripts/rehearse-codex-app-server.rb
ruby -c ./scripts/rehearse-claude-code.rb
ruby -c ./scripts/rehearse-claude-native-local-scope.rb
ruby -c ./scripts/rehearse-cursor-preadmission.rb
ruby -c ./scripts/rehearse-cursor-native-approval.rb
ruby -c ./scripts/rehearse-gemini-copilot-preadmission.rb
ruby -c ./scripts/rehearse-copilot-native-project.rb
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo metadata --locked --no-deps --format-version 1 >/dev/null
