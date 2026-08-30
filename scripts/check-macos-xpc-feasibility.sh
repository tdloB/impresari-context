#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "macOS XPC feasibility check is not applicable on this host"
  exit 0
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repository_root/target/iar-macos-xpc-feasibility"
app=$($repository_root/scripts/build-macos-xpc-feasibility.sh)
canaries="$output_root/canaries"
listener="$output_root/bin/synthetic-loopback-listener"
port_file="$output_root/loopback.port"
network_result="$output_root/loopback.result"
service="$app/Contents/XPCServices/studio.boldthaus.impresari-context.SandboxProbe.xpc"
service_executable="$service/Contents/MacOS/studio.boldthaus.impresari-context.SandboxProbe"
timeout_open_pid=
timeout_service_pid=

mkdir -p "$canaries"
for category in cache credential home repository; do
  printf '%s\n' 'synthetic-only' > "$canaries/$category"
done

run_probe() {
  probe_mode=$1
  probe_receipt=$2
  probe_errors=$3
  probe_port=$4
  : > "$probe_receipt"
  : > "$probe_errors"
  open -W -n -g -o "$probe_receipt" --stderr "$probe_errors" "$app" --args \
    "$probe_mode" \
    "$canaries/cache" \
    "$canaries/credential" \
    "$canaries/home" \
    "$canaries/repository" \
    "$$" \
    "$probe_port" || true
}

preparation_json() {
  preparation_errors=$1
  sed -n 's/^PREPARED //p' "$preparation_errors" | tail -1
}

validate_preparation() {
  preparation_errors=$1
  expected_mode=$2
  preparation=$(preparation_json "$preparation_errors")
  if ! printf '%s\n' "$preparation" | jq -e \
    --arg mode "$expected_mode" '
      .schema_name == "macos-xpc-sandbox-probe-preparation" and
      .schema_version == "1.0.0" and
      .probe_mode == $mode and
      (.service_process_id > 1) and
      .requested_limit_applied == true and
      .limit_error_number == 0 and
      .authority_added == false
    ' >/dev/null; then
    echo "invalid $expected_mode preparation receipt" >&2
    exit 10
  fi
  printf '%s\n' "$preparation" | jq -r .service_process_id
}

wait_for_process_exit() {
  process_id=$1
  attempt=0
  while kill -0 "$process_id" 2>/dev/null; do
    attempt=$((attempt + 1))
    if [ "$attempt" -ge 200 ]; then
      echo "synthetic service process $process_id did not exit" >&2
      exit 11
    fi
    sleep 0.02
  done
}

verify_service_identity() {
  process_id=$1
  observed=$(ps -p "$process_id" -o command= | sed 's/^[[:space:]]*//')
  if [ "$observed" != "$service_executable" ]; then
    echo "refusing synthetic supervisor action against unexpected process" >&2
    exit 12
  fi
}

cleanup() {
  if [ -n "$timeout_service_pid" ]; then
    kill -KILL "$timeout_service_pid" 2>/dev/null || true
  fi
  if [ -n "$timeout_open_pid" ]; then
    kill "$timeout_open_pid" 2>/dev/null || true
    wait "$timeout_open_pid" 2>/dev/null || true
  fi
  if [ -n "${listener_pid:-}" ]; then
    kill "$listener_pid" 2>/dev/null || true
    wait "$listener_pid" 2>/dev/null || true
  fi
}
trap cleanup EXIT HUP INT TERM

baseline_receipt="$output_root/receipt.json"
baseline_errors="$output_root/host.stderr"
rm -f -- "$port_file" "$network_result"
"$listener" "$port_file" "$network_result" &
listener_pid=$!
attempt=0
while [ ! -s "$port_file" ]; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 100 ]; then
    echo "synthetic loopback listener did not become ready" >&2
    exit 8
  fi
  sleep 0.02
done
loopback_port=$(cat "$port_file")
run_probe baseline "$baseline_receipt" "$baseline_errors" "$loopback_port"
baseline_service_pid=$(validate_preparation "$baseline_errors" baseline)

if ! wait "$listener_pid"; then
  echo "sandboxed service reached the synthetic loopback listener" >&2
  exit 9
fi
listener_pid=
if [ "$(cat "$network_result")" != "connection_denied" ]; then
  echo "network denial was not observed by the external listener" >&2
  exit 9
fi

if ! jq -e '
  .schema_name == "macos-xpc-sandbox-probe-receipt" and
  .schema_version == "1.0.0" and
  .probe_mode == "baseline" and
  (.service_process_id > 1) and
  .request_accepted == true and
  .app_container_read_write_verified == true and
  .canary_denials == {"cache":true,"credential":true,"home":true,"repository":true} and
  .device_access_denied == false and
  .unrelated_process_access_denied == true and
  .network_denial_verified == true and
  .resource_limits_verified == false and
  .descendant_containment_verified == false and
  .os_confined == false and
  .production_admitted == false and
  .source_retained == false and
  .authority_added == false
' "$baseline_receipt" >/dev/null; then
  echo "unexpected or incomplete macOS XPC feasibility receipt" >&2
  exit 4
fi
wait_for_process_exit "$baseline_service_pid"

cpu_receipt="$output_root/cpu-limit.json"
cpu_errors="$output_root/cpu-limit.stderr"
run_probe cpu_limit "$cpu_receipt" "$cpu_errors" 1
cpu_service_pid=$(validate_preparation "$cpu_errors" cpu_limit)
if [ -s "$cpu_receipt" ]; then
  echo "CPU exhaustion unexpectedly returned a receipt" >&2
  exit 13
fi
wait_for_process_exit "$cpu_service_pid"

memory_receipt="$output_root/memory-limit.json"
memory_errors="$output_root/memory-limit.stderr"
run_probe memory_limit "$memory_receipt" "$memory_errors" 1
memory_service_pid=$(validate_preparation "$memory_errors" memory_limit)
if [ "$memory_service_pid" = "$cpu_service_pid" ]; then
  echo "XPC service did not relaunch after CPU termination" >&2
  exit 14
fi
if ! jq -e '
  .schema_name == "macos-xpc-sandbox-resource-probe-receipt" and
  .schema_version == "1.0.0" and
  .probe_mode == "memory_limit" and
  (.service_process_id > 1) and
  .memory_allocation_denied == true and
  .fork_denied == false and
  .spawn_denied == false and
  .source_retained == false and
  .authority_added == false
' "$memory_receipt" >/dev/null; then
  echo "hard address-space growth denial was not verified" >&2
  exit 15
fi
wait_for_process_exit "$memory_service_pid"

descendant_receipt="$output_root/descendant-limit.json"
descendant_errors="$output_root/descendant-limit.stderr"
run_probe descendant_limit "$descendant_receipt" "$descendant_errors" 1
descendant_service_pid=$(validate_preparation "$descendant_errors" descendant_limit)
if ! jq -e '
  .schema_name == "macos-xpc-sandbox-resource-probe-receipt" and
  .schema_version == "1.0.0" and
  .probe_mode == "descendant_limit" and
  (.service_process_id > 1) and
  .memory_allocation_denied == false and
  .fork_denied == true and
  .spawn_denied == true and
  .source_retained == false and
  .authority_added == false
' "$descendant_receipt" >/dev/null; then
  echo "fork and posix_spawn denial was not verified" >&2
  exit 16
fi
wait_for_process_exit "$descendant_service_pid"

timeout_receipt="$output_root/supervisor-timeout.json"
timeout_errors="$output_root/supervisor-timeout.stderr"
: > "$timeout_receipt"
: > "$timeout_errors"
open -W -n -g -o "$timeout_receipt" --stderr "$timeout_errors" "$app" --args \
  supervisor_timeout \
  "$canaries/cache" \
  "$canaries/credential" \
  "$canaries/home" \
  "$canaries/repository" \
  "$$" \
  1 &
timeout_open_pid=$!
attempt=0
while ! grep -q '^PREPARED ' "$timeout_errors"; do
  attempt=$((attempt + 1))
  if [ "$attempt" -ge 100 ]; then
    echo "timeout probe did not publish its trusted preparation receipt" >&2
    exit 17
  fi
  sleep 0.02
done
timeout_service_pid=$(validate_preparation "$timeout_errors" supervisor_timeout)
verify_service_identity "$timeout_service_pid"
sleep 1
kill -KILL "$timeout_service_pid"
wait "$timeout_open_pid" || true
timeout_open_pid=
wait_for_process_exit "$timeout_service_pid"
timeout_service_pid=
if [ -s "$timeout_receipt" ]; then
  echo "hung service unexpectedly returned a receipt" >&2
  exit 18
fi

host_entitlements=$(codesign -d --entitlements - "$app" 2>&1)
service_entitlements=$(codesign -d --entitlements - "$service" 2>&1)
case "$host_entitlements" in
  *com.apple.security.app-sandbox*true*) ;;
  *) echo "host App Sandbox entitlement missing" >&2; exit 5 ;;
esac
case "$service_entitlements" in
  *com.apple.security.app-sandbox*true*) ;;
  *) echo "service App Sandbox entitlement missing" >&2; exit 6 ;;
esac
case "$service_entitlements" in
  *com.apple.security.network.client*|*com.apple.security.network.server*)
    echo "network entitlement is forbidden" >&2
    exit 7
    ;;
esac

trap - EXIT HUP INT TERM
echo "macOS App Sandbox XPC hybrid resource and lifecycle feasibility passed"
