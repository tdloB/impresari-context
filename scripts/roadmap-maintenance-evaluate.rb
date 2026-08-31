#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "date"
require "digest"
require "json"
require "optparse"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
AUTHORITY = {
  "claim_promotion" => "denied",
  "manifest_mutation" => "denied",
  "client_mutation" => "denied",
  "release_mutation" => "denied",
  "risk_acceptance" => "denied",
}.freeze

class ContractError < StandardError; end

OBSERVATION_KEYS = %w[schema_name schema_version component_id source_id checked_at outcome observed_version reason_code response_identity bytes_received authority].freeze
OBSERVATION_AUTHORITY = {
  "source_read" => "denied",
  "source_write" => "denied",
  "client_mutation" => "denied",
  "release_mutation" => "denied",
  "credential_read" => "denied",
}.freeze

def validate_observation!(observation)
  raise ContractError, "observation has an unsupported shape" unless observation.is_a?(Hash) && observation.keys.sort == OBSERVATION_KEYS.sort
  raise ContractError, "observation schema is unsupported" unless observation["schema_name"] == "roadmap-maintenance-observation" && observation["schema_version"] == "1.0.0"
  raise ContractError, "observation component identity is invalid" unless observation.fetch("component_id").match?(/\A[a-z0-9]+(?:-[a-z0-9]+)*\z/)
  raise ContractError, "observation source identity is invalid" unless observation.fetch("source_id").is_a?(String) && !observation.fetch("source_id").empty? && observation.fetch("source_id").length <= 240
  raise ContractError, "observation time is not canonical UTC" unless observation.fetch("checked_at").match?(/\A[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z\z/)
  Time.iso8601(observation.fetch("checked_at"))
  raise ContractError, "observation outcome is unsupported" unless %w[observed unavailable invalid].include?(observation.fetch("outcome"))
  raise ContractError, "observed version is invalid" unless observation.fetch("observed_version").is_a?(String) && observation.fetch("observed_version").match?(/\A[0-9A-Za-z.+_-]{1,100}\z/)
  raise ContractError, "observation reason is invalid" unless observation.fetch("reason_code").match?(/\A[a-z0-9]+(?:_[a-z0-9]+)*\z/)
  raise ContractError, "observation response identity is invalid" unless observation.fetch("response_identity").match?(/\A[0-9a-f]{64}\z/)
  raise ContractError, "observation byte count is invalid" unless observation.fetch("bytes_received").is_a?(Integer) && observation.fetch("bytes_received") >= 0
  raise ContractError, "observation authority expanded" unless observation.fetch("authority") == OBSERVATION_AUTHORITY
end

def confined_path(relative)
  raise ContractError, "maintenance path is invalid" unless relative.is_a?(String) && !relative.empty? && !Pathname.new(relative).absolute?
  path = ROOT.join(relative).cleanpath
  raise ContractError, "maintenance path escapes the repository" unless path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  path
end

def load_json(path, description)
  JSON.parse(File.binread(path))
rescue Errno::ENOENT, Errno::EACCES
  raise ContractError, "#{description} unavailable"
rescue JSON::ParserError
  raise ContractError, "#{description} is not valid JSON"
end

def receipt(component, manifest, observation, status, reason, checked_at, checks, claim_retained)
  {
    "schema_name" => "roadmap-maintenance-receipt",
    "schema_version" => "1.0.0",
    "component_id" => component.fetch("component_id"),
    "manifest_id" => manifest.fetch("manifest_id"),
    "status" => status,
    "reason_code" => reason,
    "checked_at" => checked_at,
    "admitted_versions" => manifest.fetch("supported_versions"),
    "observed_version" => observation.fetch("observed_version"),
    "claim_retained" => claim_retained,
    "manifest_identity" => Digest::SHA256.file(confined_path(component.fetch("manifest"))).hexdigest,
    "observation_identity" => Digest::SHA256.hexdigest(JSON.generate(observation.sort.to_h)),
    "checks" => checks,
    "authority" => AUTHORITY,
  }
end

options = {source_set: ROOT.join("maintenance/client-sources.json").to_s}
OptionParser.new do |arguments|
  arguments.on("--source-set FILE") { |value| options[:source_set] = value }
  arguments.on("--observations FILE") { |value| options[:observations] = value }
  arguments.on("--as-of DATE") { |value| options[:as_of] = value }
end.parse!

begin
  raise ContractError, "unexpected arguments" unless ARGV.empty?
  %i[observations as_of].each { |key| raise ContractError, "missing #{key}" unless options[key] && !options[key].empty? }
  as_of = Date.iso8601(options.fetch(:as_of))
  source_set = load_json(options.fetch(:source_set), "source set")
  observation_set = load_json(options.fetch(:observations), "observation set")
  raise ContractError, "observation set has an unsupported shape" unless observation_set.is_a?(Hash) && observation_set.keys.sort == %w[observations schema_name schema_version].sort
  raise ContractError, "observation set schema is unsupported" unless observation_set["schema_name"] == "roadmap-maintenance-observation-set" && observation_set["schema_version"] == "1.0.0"
  raise ContractError, "observation set size is invalid" unless observation_set.fetch("observations").is_a?(Array) && !observation_set.fetch("observations").empty? && observation_set.fetch("observations").length <= 20
  observation_set.fetch("observations").each { |observation| validate_observation!(observation) }
  observations = observation_set.fetch("observations").to_h { |item| [item.fetch("component_id"), item] }
  raise ContractError, "observation identities are duplicated" unless observations.length == observation_set.fetch("observations").length

  receipts = source_set.fetch("components").map do |component|
    observation = observations.fetch(component.fetch("component_id")) { raise ContractError, "component observation missing" }
    manifest_path = confined_path(component.fetch("manifest"))
    artifact_path = confined_path(component.fetch("owned_artifact"))
    manifest = load_json(manifest_path, "compatibility manifest")
    checks = ["manifest_loaded", "observation_bound"]
    evidence_path = confined_path(manifest.fetch("evidence").fetch("record"))

    artifact_matches = File.file?(artifact_path) && !File.symlink?(artifact_path) && Digest::SHA256.file(artifact_path).hexdigest == manifest.fetch("artifact").fetch("sha256")
    evidence_matches = File.file?(evidence_path) && !File.symlink?(evidence_path) && Digest::SHA256.file(evidence_path).hexdigest == manifest.fetch("evidence").fetch("sha256")
    unless artifact_matches && evidence_matches
      changed = []
      changed << "owned_artifact_changed" unless artifact_matches
      changed << "evidence_record_changed" unless evidence_matches
      next receipt(component, manifest, observation, "changed", "released_evidence_identity_changed", options.fetch(:as_of), checks + changed, false)
    end
    checks += ["owned_artifact_identity_current", "evidence_record_identity_current"]

    fresh_through = Date.iso8601(manifest.fetch("evidence").fetch("fresh_through"))
    if as_of > fresh_through
      next receipt(component, manifest, observation, "stale", "evidence_window_expired", options.fetch(:as_of), checks + ["evidence_stale"], false)
    end
    checks << "evidence_current"

    case observation.fetch("outcome")
    when "unavailable"
      receipt(component, manifest, observation, "unavailable", observation.fetch("reason_code"), options.fetch(:as_of), checks + ["upstream_unavailable"], false)
    when "invalid"
      receipt(component, manifest, observation, "invalid", observation.fetch("reason_code"), options.fetch(:as_of), checks + ["upstream_invalid"], false)
    when "observed"
      if manifest.fetch("supported_versions").include?(observation.fetch("observed_version"))
        receipt(component, manifest, observation, "current", "admitted_version_current", options.fetch(:as_of), checks + ["admitted_version_observed"], true)
      else
        receipt(component, manifest, observation, "new_version", "unadmitted_version_observed", options.fetch(:as_of), checks + ["new_version_not_admitted"], true)
      end
    else
      raise ContractError, "observation outcome is unsupported"
    end
  end
  puts JSON.pretty_generate({"schema_name" => "roadmap-maintenance-receipt-set", "schema_version" => "1.0.0", "as_of" => options.fetch(:as_of), "receipts" => receipts})
rescue ContractError, KeyError, JSON::ParserError, Date::Error, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
