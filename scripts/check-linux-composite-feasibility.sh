#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "Linux composite IAR-1B feasibility check is not applicable on this host"
  exit 0
fi

if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "Linux composite IAR-1B feasibility is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repository_root/target/iar-linux-composite-feasibility"
isolation_probe="$output_root/linux-isolation-probe"
resource_probe="$output_root/linux-cgroup-probe"
receipt="$output_root/receipt.json"
profile="$repository_root/profiles/v1/iar-linux-synthetic-v1.json"

if [ "${1:-}" != "--delegated" ]; then
  command -v systemd-run >/dev/null 2>&1 || {
    echo "systemd-run is required for this CI-only checkpoint" >&2
    exit 3
  }
  command -v sudo >/dev/null 2>&1 || {
    echo "sudo is required only to create the transient delegated CI service" >&2
    exit 3
  }
  unit="impresari-iar-composite-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}-$$"
  exec sudo systemd-run --quiet --wait --pipe --collect --service-type=exec \
    --unit="$unit" --property=Delegate=yes \
    --setenv=GITHUB_ACTIONS=true --setenv=RUNNER_ENVIRONMENT=github-hosted \
    --uid="$(id -u)" --gid="$(id -g)" \
    --working-directory="$repository_root" \
    "$repository_root/scripts/check-linux-composite-feasibility.sh" --delegated
fi

rm -rf -- "$output_root"
mkdir -p -- "$output_root/job" "$output_root/external" "$output_root/credential"
printf '%s\n' 'synthetic-job-input' > "$output_root/job/input"
printf '%s\n' 'synthetic-external-canary' > "$output_root/external/canary"
printf '%s\n' 'synthetic-credential-canary' > "$output_root/credential/canary"

cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
  "$repository_root/platform/linux-isolation-feasibility/probe.c" \
  -o "$isolation_probe"
cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
  "$repository_root/platform/linux-cgroup-feasibility/probe.c" \
  -o "$resource_probe"

cgroup_suffix=$(awk -F: '$1 == "0" { print $3 }' /proc/self/cgroup)
case "$cgroup_suffix" in /*) ;; *) echo "invalid cgroup identity" >&2; exit 4 ;; esac
case "$cgroup_suffix" in *..*) echo "unsafe cgroup identity" >&2; exit 4 ;; esac
delegation_root="/sys/fs/cgroup$cgroup_suffix"
supervisor_cgroup="$delegation_root/impresari-supervisor"
mkdir "$supervisor_cgroup"
printf '%s\n' "$$" > "$supervisor_cgroup/cgroup.procs"
printf '%s\n' '+cpu +memory +pids' > "$delegation_root/cgroup.subtree_control"

cpu_quota=$(jq -r '.limits.cpu_quota_us' "$profile")
cpu_period=$(jq -r '.limits.cpu_period_us' "$profile")
memory_bytes=$(jq -r '.limits.memory_bytes' "$profile")
processes=$(jq -r '.limits.processes' "$profile")
output_limit=$(jq -r '.limits.output_bytes' "$profile")

create_leaf() {
  leaf_path=$1
  mkdir "$leaf_path"
  printf '%s %s\n' "$cpu_quota" "$cpu_period" > "$leaf_path/cpu.max"
  printf '%s\n' "$memory_bytes" > "$leaf_path/memory.max"
  printf '%s\n' "$processes" > "$leaf_path/pids.max"
  if [ -f "$leaf_path/memory.swap.max" ]; then
    printf '%s\n' '0' > "$leaf_path/memory.swap.max"
  fi
  if [ -f "$leaf_path/memory.oom.group" ]; then
    printf '%s\n' '1' > "$leaf_path/memory.oom.group"
  fi
}

wait_empty() {
  leaf_path=$1
  attempts=0
  while [ "$attempts" -lt 200 ]; do
    if grep -qx 'populated 0' "$leaf_path/cgroup.events"; then
      return 0
    fi
    attempts=$((attempts + 1))
    sleep 0.01
  done
  return 1
}

remove_leaf() {
  wait_empty "$1"
  rmdir "$1"
}

capabilities=$($isolation_probe capabilities)
capability_value() {
  capability_name=$1
  printf '%s\n' "$capabilities" | sed -n "s/^${capability_name}=//p" | tail -1
}
checked_boolean() {
  checked_value=$1
  checked_name=$2
  case "$checked_value" in
    true|false) printf '%s\n' "$checked_value" ;;
    *) echo "invalid boolean from Linux probe: $checked_name" >&2; exit 5 ;;
  esac
}

kernel_release=$(capability_value kernel_release)
architecture=$(capability_value architecture)
landlock_abi=$(capability_value landlock_abi)
no_new_privs=$(checked_boolean "$(capability_value no_new_privs)" no_new_privs)
landlock=$(checked_boolean "$(capability_value landlock)" landlock)
seccomp_filter=$(checked_boolean "$(capability_value seccomp_filter)" seccomp_filter)
seccomp_kill_process=$(checked_boolean "$(capability_value seccomp_kill_process)" seccomp_kill_process)
architecture_filter=$(checked_boolean "$(capability_value architecture_filter)" architecture_filter)

controllers=$(cat "$delegation_root/cgroup.subtree_control")
case " $controllers " in *" cpu "*) cpu_controller=true ;; *) cpu_controller=false ;; esac
case " $controllers " in *" memory "*) memory_controller=true ;; *) memory_controller=false ;; esac
case " $controllers " in *" pids "*) pids_controller=true ;; *) pids_controller=false ;; esac
cgroup_v2=false; [ -f "$delegation_root/cgroup.controllers" ] && cgroup_v2=true
delegated_leaf=false; [ -w "$delegation_root/cgroup.subtree_control" ] && delegated_leaf=true

composite_leaf="$delegation_root/job-composite"
create_leaf "$composite_leaf"
cgroup_kill=false; [ -f "$composite_leaf/cgroup.kill" ] && cgroup_kill=true
cgroup_empty_verification=false; [ -r "$composite_leaf/cgroup.events" ] && cgroup_empty_verification=true
resource_profile_applied=false
if [ "$(cat "$composite_leaf/cpu.max")" = "$cpu_quota $cpu_period" ] && \
   [ "$(cat "$composite_leaf/memory.max")" = "$memory_bytes" ] && \
   [ "$(cat "$composite_leaf/pids.max")" = "$processes" ]; then
  resource_profile_applied=true
fi

primitive_output="$output_root/composite-worker.out"
if $isolation_probe composite "$composite_leaf" \
  "$output_root/job" "$output_root/job/input" \
  "$output_root/external/canary" "$output_root/credential/canary" \
  "$output_root/job/write-probe" > "$primitive_output"; then
  atomic_composite_worker=true
else
  atomic_composite_worker=false
fi

primitive_value() {
  primitive_name=$1
  sed -n "s/^${primitive_name}=//p" "$primitive_output" | tail -1
}
no_new_privs_effective=$(checked_boolean "$(primitive_value no_new_privs_effective)" no_new_privs_effective)
landlock_read_only_input=$(checked_boolean "$(primitive_value landlock_read_only_input)" landlock_read_only_input)
external_filesystem_denial=$(checked_boolean "$(primitive_value external_filesystem_denial)" external_filesystem_denial)
credential_denial=$(checked_boolean "$(primitive_value credential_denial)" credential_denial)
device_denial=$(checked_boolean "$(primitive_value device_denial)" device_denial)
network_denial=$(checked_boolean "$(primitive_value network_denial)" network_denial)
unrelated_descriptors_closed=$(checked_boolean "$(primitive_value unrelated_descriptors_closed)" unrelated_descriptors_closed)
descendant_denial=$(checked_boolean "$(primitive_value descendant_denial)" descendant_denial)
zero_writable_filesystem=$(checked_boolean "$(primitive_value zero_writable_filesystem)" zero_writable_filesystem)
remove_leaf "$composite_leaf"

cpu_limit=false memory_limit=false process_count_limit=false
exact_cgroup_kill=false cgroup_empty_after_job=false bounded_output=false
timeout=false crash_relaunch=false cleanup=false cross_job_isolation=false

leaf="$delegation_root/job-pids"
create_leaf "$leaf"
if [ "$($resource_probe run "$leaf" pids)" = denied ]; then process_count_limit=true; fi
remove_leaf "$leaf"

leaf="$delegation_root/job-memory"
create_leaf "$leaf"
oom_before=$(awk '$1 == "oom_kill" { print $2 }' "$leaf/memory.events.local")
if $resource_probe run "$leaf" memory; then
  oom_after=$(awk '$1 == "oom_kill" { print $2 }' "$leaf/memory.events.local")
  if [ "$oom_after" -gt "$oom_before" ]; then memory_limit=true; fi
fi
remove_leaf "$leaf"

leaf="$delegation_root/job-cpu"
create_leaf "$leaf"
if $resource_probe run "$leaf" cpu; then cpu_limit=true; fi
remove_leaf "$leaf"

leaf="$delegation_root/job-output"
create_leaf "$leaf"
set +e
$resource_probe run "$leaf" flood | head -c "$output_limit" | wc -c > "$output_root/output-count" &
pipeline_pid=$!
set -e
wait "$pipeline_pid" || true
if [ "$(tr -d ' ' < "$output_root/output-count")" = "$output_limit" ]; then bounded_output=true; fi
wait_empty "$leaf"
remove_leaf "$leaf"

leaf="$delegation_root/job-timeout"
create_leaf "$leaf"
$resource_probe run "$leaf" sleep & timeout_parent=$!
sleep 0.25
started=$(date +%s)
printf '%s\n' '1' > "$leaf/cgroup.kill"
if wait "$timeout_parent" && wait_empty "$leaf"; then
  finished=$(date +%s)
  if [ $((finished - started)) -lt 5 ]; then timeout=true; exact_cgroup_kill=true; fi
fi
if wait_empty "$leaf"; then cgroup_empty_after_job=true; fi
remove_leaf "$leaf"

crash_leaf="$delegation_root/job-crash"
relaunch_leaf="$delegation_root/job-relaunch"
create_leaf "$crash_leaf"
if $resource_probe run "$crash_leaf" crash && remove_leaf "$crash_leaf"; then
  create_leaf "$relaunch_leaf"
  if [ "$($resource_probe run "$relaunch_leaf" ok)" = ok ]; then crash_relaunch=true; fi
  remove_leaf "$relaunch_leaf"
fi

first_leaf="$delegation_root/job-a"
second_leaf="$delegation_root/job-b"
create_leaf "$first_leaf"
create_leaf "$second_leaf"
$resource_probe run "$first_leaf" sleep & first_parent=$!
$resource_probe run "$second_leaf" sleep & second_parent=$!
sleep 0.1
printf '%s\n' '1' > "$first_leaf/cgroup.kill"
if wait "$first_parent" && kill -0 "$second_parent" 2>/dev/null && \
   grep -qx 'populated 1' "$second_leaf/cgroup.events"; then
  cross_job_isolation=true
fi
printf '%s\n' '1' > "$second_leaf/cgroup.kill"
wait "$second_parent"
remove_leaf "$first_leaf"
remove_leaf "$second_leaf"

remaining=$(find "$delegation_root" -mindepth 1 -maxdepth 1 -type d ! -name impresari-supervisor | wc -l | tr -d ' ')
if [ "$remaining" = 0 ]; then cleanup=true; fi

result=failed
os_confined=false
limitations='["cgroup-resource-suite-pending","lifecycle-suite-pending","single-host-evidence","synthetic-probe-no-analysis"]'
if [ "$no_new_privs" = true ] && [ "$landlock" = true ] && \
   [ "$seccomp_filter" = true ] && [ "$seccomp_kill_process" = true ] && \
   [ "$architecture_filter" = true ] && [ "$cgroup_v2" = true ] && \
   [ "$cpu_controller" = true ] && [ "$memory_controller" = true ] && \
   [ "$pids_controller" = true ] && [ "$delegated_leaf" = true ] && \
   [ "$cgroup_kill" = true ] && [ "$cgroup_empty_verification" = true ] && \
   [ "$resource_profile_applied" = true ] && [ "$atomic_composite_worker" = true ] && \
   [ "$no_new_privs_effective" = true ] && [ "$landlock_read_only_input" = true ] && \
   [ "$external_filesystem_denial" = true ] && [ "$credential_denial" = true ] && \
   [ "$device_denial" = true ] && [ "$network_denial" = true ] && \
   [ "$unrelated_descriptors_closed" = true ] && [ "$descendant_denial" = true ] && \
   [ "$zero_writable_filesystem" = true ] && [ "$cpu_limit" = true ] && \
   [ "$memory_limit" = true ] && [ "$process_count_limit" = true ] && \
   [ "$exact_cgroup_kill" = true ] && [ "$cgroup_empty_after_job" = true ] && \
   [ "$bounded_output" = true ] && [ "$timeout" = true ] && \
   [ "$crash_relaunch" = true ] && [ "$cleanup" = true ] && \
   [ "$cross_job_isolation" = true ]; then
  result=candidate_passed
  os_confined=true
  limitations='["single-host-evidence","synthetic-probe-no-analysis"]'
fi

profile_digest="sha256:$(sha256sum "$profile" | awk '{print $1}')"
jq -n \
  --arg profile_digest "$profile_digest" --arg kernel_release "$kernel_release" \
  --arg architecture "$architecture" --arg landlock_abi "$landlock_abi" \
  --arg result "$result" --argjson no_new_privs "$no_new_privs" \
  --argjson landlock "$landlock" --argjson seccomp_filter "$seccomp_filter" \
  --argjson seccomp_kill_process "$seccomp_kill_process" \
  --argjson architecture_filter "$architecture_filter" --argjson cgroup_v2 "$cgroup_v2" \
  --argjson cpu_controller "$cpu_controller" --argjson memory_controller "$memory_controller" \
  --argjson pids_controller "$pids_controller" --argjson delegated_leaf "$delegated_leaf" \
  --argjson cgroup_kill "$cgroup_kill" --argjson cgroup_empty_verification "$cgroup_empty_verification" \
  --argjson atomic_composite_worker "$atomic_composite_worker" \
  --argjson resource_profile_applied "$resource_profile_applied" \
  --argjson no_new_privs_effective "$no_new_privs_effective" \
  --argjson landlock_read_only_input "$landlock_read_only_input" \
  --argjson external_filesystem_denial "$external_filesystem_denial" \
  --argjson credential_denial "$credential_denial" --argjson device_denial "$device_denial" \
  --argjson network_denial "$network_denial" \
  --argjson unrelated_descriptors_closed "$unrelated_descriptors_closed" \
  --argjson descendant_denial "$descendant_denial" \
  --argjson zero_writable_filesystem "$zero_writable_filesystem" \
  --argjson cpu_limit "$cpu_limit" --argjson memory_limit "$memory_limit" \
  --argjson process_count_limit "$process_count_limit" --argjson exact_cgroup_kill "$exact_cgroup_kill" \
  --argjson cgroup_empty_after_job "$cgroup_empty_after_job" --argjson bounded_output "$bounded_output" \
  --argjson timeout "$timeout" --argjson crash_relaunch "$crash_relaunch" \
  --argjson cleanup "$cleanup" --argjson cross_job_isolation "$cross_job_isolation" \
  --argjson limitations "$limitations" --argjson os_confined "$os_confined" '
  {
    schema_name:"linux-isolation-feasibility",schema_version:"1.0.0",
    prototype_id:"iar-linux-isolation-feasibility-v1",profile_id:"iar-linux-synthetic-v1",
    profile_digest:$profile_digest,
    observed_host:{operating_system:"linux",kernel_release:$kernel_release,architecture:$architecture,landlock_abi:$landlock_abi},
    result:$result,
    preflight:{no_new_privs:$no_new_privs,landlock:$landlock,seccomp_filter:$seccomp_filter,
      seccomp_kill_process:$seccomp_kill_process,architecture_filter:$architecture_filter,cgroup_v2:$cgroup_v2,
      cpu_controller:$cpu_controller,memory_controller:$memory_controller,pids_controller:$pids_controller,
      delegated_leaf:$delegated_leaf,cgroup_kill:$cgroup_kill,cgroup_empty_verification:$cgroup_empty_verification},
    checks:{atomic_cgroup_placement:$atomic_composite_worker,resource_profile_applied:$resource_profile_applied,
      no_new_privs_effective:$no_new_privs_effective,landlock_read_only_input:$landlock_read_only_input,
      external_filesystem_denial:$external_filesystem_denial,credential_denial:$credential_denial,
      device_denial:$device_denial,network_denial:$network_denial,
      unrelated_descriptors_closed:$unrelated_descriptors_closed,descendant_denial:$descendant_denial,
      zero_writable_filesystem:$zero_writable_filesystem,cpu_limit:$cpu_limit,memory_limit:$memory_limit,
      process_count_limit:$process_count_limit,exact_cgroup_kill:$exact_cgroup_kill,
      cgroup_empty_after_job:$cgroup_empty_after_job,bounded_output:$bounded_output,timeout:$timeout,
      crash_relaunch:$crash_relaunch,cleanup:$cleanup,cross_job_isolation:$cross_job_isolation},
    limitations:$limitations,os_confined:$os_confined,production_admitted:false,source_retained:false,authority_added:false
  }' > "$receipt"

if [ "$result" != candidate_passed ]; then
  cat "$receipt" >&2
  exit 7
fi

printf 'Linux composite IAR-1B feasibility: result=%s kernel=%s arch=%s Landlock ABI=%s\n' \
  "$result" "$kernel_release" "$architecture" "$landlock_abi"
