#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo "YARA-X artifact compatibility is restricted to x86_64 Linux" >&2
  exit 3
fi
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "YARA-X artifact compatibility is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
runtime_root="${RUNNER_TEMP:?RUNNER_TEMP is required}/impresari-yara-x-compatibility-${GITHUB_RUN_ID:?}-${GITHUB_RUN_ATTEMPT:-1}"
stage_root="$repository_root/target/yara-x-artifact-compatibility"
composite_yara_root="$repository_root/target/iar-linux-composite-feasibility/yara-x"
archive="$runtime_root/source.tar.gz"
source_root="$runtime_root/yara-x-60ad06971467029e77967e59d580cbbe85a1474d"
profile="$repository_root/profiles/v1/yara-x-artifact-compatibility-v1.json"
patch_file="$repository_root/third_party/yara-x/v1.20.0/impresari-module-free.patch"
rule_source="$repository_root/rules/yara-x/synthetic-compatibility-v1.yar"

cleanup() {
  rm -rf -- "$runtime_root" "$stage_root" "$composite_yara_root"
}
trap cleanup EXIT HUP INT TERM
cleanup
mkdir -p -- "$runtime_root" "$stage_root/build-home" "$stage_root/cases" "$stage_root/external" "$stage_root/credential"

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
tar -xzf "$archive" --no-same-owner --no-same-permissions -C "$runtime_root"

verify_digest() {
  checked_path=$1
  expected_digest=$2
  [ -f "$checked_path" ] && [ ! -L "$checked_path" ] && \
    [ "$(sha256sum "$checked_path" | cut -d ' ' -f 1)" = "$expected_digest" ] || {
      echo "YARA-X source identity changed: $checked_path" >&2
      exit 5
    }
}

verify_digest "$source_root/Cargo.toml" 932156f9dde9714993c26659f44335a69d37eb9132c599bf30c3e474f3535a8c
verify_digest "$source_root/Cargo.lock" 2b58dd867d95c8854bc150fd8065bb443f368783e010adbbd5a6587d71f0039d
verify_digest "$source_root/LICENSE" fdf05444c9178e662fa28810d94a1fa6ec32d7be798241c98094213317265880
verify_digest "$patch_file" b0483e81f647e302afcc1acd88afbefb37ba03649187fbec46c6ab3adde542dd
verify_digest "$rule_source" 5379d03476eebf9c06379ad8d791d5ff1879c331300869d3eaf54c0e578c812b

patch --batch --forward -p1 -d "$source_root" < "$patch_file"
verify_digest "$source_root/Cargo.toml" 4c250ee2588e0d46b8e83f5bc683c230d212e44cf2851b8f259e51752f85d24d
verify_digest "$source_root/cli/Cargo.toml" bab4198a56220fd84c699ffdc36ad6e3b8f8b8326eb1638d41a62013d99b2e21
verify_digest "$source_root/Cargo.lock" e559620a158ed90c5cc6227beadd4242cc6d7d460c8211f373a523152a742b2e

ruby "$repository_root/scripts/check-yara-x-rule-policy.rb" "$rule_source"
for forbidden in import include regex xor base64 invalid; do
  case "$forbidden" in
    import) sample='import "pe"' ;;
    include) sample='include "other.yar"' ;;
    regex) sample='rule bad : synthetic { strings: $a = /a+/ condition: $a }' ;;
    xor) sample='rule bad : synthetic { strings: $a = "a" xor condition: $a }' ;;
    base64) sample='rule bad : synthetic { strings: $a = "a" base64 condition: $a }' ;;
    invalid) sample='rule BAD { condition: true }' ;;
  esac
  sample_path="$runtime_root/rejected-$forbidden.yar"
  printf '%s\n' "$sample" > "$sample_path"
  if ruby "$repository_root/scripts/check-yara-x-rule-policy.rb" "$sample_path" >/dev/null 2>&1; then
    echo "YARA-X rule policy accepted forbidden $forbidden fixture" >&2
    exit 6
  fi
done

export CARGO_HOME="$stage_root/build-home/cargo"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
cd "$source_root"
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
    exit 7
  fi
done
grep -q 'crossbeam-epoch v0.9.20' "$runtime_root/selected-tree.txt"
grep -q 'memmap2 v0.9.11' "$runtime_root/selected-tree.txt"
grep -q 'wasmtime v45.0.3' "$runtime_root/selected-tree.txt"

export RUSTFLAGS='-C target-feature=+crt-static'
cargo +1.93.0 build --frozen --locked --profile release-lto \
  --package yara-x-cli --features pulley --target x86_64-unknown-linux-gnu
yr="$source_root/target/x86_64-unknown-linux-gnu/release-lto/yr"
verify_digest "$profile" a269da948e6a379fa764579a751219be414337093414af729389613a00f04f41
[ -x "$yr" ] || { echo "YARA-X CLI build is missing" >&2; exit 8; }

compile_home="$stage_root/compile-home"
mkdir -p -- "$compile_home"
compiled_rules="$stage_root/rules.yarc"
env -i HOME="$compile_home" LANG=C LC_ALL=C \
  "$yr" compile -o "$compiled_rules" "$rule_source"
[ -f "$compiled_rules" ] && [ ! -L "$compiled_rules" ] || {
  echo "YARA-X compiled rules are missing" >&2
  exit 8
}
[ "$(wc -c < "$compiled_rules" | tr -d ' ')" -le 2097152 ] || {
  echo "YARA-X compiled rules exceed the compatibility limit" >&2
  exit 8
}

printf '%s\n' 'external-canary' > "$stage_root/external/canary"
printf '%s\n' 'credential-canary' > "$stage_root/credential/canary"
for case_id in empty hex literal near_miss wide; do
  case_root="$stage_root/cases/$case_id"
  mkdir -p -- "$case_root/home"
  cp "$yr" "$case_root/yr"
  cp "$compiled_rules" "$case_root/rules.yarc"
  chmod 0555 "$case_root/yr"
  case "$case_id" in
    empty) : > "$case_root/input" ;;
    hex) printf '%s' 'IMPRESARI_SYNTHETIC_HEX_4D28' > "$case_root/input" ;;
    literal) printf '%s' 'IMPRESARI_SYNTHETIC_LITERAL_7A31C9' > "$case_root/input" ;;
    near_miss) printf '%s' 'IMPRESARI_SYNTHETIC_LITERAL_7A31C8' > "$case_root/input" ;;
    wide) ruby -e 'File.binwrite(ARGV.fetch(0), "IMPRESARI_SYNTHETIC_WIDE_91B6".encode("UTF-16LE"))' "$case_root/input" ;;
  esac
  chmod -R a-w "$case_root"
done

cd "$repository_root"
RUNNER_ENVIRONMENT=github-hosted ./scripts/check-linux-composite-feasibility.sh --yara-x-compatibility
ruby ./scripts/check-yara-x-artifact-compatibility.rb
ruby ./scripts/check-yara-x-live-compatibility-receipt.rb \
  "$composite_yara_root/receipt.json"
