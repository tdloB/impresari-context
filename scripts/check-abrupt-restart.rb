#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "open3"
require "tmpdir"

root = File.expand_path("..", __dir__)
build = system("cargo", "build", "-q", "-p", "context-evaluation", "--bins", "--locked", "--offline", chdir: root)
abort "fault helper build failed" unless build
helper_name = Gem.win_platform? ? "cache-fault-helper.exe" : "cache-fault-helper"
helper = File.join(root, "target", "debug", helper_name)

Dir.mktmpdir("impresari-abrupt-restart-") do |cache|
  abort "cache initialization failed" unless system(helper, "initialize", cache)
  stdin, stdout, stderr, wait = Open3.popen3(helper, "hold", cache)
  stdin.close
  ready = stdout.gets
  unless ready == "READY\n"
    Process.kill("KILL", wait.pid) rescue nil
    abort "helper did not acquire the cache: #{stderr.read}"
  end
  Process.kill("KILL", wait.pid)
  wait.value
  stdout.close
  stderr.close
  abort "cache did not recover after abrupt termination" unless system(helper, "verify", cache)
end

puts "abrupt cache restart check passed"
