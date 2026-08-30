#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "pathname"
require "rubygems/package"
require "tmpdir"
require "zlib"

ROOT = Pathname.new(__dir__).join("..").expand_path
REHEARSAL = ROOT.join("scripts/linux-package-lifecycle-rehearsal.rb")
FIXTURES = {
  "rootless_user_manager" => ROOT.join("tests/conformance/v1/valid/linux-isolation-package-lifecycle-rootless.json"),
  "externally_managed" => ROOT.join("tests/conformance/v1/valid/linux-isolation-package-lifecycle-external.json"),
}.freeze
BINARIES = %w[impresari-context impresari-context-mcp impresari-context-structural-worker].freeze

source = REHEARSAL.read
abort("package rehearsal contains privilege escalation") if source.match?(/\bsudo\b|\bpkexec\b/)
abort("package rehearsal contains network access") if source.match?(/\bcurl\b|\bwget\b|Net::HTTP|TCPSocket/)
abort("package rehearsal contains service mutation") if source.match?(/systemctl|loginctl|systemd-run/)
abort("package rehearsal does not require exact candidate source identity") unless source.include?("candidate source identity mismatch")
abort("package rehearsal does not reject same-identity replacement") unless source.include?("baseline and candidate archive identities must differ")
abort("package rehearsal does not preserve the real-login gate") unless source.include?("real_login_session_required")

FIXTURES.each do |profile, path|
  fixture = JSON.parse(path.read)
  abort("fixture profile mismatch") unless fixture.fetch("profile") == profile
  abort("fixture admitted production") unless fixture.fetch("claims").values.none?
end

def build_package(directory, label, version, source_commit)
  package = "impresari-context-#{version}-x86_64-unknown-linux-gnu"
  root = File.join(directory, "#{label}-root", package)
  FileUtils.mkdir_p(File.join(root, "bin"))
  entries = BINARIES.map do |name|
    content = if name == "impresari-context"
      "#!/bin/sh\nprintf '%s\\n' '{\"schema_name\":\"error-envelope\",\"schema_version\":\"1.0.0\",\"code\":\"invalid_input\",\"retryable\":false,\"partial_result\":false,\"recovery_action\":\"none\"}'\nexit 1\n"
    else
      "#!/bin/sh\nexit 0\n"
    end
    path = File.join(root, "bin", name)
    File.binwrite(path, content)
    File.chmod(0o755, path)
    { "path" => "bin/#{name}", "bytes" => content.bytesize.to_s, "sha256" => Digest::SHA256.hexdigest(content) }
  end
  manifest = {
    "schema_name" => "release-candidate-manifest",
    "schema_version" => "1.0.0",
    "project_version" => version,
    "target" => "x86_64-unknown-linux-gnu",
    "source_commit" => source_commit,
    "rust_toolchain" => "1.98.0",
    "files" => entries,
  }
  File.write(File.join(root, "MANIFEST.json"), "#{JSON.pretty_generate(manifest)}\n")
  archive = File.join(directory, "#{label}.tar.gz")
  Zlib::GzipWriter.open(archive) do |gzip|
    Gem::Package::TarWriter.new(gzip) do |tar|
      Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH).select { |path| File.file?(path) }.sort.each do |path|
        relative = "#{package}/#{path.delete_prefix("#{root}/")}"
        bytes = File.binread(path)
        tar.add_file_simple(relative, path.include?("/bin/") ? 0o755 : 0o644, bytes.bytesize) { |entry| entry.write(bytes) }
      end
    end
  end
  File.write("#{archive}.sha256", "#{Digest::SHA256.file(archive).hexdigest}  #{File.basename(archive)}\n")
  archive
end

if RUBY_PLATFORM.include?("linux")
  Dir.mktmpdir("impresari-package-check-") do |directory|
    baseline_sha = "1" * 40
    candidate_sha = "2" * 40
    baseline = build_package(directory, "baseline", "0.1.0", baseline_sha)
    candidate = build_package(directory, "candidate", "0.2.0", candidate_sha)
    FIXTURES.each_key do |profile|
      output, status = Open3.capture2e(
        { "RUNNER_ENVIRONMENT" => "github-hosted" }, "ruby", REHEARSAL.to_s,
        "--profile", profile, "--baseline", baseline, "--candidate", candidate,
        "--candidate-source-sha", candidate_sha
      )
      abort("#{profile} synthetic package rehearsal failed: #{output}") unless status.success?
      receipt = JSON.parse(output)
      expected_status = profile == "externally_managed" ? "package_lifecycle_candidate" : "package_lifecycle_partial"
      abort("#{profile} synthetic status mismatch") unless receipt.fetch("status") == expected_status
      abort("#{profile} synthetic rehearsal overclaimed") unless receipt.fetch("claims").values.none?
    end

    output, status = Open3.capture2e(
      { "RUNNER_ENVIRONMENT" => "github-hosted" }, "ruby", REHEARSAL.to_s,
      "--profile", "externally_managed", "--baseline", baseline, "--candidate", baseline,
      "--candidate-source-sha", baseline_sha
    )
    abort("same-identity replacement was accepted") if status.success?
    abort("same-identity failure reason drifted") unless output.include?("archive identities must differ")
  end
end

puts "linux package lifecycle rehearsal checks passed: exact package scope, honest A reentry gate, and bounded C relaunch"
