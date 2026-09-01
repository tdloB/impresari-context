#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da"
SCHEMA_DIGEST = "42b8e7cf7105ded8cd370a23f6f2add6d7ee825dc8ca743333073b9c6037a6e3"

def exact(path, digest)
  abort "missing or symlinked ADR-0104 member: #{path}" unless path.file? && !path.symlink?
  abort "ADR-0104 member digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

profile_path = ROOT.join("profiles/v1/yara-x-retained-engine-candidate-v1.json")
sidecar_path = ROOT.join("profiles/v1/yara-x-retained-engine-candidate-v1.sha256")
schema_path = ROOT.join("schemas/v1/yara-x-retained-engine-candidate.schema.json")
workflow_path = ROOT.join(".github/workflows/yara-x-retained-engine-candidate.yml")
runner_path = ROOT.join("scripts/yara-x-retained-engine-candidate.sh")
packager_path = ROOT.join("scripts/package-yara-x-retained-engine-candidate.rb")
verifier_path = ROOT.join("scripts/verify-yara-x-retained-engine-candidate.rb")

exact(profile_path, PROFILE_DIGEST)
exact(schema_path, SCHEMA_DIGEST)
abort "ADR-0104 profile sidecar changed" unless sidecar_path.read.strip == "#{PROFILE_DIGEST}  yara-x-retained-engine-candidate-v1.json"
[workflow_path, runner_path, packager_path, verifier_path].each do |path|
  abort "missing or symlinked ADR-0104 implementation member: #{path}" unless path.file? && !path.symlink?
end

profile = JSON.parse(profile_path.read)
abort "ADR-0104 profile identity changed" unless
  profile.fetch("schema_name") == "yara-x-retained-engine-candidate-profile" &&
  profile.fetch("schema_version") == "1.0.0" &&
  profile.fetch("profile_id") == "yara-x-retained-engine-candidate-v1" &&
  profile.fetch("profile_version") == "1.0.0"
abort "ADR-0104 build identity changed" unless
  profile.dig("source", "commit_sha1") == "60ad06971467029e77967e59d580cbbe85a1474d" &&
  profile.dig("source", "archive_sha256") == "sha256:8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee" &&
  profile.dig("source", "patch_sha256") == "sha256:b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd" &&
  profile.dig("source", "patched_lock_sha256") == "sha256:e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e" &&
  profile.dig("build", "image_manifest_sha256") == "sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28" &&
  profile.dig("build", "target") == "x86_64-unknown-linux-gnu" &&
  profile.dig("build", "toolchain") == "1.93.0" &&
  profile.dig("build", "profile") == "release-lto" &&
  profile.dig("build", "features") == ["pulley"]
abort "ADR-0104 storage boundary changed" unless
  profile.dig("artifact", "visibility") == "authenticated-repository-readers" &&
  profile.dig("artifact", "repository_visibility") == "public" &&
  profile.dig("artifact", "anonymous_download") == false &&
  profile.dig("artifact", "maintainer_only") == false &&
  profile.dig("artifact", "release_asset") == false &&
  profile.dig("artifact", "retention_days") == 7 &&
  profile.dig("artifact", "max_bytes") == 268_435_456 &&
  profile.dig("artifact", "members").length == 12 &&
  profile.dig("artifact", "overwrite") == false
abort "ADR-0104 profile gained authority" if profile.fetch("claims").values.any?

workflow = workflow_path.read
required_workflow = [
  "on:\n  workflow_dispatch:\n",
  "permissions: {}",
  "github.repository == 'tdloB/impresari-context' && github.ref == 'refs/heads/main'",
  "runs-on: ubuntu-24.04",
  "bash ./scripts/yara-x-retained-engine-candidate.sh",
  "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
  "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c",
  "retention-days: 7",
  "overwrite: false",
  "compression-level: 0",
  "Verify without extracting or executing candidate",
  "verify-yara-x-retained-engine-candidate.rb",
  "if: always()"
]
required_workflow.each { |fragment| abort "ADR-0104 workflow lost #{fragment.inspect}" unless workflow.include?(fragment) }
forbidden_workflow = [
  /^\s+(?:pull_request|pull_request_target|push|schedule|repository_dispatch):/,
  /workflow_dispatch:\s*\n\s+inputs:/,
  /actions\/checkout@/,
  /actions\/cache@/,
  /\b(?:secrets\.|github\.token|GITHUB_TOKEN|GH_TOKEN)\b/,
  /^permissions:\s+write-all/,
  /(?:contents|actions|checks|deployments|id-token|issues|packages|pull-requests|releases|security-events|statuses):\s+write/,
  /attest(?:ation)?|sigstore|cosign|release asset|gh release/,
  /retention-days:\s*(?!7\b)\d+/,
  /overwrite:\s*true/
]
forbidden_workflow.each { |pattern| abort "ADR-0104 workflow crossed #{pattern.inspect}" if workflow.match?(pattern) }
abort "ADR-0104 workflow must contain exactly two jobs" unless workflow.scan(/^  (?:build|verify):\n/).length == 2
abort "ADR-0104 workflow upload count changed" unless workflow.scan("actions/upload-artifact@").length == 1
abort "ADR-0104 workflow download count changed" unless workflow.scan("actions/download-artifact@").length == 1
abort "ADR-0104 workflow gained multiple artifact names" unless workflow.scan(/^\s+name: yara-x-v1\.20\.0-linux-x86_64-engine-candidate$/).length == 2

runner = runner_path.read
required_runner = [
  "docker.io/library/rust@sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28",
  "docker run --rm --network none --read-only",
  "CARGO_NET_OFFLINE=true",
  "cargo build --offline --frozen --locked --profile release-lto",
  "--package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu",
  "8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee",
  "b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd",
  "e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e",
  "CARGO_TARGET_DIR=/cargo/cargo-audit-target",
  "cargo audit --file Cargo.lock",
  "rm -rf -- /cargo/cargo-audit-target",
  "verify-yara-x-retained-engine-candidate.rb",
  "trap cleanup EXIT HUP INT TERM",
  "executed=false admitted=false production=false iar_2=false"
]
required_runner.each { |fragment| abort "ADR-0104 runner lost #{fragment.inspect}" unless runner.include?(fragment) }
forbidden_runner = [/\byr\s+(?:scan|compile)\b/, /\.yar(?:c)?\b/, /secrets\./, /GITHUB_TOKEN|GH_TOKEN/, /sudo\b/, /repository.*scan/i]
forbidden_runner.each { |pattern| abort "ADR-0104 runner crossed #{pattern.inspect}" if runner.match?(pattern) }

verifier = verifier_path.read
abort "ADR-0104 verifier gained process-launch capability" if verifier.match?(/(?:Open3|IO\.popen|Kernel\.(?:system|exec|spawn)|Process\.spawn|`)/)
abort "ADR-0104 verifier gained network capability" if verifier.match?(/(?:Net::HTTP|TCPSocket|UDPSocket|Socket\.)/)
abort "ADR-0104 verifier lost regular-file/link rejection" unless verifier.include?("entry.file?") && verifier.include?("non-regular")
abort "ADR-0104 verifier lost exact member closure" unless verifier.include?("observed.keys.sort == MEMBERS.sort")
abort "ADR-0104 verifier lost all-false claims" unless verifier.include?("FALSE_CLAIMS") && verifier.include?("admitted=false production=false iar_2=false")

packager = packager_path.read
abort "ADR-0104 packager invokes the retained candidate" if packager.match?(/(?:Open3\.capture\w*|Kernel\.(?:system|exec|spawn)|Process\.spawn|IO\.popen).*\byr\b/i)
abort "ADR-0104 packager gained rules" if packager.match?(/rules?\.(?:yar|yarc)/)
abort "ADR-0104 packager lost deterministic archive controls" unless
  packager.include?("--sort=name") && packager.include?("--mtime=@1787565021") && packager.include?("gzip.mtime = 0")

puts "ADR-0104 retained YARA-X candidate verified: manual, no-secret, authenticated-reader seven-day non-release artifact; verifier never executes candidate; admission=false"
