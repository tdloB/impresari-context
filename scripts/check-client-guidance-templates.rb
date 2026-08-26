#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

root = File.expand_path("..", __dir__)
templates = {
  "templates/client-guidance/codex/AGENTS.md" => ["Impresari Context", "packet ID"],
  "templates/client-guidance/claude/SKILL.md" => ["description:", "packet ID"],
  "templates/client-guidance/cursor/impresari-context.mdc" => ["description:", "alwaysApply: false", "packet ID"],
  "templates/client-guidance/copilot/impresari-context.instructions.md" => ["applyTo:", "packet ID"],
}
forbidden = [
  /https?:\/\//i,
  /\b(?:curl|wget|bash|zsh|powershell|cmd\.exe)\b/i,
  /\b(?:env|environment)\s*(?:=|forward)/i,
  /\b(?:auto(?:matic)?[- ]?approv|approve[- ]?all|trust[- ]?folder)\b/i,
  /\b(?:install|enable)\s+(?:an?\s+)?MCP\b/i,
]

templates.each do |relative, required|
  path = File.join(root, relative)
  abort("missing client guidance template: #{relative}") unless File.file?(path)
  text = File.read(path, encoding: "UTF-8")
  abort("oversized client guidance template: #{relative}") if text.bytesize > 16_384
  unless text.include?("ownership=exact_fixed_artifact:impresari-context")
    abort("client guidance template lacks exact ownership marker: #{relative}")
  end
  required.each do |needle|
    abort("client guidance template lacks #{needle.inspect}: #{relative}") unless text.include?(needle)
  end
  forbidden.each do |pattern|
    abort("client guidance template has forbidden authority content #{pattern.inspect}: #{relative}") if text.match?(pattern)
  end
end

puts "client guidance template checks passed: #{templates.length} templates"
