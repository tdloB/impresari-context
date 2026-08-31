#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
path = root.join(".github/workflows/yara-x-synthetic-envelope.yml")
abort "missing or symlinked synthetic envelope workflow" unless path.file? && !path.symlink?
bytes = path.read

required = [
  "on:\n  workflow_dispatch:\n", "permissions: {}", "runs-on: ubuntu-24.04",
  "github.repository == 'tdloB/impresari-context' && github.ref == 'refs/heads/main'",
  "SOURCE_SHA: 9a6bac4f8bda10b4b08ef3429587b9ae7f8bd1ce",
  "SOURCE_ARCHIVE_BYTES: \"27920771\"",
  "SOURCE_ARCHIVE_SHA256: 9dd03466b46fc1a882e39c2ce99c2d6ac0db18431a5c93ad2e24ab40922c0ef2",
  "https://codeload.github.com/tdloB/impresari-context/tar.gz/$SOURCE_SHA",
  "rustup toolchain install 1.98.0 --profile minimal",
  "./scripts/yara-x-synthetic-envelope.sh", "Verify ephemeral cleanup", "if: always()"
]
required.each { |fragment| abort "synthetic envelope workflow lost #{fragment.inspect}" unless bytes.include?(fragment) }

forbidden = [
  /^\s+(?:pull_request|pull_request_target|push|schedule|repository_dispatch):/,
  /workflow_dispatch:\s*\n\s+inputs:/,
  /actions\/checkout@/, /actions\/(?:upload|download)-artifact@/, /actions\/cache@/,
  /\b(?:secrets\.|github\.token|GITHUB_TOKEN|GH_TOKEN)\b/,
  /^permissions:\s+write-all/,
  /(?:contents|actions|checks|deployments|id-token|issues|packages|pull-requests|releases|security-events|statuses):\s+write/,
  /artifact_uploaded:\s*true/, /production_admitted:\s*true/, /iar_2_admitted:\s*true/
]
forbidden.each { |pattern| abort "synthetic envelope workflow crossed #{pattern.inspect}" if bytes.match?(pattern) }

abort "synthetic envelope workflow gained multiple jobs" unless bytes.scan(/^  [a-z][a-z0-9_-]+:\n    name:/).length == 1
abort "synthetic envelope workflow does not pin archive bytes and digest" unless
  bytes.include?("wc -c < \"$archive\"") && bytes.include?("sha256sum \"$archive\"")
abort "synthetic envelope workflow does not require an empty workspace" unless
  bytes.include?("find \"$GITHUB_WORKSPACE\" -mindepth 1 -maxdepth 1")

puts "YARA-X synthetic envelope workflow verified: manual-only credential-free immutable-source execution"
