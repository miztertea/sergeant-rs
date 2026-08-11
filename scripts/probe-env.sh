#!/usr/bin/env bash
# probe-env.sh — measure this host's environment facts and print them as a
# markdown table in the docs/environments/ format.
#
# Retro deliverable (docs/gauntlet/notes/session-retro-n-series-2026-08-11.md,
# item 1): nearly every hard stall in the N-series session was a collision
# with an unmeasured container fact discovered the expensive way, mid-task.
# Run this once at session start on ANY host and paste its table into that
# host's docs/environments/<hostname>.md before doing anything else.
#
# Contract:
#   - Every probe fails SOFT. A missing tool or a negative result prints
#     "unmeasurable: <why>" (or the negative fact itself) and the script
#     continues. This script never exits nonzero because a fact is absent —
#     only because of its OWN bug (enforced by `set -u`: an unbound
#     variable reference is a script bug, not an environment fact).
#   - No writes outside two mktemp-created directories (one under $TMPDIR,
#     one under $HOME), both removed unconditionally on exit via an EXIT
#     trap, including the immutable-bit and permission-bit fixups probes
#     need before their files can be deleted.
#   - Output is pure stdout: the markdown table, then one "paste
#     destination" line. No other stdout noise — findings that don't fit
#     a fact row go in the Evidence column, not extra prose.
set -u

# ---------------------------------------------------------------------------
# Setup: scratch dirs + unconditional cleanup
# ---------------------------------------------------------------------------

TMPDIR="${TMPDIR:-/tmp}"
PROBE_TMP_DIR=""
PROBE_HOME_DIR=""
IMMUTABLE_TEST_FILE=""

cleanup() {
  if [ -n "$IMMUTABLE_TEST_FILE" ] && [ -e "$IMMUTABLE_TEST_FILE" ]; then
    chattr -i "$IMMUTABLE_TEST_FILE" >/dev/null 2>&1 || true
  fi
  for d in "$PROBE_TMP_DIR" "$PROBE_HOME_DIR"; do
    if [ -n "$d" ] && [ -d "$d" ]; then
      chmod -R u+rwx "$d" >/dev/null 2>&1 || true
      rm -rf "$d" >/dev/null 2>&1 || true
    fi
  done
}
trap cleanup EXIT

PROBE_TMP_DIR="$(mktemp -d "${TMPDIR%/}/probe-env.XXXXXX" 2>/dev/null || true)"
PROBE_HOME_DIR="$(mktemp -d "${HOME%/}/.probe-env.XXXXXX" 2>/dev/null || true)"

# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------

have() { command -v "$1" >/dev/null 2>&1; }

# Run a command with a wall-clock bound if `timeout` exists; otherwise run it
# unbounded (only used for commands expected to return fast anyway).
bounded() {
  local secs="$1"
  shift
  if have timeout; then
    timeout "$secs" "$@"
  else
    "$@"
  fi
}

# One-line, pipe-escaped, markdown-table-safe rendering of a value.
mdcell() {
  printf '%s' "$1" | tr '\n' ' ' | sed 's/|/\\|/g'
}

FACT_NAME=()
FACT_VALUE=()
FACT_EVIDENCE=()

add_fact() {
  FACT_NAME+=("$1")
  FACT_VALUE+=("$2")
  FACT_EVIDENCE+=("$3")
}

# ---------------------------------------------------------------------------
# uid / user / groups
# ---------------------------------------------------------------------------

uid_num="$(id -u 2>/dev/null || echo unmeasurable)"
user_name="$(id -un 2>/dev/null || echo unmeasurable)"
groups_list="$(id -Gn 2>/dev/null | tr ' ' ',' || echo unmeasurable)"
add_fact "uid / user / groups" \
  "uid=${uid_num} (${user_name}); groups: ${groups_list}" \
  "id -u; id -un; id -Gn"

# ---------------------------------------------------------------------------
# DAC enforcement: chmod 000 self-read probe
# ---------------------------------------------------------------------------

if [ -n "$PROBE_TMP_DIR" ] && [ -d "$PROBE_TMP_DIR" ]; then
  dac_file="$PROBE_TMP_DIR/dac_probe"
  : > "$dac_file" 2>/dev/null
  if [ -f "$dac_file" ]; then
    chmod 000 "$dac_file" 2>/dev/null
    if cat "$dac_file" >/dev/null 2>&1; then
      dac_result="NOT enforced — chmod 000 self-read SUCCEEDED (root, or DAC override active)"
    else
      dac_result="enforced — chmod 000 self-read FAILED (permission denied)"
    fi
    chmod u+rw "$dac_file" 2>/dev/null || true
  else
    dac_result="unmeasurable: could not create probe file under $PROBE_TMP_DIR"
  fi
else
  dac_result="unmeasurable: no writable temp dir under \$TMPDIR"
fi
add_fact "DAC / permission-bit enforcement" "$dac_result" \
  "chmod 000 <file> then cat <file> as self, in a mktemp dir"

# ---------------------------------------------------------------------------
# CAP_LINUX_IMMUTABLE: chattr +i probe, cleaned up unconditionally
# ---------------------------------------------------------------------------

if ! have chattr; then
  imm_result="unmeasurable: chattr not present on this host"
elif [ -n "$PROBE_TMP_DIR" ] && [ -d "$PROBE_TMP_DIR" ]; then
  imm_file="$PROBE_TMP_DIR/immutable_probe"
  : > "$imm_file" 2>/dev/null
  if [ -f "$imm_file" ]; then
    imm_err="$(chattr +i "$imm_file" 2>&1)"
    imm_rc=$?
    if [ "$imm_rc" -eq 0 ]; then
      IMMUTABLE_TEST_FILE="$imm_file"
      imm_result="viable — chattr +i succeeded"
      chattr -i "$imm_file" >/dev/null 2>&1 || true
      IMMUTABLE_TEST_FILE=""
    else
      imm_result="not available — chattr +i failed: $(mdcell "$imm_err")"
    fi
  else
    imm_result="unmeasurable: could not create probe file under $PROBE_TMP_DIR"
  fi
else
  imm_result="unmeasurable: no writable temp dir under \$TMPDIR"
fi
add_fact "CAP_LINUX_IMMUTABLE" "$imm_result" \
  "chattr +i <file> in a mktemp dir, then chattr -i to clean up"

# ---------------------------------------------------------------------------
# Disk: free space on $HOME and $TMPDIR filesystems, quota tooling
# ---------------------------------------------------------------------------

disk_fact() {
  local dir="$1"
  if [ -d "$dir" ]; then
    df -Ph "$dir" 2>/dev/null | awk 'NR==2 {print $2" total, "$4" avail ("$5" used) on "$6}'
  else
    echo "unmeasurable: $dir does not exist"
  fi
}

home_disk="$(disk_fact "$HOME")"
[ -n "$home_disk" ] || home_disk="unmeasurable: df produced no output"
add_fact "Disk free (\$HOME fs)" "$home_disk" "df -Ph \"\$HOME\""

tmp_disk="$(disk_fact "$TMPDIR")"
[ -n "$tmp_disk" ] || tmp_disk="unmeasurable: df produced no output"
add_fact "Disk free (\$TMPDIR fs)" "$tmp_disk" "df -Ph \"\$TMPDIR\""

quota_tools=()
have quota && quota_tools+=("quota")
have repquota && quota_tools+=("repquota")
have xfs_quota && quota_tools+=("xfs_quota")
if [ "${#quota_tools[@]}" -eq 0 ]; then
  quota_result="absent (quota, repquota, xfs_quota not found)"
else
  quota_result="present: ${quota_tools[*]}"
fi
add_fact "Quota tooling" "$quota_result" "command -v quota / repquota / xfs_quota"

# ---------------------------------------------------------------------------
# O_DIRECT open behavior on $TMPDIR and $HOME's fs (python3 probe)
# ---------------------------------------------------------------------------

odirect_probe() {
  local dir="$1"
  if ! have python3; then
    echo "unmeasurable: python3 not present"
    return
  fi
  if [ -z "$dir" ] || [ ! -d "$dir" ]; then
    echo "unmeasurable: no writable mktemp dir available for this fs"
    return
  fi
  python3 - "$dir" <<'PYEOF' 2>&1
import os, sys
d = sys.argv[1]
path = os.path.join(d, "odirect_probe")
try:
    flag = os.O_DIRECT
except AttributeError:
    print("unmeasurable: os.O_DIRECT not exposed by this python3 build")
    sys.exit(0)
try:
    fd = os.open(path, os.O_RDWR | os.O_CREAT | flag, 0o600)
    os.close(fd)
    print("open SUCCEEDS")
except OSError as e:
    print("open FAILS: errno=%d (%s)" % (e.errno, e.strerror))
finally:
    try:
        os.remove(path)
    except OSError:
        pass
PYEOF
}

odirect_tmp="$(odirect_probe "$PROBE_TMP_DIR")"
add_fact "O_DIRECT open (\$TMPDIR fs)" "$(mdcell "$odirect_tmp")" \
  "python3: os.open(path, O_RDWR|O_CREAT|O_DIRECT) in a mktemp dir under \$TMPDIR"

odirect_home="$(odirect_probe "$PROBE_HOME_DIR")"
add_fact "O_DIRECT open (\$HOME fs)" "$(mdcell "$odirect_home")" \
  "python3: os.open(path, O_RDWR|O_CREAT|O_DIRECT) in a mktemp dir under \$HOME"

# ---------------------------------------------------------------------------
# Network posture: proxy env vars, HTTPS reachability (bounded, no error on fail)
# ---------------------------------------------------------------------------

proxy_names=(http_proxy https_proxy HTTP_PROXY HTTPS_PROXY no_proxy NO_PROXY ALL_PROXY all_proxy)
proxy_set=()
for n in "${proxy_names[@]}"; do
  v="${!n:-}"
  if [ -n "$v" ]; then
    proxy_set+=("$n")
  fi
done
if [ "${#proxy_set[@]}" -eq 0 ]; then
  proxy_result="none set"
else
  proxy_result="set: $(IFS=,; echo "${proxy_set[*]}") (values not printed — may carry embedded credentials)"
fi
add_fact "Proxy env vars" "$proxy_result" "checked ${#proxy_names[@]} conventional proxy var names"

https_probe() {
  local url="$1"
  if have curl; then
    local code rc
    code="$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "$url" 2>/dev/null)"
    rc=$?
    if [ "$rc" -ne 0 ] || [ -z "$code" ] || [ "$code" = "000" ]; then
      echo "unreachable (curl exit ${rc}, --max-time 5s)"
    else
      echo "HTTP ${code}"
    fi
  elif have wget; then
    if wget -q --spider --timeout=5 "$url" >/dev/null 2>&1; then
      echo "reachable (wget --spider, no status code captured)"
    else
      echo "unreachable (wget --spider failed, --timeout 5s)"
    fi
  else
    echo "unmeasurable: neither curl nor wget present"
  fi
}

gh_api="$(https_probe https://api.github.com)"
add_fact "HTTPS reachability: api.github.com" "$gh_api" \
  "curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://api.github.com"

gh_raw="$(https_probe https://raw.githubusercontent.com)"
add_fact "HTTPS reachability: raw.githubusercontent.com" "$gh_raw" \
  "curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://raw.githubusercontent.com"

# ---------------------------------------------------------------------------
# Docker: presence, daemon reachability, storage driver, cgroup version,
# user-in-docker-group. NEVER pulls images or runs containers.
# ---------------------------------------------------------------------------

if have docker; then
  docker_ver="$(bounded 3 docker --version 2>&1 | tr -d '\n')"
  docker_present="present ($docker_ver)"

  info_raw="$(bounded 5 docker info --format '{{json .}}' 2>&1)"
  info_rc=$?
  if [ "$info_rc" -eq 0 ]; then
    docker_daemon="reachable"
    if have python3; then
      storage_driver="$(printf '%s' "$info_raw" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get("Driver", "unknown"))
except Exception as e:
    print("unmeasurable: json parse failed (%s)" % e)
' 2>/dev/null)"
      cgroup_version="$(printf '%s' "$info_raw" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get("CgroupVersion", "unknown"))
except Exception as e:
    print("unmeasurable: json parse failed (%s)" % e)
' 2>/dev/null)"
    else
      storage_driver="unmeasurable: python3 absent, cannot parse docker info JSON"
      cgroup_version="unmeasurable: python3 absent, cannot parse docker info JSON"
    fi
  else
    docker_daemon="not reachable (docker info exited ${info_rc}: $(mdcell "$info_raw" | cut -c1-200))"
    storage_driver="unmeasurable: daemon unreachable"
    cgroup_version="unmeasurable: daemon unreachable"
  fi

  if id -nG 2>/dev/null | tr ' ' '\n' | grep -qx docker; then
    docker_group="user IS in the docker group"
  else
    docker_group="user NOT in the docker group (or group list unavailable)"
  fi
else
  docker_present="absent (docker not found on PATH)"
  docker_daemon="unmeasurable: docker absent"
  storage_driver="unmeasurable: docker absent"
  cgroup_version="unmeasurable: docker absent"
  docker_group="unmeasurable: docker absent"
fi

add_fact "Docker: presence / daemon / storage driver / cgroup / group" \
  "${docker_present}; daemon: ${docker_daemon}; storage driver: ${storage_driver}; cgroup: ${cgroup_version}; ${docker_group}" \
  "docker --version; docker info --format '{{json .}}' (no pull, no run); id -nG"
add_fact "Docker runtime lifecycle (pull/run/network/cleanup)" \
  "runtime lifecycle unprobed (use the docs/environments checklist)" \
  "deliberately not exercised at session start — session-start must stay cheap and offline-safe"

# ---------------------------------------------------------------------------
# claude CLI: presence, --version, auth status (loggedIn + authMethod only)
# ---------------------------------------------------------------------------

if have claude; then
  claude_version="$(bounded 5 claude --version 2>&1 | tr -d '\n')"
  [ -n "$claude_version" ] || claude_version="unmeasurable: claude --version produced no output"

  auth_raw="$(bounded 8 claude auth status --json 2>&1)"
  auth_rc=$?
  if [ "$auth_rc" -ne 0 ]; then
    auth_summary="unmeasurable: claude auth status --json exited ${auth_rc} (may not exist on this CLI version)"
  elif have python3; then
    auth_summary="$(printf '%s' "$auth_raw" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    print("loggedIn=%r authMethod=%r" % (d.get("loggedIn"), d.get("authMethod")))
except Exception as e:
    print("unmeasurable: could not parse json (%s)" % e)
' 2>/dev/null)"
    [ -n "$auth_summary" ] || auth_summary="unmeasurable: json parse produced no output"
  else
    auth_summary="unmeasurable: python3 absent, declining to hand-parse JSON that may carry tokens"
  fi
else
  claude_version="absent (claude not found on PATH)"
  auth_summary="unmeasurable: claude CLI absent"
fi
add_fact "claude CLI" "$claude_version" "command -v claude; claude --version"
add_fact "claude auth status" "$(mdcell "$auth_summary")" \
  "claude auth status --json, loggedIn+authMethod fields only — never tokens"

# ---------------------------------------------------------------------------
# cargo / rustc presence and versions, PATH caveat
# ---------------------------------------------------------------------------

path_tool_fact() {
  local tool="$1"
  if have "$tool"; then
    echo "$("$tool" --version 2>&1 | tr -d '\n') (on PATH)"
  elif [ -x "$HOME/.cargo/bin/$tool" ]; then
    echo "$("$HOME/.cargo/bin/$tool" --version 2>&1 | tr -d '\n') — found only at \$HOME/.cargo/bin, NOT on this shell's PATH; scripts must prefix PATH=\"\$HOME/.cargo/bin:\$PATH\""
  else
    echo "absent (not on PATH, not at \$HOME/.cargo/bin)"
  fi
}

add_fact "cargo" "$(mdcell "$(path_tool_fact cargo)")" "command -v cargo; cargo --version; \$HOME/.cargo/bin/cargo fallback"
add_fact "rustc" "$(mdcell "$(path_tool_fact rustc)")" "command -v rustc; rustc --version; \$HOME/.cargo/bin/rustc fallback"

# ---------------------------------------------------------------------------
# cores, kernel, container heuristic
# ---------------------------------------------------------------------------

if have nproc; then
  cores="$(nproc 2>/dev/null)"
else
  cores="unmeasurable: nproc not present"
fi
add_fact "Cores" "$cores" "nproc"

kernel="$(uname -r 2>/dev/null || echo unmeasurable)"
add_fact "Kernel" "$kernel" "uname -r"

if [ -e /.dockerenv ]; then
  dockerenv_evidence="/.dockerenv present"
else
  dockerenv_evidence="/.dockerenv absent"
fi
if [ -r /proc/1/cgroup ]; then
  if grep -Eq 'docker|kubepods|containerd|lxc' /proc/1/cgroup 2>/dev/null; then
    cgroup_evidence="/proc/1/cgroup contains container markers (docker/kubepods/containerd/lxc)"
  else
    cgroup_evidence="/proc/1/cgroup has no docker/kubepods/containerd/lxc markers"
  fi
else
  cgroup_evidence="/proc/1/cgroup unreadable"
fi
add_fact "Container heuristic (evidence, not a verdict)" \
  "${dockerenv_evidence}; ${cgroup_evidence}" \
  "test -e /.dockerenv; grep -E 'docker|kubepods|containerd|lxc' /proc/1/cgroup"

# ---------------------------------------------------------------------------
# IS_SANDBOX and root: claude skip-flag refusal (src/backend/claude.rs docs)
# ---------------------------------------------------------------------------

is_sandbox_val="${IS_SANDBOX:-<unset>}"
if [ "$uid_num" = "0" ]; then
  root_fact="uid 0 (root): claude --dangerously-skip-permissions is REFUSED under root/sudo (\"cannot be used with root/sudo privileges for security reasons\", exit 1) unless the spawning daemon's env sets IS_SANDBOX=1 — the adapter does not set this itself, per src/backend/claude.rs module docs. Current IS_SANDBOX=${is_sandbox_val}"
else
  root_fact="uid ${uid_num} (non-root): the root skip-flag refusal does not apply here. Current IS_SANDBOX=${is_sandbox_val}"
fi
add_fact "IS_SANDBOX / root skip-flag refusal" "$(mdcell "$root_fact")" \
  "id -u; \$IS_SANDBOX; src/backend/claude.rs module docs (root constraint paragraph)"

# ---------------------------------------------------------------------------
# Emit: markdown table, then the paste-destination hint. Pure stdout.
# ---------------------------------------------------------------------------

hostname_val="$(hostname 2>/dev/null || uname -n 2>/dev/null || echo unknown-host)"

printf '| Fact | Measured value | Evidence |\n'
printf '|---|---|---|\n'
i=0
while [ "$i" -lt "${#FACT_NAME[@]}" ]; do
  printf '| %s | %s | %s |\n' \
    "$(mdcell "${FACT_NAME[$i]}")" \
    "$(mdcell "${FACT_VALUE[$i]}")" \
    "$(mdcell "${FACT_EVIDENCE[$i]}")"
  i=$((i + 1))
done

printf '\nPaste destination: docs/environments/%s.md\n' "$hostname_val"
