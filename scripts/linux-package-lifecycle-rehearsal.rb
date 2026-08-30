#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "rbconfig"
require "rubygems/package"
require "tmpdir"
require "zlib"

BINARIES = %w[
  impresari-context
  impresari-context-mcp
  impresari-context-structural-worker
].freeze
PROFILES = %w[rootless_user_manager externally_managed].freeze
MAX_ARCHIVE_BYTES = 64 * 1024 * 1024
MAX_EXPANDED_BYTES = 128 * 1024 * 1024
MAX_ENTRIES = 128

class RehearsalError < StandardError; end

def sha256(path)
  Digest::SHA256.file(path).hexdigest
end

def verify_checksum!(archive)
  checksum = Pathname.new("#{archive}.sha256")
  raise RehearsalError, "missing checksum companion" unless checksum.file?
  expected = checksum.read.split.first
  raise RehearsalError, "malformed checksum companion" unless expected&.match?(/\A[0-9a-f]{64}\z/)
  raise RehearsalError, "archive checksum mismatch" unless expected == sha256(archive)
end

def safe_components!(name)
  raise RehearsalError, "archive path is empty" if name.empty?
  path = Pathname.new(name)
  components = path.each_filename.to_a
  if path.absolute? || components.any? { |part| part.empty? || part == "." || part == ".." }
    raise RehearsalError, "archive path escapes package root"
  end
  components
end

def extract_archive!(archive, destination)
  raise RehearsalError, "archive is unavailable" unless File.file?(archive)
  raise RehearsalError, "archive exceeds byte bound" if File.size(archive) > MAX_ARCHIVE_BYTES

  roots = []
  seen = {}
  entries = 0
  expanded = 0
  Zlib::GzipReader.open(archive) do |gzip|
    Gem::Package::TarReader.new(gzip) do |tar|
      tar.each do |entry|
        entries += 1
        raise RehearsalError, "archive exceeds entry bound" if entries > MAX_ENTRIES
        components = safe_components!(entry.full_name)
        raise RehearsalError, "archive contains a duplicate path" if seen[entry.full_name]
        seen[entry.full_name] = true
        roots << components.first
        target = File.join(destination, *components)
        if entry.directory?
          FileUtils.mkdir_p(target, mode: 0o755)
        elsif entry.file?
          expanded += entry.header.size
          raise RehearsalError, "archive exceeds expanded byte bound" if expanded > MAX_EXPANDED_BYTES
          FileUtils.mkdir_p(File.dirname(target), mode: 0o755)
          File.open(target, "wb", 0o600) { |file| IO.copy_stream(entry, file) }
          File.chmod(components[1] == "bin" ? 0o755 : 0o644, target)
        else
          raise RehearsalError, "archive contains unsupported entry type"
        end
      end
    end
  end
  root = roots.uniq
  raise RehearsalError, "archive must contain one top-level package" unless root.length == 1
  File.join(destination, root.fetch(0))
end

def load_package!(archive, destination, expected_source: nil)
  verify_checksum!(archive)
  root = extract_archive!(archive, destination)
  manifest_path = File.join(root, "MANIFEST.json")
  manifest = JSON.parse(File.binread(manifest_path))
  expected_keys = %w[files project_version rust_toolchain schema_name schema_version source_commit target]
  raise RehearsalError, "manifest shape is unsupported" unless manifest.keys.sort == expected_keys.sort
  raise RehearsalError, "manifest schema is unsupported" unless manifest["schema_name"] == "release-candidate-manifest" && manifest["schema_version"] == "1.0.0"
  raise RehearsalError, "manifest target is not Linux x86_64" unless manifest["target"] == "x86_64-unknown-linux-gnu"
  raise RehearsalError, "manifest project version is malformed" unless manifest["project_version"].is_a?(String) && manifest["project_version"].match?(/\A\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?\z/)
  raise RehearsalError, "manifest source identity is malformed" unless manifest["source_commit"].is_a?(String) && manifest["source_commit"].match?(/\A[0-9a-f]{40}\z/)
  expected_root = "impresari-context-#{manifest.fetch("project_version")}-x86_64-unknown-linux-gnu"
  raise RehearsalError, "package root identity mismatch" unless File.basename(root) == expected_root
  if expected_source && manifest["source_commit"] != expected_source
    raise RehearsalError, "candidate source identity mismatch"
  end

  listed = manifest.fetch("files")
  raise RehearsalError, "manifest file list is unsupported" unless listed.is_a?(Array) && listed.length.between?(3, 64)
  listed_paths = listed.map do |entry|
    raise RehearsalError, "manifest file entry shape is unsupported" unless entry.is_a?(Hash) && entry.keys.sort == %w[bytes path sha256]
    relative = entry.fetch("path")
    raise RehearsalError, "manifest file identity is malformed" unless relative.is_a?(String) && entry.fetch("bytes").is_a?(String) && entry.fetch("bytes").match?(/\A(?:0|[1-9][0-9]*)\z/) && entry.fetch("sha256").is_a?(String) && entry.fetch("sha256").match?(/\A[0-9a-f]{64}\z/)
    safe_components!(relative)
    file = File.join(root, relative)
    raise RehearsalError, "manifest file is missing" unless File.file?(file)
    raise RehearsalError, "manifest byte count mismatch" unless File.size(file).to_s == entry.fetch("bytes")
    raise RehearsalError, "manifest file digest mismatch" unless sha256(file) == entry.fetch("sha256")
    relative
  end
  raise RehearsalError, "manifest contains duplicate paths" unless listed_paths.uniq.length == listed_paths.length
  actual = Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH).select { |path| File.file?(path) }
    .map { |path| path.delete_prefix("#{root}/") }.reject { |path| path == "MANIFEST.json" }.sort
  raise RehearsalError, "archive contains an unmanifested file" unless actual == listed_paths.sort
  binary_entries = listed.select { |entry| entry.fetch("path").start_with?("bin/") }
  expected_binary_paths = BINARIES.map { |name| "bin/#{name}" }.sort
  raise RehearsalError, "package binary scope drifted" unless binary_entries.map { |entry| entry.fetch("path") }.sort == expected_binary_paths

  {
    "archive_sha256" => sha256(archive),
    "manifest_sha256" => sha256(manifest_path),
    "project_version" => manifest.fetch("project_version"),
    "source_commit" => manifest.fetch("source_commit"),
    "root" => root,
    "binary_digests" => binary_entries.to_h { |entry| [File.basename(entry.fetch("path")), entry.fetch("sha256")] },
  }
rescue JSON::ParserError, KeyError => error
  raise RehearsalError, "invalid package manifest: #{error.message}"
end

def replace_install!(package, install_dir)
  FileUtils.mkdir_p(install_dir, mode: 0o755)
  BINARIES.each do |name|
    source = File.join(package.fetch("root"), "bin", name)
    temporary = File.join(install_dir, ".#{name}.new.#{Process.pid}")
    FileUtils.cp(source, temporary)
    File.chmod(0o755, temporary)
    File.rename(temporary, File.join(install_dir, name))
  ensure
    FileUtils.rm_f(temporary) if temporary
  end
end

def verify_install!(package, install_dir)
  actual = Dir.children(install_dir).sort
  raise RehearsalError, "installed binary scope drifted" unless actual == BINARIES.sort
  BINARIES.each do |name|
    path = File.join(install_dir, name)
    raise RehearsalError, "installed binary is unavailable" unless File.file?(path) && File.executable?(path)
    raise RehearsalError, "installed binary identity mismatch" unless sha256(path) == package.fetch("binary_digests").fetch(name)
  end
end

def verify_cli_relaunch!(install_dir, home)
  output, status = Open3.capture2e({ "HOME" => home }, File.join(install_dir, "impresari-context"))
  raise RehearsalError, "operator relaunch failed" unless status.success?
  line = output.lines.find { |candidate| candidate.lstrip.start_with?("{") }
  JSON.parse(line || raise(RehearsalError, "operator relaunch emitted no machine JSON"))
rescue JSON::ParserError
  raise RehearsalError, "operator relaunch emitted invalid machine JSON"
end

def phase(name, outcome)
  { "phase" => name, "outcome" => outcome }
end

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby scripts/linux-package-lifecycle-rehearsal.rb --profile PROFILE --baseline ARCHIVE --candidate ARCHIVE --candidate-source-sha SHA"
  parser.on("--profile PROFILE") { |value| options[:profile] = value }
  parser.on("--baseline ARCHIVE") { |value| options[:baseline] = File.expand_path(value) }
  parser.on("--candidate ARCHIVE") { |value| options[:candidate] = File.expand_path(value) }
  parser.on("--candidate-source-sha SHA") { |value| options[:candidate_source] = value }
end.parse!

begin
  required = %i[profile baseline candidate candidate_source]
  missing = required.reject { |key| options[key] && !options[key].empty? }
  raise RehearsalError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  raise RehearsalError, "unsupported profile" unless PROFILES.include?(options.fetch(:profile))
  raise RehearsalError, "candidate source SHA is malformed" unless options.fetch(:candidate_source).match?(/\A[0-9a-f]{40}\z/)
  raise RehearsalError, "live package rehearsal requires Linux" unless RUBY_PLATFORM.include?("linux")
  environment = ENV.fetch("RUNNER_ENVIRONMENT", "local")
  raise RehearsalError, "unsupported rehearsal environment" unless %w[github-hosted local].include?(environment)

  receipt = nil
  Dir.mktmpdir("impresari-linux-package-lifecycle-") do |directory|
    baseline = load_package!(options.fetch(:baseline), File.join(directory, "baseline"))
    candidate = load_package!(options.fetch(:candidate), File.join(directory, "candidate"), expected_source: options.fetch(:candidate_source))
    raise RehearsalError, "baseline and candidate archive identities must differ" if baseline.fetch("archive_sha256") == candidate.fetch("archive_sha256")
    install_dir = File.join(directory, "install", "bin")
    home = File.join(directory, "home")
    FileUtils.mkdir_p(home, mode: 0o700)

    replace_install!(baseline, install_dir)
    verify_install!(baseline, install_dir)
    phases = [phase("clean_install", "passed")]

    replace_install!(candidate, install_dir)
    verify_install!(candidate, install_dir)
    phases << phase("upgrade", "passed")

    replace_install!(baseline, install_dir)
    verify_install!(baseline, install_dir)
    phases << phase("rollback", "passed")

    if options.fetch(:profile) == "externally_managed"
      verify_cli_relaunch!(install_dir, home)
      phases << phase("operator_relaunch", "passed")
      reentry = "operator_relaunch_verified"
    else
      phases << phase("logout_login", "not_observed")
      reentry = "real_login_session_required"
    end

    BINARIES.each { |name| FileUtils.rm_f(File.join(install_dir, name)) }
    raise RehearsalError, "uninstall left package files" unless Dir.children(install_dir).empty?
    phases << phase("uninstall", "passed")

    complete = options.fetch(:profile) == "externally_managed"
    receipt = {
      "schema_name" => "linux-isolation-package-lifecycle-rehearsal",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-lifecycle-v1",
      "profile" => options.fetch(:profile),
      "host" => { "operating_system" => "linux", "architecture" => RbConfig::CONFIG.fetch("host_cpu"), "environment" => environment },
      "baseline" => baseline.reject { |key, _| %w[root binary_digests].include?(key) },
      "candidate" => candidate.reject { |key, _| %w[root binary_digests].include?(key) },
      "package_scope" => BINARIES,
      "phases" => phases,
      "reentry_evidence" => reentry,
      "excluded_lifecycle_evidence" => %w[cancellation crash_recovery health_withdrawal topology_revalidation],
      "clean_state" => {
        "installed_service_unit_absent" => true,
        "authorization_policy_absent" => true,
        "unexpected_package_files_absent" => true,
        "staged_source_absent" => true,
      },
      "status" => complete ? "package_lifecycle_candidate" : "package_lifecycle_partial",
      "safe_next_step" => complete ?
        "Compose this exact package evidence with fresh external topology, cancellation, crash, and withdrawal evidence; production remains closed." :
        "Obtain a genuine fresh login-session reentry plus topology, cancellation, crash, and withdrawal evidence; do not substitute a process restart.",
      "claims" => {
        "full_lifecycle_admitted" => false,
        "production_admitted" => false,
        "real_analyzer_authorized" => false,
        "privileged_installation_authorized" => false,
        "persistent_service_authorized" => false,
      },
    }
  end
  puts JSON.pretty_generate(receipt)
rescue RehearsalError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
