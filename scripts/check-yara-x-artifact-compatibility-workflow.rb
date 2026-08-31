#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
path = root.join(".github/workflows/yara-x-artifact-compatibility.yml")
abort "missing or symlinked YARA-X compatibility workflow" unless path.file? && !path.symlink?
bytes = path.read

required = [
  "on:\n  workflow_dispatch:\n",
  "permissions: {}",
  "github.repository == 'tdloB/impresari-context' && github.ref == 'refs/heads/main'",
  "runs-on: ubuntu-24.04",
  "timeout-minutes: 120",
  "SOURCE_SHA: 1debbe5c40a9a995d9120db6108f88c95ea19978",
  "SOURCE_ARCHIVE_BYTES: \"27883115\"",
  "SOURCE_ARCHIVE_SHA256: f57ee7b08c9db78fe20475fb66c2aebc732ef8ee74ec9820ff1388119775c1fd",
  "ea2abe8460a1faab60b4ab2d854e48bdd45f1998106cd5e62229153155d254a8",
  "https://codeload.github.com/tdloB/impresari-context/tar.gz/$SOURCE_SHA",
  "rustup toolchain install 1.93.0 --profile minimal",
  "cargo +1.93.0 install cargo-audit --version 0.22.2 --locked",
  "./scripts/yara-x-artifact-compatibility.sh",
  "Verify ephemeral cleanup",
  "if: always()"
]
required.each { |fragment| abort "YARA-X compatibility workflow lost #{fragment.inspect}" unless bytes.include?(fragment) }

forbidden = [
  /^\s+(?:pull_request|pull_request_target|push|schedule|repository_dispatch):/,
  /workflow_dispatch:\s*\n\s+inputs:/,
  /actions\/checkout@/,
  /actions\/(?:upload|download)-artifact@/,
  /actions\/cache@/,
  /\b(?:secrets\.|github\.token|GITHUB_TOKEN|GH_TOKEN)\b/,
  /^permissions:\s+write-all/,
  /(?:contents|actions|checks|deployments|id-token|issues|packages|pull-requests|releases|security-events|statuses):\s+write/,
  /sudo\s+systemd-run/,
  /artifact_uploaded:\s*true/,
  /production_admitted:\s*true/,
  /iar_2_admitted:\s*true/
]
forbidden.each { |pattern| abort "YARA-X compatibility workflow crossed #{pattern.inspect}" if bytes.match?(pattern) }

abort "YARA-X workflow source archive URL is not singular" unless bytes.scan("https://codeload.github.com/tdloB/impresari-context/tar.gz/").length == 1
abort "YARA-X workflow gained multiple jobs" unless bytes.scan(/^  [a-z][a-z0-9_-]+:\n    name:/).length == 1
abort "YARA-X workflow no longer validates public archive bytes and digest" unless
  bytes.include?("wc -c < \"$archive\"") && bytes.include?("sha256sum \"$archive\"")
abort "YARA-X workflow no longer requires empty workspace" unless bytes.include?("find \"$GITHUB_WORKSPACE\" -mindepth 1 -maxdepth 1")
abort "YARA-X workflow no longer deletes the public source archive" unless bytes.include?("rm -f -- \"$archive\"")
abort "YARA-X workflow no longer verifies analyzer cleanup" unless bytes.include?("-name '*.yarc'")

puts "YARA-X compatibility workflow verified: manual-only credential-free immutable-source execution"
