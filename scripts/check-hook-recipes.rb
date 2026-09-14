#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

# Hook recipes are copied into a client by hand, so a recipe that could block,
# approve, rewrite a tool call, reach the network, or leave the project would
# carry authority Impresari does not hold. This check holds every recipe to its
# own events and script, the project scope, a short timeout, and its published
# checksum. Only the output recipe may call Impresari, only through two fixed
# lines, and only after a Bash run of one named test or build runner.

require "digest"
require "json"

ROOT = File.expand_path("..", __dir__)
DIR = File.join(ROOT, "templates", "hooks", "claude")
SESSION_START_MATCHERS = %w[startup resume clear compact fork].freeze
PROJECT_COMMAND = %r{\A"\$CLAUDE_PROJECT_DIR"/\.claude/hooks/([a-z0-9-]+\.sh)\z}
RUNNER_FILTER = /\ABash\([a-z0-9][a-z0-9 ._-]* \*\)\z/
FORBIDDEN = [
  /permissionDecision|"decision"|\bblock\b|continue"\s*:\s*false/i,
  /\bexit\s+[1-9]/,
  %r{https?://}i,
  /\b(?:curl|wget|nc|ssh|scp|rsync|python|ruby|node|impresari-context)\b/,
  /~\/|\$HOME|\/\.claude\/settings/,
].freeze
RECIPES = [
  {
    script: "impresari-context-after-compaction.sh",
    fragment: "after-compaction.settings.fragment.json",
    events: %w[SessionStart UserPromptSubmit],
    fixed_lines: [],
  },
  {
    script: "impresari-context-reduce-output.sh",
    fragment: "reduce-output.settings.fragment.json",
    events: %w[PostToolUse],
    fixed_lines: [
      'bin=$(command -v impresari-context) || { cat >/dev/null; exit 0; }',
      'env -i "$bin" hook claude-code post-tool-use || true',
    ],
  },
].freeze

def fail!(message)
  abort("hook recipe check failed: #{message}")
end

sums = File.read(File.join(DIR, "SHA256SUMS"), encoding: "UTF-8").lines.map(&:split)
listed = sums.map { |_, name| name }.sort
owned = RECIPES.map { |recipe| "claude/#{recipe[:script]}" }.sort
fail!("SHA256SUMS must list exactly the recipe scripts") unless listed == owned

entries = 0
RECIPES.each do |recipe|
  name = recipe[:script]
  script_path = File.join(DIR, name)
  fail!("missing #{name}") unless File.file?(script_path)
  script = File.read(script_path, encoding: "UTF-8")
  fail!("#{name} is larger than 4096 bytes") if script.bytesize > 4096
  fail!("#{name} lacks its ownership marker") unless script.include?("ownership=exact_fixed_artifact:impresari-context")
  fail!("#{name} does not end by exiting 0") unless script.rstrip.end_with?("exit 0")
  lines = script.lines.map(&:chomp).reject { |line| line.start_with?("#") }
  recipe[:fixed_lines].each do |fixed|
    fail!("#{name} must hold #{fixed.inspect} exactly once") unless lines.count(fixed) == 1
  end
  body = (lines - recipe[:fixed_lines]).join("\n")
  FORBIDDEN.each do |pattern|
    fail!("#{name} contains forbidden content #{pattern.inspect}") if body.match?(pattern)
  end
  recorded = sums.find { |_, listed_name| listed_name == "claude/#{name}" }&.first
  fail!("#{name} does not match its published checksum") unless recorded == Digest::SHA256.file(script_path).hexdigest

  hooks = JSON.parse(File.read(File.join(DIR, recipe[:fragment]))).fetch("hooks")
  fail!("#{recipe[:fragment]} names no hook event") if hooks.empty?
  hooks.each do |event, event_entries|
    fail!("#{event} is not an event #{name} may use") unless recipe[:events].include?(event)
    event_entries.each do |entry|
      matcher = entry.fetch("matcher", "")
      if event == "SessionStart" && !SESSION_START_MATCHERS.include?(matcher)
        fail!("SessionStart matcher #{matcher.inspect} is not a documented source")
      end
      fail!("PostToolUse matcher #{matcher.inspect} is not Bash") if event == "PostToolUse" && matcher != "Bash"
      entry.fetch("hooks").each do |hook|
        entries += 1
        fail!("#{event} hook is not a command") unless hook["type"] == "command"
        command = hook["command"].to_s.match(PROJECT_COMMAND)
        fail!("#{event} command leaves the project's .claude/hooks") unless command
        fail!("#{recipe[:fragment]} runs #{command[1]}, not #{name}") unless command[1] == name
        if event == "PostToolUse" && !hook["if"].to_s.match?(RUNNER_FILTER)
          fail!("PostToolUse hook needs an if filter naming one Bash runner, like Bash(pytest *)")
        end
        timeout = hook["timeout"]
        fail!("#{event} hook needs a timeout of at most 30 seconds") unless timeout.is_a?(Integer) && timeout.between?(1, 30)
      end
    end
  end
end

puts "hook recipe checks passed: #{RECIPES.length} recipes, #{entries} entries, checksums verified"
