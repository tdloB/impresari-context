#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require_relative "lib/linux_rootless_user_manager_rehearsal"

ROOT = Pathname.new(__dir__).join("..").expand_path
POLICY_IDENTITY = "03ff04052dae6f7990805011fe454774c3f2ba209a9cf0eea083097eacb7bac4"
FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-rootless-user-manager-rehearsal-candidate.json")
LIVE_SCRIPT = ROOT.join("scripts/linux-rootless-user-manager-rehearsal.rb")

live_source = LIVE_SCRIPT.read
abort("rootless rehearsal contains a privileged fallback") if live_source.match?(/\bsudo\b|\bpkexec\b|--system/)
abort("rootless rehearsal does not pin the user manager") unless live_source.include?('"--user"')
abort("rootless rehearsal does not request collection") unless live_source.include?('"--collect"')
abort("rootless rehearsal controller request drift") unless live_source.include?('"--property=Delegate=cpu memory pids"')

def preflight(status = "ready_for_synthetic_rehearsal")
  {
    "status" => status,
    "observed" => {
      "platform" => "linux",
      "kernel_release" => "6.17.0-synthetic",
      "raw_cgroup_path_recorded" => false,
    },
  }
end

def composite(valid = true)
  checks = LinuxRootlessUserManagerRehearsal::REQUIRED_COMPOSITE_CHECKS.to_h { |name| [name, true] }
  checks["cleanup"] = false unless valid
  {
    "schema_name" => "linux-isolation-feasibility",
    "result" => valid ? "candidate_passed" : "failed",
    "preflight" => {"cgroup_v2" => true, "cpu_controller" => true, "memory_controller" => true, "pids_controller" => true},
    "checks" => checks,
    "limitations" => valid ? ["single-host-evidence", "synthetic-probe-no-analysis"] : ["synthetic-probe-no-analysis"],
    "os_confined" => valid,
    "production_admitted" => false,
    "source_retained" => false,
    "authority_added" => false,
  }
end

def build(preflight_status: "ready_for_synthetic_rehearsal", attempted: true, created: true, collected: true, composite_value: composite)
  preflight_value = preflight(preflight_status)
  preflight_bytes = JSON.generate(preflight_value)
  composite_bytes = composite_value ? JSON.generate(composite_value) : nil
  LinuxRootlessUserManagerRehearsal.build(
    policy_identity: POLICY_IDENTITY,
    preflight_bytes: preflight_bytes,
    preflight: preflight_value,
    architecture: "x86_64",
    attempted: attempted,
    created: created,
    collected: collected,
    composite_bytes: composite_bytes,
    composite: composite_value,
  )
end

cases = {
  "candidate" => build,
  "preflight_skipped" => build(preflight_status: "insufficient_delegation", attempted: false, created: false, collected: false, composite_value: nil),
  "launch_failed" => build(created: false, collected: false, composite_value: nil),
  "composite_failed" => build(composite_value: composite(false)),
  "cleanup_failed" => build(collected: false),
}
expected = {
  "candidate" => "candidate_passed",
  "preflight_skipped" => "skipped_preflight",
  "launch_failed" => "launch_failed",
  "composite_failed" => "composite_failed",
  "cleanup_failed" => "cleanup_failed",
}
expected.each { |name, status| abort("unexpected #{name}") unless cases.fetch(name).fetch("status") == status }
abort("candidate fixture drift") unless cases.fetch("candidate") == JSON.parse(FIXTURE.read)
cases.each do |name, receipt|
  active = name == "candidate"
  abort("#{name} candidate claim is unsafe") unless receipt.fetch("rootless_candidate_active") == active
  abort("#{name} OS claim is unsafe") unless receipt.fetch("os_confined") == active
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} authorized privileged installation") unless receipt.fetch("privileged_installation_authorized") == false
  abort("#{name} used privilege") unless receipt.dig("authority", "privilege_use") == "denied"
end

puts "linux rootless user-manager rehearsal checks passed: 1 candidate and 4 fail-closed states"
