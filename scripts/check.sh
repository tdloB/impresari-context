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
ruby ./scripts/check-yara-adapter-contract.rb
ruby ./scripts/check-yara-supply-chain-contract.rb
ruby ./scripts/check-yara-x-contract.rb
ruby ./scripts/check-yara-x-artifact-compatibility.rb
ruby ./scripts/check-yara-x-artifact-compatibility-workflow.rb
ruby ./scripts/check-yara-x-ndjson-adapter.rb
ruby ./scripts/check-codex-client-lifecycle.rb
ruby ./scripts/check-claude-client-lifecycle.rb
ruby ./scripts/check-cursor-client-lifecycle.rb
ruby ./scripts/check-vscode-client-lifecycle.rb
ruby ./scripts/check-roadmap-maintenance-automation.rb
ruby ./scripts/check-macos-vm-contracts.rb
ruby ./scripts/check-macos-vm-guest-supply-chain.rb
ruby ./scripts/check-macos-vm-upstream-auth-contract.rb
ruby ./scripts/check-macos-vm-upstream-auth-contract-v2.rb
ruby ./scripts/check-macos-vm-vulnerability-review.rb
ruby ./scripts/check-macos-vm-vulnerability-review-v2.rb
ruby ./scripts/check-macos-vm-release-metadata-seal.rb
ruby ./scripts/check-windows-native-feasibility-contract.rb
ruby ./scripts/check-windows-native-synthetic-worker-contract.rb
ruby ./scripts/check-windows-basecontainer-capability-contract.rb
sh -n ./scripts/prepare-macos-vm-feasibility.sh
sh -n ./scripts/build-macos-vm-feasibility.sh
sh -n ./scripts/check-macos-vm-feasibility.sh
sh -n ./scripts/check-macos-vm-supervisor-lifecycle.sh
sh -n ./scripts/check-macos-vm-resource-canary.sh
sh -n ./scripts/check-macos-vm-host-interruption.sh
ruby -c ./scripts/build-macos-vm-initramfs.rb
ruby -c ./scripts/extract-macos-vm-kernel.rb
ruby -c ./scripts/check-macos-vm-guest-supply-chain.rb
ruby -c ./scripts/check-macos-vm-upstream-auth-contract.rb
ruby -c ./scripts/check-macos-vm-upstream-auth-contract-v2.rb
ruby -c ./scripts/check-macos-vm-vulnerability-review.rb
ruby -c ./scripts/check-macos-vm-vulnerability-review-v2.rb
ruby -c ./scripts/check-macos-vm-release-metadata-seal.rb
ruby -c ./scripts/check-windows-native-feasibility-contract.rb
ruby -c ./scripts/check-windows-native-synthetic-worker-contract.rb
ruby -c ./scripts/check-windows-basecontainer-capability-contract.rb
ruby -c ./scripts/check-yara-supply-chain-contract.rb
ruby -c ./scripts/check-yara-x-contract.rb
ruby -c ./scripts/check-yara-x-artifact-compatibility.rb
ruby -c ./scripts/check-yara-x-artifact-compatibility-workflow.rb
ruby -c ./scripts/check-yara-x-source-archive.rb
ruby -c ./scripts/check-yara-x-rule-policy.rb
ruby -c ./scripts/check-yara-x-live-compatibility-receipt.rb
ruby -c ./scripts/check-yara-x-ndjson-adapter.rb
sh -n ./scripts/yara-x-artifact-compatibility.sh
if [ "$(uname -s)" = Linux ]; then
  mkdir -p ./target/static-checks
  cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
    ./platform/linux-yara-x-compatibility/launcher.c \
    -o ./target/static-checks/linux-yara-x-launcher
fi
rustfmt --check ./platform/windows-native-feasibility/windows-native-capability-probe.rs
rustfmt --check ./platform/windows-native-feasibility/windows-native-synthetic-broker.rs
rustfmt --check ./platform/windows-native-feasibility/windows-native-synthetic-worker.rs
rustfmt --check ./platform/windows-native-feasibility/windows-basecontainer-capability-probe.rs
sh -n ./scripts/verify-macos-vm-alpine-archive.sh
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
