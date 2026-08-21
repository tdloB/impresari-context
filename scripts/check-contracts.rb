#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
SCHEMA_ROOT = ROOT.join("schemas/v1")
FIXTURE_ROOT = ROOT.join("tests/conformance/v1")
METADATA_KEYS = %w[$schema $id title description $defs].freeze
SUPPORTED_KEYS = (METADATA_KEYS + %w[$ref type const enum pattern format minLength maxLength minItems maxItems uniqueItems required properties additionalProperties items allOf if then dependentRequired]).freeze

def load_json(path)
  JSON.parse(path.read)
rescue JSON::ParserError => e
  abort("invalid JSON: #{path.relative_path_from(ROOT)}: #{e.message}")
end

SCHEMAS = Dir.glob(SCHEMA_ROOT.join("*.schema.json")).to_h do |raw|
  path = Pathname.new(raw)
  [path.basename.to_s, load_json(path)]
end.freeze

def pointer(document, fragment)
  return document if fragment.nil? || fragment.empty?
  abort("unsupported JSON pointer: ##{fragment}") unless fragment.start_with?("/")
  fragment.split("/").drop(1).reduce(document) do |value, token|
    key = token.gsub("~1", "/").gsub("~0", "~")
    abort("unresolved JSON pointer: ##{fragment}") unless value.is_a?(Hash) && value.key?(key)
    value.fetch(key)
  end
end

def resolve(reference, current_name)
  file, fragment = reference.split("#", 2)
  name = file.empty? ? current_name : File.basename(file)
  abort("remote or missing schema reference: #{reference}") unless SCHEMAS.key?(name)
  [pointer(SCHEMAS.fetch(name), fragment), name]
end

def type_matches?(instance, type)
  case type
  when "object" then instance.is_a?(Hash)
  when "array" then instance.is_a?(Array)
  when "string" then instance.is_a?(String)
  when "integer" then instance.is_a?(Integer)
  when "number" then instance.is_a?(Numeric)
  when "boolean" then instance == true || instance == false
  when "null" then instance.nil?
  else false
  end
end

def validate(instance, schema, schema_name, at = "$")
  unknown = schema.keys - SUPPORTED_KEYS
  return ["#{at}: unsupported schema keywords #{unknown.join(', ')}"] unless unknown.empty?

  if schema["$ref"]
    resolved, resolved_name = resolve(schema["$ref"], schema_name)
    return validate(instance, resolved, resolved_name, at)
  end

  errors = []
  errors << "#{at}: wrong type" if schema["type"] && !type_matches?(instance, schema["type"])
  errors << "#{at}: does not equal const" if schema.key?("const") && instance != schema["const"]
  errors << "#{at}: is not in enum" if schema["enum"] && !schema["enum"].include?(instance)

  if instance.is_a?(String)
    errors << "#{at}: below minLength" if schema["minLength"] && instance.length < schema["minLength"]
    errors << "#{at}: above maxLength" if schema["maxLength"] && instance.length > schema["maxLength"]
    errors << "#{at}: pattern mismatch" if schema["pattern"] && !Regexp.new(schema["pattern"]).match?(instance)
  end

  if instance.is_a?(Array)
    errors << "#{at}: below minItems" if schema["minItems"] && instance.length < schema["minItems"]
    errors << "#{at}: above maxItems" if schema["maxItems"] && instance.length > schema["maxItems"]
    errors << "#{at}: items are not unique" if schema["uniqueItems"] && instance.uniq.length != instance.length
    instance.each_with_index { |item, index| errors.concat(validate(item, schema["items"], schema_name, "#{at}[#{index}]")) } if schema["items"]
  end

  if instance.is_a?(Hash)
    Array(schema["required"]).each { |key| errors << "#{at}: missing #{key}" unless instance.key?(key) }
    properties = schema.fetch("properties", {})
    if schema["additionalProperties"] == false
      (instance.keys - properties.keys).each { |key| errors << "#{at}: unknown property #{key}" }
    end
    properties.each { |key, child| errors.concat(validate(instance[key], child, schema_name, "#{at}.#{key}")) if instance.key?(key) }
    schema.fetch("dependentRequired", {}).each do |key, dependencies|
      dependencies.each { |dependency| errors << "#{at}: #{key} requires #{dependency}" unless !instance.key?(key) || instance.key?(dependency) }
    end
  end

  Array(schema["allOf"]).each { |child| errors.concat(validate(instance, child, schema_name, at)) }
  if schema["if"] && validate(instance, schema["if"], schema_name, at).empty? && schema["then"]
    errors.concat(validate(instance, schema["then"], schema_name, at))
  end
  errors
end

ids = SCHEMAS.values.map { |schema| schema["$id"] }
abort("every schema must have a unique $id") if ids.any?(&:nil?) || ids.uniq.length != ids.length

SCHEMAS.each do |name, schema|
  JSON.generate(schema).scan(/"\$ref":"([^"]+)"/).flatten.each { |reference| resolve(reference, name) }
  next if %w[common.schema.json search.schema.json].include?(name)
  abort("open object schema: #{name}") if schema["type"] == "object" && schema["additionalProperties"] != false
end

registry = load_json(SCHEMA_ROOT.join("registry.json"))
registered = registry.fetch("schemas").map { |entry| entry.fetch("path") }.select { |name| name.end_with?(".schema.json") }
abort("schema registry mismatch") unless registered.sort == SCHEMAS.keys.sort

manifest = load_json(FIXTURE_ROOT.join("manifest.json"))
manifest.fetch("cases").each do |test_case|
  fixture_path = FIXTURE_ROOT.join(test_case.fetch("fixture")).cleanpath
  abort("fixture escapes root") unless fixture_path.to_s.start_with?(FIXTURE_ROOT.to_s + File::SEPARATOR)
  instance = load_json(fixture_path)
  schema, schema_name = resolve(test_case.fetch("schema"), "")
  errors = validate(instance, schema, schema_name)
  actual = errors.empty?
  next if actual == test_case.fetch("valid")
  abort("unexpected fixture verdict: #{test_case.fetch('fixture')}: #{errors.join('; ')}")
end

puts "contract checks passed: #{SCHEMAS.length} schemas, #{manifest.fetch('cases').length} fixtures"
