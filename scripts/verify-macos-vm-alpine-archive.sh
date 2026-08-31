#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu
umask 077

if [ "$#" -ne 1 ]; then
  printf 'usage: %s /absolute/path/alpine-netboot-3.24.1-aarch64.tar.gz\n' "$0" >&2
  exit 2
fi

archive=$1
case "$archive" in
  /*) ;;
  *) printf '%s\n' 'Alpine archive path must be absolute' >&2; exit 2 ;;
esac
if [ ! -f "$archive" ] || [ -L "$archive" ]; then
  printf '%s\n' 'Alpine archive must be one regular non-symlink file' >&2
  exit 3
fi
if ! command -v gpg >/dev/null 2>&1 || ! command -v gpgv >/dev/null 2>&1; then
  printf '%s\n' 'gpg and gpgv are required for the explicit upstream-authentication check' >&2
  exit 4
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
key="$repo_root/platform/macos-vm-feasibility/alpine-ncopa-0482D84022F52DF1C4E7CD43293ACD0907D9495A.asc"
signature="$repo_root/platform/macos-vm-feasibility/alpine-netboot-3.24.1-aarch64.tar.gz.asc"

if [ "$(uname -s)" = "Darwin" ]; then
  archive_bytes=$(stat -f '%z' "$archive")
else
  archive_bytes=$(stat -c '%s' "$archive")
fi
archive_sha256=$(shasum -a 256 "$archive" | awk '{print $1}')
if [ "$archive_bytes" != 431008592 ] || [ "$archive_sha256" != 54fe38fa41cce740ba379458ed63cfcd89ab06ae5e6a47a06dafe0a34e8427e8 ]; then
  printf '%s\n' 'Alpine archive identity differs from the frozen signed release' >&2
  exit 5
fi

temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/impresari-alpine-auth.XXXXXX")
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM
mkdir -p "$temporary_dir/gnupg" "$temporary_dir/extract"
chmod 700 "$temporary_dir/gnupg"
GNUPGHOME="$temporary_dir/gnupg"
export GNUPGHOME
gpg --batch --dearmor --output "$temporary_dir/alpine-release-key.gpg" "$key"
signature_status=$(gpgv --keyring "$temporary_dir/alpine-release-key.gpg" --status-fd 1 "$signature" "$archive")
printf '%s\n' "$signature_status" | grep -q '^\[GNUPG:\] VALIDSIG 0482D84022F52DF1C4E7CD43293ACD0907D9495A '

tar -xzf "$archive" -C "$temporary_dir/extract" boot/vmlinuz-virt boot/initramfs-virt
kernel="$temporary_dir/extract/boot/vmlinuz-virt"
initramfs="$temporary_dir/extract/boot/initramfs-virt"
if [ "$(uname -s)" = "Darwin" ]; then
  kernel_bytes=$(stat -f '%z' "$kernel")
  initramfs_bytes=$(stat -f '%z' "$initramfs")
else
  kernel_bytes=$(stat -c '%s' "$kernel")
  initramfs_bytes=$(stat -c '%s' "$initramfs")
fi
kernel_sha256=$(shasum -a 256 "$kernel" | awk '{print $1}')
initramfs_sha256=$(shasum -a 256 "$initramfs" | awk '{print $1}')
if [ "$kernel_bytes" != 10351104 ] || [ "$kernel_sha256" != 47970e0ee0478fe5c60824a89f162d5a353fa29466e5d3bddb0f9c506f1ed756 ]; then
  printf '%s\n' 'signed Alpine archive contains an unexpected virt kernel' >&2
  exit 6
fi
if [ "$initramfs_bytes" != 9385851 ] || [ "$initramfs_sha256" != e47d38bc88509a3db11affc09f9762f9643b026bd29441724a4729ad8e97add6 ]; then
  printf '%s\n' 'signed Alpine archive contains an unexpected virt initramfs' >&2
  exit 6
fi

printf '%s\n' 'macOS local-VM Alpine upstream authentication passed: fingerprint=0482D84022F52DF1C4E7CD43293ACD0907D9495A release=3.24.1 embedded_assets=2 release_metadata_sealed=false'
