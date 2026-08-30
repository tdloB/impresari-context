#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

if [[ "$(uname -s)" != Linux || "${GITHUB_ACTIONS:-}" != true || "${RUNNER_ENVIRONMENT:-}" != github-hosted ]]; then
  echo "live login-session rehearsal is restricted to ephemeral GitHub-hosted Linux runners" >&2
  exit 3
fi
if [[ "$(id -u)" != 0 ]]; then
  echo "live host controller requires the workflow's bounded sudo step" >&2
  exit 3
fi
if [[ $# != 6 ]]; then
  echo "usage: linux-rootless-login-session-live.sh SOURCE_ROOT CANDIDATE_ARCHIVE PACKAGE_RECEIPT OUTPUT_DIR SOURCE_SHA RUNNER_UID" >&2
  exit 2
fi

source_root=$(realpath "$1")
candidate_archive=$(realpath "$2")
package_receipt=$(realpath "$3")
output_dir=$(realpath "$4")
source_sha=$5
runner_uid=$6
[[ "$source_sha" =~ ^[0-9a-f]{40}$ ]] || { echo "invalid source identity" >&2; exit 2; }
[[ "$runner_uid" =~ ^[1-9][0-9]{0,9}$ ]] || { echo "invalid runner identity" >&2; exit 2; }
[[ -d "$source_root" && ! -L "$source_root" && -f "$candidate_archive" && ! -L "$candidate_archive" ]] || exit 2
[[ -f "$package_receipt" && ! -L "$package_receipt" && -d "$output_dir" && ! -L "$output_dir" ]] || exit 2
runner_gid=$(id -g "$runner_uid")

for command in /usr/sbin/sshd /usr/sbin/useradd /usr/sbin/userdel /usr/bin/ssh-keygen /usr/bin/ssh /usr/bin/loginctl /usr/bin/systemctl /usr/bin/jq /usr/bin/ruby; do
  [[ -x "$command" ]] || { echo "required host command unavailable: $command" >&2; exit 3; }
done

username=impctxreentry
home_dir="/home/$username"
controller_root="$output_dir/controller"
sshd_pid=""
created_user=false

cleanup_host() {
  set +e
  if [[ -n "$sshd_pid" ]]; then
    kill "$sshd_pid" 2>/dev/null
    wait "$sshd_pid" 2>/dev/null
  fi
  /usr/bin/loginctl terminate-user "$username" 2>/dev/null
  if [[ "$created_user" == true ]]; then
    /usr/sbin/userdel -r "$username" 2>/dev/null
  fi
  rm -rf -- "$home_dir" "$controller_root"
}
trap cleanup_host EXIT

getent passwd "$username" >/dev/null && { echo "temporary user already exists" >&2; exit 4; }
mkdir -p -- "$controller_root"
chmod 0700 "$controller_root"
/usr/sbin/useradd --create-home --shell /bin/bash "$username"
created_user=true
user_uid=$(id -u "$username")
user_gid=$(id -g "$username")
if id -nG "$username" | tr ' ' '\n' | grep -Eq '^(sudo|wheel)$'; then
  echo "temporary user unexpectedly has administrator group membership" >&2
  exit 4
fi
/usr/bin/loginctl disable-linger "$username" >/dev/null 2>&1 || true
[[ ! -e "/var/lib/systemd/linger/$username" ]] || exit 4

bundle="$home_dir/bundle"
mkdir -p -- "$bundle" "$home_dir/.local/bin" "$home_dir/.ssh"
cp -a -- "$source_root/." "$bundle/"
rm -rf -- "$bundle/.git" "$bundle/target" "$bundle/dist"
cp -- "$package_receipt" "$home_dir/.local/package-receipt.json"

package_root="$controller_root/package"
mkdir -p -- "$package_root"
tar -xzf "$candidate_archive" -C "$package_root"
manifest=$(find "$package_root" -mindepth 2 -maxdepth 2 -name MANIFEST.json -type f -print -quit)
[[ -n "$manifest" ]] || { echo "candidate manifest unavailable" >&2; exit 4; }
candidate_dir=$(dirname "$manifest")
expected_archive=$(/usr/bin/jq -r '.candidate.archive_sha256' "$package_receipt")
expected_manifest=$(/usr/bin/jq -r '.candidate.manifest_sha256' "$package_receipt")
[[ "$expected_archive" =~ ^[0-9a-f]{64}$ && "$expected_manifest" =~ ^[0-9a-f]{64}$ ]] || exit 4
[[ "$(sha256sum "$candidate_archive" | awk '{print $1}')" == "$expected_archive" ]] || exit 4
[[ "$(sha256sum "$manifest" | awk '{print $1}')" == "$expected_manifest" ]] || exit 4
for name in impresari-context impresari-context-mcp impresari-context-structural-worker; do
  install -m 0755 "$candidate_dir/bin/$name" "$home_dir/.local/bin/$name"
done

cat > "$home_dir/session-one.sh" <<EOF
#!/bin/sh
set -eu
export GITHUB_ACTIONS=true RUNNER_ENVIRONMENT=github-hosted GITHUB_RUN_ID='${GITHUB_RUN_ID}' GITHUB_RUN_ATTEMPT='${GITHUB_RUN_ATTEMPT:-1}'
exec /usr/bin/ruby '$bundle/scripts/linux-rootless-login-session-observe.rb' --ordinal 1 --source-root '$bundle' --package-receipt '$home_dir/.local/package-receipt.json' --expected-source-sha '$source_sha'
EOF
cat > "$home_dir/session-two.sh" <<EOF
#!/bin/sh
set -eu
export GITHUB_ACTIONS=true RUNNER_ENVIRONMENT=github-hosted GITHUB_RUN_ID='${GITHUB_RUN_ID}' GITHUB_RUN_ATTEMPT='${GITHUB_RUN_ATTEMPT:-1}'
exec /usr/bin/ruby '$bundle/scripts/linux-rootless-login-session-observe.rb' --ordinal 2 --source-root '$bundle' --package-receipt '$home_dir/.local/package-receipt.json' --expected-source-sha '$source_sha'
EOF
chmod 0755 "$home_dir/session-one.sh" "$home_dir/session-two.sh"
chown -R "$user_uid:$user_gid" "$home_dir"
chown -R root:root "$bundle"
mkdir -p -- "$bundle/target"
chown "$user_uid:$user_gid" "$bundle/target"
chown root:root "$home_dir/.local/package-receipt.json" "$home_dir/session-one.sh" "$home_dir/session-two.sh"
chmod 0700 "$home_dir/.ssh"

/usr/bin/ssh-keygen -q -t ed25519 -N '' -f "$controller_root/session-one-key"
/usr/bin/ssh-keygen -q -t ed25519 -N '' -f "$controller_root/session-two-key"
{
  printf 'restrict,command="%s" %s\n' "$home_dir/session-one.sh" "$(cat "$controller_root/session-one-key.pub")"
  printf 'restrict,command="%s" %s\n' "$home_dir/session-two.sh" "$(cat "$controller_root/session-two-key.pub")"
} > "$home_dir/.ssh/authorized_keys"
chown "$user_uid:$user_gid" "$home_dir/.ssh/authorized_keys"
chmod 0600 "$home_dir/.ssh/authorized_keys"

/usr/bin/ssh-keygen -q -t ed25519 -N '' -f "$controller_root/host-key"
port=$((22000 + (GITHUB_RUN_ID % 1000)))
cat > "$controller_root/sshd_config" <<EOF
Port $port
ListenAddress 127.0.0.1
AddressFamily inet
HostKey $controller_root/host-key
PidFile $controller_root/sshd.pid
AuthorizedKeysFile .ssh/authorized_keys
AllowUsers $username
UsePAM yes
PasswordAuthentication no
KbdInteractiveAuthentication no
ChallengeResponseAuthentication no
PermitRootLogin no
PermitEmptyPasswords no
StrictModes yes
X11Forwarding no
AllowTcpForwarding no
PermitTunnel no
GatewayPorts no
PermitUserEnvironment no
UseDNS no
PrintMotd no
LogLevel ERROR
Subsystem sftp internal-sftp
EOF

system_ssh_before=$({ /usr/bin/systemctl show ssh.service --property=LoadState,ActiveState,SubState,FragmentPath 2>/dev/null || true; } | sha256sum | awk '{print $1}')
/usr/sbin/sshd -D -e -f "$controller_root/sshd_config" 2>"$controller_root/sshd.log" &
sshd_pid=$!
sleep 0.5
kill -0 "$sshd_pid" 2>/dev/null || { sed -n '1,20p' "$controller_root/sshd.log" >&2; exit 5; }
/usr/bin/ssh -q -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 -p "$port" -i "$controller_root/session-one-key" "$username@127.0.0.1" > "$output_dir/first-session.json"
[[ -s "$output_dir/first-session.json" ]] || exit 5

wait_for_manager_stop() {
  for _ in $(seq 1 120); do
    if [[ "$(/usr/bin/systemctl is-active "user@${user_uid}.service" 2>/dev/null || true)" != active ]]; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}
wait_for_manager_stop || { echo "first user manager did not terminate" >&2; exit 5; }
/usr/bin/ruby -rjson -e 'p=ARGV.fetch(0); d=JSON.parse(File.read(p)); d["user_manager_terminated"]=true; File.write(p, JSON.pretty_generate(d)+"\n")' "$output_dir/first-session.json"

/usr/bin/ssh -q -o BatchMode=yes -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 -p "$port" -i "$controller_root/session-two-key" "$username@127.0.0.1" > "$output_dir/second-session.json"
wait_for_manager_stop || { echo "second user manager did not terminate" >&2; exit 5; }
/usr/bin/ruby -rjson -e 'p=ARGV.fetch(0); d=JSON.parse(File.read(p)); d["user_manager_terminated"]=true; File.write(p, JSON.pretty_generate(d)+"\n")' "$output_dir/second-session.json"

stopped_sshd_pid=$sshd_pid
kill "$stopped_sshd_pid"
wait "$stopped_sshd_pid" 2>/dev/null || true
sshd_pid=""
/usr/bin/loginctl terminate-user "$username" 2>/dev/null || true
/usr/sbin/userdel -r "$username"
created_user=false
rm -rf -- "$home_dir" "$controller_root"
system_ssh_after=$({ /usr/bin/systemctl show ssh.service --property=LoadState,ActiveState,SubState,FragmentPath 2>/dev/null || true; } | sha256sum | awk '{print $1}')

temporary_user_absent=false; getent passwd "$username" >/dev/null || temporary_user_absent=true
home_absent=false; [[ ! -e "$home_dir" ]] && home_absent=true
isolated_ssh_process_absent=false; kill -0 "$stopped_sshd_pid" 2>/dev/null || isolated_ssh_process_absent=true
system_ssh_service_unchanged=false; [[ "$system_ssh_before" == "$system_ssh_after" ]] && system_ssh_service_unchanged=true
user_manager_absent=false; [[ "$(/usr/bin/systemctl is-active "user@${user_uid}.service" 2>/dev/null || true)" != active ]] && user_manager_absent=true
lingering_absent=false; [[ ! -e "/var/lib/systemd/linger/$username" ]] && lingering_absent=true
product_service_absent=false; [[ -z "$(/usr/bin/systemctl list-unit-files 'impresari*' --no-legend 2>/dev/null)" ]] && product_service_absent=true
authorization_policy_absent=false; [[ -z "$(find /etc/polkit-1/rules.d /usr/share/polkit-1/rules.d -maxdepth 1 -iname '*impresari*' -print 2>/dev/null)" ]] && authorization_policy_absent=true
delegated_cgroups_empty=false; [[ ! -e "/sys/fs/cgroup/user.slice/user-${user_uid}.slice" ]] && delegated_cgroups_empty=true
staged_source_absent=false; [[ ! -e "$bundle" ]] && staged_source_absent=true

/usr/bin/jq -n \
  --argjson temporary_user_absent "$temporary_user_absent" \
  --argjson home_absent "$home_absent" \
  --argjson isolated_ssh_process_absent "$isolated_ssh_process_absent" \
  --argjson system_ssh_service_unchanged "$system_ssh_service_unchanged" \
  --argjson user_manager_absent "$user_manager_absent" \
  --argjson lingering_absent "$lingering_absent" \
  --argjson product_service_absent "$product_service_absent" \
  --argjson authorization_policy_absent "$authorization_policy_absent" \
  --argjson delegated_cgroups_empty "$delegated_cgroups_empty" \
  --argjson staged_source_absent "$staged_source_absent" \
  '{temporary_user_absent:$temporary_user_absent,home_absent:$home_absent,isolated_ssh_process_absent:$isolated_ssh_process_absent,system_ssh_service_unchanged:$system_ssh_service_unchanged,lingering_absent:$lingering_absent,user_manager_absent:$user_manager_absent,product_service_absent:$product_service_absent,authorization_policy_absent:$authorization_policy_absent,delegated_cgroups_empty:$delegated_cgroups_empty,staged_source_absent:$staged_source_absent}' \
  > "$output_dir/cleanup.json"
chown "$runner_uid:$runner_gid" "$output_dir/first-session.json" "$output_dir/second-session.json" "$output_dir/cleanup.json"
trap - EXIT
