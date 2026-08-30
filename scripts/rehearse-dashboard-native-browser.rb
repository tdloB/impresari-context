#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "timeout"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CLI = ROOT.join("target/debug/impresari-context").to_s

options = { cli: DEFAULT_CLI, timeout: 600 }
OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-dashboard-native-browser.rb [options]"
  parser.on("--cli PATH", "Impresari Context CLI executable") { |value| options[:cli] = value }
  parser.on("--timeout SECONDS", Integer, "Maximum browser rehearsal duration") { |value| options[:timeout] = value }
end.parse!

abort("timeout must be between 60 and 1800 seconds") unless (60..1800).cover?(options[:timeout])

def run!(*command)
  stdout, stderr, status = Open3.capture3(*command, chdir: ROOT.to_s)
  abort("command failed: #{command.join(' ')}\n#{stderr}\n#{stdout}") unless status.success?
  stdout
end

run!("cargo", "build", "--locked", "-p", "context-cli")
abort("missing executable: #{options[:cli]}") unless File.file?(options[:cli]) && File.executable?(options[:cli])

removed_root = nil
receipt = nil
Dir.mktmpdir("impresari-dbc4-native-", "/private/tmp") do |temporary_root|
  removed_root = temporary_root
  fixture = JSON.parse(run!(
    "cargo", "run", "--quiet", "--locked", "-p", "context-dashboard-server",
    "--example", "dbc4_fixture", "--", temporary_root,
  ))
  manifest_path = File.join(temporary_root, "private-canaries.json")
  manifest_bytes = File.binread(manifest_path)
  manifest = JSON.parse(manifest_bytes)
  canaries = manifest.fetch("canaries")
  audit_root = fixture.fetch("audit_root")
  policy_root = fixture.fetch("policy_root")
  environment = { "IMPRESARI_DBC4_PRIVATE_ENV" => canaries.fetch(6) }

  stdin = stdout = stderr = wait_thread = nil
  captured_stdout = +""
  captured_stderr = +""
  begin
    stdin, stdout, stderr, wait_thread = Open3.popen3(
      environment, options[:cli], "dashboard", "serve", audit_root, policy_root,
      chdir: ROOT.to_s,
    )
    stdin.close
    readiness_line = Timeout.timeout(15) { stdout.gets }
    abort("dashboard did not emit readiness") unless readiness_line
    captured_stdout << readiness_line
    readiness = JSON.parse(readiness_line)
    stderr_reader = Thread.new { captured_stderr << stderr.read }
    puts JSON.generate({
      "status" => "awaiting_native_browser",
      "schema_name" => "dbc4-browser-rehearsal-ready",
      "schema_version" => "1.0.0",
      "bootstrap_url" => readiness.fetch("bootstrap_url"),
      "asset_sha256" => readiness.fetch("asset_sha256"),
      "private_manifest_sha256" => fixture.fetch("private_manifest_sha256"),
      "expected_valid_rows" => fixture.fetch("valid_rows"),
      "expected_withheld_rows" => fixture.fetch("withheld_rows"),
    })
    $stdout.flush
    Timeout.timeout(options[:timeout]) do
      captured_stdout << stdout.read
      wait_thread.join
    end
    stderr_reader.join
    abort("dashboard exited unsuccessfully") unless wait_thread.value.success?

    policy_bytes = Dir.glob(File.join(policy_root, "**", "*"), File::FNM_DOTMATCH)
                      .select { |path| File.file?(path) }
                      .sort
                      .map { |path| File.binread(path) }
                      .join
    product_outputs = captured_stdout + captured_stderr + policy_bytes
    leaked = canaries.any? { |canary| product_outputs.include?(canary) }
    abort("a private DBC-4 canary escaped into process output or policy state") if leaked

    receipt = {
      "status" => "passed",
      "schema_name" => "dbc4-native-browser-host-receipt",
      "schema_version" => "1.0.0",
      "asset_sha256" => readiness.fetch("asset_sha256"),
      "private_manifest_sha256" => "sha256:#{Digest::SHA256.hexdigest(manifest_bytes)}",
      "source_canaries_absent_from_process_output" => true,
      "source_canaries_absent_from_policy_state" => true,
      "dashboard_exit_success" => true,
      "external_network_required" => false,
      "source_workspace_used" => false,
    }
  ensure
    if wait_thread&.alive?
      Process.kill("TERM", wait_thread.pid)
      wait_thread.join(5)
      Process.kill("KILL", wait_thread.pid) if wait_thread.alive?
    end
    stdout&.close unless stdout&.closed?
    stderr&.close unless stderr&.closed?
  end
end

abort("disposable DBC-4 fixture cleanup failed") if removed_root && File.exist?(removed_root)
receipt["disposable_fixture_removed"] = true
puts JSON.pretty_generate(receipt)
