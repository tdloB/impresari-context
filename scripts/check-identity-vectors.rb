#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0

require "digest"
require "json"
require "pathname"

root = Pathname.new(__dir__).join("..").expand_path
document = JSON.parse(root.join("tests/conformance/v1/identity-vectors.json").read)

document.fetch("vectors").each do |vector|
  preimage = ["impresari-context", vector.fetch("object_kind"), "1.0.0", vector.fetch("canonical_payload")].join("\0")
  abort("preimage mismatch: #{vector.fetch('object_kind')}") unless preimage.unpack1("H*") == vector.fetch("preimage_hex")
  digest = "sha256:#{Digest::SHA256.hexdigest(preimage)}"
  abort("digest mismatch: #{vector.fetch('object_kind')}") unless digest == vector.fetch("digest")
end

puts "identity vectors passed: #{document.fetch('vectors').length}"
