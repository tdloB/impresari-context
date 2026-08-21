#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "base64"
require "json"
require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
cases = JSON.parse(root.join("tests/conformance/v1/path-vectors.json").read).fetch("cases")

def canonical_decode(encoded)
  return nil if encoded.empty? || encoded.include?("=") || !encoded.match?(/\A[A-Za-z0-9_-]+\z/)
  bytes = Base64.urlsafe_decode64(encoded)
  return nil unless Base64.urlsafe_encode64(bytes, padding: false) == encoded
  bytes
rescue ArgumentError
  nil
end

def valid_unix?(bytes)
  return false if bytes.nil? || bytes.empty? || bytes.start_with?("/") || bytes.end_with?("/") || bytes.include?("\0")
  bytes.split("/", -1).none? { |part| part.empty? || part == "." || part == ".." }
end

def valid_windows?(bytes)
  return false if bytes.nil? || bytes.empty? || bytes.bytesize.odd?
  units = bytes.unpack("v*")
  return false if units.include?(0) || units.include?(0x2f) || units.first == 0x5c || units.last == 0x5c
  return false if units.length >= 2 && units[1] == 0x3a
  parts = units.slice_when { |unit| unit == 0x5c }.map { |part| part.reject { |unit| unit == 0x5c } }
  parts.none? { |part| part.empty? || part == [0x2e] || part == [0x2e, 0x2e] }
end

cases.each do |test_case|
  bytes = canonical_decode(test_case.fetch("encoded"))
  matching = (test_case["platform_family"] == "unix" && test_case["unit_encoding"] == "unix_bytes") ||
             (test_case["platform_family"] == "windows" && test_case["unit_encoding"] == "windows_utf16le")
  actual = matching && (test_case["platform_family"] == "unix" ? valid_unix?(bytes) : valid_windows?(bytes))
  abort("path verdict mismatch: #{test_case.fetch('name')}") unless actual == test_case.fetch("valid")
end

puts "path vectors passed: #{cases.length}"
