#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM feasibility check is not applicable on this host'
  exit 0
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repo_root/target/iar-macos-vm-feasibility"
asset_root="$output_root/assets"
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
initramfs_sha256=$(cat "$asset_root/impresari-initramfs.sha256")
receipt_one="$output_root/job-one-receipt.json"
receipt_two="$output_root/job-two-receipt.json"

entitlements=$(codesign -d --entitlements :- "$controller" 2>&1)
if ! printf '%s\n' "$entitlements" | grep -q '<key>com.apple.security.virtualization</key><true/>'; then
  printf '%s\n' 'controller lacks the exact virtualization entitlement' >&2
  exit 5
fi
if printf '%s\n' "$entitlements" | grep -q 'com.apple.security.network'; then
  printf '%s\n' 'controller unexpectedly has a network entitlement' >&2
  exit 6
fi

"$controller" "$asset_root" "$initramfs_sha256" job-one > "$receipt_one"
"$controller" "$asset_root" "$initramfs_sha256" job-two > "$receipt_two"

validate_receipt() {
  receipt=$1
  job_id=$2
  jq -e \
    --arg job_id "$job_id" \
    --arg initramfs "sha256:$initramfs_sha256" '
      .schema_name == "macos-local-vm-feasibility-receipt" and
      .schema_version == "1.0.0" and
      .profile_id == "iar-macos-local-vm-feasibility-v1" and
      .profile_digest == "sha256:a082df092d5180058f732d47ae99164316f3bfd3b12f4079de43575834314757" and
      .job_id == $job_id and
      .result == "feasibility_passed" and
      .kernel_digest == "sha256:8b216f74e7f89def4604adf69e2345437363aff4819101bb1551c9e83cd35cdd" and
      .initramfs_digest == $initramfs and
      (.input_digest | test("^sha256:[0-9a-f]{64}$")) and
      .virtualization_supported == true and
      .configuration_validated == true and
      .cpu_count == "1" and
      .memory_bytes == "268435456" and
      .serial_ports == "1" and
      .storage_devices == "2" and
      .network_devices == "0" and
      .directory_shares == "0" and
      .graphics_devices == "0" and
      .audio_devices == "0" and
      .input_devices == "0" and
      .exact_input_verified == true and
      .read_only_input_verified == true and
      .scratch_initially_clean == true and
      .scratch_capacity_verified == true and
      .network_device_absent == true and
      .job_removed == true and
      .vm_confined == false and
      .production_admitted == false and
      .source_retained == false and
      .authority_added == false
    ' "$receipt" >/dev/null
}

validate_receipt "$receipt_one" job-one
validate_receipt "$receipt_two" job-two

if [ "$(jq -r .input_digest "$receipt_one")" != "$(jq -r .input_digest "$receipt_two")" ]; then
  printf '%s\n' 'synthetic input identity changed across jobs' >&2
  exit 7
fi
if [ -e "$output_root/jobs/job-one" ] || [ -e "$output_root/jobs/job-two" ]; then
  printf '%s\n' 'per-job VM state survived controller cleanup' >&2
  exit 8
fi

printf 'macOS local-VM feasibility passed: initramfs=sha256:%s jobs=2\n' "$initramfs_sha256"
