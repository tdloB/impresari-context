#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "open3"
require "tempfile"

ROOT = File.expand_path("..", __dir__)
FROZEN = File.join(ROOT, "artifacts", "sbom.spdx.json")
document = JSON.parse(File.read(FROZEN))
abort "wrong SPDX version" unless document["spdxVersion"] == "SPDX-2.3"
abort "empty package inventory" if document.fetch("packages").empty?
abort "duplicate SPDX package IDs" unless document.fetch("packages").map { |item| item["SPDXID"] }.uniq.length == document.fetch("packages").length

Tempfile.create(["impresari-sbom", ".json"]) do |file|
  output, status = Open3.capture2e("ruby", File.join(ROOT, "scripts", "generate-sbom.rb"), file.path, chdir: ROOT)
  abort output unless status.success?
  abort "frozen SBOM differs from locked dependency graph" unless File.binread(file.path) == File.binread(FROZEN)
end

puts "SBOM checks passed: #{document.fetch('packages').length} packages"
