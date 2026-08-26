#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
#
# Native local-scope Claude Code MCP lifecycle rehearsal. It uses an explicit,
# disposable HOME under /private/tmp so `claude mcp add/get/remove` never
# touches the user's actual Claude configuration. The model-directed lifecycle
# and packet-equivalence check remain in rehearse-claude-code.rb.

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
DEFAULT_CLAUDE = "/Users/aaronboldt/.local/bin/claude"
SERVER = "impresari-context"
FIXTURE_NAME = "__impresari_claude_native_local_probe__.ts"
FIXTURE_CONTENT = "export const __impresari_claude_native_local_probe__ = true;\n"

options = {
  claude: DEFAULT_CLAUDE,
  mcp: ROOT.join("target/debug/impresari-context-mcp").to_s,
  prepared_root: nil,
  temporary_root: nil,
  apply: false,
}

OptionParser.new do |parser|
  parser.banner = "Usage: scripts/rehearse-claude-native-local-scope.rb [options]"
  parser.on("--claude PATH", "Claude Code CLI executable") { |value| options[:claude] = value }
  parser.on("--mcp PATH", "Impresari MCP executable") { |value| options[:mcp] = value }
  parser.on("--prepare-root PATH", "Preview or prepare a disposable root under /private/tmp") do |value|
    options[:prepared_root] = value
  end
  parser.on("--temporary-root PATH", "Run against a prepared disposable root under /private/tmp") do |value|
    options[:temporary_root] = value
  end
  parser.on("--apply", "Apply the explicit disposable-root preparation") { options[:apply] = true }
end.parse!

[options[:claude], options[:mcp]].each do |path|
  abort("missing executable: #{path}") unless File.file?(path) && File.executable?(path)
end

def temporary_root(path, allow_absent: false)
  candidate = Pathname.new(path).expand_path
  parent = Pathname.new("/private/tmp").realpath
  resolved = if candidate.exist?
               candidate.realpath
             elsif allow_absent
               candidate.parent.realpath.join(candidate.basename)
             else
               abort("prepared root does not exist: #{candidate}")
             end
  abort("prepared root must be under /private/tmp") unless resolved.to_s.start_with?("#{parent}/")
  abort("prepared root must not be a symbolic link") if candidate.symlink?
  resolved
end

def layout(root)
  {
    home: root.join("home"),
    workspace: root.join("workspace"),
    cache: root.join("cache"),
  }
end

def source_digest(root)
  Digest::SHA256.hexdigest(
    Dir.glob(File.join(root, "**", "*"), File::FNM_DOTMATCH)
       .select { |path| File.file?(path) }
       .sort
       .map { |path| "#{path.delete_prefix(root)}\t#{Digest::SHA256.file(path).hexdigest}\n" }
       .join,
  )
end

abort("choose either --prepare-root or --temporary-root, not both") if options[:prepared_root] && options[:temporary_root]

if options[:prepared_root]
  root = temporary_root(options[:prepared_root], allow_absent: true)
  paths = layout(root)
  if !options[:apply]
    puts JSON.generate({
      "status" => "preview_ready",
      "operation" => "prepare_disposable_claude_native_local_scope",
      "root" => root.to_s,
      "home" => paths.fetch(:home).to_s,
      "workspace" => paths.fetch(:workspace).to_s,
      "cache" => paths.fetch(:cache).to_s,
      "external_write_performed" => false,
      "next_step" => "Review the paths, then rerun with --apply. The subsequent rehearsal writes only the named entry in the reported disposable home.",
    })
    exit(0)
  end
  abort("prepared root already exists: #{root}") if root.exist?
  FileUtils.mkdir_p(paths.values)
  puts JSON.generate({
    "status" => "prepared",
    "operation" => "prepare_disposable_claude_native_local_scope",
    "root" => root.to_s,
    "home" => paths.fetch(:home).to_s,
    "workspace" => paths.fetch(:workspace).to_s,
    "cache" => paths.fetch(:cache).to_s,
    "external_write_performed" => true,
    "next_step" => "Run this rehearsal with --temporary-root #{root}.",
  })
  exit(0)
end

abort("--temporary-root is required for the native local-scope rehearsal") unless options[:temporary_root]
root = temporary_root(options[:temporary_root])
paths = layout(root)
paths.each_value { |path| abort("prepared layout is missing: #{path}") unless path.directory? }
abort("disposable Claude home must be empty before registration") unless Dir.children(paths.fetch(:home)).empty?
abort("disposable Claude workspace must be empty before registration") unless Dir.children(paths.fetch(:workspace)).empty?

workspace = paths.fetch(:workspace).to_s
cache = paths.fetch(:cache).to_s
home = paths.fetch(:home).to_s
fixture = File.join(workspace, FIXTURE_NAME)
File.write(fixture, FIXTURE_CONTENT)
before = source_digest(workspace)

environment = { "HOME" => home }
server_args = [
  options[:mcp],
  "--workspace", workspace,
  "--cache", cache,
  "--consumer-id", "consumer_claude_managed",
  "--role", "local_user",
]

registered = false
begin
  add_stdout, add_stderr, add_status = Open3.capture3(
    environment, options[:claude], "mcp", "add", "--scope", "local", SERVER, "--", *server_args,
    chdir: workspace,
  )
  abort("Claude Code native local-scope add failed:\n#{add_stderr}\n#{add_stdout}") unless add_status.success?
  registered = true

  get_stdout, get_stderr, get_status = Open3.capture3(
    environment, options[:claude], "mcp", "get", SERVER, chdir: workspace,
  )
  abort("Claude Code did not recognize the native local entry:\n#{get_stderr}\n#{get_stdout}") unless get_status.success?
  expected = ["Scope: Local config", "Status: ✔ Connected", "Type: stdio", "Command: #{options[:mcp]}"]
  missing = expected.reject { |value| get_stdout.include?(value) }
  abort("Claude Code native local entry did not retain the fixed contract: #{missing.join(', ')}") unless missing.empty?
  abort("source workspace changed during native local registration") unless before == source_digest(workspace)
ensure
  if registered
    remove_stdout, remove_stderr, remove_status = Open3.capture3(
      environment, options[:claude], "mcp", "remove", "--scope", "local", SERVER, chdir: workspace,
    )
    abort("Claude Code native local-scope removal failed:\n#{remove_stderr}\n#{remove_stdout}") unless remove_status.success?
    _absent_stdout, absent_stderr, absent_status = Open3.capture3(
      environment, options[:claude], "mcp", "get", SERVER, chdir: workspace,
    )
    abort("Claude Code retained the removed native local entry:\n#{absent_stderr}") if absent_status.success?
  end
end

abort("source workspace changed during native local removal") unless before == source_digest(workspace)
abort("refusing to remove an unexpected Claude rehearsal fixture") unless File.file?(fixture) && File.binread(fixture) == FIXTURE_CONTENT
File.delete(fixture)

puts JSON.generate({
  "status" => "passed",
  "claude" => options[:claude],
  "configuration_scope" => "local",
  "configuration_home" => home,
  "native_add_get_remove" => true,
  "source_immutable" => true,
  "persistent_user_registration" => false,
})
