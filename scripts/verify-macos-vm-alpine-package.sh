#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu
umask 077

if [ "$#" -ne 3 ]; then
  printf '%s\n' 'usage: verify-macos-vm-alpine-package.sh NETBOOT_ARCHIVE APKINDEX LINUX_VIRT_APK' >&2
  exit 2
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
netboot_archive=$1
apkindex=$2
package=$3
public_key="$repo_root/platform/macos-vm-feasibility/alpine-devel-616ae350.rsa.pub"

for input in "$netboot_archive" "$apkindex" "$package" "$public_key"; do
  if [ ! -f "$input" ] || [ -L "$input" ]; then
    printf 'refusing missing or symlinked Alpine verification input: %s\n' "$input" >&2
    exit 3
  fi
done

"$repo_root/scripts/verify-macos-vm-alpine-archive.sh" "$netboot_archive" >/dev/null

temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/impresari-alpine-package.XXXXXX")
cleanup() {
  rm -rf "$temporary_dir"
}
trap cleanup EXIT HUP INT TERM
mkdir -p "$temporary_dir/netboot" "$temporary_dir/initramfs"
tar -xzf "$netboot_archive" -C "$temporary_dir/netboot" boot/initramfs-virt
(
  cd "$temporary_dir/initramfs"
  gzip -dc "$temporary_dir/netboot/boot/initramfs-virt" | cpio -id --quiet \
    'etc/apk/keys/alpine-devel@lists.alpinelinux.org-616ae350.rsa.pub'
)
authenticated_key="$temporary_dir/initramfs/etc/apk/keys/alpine-devel@lists.alpinelinux.org-616ae350.rsa.pub"
if [ "$(shasum -a 256 "$authenticated_key" | awk '{print $1}')" != d11f6b21c61b4274e182eb888883a8ba8acdbf820dcc7a6d82a7d9fc2fd2836d ] || \
   ! cmp -s "$authenticated_key" "$public_key"; then
  printf '%s\n' 'Alpine package key does not match the authenticated netboot initramfs' >&2
  exit 4
fi

measure() {
  path=$1
  expected_bytes=$2
  expected_sha256=$3
  if [ "$(stat -f '%z' "$path" 2>/dev/null || stat -c '%s' "$path")" != "$expected_bytes" ] || \
     [ "$(shasum -a 256 "$path" | awk '{print $1}')" != "$expected_sha256" ]; then
    printf 'refusing unexpected Alpine input identity: %s\n' "$path" >&2
    exit 5
  fi
}

measure "$apkindex" 529311 db44420861bbe4b2ae28756f8fceee9ced313a8585c56fed85dc7667b722d0fc
measure "$package" 41557960 c9ec62df20409d06f201cea7355140d5f99d421629ad35e9a023621a3c881616

index_result=$(ruby "$repo_root/scripts/verify-alpine-apkv2.rb" index "$apkindex" "$public_key")
package_result=$(ruby "$repo_root/scripts/verify-alpine-apkv2.rb" package "$package" "$public_key")

printf '%s\n' "$index_result" | jq -e '
  .rsa_sha1_signature_verified == true and
  .segment_bytes == ["746", "528565"] and
  .signature_sha256 == "sha256:a4e5f7e1b3d6a43e84ce7b825f12366a7c5f05183184bf2245990c1ab0d95e6f" and
  .index_record.P == "linux-virt" and
  .index_record.V == "6.18.48-r0" and
  .index_record.A == "aarch64" and
  .index_record.o == "linux-lts" and
  .index_record.c == "c83b91e0fde4c1bada9b80d4e67c395b5335597b"
' >/dev/null
printf '%s\n' "$package_result" | jq -e '
  .rsa_sha1_signature_verified == true and
  .datahash_verified == true and
  .segment_bytes == ["724", "497", "41556739"] and
  .signature_sha256 == "sha256:bb8cf07336a154e23901069049e9ca6e6315c35104d829219711406033b4362e" and
  .package.pkgname == "linux-virt" and
  .package.pkgver == "6.18.48-r0" and
  .package.arch == "aarch64" and
  .package.origin == "linux-lts" and
  .package.commit == "c83b91e0fde4c1bada9b80d4e67c395b5335597b" and
  .package.datahash == "e2ec28de6d80fa2b3535fc29475a7657ed8375dec99d4da96871ffd5b1077263"
' >/dev/null

printf '%s\n' 'macOS local-VM current Alpine package authentication passed: linux-virt=6.18.48-r0 key=616ae350 rsa_sha1_provider_signature=true production_admitted=false'
