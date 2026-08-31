#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "openssl"
require "rubygems/package"
require "stringio"
require "zlib"

EXPECTED_SIGNATURE_NAME =
  ".SIGN.RSA.alpine-devel@lists.alpinelinux.org-616ae350.rsa.pub"
MAX_ARCHIVE_BYTES = 64 * 1024 * 1024
MAX_SIGNATURE_TAR_BYTES = 4096
MAX_CONTROL_TAR_BYTES = 128 * 1024
MAX_INDEX_TAR_BYTES = 8 * 1024 * 1024

abort "usage: verify-alpine-apkv2.rb MODE ARCHIVE PUBLIC_KEY" unless ARGV.length == 3
mode, archive_path, key_path = ARGV
abort "unsupported Alpine APKv2 verification mode" unless %w[index package].include?(mode)

[archive_path, key_path].each do |path|
  abort "refusing missing or symlinked APKv2 input: #{path}" unless File.file?(path) && !File.symlink?(path)
end

archive = File.binread(archive_path)
abort "refusing oversized APKv2 input" if archive.empty? || archive.bytesize > MAX_ARCHIVE_BYTES

segments = []
offset = 0
while offset < archive.bytesize
  abort "too many APKv2 gzip segments" if segments.length >= 3
  begin
    source = StringIO.new(archive.byteslice(offset..))
    reader = Zlib::GzipReader.new(source)
    inflated = reader.read
    consumed = source.pos - reader.unused.to_s.bytesize
    abort "invalid APKv2 gzip segment" unless consumed.positive?
    segments << {compressed: archive.byteslice(offset, consumed), inflated: inflated}
    offset += consumed
  rescue Zlib::GzipFile::Error, Zlib::Error => e
    abort "invalid APKv2 gzip framing: #{e.message}"
  end
end

expected_segments = mode == "package" ? 3 : 2
abort "unexpected APKv2 segment count" unless segments.length == expected_segments
abort "oversized APKv2 signature tar" if segments[0][:inflated].bytesize > MAX_SIGNATURE_TAR_BYTES

def exact_tar_file(bytes, expected_name, maximum_bytes, allowed_names = [expected_name])
  selected = nil
  Gem::Package::TarReader.new(StringIO.new(bytes)) do |tar|
    tar.each do |entry|
      next unless entry.file?
      abort "unexpected file in bounded APKv2 tar: #{entry.full_name}" unless allowed_names.include?(entry.full_name)
      next unless entry.full_name == expected_name
      abort "duplicate file in bounded APKv2 tar: #{entry.full_name}" if selected
      abort "oversized file in bounded APKv2 tar: #{entry.full_name}" if entry.header.size > maximum_bytes
      selected = entry.read
    end
  end
  abort "missing bounded APKv2 tar member: #{expected_name}" unless selected
  selected
rescue Gem::Package::TarInvalidError => e
  abort "invalid bounded APKv2 tar: #{e.message}"
end

signature = exact_tar_file(segments[0][:inflated], EXPECTED_SIGNATURE_NAME, 1024)
abort "unexpected Alpine APKv2 signature size" unless signature.bytesize == 512
key = OpenSSL::PKey::RSA.new(File.binread(key_path))
abort "Alpine APKv2 RSA/SHA-1 signature verification failed" unless
  key.verify(OpenSSL::Digest::SHA1.new, signature, segments[1][:compressed])

result = {
  schema_name: "alpine-apkv2-verification",
  schema_version: "1.0.0",
  mode: mode,
  signature_name: EXPECTED_SIGNATURE_NAME,
  signature_sha256: "sha256:#{Digest::SHA256.hexdigest(signature)}",
  rsa_sha1_signature_verified: true,
  segment_bytes: segments.map { |segment| segment[:compressed].bytesize.to_s }
}

if mode == "package"
  abort "oversized APKv2 control tar" if segments[1][:inflated].bytesize > MAX_CONTROL_TAR_BYTES
  pkginfo = exact_tar_file(segments[1][:inflated], ".PKGINFO", 16 * 1024)
  fields = pkginfo.lines(chomp: true).each_with_object({}) do |line, selected|
    key_name, value = line.split(" = ", 2)
    selected[key_name] = value if value
  end
  datahash = fields.fetch("datahash") { abort "signed APKv2 metadata lacks datahash" }
  actual_datahash = Digest::SHA256.hexdigest(segments[2][:compressed])
  abort "signed APKv2 datahash mismatch" unless datahash == actual_datahash
  result[:package] = fields.slice(
    "pkgname", "pkgver", "arch", "origin", "commit", "builddate", "size", "datahash"
  )
  result[:datahash_verified] = true
else
  abort "oversized APKINDEX tar" if segments[1][:inflated].bytesize > MAX_INDEX_TAR_BYTES
  index = exact_tar_file(
    segments[1][:inflated], "APKINDEX", MAX_INDEX_TAR_BYTES, ["DESCRIPTION", "APKINDEX"]
  )
  selected = index.split("\n\n").find do |record|
    record.include?("\nP:linux-virt\n") && record.include?("\nV:6.18.48-r0\n")
  end
  abort "signed APKINDEX lacks exact linux-virt replacement record" unless selected
  result[:index_record] = selected.lines(chomp: true).to_h { |line| line.split(":", 2) }
end

puts JSON.generate(result)
