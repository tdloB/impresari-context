#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "fileutils"
require "json"
require "open3"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
OBSERVE = ROOT.join("scripts/roadmap-maintenance-observe.rb")
EVALUATE = ROOT.join("scripts/roadmap-maintenance-evaluate.rb")
PLAN = ROOT.join("scripts/roadmap-maintenance-issue-plan.rb")
APPLY = ROOT.join("scripts/roadmap-maintenance-issue-apply.rb")
SOURCE_SET = ROOT.join("maintenance/client-sources.json")
CURRENT = ROOT.join("maintenance/fixtures/current")

def command_json(*command)
  stdout, stderr, status = Open3.capture3(*command)
  abort("command failed: #{command.join(' ')}\n#{stderr}") unless status.success?
  JSON.parse(stdout)
rescue JSON::ParserError => error
  abort("command returned invalid JSON: #{error.message}")
end

def write_json(path, value)
  File.write(path, JSON.pretty_generate(value) + "\n")
end

def single_source_set(component_id, artifact: nil)
  source_set = JSON.parse(SOURCE_SET.read)
  component = source_set.fetch("components").find { |entry| entry.fetch("component_id") == component_id }
  abort("fixture component missing") unless component
  component = Marshal.load(Marshal.dump(component))
  component["owned_artifact"] = artifact if artifact
  source_set.merge("components" => [component])
end

current_observations = command_json(RbConfig.ruby, OBSERVE.to_s, "--fixtures", CURRENT.to_s, "--checked-at", "2026-08-30T12:00:00Z")
outcomes = current_observations.fetch("observations").to_h { |entry| [entry.fetch("component_id"), entry.fetch("outcome")] }
abort("current metadata fixtures drifted") unless outcomes == {
  "codex-cli" => "observed",
  "claude-code" => "observed",
  "github-copilot-cli" => "observed",
  "vscode-copilot" => "observed",
  "cursor-agent" => "unavailable",
}
abort("observation authority expanded") unless current_observations.fetch("observations").all? { |item| item.fetch("authority").values.all? { |value| value == "denied" } }

Dir.mktmpdir("impresari-roadmap-maintenance-") do |directory|
  root = Pathname.new(directory)
  observations_path = root.join("observations.json")
  write_json(observations_path, current_observations)
  current = command_json(RbConfig.ruby, EVALUATE.to_s, "--observations", observations_path.to_s, "--as-of", "2026-08-30")
  statuses = current.fetch("receipts").to_h { |entry| [entry.fetch("component_id"), entry.fetch("status")] }
  abort("current evaluation drifted") unless statuses == {
    "codex-cli" => "current",
    "claude-code" => "current",
    "github-copilot-cli" => "current",
    "vscode-copilot" => "current",
    "cursor-agent" => "unavailable",
  }
  abort("maintenance evaluator granted authority") unless current.fetch("receipts").all? { |item| item.fetch("authority").values.all? { |value| value == "denied" } }

  stale = command_json(RbConfig.ruby, EVALUATE.to_s, "--observations", observations_path.to_s, "--as-of", "2026-12-01")
  abort("stale evidence preserved a claim") unless stale.fetch("receipts").all? { |receipt| receipt.fetch("status") == "stale" && receipt.fetch("claim_retained") == false }

  component_source = root.join("single-source.json")
  oversized_root = root.join("oversized")
  FileUtils.mkdir_p(oversized_root)
  write_json(oversized_root.join("codex-cli.json"), {"http_status" => 200, "content_type" => "application/json", "redirected" => false, "body" => "x" * 524_289})

  %w[new-version invalid redirect unavailable].each do |fixture_name|
    write_json(component_source, single_source_set("codex-cli"))
    fixture_root = ROOT.join("maintenance/fixtures", fixture_name)
    observations = command_json(RbConfig.ruby, OBSERVE.to_s, "--source-set", component_source.to_s, "--fixtures", fixture_root.to_s, "--checked-at", "2026-08-30T12:00:00Z")
    write_json(observations_path, observations)
    result = command_json(RbConfig.ruby, EVALUATE.to_s, "--source-set", component_source.to_s, "--observations", observations_path.to_s, "--as-of", "2026-08-30")
    expected = {"new-version" => "new_version", "invalid" => "invalid", "redirect" => "invalid", "unavailable" => "unavailable"}.fetch(fixture_name)
    abort("#{fixture_name} evaluation drifted") unless result.fetch("receipts").fetch(0).fetch("status") == expected
  end

  write_json(component_source, single_source_set("codex-cli"))
  oversized = command_json(RbConfig.ruby, OBSERVE.to_s, "--source-set", component_source.to_s, "--fixtures", oversized_root.to_s, "--checked-at", "2026-08-30T12:00:00Z")
  abort("oversized response was not rejected") unless oversized.fetch("observations").fetch(0).values_at("outcome", "reason_code") == ["invalid", "metadata_response_oversized"]

  changed_source = single_source_set("codex-cli", artifact: "maintenance/fixtures/changed-artifact.txt")
  write_json(component_source, changed_source)
  codex_observation = current_observations.merge("observations" => [current_observations.fetch("observations").find { |item| item.fetch("component_id") == "codex-cli" }])
  write_json(observations_path, codex_observation)
  changed = command_json(RbConfig.ruby, EVALUATE.to_s, "--source-set", component_source.to_s, "--observations", observations_path.to_s, "--as-of", "2026-08-30")
  abort("changed artifact did not withdraw claim") unless changed.fetch("receipts").fetch(0).values_at("status", "claim_retained") == ["changed", false]

  new_receipt = current.fetch("receipts").find { |item| item.fetch("component_id") == "codex-cli" }.merge(
    "status" => "new_version", "reason_code" => "unadmitted_version_observed", "observed_version" => "9.9.9", "claim_retained" => true
  )
  receipts_path = root.join("receipts.json")
  existing_path = root.join("existing.json")
  write_json(receipts_path, {"schema_name" => "roadmap-maintenance-receipt-set", "schema_version" => "1.0.0", "as_of" => "2026-08-30", "receipts" => [new_receipt]})
  write_json(existing_path, [])
  create = command_json(RbConfig.ruby, PLAN.to_s, "--receipts", receipts_path.to_s, "--existing", existing_path.to_s)
  abort("issue create plan missing") unless create.fetch("actions").map { |item| item.fetch("action") } == ["create"]
  generated = create.fetch("actions").fetch(0)
  owned_issue = {"number" => 17, "title" => generated.fetch("title"), "body" => generated.fetch("body"), "state" => "OPEN", "labels" => [{"name" => "impresari-maintenance"}]}
  write_json(existing_path, [owned_issue.merge("body" => owned_issue.fetch("body") + "\noutdated")])
  update = command_json(RbConfig.ruby, PLAN.to_s, "--receipts", receipts_path.to_s, "--existing", existing_path.to_s)
  abort("issue update plan missing") unless update.fetch("actions").map { |item| item.fetch("action") } == ["update"]

  stale_issue = owned_issue.merge("number" => 16, "body" => owned_issue.fetch("body").sub("codex-cli:new_version", "codex-cli:stale"))
  write_json(existing_path, [stale_issue])
  changed_condition = command_json(RbConfig.ruby, PLAN.to_s, "--receipts", receipts_path.to_s, "--existing", existing_path.to_s)
  abort("condition change did not close then create") unless changed_condition.fetch("actions").map { |item| item.fetch("action") } == ["close", "create"]

  duplicate = owned_issue.merge("number" => 18)
  write_json(existing_path, [owned_issue, duplicate, {"number" => 19, "title" => "unowned", "body" => "unowned", "state" => "OPEN", "labels" => []}])
  deduplicated = command_json(RbConfig.ruby, PLAN.to_s, "--receipts", receipts_path.to_s, "--existing", existing_path.to_s)
  abort("issue deduplication drifted") unless deduplicated.fetch("actions").map { |item| [item.fetch("action"), item.fetch("issue_number")] } == [["close", 18], ["noop", 17]]

  current_receipt = new_receipt.merge("status" => "current", "reason_code" => "admitted_version_current", "observed_version" => "0.149.0-alpha.4.1", "claim_retained" => true)
  write_json(receipts_path, {"schema_name" => "roadmap-maintenance-receipt-set", "schema_version" => "1.0.0", "as_of" => "2026-08-30", "receipts" => [current_receipt]})
  close = command_json(RbConfig.ruby, PLAN.to_s, "--receipts", receipts_path.to_s, "--existing", existing_path.to_s)
  abort("issue close plan touched unowned issue") unless close.fetch("actions").map { |item| item.fetch("issue_number") }.sort == [17, 18]

  denied_plan = root.join("denied-plan.json")
  write_json(denied_plan, close.merge("actions" => [close.fetch("actions").fetch(0)]))
  fake_bin = root.join("bin")
  FileUtils.mkdir_p(fake_bin)
  fake_gh = fake_bin.join("gh")
  fake_gh.write("#!/bin/sh\nexit 42\n")
  File.chmod(0o700, fake_gh)
  _stdout, _stderr, denied = Open3.capture3({"PATH" => "#{fake_bin}:#{ENV.fetch('PATH')}"}, RbConfig.ruby, APPLY.to_s, "--plan", denied_plan.to_s)
  abort("permission-denied issue mutation did not fail closed") if denied.success?
end

workflow = ROOT.join(".github/workflows/roadmap-maintenance.yml")
if workflow.exist?
  bytes = workflow.read
  abort("maintenance workflow gained broad permissions") if bytes.match?(/^permissions:\s+write-all/) || bytes.include?("contents: write") || bytes.include?("pull-requests: write") || bytes.include?("releases: write")
  abort("issue writer is not separately permissioned") unless bytes.include?("permissions: {}") && bytes.include?("issues: write")
  allowlisted = JSON.parse(SOURCE_SET.read).fetch("components").map { |component| component.dig("source", "host") }.compact.uniq
  abort("workflow embeds a non-allowlisted metadata host") unless bytes.scan(%r{https://([A-Za-z0-9.-]+)}).flatten.all? { |host| allowlisted.include?(host) }
end

puts "roadmap maintenance checks passed: bounded observation, 6 fail-closed states, and exact-owned issue lifecycle"
