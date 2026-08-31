#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM supervisor lifecycle check is not applicable on this host'
  exit 0
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repo_root/target/iar-macos-vm-feasibility"
asset_root="$output_root/assets"
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
cargo build --quiet --locked --manifest-path "$repo_root/Cargo.toml" \
  -p context-analyzer-runner --bin impresari-context-macos-vm-synthetic-supervisor
supervisor="$repo_root/target/debug/impresari-context-macos-vm-synthetic-supervisor"
controller_digest="sha256:$(shasum -a 256 "$controller" | awk '{print $1}')"

cancel_receipt="$output_root/supervisor-external-cancellation.json"
forced_receipt="$output_root/supervisor-forced-termination.json"
matrix_receipt="$output_root/supervisor-lifecycle-receipt.json"

"$supervisor" "$controller" "$asset_root" "$controller_digest" \
  supervisor-cancel external-cancellation > "$cancel_receipt"
"$supervisor" "$controller" "$asset_root" "$controller_digest" \
  supervisor-kill forced-termination-recovery > "$forced_receipt"

validate_common() {
  receipt=$1
  job_id=$2
  jq -e --arg job_id "$job_id" --arg controller "$controller_digest" '
    .schema_name == "macos-local-vm-supervisor-lifecycle-receipt" and
    .schema_version == "1.0.0" and
    .profile_id == "iar-macos-local-vm-supervisor-v2" and
    .profile_digest == "sha256:614b9da42f051518e6a6d54f15e75c492e233e2ed653bfcbf69285d130967b88" and
    .controller_profile_id == "iar-macos-local-vm-synthetic-matrix-v2" and
    .controller_profile_digest == "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888" and
    .controller_digest == $controller and
    .job_id == $job_id and
    .controller_digest_verified_before_launch == true and
    .controller_ready == true and
    .controller_reaped == true and
    .stale_job_removed == true and
    .recovery_job_succeeded == true and
    .all_job_state_removed == true and
    .vm_confined == false and
    .production_admitted == false and
    .analyzer_execution == false and
    .source_retained == false and
    .authority_added == false
  ' "$receipt" >/dev/null
}

validate_common "$cancel_receipt" supervisor-cancel
jq -e '
  .action == "external-cancellation" and
  .external_cancellation_requested == true and
  .controller_cancellation_verified == true and
  .controller_forcibly_terminated == false
' "$cancel_receipt" >/dev/null

validate_common "$forced_receipt" supervisor-kill
jq -e '
  .action == "forced-termination-recovery" and
  .external_cancellation_requested == false and
  .controller_cancellation_verified == false and
  .controller_forcibly_terminated == true
' "$forced_receipt" >/dev/null

set +e
"$supervisor" "$controller" "$asset_root" \
  sha256:0000000000000000000000000000000000000000000000000000000000000000 \
  identity-reject external-cancellation >/dev/null 2>&1
identity_status=$?
set -e
if [ "$identity_status" -eq 0 ] || [ -e "$output_root/jobs/identity-reject" ]; then
  printf '%s\n' 'Rust supervisor accepted a mismatched controller digest or staged a job' >&2
  exit 11
fi

if find "$output_root/jobs" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null | grep -q .; then
  printf '%s\n' 'macOS VM supervisor lifecycle retained per-job state' >&2
  exit 12
fi

jq -n --arg controller "$controller_digest" '
  {
    schema_name:"macos-local-vm-supervisor-lifecycle-matrix",
    schema_version:"1.0.0",
    profile_id:"iar-macos-local-vm-supervisor-v2",
    profile_digest:"sha256:614b9da42f051518e6a6d54f15e75c492e233e2ed653bfcbf69285d130967b88",
    controller_digest:$controller,
    result:"partial_lifecycle_passed",
    checks:{
      external_cancellation:true,
      forced_controller_termination:true,
      exact_child_reap:true,
      exact_stale_job_removal:true,
      recovery_after_each_action:true,
      controller_digest_mismatch_rejected_before_staging:true,
      single_audited_process_launch_site:true
    },
    remaining:[
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
  }
' > "$matrix_receipt"

printf 'macOS local-VM Rust-supervisor lifecycle passed: controller=%s actions=2 recovery_jobs=2\n' \
  "$controller_digest"
