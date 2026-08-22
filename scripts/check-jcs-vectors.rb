#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"

def skip_space(raw, index)
  index += 1 while index < raw.bytesize && [9, 10, 13, 32].include?(raw.getbyte(index))
  index
end

def scan_string(raw, index)
  raise JSON::ParserError, "expected string" unless raw.getbyte(index) == 34
  cursor = index + 1
  escaped = false
  while cursor < raw.bytesize
    byte = raw.getbyte(cursor)
    unless escaped
      return [JSON.parse(raw.byteslice(index..cursor)), cursor + 1] if byte == 34
      raise JSON::ParserError, "control character" if byte < 32
    end
    escaped = !escaped && byte == 92
    cursor += 1
  end
  raise JSON::ParserError, "unterminated string"
end

def scan_value(raw, index)
  index = skip_space(raw, index)
  case raw.getbyte(index)
  when 34
    _, index = scan_string(raw, index)
    [index, false]
  when 91
    index = skip_space(raw, index + 1)
    return [index + 1, false] if raw.getbyte(index) == 93
    duplicate = false
    loop do
      index, child_duplicate = scan_value(raw, index)
      duplicate ||= child_duplicate
      index = skip_space(raw, index)
      break [index + 1, duplicate] if raw.getbyte(index) == 93
      raise JSON::ParserError, "expected comma" unless raw.getbyte(index) == 44
      index += 1
    end
  when 123
    index = skip_space(raw, index + 1)
    return [index + 1, false] if raw.getbyte(index) == 125
    keys = {}
    duplicate = false
    loop do
      key, index = scan_string(raw, index)
      duplicate ||= keys.key?(key)
      keys[key] = true
      index = skip_space(raw, index)
      raise JSON::ParserError, "expected colon" unless raw.getbyte(index) == 58
      index, child_duplicate = scan_value(raw, index + 1)
      duplicate ||= child_duplicate
      index = skip_space(raw, index)
      break [index + 1, duplicate] if raw.getbyte(index) == 125
      raise JSON::ParserError, "expected comma" unless raw.getbyte(index) == 44
      index = skip_space(raw, index + 1)
    end
  else
    finish = index
    finish += 1 while finish < raw.bytesize && ![9, 10, 13, 32, 44, 93, 125].include?(raw.getbyte(finish))
    raise JSON::ParserError, "expected scalar" if finish == index
    JSON.parse(raw.byteslice(index...finish))
    [finish, false]
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
  finish, duplicate = scan_value(raw, 0)
  raise JSON::ParserError, "trailing content" unless skip_space(raw, finish) == raw.bytesize
  value = JSON.parse(raw)
  actual = !duplicate && valid_domain?(value) && !raw.match?(/(?<![0-9.])-0(?![0-9.eE])/)
  if actual && test_case["canonical"]
    abort("canonical mismatch: #{test_case.fetch('name')}") unless canonical(value) == test_case.fetch("canonical")
  end
rescue JSON::ParserError, JSON::GeneratorError
  actual = false
ensure
  abort("JCS verdict mismatch: #{test_case.fetch('name')}") unless actual == test_case.fetch("valid")
end

puts "JCS vectors passed: #{cases.length}"
