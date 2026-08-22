#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "open3"

root = File.expand_path("..", __dir__)
stdout, stderr, status = Open3.capture3(
  "cargo", "run", "-q", "-p", "context-evaluation", "--bin", "scale-eval",
  "--locked", "--offline", chdir: root
)
abort stderr unless status.success?
report = JSON.parse(stdout)
frozen = JSON.parse(File.read(File.join(root, "artifacts", "scale-evaluation-macos-aarch64.json")))

[report, frozen].each do |document|
  abort "wrong scale report schema" unless document["schema_name"] == "scale-evaluation-report"
  abort "two generated profiles required" unless document.fetch("profiles").length >= 2
  abort "at least five samples required" unless document.fetch("samples_per_profile") >= 5
  abort "scale failures present" unless document.fetch("failures").empty?
  document.fetch("profiles").each do |profile|
    abort "scale file count too small" unless profile.fetch("generated_files") >= 2_000
    abort "partial limit was not visible" unless profile.fetch("partial_limit_visible")
    %w[cold_snapshot warm_snapshot cold_lexical_query warm_lexical_query].each do |metric|
      values = profile.fetch(metric)
      abort "invalid timing unit" unless values.fetch("unit") == "milliseconds"
      abort "invalid timing percentile" unless values.fetch("p50") <= values.fetch("p95") && values.fetch("p95") <= values.fetch("maximum")
    end
  end
end
abort "frozen macOS report lacks measured RSS" unless frozen.fetch("peak_rss_bytes").positive?

puts "scale evaluation checks passed: #{report.fetch('profiles').length} profiles"
