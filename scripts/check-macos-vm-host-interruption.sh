#!/usr/bin/env sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  printf '%s\n' 'macOS local-VM host-interruption check is not applicable on this host'
  exit 0
fi

repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repo_root/target/iar-macos-vm-feasibility"
asset_root="$output_root/assets"
controller=$($repo_root/scripts/build-macos-vm-feasibility.sh)
cargo build --quiet --locked --manifest-path "$repo_root/Cargo.toml" \
  -p context-analyzer-runner --bin impresari-context-macos-vm-host-interruption
supervisor="$repo_root/target/debug/impresari-context-macos-vm-host-interruption"
controller_digest="sha256:$(shasum -a 256 "$controller" | awk '{print $1}')"
receipt="$output_root/host-interruption-receipt.json"

"$supervisor" "$controller" "$asset_root" "$controller_digest" \
  interrupt-proof > "$receipt"

jq -e --arg controller "$controller_digest" '
  .schema_name == "macos-local-vm-host-interruption-receipt" and
  .schema_version == "1.0.0" and
  .profile_id == "iar-macos-local-vm-interruption-v2" and
  .profile_digest == "sha256:f1b57b17d9de3b2b4de885732b6bef0f3cbf637bcba08dc1dda34724e9b18c4f" and
  .controller_profile_id == "iar-macos-local-vm-synthetic-matrix-v2" and
  .controller_profile_digest == "sha256:090aa47a283677599daeacba7af9628e1883b368a7bb7f81fedbda5a957f1888" and
  .controller_digest == $controller and
  .job_id == "interrupt-proof" and
  .interruption_source == "synthetic-job-private-trigger" and
  .sleep_observer_installed == true and
  .shared_stop_handler_used == true and
  .synthetic_interruption_requested == true and
  .virtual_machine_stopped == true and
  .controller_reaped == true and
  .stale_job_removed == true and
  .recovery_job_succeeded == true and
  .all_job_state_removed == true and
  .real_host_sleep_observed == false and
  .vm_confined == false and
  .production_admitted == false and
  .analyzer_execution == false and
  .source_retained == false and
  .authority_added == false
' "$receipt" >/dev/null

if find "$output_root/jobs" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null | grep -q .; then
  printf '%s\n' 'macOS VM host-interruption check retained per-job state' >&2
  exit 11
fi

printf 'macOS local-VM synthetic host-interruption passed: controller=%s real_host_sleep_observed=false\n' \
  "$controller_digest"
