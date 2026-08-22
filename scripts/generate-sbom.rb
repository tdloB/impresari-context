#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"

ROOT = File.expand_path("..", __dir__)
OUTPUT = ARGV.fetch(0, File.join(ROOT, "artifacts", "sbom.spdx.json"))

metadata_json, status = Open3.capture2e(
  "cargo", "metadata", "--locked", "--offline", "--format-version", "1", chdir: ROOT
)
abort metadata_json unless status.success?
metadata = JSON.parse(metadata_json)
lock_digest = Digest::SHA256.file(File.join(ROOT, "Cargo.lock")).hexdigest

packages = metadata.fetch("packages").sort_by { |package| package.fetch("id") }.map.with_index do |package, index|
  source = package["source"]
  download = source ? "https://crates.io/crates/#{package.fetch('name')}/#{package.fetch('version')}" : "NOASSERTION"
  entry = {
    "SPDXID" => "SPDXRef-Package-#{index + 1}",
    "name" => package.fetch("name"),
    "versionInfo" => package.fetch("version"),
    "downloadLocation" => download,
    "filesAnalyzed" => false,
    "licenseConcluded" => "NOASSERTION",
    "licenseDeclared" => package["license"] || "NOASSERTION",
    "copyrightText" => "NOASSERTION",
    "externalRefs" => [{
      "referenceCategory" => "PACKAGE-MANAGER",
      "referenceType" => "purl",
      "referenceLocator" => "pkg:cargo/#{package.fetch('name')}@#{package.fetch('version')}"
    }]
  }
  checksum = package["checksum"]
  entry["checksums"] = [{"algorithm" => "SHA256", "checksumValue" => checksum}] if checksum
  entry
end

document = {
  "spdxVersion" => "SPDX-2.3",
  "dataLicense" => "CC0-1.0",
  "SPDXID" => "SPDXRef-DOCUMENT",
  "name" => "impresari-context-cargo-lock-#{lock_digest[0, 12]}",
  "documentNamespace" => "https://impresari-context.invalid/sbom/#{lock_digest}",
  "creationInfo" => {
    "created" => "2026-08-21T00:00:00Z",
    "creators" => ["Tool: impresari-context/scripts/generate-sbom.rb-1.0.0"]
  },
  "documentDescribes" => packages.map { |package| package.fetch("SPDXID") },
  "packages" => packages
}

FileUtils.mkdir_p(File.dirname(OUTPUT)) unless Dir.exist?(File.dirname(OUTPUT))
File.write(OUTPUT, JSON.pretty_generate(document) + "\n")
puts "wrote #{packages.length} locked packages to #{OUTPUT}"
