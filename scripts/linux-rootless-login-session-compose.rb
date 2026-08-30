#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "optparse"
require "pathname"
require_relative "lib/linux_rootless_login_session_rehearsal"

MAX_INPUT_BYTES = 256 * 1024

def load_bounded(path, label)
  candidate = Pathname.new(path).expand_path
  raise LinuxRootlessLoginSessionRehearsal::ContractError, "#{label} must be a regular file" unless candidate.file? && !candidate.symlink?
  raise LinuxRootlessLoginSessionRehearsal::ContractError, "#{label} exceeds the byte ceiling" if candidate.size > MAX_INPUT_BYTES
  bytes = candidate.binread
  [bytes, JSON.parse(bytes)]
rescue JSON::ParserError => error
  raise LinuxRootlessLoginSessionRehearsal::ContractError, "#{label} is invalid JSON: #{error.message}"
end

options = {}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby scripts/linux-rootless-login-session-compose.rb --expected-source-sha SHA --package FILE --first-session FILE --second-session FILE --cleanup FILE"
  parser.on("--expected-source-sha SHA") { |value| options[:expected_source] = value }
  parser.on("--package FILE") { |value| options[:package] = value }
  parser.on("--first-session FILE") { |value| options[:first] = value }
  parser.on("--second-session FILE") { |value| options[:second] = value }
  parser.on("--cleanup FILE") { |value| options[:cleanup] = value }
end.parse!

begin
  missing = %i[expected_source package first second cleanup].reject { |key| options[key] && !options[key].empty? }
  raise LinuxRootlessLoginSessionRehearsal::ContractError, "missing required arguments: #{missing.join(', ')}" unless missing.empty? && ARGV.empty?
  package_bytes, package = load_bounded(options.fetch(:package), "package receipt")
  _first_bytes, first = load_bounded(options.fetch(:first), "first session")
  _second_bytes, second = load_bounded(options.fetch(:second), "second session")
  _cleanup_bytes, cleanup = load_bounded(options.fetch(:cleanup), "cleanup observation")
  receipt = LinuxRootlessLoginSessionRehearsal.build(
    expected_source: options.fetch(:expected_source),
    package_bytes: package_bytes,
    package: package,
    first: first,
    second: second,
    cleanup: cleanup,
  )
  puts JSON.pretty_generate(receipt)
rescue LinuxRootlessLoginSessionRehearsal::ContractError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
