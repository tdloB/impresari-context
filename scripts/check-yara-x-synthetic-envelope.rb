#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b"
FIXTURES = {
  "invalid/yara-x-synthetic-runner-envelope-overclaim.json" => "e0b0859cdb75bd5a470d78fd758b0cf73f7d8b8aa755315e2065e565bfa2407c",
  "valid/yara-x-synthetic-runner-envelope-control.json" => "95129d28d23e0211b4ec666bbdbc6b714a0ac7fe224cc21e3f5a4aac04a0dd79",
  "valid/yara-x-synthetic-runner-envelope-profile.json" => PROFILE_DIGEST,
  "valid/yara-x-synthetic-runner-envelope-receipt.json" => "da707e0501b26993b400ea78dca55cbc71cd694811af4a5604124a3126f49652"
}.freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid YARA-X synthetic envelope JSON: #{path}: #{e.message}"
end

profile_path = ROOT.join("profiles/v1/yara-x-synthetic-runner-envelope-v1.json")
abort "missing or symlinked synthetic envelope profile" unless profile_path.file? && !profile_path.symlink?
abort "synthetic envelope profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
sidecar = ROOT.join("profiles/v1/yara-x-synthetic-runner-envelope-v1.sha256").read.strip
abort "synthetic envelope profile sidecar changed" unless sidecar == "#{PROFILE_DIGEST}  yara-x-synthetic-runner-envelope-v1.json"
fixture_root = ROOT.join("tests/conformance/v1")
abort "synthetic envelope profile fixture drifted" unless
  profile_path.binread == fixture_root.join("valid/yara-x-synthetic-runner-envelope-profile.json").binread

profile = json(profile_path)
abort "synthetic envelope cases changed" unless profile.fetch("synthetic_cases") == %w[valid-match valid-no-match]
abort "synthetic envelope adapter binding changed" unless
  profile.fetch("adapter_profile_digest") == "sha256:e444a5fd2675a01c85370e01c9456db4dfe214e09b5887d237ee06ac30871e7c"
abort "synthetic envelope limit changed" unless profile.fetch("limits") == {
  "max_stdout_bytes" => "131072", "max_stderr_bytes" => "0", "timeout_seconds" => "10",
  "processes" => "4", "memory_bytes" => "536870912", "cpu_quota_us" => "100000",
  "cpu_period_us" => "100000"
}
abort "synthetic envelope isolation weakened" unless profile.fetch("isolation").values_at(
  "fresh_cgroup", "atomic_initial_placement", "read_only_synthetic_job",
  "network_denied", "credentials_denied", "cleanup_required"
).all?
abort "synthetic envelope input authority expanded" if profile.fetch("emitter_contract").values_at(
  "repository_input", "rule_input", "arbitrary_arguments", "environment_input",
  "network_destinations", "credentials"
).any?
claims = profile.fetch("claims")
abort "synthetic emitter claim missing" unless claims.fetch("synthetic_emitter_executed")
abort "synthetic envelope overclaims" if (claims.keys - ["synthetic_emitter_executed"]).any? { |key| claims.fetch(key) }

provenance = json(fixture_root.join("yara-x-synthetic-runner-envelope-fixture-provenance.json"))
recorded = provenance.fetch("fixtures").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "synthetic envelope fixture provenance changed" unless recorded == FIXTURES
FIXTURES.each do |relative, digest|
  path = fixture_root.join(relative)
  abort "missing or symlinked synthetic envelope fixture" unless path.file? && !path.symlink?
  abort "synthetic envelope fixture digest changed: #{relative}" unless Digest::SHA256.file(path).hexdigest == digest
end
%w[malware_content third_party_content repository_source_content credential_content network_capture_content yara_x_executed analyzer_executed production_admitted authority_added].each do |key|
  abort "synthetic envelope provenance crossed #{key}" if provenance.fetch(key)
end

receipt = json(fixture_root.join("valid/yara-x-synthetic-runner-envelope-receipt.json"))
abort "synthetic envelope fixture lost execution evidence" unless
  receipt.fetch("synthetic_emitter_executed") && receipt.fetch("synthetic_emitter_os_confined") &&
    receipt.fetch("emitter_stderr_empty") && receipt.fetch("in_memory_composition_complete") &&
    receipt.fetch("job_removed") && receipt.fetch("cgroup_removed")
%w[raw_output_retained yara_x_executed analyzer_executed production_admitted iar_2_admitted detection_quality_claimed safety_claimed authority_added].each do |key|
  abort "synthetic envelope receipt overclaims #{key}" if receipt.fetch(key)
end

source = ROOT.join("crates/context-yara-x-envelope/src/lib.rs").read
emitter = ROOT.join("crates/context-yara-x-envelope/src/emitter.rs").read
runner = ROOT.join("crates/context-analyzer-runner/src/lib.rs").read
launcher = ROOT.join("platform/linux-yara-x-compatibility/launcher.c").read
rehearsal = ROOT.join("scripts/yara-x-synthetic-envelope.sh").read
abort "synthetic envelope profile digest drifted from Rust" unless source.include?("sha256:#{PROFILE_DIGEST}")
abort "synthetic emitter gained file, network, or embedded-file input" if emitter.match?(/std::fs|std::net|File::|include_bytes!|include_str!/)
abort "synthetic emitter no longer rejects extra arguments" unless emitter.include?("arguments.next().is_some()")
abort "synthetic envelope added another Rust process launch site" unless runner.scan("Command::new").length == 1
abort "Linux launcher lost the closed synthetic mode" unless
  launcher.include?("--synthetic-envelope") && launcher.include?("run_synthetic_envelope_child")
abort "Linux launcher does not require empty emitter stderr" unless
  launcher.include?("if (synthetic_envelope && result == 0)")
abort "synthetic envelope build flags are not target-specific" unless
  rehearsal.include?("CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+crt-static'") &&
    rehearsal.include?("CARGO_TARGET_DIR=\"$build_root\"") &&
    rehearsal.include?("--target x86_64-unknown-linux-gnu")
abort "synthetic envelope build artifacts are not ephemeral" unless
  rehearsal.include?("build_root=\"$runtime_root/cargo-target\"") &&
    rehearsal.include?("$build_root/x86_64-unknown-linux-gnu/release/impresari-yara-x-synthetic-emitter") &&
    rehearsal.include?("$build_root/x86_64-unknown-linux-gnu/release/impresari-yara-x-synthetic-envelope")

puts "YARA-X synthetic envelope verified: cases=2 process=isolated-synthetic-only yara_x_executed=false production_admitted=false"
