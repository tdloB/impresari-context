#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "pathname"
require "rubygems/package"
require "zlib"

expected_root = "yara-x-60ad06971467029e77967e59d580cbbe85a1474d"
archive = Pathname.new(ARGV.fetch(0)).expand_path
abort "missing or symlinked YARA-X archive" unless archive.file? && !archive.symlink?

entries = 0
roots = []
Zlib::GzipReader.open(archive.to_s) do |gzip|
  Gem::Package::TarReader.new(gzip) do |tar|
    tar.each do |entry|
      entries += 1
      name = entry.full_name
      abort "unsafe YARA-X archive path" if
        name.empty? || name.start_with?("/") || name.include?("\0") ||
        Pathname.new(name).each_filename.any? { |part| part == ".." }
      roots << name.split("/", 2).first
      type = entry.header.typeflag
      abort "unsupported YARA-X archive entry type #{type.inspect}" unless
        entry.file? || entry.directory? || type == "g"
    end
  end
end

abort "YARA-X archive entry count changed" unless entries == 2292
abort "YARA-X archive root changed" unless roots.uniq == [expected_root]
puts "YARA-X source archive verified: entries=#{entries} root=#{expected_root}"
