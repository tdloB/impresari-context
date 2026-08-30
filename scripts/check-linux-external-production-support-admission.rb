#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
CHECKER = ROOT.join("scripts/linux-external-production-support-admission.rb")
MANIFEST = ROOT.join("tests/conformance/v1/valid/linux-external-production-support-manifest.json")

def run_check(overrides = {})
  options = {
    manifest: MANIFEST, support_surface: "github_actions_hosted", target_available: true,
    evidence_available: true, release_available: false, runner_label: "ubuntu-24.04",
    runner_image_version: "20260823.283.1", os_release: "24.04",
    kernel_release: "6.17.0-1022-azure", architecture: "x86_64", landlock_abi: "7",
    release_version: "", release_tag: "", release_archive_sha256: "", as_of: "2026-09-01",
  }.merge(overrides)
  command = [
    RbConfig.ruby, CHECKER.to_s, "--manifest", options[:manifest].to_s,
    "--support-surface", options[:support_surface], "--target-available", options[:target_available] ? "yes" : "no",
    "--evidence-available", options[:evidence_available] ? "yes" : "no", "--release-available", options[:release_available] ? "yes" : "no",
    "--runner-label", options[:runner_label], "--runner-image-version", options[:runner_image_version],
    "--os-release", options[:os_release], "--kernel-release", options[:kernel_release], "--arch", options[:architecture],
    "--landlock-abi", options[:landlock_abi], "--release-version", options[:release_version], "--release-tag", options[:release_tag],
    "--release-archive-sha256", options[:release_archive_sha256], "--as-of", options[:as_of],
  ]
  stdout, stderr, status = Open3.capture3(*command)
  abort("admission evaluator failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

cases = {
  "release_pending" => run_check,
  "stale_evidence" => run_check(as_of: "2026-09-14"),
  "changed" => run_check(kernel_release: "6.17.0-1023-azure"),
  "missing_evidence" => run_check(evidence_available: false),
  "unsupported" => run_check(support_surface: "generic_linux"),
  "unavailable" => run_check(target_available: false),
}
cases.each do |name, receipt|
  abort("unexpected #{name} status") unless receipt.fetch("status") == name
  abort("#{name} activated production support") unless receipt.fetch("support_claim_active") == false && receipt.fetch("production_admitted") == false
  abort("#{name} authorized a real analyzer") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} granted authority") unless receipt.fetch("authority").values.all? { |value| value == "denied" }
end

Dir.mktmpdir("impresari-linux-production-admission-") do |directory|
  changed = Pathname.new(directory).join("manifest.json")
  changed.write(MANIFEST.read.sub("pending_publication", "published"))
  command = [
    RbConfig.ruby, CHECKER.to_s, "--manifest", changed.to_s,
    "--support-surface", "github_actions_hosted", "--target-available", "yes",
    "--evidence-available", "yes", "--release-available", "no",
    "--runner-label", "ubuntu-24.04", "--runner-image-version", "20260823.283.1",
    "--os-release", "24.04", "--kernel-release", "6.17.0-1022-azure", "--arch", "x86_64",
    "--landlock-abi", "7", "--release-version", "", "--release-tag", "",
    "--release-archive-sha256", "", "--as-of", "2026-09-01",
  ]
  _stdout, stderr, status = Open3.capture3(*command)
  abort("changed manifest identity was accepted") if status.success?
  abort("changed manifest did not fail on pinned identity") unless stderr.include?("manifest identity is not the tracked admission")
end

puts "linux external production-support admission checks passed: release gate and 6 fail-closed states"
