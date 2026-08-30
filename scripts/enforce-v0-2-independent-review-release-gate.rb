#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"

class GateError < StandardError; end

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_SCOPE = ROOT.join("release-review/v0.2.0-independent-review-candidate-scope.json")
DEFAULT_RECORD = ROOT.join("release-review/v0.2.0-independent-review-record.json")
SHA256 = /\A[0-9a-f]{64}\z/
COMMIT = /\A[0-9a-f]{40}\z/

def exact_keys!(value, keys, description)
  raise GateError, "#{description} has an unsupported shape" unless value.is_a?(Hash) && value.keys.sort == keys.sort
end

def tracked_file!(path, description)
  raise GateError, "#{description} is unavailable" unless path.file? && !path.symlink?
end

tag = ARGV.fetch(0) { abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG RELEASE_SHA [SCOPE] [RECORD]" }
release_sha = ARGV.fetch(1) { abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG RELEASE_SHA [SCOPE] [RECORD]" }
scope_path = Pathname.new(ARGV.fetch(2, DEFAULT_SCOPE.to_s))
record_path = Pathname.new(ARGV.fetch(3, DEFAULT_RECORD.to_s))
abort "usage: enforce-v0-2-independent-review-release-gate.rb TAG RELEASE_SHA [SCOPE] [RECORD]" unless ARGV.length.between?(2, 4)

begin
  raise GateError, "release source identity is invalid" unless release_sha.match?(COMMIT)
  if tag == "v0.1.0"
    puts "independent review release gate: v0.1.0 legacy policy applies"
    exit 0
  end
  raise GateError, "no independent review release policy is recorded for #{tag}" unless tag == "v0.2.0"

  tracked_file!(scope_path, "candidate review scope")
  scope_bytes = File.binread(scope_path)
  scope = JSON.parse(scope_bytes)
  exact_keys!(scope, %w[schema_name schema_version scope_id target_version product_source_commit status triggered_boundaries review_areas required_artifacts candidate_evidence release_controls allowed_descendant_paths reviewer_requirements finding_policy claim safe_next_step], "candidate review scope")
  raise GateError, "candidate review scope identity is unsupported" unless scope.values_at("schema_name", "schema_version", "scope_id", "target_version", "status") == [
    "independent-security-review-scope", "1.0.0", "impresari-context-v0-2-0-candidate-review-v1", "0.2.0", "manual_review_required"
  ]
  product_sha = scope.fetch("product_source_commit")
  raise GateError, "reviewed product source identity is invalid" unless product_sha.match?(COMMIT)
  raise GateError, "candidate scope claim overreaches" unless scope.fetch("claim") == {
    "review_gate_satisfied" => false, "release_ready" => false, "publication_authorized" => false,
    "production_support_admitted" => false, "real_analyzer_authorized" => false,
  }

  evidence = scope.fetch("candidate_evidence")
  exact_keys!(evidence, %w[workflow_run_id workflow_url status source_commit completed_at artifacts], "candidate evidence")
  raise GateError, "candidate evidence is not bound to the reviewed product" unless evidence.fetch("status") == "success" && evidence.fetch("source_commit") == product_sha
  artifacts = evidence.fetch("artifacts")
  raise GateError, "candidate artifact set is incomplete" unless artifacts.is_a?(Array) && artifacts.length == 3
  expected_targets = %w[aarch64-apple-darwin x86_64-pc-windows-msvc x86_64-unknown-linux-gnu]
  targets = artifacts.map do |artifact|
    exact_keys!(artifact, %w[artifact_id target name workflow_artifact_sha256 archive_sha256 manifest_sha256 expires_at], "candidate artifact")
    raise GateError, "candidate artifact identity is invalid" unless %w[workflow_artifact_sha256 archive_sha256 manifest_sha256].all? { |key| artifact.fetch(key).match?(SHA256) }
    artifact.fetch("target")
  end
  raise GateError, "candidate artifact targets drifted" unless targets.sort == expected_targets

  controls = scope.fetch("release_controls")
  raise GateError, "release controls are missing" unless controls.is_a?(Array) && !controls.empty?
  controls.each do |control|
    exact_keys!(control, %w[path sha256], "release control")
    path = ROOT.join(control.fetch("path"))
    tracked_file!(path, "release control")
    raise GateError, "release control changed: #{control.fetch('path')}" unless Digest::SHA256.file(path).hexdigest == control.fetch("sha256")
  end

  if release_sha == product_sha
    changed_paths = []
  else
    _stdout, _stderr, ancestry = Open3.capture3("git", "merge-base", "--is-ancestor", product_sha, release_sha, chdir: ROOT.to_s)
    raise GateError, "release source does not descend from the reviewed product" unless ancestry.success?
    changed_stdout, changed_stderr, changed_status = Open3.capture3("git", "diff", "--name-only", "--diff-filter=ACMRT", "#{product_sha}..#{release_sha}", chdir: ROOT.to_s)
    raise GateError, "release descendant path check failed: #{changed_stderr.strip}" unless changed_status.success?
    changed_paths = changed_stdout.lines(chomp: true).reject(&:empty?)
  end
  allowed_paths = scope.fetch("allowed_descendant_paths")
  raise GateError, "allowed release metadata path list is invalid" unless allowed_paths.is_a?(Array) && allowed_paths.uniq == allowed_paths
  disallowed = changed_paths - allowed_paths
  raise GateError, "product or unreviewed release control changed after review: #{disallowed.join(', ')}" unless disallowed.empty?

  tracked_file!(record_path, "independent review record")
  record = JSON.parse(File.binread(record_path))
  exact_keys!(record, %w[schema_name schema_version scope_id target_version scope_identity product_source_commit status reviewer_reference independence_statement conflict_disclosure report_sha256 reviewed_at critical_open high_open unknown_open medium_dispositions_complete low_documentation_complete claim safe_next_step], "independent review record")
  raise GateError, "independent review record identity is unsupported" unless record.values_at("schema_name", "schema_version", "scope_id", "target_version", "status") == [
    "independent-security-review-record", "1.0.0", scope.fetch("scope_id"), "0.2.0", "review_recorded"
  ]
  raise GateError, "review record is not bound to the candidate scope" unless record.fetch("scope_identity") == Digest::SHA256.hexdigest(scope_bytes)
  raise GateError, "review record is not bound to the reviewed product" unless record.fetch("product_source_commit") == product_sha
  raise GateError, "review report identity is invalid" unless record.fetch("report_sha256").match?(SHA256)
  raise GateError, "reviewer attribution or independence is missing" if %w[reviewer_reference independence_statement conflict_disclosure].any? { |key| record.fetch(key).to_s.empty? }
  raise GateError, "blocking review findings remain open" unless record.fetch("critical_open").zero? && record.fetch("high_open").zero? && record.fetch("unknown_open").zero?
  raise GateError, "review finding dispositions are incomplete" unless record.fetch("medium_dispositions_complete") && record.fetch("low_documentation_complete")
  raise GateError, "review claim is not narrowly admitted" unless record.fetch("claim") == {
    "review_gate_satisfied" => true, "release_ready" => false, "publication_authorized" => false,
    "production_support_admitted" => false, "real_analyzer_authorized" => false,
  }
  puts "independent review release gate passed for #{tag} at #{release_sha} from reviewed product #{product_sha}"
rescue JSON::ParserError
  warn "error: independent review evidence is not valid JSON"
  exit 1
rescue GateError, KeyError => error
  warn "error: #{error.message}"
  exit 1
end
