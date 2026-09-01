#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "pathname"
require "rubygems/package"
require "stringio"
require "tmpdir"
require "time"
require "zlib"

ROOT = Pathname.new(__dir__).join("..").expand_path
CONTRACT_RELATIVE = "platform/macos-vm-feasibility/synthetic-guest-payload-contract-v1.json"
CONTRACT_DIGEST = "4e43e28f325d7ab67ff2bb23595eb9273320ff5e8597553b9a681bfdc51033d4"
PROFILE_RELATIVE = "profiles/v1/iar-macos-local-vm-synthetic-guest-materialization-v1.json"
PUBLIC_URL = "https://dl-cdn.alpinelinux.org/alpine/v3.24/main/aarch64/linux-virt-6.18.48-r0.apk"
PUBLIC_BYTES = 41_557_960
PUBLIC_SHA256 = "c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616"
EXPECTED_OUTPUTS = {
  "Image" => [36_175_872, "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5"],
  "impresari-initramfs.gz" => [38_207, "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b"]
}.freeze
EXPECTED_PACKAGE = {
  "pkgname" => "linux-virt",
  "pkgver" => "6.18.48-r0",
  "arch" => "aarch64",
  "commit" => "c83b91e0fde4c1bada9b80d4e67c395b5335597b",
  "datahash" => "e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263"
}.freeze
EXTRACTED_MEMBERS = {
  "boot/vmlinuz-virt" => 64 * 1024 * 1024,
  "lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz" => 2 * 1024 * 1024
}.freeze
MAX_CAPTURE_BYTES = 256 * 1024

abort "usage: materialize-macos-vm-synthetic-guest-candidate.rb" unless ARGV.empty?
abort "macOS guest materialization requires Darwin arm64" unless `uname -s`.strip == "Darwin" && `uname -m`.strip == "arm64"

def sha256(path)
  Digest::SHA256.file(path).hexdigest
end

def exact_repo_file(relative, digest = nil)
  path = ROOT.join(relative).cleanpath
  abort "repository input escapes root: #{relative}" unless path.to_s.start_with?(ROOT.to_s + File::SEPARATOR)
  abort "missing or symlinked repository input: #{relative}" unless path.file? && !path.symlink?
  abort "repository input digest changed: #{relative}" if digest && sha256(path) != digest
  path
end

def run_bounded(env, *command)
  stdout, stderr, status = Open3.capture3(env, *command)
  abort "child output exceeded bound: #{command.first}" if stdout.bytesize + stderr.bytesize > MAX_CAPTURE_BYTES
  abort "bounded child failed: #{command.first}: #{stderr.lines.first.to_s.strip}" unless status.success?
  [stdout, stderr]
end

def apk_segments(path)
  archive = File.binread(path)
  segments = []
  offset = 0
  while offset < archive.bytesize
    abort "unexpected APKv2 segment count" if segments.length >= 3
    source = StringIO.new(archive.byteslice(offset..))
    reader = Zlib::GzipReader.new(source)
    inflated = reader.read
    consumed = source.pos - reader.unused.to_s.bytesize
    abort "invalid APKv2 segment" unless consumed.positive?
    segments << inflated
    offset += consumed
  end
  abort "unexpected APKv2 segment count" unless segments.length == 3
  segments
rescue Zlib::GzipFile::Error, Zlib::Error => e
  abort "invalid APKv2 framing: #{e.message}"
end

def extract_exact_members(data_tar, destination)
  selected = {}
  Gem::Package::TarReader.new(StringIO.new(data_tar)) do |tar|
    tar.each do |entry|
      next unless EXTRACTED_MEMBERS.key?(entry.full_name)
      abort "bounded APK member is not a regular file: #{entry.full_name}" unless entry.file?
      abort "duplicate bounded APK member: #{entry.full_name}" if selected.key?(entry.full_name)
      abort "oversized bounded APK member: #{entry.full_name}" if entry.header.size > EXTRACTED_MEMBERS.fetch(entry.full_name)
      selected[entry.full_name] = entry.read
    end
  end
  abort "bounded APK member set incomplete" unless selected.keys.sort == EXTRACTED_MEMBERS.keys.sort
  selected.each do |name, bytes|
    output = destination.join(name)
    FileUtils.mkdir_p(output.dirname, mode: 0o700)
    File.binwrite(output, bytes)
    File.chmod(0o600, output)
  end
rescue Gem::Package::TarInvalidError => e
  abort "invalid bounded APK data tar: #{e.message}"
end

contract_path = exact_repo_file(CONTRACT_RELATIVE, CONTRACT_DIGEST)
contract = JSON.parse(contract_path.read)
profile_path = exact_repo_file(PROFILE_RELATIVE)
profile = JSON.parse(profile_path.read)
abort "materialization profile does not bind the frozen payload contract" unless
  profile.fetch("contract_path") == CONTRACT_RELATIVE &&
    profile.fetch("contract_digest") == "sha256:#{CONTRACT_DIGEST}"

recipe = contract.fetch("future_materialization_recipe")
recipe.fetch("build_inputs").each do |input|
  exact_repo_file(input.fetch("path"), input.fetch("sha256").delete_prefix("sha256:"))
end
key_path = exact_repo_file(
  recipe.dig("public_input", "verification_key_path"),
  recipe.dig("public_input", "verification_key_sha256").delete_prefix("sha256:")
)
zig_version, = run_bounded({}, "zig", "version")
abort "unexpected Zig version" unless zig_version.strip == recipe.fetch("zig_version")

root = Dir.mktmpdir("impresari-macos-guest-materialization-")
File.chmod(0o700, root)
root_path = Pathname.new(root)
result = nil
begin
  download = root_path.join("linux-virt.apk")
  partial = root_path.join("linux-virt.apk.partial")
  run_bounded({}, "curl", "--fail", "--silent", "--show-error", "--max-redirs", "0",
              "--proto", "=https", "--tlsv1.2", "--connect-timeout", "10", "--max-time", "120",
              "--output", partial.to_s, PUBLIC_URL)
  File.rename(partial, download)
  abort "downloaded APK size changed" unless download.size == PUBLIC_BYTES
  abort "downloaded APK digest changed" unless sha256(download) == PUBLIC_SHA256

  verification_stdout, = run_bounded(
    {}, "ruby", exact_repo_file("scripts/verify-alpine-apkv2.rb").to_s,
    "package", download.to_s, key_path.to_s
  )
  verification = JSON.parse(verification_stdout)
  abort "publisher authentication failed" unless
    verification.fetch("rsa_sha1_signature_verified") && verification.fetch("datahash_verified") &&
      EXPECTED_PACKAGE.all? { |key, value| verification.fetch("package").fetch(key) == value }

  extracted = root_path.join("extracted")
  FileUtils.mkdir_p(extracted, mode: 0o700)
  extract_exact_members(apk_segments(download).fetch(2), extracted)

  compressed_kernel = extracted.join("boot/vmlinuz-virt")
  compressed_module = extracted.join("lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz")
  module_path = root_path.join("virtio_blk.ko")
  Zlib::GzipReader.open(compressed_module.to_s) do |gzip|
    module_bytes = gzip.read(2 * 1024 * 1024 + 1)
    abort "inflated module exceeded bound" if module_bytes.bytesize > 2 * 1024 * 1024
    File.binwrite(module_path, module_bytes)
  end
  abort "module identity changed" unless
    module_path.size == 49_687 && sha256(module_path) == "c8eb0f6b98a18a5cc237bc3019637551f46f964a5efd215253a0946889e3f31d"

  payload = root_path.join("payload")
  build = root_path.join("build")
  FileUtils.mkdir_p(payload, mode: 0o700)
  FileUtils.mkdir_p(build, mode: 0o700)
  image = payload.join("Image")
  run_bounded({}, "ruby", exact_repo_file("scripts/extract-macos-vm-kernel.rb").to_s,
              compressed_kernel.to_s, image.to_s)

  guest_init = build.join("init")
  zig_env = {
    "ZIG_GLOBAL_CACHE_DIR" => root_path.join("zig-global-cache").to_s,
    "ZIG_LOCAL_CACHE_DIR" => root_path.join("zig-local-cache").to_s
  }
  run_bounded(
    zig_env, "zig", "cc", "-target", "aarch64-linux-musl", "-static", "-Os", "-fno-ident",
    "-Wl,--build-id=none", "-o", guest_init.to_s,
    exact_repo_file("platform/macos-vm-feasibility/Sources/GuestInit/main.c").to_s
  )
  file_stdout, = run_bounded({}, "file", guest_init.to_s)
  abort "synthetic init format changed" unless file_stdout.include?("ELF 64-bit") &&
                                                file_stdout.include?("ARM aarch64") &&
                                                file_stdout.include?("statically linked")

  initramfs = payload.join("impresari-initramfs.gz")
  run_bounded({}, "ruby", exact_repo_file("scripts/build-macos-vm-initramfs.rb").to_s,
              guest_init.to_s, module_path.to_s, initramfs.to_s)
  File.chmod(0o644, image)
  File.chmod(0o644, initramfs)

  measured = EXPECTED_OUTPUTS.map do |name, (bytes, digest)|
    path = payload.join(name)
    abort "materialized payload is missing or symlinked: #{name}" unless path.file? && !path.symlink?
    abort "materialized payload mode changed: #{name}" unless path.stat.mode & 0o777 == 0o644
    abort "materialized payload identity changed: #{name}" unless path.size == bytes && sha256(path) == digest
    {"relative_path" => name, "mode" => "0644", "bytes" => bytes.to_s, "sha256" => "sha256:#{digest}"}
  end
  abort "materialized payload set changed" unless payload.children.map { |path| path.basename.to_s }.sort == EXPECTED_OUTPUTS.keys.sort

  image_file, = run_bounded({}, "file", image.to_s)
  initramfs_file, = run_bounded({}, "file", initramfs.to_s)
  abort "kernel inspection changed" unless image_file.include?("ARM64") || image_file.include?("Linux kernel")
  abort "initramfs inspection changed" unless initramfs_file.include?("gzip compressed data")

  result = {
    "schema_name" => "macos-local-vm-synthetic-guest-materialization-record",
    "schema_version" => "1.0.0",
    "record_id" => "iar-macos-local-vm-synthetic-guest-materialization-2026-09-01.1",
    "decision" => "ADR-0112",
    "contract_id" => contract.fetch("contract_id"),
    "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
    "profile_id" => profile.fetch("profile_id"),
    "profile_digest" => "sha256:#{sha256(profile_path)}",
    "rehearsal_completed_at" => Time.now.utc.iso8601,
    "host" => {"operating_system" => "macos", "architecture" => "arm64", "zig_version" => zig_version.strip},
    "public_input" => {
      "url" => PUBLIC_URL, "bytes" => PUBLIC_BYTES.to_s, "sha256" => "sha256:#{PUBLIC_SHA256}",
      "publisher_signature_verified" => true, "signed_datahash_verified" => true,
      "package_name" => EXPECTED_PACKAGE.fetch("pkgname"), "package_version" => EXPECTED_PACKAGE.fetch("pkgver"),
      "package_architecture" => EXPECTED_PACKAGE.fetch("arch"), "package_commit" => EXPECTED_PACKAGE.fetch("commit")
    },
    "payload_members" => measured,
    "cleanup" => {
      "private_root_mode" => "0700", "private_root_name_retained" => false,
      "download_deleted" => true, "extracted_inputs_deleted" => true, "build_outputs_deleted" => true,
      "compiler_caches_deleted" => true, "raw_logs_retained" => false,
      "runnable_guest_artifacts_retained" => false, "metadata_only_retained" => true
    },
    "controls" => {
      "network_access" => true, "network_scope" => "exact-public-apk-only", "credential_access" => false,
      "compiler_process_launch" => true, "guest_payload_materialized" => true, "guest_payload_executed" => false,
      "app_assembled" => false, "apple_identity_access" => false, "signed" => false, "notarized" => false,
      "cask_created" => false, "bundle_installed" => false, "vm_launch" => false, "analyzer_execution" => false,
      "release_identity_bound" => false, "production_admitted" => false, "macos_iar_1b_admitted" => false,
      "authority_added" => false
    }
  }
ensure
  FileUtils.remove_entry_secure(root) if File.exist?(root)
end

abort "private materialization root was not deleted" if File.exist?(root)
puts JSON.pretty_generate(result)
