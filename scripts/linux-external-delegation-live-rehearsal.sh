#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != "Linux" ]; then
  echo "Linux external delegation rehearsal is not applicable on this host"
  exit 0
fi
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "Linux external delegation rehearsal is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
output_root="$repository_root/target/iar-linux-external-live"
facts="$output_root/facts.json"
receipt="$output_root/receipt.json"

if [ "${1:-}" != "--provisioned" ]; then
  [ "$#" -eq 0 ] || { echo "usage: scripts/linux-external-delegation-live-rehearsal.sh" >&2; exit 2; }
  command -v systemd-run >/dev/null 2>&1 || { echo "systemd-run is required" >&2; exit 3; }
  command -v sudo >/dev/null 2>&1 || { echo "sudo is required only for the external CI provisioner" >&2; exit 3; }
  rm -rf -- "$output_root"
  mkdir -p -- "$output_root"
  unit="impresari-iar-external-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}-$$"
  sudo systemd-run --quiet --wait --pipe --collect --service-type=exec \
    --unit="$unit" --property="Delegate=cpu memory pids" \
    --setenv=GITHUB_ACTIONS=true --setenv=RUNNER_ENVIRONMENT=github-hosted \
    --uid="$(id -u)" --gid="$(id -g)" \
    --working-directory="$repository_root" \
    "$repository_root/scripts/linux-external-delegation-live-rehearsal.sh" --provisioned
  load_state=$(systemctl show "$unit" --property=LoadState --value)
  [ "$load_state" = not-found ] || { echo "external provisioner service was not collected" >&2; exit 6; }
  ruby "$repository_root/scripts/linux-external-delegation-live-finalize.rb" > "$receipt"
  cat "$receipt"
  exit 0
fi

[ "$#" -eq 1 ] || { echo "invalid provisioned invocation" >&2; exit 2; }
cgroup_suffix=$(awk -F: '$1 == "0" { print $3 }' /proc/self/cgroup)
case "$cgroup_suffix" in /*) ;; *) echo "invalid provisioned cgroup identity" >&2; exit 4 ;; esac
case "$cgroup_suffix" in *..*) echo "unsafe provisioned cgroup identity" >&2; exit 4 ;; esac
delegation_root="/sys/fs/cgroup$cgroup_suffix"
exec 3< "$delegation_root"
ruby "$repository_root/scripts/linux-external-delegation-live-receiver.rb" > "$facts"
exec 3<&-
exec "$repository_root/scripts/check-linux-composite-feasibility.sh" --delegated
