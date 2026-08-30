#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require_relative "lib/linux_external_delegation_capability"

abort("usage: ruby scripts/linux-external-delegation-capability-probe.rb") unless ARGV.empty?

received = true
directory = false
close_on_exec = false
begin
  capability = IO.new(LinuxExternalDelegationCapability::FIXED_DESCRIPTOR, autoclose: false)
  directory = capability.stat.directory?
  capability.close_on_exec = true
  close_on_exec = capability.close_on_exec?
rescue Errno::EBADF
  received = false
end

puts JSON.pretty_generate(
  LinuxExternalDelegationCapability.build(
    received: received,
    directory: directory,
    close_on_exec: close_on_exec,
  ),
)
