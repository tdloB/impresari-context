#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
EVALUATOR = ROOT.join("scripts/linux-isolation-topology-feasibility.rb")
POLICY = ROOT.join("linux-isolation/linux-iar-1b-production-topology-v1.json")
POLICY_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-production-topology-policy.json")
RECEIPT_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-production-topology-receipt.json")

def run_check(overrides = {})
  options = {
    profile: "rootless_user_manager",
    cgroup_mode: "unified_v2",
    user_manager_available: true,
    delegation_marker: true,
    controllers: %w[cpu memory pids],
    process_contained: true,
    descendant_ownership_exclusive: true,
    synthetic_child_cycle: true,
    external_capability: "none",
    external_owner_verified: false,
    external_containment_verified: false,
  }.merge(overrides)
  command = [
    RbConfig.ruby, EVALUATOR.to_s,
    "--policy", (options.delete(:policy) || POLICY).to_s,
    "--profile", options.fetch(:profile),
    "--cgroup-mode", options.fetch(:cgroup_mode),
    "--user-manager", options.fetch(:user_manager_available) ? "yes" : "no",
    "--delegation-marker", options.fetch(:delegation_marker) ? "yes" : "no",
    "--controllers", options.fetch(:controllers).join(","),
    "--process-contained", options.fetch(:process_contained) ? "yes" : "no",
    "--exclusive-descendants", options.fetch(:descendant_ownership_exclusive) ? "yes" : "no",
    "--synthetic-child-cycle", options.fetch(:synthetic_child_cycle) ? "yes" : "no",
    "--external-capability", options.fetch(:external_capability),
    "--external-owner-verified", options.fetch(:external_owner_verified) ? "yes" : "no",
    "--external-containment-verified", options.fetch(:external_containment_verified) ? "yes" : "no",
  ]
  stdout, stderr, status = Open3.capture3(*command)
  abort("topology evaluator failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

cases = {
  "rootless_feasible" => run_check,
  "external_feasible" => run_check(
    profile: "externally_managed",
    user_manager_available: false,
    external_capability: "inherited_directory_fd",
    external_owner_verified: true,
    external_containment_verified: true,
  ),
  "administrator_deferred" => run_check(profile: "administrator_provisioned"),
  "legacy_unsupported" => run_check(cgroup_mode: "legacy_or_hybrid"),
  "manager_unavailable" => run_check(user_manager_available: false),
  "controller_missing" => run_check(controllers: %w[memory pids]),
  "raw_path_rejected" => run_check(
    profile: "externally_managed",
    user_manager_available: false,
    external_capability: "raw_path",
  ),
  "external_unverified" => run_check(
    profile: "externally_managed",
    user_manager_available: false,
    external_capability: "inherited_directory_fd",
    external_owner_verified: true,
  ),
  "child_cycle_failed" => run_check(synthetic_child_cycle: false),
}

abort("topology policy fixture drift") unless POLICY.binread == POLICY_FIXTURE.binread
abort("rootless receipt fixture drift") unless cases.fetch("rootless_feasible") == JSON.parse(RECEIPT_FIXTURE.read)

expected = {
  "rootless_feasible" => "feasible_candidate",
  "external_feasible" => "feasible_candidate",
  "administrator_deferred" => "unsupported",
  "legacy_unsupported" => "unsupported",
  "manager_unavailable" => "unavailable",
  "controller_missing" => "insufficient_delegation",
  "raw_path_rejected" => "invalid_contract",
  "external_unverified" => "insufficient_delegation",
  "child_cycle_failed" => "insufficient_delegation",
}
expected.each do |name, status|
  abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status
end

cases.each do |name, receipt|
  expected_active = %w[rootless_feasible external_feasible].include?(name)
  abort("#{name} feasibility claim state is unsafe") unless receipt.fetch("feasibility_claim_active") == expected_active
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} authorized privileged installation") unless receipt.fetch("privileged_installation_authorized") == false
  abort("#{name} granted authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end
abort("administrator profile did not remain deferred") unless cases.fetch("administrator_deferred").fetch("reason_code") == "administrator_profile_deferred"
abort("raw external path was not rejected") unless cases.fetch("raw_path_rejected").fetch("reason_code") == "raw_path_rejected"

Dir.mktmpdir("impresari-linux-topology-") do |directory|
  malformed = Pathname.new(directory).join("policy.json")
  malformed.write("{not-json")
  command = [
    RbConfig.ruby, EVALUATOR.to_s, "--policy", malformed.to_s,
    "--profile", "rootless_user_manager", "--cgroup-mode", "unified_v2",
    "--user-manager", "yes", "--delegation-marker", "yes",
    "--controllers", "cpu,memory,pids", "--process-contained", "yes",
    "--exclusive-descendants", "yes", "--synthetic-child-cycle", "yes",
    "--external-capability", "none", "--external-owner-verified", "no",
    "--external-containment-verified", "no",
  ]
  _stdout, _stderr, status = Open3.capture3(*command)
  abort("malformed topology policy was accepted") if status.success?
end

puts "linux isolation topology checks passed: 2 selected profiles and 7 fail-closed cases"
