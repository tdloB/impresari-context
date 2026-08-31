#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "json"
require "open3"
require "optparse"
require "tempfile"

class ContractError < StandardError; end

def run!(*command)
  stdout, stderr, status = Open3.capture3(*command)
  raise ContractError, "issue mutation failed" unless status.success?
  stdout
end

def verify_owned_issue!(number, key)
  document = JSON.parse(run!("gh", "issue", "view", number.to_s, "--json", "body,state,labels"))
  labels = document.fetch("labels").map { |label| label.fetch("name") }
  expected = "<!-- impresari-maintenance:#{key} -->"
  raise ContractError, "issue is not exact-owned" unless document.fetch("state") == "OPEN" && labels.include?("impresari-maintenance") && document.fetch("body").include?(expected)
rescue JSON::ParserError, KeyError
  raise ContractError, "issue ownership state is invalid"
end

options = {}
OptionParser.new do |arguments|
  arguments.on("--plan FILE") { |value| options[:plan] = value }
end.parse!

begin
  raise ContractError, "unexpected arguments" unless ARGV.empty?
  raise ContractError, "missing plan" unless options[:plan]
  plan = JSON.parse(File.binread(options.fetch(:plan)))
  raise ContractError, "issue plan schema is unsupported" unless plan["schema_name"] == "roadmap-maintenance-issue-plan" && plan["schema_version"] == "1.0.0" && plan["label"] == "impresari-maintenance"
  raise ContractError, "issue plan has an unsupported shape" unless plan.keys.sort == %w[actions label schema_name schema_version].sort
  actions = plan.fetch("actions")
  raise ContractError, "too many issue actions" unless actions.is_a?(Array) && actions.length <= 40

  applied = actions.map do |action|
    kind = action.fetch("action")
    number = action.fetch("issue_number")
    key = action.fetch("ownership_key")
    raise ContractError, "issue action has an unsupported shape" unless action.keys.sort == %w[action body issue_number ownership_key title].sort
    raise ContractError, "issue ownership key is invalid" unless key.is_a?(String) && key.match?(/\A[a-z0-9-]+:[a-z_]+\z/)
    case kind
    when "noop"
      raise ContractError, "no-op issue number is invalid" unless number.is_a?(Integer) && number.positive?
      verify_owned_issue!(number, key)
      {"action" => kind, "issue_number" => number, "ownership_key" => key}
    when "close"
      raise ContractError, "close issue number is invalid" unless number.is_a?(Integer) && number.positive?
      raise ContractError, "close action contains mutable content" unless action.fetch("title") == "" && action.fetch("body") == ""
      verify_owned_issue!(number, key)
      run!("gh", "issue", "close", number.to_s, "--reason", "completed")
      {"action" => kind, "issue_number" => number, "ownership_key" => key}
    when "create", "update"
      title = action.fetch("title")
      body = action.fetch("body")
      expected_marker = "<!-- impresari-maintenance:#{key} -->"
      raise ContractError, "issue title is invalid" unless title.is_a?(String) && title.start_with?("[maintenance] ") && title.length <= 180
      raise ContractError, "issue body is invalid" unless body.is_a?(String) && body.start_with?(expected_marker) && body.bytesize <= 8_192
      Tempfile.create(["impresari-maintenance-", ".md"]) do |file|
        file.write(body)
        file.flush
        if kind == "create"
          raise ContractError, "create issue number must be zero" unless number == 0
          output = run!("gh", "issue", "create", "--title", title, "--body-file", file.path, "--label", "impresari-maintenance")
          created = output[%r{/issues/([0-9]+)\s*\z}, 1]
          raise ContractError, "created issue identity unavailable" unless created
          number = Integer(created, 10)
        else
          raise ContractError, "update issue number is invalid" unless number.is_a?(Integer) && number.positive?
          verify_owned_issue!(number, key)
          run!("gh", "issue", "edit", number.to_s, "--title", title, "--body-file", file.path)
        end
      end
      {"action" => kind, "issue_number" => number, "ownership_key" => key}
    else
      raise ContractError, "issue action is unsupported"
    end
  end
  puts JSON.pretty_generate({"schema_name" => "roadmap-maintenance-issue-application", "schema_version" => "1.0.0", "applied" => applied})
rescue ContractError, JSON::ParserError, KeyError, OptionParser::ParseError => error
  warn "error: #{error.message}"
  exit 1
end
