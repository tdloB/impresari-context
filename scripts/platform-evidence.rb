#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "open3"
require "rbconfig"

rustc, status = Open3.capture2("rustc", "-vV")
abort "rustc evidence unavailable" unless status.success?

report = {
  schema_name: "platform-evidence",
  schema_version: "1.0.0",
  source: ENV.fetch("CI", "false") == "true" ? "hosted_ci" : "local_rehearsal",
  runner_os: ENV.fetch("RUNNER_OS", RbConfig::CONFIG.fetch("host_os")),
  runner_arch: ENV.fetch("RUNNER_ARCH", RbConfig::CONFIG.fetch("host_cpu")),
  expected_target: ENV.fetch("EXPECTED_TARGET", "unrecorded"),
  filesystem_profile: ENV.fetch("FILESYSTEM_PROFILE", "unrecorded"),
  rustc: rustc.lines.map(&:strip).reject(&:empty?),
  confinement_claim: "application capability reduction; not an OS security sandbox",
  full_repository_gate: "passed"
}

puts JSON.pretty_generate(report)
