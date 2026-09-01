#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "a7757809eae545bea1fa08d64195262b4e99fae8c2f222af9c28dce04b195391"
PATCH_DIGEST = "b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd"
RULE_DIGEST = "7769b61b7570e62f3b55eb615ffb5a6249862b9f267d1ad6305eda02e10d2c68"
ARCHIVE_DIGEST = "8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee"
FIXTURE_DIGESTS = {
  "invalid/yara-x-artifact-compatibility-overclaim.json" => "ca599d7f94236252b0df39291e64075faf441dbdd265d11705d2dfecc7832f9f",
  "valid/yara-x-artifact-compatibility-receipt.json" => "6ba8dbacf0969562226a2a2aa74309b020f4f67b6fd6803cfdee45da317f4c27"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid YARA-X compatibility JSON: #{path}: #{e.message}"
end

def exact(path, digest)
  abort "missing or symlinked YARA-X compatibility input: #{path}" unless path.file? && !path.symlink?
  abort "YARA-X compatibility input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

profile_path = ROOT.join("profiles/v1/yara-x-artifact-compatibility-v1.json")
patch_path = ROOT.join("third_party/yara-x/v1.20.0/impresari-module-free.patch")
rule_path = ROOT.join("rules/yara-x/synthetic-compatibility-v1.yar")
runner_path = ROOT.join("scripts/yara-x-artifact-compatibility.sh")
exact(profile_path, PROFILE_DIGEST)
exact(patch_path, PATCH_DIGEST)
exact(rule_path, RULE_DIGEST)

runner_text = runner_path.read
abort "YARA-X compatibility cleanup no longer restores disposable owner permissions" unless
  runner_text.include?('chmod -R u+rwX "$cleanup_root" || :')
abort "YARA-X compatibility cleanup gained unsafe permission widening" if
  runner_text.match?(/chmod[^\n]*(?:a\+w|o\+w|g\+w|0777)/)

sidecar = ROOT.join("profiles/v1/yara-x-artifact-compatibility-v1.sha256").read.strip
abort "YARA-X compatibility checksum record changed" unless
  sidecar == "#{PROFILE_DIGEST}  yara-x-artifact-compatibility-v1.json"

profile = json(profile_path)
source = profile.fetch("source")
abort "YARA-X compatibility source identity changed" unless
  source.fetch("release_tag") == "v1.20.0" &&
  source.fetch("tag_commit_sha1") == "60ad06971467029e77967e59d580cbbe85a1474d" &&
  source.fetch("archive_url") == "https://codeload.github.com/VirusTotal/yara-x/tar.gz/60ad06971467029e77967e59d580cbbe85a1474d" &&
  source.fetch("archive_sha256") == "sha256:#{ARCHIVE_DIGEST}" &&
  source.fetch("archive_bytes") == "57759292"

patch = profile.fetch("patch")
abort "YARA-X compatibility patch identity changed" unless
  patch.fetch("path") == patch_path.relative_path_from(ROOT).to_s &&
  patch.fetch("sha256") == "sha256:#{PATCH_DIGEST}" &&
  patch.fetch("patched_cargo_lock_sha256") == "sha256:e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e"

patch_text = patch_path.read
changed_files = patch_text.scan(%r{^diff --git a/(\S+) b/(\S+)$}).map do |before, after|
  abort "YARA-X patch changes unequal paths" unless before == after
  before
end
abort "YARA-X patch scope expanded" unless changed_files == ["Cargo.toml", "cli/Cargo.toml", "Cargo.lock"]
abort "YARA-X patch did not disable default modules" unless patch_text.include?('default-features = false')
abort "YARA-X patch retained parallel compilation" if patch_text.lines.grep(/^\+.*parallel-compilation/).any?
abort "YARA-X patch lost exact lock updates" unless
  patch_text.include?('+version = "0.9.20"') && patch_text.include?('+version = "0.9.11"')

build = profile.fetch("build")
abort "YARA-X compatibility build expanded" unless
  build.fetch("target") == "x86_64-unknown-linux-gnu" &&
  build.fetch("rust_toolchain") == "1.93.0" &&
  build.fetch("cargo_features") == ["pulley"] &&
  build.fetch("rustflags") == "-C target-feature=+crt-static" &&
  build.fetch("command") == ["cargo", "+1.93.0", "build", "--frozen", "--locked", "--profile", "release-lto", "--package", "yara-x-cli", "--features", "pulley", "--target", "x86_64-unknown-linux-gnu"] &&
  !build.fetch("build_output_retained") &&
  !build.fetch("runner_image_digest_pinned") &&
  !build.fetch("reproducibility_claimed")

features = profile.fetch("selected_feature_boundary")
abort "YARA-X compatibility feature graph expanded" unless
  features.fetch("enabled") == ["constant-folding", "exact-atoms", "fast-regexp", "generate-proto-code", "pulley"] &&
  %w[default_modules_enabled parallel_compilation_enabled rsa_reachable x509_parser_reachable spin_reachable wasmtime_wasi_reachable cap_std_reachable].none? { |key| features.fetch(key) } &&
  features.fetch("memmap2_version") == "0.9.11" &&
  features.fetch("crossbeam_epoch_version") == "0.9.20"

review = profile.fetch("dependency_review")
ignored = review.fetch("ignored_advisories").map { |entry| entry.fetch("id") }
abort "YARA-X compatibility advisory disposition expanded" unless
  ignored == %w[RUSTSEC-2023-0071 RUSTSEC-2026-0222 RUSTSEC-2026-0269] &&
  review.fetch("new_or_unreviewed_advisory_fails_closed")

ruleset = profile.fetch("ruleset")
expected_rules = %w[impresari_synthetic_hex_v1 impresari_synthetic_literal_v1 impresari_synthetic_wide_v1]
abort "YARA-X synthetic ruleset identity changed" unless
  ruleset.fetch("source_sha256") == "sha256:#{RULE_DIGEST}" &&
  ruleset.fetch("rule_identifiers") == expected_rules &&
  ruleset.fetch("imports").empty? && ruleset.fetch("includes").empty? &&
  %w[regular_expressions_present base64_present xor_present repository_rules_allowed compiled_rules_retained signature_present ruleset_admitted].none? { |key| ruleset.fetch(key) }

rule_text = rule_path.read
observed_rules = rule_text.scan(/^rule\s+([a-z0-9_]+)\s*:/).flatten.sort
abort "YARA-X synthetic rule identifiers changed" unless observed_rules == expected_rules
abort "YARA-X synthetic rules gained a forbidden surface" if
  rule_text.match?(/^\s*(import|include)\b|\b(base64|xor)\b|\/[^\n]+\//i)

isolation = profile.fetch("isolation")
abort "YARA-X compatibility isolation boundary weakened" unless
  isolation.fetch("backend") == "landlock-seccomp-cgroup-v2" &&
  isolation.fetch("existing_composite_check") == "scripts/check-linux-composite-feasibility.sh" &&
  isolation.fetch("transient_delegation_launch_sites_added") == "0" &&
  isolation.fetch("fresh_cgroup_per_scan") && isolation.fetch("atomic_initial_cgroup_placement") &&
  isolation.fetch("network_denied") && isolation.fetch("read_access_limited_to_staged_job") &&
  isolation.fetch("writable_filesystem_bytes") == "0" && isolation.fetch("cleanup_required")

observed_at = Time.iso8601(profile.dig("freshness", "observed_at"))
expires_at = Time.iso8601(profile.dig("freshness", "expires_at"))
abort "YARA-X compatibility evidence window changed" unless expires_at - observed_at == 14 * 86_400

claims = profile.fetch("checkpoint_claims")
true_claims = %w[source_identity_frozen patch_identity_frozen ruleset_source_created]
false_claims = claims.keys - true_claims
abort "YARA-X compatibility identity claims are incomplete" unless true_claims.all? { |key| claims.fetch(key) }
abort "YARA-X compatibility checkpoint overclaims authority" unless false_claims.none? { |key| claims.fetch(key) }

rule_files = Dir.glob(ROOT.join("rules/**/*.{yar,yara,yarc}").to_s)
  .map { |path| Pathname.new(path).relative_path_from(ROOT).to_s }
  .sort
abort "unexpected YARA-X rule artifact entered the repository" unless rule_files == [
  "rules/yara-x/production-v1-candidate.yar",
  "rules/yara-x/synthetic-compatibility-v1.yar"
]

provenance = json(ROOT.join("tests/conformance/v1/yara-x-artifact-compatibility-fixture-provenance.json"))
recorded = provenance.fetch("fixtures").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "YARA-X compatibility fixture provenance is incomplete" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(ROOT.join("tests/conformance/v1", relative), digest) }
%w[yara_x_source_content yara_x_executable_content compiled_rule_content malware_content repository_scan_input_content matched_byte_content credential_content network_capture_content authority_added].each do |key|
  abort "YARA-X compatibility fixture provenance crossed #{key}" unless provenance.fetch(key) == false
end

receipt = json(ROOT.join("tests/conformance/v1/valid/yara-x-artifact-compatibility-receipt.json"))
abort "YARA-X compatibility fixture does not bind the profile" unless receipt.fetch("profile_digest") == "sha256:#{PROFILE_DIGEST}"
false_receipt_fields = %w[source_retained executable_retained compiled_rules_retained raw_output_retained network_used_by_analyzer credentials_used repository_content_scanned artifact_uploaded executable_admitted ruleset_admitted production_admitted iar_2_admitted detection_quality_claimed malware_free_claimed authority_added]
abort "YARA-X compatibility receipt fixture overclaims authority" unless false_receipt_fields.none? { |key| receipt.fetch(key) }

puts "YARA-X artifact compatibility contract verified: source=v1.20.0 patch=#{PATCH_DIGEST} rules=3 execution=hosted-synthetic-only"
