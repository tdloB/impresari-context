#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "tempfile"

root = Pathname.new(__dir__).join("..").expand_path
profile_path = root.join("profiles/v1/yara-x-reproducibility-diagnostic-v1.json")
sidecar_path = root.join("profiles/v1/yara-x-reproducibility-diagnostic-v1.sha256")
runner_path = root.join("scripts/yara-x-reproducibility-diagnostic.sh")
receipt_tool = root.join("scripts/yara-x-reproducibility-receipt.rb")
expected_digest = "4948ca0a448f1083cc3fe52519b57f62555c319146e91ff0999f696d69a8dbf4"

[profile_path, sidecar_path, runner_path, receipt_tool].each do |path|
  abort "missing or symlinked YARA-X reproducibility member: #{path}" unless path.file? && !path.symlink?
end
abort "YARA-X reproducibility profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == expected_digest
abort "YARA-X reproducibility sidecar changed" unless sidecar_path.read.strip == "#{expected_digest}  yara-x-reproducibility-diagnostic-v1.json"

profile = JSON.parse(profile_path.read)
abort "YARA-X reproducibility profile identity changed" unless
  profile.fetch("schema_name") == "yara-x-reproducibility-diagnostic-profile" &&
  profile.fetch("schema_version") == "1.0.0" &&
  profile.fetch("profile_id") == "yara-x-reproducibility-diagnostic-v1" &&
  profile.fetch("profile_version") == "1.0.0"
abort "YARA-X reproducibility source changed" unless
  profile.dig("source", "tag_commit_sha1") == "60ad06971467029e77967e59d580cbbe85a1474d" &&
  profile.dig("source", "tree_sha1") == "4ca76a9e411067422aecda3998b9297a254306f2" &&
  profile.dig("source", "source_date_epoch") == "1787565021" &&
  profile.dig("source", "archive_sha256") == "sha256:8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee"
abort "YARA-X reproducibility build boundary changed" unless
  profile.dig("build", "clean_source_roots") == "4" &&
  profile.dig("build", "clean_target_roots") == "4" &&
  profile.dig("build", "dependency_fetches") == "1" &&
  profile.dig("build", "builds_after_fetch_are_offline") == true &&
  profile.dig("build", "canonical_source_path") == "/usr/src/yara-x" &&
  profile.dig("build", "canonical_target_path") == "/usr/src/yara-x/target"
expected_results = %w[
  baseline_same_canonical_same
  baseline_changed_canonical_same
  baseline_changed_canonical_changed
  baseline_same_canonical_changed
]
abort "YARA-X reproducibility results changed" unless profile.fetch("closed_results") == expected_results
abort "YARA-X reproducibility profile gained authority" unless profile.fetch("claims").values.all? { |value| value == false }

runner = runner_path.read
required = [
  "GITHUB_ACTIONS", "RUNNER_ENVIRONMENT", "github-hosted",
  "8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee",
  "b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd",
  "e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e",
  "cargo +1.93.0 fetch --locked --target x86_64-unknown-linux-gnu",
  "CARGO_NET_OFFLINE=true", "--offline --frozen --locked",
  "SOURCE_DATE_EPOCH=1787565021", "--remap-path-prefix=",
  "baseline-a baseline-b canonical-a canonical-b",
  "trap cleanup EXIT HUP INT TERM",
  "analyzer_executed=false artifact_uploaded=false production=false iar_2=false"
]
required.each { |fragment| abort "YARA-X reproducibility runner lost #{fragment.inspect}" unless runner.include?(fragment) }
forbidden = [/yr\s+(?:scan|compile)/, /rules\.yar/, /upload-artifact/, /secrets\./, /GITHUB_TOKEN/, /GH_TOKEN/, /sudo\b/]
forbidden.each { |pattern| abort "YARA-X reproducibility runner crossed #{pattern.inspect}" if runner.match?(pattern) }
abort "YARA-X reproducibility source download is not singular" unless runner.scan("https://codeload.github.com/VirusTotal/yara-x/tar.gz/").length == 1

digests = ["1" * 64, "2" * 64, "3" * 64, "3" * 64]
stdout, stderr, status = Open3.capture3("ruby", receipt_tool.to_s, "emit", *digests)
abort "YARA-X reproducibility receipt emitter failed: #{stderr}" unless status.success?
Tempfile.create(["yara-x-reproducibility", ".json"]) do |file|
  file.write(stdout)
  file.flush
  verify_out, verify_err, verify_status = Open3.capture3("ruby", receipt_tool.to_s, "verify", file.path)
  abort "YARA-X reproducibility receipt verifier failed: #{verify_err}" unless verify_status.success? && verify_out.include?("receipt verified")
  changed = JSON.parse(stdout).merge("authority_added" => true)
  file.rewind
  file.truncate(0)
  file.write(JSON.generate(changed))
  file.flush
  _, _, changed_status = Open3.capture3("ruby", receipt_tool.to_s, "verify", file.path)
  abort "YARA-X reproducibility receipt accepted authority" if changed_status.success?
end

puts "YARA-X reproducibility contract verified: four ephemeral offline builds, closed outcomes, zero authority"
