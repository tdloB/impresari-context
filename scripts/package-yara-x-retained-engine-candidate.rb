#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "fileutils"
require "json"
require "open3"
require "pathname"
require "zlib"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da"
POLICY_DIGEST = "fbae2b383e843d07dd5e30ad3d33a580e9094878e49c21fec21c8e977ce8891c"
REFERENCE_EXECUTABLE_DIGEST = "a35ad2ec1354a67cb2465a07fe1576e60bcfdbc18ec0b80546fca2a7faeff09d"
MEMBERS = %w[
  DEPENDENCY-NOTICES.txt LICENSE MANIFEST.json SHA256SUMS
  dependency-closure.json engine-candidate.json license-disposition.json
  provenance.json reproducibility-disposition.json sbom.spdx.json
  vulnerability-disposition.json yr
].freeze
PAYLOAD_MEMBERS = (MEMBERS - %w[MANIFEST.json SHA256SUMS]).freeze
FALSE_CLAIMS = {
  "signed" => false,
  "attested" => false,
  "published" => false,
  "executable_admitted" => false,
  "ruleset_present" => false,
  "rules_compiled" => false,
  "analyzer_executed" => false,
  "repository_scanned" => false,
  "credentials_accessed" => false,
  "production_admitted" => false,
  "iar_2_admitted" => false,
  "detection_quality_claimed" => false,
  "safety_claimed" => false,
  "malware_free_claimed" => false,
  "authority_added" => false
}.freeze

def regular!(path, label)
  stat = path.lstat
  abort "#{label} must be a regular non-symlink file" unless stat.file? && !stat.symlink?
rescue Errno::ENOENT => e
  abort "missing #{label}: #{e.message}"
end

def json(path, label)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid #{label}: #{e.message}"
end

def write_json(path, object)
  path.binwrite(JSON.pretty_generate(object) + "\n")
end

def mapped_manifest_dir(manifest_path, source_root, cargo_home)
  raw = manifest_path.delete_suffix("/Cargo.toml")
  if raw == "/usr/src/yara-x"
    source_root
  elsif raw.start_with?("/usr/src/yara-x/")
    source_root.join(raw.delete_prefix("/usr/src/yara-x/")).cleanpath
  elsif raw == "/cargo"
    cargo_home
  elsif raw.start_with?("/cargo/")
    cargo_home.join(raw.delete_prefix("/cargo/")).cleanpath
  else
    abort "dependency metadata leaked an unexpected source root: #{raw}"
  end
end

source_root = Pathname.new(ARGV.fetch(0) { abort "usage: package-yara-x-retained-engine-candidate.rb SOURCE CARGO_HOME METADATA TREE OUTPUT" }).expand_path
cargo_home = Pathname.new(ARGV.fetch(1)).expand_path
metadata_path = Pathname.new(ARGV.fetch(2)).expand_path
tree_path = Pathname.new(ARGV.fetch(3)).expand_path
output_path = Pathname.new(ARGV.fetch(4)).expand_path
profile_path = ROOT.join("profiles/v1/yara-x-retained-engine-candidate-v1.json")
yr = source_root.join("target/x86_64-unknown-linux-gnu/release-lto/yr")

[metadata_path, tree_path, profile_path, yr, source_root.join("LICENSE")].each do |path|
  regular!(path, path.basename.to_s)
end
abort "retention profile digest changed" unless Digest::SHA256.file(profile_path).hexdigest == PROFILE_DIGEST
abort "YARA-X patched lockfile changed" unless
  Digest::SHA256.file(source_root.join("Cargo.lock")).hexdigest == "e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e"

metadata = json(metadata_path, "Cargo metadata")
selected = tree_path.readlines(chomp: true).filter_map do |line|
  normalized = line.sub(/ \(\*\)\z/, "").strip
  match = normalized.match(/\A([A-Za-z0-9_.-]+) v([^\s]+)(?: \(.+\))?\z/)
  [match[1], match[2]] if match
end.uniq
abort "selected dependency closure is empty" if selected.empty?

packages_by_identity = metadata.fetch("packages").group_by { |package| [package.fetch("name"), package.fetch("version")] }
packages = selected.map do |identity|
  matches = packages_by_identity.fetch(identity) { abort "selected dependency absent from metadata: #{identity.join(' ')}" }
  abort "ambiguous dependency identity: #{identity.join(' ')}" unless matches.length == 1
  matches.first
end.sort_by { |package| [package.fetch("name"), package.fetch("version"), package["source"].to_s] }

closure = packages.map do |package|
  license = package["license"]
  license_file = package["license_file"]
  abort "dependency has no declared license identity: #{package.fetch('name')} #{package.fetch('version')}" if
    license.to_s.empty? && license_file.to_s.empty?
  {
    "name" => package.fetch("name"),
    "version" => package.fetch("version"),
    "source" => package["source"] || "workspace:yara-x-v1.20.0",
    "checksum" => package["checksum"],
    "license" => license,
    "license_file" => license_file && File.basename(license_file)
  }
end

notices = String.new("YARA-X v1.20.0 dependency notices\nGenerated from the exact locked selected-feature dependency closure.\n")
packages.each do |package|
  directory = mapped_manifest_dir(package.fetch("manifest_path"), source_root, cargo_home)
  regular!(directory.join("Cargo.toml"), "dependency manifest")
  notice_paths = directory.children.select do |path|
    path.basename.to_s.match?(/\A(?:LICENSE|COPYING|NOTICE)(?:[._-].*)?\z/i) && path.file? && !path.symlink?
  end.sort_by { |path| path.basename.to_s }
  notices << "\n===== #{package.fetch('name')} #{package.fetch('version')} =====\n"
  notices << "Declared license: #{package['license'] || 'license-file'}\n"
  notice_paths.each do |path|
    abort "dependency notice is unexpectedly large: #{path.basename}" if path.size > 1_048_576
    notices << "\n--- #{path.basename} ---\n"
    notices << path.binread.force_encoding(Encoding::UTF_8).scrub
    notices << "\n" unless notices.end_with?("\n")
  end
end
abort "dependency notices exceed 32 MiB" if notices.bytesize > 33_554_432

stage = output_path.dirname.join("stage")
FileUtils.rm_rf(stage)
FileUtils.mkdir_p(stage)
FileUtils.cp(yr, stage.join("yr"))
FileUtils.cp(source_root.join("LICENSE"), stage.join("LICENSE"))
stage.join("DEPENDENCY-NOTICES.txt").binwrite(notices)
write_json(stage.join("dependency-closure.json"), {
  "schema_name" => "yara-x-selected-dependency-closure",
  "schema_version" => "1.0.0",
  "target" => "x86_64-unknown-linux-gnu",
  "features" => ["pulley"],
  "packages" => closure
})

spdx_packages = closure.each_with_index.map do |package, index|
  item = {
    "SPDXID" => "SPDXRef-Package-#{index + 1}",
    "name" => package.fetch("name"),
    "versionInfo" => package.fetch("version"),
    "downloadLocation" => package.fetch("source").start_with?("registry+") ? "https://crates.io/crates/#{package.fetch('name')}/#{package.fetch('version')}" : "NOASSERTION",
    "filesAnalyzed" => false,
    "licenseConcluded" => "NOASSERTION",
    "licenseDeclared" => package["license"] || "NOASSERTION",
    "copyrightText" => "NOASSERTION",
    "externalRefs" => [{
      "referenceCategory" => "PACKAGE-MANAGER",
      "referenceType" => "purl",
      "referenceLocator" => "pkg:cargo/#{package.fetch('name')}@#{package.fetch('version')}"
    }]
  }
  item["checksums"] = [{"algorithm" => "SHA256", "checksumValue" => package.fetch("checksum")}] if package["checksum"]
  item
end
write_json(stage.join("sbom.spdx.json"), {
  "spdxVersion" => "SPDX-2.3",
  "dataLicense" => "CC0-1.0",
  "SPDXID" => "SPDXRef-DOCUMENT",
  "name" => "impresari-yara-x-v1.20.0-linux-x86-64-candidate",
  "documentNamespace" => "https://impresari-context.invalid/yara-x/candidate/#{Digest::SHA256.file(source_root.join('Cargo.lock')).hexdigest}",
  "creationInfo" => {
    "created" => "2026-08-24T09:50:21Z",
    "creators" => ["Tool: impresari-context/package-yara-x-retained-engine-candidate.rb-1.0.0"]
  },
  "documentDescribes" => spdx_packages.map { |package| package.fetch("SPDXID") },
  "packages" => spdx_packages
})

yr_digest = Digest::SHA256.file(yr).hexdigest
write_json(stage.join("provenance.json"), {
  "schema_name" => "yara-x-engine-candidate-build-provenance",
  "schema_version" => "1.0.0",
  "impresari_source_commit" => ENV.fetch("GITHUB_SHA"),
  "upstream_commit" => "60ad06971467029e77967e59d580cbbe85a1474d",
  "upstream_archive_sha256" => "sha256:8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee",
  "patch_sha256" => "sha256:b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd",
  "lockfile_sha256" => "sha256:e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e",
  "profile_sha256" => "sha256:#{PROFILE_DIGEST}",
  "toolchain" => "rustc 1.93.0",
  "target" => "x86_64-unknown-linux-gnu",
  "build_image" => "docker.io/library/rust@sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28",
  "build_image_index_sha256" => "sha256:d0a4aa3ca2e1088ac0c81690914a0d810f2eee188197034edf366ed010a2b382",
  "runner_image" => ENV.fetch("ImageOS"),
  "runner_image_version" => ENV.fetch("ImageVersion"),
  "runner_kernel" => ENV.fetch("IMPRESARI_RUNNER_KERNEL"),
  "runner_architecture" => ENV.fetch("IMPRESARI_RUNNER_ARCH"),
  "advisory_database_commit" => ENV.fetch("IMPRESARI_ADVISORY_DB_COMMIT"),
  "network_disabled_for_build" => true,
  "command" => "cargo build --offline --frozen --locked --profile release-lto --package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu"
})
write_json(stage.join("vulnerability-disposition.json"), {
  "schema_name" => "yara-x-engine-candidate-vulnerability-disposition",
  "schema_version" => "1.0.0",
  "cargo_audit_passed" => true,
  "advisory_database_commit" => ENV.fetch("IMPRESARI_ADVISORY_DB_COMMIT"),
  "explicitly_dispositioned" => ["RUSTSEC-2023-0071", "RUSTSEC-2026-0222", "RUSTSEC-2026-0269"],
  "new_advisories_allowed" => false,
  "human_reviewed" => false,
  "production_approved" => false
})
write_json(stage.join("license-disposition.json"), {
  "schema_name" => "yara-x-engine-candidate-license-disposition",
  "schema_version" => "1.0.0",
  "upstream_license" => "BSD-3-Clause",
  "dependency_count" => closure.length,
  "metadata_complete" => true,
  "notices_included" => true,
  "human_reviewed" => false,
  "production_approved" => false
})
write_json(stage.join("reproducibility-disposition.json"), {
  "schema_name" => "yara-x-engine-candidate-reproducibility-disposition",
  "schema_version" => "1.0.0",
  "same_job_reference_sha256" => "sha256:#{REFERENCE_EXECUTABLE_DIGEST}",
  "candidate_sha256" => "sha256:#{yr_digest}",
  "same_job_reference_matched" => yr_digest == REFERENCE_EXECUTABLE_DIGEST,
  "digest_pinned_image_used" => true,
  "cross_run_reproducibility_established" => false,
  "cross_host_reproducibility_established" => false,
  "production_approved" => false
})
write_json(stage.join("engine-candidate.json"), {
  "schema_name" => "yara-x-engine-bundle-candidate",
  "schema_version" => "1.0.0",
  "candidate_id" => "yara_x_engine_v1_20_0_linux_x86_64",
  "policy_id" => "yara-x-production-admission-v1",
  "policy_digest" => "sha256:#{POLICY_DIGEST}",
  "target" => "x86_64-unknown-linux-gnu",
  "state" => "missing_evidence",
  "artifact" => {"present" => true, "sha256" => "sha256:#{yr_digest}", "bytes" => yr.size, "retained" => true},
  "evidence" => {
    "locked_build" => true,
    "dependency_closure" => true,
    "sbom" => true,
    "build_provenance" => true,
    "vulnerability_disposition" => true,
    "license_disposition" => true,
    "reproducibility_disposition" => true,
    "signature" => false,
    "expiry" => true,
    "revocation_identity" => false
  },
  "admitted" => false,
  "claims" => {
    "production_admitted" => false,
    "iar_2_admitted" => false,
    "repository_scan_authorized" => false,
    "credential_access_authorized" => false,
    "artifact_upload_authorized" => false,
    "detection_quality_claimed" => false,
    "safety_claimed" => false,
    "malware_free_claimed" => false,
    "authority_added" => false
  }
})

manifest_files = PAYLOAD_MEMBERS.sort.map do |name|
  path = stage.join(name)
  regular!(path, name)
  {"path" => name, "bytes" => path.size, "sha256" => "sha256:#{Digest::SHA256.file(path).hexdigest}"}
end
write_json(stage.join("MANIFEST.json"), {
  "schema_name" => "yara-x-engine-candidate-file-manifest",
  "schema_version" => "1.0.0",
  "target" => "x86_64-unknown-linux-gnu",
  "files" => manifest_files
})
stage.join("SHA256SUMS").binwrite((MEMBERS - ["SHA256SUMS"]).sort.map do |name|
  "#{Digest::SHA256.file(stage.join(name)).hexdigest}  #{name}\n"
end.join)

MEMBERS.each do |name|
  regular!(stage.join(name), name)
  stage.join(name).chmod(name == "yr" ? 0o555 : 0o444)
end
abort "candidate stage gained unexpected members" unless stage.children.map { |path| path.basename.to_s }.sort == MEMBERS.sort

FileUtils.mkdir_p(output_path.dirname)
raw_tar = output_path.sub_ext("").sub_ext(".tar")
FileUtils.rm_f([raw_tar, output_path])
command = ["tar", "--sort=name", "--format=ustar", "--mtime=@1787565021", "--owner=0", "--group=0", "--numeric-owner", "-cf", raw_tar.to_s, "-C", stage.to_s, *MEMBERS.sort]
stdout, stderr, status = Open3.capture3(*command)
abort "deterministic tar failed: #{stdout}#{stderr}" unless status.success?
Zlib::GzipWriter.open(output_path.to_s, Zlib::BEST_COMPRESSION) do |gzip|
  gzip.mtime = 0
  gzip.orig_name = ""
  File.open(raw_tar, "rb") { |input| IO.copy_stream(input, gzip) }
end
FileUtils.rm_f(raw_tar)
abort "candidate archive exceeds 256 MiB" if output_path.size > 268_435_456

puts JSON.generate({
  "archive" => output_path.to_s,
  "archive_sha256" => Digest::SHA256.file(output_path).hexdigest,
  "archive_bytes" => output_path.size,
  "executable_sha256" => yr_digest,
  "executable_bytes" => yr.size,
  "sbom_sha256" => Digest::SHA256.file(stage.join("sbom.spdx.json")).hexdigest,
  "provenance_sha256" => Digest::SHA256.file(stage.join("provenance.json")).hexdigest,
  "member_count" => MEMBERS.length,
  "claims" => FALSE_CLAIMS
})
