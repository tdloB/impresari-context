#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_RELATIVE = "profiles/v1/yara-x-ndjson-adapter-v1.json"
PROFILE_DIGEST = "e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c"
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
CRATE_ROOT = ROOT.join("crates/context-yara-x-adapter")

def read_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

profile_path = ROOT.join(PROFILE_RELATIVE)
abort "missing or symlinked YARA-X NDJSON profile" unless profile_path.file? && !profile_path.symlink?
abort "YARA-X NDJSON profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/yara-x-ndjson-adapter-v1.sha256").read.strip
abort "YARA-X NDJSON profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  yara-x-ndjson-adapter-v1.json"
fixture_profile = FIXTURE_ROOT.join("valid/yara-x-ndjson-adapter-profile.json")
abort "YARA-X NDJSON profile fixture drifted" unless profile_path.binread == fixture_profile.binread

profile = read_json(profile_path)
abort "YARA-X NDJSON adapter identity changed" unless
  profile.fetch("profile_id") == "yara-x-ndjson-adapter-v1" &&
    profile.dig("adapter", "analyzer_id") == "impresari.yara-x" &&
    profile.dig("adapter", "input_format") == "yara-x-v1.20.0-one-line-ndjson" &&
    profile.dig("adapter", "result_origin") == "original_synthetic_fixture" &&
    profile.dig("adapter", "result_identity_domain") == "impresari-context/yara-x-normalized-result/v1"
abort "YARA-X NDJSON limits changed" unless profile.fetch("limits") == {
  "max_input_bytes" => "131072",
  "max_staged_path_bytes" => "4096",
  "max_rules" => "256",
  "max_observations" => "256",
  "max_tags_per_observation" => "32",
  "max_ranges_per_observation" => "32",
  "max_total_ranges" => "8192",
  "max_identifier_bytes" => "128",
  "max_normalized_output_bytes" => "2097152"
}
abort "YARA-X NDJSON adapter gained a runtime capability" if profile.fetch("runtime_capabilities").values.any?
abort "YARA-X NDJSON adapter gained a claim" if profile.fetch("claims").values.any?
%w[repository_source_bytes matched_bytes raw_output_retention commands arguments rule_source network_destinations credentials].each do |key|
  abort "YARA-X NDJSON adapter input gained #{key}" if profile.dig("input_contract", key)
end

provenance = read_json(FIXTURE_ROOT.join("yara-x-ndjson-adapter-fixture-provenance.json"))
expected_paths = %w[
  invalid/yara-x-ndjson-adapter-control-path.json
  invalid/yara-x-normalized-result-overclaim.json
  valid/yara-x-ndjson-adapter-control.json
  valid/yara-x-ndjson-adapter-profile.json
  valid/yara-x-normalized-result.json
  yara-x-ndjson/invalid-duplicate-field.ndjson
  yara-x-ndjson/invalid-duplicate-tag.ndjson
  yara-x-ndjson/invalid-extra-line.ndjson
  yara-x-ndjson/invalid-marker.ndjson
  yara-x-ndjson/invalid-path.ndjson
  yara-x-ndjson/invalid-range.ndjson
  yara-x-ndjson/invalid-unknown-field.ndjson
  yara-x-ndjson/valid-match.ndjson
  yara-x-ndjson/valid-no-match.ndjson
]
entries = provenance.fetch("fixtures")
abort "YARA-X NDJSON fixture provenance is not closed and sorted" unless entries.map { |entry| entry.fetch("path") } == expected_paths
entries.each do |entry|
  path = FIXTURE_ROOT.join(entry.fetch("path")).cleanpath
  abort "YARA-X NDJSON fixture escapes fixture root" unless path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked YARA-X NDJSON fixture" unless path.file? && !path.symlink?
  abort "YARA-X NDJSON fixture digest changed: #{entry.fetch('path')}" unless Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
end
%w[malware_content third_party_content executable_content repository_source_content credential_content network_capture_content analyzer_output_content analyzer_executed authority_added].each do |key|
  abort "YARA-X NDJSON fixture provenance crossed #{key}" if provenance.fetch(key)
end

raw_root = FIXTURE_ROOT.join("yara-x-ndjson")
raw_paths = raw_root.glob("*.ndjson").map { |path| path.relative_path_from(FIXTURE_ROOT).to_s }.sort
expected_raw_paths = expected_paths.select { |path| path.start_with?("yara-x-ndjson/") }
abort "YARA-X NDJSON raw fixture set is not closed" unless raw_paths == expected_raw_paths
raw_root.glob("*.ndjson").each do |path|
  abort "YARA-X NDJSON fixture exceeded frozen input ceiling" if path.size > 131_072
  abort "YARA-X NDJSON fixture is missing terminal LF" unless path.binread.end_with?("\n")
end
abort "valid-match fixture is not exactly one line" unless raw_root.join("valid-match.ndjson").binread.count("\n") == 1
abort "valid-no-match fixture is not exactly one line" unless raw_root.join("valid-no-match.ndjson").binread.count("\n") == 1

control = read_json(FIXTURE_ROOT.join("valid/yara-x-ndjson-adapter-control.json"))
abort "YARA-X NDJSON control is not bound to the profile" unless
  control.fetch("profile_id") == profile.fetch("profile_id") &&
    control.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    !control.fetch("authority_added")
result_path = FIXTURE_ROOT.join("valid/yara-x-normalized-result.json")
result = read_json(result_path)
abort "YARA-X normalized result retained staged path" if result_path.read.include?(control.fetch("expected_staged_path"))
abort "YARA-X normalized result gained authority" unless
  result.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}" &&
    !result.fetch("raw_output_retained") && !result.fetch("source_bytes_retained") &&
    !result.fetch("matched_bytes_retained") && !result.fetch("path_emitted") &&
    !result.fetch("analyzer_executed") && !result.fetch("os_confined") &&
    !result.fetch("production_admitted") && !result.fetch("iar_2_admitted") &&
    !result.fetch("safety_claimed") && !result.fetch("authority_added")

cargo = CRATE_ROOT.join("Cargo.toml").read
expected_dependencies = %w[context-core serde serde_json sha2]
dependency_lines = cargo.lines.drop_while { |line| line.strip != "[dependencies]" }.drop(1).take_while { |line| !line.start_with?("[") }
actual_dependencies = dependency_lines.map { |line| line[/^([a-z0-9_-]+)\s*=/, 1] }.compact.sort
abort "YARA-X NDJSON crate dependency surface changed" unless actual_dependencies == expected_dependencies.sort

source = CRATE_ROOT.join("src/lib.rs").read.split("#[cfg(test)]", 2).first
banned = [
  "std::fs", "std::process", "std::net", "std::env", "std::time", "Command::", "File::",
  "Path::", "TcpStream", "UdpSocket", "SystemTime", "Instant::", "include_bytes!", "include_str!"
]
banned.each do |token|
  abort "YARA-X NDJSON production parser gained forbidden capability token #{token}" if source.include?(token)
end
abort "YARA-X NDJSON parser profile digest drifted" unless source.include?("sha256:#{PROFILE_DIGEST}")
abort "YARA-X NDJSON parser lost unknown-field denial" unless source.scan("deny_unknown_fields").length >= 5

puts "YARA-X NDJSON adapter verified: profile=sha256:#{PROFILE_DIGEST} original_synthetic=true analyzer_executed=false production_admitted=false iar_2_admitted=false"
