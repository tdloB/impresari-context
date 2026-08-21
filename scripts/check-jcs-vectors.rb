#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"

class DuplicateRejectingHash < Hash
  def []=(key, value)
    raise JSON::ParserError, "duplicate key: #{key}" if key?(key)
    super
  end
end

def valid_domain?(value)
  case value
  when Hash then value.all? { |key, child| key.valid_encoding? && valid_domain?(child) }
  when Array then value.all? { |child| valid_domain?(child) }
  when Integer then value.between?(-9_007_199_254_740_991, 9_007_199_254_740_991)
  when Float then value.finite?
  when String then value.valid_encoding?
  else true
  end
end

def canonical(value)
  case value
  when Hash
    "{" + value.keys.sort.map { |key| "#{JSON.generate(key)}:#{canonical(value.fetch(key))}" }.join(",") + "}"
  when Array then "[" + value.map { |child| canonical(child) }.join(",") + "]"
  else JSON.generate(value)
  end
end

root = Pathname.new(__dir__).join("..").expand_path
cases = JSON.parse(root.join("tests/conformance/v1/jcs-vectors.json").read).fetch("cases")

cases.each do |test_case|
  raw = test_case.fetch("input")
  value = JSON.parse(raw, object_class: DuplicateRejectingHash)
  actual = valid_domain?(value) && !raw.match?(/(?<![0-9.])-0(?![0-9.eE])/)
  if actual && test_case["canonical"]
    abort("canonical mismatch: #{test_case.fetch('name')}") unless canonical(value) == test_case.fetch("canonical")
  end
rescue JSON::ParserError, JSON::GeneratorError
  actual = false
ensure
  abort("JCS verdict mismatch: #{test_case.fetch('name')}") unless actual == test_case.fetch("valid")
end

puts "JCS vectors passed: #{cases.length}"
