#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
CHECKER = ROOT.join("scripts/linux-isolation-maintenance.rb")
MANIFEST = ROOT.join("linux-isolation/linux-iar-1b-candidate-v1.json")

def run_check(overrides = {})
  options = {
    target_id: "ubuntu-24-04-x86-64",
    target_available: true,
    evidence_available: true,
    runner_label: "ubuntu-24.04",
    runner_image_version: "20260823.283.1",
    os_release: "24.04",
    kernel_release: "6.17.0-1022-azure",
    architecture: "x86_64",
    landlock_abi: "7",
    as_of: "2026-09-01",
  }.merge(overrides)
  command = [
    RbConfig.ruby, CHECKER.to_s,
    "--manifest", (options.delete(:manifest) || MANIFEST).to_s,
    "--target-id", options.fetch(:target_id),
    "--target-available", options.fetch(:target_available) ? "yes" : "no",
    "--evidence-available", options.fetch(:evidence_available) ? "yes" : "no",
    "--runner-label", options.fetch(:runner_label),
    "--runner-image-version", options.fetch(:runner_image_version),
    "--os-release", options.fetch(:os_release),
    "--kernel-release", options.fetch(:kernel_release),
    "--arch", options.fetch(:architecture),
    "--landlock-abi", options.fetch(:landlock_abi),
    "--as-of", options.fetch(:as_of),
  ]
  stdout, stderr, status = Open3.capture3(*command)
  abort("maintenance checker failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

manifest = JSON.parse(MANIFEST.read)
bindings = manifest.fetch("bindings")
bound_files = {
  "profile_sha256" => ROOT.join("tests/conformance/v1/valid/iar-linux-synthetic-profile.json"),
  "probe_sha256" => ROOT.join(bindings.fetch("probe_path")),
  "composite_check_sha256" => ROOT.join(bindings.fetch("composite_check_path")),
}
bound_files.each do |key, path|
  abort("bound artifact identity drift: #{path}") unless Digest::SHA256.file(path).hexdigest == bindings.fetch(key)
end

target_ids = manifest.fetch("targets").map { |target| target.fetch("target_id") }
abort("manifest target identifiers are not unique") unless target_ids.uniq == target_ids
manifest.fetch("targets").each do |target|
  evidence = target.fetch("evidence")
  fixture = ROOT.join(evidence.fetch("receipt_fixture"))
  abort("evidence identity drift: #{fixture}") unless Digest::SHA256.file(fixture).hexdigest == evidence.fetch("receipt_sha256")
end

cases = {
  "compatible_candidate" => run_check,
  "stale_evidence" => run_check(as_of: "2026-09-14"),
  "changed" => run_check(kernel_release: "6.17.0-1023-azure"),
  "missing_evidence" => run_check(evidence_available: false),
  "unsupported" => run_check(target_id: "ubuntu-99-99-x86-64"),
  "unavailable" => run_check(target_available: false),
  "diversity_only" => run_check(
    target_id: "ubuntu-22-04-x86-64",
    runner_label: "ubuntu-22.04",
    runner_image_version: "20260824.273.3",
    os_release: "22.04",
    kernel_release: "6.8.0-1064-azure",
    architecture: "x86_64",
    landlock_abi: "4",
  ),
}
expected = {
  "compatible_candidate" => "compatible_candidate",
  "stale_evidence" => "stale_evidence",
  "changed" => "changed",
  "missing_evidence" => "missing_evidence",
  "unsupported" => "unsupported",
  "unavailable" => "unavailable",
  "diversity_only" => "unsupported",
}
expected.each do |name, status|
  abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status
end
cases.each do |name, receipt|
  expected_active = name == "compatible_candidate"
  abort("#{name} candidate claim state is unsafe") unless receipt.fetch("candidate_claim_active") == expected_active
  abort("#{name} admitted production") unless receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} granted authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end
abort("diversity-only target became a candidate") unless cases.fetch("diversity_only").fetch("reason_code") == "diversity_only_not_candidate"

Dir.mktmpdir("impresari-linux-maintenance-") do |directory|
  malformed = Pathname.new(directory).join("manifest.json")
  malformed.write("{not-json")
  command = [
    RbConfig.ruby, CHECKER.to_s, "--manifest", malformed.to_s,
    "--target-id", "ubuntu-24-04-x86-64", "--target-available", "yes",
    "--evidence-available", "yes", "--runner-label", "ubuntu-24.04",
    "--runner-image-version", "20260823.283.1", "--os-release", "24.04",
    "--kernel-release", "6.17.0-1022-azure", "--arch", "x86_64",
    "--landlock-abi", "7", "--as-of", "2026-09-01",
  ]
  _stdout, _stderr, status = Open3.capture3(*command)
  abort("malformed manifest was accepted") if status.success?
end

puts "linux isolation maintenance checks passed: exact bindings and 7 fail-closed states"
