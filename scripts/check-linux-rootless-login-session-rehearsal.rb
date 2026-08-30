#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "open3"
require "pathname"
require "tmpdir"
require_relative "lib/linux_rootless_login_session_rehearsal"

ROOT = Pathname.new(__dir__).join("..").expand_path
COMPOSER = ROOT.join("scripts/linux-rootless-login-session-compose.rb")
OBSERVER = ROOT.join("scripts/linux-rootless-login-session-observe.rb")
LIVE = ROOT.join("scripts/linux-rootless-login-session-live.sh")
PACKAGE_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-package-lifecycle-rootless.json")
SOURCE_COMMIT = "6" * 40

source = COMPOSER.read
abort("login-session composer contains privilege escalation") if source.match?(/\bsudo\b|\bpkexec\b|\buseradd\b|\buserdel\b/)
abort("login-session composer contains process launch") if source.match?(/Open3|system\(|spawn\(|exec\(/)
abort("login-session composer contains network access") if source.match?(/Net::HTTP|TCPSocket|UDPSocket|\bcurl\b|\bwget\b/)
observer_source = OBSERVER.read
abort("session observer accepts a raw command") if observer_source.match?(/--command|Shellwords|system\(|spawn\(|exec\(/)
abort("session observer contains network access") if observer_source.match?(/Net::HTTP|TCPSocket|UDPSocket|\bcurl\b|\bwget\b/)
live_source = LIVE.read
abort("live rehearsal mutates the system SSH service") if live_source.match?(/systemctl\s+(?:start|stop|restart|enable|disable)\s+(?:ssh|sshd)/)
abort("live rehearsal enables lingering") if live_source.include?("enable-linger")
abort("live rehearsal installs a package") if live_source.match?(/\bapt(?:-get)?\b|\bdnf\b|\byum\b/)
abort("live rehearsal listens beyond loopback") unless live_source.include?("ListenAddress 127.0.0.1")
abort("live rehearsal does not force exact session commands") unless live_source.include?('restrict,command="%s"')

HOST = {
  "operating_system" => "linux",
  "kernel_release" => "6.17.0-synthetic",
  "architecture" => "x86_64",
  "environment" => "github-hosted-ephemeral",
}.freeze

def session(ordinal, session_identity, manager_identity)
  {
    "schema_name" => "linux-rootless-login-session-observation",
    "schema_version" => "1.0.0",
    "expected_source_commit" => SOURCE_COMMIT,
    "host" => HOST,
    "ordinal" => ordinal,
    "login_kind" => "pam_logind",
    "session_identity" => session_identity,
    "user_manager_invocation_identity" => manager_identity,
    "lingering_enabled" => false,
    "preflight" => {"status" => "ready_for_synthetic_rehearsal", "receipt_identity" => (ordinal == 1 ? "c" : "8") * 64},
    "synthetic_rehearsal" => {"status" => "candidate_passed", "receipt_identity" => (ordinal == 1 ? "d" : "9") * 64, "real_analyzer_used" => false},
    "package" => {
      "candidate_archive_sha256" => "4" * 64,
      "candidate_manifest_sha256" => "5" * 64,
      "source_commit" => SOURCE_COMMIT,
      "binary_set_identity" => "7" * 64,
    },
    "ended_cleanly" => true,
    "user_manager_terminated" => true,
  }
end

def cleanup
  LinuxRootlessLoginSessionRehearsal::CLEANUP_KEYS.to_h { |key| [key, true] }
end

package_bytes = PACKAGE_FIXTURE.binread
package = JSON.parse(package_bytes)
first = session(1, "a" * 64, "b" * 64)
second = session(2, "e" * 64, "f" * 64)

def build(package_bytes, package, first, second, cleanup_value)
  LinuxRootlessLoginSessionRehearsal.build(
    expected_source: SOURCE_COMMIT,
    package_bytes: package_bytes,
    package: package,
    first: first,
    second: second,
    cleanup: cleanup_value,
  )
end

candidate = build(package_bytes, package, first, second, cleanup)
abort("genuine session candidate missing") unless candidate.fetch("status") == "login_session_candidate"
abort("candidate reentry claim missing") unless candidate.fetch("rootless_reentry_candidate_active") == true
abort("candidate admitted production") unless candidate.values_at("production_admitted", "real_analyzer_authorized", "privileged_installation_authorized", "persistent_service_authorized").none?
abort("candidate package identity drifted") unless candidate.dig("package", "identity_preserved_across_sessions") == true
abort("candidate transition incomplete") unless candidate.fetch("transition").values_at("first_session_closed", "first_user_manager_terminated", "distinct_session_identity", "distinct_user_manager_identity", "same_package_identity").all?

unsupported_first = Marshal.load(Marshal.dump(first))
unsupported_first["host"] = HOST.merge("environment" => "local")
unsupported_second = Marshal.load(Marshal.dump(second))
unsupported_second["host"] = unsupported_first["host"]
abort("unsupported host did not fail closed") unless build(package_bytes, package, unsupported_first, unsupported_second, cleanup).fetch("status") == "unsupported"

failed_second = Marshal.load(Marshal.dump(second))
failed_second["synthetic_rehearsal"]["status"] = "failed"
abort("failed session did not fail closed") unless build(package_bytes, package, first, failed_second, cleanup).fetch("status") == "session_failed"

same_manager = Marshal.load(Marshal.dump(second))
same_manager["user_manager_invocation_identity"] = first.fetch("user_manager_invocation_identity")
abort("manager restart substitution was accepted") unless build(package_bytes, package, first, same_manager, cleanup).fetch("status") == "identity_mismatch"

dirty_cleanup = cleanup.merge("home_absent" => false)
abort("dirty teardown was accepted") unless build(package_bytes, package, first, second, dirty_cleanup).fetch("status") == "cleanup_failed"

wrong_source = Marshal.load(Marshal.dump(second))
wrong_source["package"]["source_commit"] = "1" * 40
begin
  build(package_bytes, package, first, wrong_source, cleanup)
  abort("source identity mismatch was accepted")
rescue LinuxRootlessLoginSessionRehearsal::ContractError => error
  abort("source mismatch reason drifted") unless error.message.include?("source mismatch")
end

Dir.mktmpdir("impresari-login-session-compose-") do |directory|
  paths = {
    package: File.join(directory, "package.json"),
    first: File.join(directory, "first.json"),
    second: File.join(directory, "second.json"),
    cleanup: File.join(directory, "cleanup.json"),
  }
  File.binwrite(paths.fetch(:package), package_bytes)
  File.write(paths.fetch(:first), JSON.generate(first))
  File.write(paths.fetch(:second), JSON.generate(second))
  File.write(paths.fetch(:cleanup), JSON.generate(cleanup))
  output, status = Open3.capture2e(
    "ruby", COMPOSER.to_s,
    "--expected-source-sha", SOURCE_COMMIT,
    "--package", paths.fetch(:package),
    "--first-session", paths.fetch(:first),
    "--second-session", paths.fetch(:second),
    "--cleanup", paths.fetch(:cleanup),
  )
  abort("source-free login-session composition failed: #{output}") unless status.success?
  receipt = JSON.parse(output)
  abort("source-free composition did not produce a candidate") unless receipt.fetch("status") == "login_session_candidate"
end

puts "linux rootless login-session checks passed: genuine reentry candidate and 5 fail-closed boundaries"
