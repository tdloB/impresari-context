#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

required_disclosure =
  "This release has not undergone an independent third-party security audit."
changelog = File.read("CHANGELOG.md").gsub(/\s+/, " ")
abort "v0.1.0 release notes must disclose independent audit status" unless changelog.include?(required_disclosure)

forbidden = [
  /independent (?:human )?(?:(?:security(?: and release)?|release) )?review (?:remains|is) (?:a )?public[- ]release (?:requirement|gate)/i,
  /independent (?:human )?(?:(?:security(?: and release)?|release) )?review (?:remains|is) (?:an? )?(?:open )?(?:release )?(?:blocker|gate)/i,
  /v0\.1\.0 (?:requires|must require|is blocked pending) (?:an? )?independent (?:human )?(?:security )?review/i,
  /independent (?:human )?(?:security )?review is (?:required|mandatory) (?:for|before publishing) v0\.1\.0/i,
  /before publishing v0\.1\.0.{0,120}(?:requires? )?(?:an? )?independent (?:human )?(?:security )?review/i
].freeze

# Keep representative regressions executable so equivalent wording cannot
# silently bypass the repository-policy check.
negative_fixtures = [
  "Independent security review remains a public-release gate.",
  "Independent review is a release blocker.",
  "v0.1.0 requires independent review before publication.",
  "Independent human security review is mandatory before publishing v0.1.0.",
  "Before publishing v0.1.0 the project requires an independent security review."
].freeze
negative_fixtures.each do |fixture|
  abort "release-assurance policy fixture escaped detection: #{fixture}" unless forbidden.any? { |pattern| fixture.match?(pattern) }
end

paths = ["README.md"] + Dir.glob("docs/**/*.md")
paths -= ["docs/decisions/0017-v0.1-release-assurance-policy.md"]
violations = paths.filter do |path|
  text = File.read(path).gsub(/\s+/, " ")
  forbidden.any? { |pattern| text.match?(pattern) }
end

unless violations.empty?
  abort "superseded v0.1.0 independent-review blocker language: #{violations.join(', ')}"
end

puts "release assurance policy checks passed: #{paths.length} documents"
