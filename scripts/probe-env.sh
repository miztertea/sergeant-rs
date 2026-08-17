#!/usr/bin/env bash
# probe-env.sh — measure this host's environment facts and print them as a
# markdown table in the docs/environments/ format.
#
# Retro deliverable (sergeant-rs-workspace/knowledge/evidence/gauntlet/notes/session-retro-n-series-2026-08-11.md,
# item 1): nearly every hard stall in the N-series session was a collision
# with an unmeasured container fact discovered the expensive way, mid-task.
# Run this once at session start on ANY host and paste its table into that
# host's docs/environments/<hostname>.md before doing anything else. See
# docs/DEVELOPMENT.md's "Environments" section for the workflow this script serves.
#
# Contract:
#   - Every probe fails SOFT. A missing tool or a negative result prints
#     "unmeasurable: <why>" (or the negative fact itself) and the script
#     continues. This script never exits nonzero because a fact is absent —
#     only because of its OWN bug (enforced by `set -u`: an unbound
#     variable reference is a script bug, not an environment fact).
#   - Every fact row is a MEASUREMENT taken this run, not an imported claim
#     from docs or a prior session. Anything not measured this run (e.g. a
#     behavior documented in source but not exercised here) is prose in the
#     Notes section below the table, never a row in the facts table.
#   - No writes outside two mktemp-created directories (one under $TMPDIR,
#     one under $HOME), both removed unconditionally on exit via an EXIT
#     trap, including the immutable-bit and permission-bit fixups probes
#     need before their files can be deleted.
#   - Output is pure stdout: a dated header, the markdown table, a Notes
#     section for non-measured context, then one "paste destination" line.
set -u

# ---------------------------------------------------------------------------
# Setup: scratch dirs + unconditional cleanup
# ---------------------------------------------------------------------------

TMPDIR="${TMPDIR:-/tmp}"
HOME="${HOME:-/tmp}"
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

# Run a command with a wall-clock bound if `timeout` exists; otherwise print
# an unmeasurable result rather than risk hanging. Callers that touch a
# daemon or the network MUST go through this — never invoke unbounded.
bounded() {
  local secs="$1"
  shift
  if have timeout; then
    timeout "$secs" "$@"
    return $?
  fi
  echo "unmeasurable: no timeout(1) available to bound this probe"
  return 111
}

# Collapse embedded newlines to spaces. Safe to apply repeatedly (idempotent)
# — used when composing a value from another command's possibly-multi-line
# output, BEFORE that value is stored via add_fact. Pure bash (no tr/sed
# dependency): this must survive a PATH so bare it has neither, since the
# whole point of this script is to still report something on such a host.
oneline() {
  local v="$1"
  printf '%s' "${v//$'\n'/ }"
}

# Pipe-and-backslash-escaped, markdown-table-safe rendering of a value.
# Apply exactly once per cell — at emit time only. Values passed to
# add_fact should be plain (pre-collapsed with `oneline` if needed, but NOT
# pre-escaped) since the emit loop is the single place this runs. Pure bash
# parameter expansion — no tr/sed dependency, see `oneline` above.
mdcell() {
  local v="$1"
  v="${v//$'\n'/ }"
  v="${v//\\/\\\\}"
  v="${v//|/\\|}"
  printf '%s' "$v"
}

# Fill an empty/unset value with an explicit unmeasurable marker instead of
# letting a blank cell reach the table.
nonblank() {
  if [ -n "${1:-}" ]; then
    printf '%s' "$1"
  else
    printf 'unmeasurable: probe produced no output'
  fi
}

PROBE_DATE="$(date -u +%Y-%m-%d 2>/dev/null || echo unmeasurable-date)"

FACT_NAME=()
FACT_VALUE=()
FACT_EVIDENCE=()
NOTES=()

add_fact() {
  FACT_NAME+=("$1")
  FACT_VALUE+=("$(nonblank "$2")")
  FACT_EVIDENCE+=("$(nonblank "$3"), ${PROBE_DATE}")
}

add_note() {
  NOTES+=("$1")
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
    chmod_rc=$?
    mode_now=""
    if have stat; then
      mode_now="$(stat -c %a "$dac_file" 2>/dev/null || stat -f %Lp "$dac_file" 2>/dev/null || echo "")"
    fi
    if [ "$chmod_rc" -ne 0 ]; then
      dac_result="unmeasurable: chmod 000 exited ${chmod_rc} — could not change the mode on this filesystem"
    elif [ -n "$mode_now" ] && [ "$mode_now" != "0" ] && [ "$mode_now" != "000" ]; then
      dac_result="unmeasurable: chmod 000 reported success but mode is still ${mode_now} on this filesystem (exfat/vfat/9p/virtiofs-style FS?)"
    else
      if cat "$dac_file" >/dev/null 2>&1; then
        dac_result="NOT enforced — chmod 000 self-read SUCCEEDED (see uid row: root ignores mode bits, or a DAC-override capability is active)"
      else
        dac_result="enforced — chmod 000 self-read FAILED (permission denied)"
      fi
    fi
    chmod u+rw "$dac_file" 2>/dev/null || true
  else
    dac_result="unmeasurable: could not create probe file under $PROBE_TMP_DIR"
  fi
else
  dac_result="unmeasurable: no writable temp dir under \$TMPDIR"
fi
add_fact "DAC / permission-bit enforcement" "$dac_result" \
  "chmod 000 <file>, verify mode via stat, then cat <file> as self, in a mktemp dir"

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
      if chattr -i "$imm_file" >/dev/null 2>&1; then
        IMMUTABLE_TEST_FILE=""
      fi
      # else: leave IMMUTABLE_TEST_FILE set so the EXIT trap retries cleanup.
    else
      imm_result="not available — chattr +i failed: $(oneline "$imm_err")"
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
# Disk: free space on $HOME and $TMPDIR filesystems, quota tooling,
# writable allowance (best-effort — quota may be enforced above the FS
# without any local quota tool visibility).
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
add_fact "Disk free (\$HOME fs)" "$home_disk" "df -Ph \"\$HOME\""

tmp_disk="$(disk_fact "$TMPDIR")"
add_fact "Disk free (\$TMPDIR fs)" "$tmp_disk" "df -Ph \"\$TMPDIR\""

quota_tools=()
have quota && quota_tools+=("quota")
have repquota && quota_tools+=("repquota")
have xfs_quota && quota_tools+=("xfs_quota")
if [ "${#quota_tools[@]}" -eq 0 ]; then
  quota_bin_result="absent (quota, repquota, xfs_quota not found)"
else
  quota_bin_result="present: ${quota_tools[*]}"
fi
add_fact "Quota binaries on PATH" "$quota_bin_result" \
  "command -v quota / repquota / xfs_quota (binary presence only — does not imply the binary applies to this host's filesystems)"

# Writable allowance: this is the fact that actually bit sessions (ENOSPC
# stalls), and df total/avail does not measure it — a below-filesystem quota
# can be enforced with no visible quota tool. Attempt `quota -s`; otherwise
# say plainly that no per-user limit is visible via local tooling.
if have quota; then
  quota_out="$(bounded 5 quota -s 2>&1)"
  quota_rc=$?
  if [ "$quota_rc" -eq 0 ] && [ -n "$quota_out" ]; then
    allowance_result="quota -s: $(oneline "$quota_out" | cut -c1-300)"
  else
    allowance_result="unmeasurable: quota -s exited ${quota_rc} with no usable output — no per-user limit visible via quota; allowance may still be enforced elsewhere (e.g. session sandbox, above the filesystem)"
  fi
else
  allowance_result="unmeasurable: no quota tooling on PATH — no per-user write allowance visible locally; the \$HOME/\$TMPDIR df rows above are filesystem capacity, NOT a session allowance, which may be enforced below filesystem size with no local signal"
fi
add_fact "Writable allowance (session quota, distinct from FS capacity)" "$allowance_result" \
  "quota -s, or explicit unmeasurable when no quota tool is present"

# ---------------------------------------------------------------------------
# O_DIRECT behavior on $TMPDIR and $HOME's fs (python3 probe): open, AND an
# unaligned write, since the repo's fault-injection fixtures (journal.rs)
# branch on both — "open refused" and "unaligned write silently accepted"
# both skip the fixture; only "unaligned write rejected" makes it expressible.
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
fd = None
try:
    fd = os.open(path, os.O_RDWR | os.O_CREAT | flag, 0o600)
except OSError as e:
    print("open FAILS: errno=%d (%s) -> fault-injection fixture SKIPS here (can't even open)" % (e.errno, e.strerror))
    sys.exit(0)
try:
    # Unaligned write: odd length, unaligned buffer — the shape the repo's
    # fixtures rely on rejecting.
    buf = b"x" * 513
    try:
        os.write(fd, buf)
        print("open SUCCEEDS, unaligned write ACCEPTED -> fault-injection fixture SKIPS here (alignment unenforced)")
    except OSError as e:
        print("open SUCCEEDS, unaligned write REJECTED errno=%d (%s) -> fault-injection fixture IS expressible here" % (e.errno, e.strerror))
finally:
    os.close(fd)
    try:
        os.remove(path)
    except OSError:
        pass
PYEOF
}

odirect_tmp="$(odirect_probe "$PROBE_TMP_DIR")"
add_fact "O_DIRECT open+unaligned-write (\$TMPDIR fs)" "$(oneline "$odirect_tmp")" \
  "python3: open(O_RDWR|O_CREAT|O_DIRECT) then write a 513-byte unaligned buffer, in a mktemp dir under \$TMPDIR"

odirect_home="$(odirect_probe "$PROBE_HOME_DIR")"
add_fact "O_DIRECT open+unaligned-write (\$HOME fs)" "$(oneline "$odirect_home")" \
  "python3: open(O_RDWR|O_CREAT|O_DIRECT) then write a 513-byte unaligned buffer, in a mktemp dir under \$HOME"

# ---------------------------------------------------------------------------
# Network posture: proxy env vars, HTTPS reachability (bounded, no error on
# fail). Includes a real asset-path probe, not just a bare host, since the
# fact that has actually cost sessions time is an asset-path 403 through an
# agent proxy that still 200s/301s the bare host.
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
  proxy_result="set: $(IFS=,; echo "${proxy_set[*]:-}") (values not printed — may carry embedded credentials)"
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
add_fact "HTTPS reachability: raw.githubusercontent.com (bare host)" "$gh_raw" \
  "curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://raw.githubusercontent.com"

# The real asset path docs/DEVELOPMENT.md names as the working fallback when the
# release installer 403s — reproduces the fact that actually costs sessions
# time, not just bare-host reachability.
nomistakes_asset="$(https_probe https://raw.githubusercontent.com/kunchenguid/no-mistakes/main/docs/install.sh)"
add_fact "HTTPS reachability: no-mistakes install.sh (real asset path)" "$nomistakes_asset" \
  "curl -s -o /dev/null -w '%{http_code}' --max-time 5 https://raw.githubusercontent.com/kunchenguid/no-mistakes/main/docs/install.sh"

# ---------------------------------------------------------------------------
# Docker: presence, daemon reachability, storage driver, cgroup version,
# user-in-docker-group. NEVER pulls images or runs containers.
# ---------------------------------------------------------------------------

if have docker; then
  docker_ver="$(bounded 3 docker --version 2>&1 | tr -d '\n')"
  docker_present="present ($docker_ver)"

  # stdout and stderr captured separately: dockerd routinely writes health
  # warnings to stderr on otherwise-healthy hosts, which must not poison the
  # JSON parse of stdout.
  info_out="$(bounded 5 docker info --format '{{json .}}' 2>/dev/null)"
  info_rc=$?
  if [ "$info_rc" -eq 0 ] && [ -n "$info_out" ]; then
    docker_daemon="reachable"
    if have python3; then
      storage_driver="$(printf '%s' "$info_out" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    print(d.get("Driver", "unknown"))
except Exception as e:
    print("unmeasurable: json parse failed (%s)" % e)
' 2>/dev/null)"
      cgroup_version="$(printf '%s' "$info_out" | python3 -c '
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
    info_err="$(bounded 5 docker info --format '{{json .}}' 2>&1 >/dev/null)"
    docker_daemon="not reachable (docker info exited ${info_rc}: $(oneline "$info_err" | cut -c1-200))"
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
  "docker --version; docker info --format '{{json .}}' (stdout/stderr split, no pull, no run); id -nG"
add_fact "Docker runtime lifecycle (pull/run/network/cleanup)" \
  "runtime lifecycle unprobed (use the docs/environments checklist)" \
  "deliberately not exercised at session start — session-start must stay cheap and offline-safe"

# ---------------------------------------------------------------------------
# claude CLI: presence, --version, auth status (loggedIn + authMethod only)
# ---------------------------------------------------------------------------

if have claude; then
  claude_version="$(bounded 5 claude --version 2>&1 | tr -d '\n')"

  auth_raw="$(bounded 8 claude auth status --json 2>&1)"
  auth_rc=$?
  if [ "$auth_rc" -ne 0 ]; then
    auth_summary="unmeasurable: claude auth status --json exited ${auth_rc} (may not exist on this CLI version, or timed out)"
  elif have python3; then
    auth_summary="$(printf '%s' "$auth_raw" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    print("loggedIn=%r authMethod=%r" % (d.get("loggedIn"), d.get("authMethod")))
except Exception as e:
    print("unmeasurable: could not parse json (%s)" % e)
' 2>/dev/null)"
  else
    auth_summary="unmeasurable: python3 absent, declining to hand-parse JSON that may carry tokens"
  fi
else
  claude_version="absent (claude not found on PATH)"
  auth_summary="unmeasurable: claude CLI absent"
fi
add_fact "claude CLI" "$claude_version" "command -v claude; claude --version"
add_fact "claude auth status" "$(oneline "$auth_summary")" \
  "claude auth status --json, loggedIn+authMethod fields only — never tokens"

# ---------------------------------------------------------------------------
# cargo / rustc presence and versions, PATH caveat
# ---------------------------------------------------------------------------

path_tool_fact() {
  local tool="$1"
  if have "$tool"; then
    echo "$("$tool" --version 2>&1 | tr -d '\n') (on PATH)"
  elif [ -x "${HOME}/.cargo/bin/$tool" ]; then
    echo "$("${HOME}/.cargo/bin/$tool" --version 2>&1 | tr -d '\n') — found only at \$HOME/.cargo/bin, NOT on this shell's PATH; scripts must prefix PATH=\"\$HOME/.cargo/bin:\$PATH\""
  else
    echo "absent (not on PATH, not at \$HOME/.cargo/bin)"
  fi
}

add_fact "cargo" "$(oneline "$(path_tool_fact cargo)")" "command -v cargo; cargo --version; \$HOME/.cargo/bin/cargo fallback"
add_fact "rustc" "$(oneline "$(path_tool_fact rustc)")" "command -v rustc; rustc --version; \$HOME/.cargo/bin/rustc fallback"

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
  is_docker_container=1
else
  dockerenv_evidence="/.dockerenv absent"
  is_docker_container=0
fi
if [ -r /proc/1/cgroup ]; then
  if grep -Eq 'docker|kubepods|containerd|lxc' /proc/1/cgroup 2>/dev/null; then
    cgroup_evidence="/proc/1/cgroup contains container markers (docker/kubepods/containerd/lxc)"
    is_docker_container=1
  else
    cgroup_evidence="/proc/1/cgroup has no docker/kubepods/containerd/lxc markers"
  fi
else
  cgroup_evidence="/proc/1/cgroup unreadable"
fi
add_fact "Container heuristic (evidence, not a verdict)" \
  "${dockerenv_evidence}; ${cgroup_evidence}" \
  "test -e /.dockerenv; grep -E 'docker|kubepods|containerd|lxc' /proc/1/cgroup"

is_ci=0
if [ -n "${CI:-}" ] || [ -n "${GITHUB_ACTIONS:-}" ]; then
  is_ci=1
fi

# ---------------------------------------------------------------------------
# IS_SANDBOX and uid — measured facts only. The claude.rs-documented root
# skip-flag refusal is NOT re-derived here (it requires an actual claude
# invocation to confirm on this host); it is reported as an unverified
# imported note below the table, gated on uid==0 && claude present, never
# printed as a "Measured value".
# ---------------------------------------------------------------------------

is_sandbox_val="${IS_SANDBOX:-<unset>}"
add_fact "IS_SANDBOX env var" "IS_SANDBOX=${is_sandbox_val}" "\$IS_SANDBOX"

if [ "$uid_num" = "0" ]; then
  if have claude; then
    add_note "uid 0 (root) with claude CLI present: src/backend/claude.rs module docs say \`claude --dangerously-skip-permissions\` is REFUSED under root/sudo (\"cannot be used with root/sudo privileges for security reasons\", exit 1) unless the spawning daemon's env sets IS_SANDBOX=1. NOT verified by this script this run (no claude invocation is made) — imported claim, re-check against src/backend/claude.rs and, if it matters for this session, confirm with a real turn."
  else
    add_note "uid 0 (root), claude CLI absent from PATH: cannot check the documented root skip-flag refusal on this host at all (imported claim from src/backend/claude.rs, unverifiable here without the CLI)."
  fi
fi

# ---------------------------------------------------------------------------
# Emit: dated header, markdown table, Notes section, then the paste-
# destination hint. Pure stdout (Notes carries context that doesn't fit a
# fact row, per the header contract — never silent extra noise).
# ---------------------------------------------------------------------------

hostname_val="$(hostname 2>/dev/null || uname -n 2>/dev/null || echo unknown-host)"

if [ "$is_ci" -eq 1 ]; then
  paste_target="github-runner"
else
  paste_target="$hostname_val"
  if [ "$is_docker_container" -eq 1 ]; then
    add_note "Container evidence present, but container evidence alone cannot name the host (one file per environment): the paste-destination hint below falls back to this host's hostname. If this session is the known Claude Code cloud container, the destination is docs/environments/claude-code-cloud.md; any other containerized host gets its own file on first contact."
  fi
fi

printf 'Facts measured %s on host "%s"\n\n' "$PROBE_DATE" "$hostname_val"
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

if [ "${#NOTES[@]}" -gt 0 ]; then
  printf '\nNotes (imported context, NOT measured this run — verify before relying on):\n'
  for n in "${NOTES[@]}"; do
    printf -- '- %s\n' "$n"
  done
fi

printf '\nPaste destination: docs/environments/%s.md\n' "$paste_target"
