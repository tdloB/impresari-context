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
ruby ./scripts/check-linux-isolation-maintenance.rb
ruby ./scripts/check-linux-isolation-topology-feasibility.rb
ruby ./scripts/check-linux-rootless-host-preflight.rb
ruby ./scripts/check-linux-rootless-user-manager-rehearsal.rb
ruby ./scripts/check-linux-rootless-login-session-rehearsal.rb
ruby ./scripts/check-linux-external-delegation-capability.rb
ruby ./scripts/check-linux-external-delegation-live-rehearsal.rb
ruby ./scripts/check-linux-isolation-production-lifecycle.rb
ruby ./scripts/check-linux-package-lifecycle-rehearsal.rb
ruby ./scripts/check-linux-external-lifecycle-composition.rb
ruby ./scripts/check-linux-external-production-support-admission.rb
ruby ./scripts/check-independent-security-review-readiness.rb
ruby ./scripts/check-independent-security-review-backlog.rb
ruby ./scripts/check-v0-2-independent-review-release-gate.rb
ruby ./scripts/check-codex-client-lifecycle.rb
ruby ./scripts/check-claude-client-lifecycle.rb
ruby ./scripts/check-cursor-client-lifecycle.rb
ruby ./scripts/check-vscode-client-lifecycle.rb
ruby ./scripts/check-roadmap-maintenance-automation.rb
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
ruby -c ./scripts/rehearse-vscode-copilot-guided-delivery.rb
ruby -c ./scripts/rehearse-gemini-copilot-preadmission.rb
ruby -c ./scripts/rehearse-copilot-native-project.rb
ruby -c ./scripts/rehearse-dashboard-native-browser.rb
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo metadata --locked --no-deps --format-version 1 >/dev/null
