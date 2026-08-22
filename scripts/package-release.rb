#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"

root = File.expand_path("..", __dir__)
target = ARGV.fetch(0) { abort "usage: package-release.rb TARGET OUTPUT_DIR SOURCE_SHA" }
output = File.expand_path(ARGV.fetch(1), root)
source_sha = ARGV.fetch(2)
abort "invalid source SHA" unless source_sha.match?(/\A[0-9a-f]{40}\z/)

exe = target.include?("windows") ? ".exe" : ""
names = ["impresari-context", "impresari-context-structural-worker", "impresari-context-mcp"]
package_name = "impresari-context-0.0.0-#{target}"
stage = File.join(output, package_name)
FileUtils.rm_rf(stage)
FileUtils.mkdir_p(File.join(stage, "bin"))

names.each do |name|
  source = File.join(root, "target", "release", "#{name}#{exe}")
  abort "missing release binary #{source}" unless File.file?(source)
  FileUtils.cp(source, File.join(stage, "bin"))
end

%w[LICENSE NOTICE ACKNOWLEDGMENTS.md SECURITY.md SUPPORT.md].each do |name|
  source = File.join(root, name)
  FileUtils.cp(source, stage) if File.file?(source)
end
FileUtils.cp(File.join(root, "artifacts", "sbom.spdx.json"), File.join(stage, "sbom.spdx.json"))

files = Dir.glob(File.join(stage, "**", "*"), File::FNM_DOTMATCH).select { |path| File.file?(path) }.sort
manifest = {
  "schema_name" => "release-candidate-manifest",
  "schema_version" => "1.0.0",
  "project_version" => "0.0.0",
  "target" => target,
  "source_commit" => source_sha,
  "rust_toolchain" => File.read(File.join(root, "rust-toolchain.toml")).match(/channel\s*=\s*"([^"]+)"/)&.captures&.first,
  "files" => files.map do |path|
    { "path" => path.delete_prefix("#{stage}/"), "bytes" => File.size(path).to_s, "sha256" => Digest::SHA256.file(path).hexdigest }
  end
}
File.write(File.join(stage, "MANIFEST.json"), "#{JSON.pretty_generate(manifest)}\n")

FileUtils.mkdir_p(output)
archive = File.join(output, "#{package_name}.tar.gz")
FileUtils.rm_f(archive)
tar_output, tar_status = Open3.capture2e(
  "tar", "-czf", File.basename(archive), package_name, chdir: output
)
abort tar_output unless tar_status.success?
checksum = Digest::SHA256.file(archive).hexdigest
File.write("#{archive}.sha256", "#{checksum}  #{File.basename(archive)}\n")
puts archive
