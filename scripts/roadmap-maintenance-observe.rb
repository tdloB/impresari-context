#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "net/http"
require "optparse"
require "pathname"
require "time"

ROOT = Pathname.new(__dir__).join("..").expand_path
MAXIMUM_BYTES = 524_288
TIMEOUT_SECONDS = 10
ALLOWED_HOSTS = ["api.github.com"].freeze
COMPONENT_KEYS = %w[component_id manifest owned_artifact source].freeze
GITHUB_SOURCE_KEYS = %w[adapter host path tag_prefix].freeze
UNAVAILABLE_SOURCE_KEYS = %w[adapter reason_code].freeze

class ContractError < StandardError; end
class ResponseTooLarge < StandardError; end

def closed_hash!(value, required, description)
  raise ContractError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && (required - value.keys).empty? && (value.keys - required).empty?
end

def load_source_set(path)
  source_set = JSON.parse(File.binread(path))
  closed_hash!(source_set, %w[schema_name schema_version components], "source set")
  raise ContractError, "source set schema is unsupported" unless source_set["schema_name"] == "roadmap-maintenance-source-set" && source_set["schema_version"] == "1.0.0"
  components = source_set.fetch("components")
  raise ContractError, "source set components are invalid" unless components.is_a?(Array) && !components.empty? && components.length <= 20
  ids = components.map do |component|
    closed_hash!(component, COMPONENT_KEYS, "component")
    id = component.fetch("component_id")
    raise ContractError, "component identity is invalid" unless id.is_a?(String) && id.match?(/\A[a-z0-9]+(?:-[a-z0-9]+)*\z/)
    source = component.fetch("source")
    adapter = source.is_a?(Hash) ? source["adapter"] : nil
    case adapter
    when "github_latest_release"
      closed_hash!(source, GITHUB_SOURCE_KEYS, "GitHub source")
      raise ContractError, "source host is not allowlisted" unless ALLOWED_HOSTS.include?(source.fetch("host"))
      raise ContractError, "source path is invalid" unless source.fetch("path").match?(%r{\A/repos/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/releases/latest\z})
      raise ContractError, "tag prefix is invalid" unless source.fetch("tag_prefix").is_a?(String) && source.fetch("tag_prefix").length <= 20
    when "unavailable"
      closed_hash!(source, UNAVAILABLE_SOURCE_KEYS, "unavailable source")
      raise ContractError, "unavailable reason is invalid" unless source.fetch("reason_code").match?(/\A[a-z0-9]+(?:_[a-z0-9]+)*\z/)
    else
      raise ContractError, "source adapter is unsupported"
    end
    id
  end
  raise ContractError, "component identities must be unique" unless ids.uniq.length == ids.length
  source_set
rescue JSON::ParserError
  raise ContractError, "source set is not valid JSON"
end

def bounded_live_response(source)
  uri = URI::HTTPS.build(host: source.fetch("host"), path: source.fetch("path"))
  request = Net::HTTP::Get.new(uri)
  request["Accept"] = "application/vnd.github+json"
  request["User-Agent"] = "impresari-context-roadmap-maintenance/1"
  request["X-GitHub-Api-Version"] = "2022-11-28"
  status = 0
  content_type = ""
  redirected = false
  body = +"".b
  Net::HTTP.start(uri.host, uri.port, use_ssl: true, open_timeout: TIMEOUT_SECONDS, read_timeout: TIMEOUT_SECONDS) do |http|
    http.request(request) do |response|
      status = response.code.to_i
      content_type = response["content-type"].to_s
      redirected = response.is_a?(Net::HTTPRedirection)
      response.read_body do |chunk|
        body << chunk.b
        raise ResponseTooLarge if body.bytesize > MAXIMUM_BYTES
      end
    end
  end
  {
    "http_status" => status,
    "content_type" => content_type,
    "redirected" => redirected,
    "body" => body,
    "bytes_received" => body.bytesize,
  }
rescue ResponseTooLarge
  {
    "http_status" => status,
    "content_type" => content_type,
    "redirected" => redirected,
    "body" => nil,
    "bytes_received" => body.bytesize,
  }
rescue StandardError
  {
    "http_status" => 0,
    "content_type" => "",
    "redirected" => false,
    "body" => nil,
    "bytes_received" => 0,
  }
end

def fixture_response(directory, component_id)
  path = Pathname.new(directory).join("#{component_id}.json")
  envelope = JSON.parse(File.binread(path))
  closed_hash!(envelope, %w[http_status content_type redirected body], "fixture envelope")
  body = envelope.fetch("body")
  raise ContractError, "fixture body is invalid" unless body.is_a?(String)
  envelope.merge("bytes_received" => body.b.bytesize)
rescue Errno::ENOENT
  {"http_status" => 0, "content_type" => "", "redirected" => false, "body" => nil, "bytes_received" => 0}
rescue JSON::ParserError
  raise ContractError, "fixture envelope is not valid JSON"
end

def observation(component, response, checked_at)
  source = component.fetch("source")
  base = {
    "schema_name" => "roadmap-maintenance-observation",
    "schema_version" => "1.0.0",
    "component_id" => component.fetch("component_id"),
    "source_id" => source["adapter"] == "github_latest_release" ? "https://#{source.fetch('host')}#{source.fetch('path')}" : "none",
    "checked_at" => checked_at,
    "outcome" => "unavailable",
    "observed_version" => "unavailable",
    "reason_code" => source.fetch("reason_code", "metadata_source_unavailable"),
    "response_identity" => Digest::SHA256.hexdigest(""),
    "bytes_received" => 0,
    "authority" => {
      "source_read" => "denied",
      "source_write" => "denied",
      "client_mutation" => "denied",
      "release_mutation" => "denied",
      "credential_read" => "denied",
    },
  }
  return base if source["adapter"] == "unavailable"

  body = response.fetch("body")
  base["bytes_received"] = response.fetch("bytes_received")
  base["response_identity"] = Digest::SHA256.hexdigest(body.to_s.b)
  return base.merge("reason_code" => "metadata_response_unavailable") if response.fetch("http_status") == 0
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_redirect_rejected") if response.fetch("redirected")
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_response_oversized") if response.fetch("bytes_received") > MAXIMUM_BYTES
  return base.merge("outcome" => "unavailable", "reason_code" => "metadata_http_unavailable") unless response.fetch("http_status") == 200
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_content_type_invalid") unless response.fetch("content_type").downcase.include?("application/json")

  document = JSON.parse(body)
  tag = document.is_a?(Hash) ? document["tag_name"] : nil
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_version_ambiguous") unless tag.is_a?(String) && !tag.empty? && tag.length <= 100
  prefix = source.fetch("tag_prefix")
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_tag_prefix_changed") unless prefix.empty? || tag.start_with?(prefix)
  version = prefix.empty? ? tag : tag.delete_prefix(prefix)
  return base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_version_invalid") unless version.match?(/\A[0-9]+(?:\.[0-9A-Za-z-]+)+\z/)
  base.merge("outcome" => "observed", "observed_version" => version, "reason_code" => "metadata_version_observed")
rescue JSON::ParserError
  base.merge("outcome" => "invalid", "observed_version" => "invalid", "reason_code" => "metadata_json_invalid")
end

options = {source_set: ROOT.join("maintenance/client-sources.json").to_s}
OptionParser.new do |arguments|
  arguments.on("--source-set FILE") { |value| options[:source_set] = value }
  arguments.on("--fixtures DIRECTORY") { |value| options[:fixtures] = value }
  arguments.on("--checked-at TIME") { |value| options[:checked_at] = value }
end.parse!

begin
  raise ContractError, "unexpected arguments" unless ARGV.empty?
  checked_at = options.fetch(:checked_at, Time.now.utc.iso8601)
  raise ContractError, "checked-at must be canonical UTC" unless checked_at.match?(/\A[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z\z/)
  Time.iso8601(checked_at)
  source_set = load_source_set(options.fetch(:source_set))
  observations = source_set.fetch("components").map do |component|
    source = component.fetch("source")
    response = if source["adapter"] == "unavailable"
      {"http_status" => 0, "content_type" => "", "redirected" => false, "body" => nil, "bytes_received" => 0}
    elsif options[:fixtures]
      fixture_response(options.fetch(:fixtures), component.fetch("component_id"))
    else
      bounded_live_response(source)
    end
    observation(component, response, checked_at)
  end
  puts JSON.pretty_generate({"schema_name" => "roadmap-maintenance-observation-set", "schema_version" => "1.0.0", "observations" => observations})
rescue ContractError, OptionParser::ParseError, ArgumentError => error
  warn "error: #{error.message}"
  exit 1
end
