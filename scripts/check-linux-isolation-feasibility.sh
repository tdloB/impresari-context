#!/bin/sh
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "Linux IAR-1B feasibility check is not applicable on this host"
  exit 0
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repository_root/target/iar-linux-isolation-feasibility"
probe="$output_root/linux-isolation-probe"
job_root="$output_root/job"
external_root="$output_root/external"
credential_root="$output_root/credential"
receipt="$output_root/receipt.json"
profile="$repository_root/tests/conformance/v1/valid/iar-linux-synthetic-profile.json"

rm -rf -- "$output_root"
mkdir -p -- "$job_root" "$external_root" "$credential_root"
printf '%s\n' 'synthetic-job-input' > "$job_root/input"
printf '%s\n' 'synthetic-external-canary' > "$external_root/canary"
printf '%s\n' 'synthetic-credential-canary' > "$credential_root/canary"

cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
  "$repository_root/platform/linux-isolation-feasibility/probe.c" \
  -o "$probe"

capabilities=$($probe capabilities)

capability_value() {
  capability_name=$1
  printf '%s\n' "$capabilities" | sed -n "s/^${capability_name}=//p" | tail -1
}

checked_boolean() {
  checked_value=$1
  checked_name=$2
  case "$checked_value" in
    true|false) printf '%s\n' "$checked_value" ;;
    *)
      echo "invalid boolean from Linux probe: $checked_name" >&2
      exit 3
      ;;
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
cgroup_v2=$(checked_boolean "$(capability_value cgroup_v2)" cgroup_v2)
cpu_controller=$(checked_boolean "$(capability_value cpu_controller)" cpu_controller)
memory_controller=$(checked_boolean "$(capability_value memory_controller)" memory_controller)
pids_controller=$(checked_boolean "$(capability_value pids_controller)" pids_controller)
delegated_leaf=$(checked_boolean "$(capability_value delegated_leaf)" delegated_leaf)
cgroup_kill=$(checked_boolean "$(capability_value cgroup_kill)" cgroup_kill)
cgroup_empty_verification=$(checked_boolean "$(capability_value cgroup_empty_verification)" cgroup_empty_verification)

case "$architecture" in
  x86_64|aarch64) ;;
  *)
    echo "unsupported Linux architecture identity: $architecture" >&2
    exit 3
    ;;
esac
case "$kernel_release" in
  ''|*[!A-Za-z0-9._+-]*)
    echo "unsafe Linux kernel release identity" >&2
    exit 3
    ;;
esac
case "$landlock_abi" in
  0|[1-9]|[1-9][0-9]|[1-9][0-9][0-9]) ;;
  *)
    echo "invalid Landlock ABI identity" >&2
    exit 3
    ;;
esac

no_new_privs_effective=false
landlock_read_only_input=false
external_filesystem_denial=false
credential_denial=false
device_denial=false
network_denial=false
unrelated_descriptors_closed=false
descendant_denial=false
zero_writable_filesystem=false

primitive_ready=false
if [ "$no_new_privs" = true ] && [ "$landlock" = true ] && \
   [ "$seccomp_filter" = true ] && [ "$seccomp_kill_process" = true ] && \
   [ "$architecture_filter" = true ]; then
  primitive_ready=true
fi

if [ "$primitive_ready" = true ]; then
  primitive=$($probe primitive \
    "$job_root" \
    "$job_root/input" \
    "$external_root/canary" \
    "$credential_root/canary" \
    "$job_root/forbidden-write")
  primitive_value() {
    primitive_name=$1
    printf '%s\n' "$primitive" | sed -n "s/^${primitive_name}=//p" | tail -1
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
  if [ "$no_new_privs_effective" != true ] || \
     [ "$landlock_read_only_input" != true ] || \
     [ "$external_filesystem_denial" != true ] || \
     [ "$credential_denial" != true ] || \
     [ "$device_denial" != true ] || \
     [ "$network_denial" != true ] || \
     [ "$unrelated_descriptors_closed" != true ] || \
     [ "$descendant_denial" != true ] || \
     [ "$zero_writable_filesystem" != true ]; then
    echo "Linux primitive isolation evidence was not reproduced" >&2
    exit 4
  fi
fi

result=partial
limitations='["cgroup-resource-suite-pending","lifecycle-suite-pending","single-host-evidence","synthetic-probe-no-analysis"]'
if [ "$no_new_privs" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["no-new-privs-unavailable"]')
fi
if [ "$landlock" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["landlock-unavailable"]')
fi
if [ "$seccomp_filter" != true ] || [ "$seccomp_kill_process" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["seccomp-unavailable"]')
fi
if [ "$architecture_filter" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["architecture-filter-unavailable"]')
fi
if [ "$cgroup_v2" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["cgroup-v2-unavailable"]')
fi
if [ "$cpu_controller" != true ] || [ "$memory_controller" != true ] || \
   [ "$pids_controller" != true ] || [ "$cgroup_kill" != true ] || \
   [ "$cgroup_empty_verification" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["required-cgroup-controller-unavailable"]')
fi
if [ "$delegated_leaf" != true ]; then
  result=unsupported
  limitations=$(printf '%s\n' "$limitations" | jq '. + ["cgroup-delegation-unavailable"]')
fi

profile_digest="sha256:$(sha256sum "$profile" | awk '{print $1}')"

jq -n \
  --arg profile_digest "$profile_digest" \
  --arg kernel_release "$kernel_release" \
  --arg architecture "$architecture" \
  --arg landlock_abi "$landlock_abi" \
  --arg result "$result" \
  --argjson no_new_privs "$no_new_privs" \
  --argjson landlock "$landlock" \
  --argjson seccomp_filter "$seccomp_filter" \
  --argjson seccomp_kill_process "$seccomp_kill_process" \
  --argjson architecture_filter "$architecture_filter" \
  --argjson cgroup_v2 "$cgroup_v2" \
  --argjson cpu_controller "$cpu_controller" \
  --argjson memory_controller "$memory_controller" \
  --argjson pids_controller "$pids_controller" \
  --argjson delegated_leaf "$delegated_leaf" \
  --argjson cgroup_kill "$cgroup_kill" \
  --argjson cgroup_empty_verification "$cgroup_empty_verification" \
  --argjson no_new_privs_effective "$no_new_privs_effective" \
  --argjson landlock_read_only_input "$landlock_read_only_input" \
  --argjson external_filesystem_denial "$external_filesystem_denial" \
  --argjson credential_denial "$credential_denial" \
  --argjson device_denial "$device_denial" \
  --argjson network_denial "$network_denial" \
  --argjson unrelated_descriptors_closed "$unrelated_descriptors_closed" \
  --argjson descendant_denial "$descendant_denial" \
  --argjson zero_writable_filesystem "$zero_writable_filesystem" \
  --argjson limitations "$limitations" '
  {
    schema_name: "linux-isolation-feasibility",
    schema_version: "1.0.0",
    prototype_id: "iar-linux-isolation-feasibility-v1",
    profile_id: "iar-linux-synthetic-v1",
    profile_digest: $profile_digest,
    observed_host: {
      operating_system: "linux",
      kernel_release: $kernel_release,
      architecture: $architecture,
      landlock_abi: $landlock_abi
    },
    result: $result,
    preflight: {
      no_new_privs: $no_new_privs,
      landlock: $landlock,
      seccomp_filter: $seccomp_filter,
      seccomp_kill_process: $seccomp_kill_process,
      architecture_filter: $architecture_filter,
      cgroup_v2: $cgroup_v2,
      cpu_controller: $cpu_controller,
      memory_controller: $memory_controller,
      pids_controller: $pids_controller,
      delegated_leaf: $delegated_leaf,
      cgroup_kill: $cgroup_kill,
      cgroup_empty_verification: $cgroup_empty_verification
    },
    checks: {
      atomic_cgroup_placement: false,
      resource_profile_applied: false,
      no_new_privs_effective: $no_new_privs_effective,
      landlock_read_only_input: $landlock_read_only_input,
      external_filesystem_denial: $external_filesystem_denial,
      credential_denial: $credential_denial,
      device_denial: $device_denial,
      network_denial: $network_denial,
      unrelated_descriptors_closed: $unrelated_descriptors_closed,
      descendant_denial: $descendant_denial,
      zero_writable_filesystem: $zero_writable_filesystem,
      cpu_limit: false,
      memory_limit: false,
      process_count_limit: false,
      exact_cgroup_kill: false,
      cgroup_empty_after_job: false,
      bounded_output: false,
      timeout: false,
      crash_relaunch: false,
      cleanup: false,
      cross_job_isolation: false
    },
    limitations: $limitations,
    os_confined: false,
    production_admitted: false,
    source_retained: false,
    authority_added: false
  }' > "$receipt"

if ! jq -e \
  --arg profile_digest "$profile_digest" '
    .schema_name == "linux-isolation-feasibility" and
    .schema_version == "1.0.0" and
    .prototype_id == "iar-linux-isolation-feasibility-v1" and
    .profile_id == "iar-linux-synthetic-v1" and
    .profile_digest == $profile_digest and
    (.result == "partial" or .result == "unsupported") and
    .os_confined == false and
    .production_admitted == false and
    .source_retained == false and
    .authority_added == false and
    (.limitations | index("cgroup-resource-suite-pending") != null) and
    (.limitations | index("lifecycle-suite-pending") != null) and
    (.limitations | index("single-host-evidence") != null) and
    (.limitations | index("synthetic-probe-no-analysis") != null)
  ' "$receipt" >/dev/null; then
  echo "invalid Linux IAR-1B feasibility receipt" >&2
  exit 5
fi

printf 'Linux IAR-1B feasibility: result=%s kernel=%s arch=%s Landlock ABI=%s delegated_cgroup=%s\n' \
  "$result" "$kernel_release" "$architecture" "$landlock_abi" "$delegated_leaf"
