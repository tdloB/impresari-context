#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "zlib"

abort "usage: build-macos-vm-initramfs.rb INIT MODULE OUTPUT" unless ARGV.length == 3
init_path, module_path, output_path = ARGV

entries = [
  [".", 0o040755, "".b, 0, 0],
  ["dev", 0o040755, "".b, 0, 0],
  ["dev/console", 0o020600, "".b, 5, 1],
  ["lib", 0o040755, "".b, 0, 0],
  ["lib/modules", 0o040755, "".b, 0, 0],
  ["sys", 0o040555, "".b, 0, 0],
  ["init", 0o100755, File.binread(init_path), 0, 0],
  ["lib/modules/virtio_blk.ko", 0o100444, File.binread(module_path), 0, 0]
]

def align_four(bytes)
  "\0".b * ((4 - (bytes % 4)) % 4)
end

def newc_entry(name, mode, data, inode, rdev_major, rdev_minor)
  encoded_name = "#{name}\0".b
  fields = [inode, mode, 0, 0, mode & 0o170000 == 0o040000 ? 2 : 1, 0,
            data.bytesize, 0, 0, rdev_major, rdev_minor, encoded_name.bytesize, 0]
  header = "070701#{fields.map { |value| format('%08x', value) }.join}".b
  header + encoded_name + align_four(header.bytesize + encoded_name.bytesize) +
    data + align_four(data.bytesize)
end

archive = +"".b
entries.each_with_index do |(name, mode, data, rdev_major, rdev_minor), index|
  archive << newc_entry(name, mode, data, index + 1, rdev_major, rdev_minor)
end
archive << newc_entry("TRAILER!!!", 0, "".b, entries.length + 1, 0, 0)

File.open(output_path, "wb") do |file|
  gzip = Zlib::GzipWriter.new(file, Zlib::BEST_COMPRESSION)
  gzip.mtime = 0
  gzip.orig_name = ""
  gzip.write(archive)
  gzip.close
end
