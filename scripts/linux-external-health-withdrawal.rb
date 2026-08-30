#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "optparse"
require_relative "lib/linux_external_health_withdrawal"

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby scripts/linux-external-health-withdrawal.rb --package-receipt FILE --external-receipt FILE"
  parser.on("--package-receipt FILE") { |value| options[:package] = value }
  parser.on("--external-receipt FILE") { |value| options[:external] = value }
end.parse!

abort("unexpected arguments") unless ARGV.empty?
abort("missing package receipt") unless options[:package]
abort("missing external receipt") unless options[:external]
abort("health withdrawal is restricted to ephemeral GitHub-hosted runners") unless
  ENV["GITHUB_ACTIONS"] == "true" && ENV["RUNNER_ENVIRONMENT"] == "github-hosted"

abort("package receipt exceeds 131072 bytes") if File.size(options.fetch(:package)) > 131_072
abort("external receipt exceeds 131072 bytes") if File.size(options.fetch(:external)) > 131_072
package_bytes = File.binread(options.fetch(:package))
external_bytes = File.binread(options.fetch(:external))
package = JSON.parse(package_bytes)
external = JSON.parse(external_bytes)
abort("external package receipt is not a candidate") unless
  package["profile"] == "externally_managed" && package["status"] == "package_lifecycle_candidate" &&
  package["claims"].is_a?(Hash) && package["claims"].values.none?
abort("external topology receipt is not a collected candidate") unless
  external["profile"] == "externally_managed" && external["status"] == "candidate_passed" &&
  external.dig("provisioner", "service_collected") == true && external.dig("provisioner", "persistent") == false &&
  external.dig("cleanup", "descendants_removed") == true &&
  %w[production_admitted real_analyzer_authorized privileged_installation_authorized].none? { |claim| external[claim] }

capability_available = begin
  IO.for_fd(3, autoclose: false)
  true
rescue Errno::EBADF
  false
end

clean_state = {
  "persistent_service_absent" => package.dig("clean_state", "installed_service_unit_absent") == true && external.dig("provisioner", "persistent") == false,
  "privileged_policy_absent" => package.dig("clean_state", "authorization_policy_absent") == true,
  "stale_cgroup_absent" => external.dig("provisioner", "service_collected") == true,
  "descendants_absent" => external.dig("cleanup", "descendants_removed") == true,
  "staged_source_absent" => package.dig("clean_state", "staged_source_absent") == true,
}
receipt = LinuxExternalHealthWithdrawal.build(
  package_bytes: package_bytes,
  external_bytes: external_bytes,
  capability_available: capability_available,
  clean_state: clean_state,
)
puts JSON.pretty_generate(receipt)
exit 7 unless receipt.fetch("status") == "withdrawn"
