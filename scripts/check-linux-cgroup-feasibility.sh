#!/bin/sh
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "Linux delegated-cgroup feasibility check is not applicable on this host"
  exit 0
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repository_root/target/iar-linux-cgroup-feasibility"
probe="$output_root/linux-cgroup-probe"
receipt="$output_root/receipt.json"
profile="$repository_root/profiles/v1/iar-linux-cgroup-synthetic-v1.json"

if [ "${1:-}" != "--delegated" ]; then
  command -v systemd-run >/dev/null 2>&1 || {
    echo "systemd-run is required for this CI-only checkpoint" >&2
    exit 3
  }
  command -v sudo >/dev/null 2>&1 || {
    echo "sudo is required only to create the transient delegated CI service" >&2
    exit 3
  }
  unit="impresari-iar-cgroup-${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-1}-$$"
  exec sudo systemd-run --quiet --wait --pipe --collect --service-type=exec \
    --unit="$unit" --property=Delegate=yes \
    --uid="$(id -u)" --gid="$(id -g)" \
    --working-directory="$repository_root" \
    "$repository_root/scripts/check-linux-cgroup-feasibility.sh" --delegated
fi

rm -rf -- "$output_root"
mkdir -p -- "$output_root"
cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
  "$repository_root/platform/linux-cgroup-feasibility/probe.c" -o "$probe"

cgroup_suffix=$(awk -F: '$1 == "0" { print $3 }' /proc/self/cgroup)
case "$cgroup_suffix" in /*) ;; *) echo "invalid cgroup identity" >&2; exit 4 ;; esac
case "$cgroup_suffix" in *..*) echo "unsafe cgroup identity" >&2; exit 4 ;; esac
delegation_root="/sys/fs/cgroup$cgroup_suffix"
supervisor_cgroup="$delegation_root/impresari-supervisor"
mkdir "$supervisor_cgroup"
printf '%s\n' "$$" > "$supervisor_cgroup/cgroup.procs"
printf '%s\n' '+cpu +memory +pids' > "$delegation_root/cgroup.subtree_control"

create_leaf() {
  leaf=$1
  mkdir "$leaf"
  printf '%s\n' '50000 100000' > "$leaf/cpu.max"
  printf '%s\n' '33554432' > "$leaf/memory.max"
  printf '%s\n' '1' > "$leaf/pids.max"
  if [ -f "$leaf/memory.swap.max" ]; then printf '%s\n' '0' > "$leaf/memory.swap.max"; fi
  if [ -f "$leaf/memory.oom.group" ]; then printf '%s\n' '1' > "$leaf/memory.oom.group"; fi
}
wait_empty() {
  leaf=$1
  attempts=0
  while [ "$attempts" -lt 200 ]; do
    if grep -qx 'populated 0' "$leaf/cgroup.events"; then return 0; fi
    attempts=$((attempts + 1))
    sleep 0.01
  done
  return 1
}
remove_leaf() { wait_empty "$1" && rmdir "$1"; }

cpu_limit=false memory_limit=false process_count_limit=false
exact_cgroup_kill=false cgroup_empty_after_job=false bounded_output=false
timeout=false crash_relaunch=false cleanup=false cross_job_isolation=false
clone_into_cgroup=false

leaf="$delegation_root/job-clone"
create_leaf "$leaf"
if [ "$($probe run "$leaf" ok)" = ok ]; then clone_into_cgroup=true; fi
remove_leaf "$leaf"

leaf="$delegation_root/job-pids"
create_leaf "$leaf"
if [ "$($probe run "$leaf" pids)" = denied ]; then process_count_limit=true; fi
remove_leaf "$leaf"

leaf="$delegation_root/job-memory"
create_leaf "$leaf"
oom_before=$(awk '$1 == "oom_kill" { print $2 }' "$leaf/memory.events.local")
if $probe run "$leaf" memory; then
  oom_after=$(awk '$1 == "oom_kill" { print $2 }' "$leaf/memory.events.local")
  if [ "$oom_after" -gt "$oom_before" ]; then memory_limit=true; fi
fi
remove_leaf "$leaf"

leaf="$delegation_root/job-cpu"
create_leaf "$leaf"
if $probe run "$leaf" cpu; then
  throttled=$(awk '$1 == "nr_throttled" { print $2 }' "$leaf/cpu.stat")
  if [ "$throttled" -gt 0 ]; then cpu_limit=true; fi
fi
remove_leaf "$leaf"

leaf="$delegation_root/job-output"
create_leaf "$leaf"
output_limit=$(jq -r '.limits.output_bytes' "$profile")
set +e
$probe run "$leaf" flood | head -c "$output_limit" | wc -c > "$output_root/output-count" &
pipeline_pid=$!
set -e
wait "$pipeline_pid" || true
if [ "$(tr -d ' ' < "$output_root/output-count")" = "$output_limit" ]; then bounded_output=true; fi
wait_empty "$leaf"
remove_leaf "$leaf"

leaf="$delegation_root/job-timeout"
create_leaf "$leaf"
$probe run "$leaf" sleep & timeout_parent=$!
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
if $probe run "$crash_leaf" crash && remove_leaf "$crash_leaf"; then
  create_leaf "$relaunch_leaf"
  if [ "$($probe run "$relaunch_leaf" ok)" = ok ]; then crash_relaunch=true; fi
  remove_leaf "$relaunch_leaf"
fi

first_leaf="$delegation_root/job-a"
second_leaf="$delegation_root/job-b"
create_leaf "$first_leaf"; create_leaf "$second_leaf"
$probe run "$first_leaf" sleep & first_parent=$!
$probe run "$second_leaf" sleep & second_parent=$!
sleep 0.1
printf '%s\n' '1' > "$first_leaf/cgroup.kill"
if wait "$first_parent" && kill -0 "$second_parent" 2>/dev/null && \
   grep -qx 'populated 1' "$second_leaf/cgroup.events"; then
  cross_job_isolation=true
fi
printf '%s\n' '1' > "$second_leaf/cgroup.kill"
wait "$second_parent"
remove_leaf "$first_leaf"; remove_leaf "$second_leaf"

remaining=$(find "$delegation_root" -mindepth 1 -maxdepth 1 -type d ! -name impresari-supervisor | wc -l | tr -d ' ')
if [ "$remaining" = 0 ]; then cleanup=true; fi

controllers=$(cat "$delegation_root/cgroup.subtree_control")
case " $controllers " in *" cpu "*) cpu_controller=true ;; *) cpu_controller=false ;; esac
case " $controllers " in *" memory "*) memory_controller=true ;; *) memory_controller=false ;; esac
case " $controllers " in *" pids "*) pids_controller=true ;; *) pids_controller=false ;; esac
cgroup_v2=false; [ -f "$delegation_root/cgroup.controllers" ] && cgroup_v2=true
delegated_subtree=false; [ -w "$delegation_root/cgroup.subtree_control" ] && delegated_subtree=true
cgroup_kill=false; [ -f "$delegation_root/cgroup.kill" ] && cgroup_kill=true
cgroup_empty_verification=false; [ -r "$delegation_root/cgroup.events" ] && cgroup_empty_verification=true

result=failed
resource_lifecycle_confined=false
limitations='["resource-lifecycle-check-failed","single-host-evidence","composite-linux-admission-pending","synthetic-probe-no-analysis"]'
if [ "$clone_into_cgroup" = true ] && [ "$cpu_limit" = true ] && \
   [ "$memory_limit" = true ] && [ "$process_count_limit" = true ] && \
   [ "$exact_cgroup_kill" = true ] && [ "$cgroup_empty_after_job" = true ] && \
   [ "$bounded_output" = true ] && [ "$timeout" = true ] && \
   [ "$crash_relaunch" = true ] && [ "$cleanup" = true ] && \
   [ "$cross_job_isolation" = true ]; then
  result=candidate_passed
  resource_lifecycle_confined=true
  limitations='["single-host-evidence","composite-linux-admission-pending","synthetic-probe-no-analysis"]'
fi

kernel_release=$(uname -r); architecture=$(uname -m)
profile_digest="sha256:$(sha256sum "$profile" | awk '{print $1}')"
jq -n --arg digest "$profile_digest" --arg kernel "$kernel_release" \
  --arg architecture "$architecture" --arg result "$result" \
  --argjson cgroup_v2 "$cgroup_v2" --argjson cpu_controller "$cpu_controller" \
  --argjson memory_controller "$memory_controller" --argjson pids_controller "$pids_controller" \
  --argjson delegated_subtree "$delegated_subtree" --argjson clone_into_cgroup "$clone_into_cgroup" \
  --argjson cgroup_kill "$cgroup_kill" --argjson empty_verification "$cgroup_empty_verification" \
  --argjson cpu_limit "$cpu_limit" --argjson memory_limit "$memory_limit" \
  --argjson process_count_limit "$process_count_limit" --argjson exact_kill "$exact_cgroup_kill" \
  --argjson empty_after "$cgroup_empty_after_job" --argjson bounded_output "$bounded_output" \
  --argjson timeout "$timeout" --argjson crash_relaunch "$crash_relaunch" \
  --argjson cleanup "$cleanup" --argjson cross_job "$cross_job_isolation" \
  --argjson limitations "$limitations" --argjson confined "$resource_lifecycle_confined" '
  {schema_name:"linux-cgroup-feasibility",schema_version:"1.0.0",prototype_id:"iar-linux-cgroup-feasibility-v1",profile_id:"iar-linux-cgroup-synthetic-v1",profile_digest:$digest,
   observed_host:{operating_system:"linux",kernel_release:$kernel,architecture:$architecture,delegation_provider:"systemd-transient-service"},result:$result,
   preflight:{cgroup_v2:$cgroup_v2,cpu_controller:$cpu_controller,memory_controller:$memory_controller,pids_controller:$pids_controller,delegated_subtree:$delegated_subtree,clone_into_cgroup:$clone_into_cgroup,cgroup_kill:$cgroup_kill,cgroup_empty_verification:$empty_verification},
   checks:{cpu_limit:$cpu_limit,memory_limit:$memory_limit,process_count_limit:$process_count_limit,exact_cgroup_kill:$exact_kill,cgroup_empty_after_job:$empty_after,bounded_output:$bounded_output,timeout:$timeout,crash_relaunch:$crash_relaunch,cleanup:$cleanup,cross_job_isolation:$cross_job},
   limitations:$limitations,resource_lifecycle_confined:$confined,os_confined:false,production_admitted:false,source_retained:false,authority_added:false}' > "$receipt"

if [ "$result" != candidate_passed ]; then cat "$receipt" >&2; exit 7; fi
printf 'Linux delegated-cgroup feasibility: result=%s kernel=%s arch=%s\n' "$result" "$kernel_release" "$architecture"
