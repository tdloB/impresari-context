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
envelope_build="$runtime_root/impresari-envelope-target"

cleanup() {
  for cleanup_root in "$runtime_root" "$stage_root" "$composite_yara_root"; do
    if [ -e "$cleanup_root" ] && [ ! -L "$cleanup_root" ]; then
      chmod -R u+rwX "$cleanup_root" || :
    fi
  done
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
verify_digest "$rule_source" 7769b61b7570e62f3b55eb615ffb5a6249862b9f267d1ad6305eda02e10d2c68

patch --batch --forward -p1 -d "$source_root" < "$patch_file"
verify_digest "$source_root/Cargo.toml" 4c250ee2588e0d46b8e83f5bc683c230d212e44cf2851b8f259e51752f85d24d
verify_digest "$source_root/cli/Cargo.toml" a141a064f49eedc1d2bd079e95f1ce187d7d9fba845f6e801ed7c44eaa378402
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

CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+crt-static' \
  CARGO_TARGET_DIR="$envelope_build" cargo +1.98.0 build --locked --release \
  --target x86_64-unknown-linux-gnu --package context-yara-x-envelope \
  --bin impresari-yara-x-live-synthetic-envelope
live_coordinator="$envelope_build/x86_64-unknown-linux-gnu/release/impresari-yara-x-live-synthetic-envelope"
[ -x "$live_coordinator" ] || { echo "YARA-X live coordinator build is missing" >&2; exit 8; }

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
verify_digest "$profile" a7757809eae545bea1fa08d64195262b4e99fae8c2f222af9c28dce04b195391
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
    hex) printf '\111\115\120\000\377\122\105\123\101\122\111\177\130\061' > "$case_root/input" ;;
    literal) printf '%s' 'IMPRESARI_SYNTHETIC_LITERAL_7A31C9' > "$case_root/input" ;;
    near_miss) printf '%s' 'IMPRESARI_SYNTHETIC_LITERAL_7A31C8' > "$case_root/input" ;;
    wide) ruby -e 'File.binwrite(ARGV.fetch(0), "IMPRESARI_SYNTHETIC_WIDE_91B6".encode("UTF-16LE"))' "$case_root/input" ;;
  esac
  chmod 0555 "$case_root/yr"
  chmod 0444 "$case_root/rules.yarc" "$case_root/input"
done

cd "$repository_root"
RUNNER_ENVIRONMENT=github-hosted YARA_X_LIVE_COORDINATOR="$live_coordinator" \
  ./scripts/check-linux-composite-feasibility.sh --yara-x-compatibility
ruby ./scripts/check-yara-x-artifact-compatibility.rb
ruby ./scripts/check-yara-x-live-compatibility-receipt.rb \
  "$composite_yara_root/receipt.json"
