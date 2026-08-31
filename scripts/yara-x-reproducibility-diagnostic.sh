#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo "YARA-X reproducibility diagnostic is restricted to x86_64 Linux" >&2
  exit 3
fi
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "YARA-X reproducibility diagnostic is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
runtime_root="${RUNNER_TEMP:?RUNNER_TEMP is required}/impresari-yara-x-reproducibility-${GITHUB_RUN_ID:?}-${GITHUB_RUN_ATTEMPT:-1}"
archive="$runtime_root/source.tar.gz"
source_name="yara-x-60ad06971467029e77967e59d580cbbe85a1474d"
patch_file="$repository_root/third_party/yara-x/v1.20.0/impresari-module-free.patch"
profile="$repository_root/profiles/v1/yara-x-reproducibility-diagnostic-v1.json"
receipt="$runtime_root/receipt.json"
cargo_home="$runtime_root/cargo-home"

cleanup() {
  if [ -e "$runtime_root" ] && [ ! -L "$runtime_root" ]; then
    chmod -R u+rwX "$runtime_root" || :
    rm -rf -- "$runtime_root"
  fi
}
trap cleanup EXIT HUP INT TERM
cleanup
mkdir -p -- "$runtime_root" "$cargo_home"

curl --fail --location --proto '=https' --tlsv1.2 \
  --output "$archive" \
  'https://codeload.github.com/VirusTotal/yara-x/tar.gz/60ad06971467029e77967e59d580cbbe85a1474d'
[ "$(wc -c < "$archive" | tr -d ' ')" = 57759292 ] || {
  echo "YARA-X source archive size changed" >&2
  exit 4
}
[ "$(sha256sum "$archive" | cut -d ' ' -f 1)" = 8a85bf120eeb6483e012aed6ca610782f961556a712e259b6b3fa63137b760ee ] || {
  echo "YARA-X source archive digest changed" >&2
  exit 4
}
ruby "$repository_root/scripts/check-yara-x-source-archive.rb" "$archive"
[ "$(sha256sum "$profile" | cut -d ' ' -f 1)" = 4948ca0a448f1083cc3fe52519b57f62555c319146e91ff0999f696d69a8dbf4 ] || {
  echo "YARA-X reproducibility profile digest changed" >&2
  exit 4
}
[ "$(sha256sum "$patch_file" | cut -d ' ' -f 1)" = b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd ] || {
  echo "YARA-X patch digest changed" >&2
  exit 4
}

for build_id in baseline-a baseline-b canonical-a canonical-b; do
  build_root="$runtime_root/$build_id"
  mkdir -p -- "$build_root/source" "$build_root/target"
  tar -xzf "$archive" --no-same-owner --no-same-permissions \
    --strip-components=1 -C "$build_root/source"
  patch --batch --forward -p1 -d "$build_root/source" < "$patch_file"
  [ "$(sha256sum "$build_root/source/Cargo.lock" | cut -d ' ' -f 1)" = e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e ] || {
    echo "patched YARA-X lockfile changed for $build_id" >&2
    exit 5
  }
done
rm -f -- "$archive"

export CARGO_HOME="$cargo_home"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
cd "$runtime_root/baseline-a/source"
cargo +1.93.0 fetch --locked --target x86_64-unknown-linux-gnu
cargo audit --file Cargo.lock \
  --ignore RUSTSEC-2023-0071 \
  --ignore RUSTSEC-2026-0222 \
  --ignore RUSTSEC-2026-0269
cargo +1.93.0 tree --locked --target x86_64-unknown-linux-gnu \
  --package yara-x-cli --features pulley > "$runtime_root/selected-tree.txt"
for forbidden_dependency in 'rsa v' 'spin v' 'x509-parser v' 'wasmtime-wasi v' 'cap-std v'; do
  if grep -q "$forbidden_dependency" "$runtime_root/selected-tree.txt"; then
    echo "forbidden reachable YARA-X dependency: $forbidden_dependency" >&2
    exit 6
  fi
done
grep -q 'crossbeam-epoch v0.9.20' "$runtime_root/selected-tree.txt"
grep -q 'memmap2 v0.9.11' "$runtime_root/selected-tree.txt"
grep -q 'wasmtime v45.0.3' "$runtime_root/selected-tree.txt"

build_one() {
  build_id=$1
  mode=$2
  build_root="$runtime_root/$build_id"
  source_root="$build_root/source"
  target_root="$build_root/target"
  rustflags='-C target-feature=+crt-static'
  if [ "$mode" = canonical ]; then
    rustflags="$rustflags --remap-path-prefix=$source_root=/usr/src/yara-x --remap-path-prefix=$target_root=/usr/src/yara-x/target"
  fi
  (
    cd "$source_root"
    if [ "$mode" = canonical ]; then
      CARGO_NET_OFFLINE=true CARGO_INCREMENTAL=0 \
      SOURCE_DATE_EPOCH=1787565021 TZ=UTC LANG=C LC_ALL=C ZERO_AR_DATE=1 \
      RUSTFLAGS="$rustflags" CARGO_TARGET_DIR="$target_root" \
        cargo +1.93.0 build --offline --frozen --locked --profile release-lto \
          --package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu
    else
      CARGO_NET_OFFLINE=true RUSTFLAGS="$rustflags" CARGO_TARGET_DIR="$target_root" \
        cargo +1.93.0 build --offline --frozen --locked --profile release-lto \
          --package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu
    fi
  )
  executable="$target_root/x86_64-unknown-linux-gnu/release-lto/yr"
  [ -x "$executable" ] || { echo "YARA-X build output missing for $build_id" >&2; exit 7; }
  sha256sum "$executable" | cut -d ' ' -f 1
}

baseline_a=$(build_one baseline-a baseline)
baseline_b=$(build_one baseline-b baseline)
canonical_a=$(build_one canonical-a canonical)
canonical_b=$(build_one canonical-b canonical)

ruby "$repository_root/scripts/yara-x-reproducibility-receipt.rb" emit \
  "$baseline_a" "$baseline_b" "$canonical_a" "$canonical_b" > "$receipt"
ruby "$repository_root/scripts/yara-x-reproducibility-receipt.rb" verify "$receipt"
result=$(ruby -rjson -e 'puts JSON.parse(File.read(ARGV.fetch(0))).fetch("result")' "$receipt")
printf '%s\n' "YARA-X reproducibility diagnostic: result=$result baseline_a=sha256:$baseline_a baseline_b=sha256:$baseline_b canonical_a=sha256:$canonical_a canonical_b=sha256:$canonical_b analyzer_executed=false artifact_uploaded=false production=false iar_2=false"
