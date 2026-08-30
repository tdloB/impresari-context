#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require_relative "lib/linux_external_delegation_live_rehearsal"

ROOT = Pathname.new(__dir__).join("..").expand_path
LIVE = ROOT.join("scripts/linux-external-delegation-live-rehearsal.sh")
RECEIVER = ROOT.join("scripts/linux-external-delegation-live-receiver.rb")
FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-external-delegation-live-rehearsal-candidate.json")

live_source = LIVE.read
receiver_source = RECEIVER.read
abort("external rehearsal must use exactly one sudo systemd-run") unless live_source.scan(/sudo systemd-run/).length == 1
abort("external rehearsal delegation drift") unless live_source.include?('--property="Delegate=cpu memory pids"')
abort("external rehearsal is persistent") unless live_source.include?("--collect")
abort("receiver descriptor is not fixed") unless receiver_source.include?("LinuxExternalDelegationCapability::FIXED_DESCRIPTOR")
abort("receiver does not close descriptor inheritance") unless receiver_source.include?("capability.close_on_exec = true")
abort("receiver accepts arguments") unless receiver_source.include?('unless ARGV.empty?')

def facts
  {
    "observed_host" => {"operating_system" => "linux", "kernel_release" => "6.17.0-synthetic", "architecture" => "x86_64"},
    "service_created" => true,
    "capability" => {"transport" => "inherited_directory_fd", "descriptor_slot" => 3, "received" => true, "directory_verified" => true, "close_on_exec_set" => true, "raw_path_received" => false, "raw_cgroup_path_recorded" => false},
    "revalidation" => {"executed" => true, "unified_cgroup_v2" => true, "owner_verified" => true, "process_contained" => true, "exclusive_descendants" => true, "delegation_writable" => true, "controllers" => %w[cpu memory pids], "raw_cgroup_path_recorded" => false},
    "composite" => {"executed" => true, "result" => "candidate_passed", "receipt_identity" => "a" * 64, "real_analyzer_used" => false},
    "cleanup" => {"attempted" => true, "descendants_removed" => true},
  }
end

candidate = LinuxExternalDelegationLiveRehearsal.build(facts: facts, provisioner_collected: true)
abort("candidate fixture drift") unless candidate == JSON.parse(FIXTURE.read)
cases = {
  "candidate" => candidate,
  "capability" => LinuxExternalDelegationLiveRehearsal.build(facts: facts.tap { |value| value["capability"]["directory_verified"] = false }, provisioner_collected: true),
  "revalidation" => LinuxExternalDelegationLiveRehearsal.build(facts: facts.tap { |value| value["revalidation"]["owner_verified"] = false }, provisioner_collected: true),
  "composite" => LinuxExternalDelegationLiveRehearsal.build(facts: facts.tap { |value| value["composite"]["result"] = "failed" }, provisioner_collected: true),
  "cleanup" => LinuxExternalDelegationLiveRehearsal.build(facts: facts.tap { |value| value["cleanup"]["descendants_removed"] = false }, provisioner_collected: true),
  "provisioner_cleanup" => LinuxExternalDelegationLiveRehearsal.build(facts: facts, provisioner_collected: false),
}
expected = {"candidate" => "candidate_passed", "capability" => "capability_failed", "revalidation" => "revalidation_failed", "composite" => "composite_failed", "cleanup" => "cleanup_failed", "provisioner_cleanup" => "provisioner_cleanup_failed"}
expected.each { |name, status| abort("unexpected #{name}") unless cases.fetch(name).fetch("status") == status }
cases.each do |name, receipt|
  active = name == "candidate"
  abort("#{name} candidate drift") unless receipt.fetch("external_candidate_active") == active
  abort("#{name} OS claim drift") unless receipt.fetch("os_confined") == active
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} authorized privileged install") unless receipt.fetch("privileged_installation_authorized") == false
end

puts "linux external live rehearsal checks passed: 1 candidate and 5 fail-closed states"
