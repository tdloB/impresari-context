#!/usr/bin/env ruby
# SPDX-License-Identifier: Apache-2.0
# frozen_string_literal: true

require "digest"
require "fileutils"
require "find"
require "json"
require "open3"
require "pathname"
require "rubygems/package"
require "stringio"
require "tmpdir"
require "time"
require "zlib"

ROOT = Pathname.new(__dir__).join("..").expand_path
SOURCE_REVISION = "aca656771f9286b13fbcc046b133ade62b58da2a"
SOURCE_ARCHIVE_BYTES = 35_573_760
SOURCE_ARCHIVE_SHA256 = "f26fcf7ccdc6cb499e3eacc1f479a93083c58d397c8730b72a56d43d8c0adb8b"
SOURCE_DATE_EPOCH = "1788243888"
PRODUCT_VERSION = "0.2.0"
TARGET = "aarch64-apple-darwin"
CONTRACT_DIGEST = "ebf78abf0a8b1609cf891b96f092065a3e957d4b819e221d47440bacc4f9cf9c"
COMPOSITION_DIGEST = "88ee55c39b6735d645b285ca43ba203f4517f3d695aeaafc29d604c25eb6a167"
GUEST_CONTRACT_DIGEST = "4e43e28f325d7ab67ff2bb23595eb9273320ff5e8597553b9a681bfdc51033d4"
SEAL_DIGEST = "c0294a88c2c7fe1d33bdd8ddfbb55e26e6595f02c12a9645c898f36148aa82e1"
EXPECTED_CONTRACT_IDENTITY = "8d3da788a95c6cf638537218722e5fe32629710a10a3b25c0ac282280ed5720e"
EXPECTED_MATERIAL_IDENTITY = "39ae0afbb77eff80ff5308cc4fe811b7cc266b42d02b4457aa5295310908b11e"
PUBLIC_URL = "https://dl-cdn.alpinelinux.org/alpine/v3.24/main/aarch64/linux-virt-6.18.48-r0.apk"
PUBLIC_BYTES = 41_557_960
PUBLIC_SHA256 = "c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616"
MAX_CAPTURE_BYTES = 2 * 1024 * 1024
MAX_ARCHIVE_EXPANDED_BYTES = 256 * 1024 * 1024

EXPECTED_PACKAGE = {
  "pkgname" => "linux-virt", "pkgver" => "6.18.48-r0", "arch" => "aarch64",
  "commit" => "c83b91e0fde4c1bada9b80d4e67c395b5335597b",
  "datahash" => "e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263"
}.freeze

EXTRACTED_MEMBERS = {
  "boot/vmlinuz-virt" => 64 * 1024 * 1024,
  "lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz" => 2 * 1024 * 1024
}.freeze

PRODUCTS = {
  "cli-supervisor-entrypoint" => {
    "bundle_path" => "Contents/MacOS/impresari-context",
    "output" => "impresari-context", "bytes" => 8_261_920,
    "sha256" => "fa1992cd02678c03888a4a5f5a42849880dba42ef9e2b59153c5e66749499bd9",
    "command" => %w[cargo build --offline --locked --release --target aarch64-apple-darwin -p context-cli --bin impresari-context]
  },
  "local-stdio-mcp-server" => {
    "bundle_path" => "Contents/Helpers/impresari-context-mcp",
    "output" => "impresari-context-mcp", "bytes" => 4_496_400,
    "sha256" => "4324a95f4a6ceeb506f659bda8d8a6cb54cb00cbfa0248e81f6b98bb815e086c",
    "command" => %w[cargo build --offline --locked --release --target aarch64-apple-darwin -p context-mcp --bin impresari-context-mcp]
  },
  "isolated-structural-worker" => {
    "bundle_path" => "Contents/Helpers/impresari-context-structural-worker",
    "output" => "impresari-context-structural-worker", "bytes" => 35_820_544,
    "sha256" => "ab2efcae9c89c2a3cf8543c5be5cf6a63650e0ef689ec2be95df5b48aad103a7",
    "command" => %w[cargo build --offline --locked --release --target aarch64-apple-darwin -p context-structural --bin impresari-context-structural-worker]
  },
  "local-vm-controller" => {
    "bundle_path" => "Contents/Helpers/impresari-context-vm-controller",
    "output" => "impresari-context-vm-controller", "bytes" => 274_704,
    "sha256" => "48689796ad27aa4413a95d23ebb318d14c64a786cf0c5ab1b12553d5d656b7a5"
  }
}.freeze

GUEST_OUTPUTS = {
  "Image" => [36_175_872, "4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5"],
  "impresari-initramfs.gz" => [38_207, "89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b"]
}.freeze

abort "usage: rehearse-macos-vm-ephemeral-unsigned-release-candidate.rb" unless ARGV.empty?
abort "macOS unsigned candidate rehearsal requires Darwin arm64" unless `uname -s`.strip == "Darwin" && `uname -m`.strip == "arm64"

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

def run_bounded(env, *command, chdir: nil)
  options = chdir ? {chdir: chdir.to_s} : {}
  stdout, stderr, status = Open3.capture3(env, *command, **options)
  abort "child output exceeded bound: #{command.first}" if stdout.bytesize + stderr.bytesize > MAX_CAPTURE_BYTES
  abort "bounded child failed: #{command.first}: #{stderr.lines.first.to_s.strip}" unless status.success?
  [stdout, stderr]
end

def clean_relative(path)
  value = Pathname.new(path)
  !value.absolute? && value.cleanpath.to_s == path && !value.each_filename.include?("..")
end

def extract_source_archive(archive, destination)
  total = 0
  File.open(archive, "rb") do |io|
    Gem::Package::TarReader.new(io) do |tar|
      tar.each do |entry|
        relative = entry.full_name.delete_suffix("/")
        next if relative.empty?
        if relative == "pax_global_header" && entry.header.typeflag == "g"
          abort "oversized source archive PAX header" if entry.header.size > 4096
          entry.read
          next
        end
        abort "unsafe source archive path" unless clean_relative(relative)
        output = destination.join(relative)
        abort "source archive output escaped root" unless output.cleanpath.to_s.start_with?(destination.to_s + File::SEPARATOR)
        if entry.directory?
          FileUtils.mkdir_p(output, mode: 0o755)
        elsif entry.file?
          total += entry.header.size
          abort "source archive expanded-size limit exceeded" if total > MAX_ARCHIVE_EXPANDED_BYTES
          FileUtils.mkdir_p(output.dirname, mode: 0o755)
          File.open(output, File::WRONLY | File::CREAT | File::EXCL | File::BINARY, 0o600) do |file|
            IO.copy_stream(entry, file)
          end
          File.chmod((entry.header.mode & 0o111).zero? ? 0o644 : 0o755, output)
        else
          abort "source archive contains a link or special member: #{relative}"
        end
      end
    end
  end
end

def apk_segments(path)
  archive = File.binread(path)
  segments = []
  offset = 0
  while offset < archive.bytesize
    abort "unexpected APKv2 segment count" if segments.length >= 3
    source = StringIO.new(archive.byteslice(offset..))
    reader = Zlib::GzipReader.new(source)
    segments << reader.read
    consumed = source.pos - reader.unused.to_s.bytesize
    abort "invalid APKv2 segment" unless consumed.positive?
    offset += consumed
  end
  abort "unexpected APKv2 segment count" unless segments.length == 3
  segments
rescue Zlib::GzipFile::Error, Zlib::Error => e
  abort "invalid APKv2 framing: #{e.message}"
end

def extract_exact_apk_members(data_tar, destination)
  selected = {}
  Gem::Package::TarReader.new(StringIO.new(data_tar)) do |tar|
    tar.each do |entry|
      next unless EXTRACTED_MEMBERS.key?(entry.full_name)
      abort "bounded APK member is not regular" unless entry.file?
      abort "duplicate bounded APK member" if selected.key?(entry.full_name)
      abort "oversized bounded APK member" if entry.header.size > EXTRACTED_MEMBERS.fetch(entry.full_name)
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
end

def info_plist
  <<~PLIST
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
      <key>CFBundleDisplayName</key>
      <string>Impresari Context (Synthetic)</string>
      <key>CFBundleExecutable</key>
      <string>impresari-context</string>
      <key>CFBundleIdentifier</key>
      <string>dev.impresari.context.synthetic-assembly</string>
      <key>CFBundlePackageType</key>
      <string>APPL</string>
      <key>CFBundleShortVersionString</key>
      <string>0.0.0</string>
      <key>CFBundleVersion</key>
      <string>0</string>
    </dict>
    </plist>
  PLIST
end

def file_inventory(app)
  records = []
  Find.find(app.to_s) do |raw|
    next if raw == app.to_s
    path = Pathname.new(raw)
    relative = path.relative_path_from(app).to_s
    stat = path.lstat
    abort "candidate contains a symlink: #{relative}" if stat.symlink?
    kind = stat.directory? ? "directory" : (stat.file? ? "file" : nil)
    abort "candidate contains a special file: #{relative}" unless kind
    records << {
      "path" => relative, "kind" => kind, "mode" => format("%04o", stat.mode & 0o7777),
      "bytes" => kind == "file" ? stat.size.to_s : "0",
      "sha256" => kind == "file" ? "sha256:#{sha256(path)}" : "none"
    }
  end
  records.sort_by { |entry| entry.fetch("path") }
end

def command_version(*command)
  stdout, stderr = run_bounded({}, *command)
  [stdout, stderr].join.strip
end

exact_repo_file("platform/macos-vm-feasibility/release-identity-contract-v1.json", CONTRACT_DIGEST)
composition = JSON.parse(exact_repo_file("platform/macos-vm-feasibility/unsigned-release-candidate-composition-v1.json", COMPOSITION_DIGEST).read)
guest_contract = JSON.parse(exact_repo_file("platform/macos-vm-feasibility/synthetic-guest-payload-contract-v1.json", GUEST_CONTRACT_DIGEST).read)
seal = exact_repo_file("platform/macos-vm-feasibility/guest-release-metadata-seal-v1.json", SEAL_DIGEST)
recipe = guest_contract.fetch("future_materialization_recipe")
recipe.fetch("build_inputs").each { |input| exact_repo_file(input.fetch("path"), input.fetch("sha256").delete_prefix("sha256:")) }
key_path = exact_repo_file(recipe.dig("public_input", "verification_key_path"), recipe.dig("public_input", "verification_key_sha256").delete_prefix("sha256:"))
abort "unexpected Zig version" unless command_version("zig", "version") == recipe.fetch("zig_version")

root = Dir.mktmpdir("impresari-macos-unsigned-candidate-")
File.chmod(0o700, root)
root_path = Pathname.new(root)
candidate = nil
rehearsal = nil

begin
  abort "temporary candidate root is not private" unless (root_path.stat.mode & 0o7777) == 0o700
  source_archive = root_path.join("source.tar")
  run_bounded({}, "git", "archive", "--format=tar", "--output=#{source_archive}", SOURCE_REVISION, chdir: ROOT)
  abort "source archive identity changed" unless source_archive.size == SOURCE_ARCHIVE_BYTES && sha256(source_archive) == SOURCE_ARCHIVE_SHA256
  source = root_path.join("source")
  FileUtils.mkdir_p(source, mode: 0o700)
  extract_source_archive(source_archive, source)

  target = root_path.join("product-target")
  module_cache = root_path.join("swift-module-cache")
  FileUtils.mkdir_p(target, mode: 0o700)
  FileUtils.mkdir_p(module_cache, mode: 0o700)
  build_env = {
    "CARGO_TARGET_DIR" => target.to_s, "CARGO_NET_OFFLINE" => "true", "CARGO_INCREMENTAL" => "0",
    "SOURCE_DATE_EPOCH" => SOURCE_DATE_EPOCH, "LC_ALL" => "C", "LANG" => "C"
  }
  build_logs = {}
  PRODUCTS.each do |unit_id, product|
    next unless product.key?("command")
    stdout, stderr = run_bounded(build_env, *product.fetch("command"), chdir: source)
    build_logs[unit_id] = Digest::SHA256.hexdigest(stdout + stderr)
  end
  controller = PRODUCTS.fetch("local-vm-controller")
  controller_output = target.join("iar-macos-vm-release/bin", controller.fetch("output"))
  FileUtils.mkdir_p(controller_output.dirname, mode: 0o700)
  swift_command = [
    "xcrun", "swiftc", "-swift-version", "5", "-O", "-module-cache-path", module_cache.to_s,
    "-framework", "Virtualization", "-framework", "CryptoKit", "-framework", "AppKit",
    "-o", controller_output.to_s, "platform/macos-vm-feasibility/Sources/Controller/main.swift"
  ]
  stdout, stderr = run_bounded({"SOURCE_DATE_EPOCH" => SOURCE_DATE_EPOCH, "LC_ALL" => "C", "LANG" => "C"}, *swift_command, chdir: source)
  build_logs["local-vm-controller"] = Digest::SHA256.hexdigest(stdout + stderr)

  download = root_path.join("linux-virt.apk")
  partial = root_path.join("linux-virt.apk.partial")
  run_bounded({}, "curl", "--fail", "--silent", "--show-error", "--max-redirs", "0",
              "--proto", "=https", "--tlsv1.2", "--connect-timeout", "10", "--max-time", "120",
              "--output", partial.to_s, PUBLIC_URL)
  File.rename(partial, download)
  abort "downloaded APK identity changed" unless download.size == PUBLIC_BYTES && sha256(download) == PUBLIC_SHA256
  verification_stdout, = run_bounded({}, "ruby", exact_repo_file("scripts/verify-alpine-apkv2.rb").to_s, "package", download.to_s, key_path.to_s)
  verification = JSON.parse(verification_stdout)
  abort "publisher authentication failed" unless verification.fetch("rsa_sha1_signature_verified") && verification.fetch("datahash_verified") && EXPECTED_PACKAGE.all? { |key, value| verification.fetch("package").fetch(key) == value }

  extracted = root_path.join("guest-extracted")
  FileUtils.mkdir_p(extracted, mode: 0o700)
  extract_exact_apk_members(apk_segments(download).fetch(2), extracted)
  compressed_kernel = extracted.join("boot/vmlinuz-virt")
  compressed_module = extracted.join("lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz")
  guest_build = root_path.join("guest-build")
  FileUtils.mkdir_p(guest_build, mode: 0o700)
  module_path = guest_build.join("virtio_blk.ko")
  Zlib::GzipReader.open(compressed_module.to_s) do |gzip|
    bytes = gzip.read(2 * 1024 * 1024 + 1)
    abort "inflated module exceeded bound" if bytes.bytesize > 2 * 1024 * 1024
    File.binwrite(module_path, bytes)
  end
  abort "module identity changed" unless module_path.size == 49_687 && sha256(module_path) == "c8eb0f6b98a18a5cc237bc3019637551f46f964a5efd215253a0946889e3f31d"
  image = guest_build.join("Image")
  run_bounded({}, "ruby", exact_repo_file("scripts/extract-macos-vm-kernel.rb").to_s, compressed_kernel.to_s, image.to_s)
  guest_init = guest_build.join("init")
  zig_env = {"ZIG_GLOBAL_CACHE_DIR" => root_path.join("zig-global-cache").to_s, "ZIG_LOCAL_CACHE_DIR" => root_path.join("zig-local-cache").to_s}
  run_bounded(zig_env, "zig", "cc", "-target", "aarch64-linux-musl", "-static", "-Os", "-fno-ident", "-Wl,--build-id=none", "-o", guest_init.to_s, exact_repo_file("platform/macos-vm-feasibility/Sources/GuestInit/main.c").to_s)
  initramfs = guest_build.join("impresari-initramfs.gz")
  run_bounded({}, "ruby", exact_repo_file("scripts/build-macos-vm-initramfs.rb").to_s, guest_init.to_s, module_path.to_s, initramfs.to_s)
  [image, initramfs].each { |path| File.chmod(0o644, path) }
  GUEST_OUTPUTS.each do |name, (bytes, digest)|
    path = guest_build.join(name)
    abort "guest output identity changed: #{name}" unless path.size == bytes && sha256(path) == digest
  end

  app = root_path.join("Impresari Context.app")
  %w[Contents Contents/Helpers Contents/MacOS Contents/Resources Contents/Resources/macos-vm Contents/Resources/macos-vm/guest].each do |relative|
    FileUtils.mkdir_p(app.join(relative), mode: 0o755)
    File.chmod(0o755, app.join(relative))
  end
  File.binwrite(app.join("Contents/Info.plist"), info_plist)
  File.chmod(0o644, app.join("Contents/Info.plist"))
  FileUtils.cp(seal, app.join("Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json"), preserve: false)
  File.chmod(0o644, app.join("Contents/Resources/macos-vm/guest-release-metadata-seal-v1.json"))

  artifacts = []
  PRODUCTS.each do |unit_id, product|
    source_output = if unit_id == "local-vm-controller"
                      controller_output
                    else
                      target.join(TARGET, "release", product.fetch("output"))
                    end
    abort "product output identity changed: #{unit_id}" unless source_output.size == product.fetch("bytes") && sha256(source_output) == product.fetch("sha256")
    format, = run_bounded({}, "file", source_output.to_s)
    arch, = run_bounded({}, "lipo", "-archs", source_output.to_s)
    _, signing = run_bounded({}, "codesign", "-dv", "--verbose=4", source_output.to_s)
    abort "product output format changed: #{unit_id}" unless format.include?("Mach-O 64-bit executable arm64")
    abort "product output architecture changed: #{unit_id}" unless arch.strip == "arm64"
    abort "product output is not linker ad-hoc: #{unit_id}" unless signing.include?("Signature=adhoc") && !signing.include?("Authority=Developer ID")
    destination = app.join(product.fetch("bundle_path"))
    FileUtils.cp(source_output, destination, preserve: false)
    File.chmod(0o755, destination)
    artifacts << {
      "unit_id" => unit_id, "bundle_path" => product.fetch("bundle_path"),
      "bytes" => product.fetch("bytes").to_s, "sha256" => "sha256:#{product.fetch('sha256')}",
      "file_format" => "mach-o-64-arm64", "architectures" => ["arm64"], "unsigned" => true,
      "build_log_sha256" => "sha256:#{build_logs.fetch(unit_id)}"
    }
  end
  GUEST_OUTPUTS.each do |name, _identity|
    destination = app.join("Contents/Resources/macos-vm/guest", name)
    FileUtils.cp(guest_build.join(name), destination, preserve: false)
    File.chmod(0o644, destination)
  end

  actual = file_inventory(app)
  expected_files = composition.fetch("material_projection").map do |entry|
    {"path" => entry.fetch("path"), "kind" => "file", "mode" => entry.fetch("required_mode"), "bytes" => entry.fetch("bytes"), "sha256" => entry.fetch("sha256")}
  end
  actual_files = actual.select { |entry| entry.fetch("kind") == "file" }
  abort "candidate file inventory changed" unless actual_files == expected_files
  expected_directories = %w[Contents Contents/Helpers Contents/MacOS Contents/Resources Contents/Resources/macos-vm Contents/Resources/macos-vm/guest]
  actual_directories = actual.select { |entry| entry.fetch("kind") == "directory" }
  abort "candidate directory inventory changed" unless actual_directories.map { |entry| entry.fetch("path") } == expected_directories
  abort "candidate directory mode changed" unless actual_directories.all? { |entry| entry.fetch("mode") == "0755" }

  contract_rows = [["candidate-source-revision", SOURCE_REVISION], ["product-version", PRODUCT_VERSION]]
  contract_rows.concat(actual_files.map { |entry| [entry.fetch("path"), entry.fetch("bytes"), entry.fetch("sha256")] })
  contract_identity = Digest::SHA256.hexdigest(contract_rows.map { |row| row.join("\t") + "\n" }.join)
  abort "ADR-0109 compound identity changed" unless contract_identity == EXPECTED_CONTRACT_IDENTITY
  material_rows = [["candidate-source-revision", SOURCE_REVISION], ["product-version", PRODUCT_VERSION], ["target", TARGET]]
  material_rows.concat(actual_files.map { |entry| [entry.fetch("path"), entry.fetch("kind"), entry.fetch("mode"), entry.fetch("bytes"), entry.fetch("sha256")] })
  material_identity = Digest::SHA256.hexdigest(material_rows.map { |row| row.join("\t") + "\n" }.join)
  abort "ADR-0113 material identity changed" unless material_identity == EXPECTED_MATERIAL_IDENTITY

  guest_rows = GUEST_OUTPUTS.map do |name, (bytes, digest)|
    [name, "0644", bytes.to_s, "sha256:#{digest}"]
  end
  payload_inventory = Digest::SHA256.hexdigest(guest_rows.map { |row| row.join("\t") + "\n" }.join)
  artifacts.sort_by! { |artifact| artifact.fetch("unit_id") }

  candidate = {
    "schema_name" => "macos-local-vm-unsigned-release-candidate", "schema_version" => "1.0.0",
    "contract_id" => "iar-macos-local-vm-release-identity-2026-09-01.1", "contract_digest" => "sha256:#{CONTRACT_DIGEST}",
    "candidate_id" => "impresari-context-macos-arm64-0.2.0-candidate-1",
    "candidate_source_revision" => SOURCE_REVISION, "candidate_source_archive_sha256" => "sha256:#{SOURCE_ARCHIVE_SHA256}",
    "product_version" => PRODUCT_VERSION, "target" => TARGET,
    "build_environment" => {
      "macos_product_version" => command_version("sw_vers", "-productVersion"),
      "macos_build_version" => command_version("sw_vers", "-buildVersion"),
      "xcode_version" => command_version("xcodebuild", "-version").lines.fetch(0).sub("Xcode ", "").strip,
      "xcode_build_version" => command_version("xcodebuild", "-version").lines.fetch(1).sub("Build version ", "").strip,
      "apple_sdk_version" => command_version("xcrun", "--sdk", "macosx", "--show-sdk-version"),
      "swift_version" => command_version("xcrun", "swiftc", "--version"),
      "rustc_verbose_version" => command_version("rustc", "-vV"),
      "cargo_version" => command_version("cargo", "--version"), "target_triple" => TARGET
    },
    "artifacts" => artifacts,
    "guest" => {
      "guest_release_id" => "iar-macos-local-vm-guest-2026-08-31.1",
      "guest_metadata_set_digest" => "sha256:ea29c43f36493f7e61935f33a64822805c8275d804c5384c3e8becea849fc54b",
      "guest_metadata_seal_digest" => "sha256:#{SEAL_DIGEST}", "payload_inventory_sha256" => "sha256:#{payload_inventory}"
    },
    "product_evidence" => {
      "spdx_2_3_sbom_sha256" => "sha256:bb249501b6d693edaff188edc2344d1d1a62a94bd13ace8488f4a03e5273a3bb",
      "license_inventory_sha256" => "sha256:6f7183c6b0c46d7121c371df536f810677ed843a9f282d454859c3ab04a4c219",
      "vulnerability_assessment_sha256" => "sha256:73a56792d4a09d3cf12329e3d46f289ace496eaab42c391c57689726197daea1",
      "reproducibility_disposition_sha256" => "sha256:b74119c2acebfdc919c7852cee904016483f93113471bae3a23fd5f56135b59b"
    },
    "compound_identity" => "sha256:#{contract_identity}",
    "controls" => {
      "candidate_materialized" => true, "release_identity_bound" => true,
      "developer_id_signature_verified" => false, "apple_notarization_verified" => false,
      "bundle_installed" => false, "cask_created" => false, "github_publication_attestation_verified" => false,
      "cask_lifecycle_verified" => false, "sealed_distribution" => false, "vm_launch" => false,
      "analyzer_execution" => false, "production_admitted" => false, "macos_iar_1b_admitted" => false,
      "authority_added" => false
    }
  }
  candidate_bytes = JSON.pretty_generate(candidate) + "\n"
  rehearsal = {
    "schema_name" => "macos-local-vm-ephemeral-unsigned-release-candidate-rehearsal", "schema_version" => "1.0.0",
    "rehearsal_id" => "iar-macos-local-vm-ephemeral-unsigned-release-candidate-2026-09-01.1",
    "decision" => "ADR-0114", "completed_at" => Time.now.utc.iso8601,
    "candidate_record_sha256" => "sha256:#{Digest::SHA256.hexdigest(candidate_bytes)}",
    "composition_record_sha256" => "sha256:#{COMPOSITION_DIGEST}",
    "contract_compound_identity" => "sha256:#{contract_identity}",
    "material_projection_identity" => "sha256:#{material_identity}",
    "file_count" => "8", "directory_count" => "6", "private_root_mode" => "0700",
    "source_archive_verified" => true, "publisher_signature_verified" => true,
    "signed_datahash_verified" => true, "product_identities_exact" => true,
    "guest_identities_exact" => true, "app_tree_exact" => true,
    "filesystem_modes_exact" => true, "simultaneous_component_custody" => true,
    "candidate_materialized" => true, "app_assembled" => true, "release_identity_bound" => true,
    "produced_artifacts_executed" => false, "network_access" => true,
    "network_scope" => "exact-public-apk-only", "credential_access" => false,
    "apple_identity_access" => false, "developer_id_signature_verified" => false,
    "apple_notarization_verified" => false, "archive_created" => false, "cask_created" => false,
    "bundle_installed" => false, "vm_launch" => false, "analyzer_execution" => false,
    "production_admitted" => false, "macos_iar_1b_admitted" => false, "authority_added" => false
  }
ensure
  FileUtils.remove_entry_secure(root) if File.exist?(root)
end

abort "private candidate root was not deleted" if File.exist?(root)
rehearsal["cleanup_verified"] = true
rehearsal["runnable_artifacts_retained"] = false
rehearsal["raw_build_logs_retained"] = false
puts JSON.pretty_generate({"candidate" => candidate, "rehearsal" => rehearsal})
