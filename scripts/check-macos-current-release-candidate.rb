#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "json"
require "pathname"

ROOT = Pathname.new(__dir__).join("..").expand_path
CANDIDATE = ROOT.join("platform/macos-vm-feasibility/current-release-candidate-v1.json")
STATUS = ROOT.join("tests/conformance/v1/valid/macos-current-release-candidate-absent.json")

abort "current release candidate path must not be a symlink" if CANDIDATE.symlink?
if CANDIDATE.exist?
  abort "a current candidate exists but ADR-0122 has no approved candidate lineage; create and approve the release-candidate contract before validation"
end

status = JSON.parse(STATUS.read)
abort "current release absent status overclaims authority" unless status.fetch("status") == "release_candidate_absent" && %w[candidate_manifest_present current_source_verified historical_result_accepted release_admitted publication_authorized authority_added].none? { |key| status.fetch(key) }

if ARGV == ["--ordinary-status"]
  puts JSON.generate(status)
  exit 0
end

abort "current release gate denied: release_candidate_absent"
