#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "pathname"

path = Pathname.new(ARGV.fetch(0)).expand_path
abort "missing or symlinked YARA-X rule source" unless path.file? && !path.symlink?
abort "YARA-X rule source exceeds 262144 bytes" if path.size > 262_144

source = path.read(encoding: "UTF-8")
abort "YARA-X rule source is not valid UTF-8" unless source.valid_encoding?
body = source.lines.reject { |line| line.lstrip.start_with?("//") }.join

forbidden = {
  "import" => /^\s*import\b/,
  "include" => /^\s*include\b/,
  "regular expression" => %r{=\s*/},
  "base64 modifier" => /\bbase64(?:wide)?\b/i,
  "xor modifier" => /\bxor\b/i,
  "external variable" => /\bextern(?:al)?\b/i,
  "module reference" => /\b(?:pe|elf|macho|dotnet|crx|hash|math|time|vt|zip)\s*\./i
}
forbidden.each do |label, pattern|
  abort "YARA-X rule policy rejected #{label}" if body.match?(pattern)
end

identifiers = body.scan(/^\s*rule\s+([a-z][a-z0-9_]*)\s*:/).flatten
abort "YARA-X rule policy requires one to 256 unique rules" unless
  identifiers.length.between?(1, 256) && identifiers.uniq.length == identifiers.length
abort "YARA-X rule identifier exceeds 128 bytes" if identifiers.any? { |identifier| identifier.bytesize > 128 }

string_lines = body.lines.grep(/^\s*\$[a-z][a-z0-9_]*\s*=/)
abort "YARA-X rule policy requires one to 32 patterns per rule" if
  string_lines.empty? || string_lines.length > identifiers.length * 32
string_lines.each do |line|
  literal = line.match?(/^\s*\$[a-z][a-z0-9_]*\s*=\s*"[\x20-\x7e]+"(?:\s+(?:ascii|wide|fullword|nocase))*\s*$/)
  hex = line.match?(/^\s*\$[a-z][a-z0-9_]*\s*=\s*\{(?:\s+[0-9A-F]{2})+\s+\}\s*$/)
  abort "YARA-X rule policy rejected pattern syntax" unless literal || hex
end

allowed_condition = /\A[\s\$a-z0-9_().<>=!&|+-]+\z/i
conditions = body.scan(/\bcondition:\s*(.*?)\s*\}/m).flatten
abort "YARA-X rule policy requires one condition per rule" unless conditions.length == identifiers.length
conditions.each do |condition|
  abort "YARA-X rule policy rejected condition syntax" unless condition.match?(allowed_condition)
end

puts "YARA-X rule policy verified: rules=#{identifiers.length} patterns=#{string_lines.length}"
