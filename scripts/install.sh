#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

repository="tdloB/impresari-context"
release_version=""
install_dir="${IMPRESARI_INSTALL_DIR:-${HOME:-}/.local/bin}"

usage() {
    cat <<'EOF'
Usage: install.sh --version vX.Y.Z [--install-dir PATH]

Downloads one published Impresari Context release archive, verifies its
published SHA-256 checksum, and installs the three release binaries.

The version is required. This installer never resolves or installs "latest".
Set IMPRESARI_INSTALL_DIR or pass --install-dir to change the destination.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || { printf 'missing --version value\n' >&2; exit 2; }
            release_version=$2
            shift 2
            ;;
        --install-dir)
            [ "$#" -ge 2 ] || { printf 'missing --install-dir value\n' >&2; exit 2; }
            install_dir=$2
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            printf 'unknown argument: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if ! printf '%s\n' "$release_version" | grep -Eq \
    '^v[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$'; then
    printf 'a pinned semantic version such as v0.1.0 is required\n' >&2
    exit 2
fi

if [ -z "$install_dir" ]; then
    printf 'set HOME, IMPRESARI_INSTALL_DIR, or --install-dir\n' >&2
    exit 2
fi
if [ -L "$install_dir" ] || { [ -e "$install_dir" ] && [ ! -d "$install_dir" ]; }; then
    printf 'install directory must be a real directory, not a symlink or file\n' >&2
    exit 1
fi

system=$(uname -s)
machine=$(uname -m)
case "$system:$machine" in
    Darwin:arm64|Darwin:aarch64) target="aarch64-apple-darwin" ;;
    Linux:x86_64|Linux:amd64) target="x86_64-unknown-linux-gnu" ;;
    *)
        printf 'unsupported installer platform: %s %s\n' "$system" "$machine" >&2
        printf 'download the matching release archive manually\n' >&2
        exit 1
        ;;
esac

numeric_version=${release_version#v}
package="impresari-context-${numeric_version}-${target}"
archive="${package}.tar.gz"
base_url="https://github.com/${repository}/releases/download/${release_version}"
temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/impresari-install.XXXXXX")
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

download() {
    source_url=$1
    destination=$2
    if command -v curl >/dev/null 2>&1; then
        curl --fail --location --silent --show-error \
            --proto '=https' --tlsv1.2 \
            --output "$destination" "$source_url"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only --quiet --output-document="$destination" "$source_url"
    else
        printf 'curl or wget is required\n' >&2
        exit 1
    fi
}

download "${base_url}/${archive}" "${temporary_dir}/${archive}"
download "${base_url}/${archive}.sha256" "${temporary_dir}/${archive}.sha256"

expected_checksum=$(awk 'NR == 1 { print $1 }' "${temporary_dir}/${archive}.sha256")
if ! printf '%s\n' "$expected_checksum" | grep -Eq '^[0-9a-fA-F]{64}$'; then
    printf 'published checksum file is malformed\n' >&2
    exit 1
fi
if command -v shasum >/dev/null 2>&1; then
    actual_checksum=$(shasum -a 256 "${temporary_dir}/${archive}" | awk '{ print $1 }')
elif command -v sha256sum >/dev/null 2>&1; then
    actual_checksum=$(sha256sum "${temporary_dir}/${archive}" | awk '{ print $1 }')
else
    printf 'shasum or sha256sum is required\n' >&2
    exit 1
fi
if [ "$actual_checksum" != "$expected_checksum" ]; then
    printf 'release archive checksum mismatch\n' >&2
    exit 1
fi
printf '%s: OK\n' "$archive"

if tar -tzf "${temporary_dir}/${archive}" | grep -Ev "^${package}(/|$)" | grep -q .; then
    printf 'release archive contains an unexpected top-level path\n' >&2
    exit 1
fi
if tar -tzf "${temporary_dir}/${archive}" | grep -Eq '(^|/)\.\.(/|$)'; then
    printf 'release archive contains a traversal path\n' >&2
    exit 1
fi
tar -xzf "${temporary_dir}/${archive}" -C "$temporary_dir"
for binary in impresari-context impresari-context-mcp impresari-context-structural-worker; do
    source_path="${temporary_dir}/${package}/bin/${binary}"
    [ -f "$source_path" ] || { printf 'release archive is missing %s\n' "$binary" >&2; exit 1; }
    if [ -e "${install_dir}/${binary}" ]; then
        printf 'refusing to overwrite existing binary: %s\n' "${install_dir}/${binary}" >&2
        printf 'remove it explicitly or choose another --install-dir\n' >&2
        exit 1
    fi
done

mkdir -p "$install_dir"
for binary in impresari-context impresari-context-mcp impresari-context-structural-worker; do
    install -m 0755 "${temporary_dir}/${package}/bin/${binary}" "${install_dir}/${binary}"
done

printf 'Installed Impresari Context %s to %s\n' "$release_version" "$install_dir"
