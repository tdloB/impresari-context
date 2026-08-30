#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require_relative "lib/linux_rootless_host_preflight"
require_relative "lib/linux_rootless_user_manager_rehearsal"

abort("usage: ruby scripts/linux-rootless-user-manager-rehearsal.rb") unless ARGV.empty?
abort("rootless rehearsal is restricted to ephemeral GitHub-hosted Linux runners") unless RUBY_PLATFORM.include?("linux") && ENV["GITHUB_ACTIONS"] == "true" && ENV["RUNNER_ENVIRONMENT"] == "github-hosted"

begin
root = Pathname.new(__dir__).join("..").expand_path
policy = root.join("linux-isolation/linux-iar-1b-production-topology-v1.json")
policy_identity = LinuxRootlessHostPreflight.policy_identity(policy)
preflight_path = root.join("scripts/linux-rootless-host-preflight.rb")
preflight_bytes, preflight_stderr, preflight_status = Open3.capture3(RbConfig.ruby, preflight_path.to_s)
abort("bounded rootless preflight failed: #{preflight_stderr.lines.first}") unless preflight_status.success?
preflight = JSON.parse(preflight_bytes)
architecture = RbConfig::CONFIG.fetch("host_cpu")

unless preflight.fetch("status") == "ready_for_synthetic_rehearsal"
  puts JSON.pretty_generate(LinuxRootlessUserManagerRehearsal.build(
    policy_identity: policy_identity,
    preflight_bytes: preflight_bytes,
    preflight: preflight,
    architecture: architecture,
    attempted: false,
    created: false,
    collected: false,
  ))
  exit 0
end

run_id = ENV.fetch("GITHUB_RUN_ID")
attempt = ENV.fetch("GITHUB_RUN_ATTEMPT", "1")
abort("invalid hosted run identity") unless run_id.match?(/\A[1-9][0-9]{0,19}\z/) && attempt.match?(/\A[1-9][0-9]{0,5}\z/)
unit = "impresari-rootless-#{run_id}-#{attempt}"
systemd_run = "/usr/bin/systemd-run"
systemctl = "/usr/bin/systemctl"
abort("required user-manager client unavailable") unless File.executable?(systemd_run) && File.executable?(systemctl)

composite_script = root.join("scripts/check-linux-composite-feasibility.sh")
receipt_path = root.join("target/iar-linux-composite-feasibility/receipt.json")
if receipt_path.exist? || receipt_path.symlink?
  abort("unsafe prior composite receipt state") unless receipt_path.file? && !receipt_path.symlink?
  receipt_path.delete
end
command = [
  systemd_run,
  "--user",
  "--quiet",
  "--wait",
  "--pipe",
  "--collect",
  "--service-type=exec",
  "--unit=#{unit}",
  "--property=Delegate=cpu memory pids",
  "--setenv=GITHUB_ACTIONS=true",
  "--setenv=RUNNER_ENVIRONMENT=github-hosted",
  "--setenv=PATH=/usr/bin:/bin",
  "--working-directory=#{root}",
  composite_script.to_s,
  "--delegated",
]
_stdout, _stderr, launch_status = Open3.capture3(*command)

load_state, _cleanup_stderr, cleanup_status = Open3.capture3(systemctl, "--user", "show", unit, "--property=LoadState", "--value")
collected = cleanup_status.success? && load_state.strip == "not-found"
composite_bytes = if receipt_path.file? && !receipt_path.symlink? && receipt_path.size <= 65_536
  receipt_path.binread
end
composite = composite_bytes ? JSON.parse(composite_bytes) : nil

receipt = LinuxRootlessUserManagerRehearsal.build(
  policy_identity: policy_identity,
  preflight_bytes: preflight_bytes,
  preflight: preflight,
  architecture: architecture,
  attempted: true,
  created: launch_status.success? || !composite.nil?,
  collected: collected,
  composite_bytes: composite_bytes,
  composite: composite,
)
puts JSON.pretty_generate(receipt)
exit(receipt.fetch("status") == "candidate_passed" ? 0 : 7)
rescue JSON::ParserError
  abort("rootless rehearsal received malformed bounded JSON")
end
