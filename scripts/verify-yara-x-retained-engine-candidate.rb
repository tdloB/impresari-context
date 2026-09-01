#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"
require "rubygems/package"
require "time"
require "zlib"

PROFILE_DIGEST = "c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da"
IMAGE_DIGEST = "7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28"
POLICY_DIGEST = "fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c"
MEMBERS = %w[
  DEPENDENCY-NOTICES.txt LICENSE MANIFEST.json SHA256SUMS
  dependency-closure.json engine-candidate.json license-disposition.json
  provenance.json reproducibility-disposition.json sbom.spdx.json
  vulnerability-disposition.json yr
].freeze
FALSE_CLAIMS = %w[
  signed attested published executable_admitted ruleset_present rules_compiled
  analyzer_executed repository_scanned credentials_accessed production_admitted
  iar_2_admitted detection_quality_claimed safety_claimed malware_free_claimed
  authority_added
].freeze

def parse_json(bytes, label)
  JSON.parse(bytes)
rescue JSON::ParserError => e
  abort "invalid #{label}: #{e.message}"
end

def exact_keys!(object, keys, label)
  abort "#{label} is not a closed object" unless object.is_a?(Hash) && object.keys.sort == keys.sort
end

archive = Pathname.new(ARGV.shift || abort("usage: verify-yara-x-retained-engine-candidate.rb ARCHIVE [RECEIPT]")).expand_path
receipt_argument = ARGV.shift
receipt_path = receipt_argument && Pathname.new(receipt_argument).expand_path
abort "unexpected verifier arguments" unless ARGV.empty?
stat = archive.lstat
abort "candidate archive must be a regular non-symlink file" unless stat.file? && !stat.symlink?
abort "candidate archive exceeds 256 MiB" if stat.size > 268_435_456

observed = {}
Zlib::GzipReader.open(archive.to_s) do |gzip|
  Gem::Package::TarReader.new(gzip) do |tar|
    tar.each do |entry|
      name = entry.full_name
      abort "candidate archive contains an unsafe or unexpected path: #{name}" unless MEMBERS.include?(name)
      abort "candidate archive contains a duplicate member: #{name}" if observed.key?(name)
      abort "candidate archive contains a link or non-regular member: #{name}" unless entry.file?
      abort "candidate archive member is too large: #{name}" if entry.size > 268_435_456
      digest = Digest::SHA256.new
      bytes = +""
      while (chunk = entry.read(1_048_576)) && !chunk.empty?
        digest.update(chunk)
        bytes << chunk unless name == "yr"
      end
      observed[name] = {"bytes" => entry.size, "sha256" => digest.hexdigest, "content" => bytes}
    end
  end
end
abort "candidate archive member set changed" unless observed.keys.sort == MEMBERS.sort

checksum_lines = observed.fetch("SHA256SUMS").fetch("content").lines.map(&:chomp)
expected_checksum_members = (MEMBERS - ["SHA256SUMS"]).sort
abort "candidate checksum member count changed" unless checksum_lines.length == expected_checksum_members.length
checksum_lines.zip(expected_checksum_members).each do |line, name|
  match = line.match(/\A([0-9a-f]{64})  ([A-Za-z0-9_.-]+)\z/)
  abort "invalid candidate checksum line" unless match && match[2] == name
  abort "candidate checksum mismatch: #{name}" unless match[1] == observed.fetch(name).fetch("sha256")
end

manifest = parse_json(observed.fetch("MANIFEST.json").fetch("content"), "candidate manifest")
exact_keys!(manifest, %w[schema_name schema_version target files], "candidate manifest")
abort "unexpected candidate manifest identity" unless
  manifest.fetch("schema_name") == "yara-x-engine-candidate-file-manifest" &&
  manifest.fetch("schema_version") == "1.0.0" &&
  manifest.fetch("target") == "x86_64-unknown-linux-gnu"
expected_payload = (MEMBERS - %w[MANIFEST.json SHA256SUMS]).sort
files = manifest.fetch("files")
abort "candidate manifest file count changed" unless files.length == expected_payload.length
files.zip(expected_payload).each do |file, name|
  exact_keys!(file, %w[path bytes sha256], "candidate manifest file")
  abort "candidate manifest path changed" unless file.fetch("path") == name
  abort "candidate manifest bytes changed: #{name}" unless file.fetch("bytes") == observed.fetch(name).fetch("bytes")
  abort "candidate manifest digest changed: #{name}" unless file.fetch("sha256") == "sha256:#{observed.fetch(name).fetch('sha256')}"
end

sbom = parse_json(observed.fetch("sbom.spdx.json").fetch("content"), "candidate SBOM")
abort "candidate SBOM is not SPDX 2.3" unless
  sbom.fetch("spdxVersion") == "SPDX-2.3" && sbom.fetch("dataLicense") == "CC0-1.0"
packages = sbom.fetch("packages")
abort "candidate SBOM package inventory is empty" if packages.empty?
abort "candidate SBOM package IDs are not unique" unless packages.map { |package| package.fetch("SPDXID") }.uniq.length == packages.length
abort "candidate SBOM introduced analyzed source files" if packages.any? { |package| package.fetch("filesAnalyzed") }

closure = parse_json(observed.fetch("dependency-closure.json").fetch("content"), "dependency closure")
exact_keys!(closure, %w[schema_name schema_version target features packages], "dependency closure")
abort "unexpected dependency closure identity" unless
  closure.fetch("schema_name") == "yara-x-selected-dependency-closure" &&
  closure.fetch("schema_version") == "1.0.0" &&
  closure.fetch("target") == "x86_64-unknown-linux-gnu" &&
  closure.fetch("features") == ["pulley"]
abort "dependency closure and SBOM diverged" unless closure.fetch("packages").length == packages.length
closure.fetch("packages").each do |package|
  exact_keys!(package, %w[name version source checksum license license_file], "dependency closure package")
  abort "dependency closure leaked a host path" if package.fetch("source").start_with?("/")
end

candidate = parse_json(observed.fetch("engine-candidate.json").fetch("content"), "engine candidate")
exact_keys!(candidate, %w[schema_name schema_version candidate_id policy_id policy_digest target state artifact evidence admitted claims], "engine candidate")
abort "engine candidate identity changed" unless
  candidate.fetch("schema_name") == "yara-x-engine-bundle-candidate" &&
  candidate.fetch("schema_version") == "1.0.0" &&
  candidate.fetch("policy_id") == "yara-x-production-admission-v1" &&
  candidate.fetch("policy_digest") == "sha256:#{POLICY_DIGEST}" &&
  candidate.fetch("target") == "x86_64-unknown-linux-gnu" &&
  candidate.fetch("state") == "missing_evidence" &&
  candidate.fetch("admitted") == false
artifact = candidate.fetch("artifact")
exact_keys!(artifact, %w[present sha256 bytes retained], "engine candidate artifact")
abort "engine candidate does not bind the retained executable" unless
  artifact.fetch("present") && artifact.fetch("retained") &&
  artifact.fetch("sha256") == "sha256:#{observed.fetch('yr').fetch('sha256')}" &&
  artifact.fetch("bytes") == observed.fetch("yr").fetch("bytes")
exact_keys!(candidate.fetch("claims"), %w[production_admitted iar_2_admitted repository_scan_authorized credential_access_authorized artifact_upload_authorized detection_quality_claimed safety_claimed malware_free_claimed authority_added], "engine candidate claims")
abort "engine candidate overclaims authority" if candidate.fetch("claims").values.any?
evidence = candidate.fetch("evidence")
exact_keys!(evidence, %w[locked_build dependency_closure sbom build_provenance vulnerability_disposition license_disposition reproducibility_disposition signature expiry revocation_identity], "engine candidate evidence")
abort "engine candidate smuggled signing or revocation evidence" if evidence.fetch("signature") || evidence.fetch("revocation_identity")
abort "engine candidate lost retention evidence" unless evidence.fetch("expiry")

provenance = parse_json(observed.fetch("provenance.json").fetch("content"), "build provenance")
exact_keys!(provenance, %w[schema_name schema_version impresari_source_commit upstream_commit upstream_archive_sha256 patch_sha256 lockfile_sha256 profile_sha256 toolchain target build_image build_image_index_sha256 runner_image runner_image_version runner_kernel runner_architecture advisory_database_commit network_disabled_for_build command], "build provenance")
abort "build provenance lost the digest-pinned image" unless
  provenance.fetch("build_image").end_with?("@sha256:#{IMAGE_DIGEST}") &&
  provenance.fetch("network_disabled_for_build") == true &&
  provenance.fetch("profile_sha256") == "sha256:#{PROFILE_DIGEST}"
abort "build provenance source commit differs from this run" if
  ENV["GITHUB_SHA"] && provenance.fetch("impresari_source_commit") != ENV.fetch("GITHUB_SHA")

vulnerability = parse_json(observed.fetch("vulnerability-disposition.json").fetch("content"), "vulnerability disposition")
license = parse_json(observed.fetch("license-disposition.json").fetch("content"), "license disposition")
reproducibility = parse_json(observed.fetch("reproducibility-disposition.json").fetch("content"), "reproducibility disposition")
exact_keys!(vulnerability, %w[schema_name schema_version cargo_audit_passed advisory_database_commit explicitly_dispositioned new_advisories_allowed human_reviewed production_approved], "vulnerability disposition")
exact_keys!(license, %w[schema_name schema_version upstream_license dependency_count metadata_complete notices_included human_reviewed production_approved], "license disposition")
exact_keys!(reproducibility, %w[schema_name schema_version same_job_reference_sha256 candidate_sha256 same_job_reference_matched digest_pinned_image_used cross_run_reproducibility_established cross_host_reproducibility_established production_approved], "reproducibility disposition")
abort "candidate vulnerability disposition overclaims approval" unless vulnerability.fetch("cargo_audit_passed") && !vulnerability.fetch("human_reviewed") && !vulnerability.fetch("production_approved")
abort "candidate license disposition overclaims approval" unless license.fetch("metadata_complete") && license.fetch("notices_included") && !license.fetch("human_reviewed") && !license.fetch("production_approved")
abort "candidate reproducibility disposition overclaims" unless
  reproducibility.fetch("digest_pinned_image_used") &&
  !reproducibility.fetch("cross_run_reproducibility_established") &&
  !reproducibility.fetch("cross_host_reproducibility_established") &&
  !reproducibility.fetch("production_approved")
abort "candidate reproducibility digest is detached" unless
  reproducibility.fetch("candidate_sha256") == "sha256:#{observed.fetch('yr').fetch('sha256')}"

archive_digest = Digest::SHA256.file(archive).hexdigest
if receipt_path
  abort "verification receipt requires GitHub Actions" unless ENV["GITHUB_ACTIONS"] == "true"
  created = Time.at(Time.now.to_i).utc
  claims = FALSE_CLAIMS.to_h { |key| [key, false] }
  receipt = {
    "schema_name" => "yara-x-retained-engine-candidate-verification-receipt",
    "schema_version" => "1.0.0",
    "profile_id" => "yara-x-retained-engine-candidate-v1",
    "profile_sha256" => "sha256:#{PROFILE_DIGEST}",
    "repository" => ENV.fetch("GITHUB_REPOSITORY"),
    "source_commit" => ENV.fetch("GITHUB_SHA"),
    "run_id" => ENV.fetch("GITHUB_RUN_ID"),
    "run_attempt" => ENV.fetch("GITHUB_RUN_ATTEMPT"),
    "build_job" => "build",
    "verify_job" => "verify",
    "artifact_id" => ENV.fetch("IMPRESARI_ARTIFACT_ID"),
    "artifact_name" => "yara-x-v1.20.0-linux-x86_64-engine-candidate",
    "requested_retention_days" => 7,
    "archive_sha256" => "sha256:#{archive_digest}",
    "archive_bytes" => archive.size,
    "executable_sha256" => "sha256:#{observed.fetch('yr').fetch('sha256')}",
    "executable_bytes" => observed.fetch("yr").fetch("bytes"),
    "sbom_sha256" => "sha256:#{observed.fetch('sbom.spdx.json').fetch('sha256')}",
    "provenance_sha256" => "sha256:#{observed.fetch('provenance.json').fetch('sha256')}",
    "image_manifest_sha256" => "sha256:#{IMAGE_DIGEST}",
    "created_at" => created.iso8601,
    "requested_expires_at" => (created + (7 * 24 * 60 * 60)).iso8601,
    "member_count" => MEMBERS.length,
    "verified" => true,
    "claims" => claims
  }
  receipt_path.binwrite(JSON.pretty_generate(receipt) + "\n")
end

puts "YARA-X retained candidate verified: archive=sha256:#{archive_digest} bytes=#{archive.size} executable=sha256:#{observed.fetch('yr').fetch('sha256')} members=#{MEMBERS.length} executed=false admitted=false production=false iar_2=false"
