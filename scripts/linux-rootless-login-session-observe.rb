#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "open3"
require "optparse"
require "pathname"
require "rbconfig"

class ObservationError < StandardError; end

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby linux-rootless-login-session-observe.rb --ordinal N --source-root DIR --package-receipt FILE --expected-source-sha SHA"
  parser.on("--ordinal N", Integer) { |value| options[:ordinal] = value }
  parser.on("--source-root DIR") { |value| options[:source_root] = Pathname.new(value).expand_path }
  parser.on("--package-receipt FILE") { |value| options[:package_receipt] = Pathname.new(value).expand_path }
  parser.on("--expected-source-sha SHA") { |value| options[:expected_source] = value }
end.parse!

def bounded_json_command(*command)
  stdout, stderr, status = Open3.capture3(*command)
  raise ObservationError, "bounded command output exceeded limit" if stdout.bytesize > 256 * 1024 || stderr.bytesize > 32 * 1024
  raise ObservationError, "bounded command failed" unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError
  raise ObservationError, "bounded command emitted invalid JSON"
end

def sha256_file(path)
  raise ObservationError, "package binary is unavailable" unless path.file? && !path.symlink? && path.size <= 128 * 1024 * 1024
  Digest::SHA256.file(path).hexdigest
end

begin
  missing = %i[ordinal source_root package_receipt expected_source].reject { |key| options[key] }
  raise ObservationError, "missing required arguments" unless missing.empty? && ARGV.empty?
  raise ObservationError, "invalid session ordinal" unless [1, 2].include?(options.fetch(:ordinal))
  expected_source = options.fetch(:expected_source)
  raise ObservationError, "invalid source identity" unless expected_source.match?(/\A[0-9a-f]{40}\z/)
  raise ObservationError, "live observation requires an ephemeral GitHub-hosted Linux session" unless
    RUBY_PLATFORM.include?("linux") && ENV["GITHUB_ACTIONS"] == "true" && ENV["RUNNER_ENVIRONMENT"] == "github-hosted"
  raise ObservationError, "session observer must not run as root" if Process.uid.zero?

  source_root = options.fetch(:source_root)
  package_path = options.fetch(:package_receipt)
  raise ObservationError, "source root is unavailable" unless source_root.directory? && !source_root.symlink?
  raise ObservationError, "package receipt is unavailable" unless package_path.file? && !package_path.symlink? && package_path.size <= 256 * 1024
  package_receipt = JSON.parse(package_path.read)
  raise ObservationError, "package receipt source mismatch" unless package_receipt.dig("candidate", "source_commit") == expected_source

  session_id = ENV.fetch("XDG_SESSION_ID", "")
  raise ObservationError, "PAM/logind session identity is unavailable" unless session_id.match?(/\A[0-9A-Za-z_.-]{1,64}\z/)
  runtime, runtime_error, runtime_status = Open3.capture3(
    "/usr/bin/loginctl", "show-user", Process.uid.to_s, "--property=RuntimePath", "--value"
  )
  raise ObservationError, "logind runtime lookup failed: #{runtime_error.lines.first}" unless runtime_status.success?
  runtime = runtime.strip
  raise ObservationError, "logind runtime directory is unavailable" unless
    runtime == "/run/user/#{Process.uid}" && File.directory?(runtime) && !File.symlink?(runtime)
  ENV["XDG_RUNTIME_DIR"] = runtime

  manager_id, manager_error, manager_status = Open3.capture3(
    "/usr/bin/systemctl", "show", "user@#{Process.uid}.service", "--property=InvocationID", "--value"
  )
  raise ObservationError, "user-manager identity lookup failed: #{manager_error.lines.first}" unless manager_status.success?
  manager_id = manager_id.strip
  raise ObservationError, "user-manager invocation identity is unavailable" unless manager_id.match?(/\A[0-9a-f]{32}\z/)

  linger, linger_error, linger_status = Open3.capture3(
    "/usr/bin/loginctl", "show-user", Process.uid.to_s, "--property=Linger", "--value"
  )
  raise ObservationError, "linger lookup failed: #{linger_error.lines.first}" unless linger_status.success?
  raise ObservationError, "temporary session user unexpectedly lingers" unless linger.strip == "no"

  preflight_path = source_root.join("scripts/linux-rootless-host-preflight.rb")
  rehearsal_path = source_root.join("scripts/linux-rootless-user-manager-rehearsal.rb")
  preflight = bounded_json_command(RbConfig.ruby, preflight_path.to_s)
  rehearsal = bounded_json_command(RbConfig.ruby, rehearsal_path.to_s)
  preflight_bytes = JSON.generate(preflight)
  rehearsal_bytes = JSON.generate(rehearsal)

  binaries = %w[impresari-context impresari-context-mcp impresari-context-structural-worker]
  binary_digests = binaries.to_h do |name|
    [name, sha256_file(Pathname.new(ENV.fetch("HOME")).join(".local/bin", name))]
  end
  binary_set_identity = Digest::SHA256.hexdigest(
    binary_digests.sort.map { |name, digest| "#{name}:#{digest}\n" }.join
  )
  architecture = RbConfig::CONFIG.fetch("host_cpu")
  architecture = "other" unless %w[x86_64 aarch64].include?(architecture)
  observation = {
    "schema_name" => "linux-rootless-login-session-observation",
    "schema_version" => "1.0.0",
    "expected_source_commit" => expected_source,
    "host" => {
      "operating_system" => "linux",
      "kernel_release" => preflight.dig("observed", "kernel_release"),
      "architecture" => architecture,
      "environment" => "github-hosted-ephemeral",
    },
    "ordinal" => options.fetch(:ordinal),
    "login_kind" => "pam_logind",
    "session_identity" => Digest::SHA256.hexdigest("logind-session:\0#{session_id}"),
    "user_manager_invocation_identity" => Digest::SHA256.hexdigest("user-manager:\0#{manager_id}"),
    "lingering_enabled" => false,
    "preflight" => {"status" => preflight.fetch("status"), "receipt_identity" => Digest::SHA256.hexdigest(preflight_bytes)},
    "synthetic_rehearsal" => {
      "status" => rehearsal.fetch("status"),
      "receipt_identity" => Digest::SHA256.hexdigest(rehearsal_bytes),
      "real_analyzer_used" => false,
    },
    "package" => {
      "candidate_archive_sha256" => package_receipt.dig("candidate", "archive_sha256"),
      "candidate_manifest_sha256" => package_receipt.dig("candidate", "manifest_sha256"),
      "source_commit" => package_receipt.dig("candidate", "source_commit"),
      "binary_set_identity" => binary_set_identity,
    },
    "ended_cleanly" => true,
    "user_manager_terminated" => false,
  }
  puts JSON.pretty_generate(observation)
rescue ObservationError, JSON::ParserError, KeyError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
