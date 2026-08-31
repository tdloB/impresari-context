#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu
umask 077

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM asset preparation requires macOS arm64' >&2
  exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
asset_root="$repo_root/target/iar-macos-vm-feasibility/assets"
archive_root="$repo_root/target/iar-macos-vm-feasibility/alpine-initramfs"
kernel="$asset_root/vmlinuz-virt"
kernel_image="$asset_root/Image"
upstream_initramfs="$asset_root/initramfs-virt"
module="$asset_root/virtio_blk.ko"

mkdir -p "$asset_root" "$archive_root"

fetch_exact() {
  url=$1
  output=$2
  expected_bytes=$3
  expected_sha256=$4
  if [ -L "$output" ] || [ -L "$output.partial" ]; then
    printf 'refusing symlinked macOS VM asset path: %s\n' "$output" >&2
    exit 3
  fi
  if [ ! -f "$output" ]; then
    if [ -e "$output.partial" ]; then
      printf 'refusing pre-existing partial macOS VM asset: %s\n' "$output.partial" >&2
      exit 3
    fi
    curl --fail --silent --show-error --max-redirs 0 \
      --proto '=https' --tlsv1.2 \
      --connect-timeout 10 --max-time 120 --output "$output.partial" "$url"
    mv "$output.partial" "$output"
  fi
  actual_bytes=$(stat -f '%z' "$output")
  actual_sha256=$(shasum -a 256 "$output" | awk '{print $1}')
  if [ "$actual_bytes" != "$expected_bytes" ] || [ "$actual_sha256" != "$expected_sha256" ]; then
    printf 'refusing unexpected macOS VM asset: %s\n' "$output" >&2
    exit 3
  fi
}

fetch_exact \
  'https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64/netboot/vmlinuz-virt' \
  "$kernel" 10351104 47970e0ee0478fe5c60824a89f162d5a353fa29466e5d3bddb0f9c506f1ed756
fetch_exact \
  'https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64/netboot/initramfs-virt' \
  "$upstream_initramfs" 9385851 e47d38bc88509a3db11affc09f9762f9643b026bd29441724a4729ad8e97add6

ruby "$repo_root/scripts/extract-macos-vm-kernel.rb" "$kernel" "$kernel_image"
kernel_image_bytes=$(stat -f '%z' "$kernel_image")
kernel_image_sha256=$(shasum -a 256 "$kernel_image" | awk '{print $1}')
if [ "$kernel_image_bytes" != 36110336 ] || [ "$kernel_image_sha256" != 8b216f74e7f89def4604adf69e2345437363aff4819101bb1551c9e83cd35cdd ]; then
  printf '%s\n' 'refusing unexpected extracted ARM64 Linux kernel image' >&2
  exit 5
fi

if [ ! -f "$module" ]; then
  (
    cd "$archive_root"
    gzip -dc "$upstream_initramfs" | cpio -id --quiet \
      'usr/lib/modules/6.18.35-0-virt/kernel/drivers/block/virtio_blk.ko'
  )
  cp "$archive_root/usr/lib/modules/6.18.35-0-virt/kernel/drivers/block/virtio_blk.ko" "$module"
fi

module_bytes=$(stat -f '%z' "$module")
module_sha256=$(shasum -a 256 "$module" | awk '{print $1}')
if [ "$module_bytes" != 49687 ] || [ "$module_sha256" != 80341fdb0869f5df4813b7bfb4a1cd77d2f6cd7c26c04fc15706cbc44d680ef6 ]; then
  printf '%s\n' 'refusing unexpected virtio_blk module' >&2
  exit 4
fi

printf '%s\n' "$asset_root"
