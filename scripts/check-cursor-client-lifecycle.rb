#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "fileutils"
require "json"
require "open3"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
CHECKER = ROOT.join("scripts/client-lifecycle-health.rb")
MANIFEST = ROOT.join("client-lifecycle/cursor-native-guidance-v1.json")
TEMPLATE = ROOT.join("templates/client-guidance/cursor/impresari-context.mdc")

def run_check(target:, version: "2026.08.11-e8db854", available: true, os: "macos", arch: "aarch64", as_of: "2026-08-29")
  command = [RbConfig.ruby, CHECKER.to_s, "--manifest", MANIFEST.to_s, "--target", target.to_s, "--client-version", version, "--client-available", available ? "yes" : "no", "--os", os, "--arch", arch, "--as-of", as_of]
  stdout, stderr, status = Open3.capture3(*command)
  abort("health checker failed: #{stderr}") unless status.success?
  JSON.parse(stdout)
end

manifest = JSON.parse(MANIFEST.read)
abort("released template identity drift") unless Digest::SHA256.file(TEMPLATE).hexdigest == manifest.dig("artifact", "sha256")
evidence = ROOT.join(manifest.dig("evidence", "record"))
abort("evidence identity drift") unless Digest::SHA256.file(evidence).hexdigest == manifest.dig("evidence", "sha256")
public_manifest = JSON.parse(ROOT.join("docs/reference/compatibility-manifest-v1.json").read)
cursor = public_manifest.fetch("client_support").find { |entry| entry["client"] == "Cursor" }
abort("public CI-4 claim missing") unless cursor && cursor.fetch("lifecycle_maintenance") == {
  "level" => "L4",
  "manifest" => "client-lifecycle/cursor-native-guidance-v1.json",
  "scope" => manifest.fetch("surface"),
  "client_version" => manifest.fetch("supported_versions").fetch(0),
  "os" => manifest.fetch("supported_os").fetch(0),
  "arch" => manifest.fetch("supported_arch").fetch(0),
  "fresh_through" => manifest.dig("evidence", "fresh_through"),
}

Dir.mktmpdir("impresari-cursor-client-lifecycle-") do |directory|
  root = Pathname.new(directory)
  source = root.join("source.txt")
  source.write("immutable source fixture\n")
  source_before = Digest::SHA256.file(source).hexdigest
  owned = root.join("owned-rule.mdc")
  FileUtils.cp(TEMPLATE, owned)

  cases = {
    "compatible" => run_check(target: owned),
    "stale_evidence" => run_check(target: owned, as_of: "2026-11-26"),
    "unsupported" => run_check(target: owned, os: "windows"),
    "unknown" => run_check(target: owned, version: "9.9.9"),
    "client_unavailable" => run_check(target: owned, available: false),
    "missing" => run_check(target: root.join("missing-rule.mdc")),
  }
  changed = root.join("changed-rule.mdc")
  changed.write(TEMPLATE.read + "\nchanged\n")
  cases["changed"] = run_check(target: changed)
  unowned = root.join("unowned-rule.mdc")
  unowned.write("unowned\n")
  cases["unowned"] = run_check(target: unowned)

  expected = {
    "compatible" => "compatible",
    "stale_evidence" => "stale_evidence",
    "unsupported" => "unsupported",
    "unknown" => "unknown",
    "client_unavailable" => "degraded",
    "missing" => "degraded",
    "changed" => "degraded",
    "unowned" => "degraded",
  }
  expected.each { |name, status| abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status }
  cases.each_value do |result|
    abort("checker granted authority") unless result.fetch("authority").values.all? { |value| value == "denied" }
  end
  abort("source fixture changed") unless Digest::SHA256.file(source).hexdigest == source_before

  removal_root = root.join("removal")
  removal_target = removal_root.join(manifest.dig("artifact", "owned_relative_path"))
  FileUtils.mkdir_p(removal_target.dirname)
  FileUtils.cp(TEMPLATE, removal_target)
  unrelated = removal_target.dirname.join("unrelated.md")
  unrelated.write("preserve\n")
  abort("removal precondition identity mismatch") unless Digest::SHA256.file(removal_target).hexdigest == manifest.dig("artifact", "sha256")
  File.delete(removal_target)
  abort("exact removal failed") if removal_target.exist?
  abort("exact removal touched unrelated content") unless unrelated.read == "preserve\n"
end

malformed = ROOT.join("client-lifecycle/malformed-cursor-test-only.json")
begin
  File.write(malformed, "{not-json")
  _stdout, _stderr, status = Open3.capture3(RbConfig.ruby, CHECKER.to_s, "--manifest", malformed.to_s, "--target", TEMPLATE.to_s, "--client-version", "2026.08.11-e8db854", "--client-available", "yes", "--os", "macos", "--arch", "aarch64", "--as-of", "2026-08-29")
  abort("malformed manifest was accepted") if status.success?
ensure
  File.delete(malformed) if malformed.exist?
end

puts "Cursor client lifecycle checks passed: manifest identity and 8 deterministic health states"
