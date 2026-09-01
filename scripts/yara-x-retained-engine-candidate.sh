#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo "YARA-X candidate retention is restricted to x86_64 Linux" >&2
  exit 3
fi
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "YARA-X candidate retention is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi
if [ "${GITHUB_REPOSITORY:-}" != tdloB/impresari-context ] || [ "${GITHUB_REF:-}" != refs/heads/main ]; then
  echo "YARA-X candidate retention requires canonical main" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
runtime_root="${RUNNER_TEMP:?RUNNER_TEMP is required}/impresari-yara-x-retained-${GITHUB_RUN_ID:?}-${GITHUB_RUN_ATTEMPT:-1}"
output_root="$repository_root/target/yara-x-retained-engine-candidate"
source_archive="$runtime_root/yara-x-source.tar.gz"
source_root="$runtime_root/yara-x"
cargo_home="$runtime_root/cargo-home"
metadata="$runtime_root/metadata.json"
tree="$runtime_root/dependency-tree.txt"
package_result="$runtime_root/package-result.json"
archive="$output_root/yara-x-v1.20.0-linux-x86_64-engine-candidate.tar.gz"
patch_file="$repository_root/third_party/yara-x/v1.20.0/impresari-module-free.patch"
profile="$repository_root/profiles/v1/yara-x-retained-engine-candidate-v1.json"
build_image='docker.io/library/rust@sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28'

cleanup() {
  if [ -e "$runtime_root" ] && [ ! -L "$runtime_root" ]; then
    chmod -R u+rwX "$runtime_root" || :
    rm -rf -- "$runtime_root"
  fi
}
trap cleanup EXIT HUP INT TERM
cleanup
rm -rf -- "$output_root"
mkdir -p -- "$runtime_root" "$source_root" "$cargo_home" "$output_root"

[ "$(sha256sum "$profile" | cut -d ' ' -f 1)" = c0fbe929ccb253eda0a93fc9adee77a4d9ca28827bd21bbdaaab7820874c71da ] || {
  echo "retention profile digest changed" >&2
  exit 4
}
[ "$(sha256sum "$patch_file" | cut -d ' ' -f 1)" = b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd ] || {
  echo "YARA-X patch digest changed" >&2
  exit 4
}

docker pull "$build_image"
[ "$(docker image inspect --format '{{.Os}}/{{.Architecture}}' "$build_image")" = linux/amd64 ] || {
  echo "build image platform changed" >&2
  exit 4
}
docker image inspect --format '{{json .RepoDigests}}' "$build_image" | grep -q 'sha256:7274e0edb5b47eda8053b350ebf3d489f7e0f65d2d7e77b16076299c7c047c28' || {
  echo "build image digest changed" >&2
  exit 4
}

curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$source_archive" \
  'https://codeload.github.com/VirusTotal/yara-x/tar.gz/60ad06971467029e77967e59d580cbbe85a1474d'
[ "$(wc -c < "$source_archive" | tr -d ' ')" = 57759292 ] || { echo "YARA-X source archive size changed" >&2; exit 4; }
[ "$(sha256sum "$source_archive" | cut -d ' ' -f 1)" = 8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee ] || {
  echo "YARA-X source archive digest changed" >&2
  exit 4
}
ruby "$repository_root/scripts/check-yara-x-source-archive.rb" "$source_archive"
tar -xzf "$source_archive" --no-same-owner --no-same-permissions --strip-components=1 -C "$source_root"
rm -f -- "$source_archive"
patch --batch --forward -p1 -d "$source_root" < "$patch_file"
[ "$(sha256sum "$source_root/Cargo.lock" | cut -d ' ' -f 1)" = e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e ] || {
  echo "patched YARA-X lockfile changed" >&2
  exit 5
}

uid=$(id -u)
gid=$(id -g)
docker run --rm --user "$uid:$gid" \
  -e HOME=/tmp/home -e CARGO_HOME=/cargo \
  --mount "type=bind,src=$source_root,dst=/usr/src/yara-x" \
  --mount "type=bind,src=$cargo_home,dst=/cargo" \
  --tmpfs /tmp:rw,nosuid,nodev,size=1073741824 \
  "$build_image" sh -ceu '
    mkdir -p "$HOME"
    rustc --version | grep -q "^rustc 1.93.0 "
    cd /usr/src/yara-x
    cargo fetch --locked --target x86_64-unknown-linux-gnu
    cargo install cargo-audit --version 0.22.2 --locked
    cargo audit --file Cargo.lock \
      --ignore RUSTSEC-2023-0071 \
      --ignore RUSTSEC-2026-0222 \
      --ignore RUSTSEC-2026-0269
    git -C /cargo/advisory-db rev-parse HEAD > /usr/src/yara-x/.impresari-advisory-db-commit
  '

docker run --rm --network none --read-only --user "$uid:$gid" \
  -e HOME=/tmp/home -e CARGO_HOME=/cargo -e CARGO_NET_OFFLINE=true \
  -e CARGO_INCREMENTAL=0 -e SOURCE_DATE_EPOCH=1787565021 \
  -e TZ=UTC -e LANG=C -e LC_ALL=C -e ZERO_AR_DATE=1 \
  -e 'RUSTFLAGS=-C target-feature=+crt-static' \
  --mount "type=bind,src=$source_root,dst=/usr/src/yara-x" \
  --mount "type=bind,src=$cargo_home,dst=/cargo,readonly" \
  --tmpfs /tmp:rw,nosuid,nodev,size=1073741824 \
  "$build_image" sh -ceu '
    mkdir -p "$HOME"
    cd /usr/src/yara-x
    cargo tree --offline --locked --target x86_64-unknown-linux-gnu \
      --package yara-x-cli --features pulley --prefix none --format "{p}" > /usr/src/yara-x/.impresari-dependency-tree
    for forbidden_dependency in "rsa v" "spin v" "x509-parser v" "wasmtime-wasi v" "cap-std v"; do
      if grep -q "$forbidden_dependency" /usr/src/yara-x/.impresari-dependency-tree; then
        echo "forbidden reachable YARA-X dependency: $forbidden_dependency" >&2
        exit 6
      fi
    done
    cargo metadata --offline --locked --format-version 1 \
      --filter-platform x86_64-unknown-linux-gnu > /usr/src/yara-x/.impresari-metadata.json
    cargo build --offline --frozen --locked --profile release-lto \
      --package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu
  '

mv -- "$source_root/.impresari-metadata.json" "$metadata"
mv -- "$source_root/.impresari-dependency-tree" "$tree"
advisory_db_commit=$(tr -d '\r\n' < "$source_root/.impresari-advisory-db-commit")
case "$advisory_db_commit" in (*[!0-9a-f]*|'') echo "invalid advisory database commit" >&2; exit 7;; esac
[ "${#advisory_db_commit}" = 40 ] || { echo "invalid advisory database commit length" >&2; exit 7; }

export IMPRESARI_ADVISORY_DB_COMMIT="$advisory_db_commit"
export IMPRESARI_RUNNER_KERNEL="$(uname -r)"
export IMPRESARI_RUNNER_ARCH="$(uname -m)"
ruby "$repository_root/scripts/package-yara-x-retained-engine-candidate.rb" \
  "$source_root" "$cargo_home" "$metadata" "$tree" "$archive" > "$package_result"
ruby "$repository_root/scripts/verify-yara-x-retained-engine-candidate.rb" "$archive"

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  ruby -rjson -e '
    result = JSON.parse(File.read(ARGV.fetch(0)))
    result.each { |key, value| puts "#{key}=#{value}" unless key == "claims" }
  ' "$package_result" >> "$GITHUB_OUTPUT"
fi
printf '%s\n' "YARA-X retained candidate built: archive=$(ruby -rjson -e 'puts JSON.parse(File.read(ARGV.fetch(0))).fetch("archive_sha256")' "$package_result") executed=false admitted=false production=false iar_2=false"
