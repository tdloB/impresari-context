#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROFILE_DIGEST = "9dbb28f52510e63e18834f0ece42a807b4ae03a9fff13fa97f954492a4631d62"
SOURCE_DIGEST = "2c793693e57d6e2f25cf5a38a38033b32afcf05bc56cc6deb088601d140fa9f7"
SCHEMA_DIGEST = "c8b5aa1004d2d192031e0f09e23e5af69111e1c685a67240713e82d9b690c881"
PROVENANCE_DIGEST = "f999a550c89c3f7621a74fa7f4d3011ba458f8f7e70670824e736ce14b201677"
CONFORMANCE_DIGESTS = {
  "valid/yara-x-production-ruleset-source-receipt.json" => "4676aa61b75d6dee12d86b84b62c3c57fc542ec3ac2152276c87334ab4409281",
  "valid/yara-x-production-ruleset-review-scope.json" => "e58098cd17bcdce7856b24bd7a5e10583070c0e3edaf8bbeb99102a6fcb276fa",
  "invalid/yara-x-production-ruleset-ai-review.json" => "42da543ad12896a6ae5c47f376b7f91c4b6352a62e6e127984ed6b45cc1ec4eb",
  "invalid/yara-x-production-ruleset-compiled-overclaim.json" => "9d04082969e6a8ea49e24a63ec5324a59e984fd8f29ca0fe8a21af060990aa66"
}.freeze
FIXTURE_ROLES = %w[benign_collision mutation near_miss positive].freeze
FALSE_CLAIMS = %w[
  human_review_complete compiled_rules_present compiler_executed analyzer_executed
  artifact_retained artifact_uploaded signed published ruleset_admitted
  production_admitted repository_scan_authorized iar_2_admitted
  detection_quality_claimed safety_claimed malware_free_claimed authority_added
].freeze

def json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort "invalid ADR-0106 JSON: #{path}: #{e.message}"
end

def exact(path, digest)
  abort "missing or symlinked ADR-0106 input: #{path}" unless path.file? && !path.symlink?
  abort "ADR-0106 input digest changed: #{path}" unless Digest::SHA256.file(path).hexdigest == digest
end

def literal_match?(bytes, text, modifiers)
  candidates = []
  candidates << text.b if modifiers.include?("ascii")
  candidates << text.encode("UTF-16LE").b if modifiers.include?("wide")
  candidates.any? do |candidate|
    haystack = modifiers.include?("nocase") ? bytes.downcase : bytes
    needle = modifiers.include?("nocase") ? candidate.downcase : candidate
    offset = 0
    matched = false
    while (index = haystack.index(needle, offset))
      if !modifiers.include?("fullword")
        matched = true
        break
      end
      before = index.zero? ? nil : haystack.getbyte(index - 1)
      after_index = index + needle.bytesize
      after = after_index >= haystack.bytesize ? nil : haystack.getbyte(after_index)
      word = ->(byte) { byte && ((byte >= 48 && byte <= 57) || (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122) || byte == 95) }
      starts_word = word.call(needle.getbyte(0))
      ends_word = word.call(needle.getbyte(needle.bytesize - 1))
      unless (starts_word && word.call(before)) || (ends_word && word.call(after))
        matched = true
        break
      end
      offset = index + 1
    end
    matched
  end
end

profile_path = ROOT.join("profiles/v1/yara-x-production-ruleset-v1.json")
sidecar_path = ROOT.join("profiles/v1/yara-x-production-ruleset-v1.sha256")
schema_path = ROOT.join("schemas/v1/yara-x-production-ruleset.schema.json")
source_path = ROOT.join("rules/yara-x/production-v1-candidate.yar")
conformance_root = ROOT.join("tests/conformance/v1")
provenance_path = conformance_root.join("yara-x-production-ruleset-fixture-provenance.json")

exact(profile_path, PROFILE_DIGEST)
exact(schema_path, SCHEMA_DIGEST)
exact(source_path, SOURCE_DIGEST)
exact(provenance_path, PROVENANCE_DIGEST)
CONFORMANCE_DIGESTS.each { |relative, digest| exact(conformance_root.join(relative), digest) }
abort "ADR-0106 profile sidecar changed" unless
  sidecar_path.read.strip == "#{PROFILE_DIGEST}  yara-x-production-ruleset-v1.json"

profile = json(profile_path)
abort "ADR-0106 source binding changed" unless
  profile.fetch("schema_name") == "yara-x-production-ruleset-profile" &&
  profile.fetch("profile_id") == "yara-x-production-ruleset-v1" &&
  profile.fetch("decision") == "ADR-0106" &&
  profile.fetch("state") == "source_candidate_review_required" &&
  profile.dig("source", "path") == "rules/yara-x/production-v1-candidate.yar" &&
  profile.dig("source", "sha256") == "sha256:#{SOURCE_DIGEST}" &&
  profile.dig("source", "bytes") == source_path.size &&
  profile.dig("source", "provenance") == "project-owned-original" &&
  !profile.dig("source", "synthetic_compatibility_source") &&
  !profile.dig("source", "third_party_content") &&
  !profile.dig("source", "reviewed") &&
  !profile.dig("source", "compiled")
abort "ADR-0106 profile overclaims" unless FALSE_CLAIMS.none? { |key| profile.fetch("claims").fetch(key) }

boundary = profile.fetch("language_boundary")
abort "ADR-0106 language surface expanded" unless
  boundary.fetch("allowed_pattern_kinds") == %w[literal_text hex] &&
  boundary.fetch("allowed_modifiers") == %w[ascii fullword nocase wide] &&
  boundary.fetch("allowed_condition_forms") == %w[and pattern_boolean] &&
  boundary.fetch("maximum_source_bytes") == 262_144 &&
  boundary.fetch("maximum_rules") == 256 &&
  boundary.fetch("maximum_patterns_per_rule") == 32 &&
  boundary.fetch("maximum_identifier_bytes") == 128 &&
  boundary.reject { |key, _| key.start_with?("allowed_") || key.start_with?("maximum_") }.values.none?

review = profile.fetch("review_contract")
abort "ADR-0106 manual review boundary changed" unless
  review.fetch("human") && review.fetch("independent_of_rule_authorship") &&
  review.fetch("yara_or_malware_analysis_experience") && review.fetch("conflict_disclosure") &&
  review.fetch("attributable_report") && review.fetch("exact_source_digest_binding") &&
  review.fetch("per_rule_disposition") && review.fetch("critical_high_unknown_must_be_zero") &&
  !review.fetch("ai_may_be_reviewer") && !review.fetch("complete")

source = source_path.binread
abort "ADR-0106 source is oversized or not UTF-8" unless source.bytesize <= 262_144 && source.dup.force_encoding("UTF-8").valid_encoding?
abort "ADR-0106 source lost ownership and non-claim headers" unless
  source.include?("SPDX-License-Identifier: Apache-2.0") &&
  source.include?("Copyright 2026 Aaron Boldt") &&
  source.include?("Unreviewed, uncompiled, unexecuted, unsigned, and not production-admitted")

body = source.lines.reject { |line| line.lstrip.start_with?("//") }.join
forbidden = {
  "import" => /^\s*import\b/, "include" => /^\s*include\b/,
  "regular expression" => %r{=\s*/}, "base64" => /\bbase64(?:wide)?\b/i,
  "xor" => /\bxor\b/i, "external variable" => /\bextern(?:al)?\b/i,
  "module" => /\b(?:pe|elf|macho|dotnet|crx|hash|math|time|vt|zip)\s*\./i,
  "private rule" => /^\s*private\s+rule\b/i, "global rule" => /^\s*global\s+rule\b/i
}.freeze
forbidden.each { |label, pattern| abort "ADR-0106 source contains forbidden #{label}" if body.match?(pattern) }

blocks = body.split(/(?=^rule )/).reject { |part| part.strip.empty? }
rules = profile.fetch("rules")
abort "ADR-0106 rule count changed" unless blocks.length == rules.length && rules.length == 3

parsed_rules = {}
blocks.each do |block|
  header = block.match(/\Arule\s+([a-z][a-z0-9_]*)\s*:\s*([^\n]+)\n\{/)
  abort "ADR-0106 rule header is outside the closed form" unless header
  identifier = header[1]
  abort "ADR-0106 duplicate rule identifier" if parsed_rules.key?(identifier)
  abort "ADR-0106 rule identifier exceeds 128 bytes" if identifier.bytesize > 128

  metadata_text = block[/\bmeta:\s*\n(.*?)\n\s*strings:/m, 1]
  strings_text = block[/\bstrings:\s*\n(.*?)\n\s*condition:/m, 1]
  condition = block[/\bcondition:\s*\n\s*(.*?)\s*\n\}/m, 1]
  abort "ADR-0106 rule sections are malformed" unless metadata_text && strings_text && condition

  metadata = metadata_text.lines.to_h do |line|
    match = line.match(/^\s*([a-z_]+)\s*=\s*"([^"\n]+)"\s*$/)
    abort "ADR-0106 rule metadata is outside the closed form" unless match
    [match[1], match[2]]
  end
  abort "ADR-0106 rule metadata keys changed" unless metadata.keys.sort == %w[category claim owner purpose]
  abort "ADR-0106 rule metadata overclaims" unless metadata.fetch("owner") == "Impresari Context" && metadata.fetch("claim") == "observation_only"

  patterns = strings_text.lines.to_h do |line|
    if (match = line.match(/^\s*\$([a-z][a-z0-9_]*)\s*=\s*"([\x20-\x7e]+)"\s+((?:ascii|wide|fullword|nocase)(?:\s+(?:ascii|wide|fullword|nocase))*)\s*$/))
      [match[1], {"kind" => "literal", "value" => match[2], "modifiers" => match[3].split}]
    elsif (match = line.match(/^\s*\$([a-z][a-z0-9_]*)\s*=\s*\{((?:\s+[0-9A-F]{2})+\s+)\}\s*$/))
      [match[1], {"kind" => "hex", "value" => match[2].split.map { |byte| byte.to_i(16) }}]
    else
      abort "ADR-0106 pattern is outside the literal/hex surface"
    end
  end
  abort "ADR-0106 rule requires one to 32 unique patterns" unless patterns.length.between?(1, 32)
  condition_ids = condition.split(/\s+and\s+/).map { |term| term.delete_prefix("$") }
  abort "ADR-0106 condition is outside the all-pattern boolean form" unless
    condition_ids.length == patterns.length && condition_ids.sort == patterns.keys.sort

  parsed_rules[identifier] = {
    "tags" => header[2].split.sort,
    "metadata" => metadata,
    "patterns" => patterns
  }
end

profile_identifiers = rules.map { |rule| rule.fetch("identifier") }
abort "ADR-0106 profile and source rule identities differ" unless profile_identifiers.sort == parsed_rules.keys.sort && profile_identifiers.uniq.length == 3

all_fixture_paths = []
rules.each do |rule|
  parsed = parsed_rules.fetch(rule.fetch("identifier"))
  abort "ADR-0106 tags differ from source" unless rule.fetch("tags").sort == parsed.fetch("tags")
  abort "ADR-0106 purpose/category differs from source" unless
    rule.fetch("purpose") == parsed.dig("metadata", "purpose") &&
    rule.fetch("category") == parsed.dig("metadata", "category") &&
    rule.fetch("pattern_count") == parsed.fetch("patterns").length
  abort "ADR-0106 rule lost limitations" if rule.fetch("known_false_positives").empty? || rule.fetch("blind_spots").empty?

  fixtures = rule.fetch("fixtures")
  abort "ADR-0106 fixture role matrix changed" unless fixtures.map { |fixture| fixture.fetch("role") }.sort == FIXTURE_ROLES
  fixtures.each do |fixture|
    relative = fixture.fetch("path")
    abort "ADR-0106 fixture path escaped the generated root" unless relative.match?(%r{\Atests/yara-x-production-ruleset/generated/[a-z0-9-]+\.fixture\z})
    fixture_path = ROOT.join(relative)
    exact(fixture_path, fixture.fetch("sha256").delete_prefix("sha256:"))
    abort "ADR-0106 fixture byte count changed" unless fixture_path.size == fixture.fetch("bytes") && fixture_path.size.between?(1, 4096)
    bytes = fixture_path.binread
    abort "ADR-0106 fixture is not original-generated marked text" unless bytes.downcase.include?("impresari")
    abort "ADR-0106 fixture contains a live network destination" if bytes.match?(%r{https?://(?!invalid\.example/)})

    observed = parsed.fetch("patterns").values.all? do |pattern|
      if pattern.fetch("kind") == "hex"
        bytes.bytes.each_cons(pattern.fetch("value").length).any? { |window| window == pattern.fetch("value") }
      else
        literal_match?(bytes, pattern.fetch("value"), pattern.fetch("modifiers"))
      end
    end
    expected = fixture.fetch("expected") == "match"
    abort "ADR-0106 source-only fixture expectation changed: #{relative}" unless observed == expected
    all_fixture_paths << relative
  end
end
abort "ADR-0106 fixture paths are not unique" unless all_fixture_paths.uniq.length == 12

provenance = json(provenance_path)
provenance_fixtures = provenance.fetch("fixtures").to_h do |fixture|
  [fixture.fetch("path"), fixture.fetch("sha256")]
end
expected_fixture_digests = rules.flat_map { |rule| rule.fetch("fixtures") }.to_h do |fixture|
  [fixture.fetch("path"), fixture.fetch("sha256").delete_prefix("sha256:")]
end
abort "ADR-0106 provenance lost the exact source identity" unless
  provenance.fetch("schema_name") == "fixture-provenance" &&
  provenance.fetch("decision") == "ADR-0106" &&
  provenance.fetch("review_status") == "author_provenance_recorded_independent_review_required" &&
  provenance.dig("rule_source", "path") == profile.dig("source", "path") &&
  provenance.dig("rule_source", "sha256") == SOURCE_DIGEST &&
  provenance.dig("rule_source", "license") == "Apache-2.0" &&
  provenance.dig("rule_source", "project_owned_original") &&
  !provenance.dig("rule_source", "third_party_content")
abort "ADR-0106 fixture provenance does not bind the exact generated corpus" unless
  provenance_fixtures == expected_fixture_digests
required_true_provenance = %w[impresari_owned generated non_malicious]
required_false_provenance = %w[
  executable_content malware_content third_party_content repository_source_content
  credential_content network_capture_content live_network_destination_content
  compiler_executed analyzer_executed human_independent_review_complete
  production_admitted authority_added
]
abort "ADR-0106 fixture provenance changed its ownership boundary" unless
  required_true_provenance.all? { |key| provenance.fetch(key) } &&
  required_false_provenance.none? { |key| provenance.fetch(key) }

receipt = json(conformance_root.join("valid/yara-x-production-ruleset-source-receipt.json"))
scope = json(conformance_root.join("valid/yara-x-production-ruleset-review-scope.json"))
[receipt, scope].each do |record|
  abort "ADR-0106 bounded record overclaims" unless FALSE_CLAIMS.none? { |key| record.fetch("claims").fetch(key) }
end
abort "ADR-0106 receipt lost the source-only result" unless
  receipt.fetch("state") == "source_candidate_review_required" &&
  receipt.fetch("rules") == 3 && receipt.fetch("fixtures") == 12 &&
  receipt.fetch("checks").values.all?
abort "ADR-0106 review scope lost the manual gate" unless
  scope.fetch("status") == "manual_review_required" &&
  scope.dig("required_review", "human") && scope.dig("required_review", "independent") &&
  scope.dig("required_review", "experienced") && !scope.dig("required_review", "ai_reviewer")

registry = json(ROOT.join("schemas/v1/registry.json")).fetch("schemas")
abort "ADR-0106 schema is not registered exactly once" unless
  registry.count { |entry| entry == {"name" => "yara-x-production-ruleset", "path" => "yara-x-production-ruleset.schema.json", "identity_object_kind" => nil} } == 1

manifest = json(conformance_root.join("manifest.json")).fetch("cases")
expected_cases = {
  "valid/yara-x-production-ruleset-source-receipt.json" => ["yara-x-production-ruleset.schema.json#/$defs/sourceReceipt", true],
  "valid/yara-x-production-ruleset-review-scope.json" => ["yara-x-production-ruleset.schema.json#/$defs/reviewScope", true],
  "invalid/yara-x-production-ruleset-ai-review.json" => ["yara-x-production-ruleset.schema.json#/$defs/reviewRecord", false],
  "invalid/yara-x-production-ruleset-compiled-overclaim.json" => ["yara-x-production-ruleset.schema.json#/$defs/sourceReceipt", false]
}.freeze
expected_cases.each do |fixture, (schema, valid)|
  abort "ADR-0106 conformance declaration changed: #{fixture}" unless
    manifest.count { |entry| entry == {"fixture" => fixture, "schema" => schema, "valid" => valid} } == 1
end

compiled = Dir.glob(ROOT.join("{rules,tests/yara-x-production-ruleset}/**/*.{yarc,bin,compiled}").to_s)
abort "ADR-0106 compiled artifact entered the source-only package" unless compiled.empty?

puts "YARA-X production ruleset source verified: rules=3 fixtures=12 state=manual_review_required compiler=false analyzer=false production=false"
