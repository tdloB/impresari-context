#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
set -eu

if [ "$(uname -s)" != Linux ] || [ "$(uname -m)" != x86_64 ]; then
  echo "YARA-X synthetic envelope is restricted to x86_64 Linux" >&2
  exit 3
fi
if [ "${GITHUB_ACTIONS:-}" != true ] || [ "${RUNNER_ENVIRONMENT:-}" != github-hosted ]; then
  echo "YARA-X synthetic envelope is restricted to ephemeral GitHub-hosted runners" >&2
  exit 3
fi

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
stage_root="$repository_root/target/yara-x-synthetic-envelope"
runtime_root="${RUNNER_TEMP:?RUNNER_TEMP is required}/impresari-yara-x-envelope-${GITHUB_RUN_ID:?}-${GITHUB_RUN_ATTEMPT:-1}"
build_root="$runtime_root/cargo-target"

cleanup() {
  for cleanup_root in "$stage_root" "$runtime_root"; do
    if [ -e "$cleanup_root" ] && [ ! -L "$cleanup_root" ]; then
      chmod -R u+rwX "$cleanup_root" || :
    fi
  done
  rm -rf -- "$stage_root" "$runtime_root"
  rm -f -- \
    "$repository_root/target/release/impresari-yara-x-synthetic-emitter" \
    "$repository_root/target/release/impresari-yara-x-synthetic-envelope"
}
trap cleanup EXIT HUP INT TERM

if [ "${1:-}" != --delegated ]; then
  command -v systemd-run >/dev/null 2>&1 || { echo "systemd-run is required" >&2; exit 3; }
  command -v sudo >/dev/null 2>&1 || { echo "sudo is required for the ephemeral delegated CI service" >&2; exit 3; }
  cleanup
  mkdir -p -- "$stage_root" "$runtime_root" "$stage_root/external" "$stage_root/credential"
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS='-C target-feature=+crt-static' \
    CARGO_TARGET_DIR="$build_root" cargo build --locked --release \
    --target x86_64-unknown-linux-gnu --package context-yara-x-envelope --bins
  cc -std=c17 -O2 -Wall -Wextra -Werror -pedantic \
    "$repository_root/platform/linux-yara-x-compatibility/launcher.c" \
    -o "$stage_root/launcher-template"
  cp "$build_root/x86_64-unknown-linux-gnu/release/impresari-yara-x-synthetic-emitter" \
    "$stage_root/emitter-template"
  cp "$build_root/x86_64-unknown-linux-gnu/release/impresari-yara-x-synthetic-envelope" \
    "$stage_root/coordinator"
  chmod 0555 "$stage_root/launcher-template" "$stage_root/emitter-template" "$stage_root/coordinator"
  printf '%s\n' synthetic-external-canary > "$stage_root/external/canary"
  printf '%s\n' synthetic-credential-canary > "$stage_root/credential/canary"
  unit="impresari-yara-envelope-${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT:-1}-$$"
  exec sudo systemd-run --quiet --wait --pipe --collect --service-type=exec \
    --unit="$unit" --property=Delegate=yes \
    --setenv=GITHUB_ACTIONS=true --setenv=RUNNER_ENVIRONMENT=github-hosted \
    --setenv=RUNNER_TEMP="$RUNNER_TEMP" --setenv=GITHUB_RUN_ID="$GITHUB_RUN_ID" \
    --setenv=GITHUB_RUN_ATTEMPT="${GITHUB_RUN_ATTEMPT:-1}" \
    --uid="$(id -u)" --gid="$(id -g)" --working-directory="$repository_root" \
    "$repository_root/scripts/yara-x-synthetic-envelope.sh" --delegated
fi

cgroup_suffix=$(awk -F: '$1 == "0" { print $3 }' /proc/self/cgroup)
case "$cgroup_suffix" in /*) ;; *) echo "invalid cgroup identity" >&2; exit 4 ;; esac
case "$cgroup_suffix" in *..*) echo "unsafe cgroup identity" >&2; exit 4 ;; esac
delegation_root="/sys/fs/cgroup$cgroup_suffix"
supervisor_cgroup="$delegation_root/impresari-supervisor"
mkdir "$supervisor_cgroup"
printf '%s\n' "$$" > "$supervisor_cgroup/cgroup.procs"
printf '%s\n' '+cpu +memory +pids' > "$delegation_root/cgroup.subtree_control"

for case_id in valid-match valid-no-match; do
  job_id="synthetic-$case_id"
  job_root="$stage_root/job-$job_id"
  cgroup_leaf="$delegation_root/job-$job_id"
  mkdir "$job_root" "$cgroup_leaf"
  cp "$stage_root/launcher-template" "$job_root/launcher"
  cp "$stage_root/emitter-template" "$job_root/emitter"
  chmod 0555 "$job_root/launcher" "$job_root/emitter"
  printf '%s %s\n' 100000 100000 > "$cgroup_leaf/cpu.max"
  printf '%s\n' 536870912 > "$cgroup_leaf/memory.max"
  printf '%s\n' 4 > "$cgroup_leaf/pids.max"
  if [ -f "$cgroup_leaf/memory.swap.max" ]; then printf '%s\n' 0 > "$cgroup_leaf/memory.swap.max"; fi
  if [ -f "$cgroup_leaf/memory.oom.group" ]; then printf '%s\n' 1 > "$cgroup_leaf/memory.oom.group"; fi

  emitter_digest="sha256:$(sha256sum "$job_root/emitter" | cut -d ' ' -f 1)"
  launcher_digest="sha256:$(sha256sum "$job_root/launcher" | cut -d ' ' -f 1)"
  case "$case_id" in
    valid-match)
      stdout_bytes=208
      stdout_digest=sha256:94e1dcb48bf00299178d75cfcf94a7551304a7a5d7b390e508d7a07079360499
      ;;
    valid-no-match)
      stdout_bytes=43
      stdout_digest=sha256:bc832c47989e5aaefbb5c43e9b04d5ac9ef790601b30257fff04bab1793ff6b0
      ;;
  esac
  control="$runtime_root/$case_id.control.json"
  output="$runtime_root/$case_id.output.json"
  errors="$runtime_root/$case_id.stderr"
  jq -n \
    --arg job_id "$job_id" --arg emitter_digest "$emitter_digest" \
    --arg launcher_digest "$launcher_digest" --arg case_id "$case_id" \
    --arg stdout_bytes "$stdout_bytes" --arg stdout_digest "$stdout_digest" \
    --arg launcher_path "$job_root/launcher" --arg emitter_path "$job_root/emitter" \
    --arg job_root "$job_root" --arg cgroup_leaf "$cgroup_leaf" \
    --arg external_canary "$stage_root/external/canary" \
    --arg credential_canary "$stage_root/credential/canary" \
    --arg write_probe "$job_root/forbidden-write" '
    {
      envelope:{
        schema_name:"yara-x-synthetic-runner-envelope-control",schema_version:"1.0.0",
        job_id:$job_id,profile_id:"yara-x-synthetic-runner-envelope-v1",
        profile_digest:"sha256:356f1ae13bec35ac41693936ddfe6856f8aad713d2a79b10b1de71557eb9a30b",
        emitter_digest:$emitter_digest,launcher_digest:$launcher_digest,case_id:$case_id,
        expected_stdout_bytes:$stdout_bytes,expected_stdout_digest:$stdout_digest,
        adapter:{workspace_snapshot:"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
          manifest_id:"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          artifact_hash:"sha256:1111111111111111111111111111111111111111111111111111111111111111",
          artifact_bytes:"64",expected_staged_path:"/staged/artifact.bin",
          executable_digest:"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
          ruleset_digest:"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
          completed_at:"2026-08-31T00:00:00Z"},authority_added:false
      },
      launcher_path:$launcher_path,emitter_path:$emitter_path,job_root:$job_root,
      cgroup_leaf:$cgroup_leaf,external_canary:$external_canary,
      credential_canary:$credential_canary,write_probe:$write_probe
    }' > "$control"

  "$stage_root/coordinator" < "$control" > "$output" 2> "$errors"
  [ ! -s "$errors" ] || { echo "synthetic coordinator emitted stderr" >&2; exit 5; }
  [ ! -e "$job_root" ] || { echo "synthetic job cleanup failed" >&2; exit 5; }
  [ ! -e "$cgroup_leaf" ] || { echo "synthetic cgroup cleanup failed" >&2; exit 5; }
  jq -e --arg case_id "$case_id" --arg stdout_digest "$stdout_digest" '
    .receipt.case_id == $case_id and .receipt.stdout_digest == $stdout_digest and
    .receipt.synthetic_emitter_executed and .receipt.synthetic_emitter_os_confined and
    .receipt.emitter_stderr_empty and .receipt.in_memory_composition_complete and
    .receipt.job_removed and .receipt.cgroup_removed and
    (.receipt.raw_output_retained | not) and (.receipt.yara_x_executed | not) and
    (.receipt.analyzer_executed | not) and (.receipt.production_admitted | not) and
    (.receipt.iar_2_admitted | not) and (.receipt.detection_quality_claimed | not) and
    (.receipt.safety_claimed | not) and (.receipt.authority_added | not) and
    (.normalized_result.analyzer_executed | not) and (.normalized_result.production_admitted | not)
  ' "$output" >/dev/null
  rm -f -- "$control" "$output" "$errors"
done

printf '%s\n' 'YARA-X synthetic runner envelope passed: cases=2 os_confined=true yara_x_executed=false production_admitted=false'
