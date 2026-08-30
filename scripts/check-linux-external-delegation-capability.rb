#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require "rbconfig"
require "tmpdir"
require_relative "lib/linux_external_delegation_capability"

ROOT = Pathname.new(__dir__).join("..").expand_path
PROBE = ROOT.join("scripts/linux-external-delegation-capability-probe.rb")
FIXTURE = ROOT.join("tests/conformance/v1/valid/linux-external-delegation-capability-ready.json")

source = PROBE.read
abort("probe accepts caller arguments") unless source.include?('unless ARGV.empty?')
abort("probe descriptor is not fixed") unless source.include?("LinuxExternalDelegationCapability::FIXED_DESCRIPTOR")
abort("probe does not close inheritance") unless source.include?("close_on_exec = true")

cases = {
  "ready" => LinuxExternalDelegationCapability.build(received: true, directory: true, close_on_exec: true),
  "missing" => LinuxExternalDelegationCapability.build(received: false, directory: false, close_on_exec: false),
  "raw_path" => LinuxExternalDelegationCapability.build(received: true, directory: true, close_on_exec: true, raw_path_received: true),
  "wrong_slot" => LinuxExternalDelegationCapability.build(received: true, directory: true, close_on_exec: true, descriptor_slot: 4),
  "not_directory" => LinuxExternalDelegationCapability.build(received: true, directory: false, close_on_exec: true),
  "leakage_open" => LinuxExternalDelegationCapability.build(received: true, directory: true, close_on_exec: false),
}

expected = {
  "ready" => "transport_ready_for_host_rehearsal",
  "missing" => "unavailable",
  "raw_path" => "invalid_contract",
  "wrong_slot" => "invalid_contract",
  "not_directory" => "invalid_contract",
  "leakage_open" => "invalid_contract",
}
expected.each { |name, status| abort("unexpected #{name} status") unless cases.fetch(name).fetch("status") == status }
abort("ready fixture drift") unless cases.fetch("ready") == JSON.parse(FIXTURE.read)

cases.each do |name, receipt|
  active = name == "ready"
  abort("#{name} transport claim drift") unless receipt.fetch("transport_contract_active") == active
  abort("#{name} OS overclaim") unless receipt.fetch("os_confined") == false
  abort("#{name} production overclaim") unless receipt.fetch("production_admitted") == false
  abort("#{name} analyzer overclaim") unless receipt.fetch("real_analyzer_authorized") == false
  abort("#{name} privilege overclaim") unless receipt.fetch("privileged_installation_authorized") == false
  abort("#{name} path disclosure") unless receipt.dig("capability", "raw_cgroup_path_recorded") == false
end

if RUBY_PLATFORM.include?("linux")
  Dir.mktmpdir("impresari-external-capability-") do |directory|
    capability = File.open(directory, File::RDONLY)
    read_end, write_end = IO.pipe
    error_read, error_write = IO.pipe
    pid = Process.spawn(
      {}, RbConfig.ruby, PROBE.to_s,
      LinuxExternalDelegationCapability::FIXED_DESCRIPTOR => capability,
      out: write_end,
      err: error_write,
      unsetenv_others: true,
      close_others: true,
    )
    write_end.close
    error_write.close
    output = read_end.read
    errors = error_read.read
    _waited_pid, status = Process.wait2(pid)
    abort("transport probe failed: #{errors}") unless status.success?
    observed = JSON.parse(output)
    abort("live inherited descriptor transport failed") unless observed == cases.fetch("ready")
  ensure
    capability&.close
    read_end&.close
    error_read&.close
  end
end

puts "linux external delegation capability checks passed: 1 transport-ready and 5 fail-closed states"
