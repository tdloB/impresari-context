#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
GATE = ROOT.join("scripts/enforce-v0-2-independent-review-release-gate.rb")
PREPARED_SCOPE = ROOT.join("release-review/v0.2.0-independent-review-scope.json")
SOURCE_SHA = "1ed4500a6d3ac4a0d375c62f1c208ba8ddf98d51"

def run_gate(tag, source_sha, scope)
  Open3.capture3(RbConfig.ruby, GATE.to_s, tag, source_sha, scope.to_s)
end

_stdout, stderr, status = run_gate("v0.2.0", SOURCE_SHA, PREPARED_SCOPE)
abort("prepared scope passed the release gate") if status.success?
abort("prepared scope did not fail on missing review") unless stderr.include?("review scope has an unsupported shape")

legacy_stdout, legacy_stderr, legacy_status = run_gate("v0.1.0", SOURCE_SHA, PREPARED_SCOPE)
abort("v0.1.0 legacy policy was broken: #{legacy_stderr}") unless legacy_status.success? && legacy_stdout.include?("legacy policy applies")

Dir.mktmpdir("impresari-review-release-gate-") do |directory|
  admitted = JSON.parse(PREPARED_SCOPE.read)
  admitted["status"] = "review_recorded"
  admitted["claim"]["review_gate_satisfied"] = true
  admitted["report"] = {
    "reviewer_reference" => "Independent application-security reviewer",
    "independence_statement" => "Did not implement the reviewed source.",
    "conflict_disclosure" => "No conflict disclosed.",
    "report_sha256" => "a" * 64,
    "reviewed_commit" => SOURCE_SHA,
    "reviewed_at" => "2026-08-30",
    "critical_open" => 0,
    "high_open" => 0,
    "medium_dispositions_complete" => true,
    "low_documentation_complete" => true,
  }
  admitted_path = Pathname.new(directory).join("admitted.json")
  admitted_path.write(JSON.pretty_generate(admitted) + "\n")

  stdout, stderr, status = run_gate("v0.2.0", SOURCE_SHA, admitted_path)
  abort("exact admitted review was rejected: #{stderr}") unless status.success? && stdout.include?("release gate passed")

  _stdout, stderr, status = run_gate("v0.2.0", "0" * 40, admitted_path)
  abort("source-mismatched review passed") if status.success?
  abort("source mismatch reason drifted") unless stderr.include?("reviewed source does not match")

  _stdout, stderr, status = run_gate("v0.3.0", SOURCE_SHA, admitted_path)
  abort("unrecorded release policy passed") if status.success?
  abort("unrecorded policy reason drifted") unless stderr.include?("no independent review release policy")
end

puts "v0.2 independent review release gate checks passed: legacy preserved, current blocked, exact future admission accepted"
