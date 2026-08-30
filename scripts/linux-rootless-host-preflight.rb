#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"
require_relative "lib/linux_rootless_host_preflight"

abort("usage: ruby scripts/linux-rootless-host-preflight.rb") unless ARGV.empty?

root = Pathname.new(__dir__).join("..").expand_path
policy = root.join("linux-isolation/linux-iar-1b-production-topology-v1.json")
identity = LinuxRootlessHostPreflight.policy_identity(policy)
observed = LinuxRootlessHostPreflight.live_observation
puts JSON.pretty_generate(LinuxRootlessHostPreflight.assess(identity, observed))
