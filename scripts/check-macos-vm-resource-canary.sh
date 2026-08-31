#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM resource/canary check is not applicable on this host'
  exit 0
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repo_root/target/iar-macos-vm-feasibility"
asset_root="$output_root/assets"
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
first_initramfs_sha256=$(cat "$asset_root/impresari-resource-initramfs.sha256")
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
initramfs_sha256=$(cat "$asset_root/impresari-resource-initramfs.sha256")
if [ "$first_initramfs_sha256" != "$initramfs_sha256" ] || \
   [ "$initramfs_sha256" != 1a4029b781020260e4cb8c18271e3a01e1920f1448d87a71678e12cc617a1ec3 ]; then
  printf '%s\n' 'resource/canary initramfs did not reproduce its frozen identity' >&2
  exit 4
fi

cargo build --quiet --locked --manifest-path "$repo_root/Cargo.toml" \
  -p context-analyzer-runner --bin impresari-context-macos-vm-resource-canary
supervisor="$repo_root/target/debug/impresari-context-macos-vm-resource-canary"
controller_digest="sha256:$(shasum -a 256 "$controller" | awk '{print $1}')"
receipt="$output_root/resource-canary-supervisor-receipt.json"

"$supervisor" "$controller" "$asset_root" "$controller_digest" resource-canary > "$receipt"

jq -e --arg controller "$controller_digest" '
  .schema_name == "macos-local-vm-resource-canary-supervisor-receipt" and
  .schema_version == "1.0.0" and
  .profile_id == "iar-macos-local-vm-resource-canary-v2" and
  .profile_digest == "sha256:82d3cbf4b68866b92794a06e86948ccaf2492b3b4cb38e7e70503562c61d1de0" and
  .controller_digest == $controller and
  .job_id == "resource-canary" and
  .kernel_digest == "sha256:4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5" and
  .initramfs_digest == "sha256:1a4029b781020260e4cb8c18271e3a01e1920f1448d87a71678e12cc617a1ec3" and
  .input_digest == "sha256:3050d67653f05f1db0dcef073a64f6fc9f9ac2e55c7b1881e7372151b3e4fd99" and
  .controller_digest_verified_before_launch == true and
  .configuration_validated == true and
  .cpu_count == "1" and
  .memory_bytes == "268435456" and
  .storage_devices == "2" and
  .network_devices == "0" and
  .directory_shares == "0" and
  .host_canary_corpus_created == true and
  .host_canary_corpus_unchanged == true and
  .attached_device_set_exact == true and
  .host_canary_bytes_absent == true and
  .host_paths_absent == true and
  .host_process_invisible == true and
  .memory_pressure_contained == true and
  .memory_oom_kills == "1" and
  .cpu_pressure_bounded == true and
  (.cpu_usage_usec | test("^[1-9][0-9]*$") and tonumber >= 50000 and tonumber <= 400000) and
  (.cpu_throttled_periods | test("^[1-9][0-9]*$")) and
  .pids_peak == "1" and
  .job_cgroup_removed == true and
  .job_removed == true and
  .vm_confined == false and
  .production_admitted == false and
  .analyzer_execution == false and
  .source_retained == false and
  .authority_added == false
' "$receipt" >/dev/null

set +e
"$supervisor" "$controller" "$asset_root" \
  sha256:0000000000000000000000000000000000000000000000000000000000000000 \
  resource-reject >/dev/null 2>&1
identity_status=$?
set -e
if [ "$identity_status" -eq 0 ] || [ -e "$output_root/jobs/resource-reject" ]; then
  printf '%s\n' 'resource/canary supervisor accepted the wrong controller identity' >&2
  exit 5
fi
if find "$output_root/jobs" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null | grep -q .; then
  printf '%s\n' 'resource/canary check retained per-job state' >&2
  exit 6
fi

printf 'macOS local-VM resource/canary evidence passed: profile=sha256:%s controller=%s\n' \
  82d3cbf4b68866b92794a06e86948ccaf2492b3b4cb38e7e70503562c61d1de0 \
  "$controller_digest"
