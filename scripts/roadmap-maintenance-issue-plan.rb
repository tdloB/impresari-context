#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "optparse"

LABEL = "impresari-maintenance"
ACTIVE = %w[new_version stale changed unavailable invalid].freeze

class ContractError < StandardError; end

def load_json(path, description)
  JSON.parse(File.binread(path))
rescue Errno::ENOENT, Errno::EACCES
  raise ContractError, "#{description} unavailable"
rescue JSON::ParserError
  raise ContractError, "#{description} is not valid JSON"
end

def marker(component, condition)
  "<!-- impresari-maintenance:#{component}:#{condition} -->"
end

def validate_receipt!(receipt)
  component = receipt.fetch("component_id")
  status = receipt.fetch("status")
  raise ContractError, "receipt component is invalid" unless component.match?(/\A[a-z0-9]+(?:-[a-z0-9]+)*\z/)
  raise ContractError, "receipt status is unsupported" unless (ACTIVE + ["current"]).include?(status)
  raise ContractError, "receipt reason is invalid" unless receipt.fetch("reason_code").match?(/\A[a-z0-9]+(?:_[a-z0-9]+)*\z/)
  raise ContractError, "receipt date is invalid" unless receipt.fetch("checked_at").match?(/\A[0-9]{4}-[0-9]{2}-[0-9]{2}\z/)
  versions = receipt.fetch("admitted_versions")
  raise ContractError, "admitted versions are invalid" unless versions.is_a?(Array) && !versions.empty? && versions.length <= 20 && versions.all? { |version| version.is_a?(String) && version.match?(/\A[0-9A-Za-z.+_-]{1,100}\z/) }
  raise ContractError, "observed version is invalid" unless receipt.fetch("observed_version").match?(/\A[0-9A-Za-z.+_-]{1,100}\z/)
  raise ContractError, "claim state is invalid" unless [true, false].include?(receipt.fetch("claim_retained"))
  %w[manifest_identity observation_identity].each do |field|
    raise ContractError, "receipt identity is invalid" unless receipt.fetch(field).match?(/\A[0-9a-f]{64}\z/)
  end
end

def title(receipt)
  "[maintenance] #{receipt.fetch('component_id')}: #{receipt.fetch('status')}"
end

def body(receipt)
  <<~BODY.chomp
    #{marker(receipt.fetch("component_id"), receipt.fetch("status"))}
    A scheduled, source-free maintenance check reported an actionable condition.

    - Component: `#{receipt.fetch("component_id")}`
    - Condition: `#{receipt.fetch("status")}`
    - Reason: `#{receipt.fetch("reason_code")}`
    - Checked: `#{receipt.fetch("checked_at")}`
    - Observed version: `#{receipt.fetch("observed_version")}`
    - Admitted versions: `#{receipt.fetch("admitted_versions").join(", ")}`
    - Existing exact-version claim retained: `#{receipt.fetch("claim_retained")}`
    - Manifest SHA-256: `#{receipt.fetch("manifest_identity")}`
    - Observation SHA-256: `#{receipt.fetch("observation_identity")}`

    This issue is automation-owned. It does not admit a version, modify a manifest,
    accept risk, merge code, or publish a release.
  BODY
end

options = {}
OptionParser.new do |arguments|
  arguments.on("--receipts FILE") { |value| options[:receipts] = value }
  arguments.on("--existing FILE") { |value| options[:existing] = value }
end.parse!

begin
  raise ContractError, "unexpected arguments" unless ARGV.empty?
  %i[receipts existing].each { |key| raise ContractError, "missing #{key}" unless options[key] }
  receipts = load_json(options.fetch(:receipts), "receipt set").fetch("receipts")
  existing = load_json(options.fetch(:existing), "existing issue set")
  raise ContractError, "existing issue set must be an array" unless existing.is_a?(Array)
  owned = existing.select do |issue|
    labels = issue.fetch("labels", []).map { |label| label.is_a?(Hash) ? label["name"] : label }
    labels.include?(LABEL) && issue.fetch("body", "").match?(%r{<!-- impresari-maintenance:[a-z0-9-]+:[a-z_]+ -->})
  end
  actions = []

  receipts.each do |receipt|
    validate_receipt!(receipt)
    component = receipt.fetch("component_id")
    status = receipt.fetch("status")
    component_issues = owned.select { |issue| issue.fetch("body").include?("<!-- impresari-maintenance:#{component}:") && issue.fetch("state") == "OPEN" }
    desired = ACTIVE.include?(status) ? marker(component, status) : nil
    desired_matches = desired ? component_issues.select { |issue| issue.fetch("body").include?(desired) }.sort_by { |issue| issue.fetch("number") } : []
    canonical = desired_matches.first
    component_issues.each do |issue|
      next if canonical && issue.fetch("number") == canonical.fetch("number")
      actions << {"action" => "close", "issue_number" => issue.fetch("number"), "ownership_key" => issue.fetch("body")[/<!-- impresari-maintenance:([^>]+) -->/, 1], "title" => "", "body" => ""}
    end
    next unless desired
    match = canonical
    if match
      generated = body(receipt)
      actions << {"action" => match.fetch("title") == title(receipt) && match.fetch("body") == generated ? "noop" : "update", "issue_number" => match.fetch("number"), "ownership_key" => "#{component}:#{status}", "title" => title(receipt), "body" => generated}
    else
      actions << {"action" => "create", "issue_number" => 0, "ownership_key" => "#{component}:#{status}", "title" => title(receipt), "body" => body(receipt)}
    end
  end

  puts JSON.pretty_generate({"schema_name" => "roadmap-maintenance-issue-plan", "schema_version" => "1.0.0", "label" => LABEL, "actions" => actions})
rescue ContractError, KeyError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
