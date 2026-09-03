#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

# ADR-0016 depends on every decision having one stable identity. Two records
# sharing a number make every citation of that number ambiguous, and because
# the colliding files have different names, git merges them without conflict
# and no other check notices.

DECISIONS_DIR = "docs/decisions"
INDEX_PATH = File.join(DECISIONS_DIR, "README.md")

def decision_identity_violations(names, headings, index_text)
  violations = []

  by_number = Hash.new { |hash, key| hash[key] = [] }
  names.each do |name|
    number = name[/\A(\d{4})-/, 1]
    by_number[number] << name if number
  end

  by_number.sort.each do |number, claimants|
    next if claimants.length < 2

    violations << "ADR-#{number} is claimed by #{claimants.sort.join(' and ')}"
  end

  names.sort.each do |name|
    number = name[/\A(\d{4})-/, 1]
    next unless number

    heading = headings.fetch(name, "")
    unless heading.match?(/\A# ADR-#{number}\b/)
      violations << "#{name} heading does not state ADR-#{number}"
    end
    unless index_text.include?("(#{name})")
      violations << "#{name} is missing from the decisions index"
    end
  end

  violations
end

# Keep representative regressions executable so a collision, a mislabeled
# heading, or an unindexed record cannot silently pass this check.
negative_fixtures = [
  [
    "duplicate number",
    ["0116-observe-repository-reads.md", "0116-gate-external-delivery.md"],
    {
      "0116-observe-repository-reads.md" => "# ADR-0116: Observe repository reads",
      "0116-gate-external-delivery.md" => "# ADR-0116: Gate external delivery"
    },
    "(0116-observe-repository-reads.md) (0116-gate-external-delivery.md)"
  ],
  [
    "heading states a different number",
    ["0123-gate-external-delivery.md"],
    { "0123-gate-external-delivery.md" => "# ADR-0116: Gate external delivery" },
    "(0123-gate-external-delivery.md)"
  ],
  [
    "record missing from the index",
    ["0123-gate-external-delivery.md"],
    { "0123-gate-external-delivery.md" => "# ADR-0123: Gate external delivery" },
    ""
  ]
].freeze

negative_fixtures.each do |label, names, headings, index_text|
  if decision_identity_violations(names, headings, index_text).empty?
    abort "decision-identity fixture escaped detection: #{label}"
  end
end

names = Dir.children(DECISIONS_DIR).select { |name| name.match?(/\A\d{4}-.*\.md\z/) }
abort "no decision records found in #{DECISIONS_DIR}" if names.empty?

headings = names.to_h do |name|
  [name, File.foreach(File.join(DECISIONS_DIR, name)).first.to_s.strip]
end
index_text = File.read(INDEX_PATH)

violations = decision_identity_violations(names, headings, index_text)
unless violations.empty?
  abort "decision identity violations:\n  #{violations.join("\n  ")}"
end

puts "decision identity checks passed: #{names.length} records"
