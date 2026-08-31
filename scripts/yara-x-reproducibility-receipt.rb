#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE = ROOT.join("profiles/v1/yara-x-reproducibility-diagnostic-v1.json")
HEX = /\A[0-9a-f]{64}\z/
RESULTS = %w[
  baseline_same_canonical_same
  baseline_changed_canonical_same
  baseline_changed_canonical_changed
  baseline_same_canonical_changed
].freeze
KEYS = %w[
  schema_name schema_version profile_id profile_sha256
  baseline_a_sha256 baseline_b_sha256 canonical_a_sha256 canonical_b_sha256
  result source_date_epoch offline_builds distinct_source_roots cleanup_required
  analyzer_executed rules_compiled artifact_uploaded artifact_retained
  production_admitted iar_2 authority_added
].freeze

def result_for(digests)
  baseline_same = digests.fetch(0) == digests.fetch(1)
  canonical_same = digests.fetch(2) == digests.fetch(3)
  "baseline_#{baseline_same ? 'same' : 'changed'}_canonical_#{canonical_same ? 'same' : 'changed'}"
end

def build_receipt(digests)
  raise "invalid executable digest" unless digests.length == 4 && digests.all? { |digest| HEX.match?(digest) }

  {
    "schema_name" => "yara-x-reproducibility-diagnostic-receipt",
    "schema_version" => "1.0.0",
    "profile_id" => "yara-x-reproducibility-diagnostic-v1",
    "profile_sha256" => "sha256:#{Digest::SHA256.file(PROFILE).hexdigest}",
    "baseline_a_sha256" => "sha256:#{digests.fetch(0)}",
    "baseline_b_sha256" => "sha256:#{digests.fetch(1)}",
    "canonical_a_sha256" => "sha256:#{digests.fetch(2)}",
    "canonical_b_sha256" => "sha256:#{digests.fetch(3)}",
    "result" => result_for(digests),
    "source_date_epoch" => "1787565021",
    "offline_builds" => true,
    "distinct_source_roots" => true,
    "cleanup_required" => true,
    "analyzer_executed" => false,
    "rules_compiled" => false,
    "artifact_uploaded" => false,
    "artifact_retained" => false,
    "production_admitted" => false,
    "iar_2" => false,
    "authority_added" => false
  }
end

def verify_receipt(receipt)
  raise "receipt keys changed" unless receipt.keys.sort == KEYS.sort
  digests = %w[baseline_a_sha256 baseline_b_sha256 canonical_a_sha256 canonical_b_sha256].map do |key|
    value = receipt.fetch(key)
    raise "invalid receipt digest" unless value.start_with?("sha256:") && HEX.match?(value.delete_prefix("sha256:"))
    value.delete_prefix("sha256:")
  end
  raise "receipt result changed" unless RESULTS.include?(receipt.fetch("result")) && receipt.fetch("result") == result_for(digests)
  expected = build_receipt(digests)
  raise "receipt contract changed" unless receipt == expected
  true
end

case ARGV.shift
when "emit"
  puts JSON.generate(build_receipt(ARGV))
when "verify"
  path = Pathname.new(ARGV.fetch(0)).expand_path
  raise "missing or symlinked receipt" unless path.file? && !path.symlink?
  verify_receipt(JSON.parse(path.read))
  puts "YARA-X reproducibility receipt verified"
else
  abort "usage: yara-x-reproducibility-receipt.rb emit <four sha256 hex digests> | verify <receipt.json>"
end
