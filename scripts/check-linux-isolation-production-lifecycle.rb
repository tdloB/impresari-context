#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
EVALUATOR = ROOT.join("scripts/linux-isolation-production-lifecycle.rb")
POLICY = ROOT.join("linux-isolation/linux-iar-1b-production-lifecycle-v1.json")
POLICY_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-production-lifecycle-policy.json")
OBSERVATIONS_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-production-lifecycle-observations.json")
RECEIPT_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-production-lifecycle-receipt.json")

def evaluate(observations_path)
  stdout, stderr, status = Open3.capture3(
    RbConfig.ruby, EVALUATOR.to_s,
    "--policy", POLICY.to_s,
    "--observations", observations_path.to_s,
  )
  abort("lifecycle evaluator failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

def rootless_observations
  JSON.parse(OBSERVATIONS_FIXTURE.read)
end

def external_observations
  document = rootless_observations
  document["profile"] = "externally_managed"
  relaunch = document.fetch("phases")[3]
  relaunch["phase"] = "operator_relaunch"
  relaunch["operation_evidence"] = "operator_relaunch_verified"
  document
end

def with_observations(document)
  Dir.mktmpdir("impresari-linux-lifecycle-") do |directory|
    path = Pathname.new(directory).join("observations.json")
    path.write(JSON.pretty_generate(document) + "\n")
    yield path
  end
end

abort("lifecycle policy fixture drift") unless POLICY.binread == POLICY_FIXTURE.binread
candidate = evaluate(OBSERVATIONS_FIXTURE)
abort("lifecycle receipt fixture drift") unless candidate == JSON.parse(RECEIPT_FIXTURE.read)

cases = {"rootless_candidate" => candidate}
with_observations(external_observations) { |path| cases["external_candidate"] = evaluate(path) }

incomplete = rootless_observations
uninstall = incomplete.fetch("phases").last
uninstall["outcome"] = "not_observed"
uninstall["package_identity_verified"] = false
uninstall["topology_revalidated"] = false
uninstall["clean_state"].transform_values! { false }
with_observations(incomplete) { |path| cases["incomplete"] = evaluate(path) }

failed = rootless_observations
failed.fetch("phases")[4]["outcome"] = "failed"
with_observations(failed) { |path| cases["lifecycle_failed"] = evaluate(path) }

withdrawal = rootless_observations
withdrawal.fetch("phases")[6]["claim_withdrawn"] = false
with_observations(withdrawal) { |path| cases["withdrawal_failed"] = evaluate(path) }

invalid = rootless_observations
invalid.fetch("phases")[1]["operation_evidence"] = "rollback_identity_restored"
with_observations(invalid) { |path| cases["invalid_contract"] = evaluate(path) }

expected = {
  "rootless_candidate" => "lifecycle_candidate",
  "external_candidate" => "lifecycle_candidate",
  "incomplete" => "incomplete",
  "lifecycle_failed" => "lifecycle_failed",
  "withdrawal_failed" => "withdrawal_failed",
  "invalid_contract" => "invalid_contract",
}
expected.each do |name, status|
  abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status
end

cases.each do |name, receipt|
  active = %w[rootless_candidate external_candidate].include?(name)
  abort("#{name} lifecycle candidate state is unsafe") unless receipt.fetch("lifecycle_candidate_active") == active
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} authorized release packaging") unless receipt.fetch("release_packaging_authorized") == false
  abort("#{name} authorized privileged installation") unless receipt.fetch("privileged_installation_authorized") == false
  abort("#{name} authorized a persistent service") unless receipt.fetch("persistent_service_authorized") == false
  abort("#{name} granted authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end
abort("rootless health withdrawal was not verified") unless cases.fetch("rootless_candidate").fetch("health_withdrawal_verified")
abort("failed health withdrawal was reported as verified") if cases.fetch("withdrawal_failed").fetch("health_withdrawal_verified")
abort("external profile did not use operator relaunch") unless cases.fetch("external_candidate").fetch("evaluated_phases").include?("operator_relaunch")

Dir.mktmpdir("impresari-linux-lifecycle-malformed-") do |directory|
  malformed = Pathname.new(directory).join("policy.json")
  malformed.write("{not-json")
  _stdout, _stderr, status = Open3.capture3(
    RbConfig.ruby, EVALUATOR.to_s,
    "--policy", malformed.to_s,
    "--observations", OBSERVATIONS_FIXTURE.to_s,
  )
  abort("malformed lifecycle policy was accepted") if status.success?
end

puts "linux isolation production-lifecycle checks passed: 2 selected profiles and 4 fail-closed states"
