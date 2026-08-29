#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "macOS XPC feasibility check is not applicable on this host"
  exit 0
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
app=$($repository_root/scripts/build-macos-xpc-feasibility.sh)
receipt="$repository_root/target/iar-macos-xpc-feasibility/receipt.json"
errors="$repository_root/target/iar-macos-xpc-feasibility/host.stderr"
canaries="$repository_root/target/iar-macos-xpc-feasibility/canaries"
listener="$repository_root/target/iar-macos-xpc-feasibility/bin/synthetic-loopback-listener"
port_file="$repository_root/target/iar-macos-xpc-feasibility/loopback.port"
network_result="$repository_root/target/iar-macos-xpc-feasibility/loopback.result"
mkdir -p "$canaries"
for category in cache credential home repository; do
  printf '%s\n' 'synthetic-only' > "$canaries/$category"
done
: > "$receipt"
: > "$errors"
rm -f -- "$port_file" "$network_result"
"$listener" "$port_file" "$network_result" &
listener_pid=$!
cleanup_listener() {
  kill "$listener_pid" 2>/dev/null || true
  wait "$listener_pid" 2>/dev/null || true
}
trap cleanup_listener EXIT HUP INT TERM
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
open -W -n -g -o "$receipt" --stderr "$errors" "$app" --args \
  "$canaries/cache" \
  "$canaries/credential" \
  "$canaries/home" \
  "$canaries/repository" \
  "$$" \
  "$loopback_port"

if ! wait "$listener_pid"; then
  echo "sandboxed service reached the synthetic loopback listener" >&2
  exit 9
fi
trap - EXIT HUP INT TERM
if [ "$(cat "$network_result")" != "connection_denied" ]; then
  echo "network denial was not observed by the external listener" >&2
  exit 9
fi

if ! jq -e '
  .schema_name == "macos-xpc-sandbox-probe-receipt" and
  .schema_version == "1.0.0" and
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
' "$receipt" >/dev/null; then
  echo "unexpected or incomplete macOS XPC feasibility receipt" >&2
  exit 4
fi

host_entitlements=$(codesign -d --entitlements - "$app" 2>&1)
service="$app/Contents/XPCServices/studio.boldthaus.impresari-context.SandboxProbe.xpc"
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

echo "macOS App Sandbox XPC synthetic transport feasibility passed"
