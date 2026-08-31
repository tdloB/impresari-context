#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
path = root.join(".github/workflows/yara-x-reproducibility-diagnostic.yml")
abort "missing or symlinked YARA-X reproducibility workflow" unless path.file? && !path.symlink?
bytes = path.read

required = [
  "on:\n  workflow_dispatch:\n",
  "permissions: {}",
  "github.repository == 'tdloB/impresari-context' && github.ref == 'refs/heads/main'",
  "runs-on: ubuntu-24.04",
  "timeout-minutes: 120",
  "SOURCE_SHA: ae4e0bea1ed9576abecb998250ad06fc2081f2a8",
  "SOURCE_ARCHIVE_BYTES: \"27959111\"",
  "SOURCE_ARCHIVE_SHA256: 2e6323cffce957108429c804dd4f9876a6a0d27fdef31569029213807c3e04a2",
  "https://codeload.github.com/tdloB/impresari-context/tar.gz/$SOURCE_SHA",
  "rustup toolchain install 1.93.0 --profile minimal",
  "cargo +1.93.0 install cargo-audit --version 0.22.2 --locked",
  "./scripts/yara-x-reproducibility-diagnostic.sh",
  "Verify ephemeral cleanup",
  "if: always()"
]
required.each { |fragment| abort "YARA-X reproducibility workflow lost #{fragment.inspect}" unless bytes.include?(fragment) }

forbidden = [
  /^\s+(?:pull_request|pull_request_target|push|schedule|repository_dispatch):/,
  /workflow_dispatch:\s*\n\s+inputs:/,
  /actions\/checkout@/,
  /actions\/(?:upload|download)-artifact@/,
  /actions\/cache@/,
  /\b(?:secrets\.|github\.token|GITHUB_TOKEN|GH_TOKEN)\b/,
  /^permissions:\s+write-all/,
  /(?:contents|actions|checks|deployments|id-token|issues|packages|pull-requests|releases|security-events|statuses):\s+write/,
  /sudo\b/,
  /artifact_uploaded:\s*true/,
  /production_admitted:\s*true/,
  /iar_2(?:_admitted)?:\s*true/
]
forbidden.each { |pattern| abort "YARA-X reproducibility workflow crossed #{pattern.inspect}" if bytes.match?(pattern) }

abort "YARA-X reproducibility workflow source archive URL is not singular" unless bytes.scan("https://codeload.github.com/tdloB/impresari-context/tar.gz/").length == 1
abort "YARA-X reproducibility workflow gained multiple jobs" unless bytes.scan(/^  [a-z][a-z0-9_-]+:\n    name:/).length == 1
abort "YARA-X reproducibility workflow no longer verifies cleanup" unless bytes.include?("impresari-yara-x-reproducibility") && bytes.include?("-name yr")

puts "YARA-X reproducibility workflow verified: manual-only no-secret build diagnostic without upload or execution"
