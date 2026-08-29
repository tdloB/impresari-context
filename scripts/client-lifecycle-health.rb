#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "date"
require "digest"
require "json"
require "optparse"
require "pathname"

AUTHORITY = {
  "source_read" => "denied",
  "source_write" => "denied",
  "client_mutation" => "denied",
  "process_execution" => "denied",
  "network" => "denied",
  "background_monitoring" => "denied",
}.freeze

class ContractError < StandardError; end

def load_manifest(path)
  document = JSON.parse(File.binread(path))
  required = %w[schema_name schema_version manifest_id client surface supported_versions supported_os supported_arch artifact evidence lifecycle safe_next_steps]
  allowed = required + %w[delivery_contract]
  raise ContractError, "manifest has an unsupported shape" unless document.is_a?(Hash) && (required - document.keys).empty?
  raise ContractError, "manifest schema is unsupported" unless document["schema_name"] == "client-lifecycle-compatibility-manifest" && document["schema_version"] == "1.0.0"
  raise ContractError, "manifest has unknown fields" unless (document.keys - allowed).empty?
  raise ContractError, "manifest lifecycle grants forbidden authority" unless document["lifecycle"] == {
    "health_check" => "explicit_read_only",
    "automatic_repair" => "denied",
    "background_monitoring" => "denied",
    "exact_removal" => "explicit_owned_artifact_only",
  }
  artifact = document["artifact"]
  evidence = document["evidence"]
  raise ContractError, "manifest artifact is malformed" unless artifact.is_a?(Hash) && %w[kind owned_relative_path ownership_marker sha256].all? { |key| artifact[key].is_a?(String) }
  raise ContractError, "manifest evidence is malformed" unless evidence.is_a?(Hash) && %w[record sha256 observed_at fresh_through].all? { |key| evidence[key].is_a?(String) }
  raise ContractError, "manifest identity is malformed" unless [artifact["sha256"], evidence["sha256"]].all? { |value| value.match?(/\A[0-9a-f]{64}\z/) }
  if document.key?("delivery_contract")
    delivery = document.fetch("delivery_contract")
    raise ContractError, "manifest delivery contract is malformed" unless delivery == {
      "level" => "L3",
      "mode" => "ask",
      "protocol_scope" => "code_chat_ask_prompt_stdin_context_v1",
      "source_workspace_exposed" => false,
      "provider_delivery_inferred" => false,
      "operator_confirmation_required" => true,
      "authority_added" => false,
    }
  end
  document
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "manifest unavailable: #{error.class}"
rescue JSON::ParserError
  raise ContractError, "manifest is not valid JSON"
end

def safe_date(value, description)
  Date.iso8601(value)
rescue Date::Error
  raise ContractError, "#{description} must be an ISO date"
end

def receipt(manifest:, manifest_bytes:, options:, status:, reason:, checks:)
  {
    "schema_name" => "client-lifecycle-health-receipt",
    "schema_version" => "1.0.0",
    "status" => status,
    "reason_code" => reason,
    "manifest_id" => manifest.fetch("manifest_id"),
    "client" => manifest.fetch("client"),
    "surface" => manifest.fetch("surface"),
    "observed" => {
      "client_version" => options.fetch(:client_version),
      "client_available" => options.fetch(:client_available),
      "os" => options.fetch(:os),
      "arch" => options.fetch(:arch),
      "as_of" => options.fetch(:as_of),
      "target_supplied" => true,
    },
    "manifest_identity" => Digest::SHA256.hexdigest(manifest_bytes),
    "evidence_identity" => manifest.fetch("evidence").fetch("sha256"),
    "checks" => checks,
    "safe_next_step" => manifest.fetch("safe_next_steps").fetch(status),
    "authority" => AUTHORITY,
  }
end

def assess(manifest_path, target_path, options)
  manifest_bytes = File.binread(manifest_path)
  manifest = load_manifest(manifest_path)
  checks = ["manifest_valid"]

  unless options.fetch(:client_available)
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "degraded", reason: "client_unavailable", checks: checks + ["client_unavailable"])
  end

  unless manifest.fetch("supported_versions").include?(options.fetch(:client_version))
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "unknown", reason: "client_version_unrecorded", checks: checks + ["client_version_unrecorded"])
  end
  unless manifest.fetch("supported_os").include?(options.fetch(:os)) && manifest.fetch("supported_arch").include?(options.fetch(:arch))
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "unsupported", reason: "platform_unrecorded", checks: checks + ["platform_unrecorded"])
  end
  checks << "scope_supported"

  as_of = safe_date(options.fetch(:as_of), "as-of")
  fresh_through = safe_date(manifest.fetch("evidence").fetch("fresh_through"), "fresh-through")
  if as_of > fresh_through
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "stale_evidence", reason: "evidence_window_expired", checks: checks + ["evidence_stale"])
  end
  checks << "evidence_current"
  checks << "delivery_contract_bound" if manifest.key?("delivery_contract")

  target = Pathname.new(target_path)
  unless target.absolute?
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "degraded", reason: "target_not_absolute", checks: checks + ["target_rejected"])
  end
  unless File.file?(target) && !File.symlink?(target)
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "degraded", reason: "owned_target_missing", checks: checks + ["owned_target_missing"])
  end
  bytes = File.binread(target)
  artifact = manifest.fetch("artifact")
  unless bytes.include?(artifact.fetch("ownership_marker"))
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "degraded", reason: "target_not_owned", checks: checks + ["ownership_marker_mismatch"])
  end
  unless Digest::SHA256.hexdigest(bytes) == artifact.fetch("sha256")
    return receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "degraded", reason: "owned_target_changed", checks: checks + ["artifact_identity_mismatch"])
  end

  receipt(manifest: manifest, manifest_bytes: manifest_bytes, options: options, status: "compatible", reason: "contract_matches", checks: checks + ["owned_artifact_matches"])
rescue Errno::ENOENT, Errno::EACCES => error
  raise ContractError, "input unavailable: #{error.class}"
end

options = {}
parser = OptionParser.new do |arguments|
  arguments.banner = "Usage: ruby scripts/client-lifecycle-health.rb --manifest FILE --target FILE --client-version VERSION --client-available yes|no --os OS --arch ARCH --as-of YYYY-MM-DD"
  arguments.on("--manifest FILE") { |value| options[:manifest] = value }
  arguments.on("--target FILE") { |value| options[:target] = value }
  arguments.on("--client-version VERSION") { |value| options[:client_version] = value }
  arguments.on("--client-available VALUE", %w[yes no]) { |value| options[:client_available] = value == "yes" }
  arguments.on("--os OS") { |value| options[:os] = value }
  arguments.on("--arch ARCH") { |value| options[:arch] = value }
  arguments.on("--as-of DATE") { |value| options[:as_of] = value }
end

begin
  parser.parse!
  required = %i[manifest target client_version client_available os arch as_of]
  missing = required.reject { |key| options.key?(key) && options[key] != "" }
  raise ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  puts JSON.pretty_generate(assess(options.fetch(:manifest), options.fetch(:target), options))
rescue ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
