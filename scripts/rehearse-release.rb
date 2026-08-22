#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "json"
require "open3"
require "tmpdir"

archive = File.expand_path(ARGV.fetch(0) { abort "usage: rehearse-release.rb ARCHIVE" })
checksum_path = "#{archive}.sha256"
abort "missing archive or checksum" unless File.file?(archive) && File.file?(checksum_path)
expected = File.read(checksum_path).split.first
abort "archive checksum mismatch" unless expected == Digest::SHA256.file(archive).hexdigest

Dir.mktmpdir("impresari-release-rehearsal-") do |directory|
  output, status = Open3.capture2e(
    "tar", "-xzf", File.basename(archive), "-C", directory, chdir: File.dirname(archive)
  )
  abort output unless status.success?
  root = Dir.glob(File.join(directory, "impresari-context-*")).fetch(0)
  manifest = JSON.parse(File.read(File.join(root, "MANIFEST.json")))
  manifest.fetch("files").each do |entry|
    path = File.join(root, entry.fetch("path"))
    abort "missing packaged file" unless File.file?(path)
    abort "packaged size mismatch" unless File.size(path).to_s == entry.fetch("bytes")
    abort "packaged digest mismatch" unless Digest::SHA256.file(path).hexdigest == entry.fetch("sha256")
  end
  exe = Gem.win_platform? ? ".exe" : ""
  cli_out, = Open3.capture2e({ "HOME" => directory }, File.join(root, "bin", "impresari-context#{exe}"))
  JSON.parse(cli_out.lines.find { |line| line.lstrip.start_with?("{") } || abort("CLI emitted no machine JSON"))
  mcp = File.join(root, "bin", "impresari-context-mcp#{exe}")
  mcp_stdout, mcp_stderr, mcp_status = Open3.capture3({ "HOME" => directory }, mcp)
  abort "MCP wrote non-protocol startup output" unless mcp_stdout.empty?
  abort "MCP missing safe usage failure" if mcp_status.success? || mcp_stderr.empty?

  workspace = File.join(directory, "workspace")
  cache = File.join(directory, "cache")
  Dir.mkdir(workspace)
  File.write(File.join(workspace, "source.rs"), "fn verified_source() {}\n")
  protocol = [
    { jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-11-25", capabilities: {}, clientInfo: { name: "release-rehearsal", version: "1" } } },
    { jsonrpc: "2.0", method: "notifications/initialized" },
    { jsonrpc: "2.0", id: 2, method: "tools/list" }
  ].map(&:to_json).join("\n") + "\n"
  live_stdout, live_stderr, live_status = Open3.capture3(
    { "HOME" => directory }, mcp,
    "--workspace", workspace, "--cache", cache,
    "--consumer-id", "consumer_release_rehearsal", "--role", "release_rehearsal",
    "--occurred-at", "2026-08-22T00:00:00Z", stdin_data: protocol
  )
  abort "MCP clean launch failed: #{live_stderr}" unless live_status.success?
  responses = live_stdout.lines.map { |line| JSON.parse(line) }
  abort "MCP clean launch returned unexpected response count" unless responses.length == 2
  abort "MCP tools unavailable after initialization" unless responses.last.dig("result", "tools")&.length == 4
end

puts "release candidate rehearsal passed: #{File.basename(archive)}"
