#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "stringio"
require "zlib"

abort "usage: extract-macos-vm-kernel.rb ZBOOT OUTPUT" unless ARGV.length == 2
source_path, output_path = ARGV
source = File.binread(source_path)
abort "invalid ARM64 zboot header" unless source.byteslice(0, 2) == "MZ" && source.byteslice(4, 4) == "zimg"

payload_offset = source.byteslice(8, 4).unpack1("V")
payload_size = source.byteslice(12, 4).unpack1("V")
compression = source.byteslice(24, 4).delete("\0")
abort "unsupported ARM64 zboot compression" unless compression == "gzip"
abort "invalid ARM64 zboot payload range" unless payload_offset.positive? && payload_size.positive? &&
                                                 payload_offset + payload_size <= source.bytesize

payload = source.byteslice(payload_offset, payload_size)
image = Zlib::GzipReader.new(StringIO.new(payload)).read
abort "invalid raw ARM64 Linux image" unless image.bytesize >= 64 && image.byteslice(56, 4) == "ARM\x64".b
File.binwrite(output_path, image)
