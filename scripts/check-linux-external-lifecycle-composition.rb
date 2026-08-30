#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "open3"
require "pathname"
require "rbconfig"
require "tmpdir"
require_relative "lib/linux_external_health_withdrawal"

ROOT = Pathname.new(__dir__).join("..").expand_path
COMPOSER = ROOT.join("scripts/linux-external-lifecycle-compose.rb")
COLLECTOR = ROOT.join("scripts/linux-external-health-withdrawal.rb")
PACKAGE_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-package-lifecycle-external.json")
EXTERNAL_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-external-delegation-live-rehearsal-candidate.json")
COMPOSITE_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-isolation-feasibility-candidate.json")
WITHDRAWAL_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-external-health-withdrawal.json")
COMPOSITION_FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-external-lifecycle-composition.json")
SOURCE_SHA = "6666666666666666666666666666666666666666"

def write_json(path, document)
  path.binwrite(JSON.pretty_generate(document) + "\n")
end

def compose(package:, external:, composite:, capability_available: false, expected_source: SOURCE_SHA)
  Dir.mktmpdir("impresari-linux-external-composition-") do |directory|
    root = Pathname.new(directory)
    package_path = root.join("package.json")
    external_path = root.join("external.json")
    composite_path = root.join("composite.json")
    withdrawal_path = root.join("withdrawal.json")
    write_json(package_path, package)
    write_json(composite_path, composite)
    external["observed_host"] = composite.fetch("observed_host").slice("operating_system", "kernel_release", "architecture")
    external["composite"]["receipt_identity"] = Digest::SHA256.hexdigest(composite_path.binread)
    write_json(external_path, external)
    clean_state = {
      "persistent_service_absent" => true,
      "privileged_policy_absent" => true,
      "stale_cgroup_absent" => true,
      "descendants_absent" => true,
      "staged_source_absent" => true,
    }
    withdrawal = LinuxExternalHealthWithdrawal.build(
      package_bytes: package_path.binread,
      external_bytes: external_path.binread,
      capability_available: capability_available,
      clean_state: clean_state,
    )
    write_json(withdrawal_path, withdrawal)
    stdout, stderr, status = Open3.capture3(
      RbConfig.ruby, COMPOSER.to_s,
      "--expected-source-sha", expected_source,
      "--package-receipt", package_path.to_s,
      "--external-receipt", external_path.to_s,
      "--composite-receipt", composite_path.to_s,
      "--withdrawal-receipt", withdrawal_path.to_s,
    )
    [JSON.parse(stdout), stderr, status, withdrawal]
  end
end

def documents
  [JSON.parse(PACKAGE_FIXTURE.read), JSON.parse(EXTERNAL_FIXTURE.read), JSON.parse(COMPOSITE_FIXTURE.read)]
end

package, external, composite = documents
candidate, stderr, status, withdrawal = compose(package: package, external: external, composite: composite)
abort("candidate composition failed: #{stderr}") unless status.success?
abort("candidate composition drift") unless candidate == JSON.parse(COMPOSITION_FIXTURE.read)
abort("withdrawal fixture drift") unless withdrawal == JSON.parse(WITHDRAWAL_FIXTURE.read)

if RbConfig::CONFIG.fetch("host_os").include?("linux")
  collector_stdout, collector_stderr, collector_status = Open3.capture3(
    {"GITHUB_ACTIONS" => "true", "RUNNER_ENVIRONMENT" => "github-hosted"},
    "bash", "-c", 'exec 3</dev/null; exec "$@"', "withdrawal-collector",
    RbConfig.ruby, COLLECTOR.to_s,
    "--package-receipt", PACKAGE_FIXTURE.to_s,
    "--external-receipt", EXTERNAL_FIXTURE.to_s,
  )
  abort("executable withdrawal collector did not return its closed withdrawal status: #{collector_stderr}") unless
    collector_status.success?
  abort("executable withdrawal collector drift") unless
    JSON.parse(collector_stdout) == JSON.parse(WITHDRAWAL_FIXTURE.read)
end

cases = {"candidate" => candidate}
package, external, composite = documents
cases["identity_mismatch"], = compose(package: package, external: external, composite: composite, expected_source: "7" * 40)
package, external, composite = documents
package["status"] = "package_lifecycle_partial"
cases["package_failed"], = compose(package: package, external: external, composite: composite)
package, external, composite = documents
external["status"] = "revalidation_failed"
external["external_candidate_active"] = false
external["os_confined"] = false
cases["external_failed"], = compose(package: package, external: external, composite: composite)
package, external, composite = documents
composite["checks"]["timeout"] = false
cases["interruption_failed"], = compose(package: package, external: external, composite: composite)
package, external, composite = documents
cases["withdrawal_failed"], = compose(package: package, external: external, composite: composite, capability_available: true)

expected = {
  "candidate" => "lifecycle_candidate",
  "identity_mismatch" => "identity_mismatch",
  "package_failed" => "package_failed",
  "external_failed" => "external_failed",
  "interruption_failed" => "interruption_failed",
  "withdrawal_failed" => "withdrawal_failed",
}
expected.each do |name, expected_status|
  receipt = cases.fetch(name)
  abort("unexpected #{name} state") unless receipt.fetch("status") == expected_status
  abort("#{name} candidate activation drift") unless receipt.fetch("lifecycle_candidate_active") == (name == "candidate")
  %w[production_admitted real_analyzer_authorized release_packaging_authorized privileged_installation_authorized persistent_service_authorized].each do |claim|
    abort("#{name} overclaimed #{claim}") unless receipt.fetch(claim) == false
  end
end

live_source = ROOT.join("scripts/linux-external-delegation-live-rehearsal.sh").read
abort("composition path must retain exactly one sudo systemd-run") unless live_source.scan(/sudo systemd-run/).length == 1
abort("external receipt is not persisted after collection") unless live_source.include?('> "$receipt"')
collector_source = ROOT.join("scripts/linux-external-health-withdrawal.rb").read
%w[systemd-run systemctl sudo curl wget].each do |forbidden|
  abort("health collector gained forbidden #{forbidden} authority") if collector_source.include?(forbidden)
end

puts "linux external lifecycle composition checks passed: 1 candidate and 5 fail-closed states"
