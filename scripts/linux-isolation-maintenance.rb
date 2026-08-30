#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "date"
require "digest"
require "json"
require "optparse"

AUTHORITY = {
  "source_read" => "denied",
  "source_write" => "denied",
  "host_discovery" => "denied",
  "process_execution" => "denied",
  "network" => "denied",
  "credential_access" => "denied",
  "privilege_use" => "denied",
  "service_mutation" => "denied",
  "background_monitoring" => "denied",
  "analyzer_execution" => "denied",
}.freeze

WITHDRAW_ON = %w[
  evidence_expired evidence_missing target_unavailable runner_image_changed
  kernel_changed architecture_changed landlock_abi_changed bound_artifact_changed
].freeze

class ContractError < StandardError; end

def exact_keys!(value, required, description)
  unless value.is_a?(Hash) && value.keys.sort == required.sort
    raise ContractError, "#{description} has an unsupported shape"
  end
end

def sha256?(value)
  value.is_a?(String) && value.match?(/\A[0-9a-f]{64}\z/)
end

def matches?(value, pattern)
  value.is_a?(String) && value.match?(pattern)
end

def iso_date(value, description)
  Date.iso8601(value)
rescue Date::Error
  raise ContractError, "#{description} must be an ISO date"
end

def load_manifest(bytes)
  document = JSON.parse(bytes)
  exact_keys!(document, %w[schema_name schema_version manifest_id backend claim bindings targets maintenance safe_next_steps], "manifest")
  raise ContractError, "manifest schema is unsupported" unless document["schema_name"] == "linux-isolation-candidate-manifest" && document["schema_version"] == "1.0.0"
  raise ContractError, "manifest backend is unsupported" unless document["backend"] == "landlock-seccomp-cgroup-v2"
  raise ContractError, "manifest identity is malformed" unless matches?(document["manifest_id"], /\A[a-z0-9]+(?:-[a-z0-9]+)*\z/)
  exact_keys!(document["claim"], %w[level candidate_only broad_linux_support production_admitted real_analyzer_authorized], "manifest claim")
  raise ContractError, "manifest claim overreaches" unless document["claim"] == {
    "level" => "IAR-1B", "candidate_only" => true, "broad_linux_support" => false,
    "production_admitted" => false, "real_analyzer_authorized" => false,
  }
  exact_keys!(document["bindings"], %w[profile_id profile_sha256 probe_path probe_sha256 composite_check_path composite_check_sha256], "manifest bindings")
  bindings = document["bindings"]
  raise ContractError, "manifest bindings are malformed" unless bindings["profile_id"] == "iar-linux-synthetic-v1" &&
    bindings["probe_path"] == "platform/linux-isolation-feasibility/probe.c" &&
    bindings["composite_check_path"] == "scripts/check-linux-composite-feasibility.sh" &&
    %w[profile_sha256 probe_sha256 composite_check_sha256].all? { |key| sha256?(bindings[key]) }
  exact_keys!(document["maintenance"], %w[health_check host_discovery automatic_repair background_monitoring privilege_use service_mutation analyzer_execution production_admission withdraw_on], "manifest maintenance")
  maintenance = document["maintenance"]
  raise ContractError, "manifest maintenance grants forbidden authority" unless maintenance.reject { |key, _| key == "withdraw_on" } == {
    "health_check" => "explicit_source_free", "host_discovery" => "denied",
    "automatic_repair" => "denied", "background_monitoring" => "denied",
    "privilege_use" => "denied", "service_mutation" => "denied",
    "analyzer_execution" => "denied", "production_admission" => "denied",
  } && maintenance["withdraw_on"].sort == WITHDRAW_ON.sort
  safe_steps = document["safe_next_steps"]
  statuses = %w[compatible_candidate stale_evidence changed missing_evidence unsupported unavailable]
  exact_keys!(safe_steps, statuses, "manifest safe steps")
  raise ContractError, "manifest safe step is malformed" unless safe_steps.values.all? { |value| value.is_a?(String) && !value.empty? && value.bytesize <= 300 }
  targets = document["targets"]
  raise ContractError, "manifest targets are malformed" unless targets.is_a?(Array) && targets.length.between?(1, 20)
  ids = targets.map do |target|
    exact_keys!(target, %w[target_id classification runner_label runner_image_version os_release kernel_release architecture landlock_abi evidence], "manifest target")
    valid_target = matches?(target["target_id"], /\A[a-z0-9]+(?:-[a-z0-9]+)*\z/) &&
      matches?(target["runner_label"], /\Aubuntu-[0-9]{2}\.[0-9]{2}(?:-arm)?\z/) &&
      matches?(target["runner_image_version"], /\A[0-9]{8}\.[0-9]+\.[0-9]+\z/) &&
      matches?(target["os_release"], /\A[0-9]{2}\.[0-9]{2}\z/) &&
      matches?(target["kernel_release"], /\A[A-Za-z0-9._+-]+\z/) &&
      %w[x86_64 aarch64].include?(target["architecture"]) &&
      matches?(target["landlock_abi"], /\A[1-9][0-9]*\z/)
    raise ContractError, "manifest target identity is malformed" unless valid_target
    raise ContractError, "manifest target classification is unsupported" unless %w[candidate_scope kernel_diversity_only].include?(target["classification"])
    evidence = target["evidence"]
    exact_keys!(evidence, %w[pull_request job_id observed_at fresh_through receipt_fixture receipt_sha256], "manifest target evidence")
    valid_evidence = matches?(evidence["pull_request"], /\A[1-9][0-9]*\z/) &&
      matches?(evidence["job_id"], /\A[1-9][0-9]*\z/) &&
      matches?(evidence["receipt_fixture"], /\Atests\/conformance\/v1\/valid\/linux-isolation-feasibility-.+\.json\z/) &&
      sha256?(evidence["receipt_sha256"])
    raise ContractError, "manifest evidence identity is malformed" unless valid_evidence
    raise ContractError, "manifest evidence dates are inverted" if iso_date(evidence["observed_at"], "observed-at") > iso_date(evidence["fresh_through"], "fresh-through")
    target["target_id"]
  end
  raise ContractError, "manifest target identifiers are not unique" unless ids.uniq == ids
  document
rescue JSON::ParserError
  raise ContractError, "manifest is not valid JSON"
end

def result(manifest, manifest_bytes, target, options, status, reason, checks)
  {
    "schema_name" => "linux-isolation-candidate-health-receipt",
    "schema_version" => "1.0.0",
    "status" => status,
    "reason_code" => reason,
    "manifest_id" => manifest.fetch("manifest_id"),
    "target_id" => options.fetch(:target_id),
    "observed" => {
      "target_available" => options.fetch(:target_available),
      "evidence_available" => options.fetch(:evidence_available),
      "runner_label" => options.fetch(:runner_label),
      "runner_image_version" => options.fetch(:runner_image_version),
      "os_release" => options.fetch(:os_release),
      "kernel_release" => options.fetch(:kernel_release),
      "architecture" => options.fetch(:architecture),
      "landlock_abi" => options.fetch(:landlock_abi),
      "as_of" => options.fetch(:as_of),
    },
    "manifest_identity" => Digest::SHA256.hexdigest(manifest_bytes),
    "evidence_identity" => target ? target.fetch("evidence").fetch("receipt_sha256") : "0" * 64,
    "checks" => checks,
    "safe_next_step" => manifest.fetch("safe_next_steps").fetch(status),
    "candidate_claim_active" => status == "compatible_candidate",
    "production_admitted" => false,
    "real_analyzer_authorized" => false,
    "authority" => AUTHORITY,
  }
end

def assess(manifest_path, options)
  manifest_bytes = File.binread(manifest_path)
  manifest = load_manifest(manifest_bytes)
  checks = ["manifest_valid"]
  target = manifest.fetch("targets").find { |entry| entry["target_id"] == options.fetch(:target_id) }
  return result(manifest, manifest_bytes, nil, options, "unsupported", "target_unrecorded", checks + ["target_unrecorded"]) unless target
  return result(manifest, manifest_bytes, target, options, "unsupported", "diversity_only_not_candidate", checks + ["target_diversity_only"]) unless target["classification"] == "candidate_scope"
  checks << "target_candidate_scoped"
  return result(manifest, manifest_bytes, target, options, "unavailable", "target_unavailable", checks + ["target_unavailable"]) unless options.fetch(:target_available)
  checks << "target_available"
  return result(manifest, manifest_bytes, target, options, "missing_evidence", "evidence_missing", checks + ["evidence_missing"]) unless options.fetch(:evidence_available)
  checks << "evidence_available"
  as_of = iso_date(options.fetch(:as_of), "as-of")
  fresh_through = iso_date(target.dig("evidence", "fresh_through"), "fresh-through")
  return result(manifest, manifest_bytes, target, options, "stale_evidence", "evidence_expired", checks + ["evidence_stale"]) if as_of > fresh_through
  checks << "evidence_current"
  identity = {
    runner_label: "runner_label", runner_image_version: "runner_image_version",
    os_release: "os_release", kernel_release: "kernel_release",
    architecture: "architecture", landlock_abi: "landlock_abi",
  }
  changed = identity.each_with_object([]) do |(option_key, target_key), fields|
    fields << target_key unless options.fetch(option_key) == target.fetch(target_key)
  end
  unless changed.empty?
    return result(manifest, manifest_bytes, target, options, "changed", "target_identity_changed", checks + changed.map { |field| "#{field}_changed" })
  end
  result(manifest, manifest_bytes, target, options, "compatible_candidate", "exact_candidate_current", checks + ["exact_identity_matches"])
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "manifest unavailable: #{error.class}"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.banner = "Usage: ruby scripts/linux-isolation-maintenance.rb --manifest FILE --target-id ID --target-available yes|no --evidence-available yes|no --runner-label LABEL --runner-image-version VERSION --os-release VERSION --kernel-release VERSION --arch ARCH --landlock-abi ABI --as-of YYYY-MM-DD"
  arguments.on("--manifest FILE") { |value| options[:manifest] = value }
  arguments.on("--target-id ID") { |value| options[:target_id] = value }
  arguments.on("--target-available VALUE", %w[yes no]) { |value| options[:target_available] = value == "yes" }
  arguments.on("--evidence-available VALUE", %w[yes no]) { |value| options[:evidence_available] = value == "yes" }
  arguments.on("--runner-label LABEL") { |value| options[:runner_label] = value }
  arguments.on("--runner-image-version VERSION") { |value| options[:runner_image_version] = value }
  arguments.on("--os-release VERSION") { |value| options[:os_release] = value }
  arguments.on("--kernel-release VERSION") { |value| options[:kernel_release] = value }
  arguments.on("--arch ARCH") { |value| options[:architecture] = value }
  arguments.on("--landlock-abi ABI") { |value| options[:landlock_abi] = value }
  arguments.on("--as-of DATE") { |value| options[:as_of] = value }
end

begin
  parser.parse!
  required = %i[manifest target_id target_available evidence_available runner_label runner_image_version os_release kernel_release architecture landlock_abi as_of]
  missing = required.reject { |key| options.key?(key) && options[key] != "" }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  puts JSON.pretty_generate(assess(options.fetch(:manifest), options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
