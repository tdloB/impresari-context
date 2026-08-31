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
archive_root="$repo_root/target/iar-macos-vm-feasibility/alpine-package-6.18.48-r0"
kernel="$asset_root/vmlinuz-virt"
kernel_image="$asset_root/Image"
package="$asset_root/linux-virt-6.18.48-r0.apk"
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
  'https://dl-cdn.alpinelinux.org/alpine/v3.24/main/aarch64/linux-virt-6.18.48-r0.apk' \
  "$package" 41557960 c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616

package_result=$(ruby "$repo_root/scripts/verify-alpine-apkv2.rb" package "$package" \
  "$repo_root/platform/macos-vm-feasibility/alpine-devel-616ae350.rsa.pub")
printf '%s\n' "$package_result" | jq -e '
  .rsa_sha1_signature_verified == true and
  .datahash_verified == true and
  .package.pkgname == "linux-virt" and
  .package.pkgver == "6.18.48-r0" and
  .package.arch == "aarch64" and
  .package.commit == "c83b91e0fde4c1bada9b80d4e67c395b5335597b"
' >/dev/null

if [ ! -f "$archive_root/boot/vmlinuz-virt" ] || \
   [ ! -f "$archive_root/lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz" ]; then
  tar -xzf "$package" -C "$archive_root" \
    boot/vmlinuz-virt \
    lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko.gz
fi
cp "$archive_root/boot/vmlinuz-virt" "$kernel"

ruby "$repo_root/scripts/extract-macos-vm-kernel.rb" "$kernel" "$kernel_image"
kernel_image_bytes=$(stat -f '%z' "$kernel_image")
kernel_image_sha256=$(shasum -a 256 "$kernel_image" | awk '{print $1}')
if [ "$kernel_image_bytes" != 36175872 ] || [ "$kernel_image_sha256" != 4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5 ]; then
  printf '%s\n' 'refusing unexpected extracted ARM64 Linux kernel image' >&2
  exit 5
fi

package_module="$archive_root/lib/modules/6.18.48-0-virt/kernel/drivers/block/virtio_blk.ko"
if [ ! -f "$package_module" ]; then
  gzip -dk "$package_module.gz"
fi
cp "$package_module" "$module"

module_bytes=$(stat -f '%z' "$module")
module_sha256=$(shasum -a 256 "$module" | awk '{print $1}')
if [ "$module_bytes" != 49687 ] || [ "$module_sha256" != c8eb0f6b98a18a5cc237bc3019637551f46f964a5efd215253a0946889e3f31d ]; then
  printf '%s\n' 'refusing unexpected virtio_blk module' >&2
  exit 4
fi

printf '%s\n' "$asset_root"
