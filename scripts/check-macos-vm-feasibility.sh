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
first_initramfs_sha256=$(cat "$asset_root/impresari-initramfs.sha256")
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
initramfs_sha256=$(cat "$asset_root/impresari-initramfs.sha256")
if [ "$first_initramfs_sha256" != "$initramfs_sha256" ]; then
  printf '%s\n' 'repeated initramfs builds were not byte-identical' >&2
  exit 4
fi
receipt_one="$output_root/job-one-receipt.json"
receipt_two="$output_root/job-two-receipt.json"
receipt_recovery="$output_root/job-recovery-receipt.json"

entitlements=$(codesign -d --entitlements :- "$controller" 2>&1)
if ! printf '%s\n' "$entitlements" | grep -q '<key>com.apple.security.virtualization</key><true/>'; then
  printf '%s\n' 'controller lacks the exact virtualization entitlement' >&2
  exit 5
fi
if printf '%s\n' "$entitlements" | grep -q 'com.apple.security.network'; then
  printf '%s\n' 'controller unexpectedly has a network entitlement' >&2
  exit 6
fi

if [ "$initramfs_sha256" != 89c50636f21054dfcfd1761a1bfcf613df302960317876b3e137e1267b45397b ]; then
  printf '%s\n' 'built initramfs does not match the frozen matrix identity' >&2
  exit 7
fi

"$controller" "$asset_root" job-one success > "$receipt_one"
"$controller" "$asset_root" job-two success > "$receipt_two"

validate_receipt() {
  receipt=$1
  job_id=$2
  jq -e \
    --arg job_id "$job_id" \
    --arg initramfs "sha256:$initramfs_sha256" '
      .schema_name == "macos-local-vm-matrix-job-receipt" and
      .schema_version == "1.0.0" and
      .profile_id == "iar-macos-local-vm-synthetic-matrix-v2" and
      .profile_digest == "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888" and
      .job_id == $job_id and
      .result == "feasibility_passed" and
      .kernel_digest == "sha256:4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5" and
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

expect_failure() {
  scenario=$1
  category=$2
  receipt="$output_root/$scenario-failure.json"
  set +e
  "$controller" "$asset_root" "$scenario" "$scenario" > "$receipt"
  status=$?
  set -e
  if [ "$status" -eq 0 ]; then
    printf 'fault scenario unexpectedly succeeded: %s\n' "$scenario" >&2
    exit 9
  fi
  jq -e --arg category "$category" '
    .schema_name == "macos-local-vm-matrix-failure" and
    .schema_version == "1.0.0" and
    .profile_id == "iar-macos-local-vm-synthetic-matrix-v2" and
    .profile_digest == "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888" and
    .category == $category and
    .vm_confined == false and
    .production_admitted == false and
    .analyzer_execution == false and
    .source_retained == false and
    .authority_added == false
  ' "$receipt" >/dev/null
  if [ -e "$output_root/jobs/$scenario" ]; then
    printf 'fault scenario retained per-job state: %s\n' "$scenario" >&2
    exit 10
  fi
}

expect_failure malformed-result guest_failed
expect_failure output-flood output_limit
expect_failure timeout timeout
expect_failure descendant-timeout timeout
expect_failure early-exit guest_failed
expect_failure cancellation cancelled

tampered_assets="$output_root/tampered-assets"
mkdir -p "$tampered_assets"
cp "$asset_root/Image" "$tampered_assets/Image"
cp "$asset_root/impresari-initramfs.gz" "$tampered_assets/impresari-initramfs.gz"
printf 'x' >> "$tampered_assets/impresari-initramfs.gz"
tampered_receipt="$output_root/tampered-identity-failure.json"
set +e
"$controller" "$tampered_assets" tampered-identity success > "$tampered_receipt"
tampered_status=$?
set -e
if [ "$tampered_status" -eq 0 ]; then
  printf '%s\n' 'tampered guest identity unexpectedly succeeded' >&2
  exit 14
fi
jq -e '.schema_name == "macos-local-vm-matrix-failure" and .category == "invalid_identity" and .analyzer_execution == false and .source_retained == false and .authority_added == false' \
  "$tampered_receipt" >/dev/null
if [ -e "$output_root/jobs/tampered-identity" ]; then
  printf '%s\n' 'identity rejection staged a job' >&2
  exit 15
fi

"$controller" "$asset_root" job-recovery success > "$receipt_recovery"
validate_receipt "$receipt_recovery" job-recovery

matrix_receipt="$output_root/synthetic-matrix-receipt.json"
jq -n \
  --arg initramfs "sha256:$initramfs_sha256" \
  '{
    schema_name:"macos-local-vm-synthetic-matrix-receipt",
    schema_version:"1.0.0",
    profile_id:"iar-macos-local-vm-synthetic-matrix-v2",
    profile_digest:"sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888",
    kernel_digest:"sha256:4c78ec153e7b8cf17011d44423ec2e11c9618933d4b931c60e63c240bf6db2f5",
    initramfs_digest:$initramfs,
    result:"partial_matrix_passed",
    checks:{
      repeated_build_identity:true,
      exact_guest_identity:true,
      success_recovery:true,
      malformed_result_rejected:true,
      serial_flood_bounded_and_rejected:true,
      timeout_stopped_and_removed:true,
      descendant_vm_stopped_and_removed:true,
      early_exit_rejected_and_removed:true,
      controller_cancellation_stopped_and_removed:true,
      tampered_guest_rejected_before_staging:true,
      cross_job_state_absent:true
    },
    remaining:[
      "external_supervisor_cancellation",
      "forced_host_process_termination_recovery",
      "host_sleep_and_interruption",
      "guest_memory_pressure",
      "guest_cpu_accounting",
      "host_canary_denial_corpus",
      "multi_host_evidence",
      "distribution_and_independent_review"
    ],
    vm_confined:false,
    production_admitted:false,
    analyzer_execution:false,
    source_retained:false,
    authority_added:false
  }' > "$matrix_receipt"

printf 'macOS local-VM synthetic matrix passed: initramfs=sha256:%s success_jobs=3 fault_jobs=7\n' "$initramfs_sha256"
