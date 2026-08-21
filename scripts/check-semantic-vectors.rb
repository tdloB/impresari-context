#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
vectors = JSON.parse(root.join("tests/conformance/v1/semantic-vectors.json").read).fetch("cases")
decimal = /\A(?:0|[1-9][0-9]*)\z/

vectors.each do |test_case|
  actual = case test_case.fetch("kind")
           when "span"
             test_case.fetch("start").match?(decimal) && test_case.fetch("end").match?(decimal) &&
               test_case.fetch("start").to_i <= test_case.fetch("end").to_i
           when "accounting"
             %w[reserved delivered requested].all? { |key| test_case.fetch(key).match?(decimal) } &&
               test_case.fetch("reserved").to_i <= test_case.fetch("delivered").to_i &&
               test_case.fetch("delivered").to_i <= test_case.fetch("requested").to_i
           when "decimal"
             test_case.fetch("value").match?(decimal)
           when "line_span"
             start_position = [test_case.fetch("start_line").to_i, test_case.fetch("start_column").to_i]
             end_position = [test_case.fetch("end_line").to_i, test_case.fetch("end_column").to_i]
             (start_position <=> end_position) <= 0
           else
             abort("unknown semantic vector kind")
           end
  abort("semantic verdict mismatch: #{test_case.fetch('name')}") unless actual == test_case.fetch("valid")
end

puts "semantic vectors passed: #{vectors.length}"
