#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "etc"
require_relative "lib/linux_external_delegation_capability"
require_relative "lib/linux_external_delegation_live_rehearsal"

abort("usage: ruby scripts/linux-external-delegation-live-receiver.rb") unless ARGV.empty?
abort("external live receiver requires Linux") unless RUBY_PLATFORM.include?("linux")

fd_root = "/proc/self/fd/#{LinuxExternalDelegationCapability::FIXED_DESCRIPTOR}"
capability = IO.new(LinuxExternalDelegationCapability::FIXED_DESCRIPTOR, autoclose: false)
directory_verified = capability.stat.directory?
membership = File.binread("/proc/self/cgroup", 4096)
match = membership.match(/\A0::(\/[^\n]*)\n?\z/)
current_root = match ? "/sys/fs/cgroup#{match[1]}" : nil

controllers = if directory_verified
  available = File.binread("#{fd_root}/cgroup.controllers", 4096).split
  LinuxExternalDelegationLiveRehearsal::REQUIRED_CONTROLLERS.select { |name| available.include?(name) }
else
  []
end
processes = directory_verified ? File.binread("#{fd_root}/cgroup.procs", 65_536).split : []
children = directory_verified ? Dir.children(fd_root).select { |name| File.directory?("#{fd_root}/#{name}") } : []
same_boundary = current_root && File.stat(fd_root).ino == File.stat(current_root).ino && File.stat(fd_root).dev == File.stat(current_root).dev
owner_verified = directory_verified && File.stat("#{fd_root}/cgroup.procs").uid == Process.euid
delegation_writable = directory_verified && File.writable?("#{fd_root}/cgroup.procs") && File.writable?("#{fd_root}/cgroup.subtree_control")

capability.close_on_exec = true
capability_facts = {
  "transport" => "inherited_directory_fd",
  "descriptor_slot" => 3,
  "received" => true,
  "directory_verified" => directory_verified,
  "close_on_exec_set" => capability.close_on_exec?,
  "raw_path_received" => false,
  "raw_cgroup_path_recorded" => false,
}
revalidation = {
  "executed" => true,
  "unified_cgroup_v2" => File.file?("/sys/fs/cgroup/cgroup.controllers") && File.file?("#{fd_root}/cgroup.controllers"),
  "owner_verified" => owner_verified,
  "process_contained" => same_boundary && processes.include?(Process.pid.to_s),
  "exclusive_descendants" => children.empty?,
  "delegation_writable" => delegation_writable,
  "controllers" => controllers,
  "raw_cgroup_path_recorded" => false,
}

puts JSON.pretty_generate(
  "observed_host" => {
    "operating_system" => "linux",
    "kernel_release" => File.binread("/proc/sys/kernel/osrelease", 256).strip,
    "architecture" => Etc.uname.fetch(:machine),
  },
  "capability" => capability_facts,
  "revalidation" => revalidation,
)

ready = capability_facts.values_at("received", "directory_verified", "close_on_exec_set").all? &&
  revalidation.values_at("unified_cgroup_v2", "owner_verified", "process_contained", "exclusive_descendants", "delegation_writable").all? &&
  controllers == LinuxExternalDelegationLiveRehearsal::REQUIRED_CONTROLLERS
exit 5 unless ready
