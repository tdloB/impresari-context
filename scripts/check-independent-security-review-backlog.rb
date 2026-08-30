#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
CHECKER = ROOT.join("scripts/independent-security-review-backlog.rb")
BACKLOG = ROOT.join("release-review/v0.2.0-independent-review-backlog.json")
PREPARED_SCOPE = ROOT.join("release-review/v0.2.0-independent-review-scope.json")
BACKLOG_FIXTURE = ROOT.join("tests/conformance/v1/valid/independent-security-review-backlog.json")

abort("review backlog and conformance fixture drifted") unless JSON.parse(BACKLOG.read) == JSON.parse(BACKLOG_FIXTURE.read)
abort("prepared review scope identity drifted") unless Digest::SHA256.file(PREPARED_SCOPE).hexdigest == "98a248d7133c85366a16b0a443dab15f131529d1bc4e3d8587b0adfc7925a45c"

def run_check(overrides = {})
  options = {
    backlog: BACKLOG, prepared_scope: PREPARED_SCOPE, backlog_available: true,
    prepared_scope_available: true, target_version: "0.2.0",
    current_product_commit: "1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51", release_requested: false,
  }.merge(overrides)
  command = [
    RbConfig.ruby, CHECKER.to_s, "--backlog", options[:backlog].to_s,
    "--prepared-scope", options[:prepared_scope].to_s,
    "--backlog-available", options[:backlog_available] ? "yes" : "no",
    "--prepared-scope-available", options[:prepared_scope_available] ? "yes" : "no",
    "--target-version", options[:target_version], "--current-product-commit", options[:current_product_commit],
    "--release-requested", options[:release_requested] ? "yes" : "no",
  ]
  stdout, stderr, status = Open3.capture3(*command)
  abort("review-backlog evaluator failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

cases = {
  "development_continues" => run_check,
  "scope_refresh_required" => run_check(current_product_commit: "0" * 40),
  "review_required_before_release" => run_check(release_requested: true),
  "missing_backlog" => run_check(backlog_available: false),
  "missing_scope" => run_check(prepared_scope_available: false),
  "invalid_commit" => run_check(current_product_commit: "invalid"),
  "unsupported" => run_check(target_version: "0.3.0"),
}
expected = {
  "development_continues" => "development_continues", "scope_refresh_required" => "scope_refresh_required",
  "review_required_before_release" => "review_required_before_release", "missing_backlog" => "missing_evidence",
  "missing_scope" => "missing_evidence", "invalid_commit" => "changed", "unsupported" => "unsupported",
}
expected.each do |name, status|
  receipt = cases.fetch(name)
  abort("unexpected #{name} status") unless receipt.fetch("status") == status
  abort("#{name} blocked ordinary roadmap development") unless receipt.fetch("roadmap_development_blocked") == false
  abort("#{name} admitted review or release") unless %w[review_gate_satisfied release_ready tag_authorized publication_authorized production_support_admitted real_analyzer_authorized].all? { |key| receipt.fetch(key) == false }
  abort("#{name} granted evaluator authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end

Dir.mktmpdir("impresari-review-backlog-") do |directory|
  changed = Pathname.new(directory).join("backlog.json")
  changed.write(BACKLOG.read.sub("deferred_to_release_candidate", "waived"))
  command = [
    RbConfig.ruby, CHECKER.to_s, "--backlog", changed.to_s, "--prepared-scope", PREPARED_SCOPE.to_s,
    "--backlog-available", "yes", "--prepared-scope-available", "yes", "--target-version", "0.2.0",
    "--current-product-commit", "1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51", "--release-requested", "no",
  ]
  _stdout, stderr, status = Open3.capture3(*command)
  abort("changed backlog identity was accepted") if status.success?
  abort("changed backlog did not fail on pinned identity") unless stderr.include?("backlog identity is not the tracked scheduling decision")
end

puts "independent security review backlog checks passed: development continues and 6 fail-closed states"
