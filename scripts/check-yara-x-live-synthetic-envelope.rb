#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "2aa5e203f71089688baa41556c6775e7dcca98c7e6aab726442ff99fb5f8cd26"
FIXTURES = {
  "invalid/yara-x-live-synthetic-envelope-overclaim.json" => "89888ead51ab9fc94596dd98c268c2b2a79df2192ffd68a1750aaf0c1ffa81b7",
  "valid/yara-x-live-synthetic-envelope-control.json" => "8936b40db4274e340eaf3056ce3c360701570f64988cd26aef69e57f06da5e6e",
  "valid/yara-x-live-synthetic-envelope-profile.json" => PROFILE_DIGEST,
  "valid/yara-x-live-synthetic-envelope-receipt.json" => "3f35766b0dd28be6cc94dcd39b52dd593ddc6a87f11bee63778946005ba49ca0"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid live YARA-X synthetic envelope JSON: #{path}: #{e.message}"
end

def exact(path, digest)
  abort "missing or symlinked live YARA-X input: #{path}" unless path.file? && !path.symlink?
  abort "live YARA-X input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

profile_path = ROOT.join("profiles/v1/yara-x-live-synthetic-envelope-v1.json")
exact(profile_path, PROFILE_DIGEST)
sidecar = ROOT.join("profiles/v1/yara-x-live-synthetic-envelope-v1.sha256").read.strip
abort "live YARA-X profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  yara-x-live-synthetic-envelope-v1.json"

fixture_root = ROOT.join("tests/conformance/v1")
abort "live YARA-X profile fixture drifted" unless
  profile_path.binread == fixture_root.join("valid/yara-x-live-synthetic-envelope-profile.json").binread
profile = json(profile_path)
profile_schema = json(ROOT.join("schemas/v1/yara-x-live-synthetic-envelope-profile.schema.json"))
abort "live YARA-X profile schema no longer freezes the exact profile" unless profile_schema.fetch("const") == profile
abort "live YARA-X case set changed" unless profile.fetch("cases") == %w[empty hex literal near-miss wide]
abort "live YARA-X adapter binding changed" unless
  profile.fetch("adapter_profile_digest") == "sha256:e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c"
abort "live YARA-X runner site changed" unless
  profile.fetch("process_launch_site") == "crates/context-analyzer-runner/src/lib.rs"
abort "live YARA-X isolation backend changed" unless
  profile.fetch("isolation_backend") == "linux-cgroup-v2-landlock-seccomp"
abort "live YARA-X input origin expanded" unless
  profile.fetch("input_origin") == "impresari-original-synthetic-generated"
claims = profile.fetch("claims")
abort "live YARA-X execution evidence was lost" unless
  claims.fetch("yara_x_executed") && claims.fetch("os_confined") && claims.fetch("synthetic_input_only")
closed_claims = claims.keys - %w[yara_x_executed os_confined synthetic_input_only]
abort "live YARA-X profile overclaims authority" if closed_claims.any? { |key| claims.fetch(key) }

provenance = json(fixture_root.join("yara-x-live-synthetic-envelope-fixture-provenance.json"))
recorded = provenance.fetch("fixtures").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "live YARA-X fixture provenance changed" unless recorded == FIXTURES
FIXTURES.each { |relative, digest| exact(fixture_root.join(relative), digest) }
%w[malware_content third_party_content repository_source_content credential_content network_capture_content yara_x_executed production_admitted authority_added].each do |key|
  abort "live YARA-X fixture provenance crossed #{key}" if provenance.fetch(key)
end

receipt = json(fixture_root.join("valid/yara-x-live-synthetic-envelope-receipt.json"))
abort "live YARA-X receipt lost bounded execution evidence" unless
  receipt.fetch("yara_x_executed") && receipt.fetch("os_confined") &&
    receipt.fetch("in_memory_composition_complete") && receipt.fetch("synthetic_input_only") &&
    receipt.fetch("job_removed") && receipt.fetch("cgroup_removed")
%w[raw_output_retained executable_admitted ruleset_admitted production_admitted iar_2_admitted detection_quality_claimed safety_claimed authority_added].each do |key|
  abort "live YARA-X receipt overclaims #{key}" if receipt.fetch(key)
end

envelope = ROOT.join("crates/context-yara-x-envelope/src/lib.rs").read
runner = ROOT.join("crates/context-analyzer-runner/src/lib.rs").read
rehearsal = ROOT.join("scripts/yara-x-artifact-compatibility.sh").read
composite = ROOT.join("scripts/check-linux-composite-feasibility.sh").read
abort "live YARA-X profile digest drifted from Rust" unless envelope.include?("sha256:#{PROFILE_DIGEST}")
abort "live YARA-X envelope gained a direct launch site" if envelope.match?(/Command\s*::\s*new/)
abort "Analyzer Runner no longer has exactly one process launch site" unless runner.scan("Command::new").length == 1
abort "live YARA-X capture no longer uses the audited runner" unless
  envelope.include?("capture_yara_x_compatibility_process") && runner.include?("capture_yara_x_compatibility_process")
abort "live coordinator build is not locked and target-specific" unless
  rehearsal.include?("cargo +1.98.0 build --locked --release") &&
    rehearsal.include?("--bin impresari-yara-x-live-synthetic-envelope") &&
    rehearsal.include?("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+crt-static'")
abort "live coordinator was not handed into the admitted composite boundary" unless
  composite.include?('YARA_X_LIVE_COORDINATOR="${YARA_X_LIVE_COORDINATOR:?}"') &&
    composite.include?('"$live_coordinator" < "$live_control" > "$live_output"')
abort "live envelope permits repository or credential input" if
  composite.match?(/repository_content_scanned:true|credentials_used:true|production_admitted:true|iar_2_admitted:true/)

puts "YARA-X live synthetic envelope verified: cases=5 yara_x_executed=true os_confined=true production_admitted=false"
