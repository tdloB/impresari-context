#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "optparse"

PHASES = %w[clean_install upgrade rollback operator_relaunch cancellation crash_recovery health_withdrawal uninstall].freeze
PACKAGE_SCOPE = %w[impresari-context impresari-context-mcp impresari-context-structural-worker].freeze
PACKAGE_PHASES = [
  {"phase" => "clean_install", "outcome" => "passed"},
  {"phase" => "upgrade", "outcome" => "passed"},
  {"phase" => "rollback", "outcome" => "passed"},
  {"phase" => "operator_relaunch", "outcome" => "passed"},
  {"phase" => "uninstall", "outcome" => "passed"},
].freeze
PACKAGE_CLEAN_KEYS = %w[installed_service_unit_absent authorization_policy_absent unexpected_package_files_absent staged_source_absent].freeze
PACKAGE_CLAIM_KEYS = %w[full_lifecycle_admitted production_admitted real_analyzer_authorized privileged_installation_authorized persistent_service_authorized].freeze
COMPOSITE_PREFLIGHT_KEYS = %w[no_new_privs landlock seccomp_filter seccomp_kill_process architecture_filter cgroup_v2 cpu_controller memory_controller pids_controller delegated_leaf cgroup_kill cgroup_empty_verification].freeze
COMPOSITE_CHECK_KEYS = %w[atomic_cgroup_placement resource_profile_applied no_new_privs_effective landlock_read_only_input external_filesystem_denial credential_denial device_denial network_denial unrelated_descriptors_closed descendant_denial zero_writable_filesystem cpu_limit memory_limit process_count_limit exact_cgroup_kill cgroup_empty_after_job bounded_output timeout crash_relaunch cleanup cross_job_isolation].freeze
WITHDRAWAL_CLEAN_KEYS = %w[persistent_service_absent privileged_policy_absent stale_cgroup_absent descendants_absent staged_source_absent].freeze
WITHDRAWAL_CLAIM_KEYS = %w[production_admitted real_analyzer_authorized privileged_installation_authorized persistent_service_authorized].freeze
EXTERNAL_AUTHORITY = {
  "workspace_source_read" => "denied",
  "source_write" => "synthetic_target_only",
  "process_execution" => "fixed_synthetic_composite_only",
  "cgroup_mutation" => "inherited_delegated_descendants_only",
  "service_mutation_by_impresari" => "denied",
  "operator_provisioning" => "one_ephemeral_ci_service_only",
  "network" => "denied",
  "credential_access" => "denied",
  "impresari_privilege_use" => "denied",
  "persistent_service" => "denied",
  "analyzer_execution" => "denied",
}.freeze
SAFE_NEXT_STEPS = {
  "lifecycle_candidate" => "Retain this exact-host C lifecycle candidate and proceed to expiring production-support admission; production and analyzers remain closed.",
  "identity_mismatch" => "Reject the composition and reproduce every input in one exact-source hosted run with linked receipt identities.",
  "package_failed" => "Keep C withdrawn and rerun the exact package lifecycle without changing the accepted package or authority boundary.",
  "external_failed" => "Keep C withdrawn and reproduce the externally managed topology candidate and cleanup evidence.",
  "interruption_failed" => "Keep C withdrawn until exact cancellation, empty-cgroup, timeout, crash-relaunch, and cleanup evidence all pass.",
  "withdrawal_failed" => "Keep C withdrawn until absence of the external capability deterministically withdraws the claim and leaves clean state.",
}.freeze

def identity(bytes)
  Digest::SHA256.hexdigest(bytes)
end

def load_document(path, name)
  abort("#{name} exceeds 131072 bytes") if File.size(path) > 131_072
  bytes = File.binread(path)
  [JSON.parse(bytes), bytes]
rescue JSON::ParserError
  abort("#{name} is not valid JSON")
rescue Errno::ENOENT, Errno::EACCES => error
  abort("#{name} unavailable: #{error.class}")
end

def false_claims?(document, keys)
  document.is_a?(Hash) && document.keys.sort == keys.sort && keys.all? { |key| document[key] == false }
end

def all_false_fields?(document, keys)
  document.is_a?(Hash) && keys.all? { |key| document[key] == false }
end

def exact_true_fields?(document, keys)
  document.is_a?(Hash) && document.keys.sort == keys.sort && document.values.all? { |value| value == true }
end

def sha256?(value)
  value.is_a?(String) && value.match?(/\A[0-9a-f]{64}\z/)
end

def source_commit?(value)
  value.is_a?(String) && value.match?(/\A[0-9a-f]{40}\z/)
end

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby scripts/linux-external-lifecycle-compose.rb --expected-source-sha SHA --package-receipt FILE --external-receipt FILE --composite-receipt FILE --withdrawal-receipt FILE"
  parser.on("--expected-source-sha SHA") { |value| options[:source] = value }
  parser.on("--package-receipt FILE") { |value| options[:package] = value }
  parser.on("--external-receipt FILE") { |value| options[:external] = value }
  parser.on("--composite-receipt FILE") { |value| options[:composite] = value }
  parser.on("--withdrawal-receipt FILE") { |value| options[:withdrawal] = value }
end.parse!

abort("unexpected arguments") unless ARGV.empty?
missing = %i[source package external composite withdrawal].reject { |key| options[key] && !options[key].empty? }
abort("missing required arguments: #{missing.join(', ')}") unless missing.empty?
expected_source = options.fetch(:source)
abort("invalid expected source SHA") unless expected_source.match?(/\A[0-9a-f]{40}\z/)

package, package_bytes = load_document(options.fetch(:package), "package receipt")
external, external_bytes = load_document(options.fetch(:external), "external receipt")
composite, composite_bytes = load_document(options.fetch(:composite), "composite receipt")
withdrawal, withdrawal_bytes = load_document(options.fetch(:withdrawal), "withdrawal receipt")

package_identity = identity(package_bytes)
external_identity = identity(external_bytes)
composite_identity = identity(composite_bytes)
withdrawal_identity = identity(withdrawal_bytes)

scope_verified = package["package_scope"] == PACKAGE_SCOPE
package_clean = exact_true_fields?(package["clean_state"], PACKAGE_CLEAN_KEYS)
package_candidate = package["candidate"].is_a?(Hash) &&
  package["candidate"].keys.sort == %w[archive_sha256 manifest_sha256 project_version source_commit].sort &&
  sha256?(package.dig("candidate", "archive_sha256")) && sha256?(package.dig("candidate", "manifest_sha256")) &&
  package.dig("candidate", "project_version") == "0.2.0" && source_commit?(package.dig("candidate", "source_commit"))
package_ok = package["schema_name"] == "linux-isolation-package-lifecycle-rehearsal" &&
  package["schema_version"] == "1.0.0" && package["policy_id"] == "linux-iar-1b-production-lifecycle-v1" &&
  package["profile"] == "externally_managed" && package["status"] == "package_lifecycle_candidate" &&
  package["phases"] == PACKAGE_PHASES && package["reentry_evidence"] == "operator_relaunch_verified" &&
  package["excluded_lifecycle_evidence"] == %w[cancellation crash_recovery health_withdrawal topology_revalidation] &&
  package_candidate && package.dig("baseline", "archive_sha256") != package.dig("candidate", "archive_sha256") &&
  scope_verified && package_clean && false_claims?(package["claims"], PACKAGE_CLAIM_KEYS)

external_host = external["observed_host"]
external_host_valid = external_host.is_a?(Hash) && external_host.keys.sort == %w[operating_system kernel_release architecture].sort &&
  external_host["operating_system"] == "linux" && %w[x86_64 aarch64].include?(external_host["architecture"]) &&
  external_host["kernel_release"].is_a?(String) && external_host["kernel_release"].match?(/\A[A-Za-z0-9._+-]{1,128}\z/)
external_ok = external["schema_name"] == "linux-external-delegation-live-rehearsal" &&
  external["schema_version"] == "1.0.0" && external["profile"] == "externally_managed" &&
  external_host_valid && external["policy_identity"] == "03ff04052dae6f7990805011fe454774c3f2ba209a9cf0eea083097eacb7bac4" &&
  external["status"] == "candidate_passed" && external["external_candidate_active"] == true &&
  external["os_confined"] == true && external.dig("provisioner", "service_collected") == true &&
  external.dig("provisioner", "operator_privilege_used") == true &&
  external.dig("provisioner", "impresari_privilege_used") == false &&
  external.dig("provisioner", "provider") == "ephemeral_ci_transient_system_service" &&
  external.dig("provisioner", "service_created") == true && external.dig("provisioner", "persistent") == false &&
  external.dig("provisioner", "unit_name_recorded") == false && external.dig("cleanup", "attempted") == true &&
  external.dig("cleanup", "descendants_removed") == true && sha256?(external.dig("composite", "receipt_identity")) &&
  external.dig("composite", "result") == "candidate_passed" && external["authority"] == EXTERNAL_AUTHORITY &&
  all_false_fields?(external, %w[production_admitted real_analyzer_authorized privileged_installation_authorized])

checks = composite.fetch("checks", {})
preflight = composite.fetch("preflight", {})
composite_host = composite["observed_host"]
composite_host_valid = composite_host.is_a?(Hash) && composite_host.keys.sort == %w[operating_system kernel_release architecture landlock_abi].sort &&
  composite_host["operating_system"] == "linux" && %w[x86_64 aarch64].include?(composite_host["architecture"]) &&
  composite_host["kernel_release"].is_a?(String) && composite_host["kernel_release"].match?(/\A[A-Za-z0-9._+-]{1,96}\z/)
interruption_ok = composite["schema_name"] == "linux-isolation-feasibility" &&
  composite["schema_version"] == "1.0.0" && composite["result"] == "candidate_passed" &&
  composite_host_valid && exact_true_fields?(preflight, COMPOSITE_PREFLIGHT_KEYS) &&
  exact_true_fields?(checks, COMPOSITE_CHECK_KEYS) && composite["limitations"] == %w[single-host-evidence synthetic-probe-no-analysis] &&
  composite["os_confined"] == true && composite["production_admitted"] == false &&
  composite["source_retained"] == false && composite["authority_added"] == false

withdrawal_clean = exact_true_fields?(withdrawal["clean_state"], WITHDRAWAL_CLEAN_KEYS)
withdrawal_ok = withdrawal["schema_name"] == "linux-external-health-withdrawal" &&
  withdrawal["schema_version"] == "1.0.0" && withdrawal["profile"] == "externally_managed" &&
  withdrawal["changed_prerequisite"] == "inherited_delegation_capability_unavailable" && withdrawal["capability_descriptor"] == 3 &&
  withdrawal["status"] == "withdrawn" && withdrawal["capability_available"] == false &&
  withdrawal["topology_revalidated"] == false && withdrawal["claim_withdrawn"] == true && withdrawal_clean &&
  all_false_fields?(withdrawal, WITHDRAWAL_CLAIM_KEYS)

host_match = external_host_valid && composite_host_valid &&
  external.dig("observed_host", "operating_system") == composite.dig("observed_host", "operating_system") &&
  external.dig("observed_host", "kernel_release") == composite.dig("observed_host", "kernel_release") &&
  external.dig("observed_host", "architecture") == composite.dig("observed_host", "architecture")
identity_ok = package.dig("candidate", "source_commit") == expected_source &&
  external.dig("composite", "receipt_identity") == composite_identity &&
  withdrawal["package_receipt_identity"] == package_identity &&
  withdrawal["external_receipt_identity"] == external_identity && host_match

status = if !identity_ok
  "identity_mismatch"
elsif !package_ok
  "package_failed"
elsif !external_ok
  "external_failed"
elsif !interruption_ok
  "interruption_failed"
elsif !withdrawal_ok
  "withdrawal_failed"
else
  "lifecycle_candidate"
end

receipt = {
  "schema_name" => "linux-external-lifecycle-composition",
  "schema_version" => "1.0.0",
  "policy_id" => "linux-iar-1b-production-lifecycle-v1",
  "profile" => "externally_managed",
  "expected_source_commit" => expected_source,
  "observed_host" => external.fetch("observed_host"),
  "package" => {
    "receipt_identity" => package_identity,
    "archive_identity" => package.dig("candidate", "archive_sha256"),
    "manifest_identity" => package.dig("candidate", "manifest_sha256"),
    "source_commit" => package.dig("candidate", "source_commit"),
    "status" => package["status"],
    "scope_verified" => scope_verified,
    "clean_state_verified" => package_clean,
  },
  "external" => {
    "receipt_identity" => external_identity,
    "composite_identity" => external.dig("composite", "receipt_identity"),
    "status" => external["status"],
    "os_confined" => external["os_confined"],
    "provisioner_collected" => external.dig("provisioner", "service_collected"),
    "operator_privilege_used" => external.dig("provisioner", "operator_privilege_used"),
    "impresari_privilege_used" => external.dig("provisioner", "impresari_privilege_used"),
    "persistent" => external.dig("provisioner", "persistent"),
  },
  "interruption" => {
    "receipt_identity" => composite_identity,
    "result" => composite["result"],
    "exact_cgroup_kill" => checks["exact_cgroup_kill"],
    "cgroup_empty_after_job" => checks["cgroup_empty_after_job"],
    "timeout" => checks["timeout"],
    "crash_relaunch" => checks["crash_relaunch"],
    "cleanup" => checks["cleanup"],
  },
  "withdrawal" => {
    "receipt_identity" => withdrawal_identity,
    "prior_external_receipt_identity" => withdrawal["external_receipt_identity"],
    "status" => withdrawal["status"],
    "capability_available" => withdrawal["capability_available"],
    "topology_revalidated" => withdrawal["topology_revalidated"],
    "claim_withdrawn" => withdrawal["claim_withdrawn"],
    "clean_state_verified" => withdrawal_clean,
  },
  "phases" => PHASES,
  "status" => status,
  "reason_code" => status == "lifecycle_candidate" ? "exact_external_lifecycle_evidence_composed" : status,
  "safe_next_step" => SAFE_NEXT_STEPS.fetch(status),
  "lifecycle_candidate_active" => status == "lifecycle_candidate",
  "health_withdrawal_verified" => withdrawal_ok,
  "production_admitted" => false,
  "real_analyzer_authorized" => false,
  "release_packaging_authorized" => false,
  "privileged_installation_authorized" => false,
  "persistent_service_authorized" => false,
}
puts JSON.pretty_generate(receipt)
exit 7 unless status == "lifecycle_candidate"
