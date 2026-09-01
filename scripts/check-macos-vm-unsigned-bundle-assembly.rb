#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "find"
require "json"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-unsigned-bundle-assembly-v1.json"
PROFILE_DIGEST = "0d661ae58e579d899325b130a0de597e5e82075331bef17ee4430f280a3db3eb"
SPEC_RELATIVE = "platform/macos-vm-feasibility/unsigned-synthetic-bundle-assembly-v1.json"
SPEC_DIGEST = "36978dfd1f475d219ed7168d7f00c17fca1dcd5951e771e6dd81a5cfff7058d9"
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/cask-package-contract-v1.json"
CONTRACT_DIGEST = "4f249a15c1cd0b5283c937d49cc1888c3ab56b2a9a22847b8913901c72d5f676"
SEAL_RELATIVE = "platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
TREE_DIGEST = "ace9ff8230be69e0df6a8e7977fde6cf82a8ecb9221be841f49718a4c6f79813"

FIXTURE_DIGESTS = {
  "valid/macos-local-vm-unsigned-bundle-assembly.json" => SPEC_DIGEST,
  "valid/iar-macos-local-vm-unsigned-bundle-assembly-profile.json" => PROFILE_DIGEST,
  "valid/macos-local-vm-unsigned-bundle-assembly-receipt.json" => "8a34e524214e2f63da41e89f7841b872073844a0ea6c3eb1b6c0c42b4248cf25",
  "invalid/macos-local-vm-unsigned-bundle-assembly-overclaim.json" => "37ebd988d551ddfeba942ea4a324ec279d51b8de4b54348b1f4d0ed0c1ec8dc4"
}.freeze

MARKER_ROLES = {
  "Contents/Helpers/impresari-context-mcp" => "local-stdio-mcp-server",
  "Contents/Helpers/impresari-context-structural-worker" => "isolated-structural-worker",
  "Contents/Helpers/impresari-context-vm-controller" => "local-vm-controller",
  "Contents/MacOS/impresari-context" => "cli-supervisor-entrypoint",
  "Contents/Resources/macos-vm/guest/SYNTHETIC-ONLY.txt" => "closed-guest-payload-placeholder"
}.freeze

LATER_FALSE_CLAIMS = %w[
  network_access credential_access repository_source_access process_launch
  archive_created bundle_installed cask_created release_identity_bound
  github_publication_attestation_verified developer_id_signature_verified
  apple_notarization_verified cask_lifecycle_verified sealed_distribution
  production_admitted macos_iar_1b_admitted vm_launch analyzer_execution
  authority_added
].freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid JSON: #{path}: #{e.message}"
end

def exact(path, digest, label)
  abort "missing #{label}: #{path}" unless path.file?
  abort "refusing symlinked #{label}: #{path}" if path.symlink?
  abort "#{label} digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
  path
end

def clean_relative(path)
  value = Pathname.new(path)
  !value.absolute? && value.cleanpath.to_s == path && !value.each_filename.include?("..") && path.start_with?("Contents")
end

def info_plist
  <<~PLIST
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
      <key>CFBundleDisplayName</key>
      <string>Impresari Context (Synthetic)</string>
      <key>CFBundleExecutable</key>
      <string>impresari-context</string>
      <key>CFBundleIdentifier</key>
      <string>dev.impresari.context.synthetic-assembly</string>
      <key>CFBundlePackageType</key>
      <string>APPL</string>
      <key>CFBundleShortVersionString</key>
      <string>0.0.0</string>
      <key>CFBundleVersion</key>
      <string>0</string>
    </dict>
    </plist>
  PLIST
end

def marker(path)
  role = MARKER_ROLES.fetch(path)
  "IMPRESARI_CONTEXT_SYNTHETIC_NONEXECUTABLE_PLACEHOLDER_V1\nrole=#{role}\npath=#{path}\n"
end

def source_bytes(entry, seal_bytes)
  case entry.fetch("source")
  when "generated-info-plist-v1"
    info_plist
  when "generated-nonexecutable-marker-v1"
    marker(entry.fetch("path"))
  when "copy-exact-project-metadata"
    seal_bytes
  else
    abort "unsupported assembly source: #{entry.fetch('source')}"
  end
end

def filesystem_records(app)
  records = []
  Find.find(app.to_s) do |raw|
    next if raw == app.to_s
    path = Pathname.new(raw)
    relative = path.relative_path_from(app).to_s
    stat = path.lstat
    abort "assembled tree contains a symlink: #{relative}" if stat.symlink?
    kind = stat.directory? ? "directory" : (stat.file? ? "file" : nil)
    abort "assembled tree contains a special file: #{relative}" unless kind
    records << {
      "path" => relative,
      "kind" => kind,
      "mode" => format("%04o", stat.mode & 0o7777),
      "bytes" => kind == "file" ? stat.size.to_s : "0",
      "sha256" => kind == "file" ? "sha256:#{Digest::SHA256.file(path).hexdigest}" : "none"
    }
  end
  records.sort_by { |entry| entry.fetch("path") }
end

def tree_digest(records)
  bytes = records.map do |entry|
    %w[path kind mode bytes sha256].map { |key| entry.fetch(key) }.join("\t") + "\n"
  end.join
  Digest::SHA256.hexdigest(bytes)
end

def assemble_once(spec, seal_bytes)
  temporary_root = nil
  digest = nil
  Dir.mktmpdir("impresari-macos-unsigned-bundle-") do |raw_root|
    temporary_root = Pathname.new(raw_root)
    abort "temporary assembly root is not a fresh directory" unless temporary_root.directory? && !temporary_root.symlink?
    # Windows Ruby mode bits are not ACL evidence. Keep Windows CI useful for
    # structural, determinism, and cleanup checks without presenting it as
    # proof of the target macOS/POSIX private-directory policy.
    unless Gem.win_platform?
      File.chmod(0o700, temporary_root)
      abort "temporary assembly root is not private" unless (temporary_root.stat.mode & 0o7777) == 0o700
    end

    app = temporary_root.join(spec.fetch("app_bundle"))
    Dir.mkdir(app, 0o755)
    entries = spec.fetch("entries")
    entries.select { |entry| entry.fetch("kind") == "directory" }.sort_by { |entry| entry.fetch("path").count("/") }.each do |entry|
      relative = entry.fetch("path")
      abort "unsafe assembly path: #{relative}" unless clean_relative(relative)
      target = app.join(relative)
      Dir.mkdir(target, entry.fetch("mode").to_i(8))
      File.chmod(entry.fetch("mode").to_i(8), target)
    end

    entries.select { |entry| entry.fetch("kind") == "file" }.each do |entry|
      relative = entry.fetch("path")
      abort "unsafe assembly path: #{relative}" unless clean_relative(relative)
      target = app.join(relative)
      bytes = source_bytes(entry, seal_bytes)
      File.open(target, File::WRONLY | File::CREAT | File::EXCL, 0o600) { |file| file.write(bytes) }
      File.chmod(entry.fetch("mode").to_i(8), target)
    end

    actual = filesystem_records(app)
    expected = entries.map { |entry| entry.slice("path", "kind", "mode", "bytes", "sha256") }.sort_by { |entry| entry.fetch("path") }
    if Gem.win_platform?
      # Windows CI proves the portable tree only. Its Ruby mode bits are not
      # evidence for the target macOS bundle modes or executable-bit policy.
      structural_keys = %w[path kind bytes sha256]
      actual_structure = actual.map { |entry| entry.slice(*structural_keys) }
      expected_structure = expected.map { |entry| entry.slice(*structural_keys) }
      abort "assembled bundle structure does not match the closed specification" unless actual_structure == expected_structure
      digest = tree_digest(expected)
    else
      abort "assembled bundle tree does not match the closed specification" unless actual == expected
      digest = tree_digest(actual)
      abort "synthetic entrypoint became executable" unless (app.join("Contents/MacOS/impresari-context").stat.mode & 0o111).zero?
    end
    abort "assembled bundle tree digest changed" unless digest == TREE_DIGEST
    abort "synthetic Info.plist changed" unless app.join("Contents/Info.plist").binread == info_plist
    abort "metadata seal copy changed" unless Digest::SHA256.file(app.join("Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json")).hexdigest == SEAL_DIGEST
  end
  abort "temporary assembly root was not cleaned up" if temporary_root&.exist?
  digest
end

profile_path = exact(ROOT.join(PROFILE_RELATIVE), PROFILE_DIGEST, "unsigned bundle assembly profile")
sidecar = ROOT.join("profiles/v1/iar-macos-local-vm-unsigned-bundle-assembly-v1.sha256").read.strip
abort "unsigned bundle profile checksum record mismatch" unless sidecar == "#{PROFILE_DIGEST}  iar-macos-local-vm-unsigned-bundle-assembly-v1.json"
profile = json(profile_path)
abort "unsigned bundle profile fixture drifted" unless profile_path.binread == FIXTURE_ROOT.join("valid/iar-macos-local-vm-unsigned-bundle-assembly-profile.json").binread

spec_path = exact(ROOT.join(SPEC_RELATIVE), SPEC_DIGEST, "unsigned bundle assembly specification")
spec = json(spec_path)
abort "unsigned bundle specification fixture drifted" unless spec_path.binread == FIXTURE_ROOT.join("valid/macos-local-vm-unsigned-bundle-assembly.json").binread
abort "profile does not bind the exact assembly inputs" unless
  profile.fetch("assembly_spec_path") == SPEC_RELATIVE &&
    profile.fetch("assembly_spec_digest") == "sha256:#{SPEC_DIGEST}" &&
    profile.fetch("package_contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("package_contract_digest") == "sha256:#{CONTRACT_DIGEST}" &&
    profile.fetch("metadata_seal_path") == SEAL_RELATIVE &&
    profile.fetch("metadata_seal_digest") == "sha256:#{SEAL_DIGEST}" &&
    profile.fetch("expected_tree_digest") == "sha256:#{TREE_DIGEST}"

contract = json(exact(ROOT.join(CONTRACT_RELATIVE), CONTRACT_DIGEST, "cask package contract"))
seal_path = exact(ROOT.join(SEAL_RELATIVE), SEAL_DIGEST, "release metadata seal")
seal_bytes = seal_path.binread
abort "assembly contract identity changed" unless spec.fetch("contract_id") == contract.fetch("contract_id")
abort "assembly metadata binding changed" unless
  spec.fetch("guest_release_id") == contract.fetch("release_bindings").fetch("guest_release_id") &&
    spec.fetch("metadata_set_digest") == contract.fetch("release_bindings").fetch("metadata_set_digest")

entries = spec.fetch("entries")
paths = entries.map { |entry| entry.fetch("path") }
abort "assembly paths are not closed and sorted" unless paths == paths.sort && paths.uniq == paths
abort "assembly entry ceiling changed" unless entries.length == profile.fetch("maximum_entries").to_i
abort "assembly contains an unsafe path" unless paths.all? { |path| clean_relative(path) }
abort "assembly contains an executable payload" unless entries.select { |entry| entry.fetch("kind") == "file" }.all? { |entry| (entry.fetch("mode").to_i(8) & 0o111).zero? }
abort "assembly file ceiling exceeded" unless entries.select { |entry| entry.fetch("kind") == "file" }.all? { |entry| entry.fetch("bytes").to_i <= profile.fetch("maximum_file_bytes").to_i }

contract_paths = contract.fetch("bundle_layout").map { |entry| entry.fetch("path") }
abort "assembly omitted a contract role" unless (contract_paths - paths).empty?
abort "assembly did not add exact synthetic metadata and guest marker only" unless
  (paths - contract_paths - ["Contents", "Contents/Helpers", "Contents/Info.plist", "Contents/MacOS", "Contents/Resources", "Contents/Resources/macos-vm", "Contents/Resources/macos-vm/guest/SYNTHETIC-ONLY.txt"]).empty?

controls = spec.fetch("controls")
abort "synthetic assembly controls changed" unless
  controls.fetch("offline_only") && controls.fetch("private_temporary_root") &&
    controls.fetch("two_run_determinism_required") && controls.fetch("cleanup_required") &&
    controls.fetch("synthetic_payloads_only") && !controls.fetch("executable_payloads_present") &&
    !controls.fetch("symlinks_allowed") && !controls.fetch("special_files_allowed") &&
    !controls.fetch("signing_allowed") && !controls.fetch("notarization_allowed") &&
    LATER_FALSE_CLAIMS.all? { |key| !controls.fetch(key) }

runs = Array.new(profile.fetch("assembly_runs").to_i) { assemble_once(spec, seal_bytes) }
abort "two assembly runs were not identical" unless runs.uniq == [TREE_DIGEST]

receipt = {
  "schema_name" => "macos-local-vm-unsigned-bundle-assembly-receipt",
  "schema_version" => "1.0.0",
  "profile_id" => profile.fetch("profile_id"),
  "profile_digest" => "sha256:#{PROFILE_DIGEST}",
  "assembly_id" => spec.fetch("assembly_id"),
  "assembly_spec_digest" => "sha256:#{SPEC_DIGEST}",
  "contract_id" => spec.fetch("contract_id"),
  "package_contract_digest" => "sha256:#{CONTRACT_DIGEST}",
  "guest_release_id" => spec.fetch("guest_release_id"),
  "metadata_set_digest" => spec.fetch("metadata_set_digest"),
  "tree_digest" => "sha256:#{TREE_DIGEST}",
  "status" => "unsigned_synthetic_bundle_assembled",
  "assembly_runs" => profile.fetch("assembly_runs"),
  "entry_count" => entries.length.to_s,
  "bundles_identical" => true,
  "bundle_layout_exact" => true,
  "info_plist_exact" => true,
  "metadata_seal_copy_exact" => true,
  "synthetic_payloads_only" => true,
  "executable_payloads_present" => false,
  "symlinks_absent" => true,
  "special_files_absent" => true,
  "private_temp_root_verified" => true,
  "cleanup_verified" => true,
  "synthetic_app_bundle_assembled" => true,
  "release_app_bundle_assembled" => false,
  "network_access" => false,
  "credential_access" => false,
  "repository_source_access" => false,
  "process_launch" => false,
  "archive_created" => false,
  "bundle_installed" => false,
  "cask_created" => false,
  "release_identity_bound" => false,
  "github_publication_attestation_verified" => false,
  "developer_id_signature_verified" => false,
  "apple_notarization_verified" => false,
  "cask_lifecycle_verified" => false,
  "sealed_distribution" => false,
  "production_admitted" => false,
  "macos_iar_1b_admitted" => false,
  "vm_launch" => false,
  "analyzer_execution" => false,
  "authority_added" => false
}

fixture_receipt = json(FIXTURE_ROOT.join("valid/macos-local-vm-unsigned-bundle-assembly-receipt.json"))
abort "unsigned bundle assembly receipt fixture drifted" unless receipt == fixture_receipt

provenance = json(FIXTURE_ROOT.join("macos-local-vm-unsigned-bundle-assembly-fixture-provenance.json"))
abort "unsigned bundle fixture provenance boundary changed" unless
  provenance.fetch("review_status") == "approved_original_synthetic_and_project_metadata_only" &&
    %w[contains_executable_artifacts contains_malware_or_live_signatures contains_third_party_source contains_private_or_customer_source network_or_provider_data_used].none? { |key| provenance.fetch(key) }
recorded = provenance.fetch("cases").to_h { |entry| [entry.fetch("path"), entry.fetch("sha256")] }
abort "unsigned bundle fixture provenance inventory changed" unless recorded == FIXTURE_DIGESTS
FIXTURE_DIGESTS.each { |relative, digest| exact(FIXTURE_ROOT.join(relative), digest, "unsigned bundle fixture") }

puts "macOS local-VM unsigned synthetic bundle assembled twice: tree=sha256:#{TREE_DIGEST} runnable=false installed=false signed=false"
