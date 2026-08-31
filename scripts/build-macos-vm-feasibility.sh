#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM build requires macOS arm64' >&2
  exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repo_root/target/iar-macos-vm-feasibility"
asset_root="$output_root/assets"
bin_root="$output_root/bin"
guest_init="$bin_root/impresari-vm-init"
controller="$bin_root/impresari-context-vm-controller"
initramfs="$asset_root/impresari-initramfs.gz"
module="$asset_root/virtio_blk.ko"

for asset in "$asset_root/Image" "$module"; do
  if [ ! -f "$asset" ]; then
    printf '%s\n' 'run scripts/prepare-macos-vm-feasibility.sh first' >&2
    exit 3
  fi
done
if ! command -v zig >/dev/null 2>&1; then
  printf '%s\n' 'Zig is required to cross-compile the synthetic Linux init' >&2
  exit 4
fi

mkdir -p "$bin_root"
ZIG_GLOBAL_CACHE_DIR="$output_root/zig-global-cache" \
ZIG_LOCAL_CACHE_DIR="$output_root/zig-local-cache" \
zig cc -target aarch64-linux-musl -static -Os -fno-ident \
  -Wl,--build-id=none \
  -o "$guest_init" \
  "$repo_root/platform/macos-vm-feasibility/Sources/GuestInit/main.c"

if ! file "$guest_init" | grep -q 'ELF 64-bit.*ARM aarch64.*statically linked'; then
  printf '%s\n' 'synthetic guest init is not a static ARM64 Linux executable' >&2
  exit 5
fi

ruby "$repo_root/scripts/build-macos-vm-initramfs.rb" \
  "$guest_init" "$module" "$initramfs"

xcrun swiftc -swift-version 5 -O \
  -module-cache-path "$output_root/swift-module-cache" \
  -framework Virtualization -framework CryptoKit \
  -o "$controller" \
  "$repo_root/platform/macos-vm-feasibility/Sources/Controller/main.swift"
codesign --force --sign - \
  --entitlements "$repo_root/platform/macos-vm-feasibility/Resources/Controller.entitlements" \
  "$controller" >/dev/null

initramfs_sha256=$(shasum -a 256 "$initramfs" | awk '{print $1}')
printf '%s\n' "$initramfs_sha256" > "$asset_root/impresari-initramfs.sha256"
printf '%s\n' "$controller"
