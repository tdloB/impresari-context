#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"
require_relative "lib/linux_rootless_host_preflight"

ROOT = Pathname.new(__dir__).join("..").expand_path
POLICY = ROOT.join("linux-isolation/linux-iar-1b-production-topology-v1.json")
FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-rootless-host-preflight-ready.json")
IDENTITY = LinuxRootlessHostPreflight.policy_identity(POLICY)

Dir.mktmpdir("impresari-rootless-preflight-") do |directory|
  empty = File.join(directory, "empty")
  File.write(empty, "")
  abort("empty platform file was not handled") unless LinuxRootlessHostPreflight.fixed_read(empty, 16) == ""
end

def observation(overrides = {})
  LinuxRootlessHostPreflight.base_observation.merge(
    "platform" => "linux",
    "kernel_release" => "6.17.0-synthetic",
    "cgroup_mode" => "unified_v2",
    "current_membership_valid" => true,
    "user_manager_cgroup_present" => true,
    "user_manager_transport_present" => true,
    "user_manager_process_present" => true,
    "delegation_write_marker" => true,
    "controllers" => %w[cpu memory pids],
  ).merge(overrides)
end

def assess(overrides = {})
  LinuxRootlessHostPreflight.assess(IDENTITY, observation(overrides))
end

cases = {
  "ready" => assess,
  "non_linux" => assess("platform" => "other", "kernel_release" => "not_observed", "cgroup_mode" => "unavailable"),
  "cgroup_unavailable" => assess("cgroup_mode" => "unavailable"),
  "legacy" => assess("cgroup_mode" => "legacy_or_hybrid"),
  "invalid_membership" => assess("current_membership_valid" => false),
  "manager_cgroup_missing" => assess("user_manager_cgroup_present" => false),
  "manager_transport_missing" => assess("user_manager_transport_present" => false),
  "manager_process_missing" => assess("user_manager_process_present" => false),
  "controller_missing" => assess("controllers" => %w[memory pids]),
  "delegation_marker_missing" => assess("delegation_write_marker" => false),
}

expected = {
  "ready" => "ready_for_synthetic_rehearsal",
  "non_linux" => "unsupported",
  "cgroup_unavailable" => "unavailable",
  "legacy" => "unsupported",
  "invalid_membership" => "invalid_host_state",
  "manager_cgroup_missing" => "unavailable",
  "manager_transport_missing" => "unavailable",
  "manager_process_missing" => "unavailable",
  "controller_missing" => "insufficient_delegation",
  "delegation_marker_missing" => "insufficient_delegation",
}
expected.each do |name, status|
  abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status
end

abort("ready fixture drift") unless cases.fetch("ready") == JSON.parse(FIXTURE.read)
cases.each do |name, receipt|
  active = name == "ready"
  abort("#{name} preflight candidate state is unsafe") unless receipt.fetch("preflight_candidate_active") == active
  abort("#{name} claimed a synthetic child cycle") unless receipt.fetch("synthetic_child_cycle_executed") == false
  abort("#{name} claimed OS confinement") unless receipt.fetch("os_confined") == false
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} authorized privileged installation") unless receipt.fetch("privileged_installation_authorized") == false
  abort("#{name} recorded a raw cgroup path") unless receipt.dig("observed", "raw_cgroup_path_recorded") == false
  denied = receipt.fetch("authority").reject { |key, _value| %w[policy_read host_metadata_read].include?(key) }
  abort("#{name} granted operational authority") unless denied.values.all? { |value| value == "denied" }
end

stdout, stderr, status = Open3.capture3(RbConfig.ruby, ROOT.join("scripts/linux-rootless-host-preflight.rb").to_s)
abort("live host preflight failed: #{stderr}") unless status.success?
live = JSON.parse(stdout)
abort("live preflight recorded a raw path") unless live.dig("observed", "raw_cgroup_path_recorded") == false
abort("live preflight overclaimed production") unless live.fetch("production_admitted") == false && live.fetch("os_confined") == false

puts "linux rootless host preflight checks passed: 1 ready candidate and 9 fail-closed cases; live=#{live.fetch('status')}"
