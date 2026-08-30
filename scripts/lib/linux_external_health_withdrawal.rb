#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"

module LinuxExternalHealthWithdrawal
  module_function

  def identity(bytes)
    Digest::SHA256.hexdigest(bytes)
  end

  def build(package_bytes:, external_bytes:, capability_available:, clean_state:)
    withdrawn = capability_available == false && clean_state.values.all?
    {
      "schema_name" => "linux-external-health-withdrawal",
      "schema_version" => "1.0.0",
      "policy_id" => "linux-iar-1b-production-lifecycle-v1",
      "profile" => "externally_managed",
      "package_receipt_identity" => identity(package_bytes),
      "external_receipt_identity" => identity(external_bytes),
      "changed_prerequisite" => "inherited_delegation_capability_unavailable",
      "capability_descriptor" => 3,
      "capability_available" => capability_available,
      "topology_revalidated" => capability_available,
      "claim_withdrawn" => withdrawn,
      "clean_state" => clean_state,
      "status" => withdrawn ? "withdrawn" : "withdrawal_failed",
      "reason_code" => withdrawn ? "external_capability_absent_claim_withdrawn" : "external_capability_withdrawal_not_proven",
      "safe_next_step" => withdrawn ?
        "Retain this explicit withdrawal evidence; any later candidate requires a newly supplied and revalidated external capability." :
        "Keep the external lifecycle candidate withdrawn until the missing capability and clean-state conditions are proven source-free.",
      "production_admitted" => false,
      "real_analyzer_authorized" => false,
      "privileged_installation_authorized" => false,
      "persistent_service_authorized" => false,
    }
  end
end
