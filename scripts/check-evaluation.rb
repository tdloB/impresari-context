#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "open3"

root = File.expand_path("..", __dir__)
stdout, stderr, status = Open3.capture3(
  "cargo", "run", "-q", "-p", "context-evaluation", "--locked", "--offline", chdir: root
)
abort stderr unless status.success?
actual = JSON.parse(stdout)
frozen = JSON.parse(File.read(File.join(root, "artifacts", "evaluation-local.json")))
abort "evaluation output differs from frozen local result" unless actual == frozen
abort "synthetic corpus minimum not met" unless actual.fetch("fixture_count") >= 12
abort "held-out split below 25%" unless actual.fetch("heldout_count") * 4 >= actual.fetch("fixture_count")
abort "evaluation failures present" unless actual.fetch("failures").empty?

puts "evaluation checks passed: #{actual.fetch('fixture_count')} fixtures"
