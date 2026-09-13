#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

# Hook recipes are copied into a client by hand, so a recipe that could block,
# approve, rewrite a tool call, reach the network, or leave the project would
# carry authority Impresari does not hold. This check holds every recipe to the
# add-context events, the project scope, a short timeout, and its published
# checksum.

require "digest"
require "json"

ROOT = File.expand_path("..", __dir__)
DIR = File.join(ROOT, "templates", "hooks", "claude")
SCRIPT = "impresari-context-after-compaction.sh"
ADD_CONTEXT_EVENTS = %w[SessionStart UserPromptSubmit].freeze
SESSION_START_MATCHERS = %w[startup resume clear compact fork].freeze
PROJECT_COMMAND = %r{\A"\$CLAUDE_PROJECT_DIR"/\.claude/hooks/[a-z0-9-]+\.sh\z}
FORBIDDEN = [
  /permissionDecision|"decision"|\bblock\b|continue"\s*:\s*false/i,
  /\bexit\s+[1-9]/,
  %r{https?://}i,
  /\b(?:curl|wget|nc|ssh|scp|rsync|python|ruby|node|impresari-context)\b/,
  /~\/|\$HOME|\/\.claude\/settings/,
].freeze

def fail!(message)
  abort("hook recipe check failed: #{message}")
end

script_path = File.join(DIR, SCRIPT)
fail!("missing #{SCRIPT}") unless File.file?(script_path)
script = File.read(script_path, encoding: "UTF-8")
fail!("#{SCRIPT} is larger than 4096 bytes") if script.bytesize > 4096
fail!("#{SCRIPT} lacks its ownership marker") unless script.include?("ownership=exact_fixed_artifact:impresari-context")
fail!("#{SCRIPT} does not end by exiting 0") unless script.rstrip.end_with?("exit 0")
body = script.lines.reject { |line| line.start_with?("#") }.join
FORBIDDEN.each do |pattern|
  fail!("#{SCRIPT} contains forbidden content #{pattern.inspect}") if body.match?(pattern)
end

sums = File.read(File.join(DIR, "SHA256SUMS"), encoding: "UTF-8").lines.map(&:split)
recorded = sums.find { |_, name| name == "claude/#{SCRIPT}" }&.first
fail!("SHA256SUMS does not list claude/#{SCRIPT}") unless recorded
fail!("#{SCRIPT} does not match its published checksum") unless recorded == Digest::SHA256.file(script_path).hexdigest

settings = JSON.parse(File.read(File.join(DIR, "settings.fragment.json")))
hooks = settings.fetch("hooks")
fail!("settings fragment names no hook event") if hooks.empty?
hooks.each do |event, entries|
  fail!("#{event} is not an add-context event") unless ADD_CONTEXT_EVENTS.include?(event)
  entries.each do |entry|
    matcher = entry.fetch("matcher", "")
    if event == "SessionStart" && !SESSION_START_MATCHERS.include?(matcher)
      fail!("SessionStart matcher #{matcher.inspect} is not a documented source")
    end
    entry.fetch("hooks").each do |hook|
      fail!("#{event} hook is not a command") unless hook["type"] == "command"
      fail!("#{event} command leaves the project's .claude/hooks") unless hook["command"].to_s.match?(PROJECT_COMMAND)
      timeout = hook["timeout"]
      fail!("#{event} hook needs a timeout of at most 30 seconds") unless timeout.is_a?(Integer) && timeout.between?(1, 30)
    end
  end
end

puts "hook recipe checks passed: #{hooks.values.flatten.length} entries, checksum verified"
