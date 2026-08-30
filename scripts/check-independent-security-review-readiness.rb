#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
CHECKER = ROOT.join("scripts/independent-security-review-readiness.rb")
SCOPE = ROOT.join("release-review/v0.2.0-independent-review-scope.json")
SCOPE_FIXTURE = ROOT.join("tests/conformance/v1/valid/independent-security-review-scope.json")

scope_document = JSON.parse(SCOPE.read)
scope_fixture = JSON.parse(SCOPE_FIXTURE.read)
abort("review scope and conformance fixture drifted") unless scope_document == scope_fixture
artifacts = scope_document.fetch("required_artifacts")
abort("review artifact list contains duplicates") unless artifacts.uniq == artifacts
artifacts.each do |relative|
  path = ROOT.join(relative)
  abort("required review artifact is missing: #{relative}") unless path.file? && !path.symlink?
end

def run_check(overrides = {})
  options = {
    scope: SCOPE, scope_available: true, report_available: false, reviewer_independent: false,
    target_version: "0.2.0", product_source_commit: "1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51",
    report_sha256: "", critical_open: 0, high_open: 0,
  }.merge(overrides)
  command = [
    RbConfig.ruby, CHECKER.to_s, "--scope", options[:scope].to_s,
    "--scope-available", options[:scope_available] ? "yes" : "no",
    "--report-available", options[:report_available] ? "yes" : "no",
    "--reviewer-independent", options[:reviewer_independent] ? "yes" : "no",
    "--target-version", options[:target_version], "--product-source-commit", options[:product_source_commit],
    "--report-sha256", options[:report_sha256], "--critical-open", options[:critical_open].to_s,
    "--high-open", options[:high_open].to_s,
  ]
  stdout, stderr, status = Open3.capture3(*command)
  abort("review-readiness evaluator failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

cases = {
  "manual_review_required" => run_check,
  "changed" => run_check(product_source_commit: "0" * 40),
  "missing_evidence" => run_check(scope_available: false),
  "invalid_independence" => run_check(report_available: true, reviewer_independent: false, report_sha256: "1" * 64),
  "invalid_findings" => run_check(report_available: true, reviewer_independent: true, report_sha256: "1" * 64, high_open: 1),
  "unsupported" => run_check(target_version: "0.3.0"),
}
expected = {
  "manual_review_required" => "manual_review_required", "changed" => "changed",
  "missing_evidence" => "missing_evidence", "invalid_independence" => "invalid_review",
  "invalid_findings" => "invalid_review", "unsupported" => "unsupported",
}
expected.each do |name, status|
  receipt = cases.fetch(name)
  abort("unexpected #{name} status") unless receipt.fetch("status") == status
  abort("#{name} satisfied the review gate") unless receipt.fetch("review_gate_satisfied") == false
  abort("#{name} granted release or runtime authority") unless %w[release_ready publication_authorized production_support_admitted real_analyzer_authorized].all? { |key| receipt.fetch(key) == false }
  abort("#{name} granted evaluator authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end

Dir.mktmpdir("impresari-review-readiness-") do |directory|
  changed = Pathname.new(directory).join("scope.json")
  changed.write(SCOPE.read.sub("manual_review_required", "review_recorded"))
  command = [
    RbConfig.ruby, CHECKER.to_s, "--scope", changed.to_s, "--scope-available", "yes",
    "--report-available", "no", "--reviewer-independent", "no", "--target-version", "0.2.0",
    "--product-source-commit", "1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51",
    "--report-sha256", "", "--critical-open", "0", "--high-open", "0",
  ]
  _stdout, stderr, status = Open3.capture3(*command)
  abort("changed scope identity was accepted") if status.success?
  abort("changed scope did not fail on pinned identity") unless stderr.include?("scope identity is not the tracked review package")
end

puts "independent security review readiness checks passed: manual gate and 5 fail-closed states"
