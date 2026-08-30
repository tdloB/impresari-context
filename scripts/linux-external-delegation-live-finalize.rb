#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require_relative "lib/linux_external_delegation_live_rehearsal"

abort("usage: ruby scripts/linux-external-delegation-live-finalize.rb") unless ARGV.empty?
root = Pathname.new(__dir__).join("..").expand_path
facts = JSON.parse(root.join("target/iar-linux-external-live/facts.json").read)
composite_path = root.join("target/iar-linux-composite-feasibility/receipt.json")
composite_receipt = JSON.parse(composite_path.read)
composite_passed = composite_receipt.fetch("result") == "candidate_passed" &&
  composite_receipt.dig("checks", "cleanup") == true
facts["service_created"] = true
facts["composite"] = {
  "executed" => true,
  "result" => composite_passed ? "candidate_passed" : "failed",
  "receipt_identity" => LinuxExternalDelegationLiveRehearsal.identity(composite_path),
  "real_analyzer_used" => false,
}
facts["cleanup"] = {"attempted" => true, "descendants_removed" => composite_passed}
receipt = LinuxExternalDelegationLiveRehearsal.build(facts: facts, provisioner_collected: true)
puts JSON.pretty_generate(receipt)
exit 7 unless receipt.fetch("status") == "candidate_passed"
