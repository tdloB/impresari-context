#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require "time"

path = Pathname.new(ARGV.fetch(0)).expand_path
abort "missing or symlinked YARA-X live receipt" unless path.file? && !path.symlink?
receipt = JSON.parse(path.read)

abort "YARA-X live receipt identity changed" unless
  receipt.fetch("schema_name") == "yara-x-artifact-compatibility-receipt" &&
  receipt.fetch("schema_version") == "1.0.0" &&
  receipt.fetch("profile_id") == "yara-x-artifact-compatibility-v1" &&
  receipt.fetch("profile_digest") == "sha256:ea2abe8460a1faab60b4ab2d854e48bdd45f1998106cd5e62229153155d254a8" &&
  receipt.fetch("source_archive_sha256") == "sha256:8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee" &&
  receipt.fetch("patch_sha256") == "sha256:b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd" &&
  receipt.fetch("ruleset_source_sha256") == "sha256:5379d03476eebf9c06379ad8d791d5ff1879c331300869d3eaf54c0e578c812b"

Time.iso8601(receipt.fetch("recorded_at"))
host = receipt.fetch("observed_host")
abort "YARA-X live receipt host is outside the exact candidate" unless
  host.fetch("runner_label") == "ubuntu-24.04" &&
  host.fetch("architecture") == "x86_64" &&
  host.fetch("kernel_release").match?(/\A[A-Za-z0-9._+-]+\z/) &&
  host.fetch("landlock_abi").match?(/\A[1-9][0-9]{0,2}\z/)

expected = {
  "empty" => [],
  "hex" => ["impresari_synthetic_hex_v1"],
  "literal" => ["impresari_synthetic_literal_v1"],
  "near_miss" => [],
  "wide" => ["impresari_synthetic_wide_v1"]
}
cases = receipt.fetch("cases")
abort "YARA-X compatibility case set changed" unless cases.map { |entry| entry.fetch("case_id") } == expected.keys
cases.each do |entry|
  case_id = entry.fetch("case_id")
  abort "YARA-X compatibility expectation changed" unless entry.fetch("expected_rule_identifiers") == expected.fetch(case_id)
  abort "YARA-X compatibility observation mismatched" unless entry.fetch("observed_rule_identifiers") == expected.fetch(case_id)
  abort "YARA-X compatibility output digest is malformed" unless entry.fetch("output_sha256").match?(/\Asha256:[0-9a-f]{64}\z/)
end

checks = receipt.fetch("checks")
abort "YARA-X compatibility did not pass every closed check" unless checks.values.all?
abort "YARA-X compatibility did not remain candidate-only" unless
  receipt.fetch("result") == "candidate_passed" &&
  receipt.fetch("limitations") == ["synthetic-only", "single-host-evidence", "mutable-runner-image", "unsigned-artifacts", "no-live-parser", "not-production", "not-iar-2", "not-a-detection-quality-or-safety-verdict"]

false_fields = %w[source_retained executable_retained compiled_rules_retained raw_output_retained network_used_by_analyzer credentials_used repository_content_scanned artifact_uploaded executable_admitted ruleset_admitted production_admitted iar_2_admitted detection_quality_claimed malware_free_claimed authority_added]
abort "YARA-X live receipt overclaims authority" unless false_fields.none? { |field| receipt.fetch(field) }

%w[executable_sha256 compiled_rules_sha256].each do |field|
  abort "YARA-X live receipt #{field} is malformed" unless receipt.fetch(field).match?(/\Asha256:[0-9a-f]{64}\z/)
end

puts "YARA-X synthetic compatibility: result=candidate_passed host=#{host.fetch('kernel_release')} executable=#{receipt.fetch('executable_sha256')} rules=#{receipt.fetch('compiled_rules_sha256')} cases=5 production=false iar_2=false"
