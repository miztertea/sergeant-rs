#!/usr/bin/env bash
# scripts/perf/common.sh — shared helpers for the P1-PERF load/stress harness.
#
# Sourced by every s*-*.sh scenario and by run-all.sh. Nothing here measures
# anything on its own; it supplies the four things every scenario needs:
#
#   * a scenario workspace   — fresh data dir + a seeded git repo (demo.sh's
#                              recipe: .sergeant/workflows/<name>/<stage>/)
#   * a daemon under control — spawned with a chosen SGT_FAKE_SCRIPT, its pid
#                              read from the runtime descriptor (the documented
#                              client handshake), stopped and verified stopped
#   * /proc sampling         — VmRSS / VmHWM / threads / fd count / utime+stime
#                              into CSV, either one row on demand or a
#                              background sampler at a chosen interval
#   * measurement plumbing   — latency timing, percentiles, a summary block
#                              that is printed and written as JSON, and the
#                              hygiene sweep
#
# The harness drives the shipped release binary and its loopback HTTP API and
# adds no Rust (P1-PERF non-goal: "new Rust code beyond what a scenario
# strictly needs"). High-rate cells drive POST /v1/... with curl rather than
# one CLI process per call, because the CLI spawns a process per call and that
# is the client's cost, not the daemon's.
#
# Nothing under here writes into the repository tree: every scenario's data
# dirs, repos and raw output live under the <outdir> it is given.

# --- resolution ------------------------------------------------------------

PERF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PERF_REPO_ROOT="$(cd "$PERF_DIR/../.." && pwd)"
# The unit under test is the release build (P1-PERF: "the baseline describes
# what ships, not debug-profile DuckDB").
SGT_BIN="${SGT_BIN:-$PERF_REPO_ROOT/target/release/sgt}"

PERF_CLK_TCK="$(getconf CLK_TCK 2>/dev/null || echo 100)"

# Timing: CLOCK_REALTIME nanoseconds. `date +%s%N` costs a fork (~2 ms on this
# container) which is the same order as the thing being measured, so the
# fork-free bash-5 EPOCHREALTIME is preferred and `date` is the fallback. Same
# clock, same units.
perf_now_ns() {
  local t=${EPOCHREALTIME:-}
  if [ -n "$t" ]; then
    t=${t/,/.}
    local sec=${t%.*} frac=${t#*.}
    while [ ${#frac} -lt 9 ]; do frac="${frac}0"; done
    printf '%s%s\n' "$sec" "${frac:0:9}"
  else
    date +%s%N
  fi
}

# perf_mark VAR — assign "now" in ns to VAR without forking a subshell.
perf_mark() {
  local -n __perf_mark_out=$1
  local t=${EPOCHREALTIME:-}
  if [ -n "$t" ]; then
    t=${t/,/.}
    local sec=${t%.*} frac=${t#*.}
    while [ ${#frac} -lt 9 ]; do frac="${frac}0"; done
    __perf_mark_out="${sec}${frac:0:9}"
  else
    __perf_mark_out="$(date +%s%N)"
  fi
}

perf_die() { printf '\033[31mperf: %s\033[0m\n' "$*" >&2; exit 1; }
perf_say() { printf '   %s\n' "$*"; }
perf_step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }
perf_warn() { printf '\033[33m   ! %s\033[0m\n' "$*" >&2; }

perf_require_binary() {
  [ -x "$SGT_BIN" ] || perf_die "no release binary at $SGT_BIN (cargo build --release)"
  for tool in curl jq python3 git; do
    command -v "$tool" >/dev/null || perf_die "missing required tool: $tool"
  done
  # The tested binary's identity, captured exactly once — a live rev-parse per
  # scenario tracks the checkout, not the binary: a commit landing mid-matrix
  # made 6 of 8 scenarios self-report code that was never in the binary
  # (issue #50). Exported so scenarios spawned by run-all.sh inherit the pin
  # instead of re-reading HEAD. A failed rev-parse warns and is never cached
  # or exported, so a later call (or a child scenario) retries instead of
  # inheriting a permanent "unknown".
  if [ -z "${PERF_COMMIT:-}" ] || [ "$PERF_COMMIT" = unknown ]; then
    if PERF_COMMIT="$(git -C "$PERF_REPO_ROOT" rev-parse HEAD 2>/dev/null)"; then
      export PERF_COMMIT
    else
      perf_warn "git rev-parse failed for $PERF_REPO_ROOT — commit recorded as unknown"
      PERF_COMMIT=unknown
    fi
  else
    # An inherited pin is the designed path (run-all.sh exports it for its
    # scenarios) but the value is trusted, so at least say when it cannot be
    # a sha of this repo.
    case "$PERF_COMMIT" in
      *[!0-9a-f]*) perf_warn "inherited PERF_COMMIT '$PERF_COMMIT' does not look like a commit sha" ;;
    esac
    export PERF_COMMIT
  fi
  # The tree's cleanliness at pin time, pinned beside the sha (and inherited
  # the same way): a sha alone claims code the working tree may never have
  # matched, and that fact belongs in the artifacts, not in a rerun's
  # archaeology.
  if [ -z "${PERF_TREE_DIRTY:-}" ]; then
    if [ "$PERF_COMMIT" = unknown ]; then
      PERF_TREE_DIRTY=unknown
    elif [ -n "$(git -C "$PERF_REPO_ROOT" status --porcelain 2>/dev/null)" ]; then
      PERF_TREE_DIRTY=1
      perf_warn "working tree at $PERF_REPO_ROOT is dirty — the commit field names a sha the tree did not match"
    else
      PERF_TREE_DIRTY=0
    fi
    export PERF_TREE_DIRTY
  fi
  # The binary's own timestamp, pinned at the same moment: mtime forensics are
  # what reconstructed the truth for issue #50, so record it in the artifacts
  # and warn up front when the binary predates HEAD — then the commit field
  # cannot describe the code in the binary and the run needs a rebuild first.
  if [ -z "${PERF_BIN_MTIME:-}" ] || [ "$PERF_BIN_MTIME" = 0 ]; then
    PERF_BIN_MTIME="$(stat -c %Y "$SGT_BIN" 2>/dev/null || echo 0)"
    if [ "$PERF_BIN_MTIME" != 0 ]; then
      export PERF_BIN_MTIME
      local head_ct
      head_ct="$(git -C "$PERF_REPO_ROOT" log -1 --format=%ct HEAD 2>/dev/null || echo 0)"
      PERF_BIN_STALE=0
      if [ "$head_ct" -gt 0 ] && [ "$PERF_BIN_MTIME" -lt "$head_ct" ]; then
        PERF_BIN_STALE=1
        perf_warn "binary $SGT_BIN predates HEAD ($PERF_COMMIT) — rebuild, or the commit field describes code that is not in the binary"
      fi
      export PERF_BIN_STALE
    fi
  fi
}

# --- scenario lifecycle ----------------------------------------------------

# perf_init <outdir> <scenario-name>
#
# Creates the output dir, the scenario's scratch dir (data dirs and seed repos
# live there, never in the repo tree), opens the summary accumulator, and
# installs the teardown trap so an aborted scenario still stops its daemon.
perf_init() {
  PERF_OUT="$1"
  PERF_SCENARIO="$2"
  perf_require_binary
  mkdir -p "$PERF_OUT"
  PERF_OUT="$(cd "$PERF_OUT" && pwd)"
  PERF_SCRATCH="$PERF_OUT/scratch"
  mkdir -p "$PERF_SCRATCH"
  PERF_SUMMARY_TSV="$PERF_OUT/${PERF_SCENARIO}-summary.tsv"
  : > "$PERF_SUMMARY_TSV"
  PERF_DAEMON_PID=""
  PERF_SAMPLER_PID=""
  PERF_FORCE_KILLED=0
  PERF_REPOS=()
  PERF_START_NS="$(perf_now_ns)"
  trap 'perf_teardown' EXIT
  perf_step "$PERF_SCENARIO — output $PERF_OUT"
  perf_kv scenario "$PERF_SCENARIO"
  perf_kv binary "$SGT_BIN"
  perf_kv binary_mtime "${PERF_BIN_MTIME:-0}"
  perf_kv commit "$PERF_COMMIT"
  # The provenance caveats land in the summary artifact itself, not only on
  # stderr (#50 was about the artifact): a binary older than HEAD at pin
  # time, a dirty tree behind the pinned sha, and — re-statted here, once
  # per scenario — a rebuild that swapped the code under the pinned label
  # mid-matrix.
  perf_kv binary_predates_head "${PERF_BIN_STALE:-0}"
  perf_kv commit_dirty_tree "${PERF_TREE_DIRTY:-unknown}"
  local perf_mtime_now
  perf_mtime_now="$(stat -c %Y "$SGT_BIN" 2>/dev/null || echo 0)"
  if [ "$perf_mtime_now" != "${PERF_BIN_MTIME:-0}" ]; then
    perf_warn "binary $SGT_BIN was rebuilt since the run pinned it (mtime ${PERF_BIN_MTIME:-0} -> $perf_mtime_now) — this scenario's commit label may not describe its code"
    perf_kv binary_swapped_since_pin 1
    perf_kv binary_mtime_now "$perf_mtime_now"
  else
    perf_kv binary_swapped_since_pin 0
  fi
  perf_kv started_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}

perf_teardown() {
  local rc=$?
  perf_sampler_stop >/dev/null 2>&1 || true
  if [ -n "${PERF_DAEMON_PID:-}" ] && kill -0 "$PERF_DAEMON_PID" 2>/dev/null; then
    perf_daemon_stop "$PERF_DAEMON_PID" >/dev/null 2>&1 || true
  fi
  return $rc
}

# --- the summary block -----------------------------------------------------

# perf_kv <key> <value> — one distilled measurement.
perf_kv() { printf '%s\t%s\n' "$1" "$2" >> "$PERF_SUMMARY_TSV"; }

# perf_summary — print the distilled block and write <scenario>-summary.json.
perf_summary() {
  python3 - "$PERF_SUMMARY_TSV" "$PERF_OUT/${PERF_SCENARIO}-summary.json" "$PERF_SCENARIO" <<'PY'
import json, sys
tsv, out, name = sys.argv[1], sys.argv[2], sys.argv[3]
rows = []
for line in open(tsv):
    line = line.rstrip("\n")
    if not line:
        continue
    key, _, value = line.partition("\t")
    rows.append((key, value))
doc = {}
for key, value in rows:
    try:
        doc[key] = int(value)
    except ValueError:
        try:
            doc[key] = float(value)
        except ValueError:
            doc[key] = value
json.dump(doc, open(out, "w"), indent=2, sort_keys=False)
width = max((len(k) for k, _ in rows), default=0)
print()
print(f"\033[1m-- {name} summary " + "-" * max(0, 58 - len(name)) + "\033[0m")
for key, value in rows:
    print(f"   {key.ljust(width)}  {value}")
print(f"   (json: {out})")
PY
}

# --- seed repo (demo.sh's recipe) ------------------------------------------

# perf_seed_repo <repo-dir> [stage-count]
#
# A throwaway git repository carrying a `.sergeant/workflows/software-change`
# workflow — the same layout scripts/demo.sh builds, which is what makes the
# submit path materialize a surface and run stages instead of parking the work
# in `pending`.
perf_seed_repo() {
  local repo="$1" stages="${2:-2}"
  mkdir -p "$repo"
  git init -q -b main "$repo"
  git -C "$repo" config user.email perf@example.invalid
  git -C "$repo" config user.name "sgt perf harness"
  git -C "$repo" config commit.gpgsign false
  printf 'def settle(payment):\n    return gateway.settle(payment)\n' > "$repo/payments.py"
  local wf="$repo/.sergeant/workflows/software-change"
  mkdir -p "$wf"
  local names=(10-implement 20-review 30-verify 40-land 50-followup)
  local ids=() i id
  for ((i = 0; i < stages; i++)); do
    if [ "$i" -lt "${#names[@]}" ]; then id="${names[$i]}"; else id="$(printf '%02d-stage%d' $(((i + 1) * 10)) $((i + 1)))"; fi
    ids+=("$id")
    mkdir -p "$wf/$id"
    printf 'Stage %s of the perf workflow.\n' "$id" > "$wf/$id/CONTEXT.md"
  done
  {
    printf '[workflow]\nname = "software-change"\nversion = "1"\nstages = ['
    local sep=""
    for id in "${ids[@]}"; do printf '%s"%s"' "$sep" "$id"; sep=", "; done
    printf ']\n'
  } > "$wf/workflow.toml"
  git -C "$repo" add -A
  git -C "$repo" commit -qm "perf harness seed"
  PERF_REPOS+=("$repo")
  PERF_REPO="$repo"
}

# --- daemon control --------------------------------------------------------

perf_descriptor_field() {
  local dd="$1" field="$2"
  [ -f "$dd/runtime.json" ] || return 1
  jq -r --arg f "$field" '.[$f] // empty' "$dd/runtime.json" 2>/dev/null
}

# perf_daemon_start <data-dir> [fake-script] [ready-timeout-s]
#
# Spawns `sgt daemon` detached in its own session with the given
# SGT_FAKE_SCRIPT (empty = the fake's default: every stage completes), then
# waits for the documented client handshake — the runtime descriptor plus a
# healthy /healthz — and exports:
#
#   PERF_DAEMON_PID  PERF_ENDPOINT  PERF_TOKEN  PERF_DATA_DIR
#   PERF_DAEMON_START_NS  PERF_DAEMON_READY_NS  PERF_DAEMON_READY_MS
#
# PERF_DAEMON_READY_MS is the cold-start figure S5 reports: spawn to first
# answered request, which is journal replay + registry fold + DuckDB rebuild +
# listener bind.
perf_daemon_start() {
  local dd="$1" script="${2:-}" timeout="${3:-300}"
  mkdir -p "$dd"
  PERF_DATA_DIR="$dd"
  rm -f "$dd/runtime.json"
  local log="$dd/daemon.stdio.log"
  local t0 t1
  perf_mark t0
  if [ -n "$script" ]; then
    SGT_FAKE_SCRIPT="$script" setsid "$SGT_BIN" daemon --data-dir "$dd" >>"$log" 2>&1 &
  else
    env -u SGT_FAKE_SCRIPT setsid "$SGT_BIN" daemon --data-dir "$dd" >>"$log" 2>&1 &
  fi
  # `setsid` forks when the caller is already a process-group leader, so the
  # job's pid is not necessarily the daemon's. The descriptor is authoritative:
  # the daemon writes its own pid there, after the listener is live.
  local start_s=$SECONDS
  local deadline=$((SECONDS + timeout)) endpoint="" token="" pid=""
  while [ "$SECONDS" -lt "$deadline" ]; do
    if [ -f "$dd/runtime.json" ]; then
      endpoint="$(perf_descriptor_field "$dd" endpoint || true)"
      token="$(perf_descriptor_field "$dd" token || true)"
      pid="$(perf_descriptor_field "$dd" pid || true)"
      if [ -n "$endpoint" ] && [ -n "$token" ] && [ -n "$pid" ]; then
        if curl -sS -m 5 -o /dev/null -w '%{http_code}' "$endpoint/healthz" 2>/dev/null | grep -q 200; then
          break
        fi
      fi
    fi
    # Fail fast on a daemon that died instead of waiting out the whole
    # timeout: after a grace period, no process for this data dir means the
    # start failed and the log already says why.
    if [ $((SECONDS - start_s)) -ge 10 ] && ! pgrep -f "sgt daemon --data-dir $dd" > /dev/null 2>&1; then
      break
    fi
    sleep 0.02
  done
  perf_mark t1
  if [ -z "$endpoint" ]; then
    perf_warn "daemon log tail:"; tail -20 "$log" >&2 || true
    perf_die "daemon never published a usable descriptor for $dd"
  fi
  PERF_DAEMON_PID="$pid"
  PERF_ENDPOINT="$endpoint"
  PERF_TOKEN="$token"
  PERF_DAEMON_START_NS="$t0"
  PERF_DAEMON_READY_NS="$t1"
  PERF_DAEMON_READY_MS="$(perf_ms $((t1 - t0)))"
  PERF_FAKE_SCRIPT_IN_USE="$script"
}

# perf_empty_start_ms <scratch-base> [samples]
#
# Cold start of a daemon on a brand-new *empty* data dir, best of N. This is
# the fixed overhead every start pays — exec, DuckDB open, bind, descriptor —
# and it is what has to be subtracted from a loaded cold start before the
# remainder can be called "replay". Measured next to the loaded start rather
# than once per run, because a single sample taken minutes earlier on a busy
# container is not the same machine.
#
# The caller's current daemon coordinates are saved and restored, so this can
# be called from inside a scenario without losing track of its daemon.
perf_empty_start_ms() {
  local base="$1" samples="${2:-2}" best="" i tmpdd ms
  local save_pid="${PERF_DAEMON_PID:-}" save_ep="${PERF_ENDPOINT:-}"
  local save_tok="${PERF_TOKEN:-}" save_dd="${PERF_DATA_DIR:-}"
  for ((i = 1; i <= samples; i++)); do
    tmpdd="$base/empty-start-$i"
    rm -rf "$tmpdd"; mkdir -p "$tmpdd"
    perf_daemon_start "$tmpdd"
    ms="$PERF_DAEMON_READY_MS"
    perf_daemon_stop
    rm -rf "$tmpdd"
    best="$(awk -v a="$best" -v b="$ms" 'BEGIN{ if (a == "" || b + 0 < a + 0) print b; else print a }')"
  done
  PERF_DAEMON_PID="$save_pid"; PERF_ENDPOINT="$save_ep"
  PERF_TOKEN="$save_tok"; PERF_DATA_DIR="$save_dd"
  printf '%s' "$best"
}

# perf_daemon_stop [pid] [timeout-s] — SIGTERM, then SIGKILL if it will not go.
# Sets PERF_FORCE_KILLED=1 (and warns) when SIGKILL was needed: that is a
# finding, not a cleanup detail.
perf_daemon_stop() {
  local pid="${1:-${PERF_DAEMON_PID:-}}" timeout="${2:-20}" i
  [ -n "$pid" ] || return 0
  kill -0 "$pid" 2>/dev/null || { PERF_DAEMON_PID=""; return 0; }
  kill -TERM "$pid" 2>/dev/null || true
  for ((i = 0; i < timeout * 20; i++)); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.05
  done
  if kill -0 "$pid" 2>/dev/null; then
    perf_warn "daemon $pid ignored SIGTERM for ${timeout}s — SIGKILL"
    kill -9 "$pid" 2>/dev/null || true
    PERF_FORCE_KILLED=1
    sleep 0.3
  fi
  [ "$pid" = "${PERF_DAEMON_PID:-}" ] && PERF_DAEMON_PID=""
  return 0
}

# perf_daemon_kill9 [pid] — the S6 crash: no chance to shut down cleanly.
perf_daemon_kill9() {
  local pid="${1:-${PERF_DAEMON_PID:-}}"
  [ -n "$pid" ] || return 0
  kill -9 "$pid" 2>/dev/null || true
  local i
  for ((i = 0; i < 200; i++)); do
    kill -0 "$pid" 2>/dev/null || break
    sleep 0.05
  done
  [ "$pid" = "${PERF_DAEMON_PID:-}" ] && PERF_DAEMON_PID=""
  return 0
}

# --- /proc sampling --------------------------------------------------------

PERF_SAMPLE_HEADER='ts_ns,rss_kb,hwm_kb,threads,fds,utime_ticks,stime_ticks,cpu_sec,label'

# perf_sample_row <pid> [label] — one CSV row from /proc/<pid>/{status,stat,fd}.
# Prints nothing and returns 1 if the process is gone (a sampler must not die
# because the thing it samples did).
perf_sample_row() {
  local pid="$1" label="${2:-}"
  [ -d "/proc/$pid" ] || return 1
  local rss=0 hwm=0 thr=0 line
  while IFS= read -r line; do
    case "$line" in
      VmRSS:*) rss="${line//[!0-9]/}" ;;
      VmHWM:*) hwm="${line//[!0-9]/}" ;;
      Threads:*) thr="${line//[!0-9]/}" ;;
    esac
  done < "/proc/$pid/status" 2>/dev/null || return 1
  local stat rest fields utime=0 stime=0
  stat="$(< "/proc/$pid/stat")" 2>/dev/null || return 1
  rest="${stat#*') '}"
  read -ra fields <<< "$rest"
  # After the comm field, index 0 is stat field 3; utime is field 14, stime 15.
  utime="${fields[11]:-0}"
  stime="${fields[12]:-0}"
  local fds=0 entries
  entries=(/proc/"$pid"/fd/*)
  if [ -e "${entries[0]}" ]; then fds="${#entries[@]}"; fi
  local cpu
  cpu="$(awk -v u="$utime" -v s="$stime" -v t="$PERF_CLK_TCK" 'BEGIN{printf "%.3f", (u+s)/t}')"
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$(perf_now_ns)" "$rss" "$hwm" "$thr" "$fds" "$utime" "$stime" "$cpu" "$label"
}

# perf_sampler_start <pid> <csv> [interval-s] [label]
perf_sampler_start() {
  local pid="$1" csv="$2" interval="${3:-0.5}" label="${4:-}"
  echo "$PERF_SAMPLE_HEADER" > "$csv"
  (
    while kill -0 "$pid" 2>/dev/null; do
      perf_sample_row "$pid" "$label" >> "$csv" 2>/dev/null || true
      sleep "$interval"
    done
  ) &
  PERF_SAMPLER_PID=$!
}

perf_sampler_stop() {
  [ -n "${PERF_SAMPLER_PID:-}" ] || return 0
  kill "$PERF_SAMPLER_PID" 2>/dev/null || true
  wait "$PERF_SAMPLER_PID" 2>/dev/null || true
  PERF_SAMPLER_PID=""
  return 0
}

# perf_sample_stats <csv> — "peak_rss_kb peak_hwm_kb max_fds max_threads
# last_cpu_sec first_rss_kb last_rss_kb samples" from a sampler CSV.
perf_sample_stats() {
  python3 - "$1" <<'PY'
import csv, sys
rows = list(csv.DictReader(open(sys.argv[1])))
rows = [r for r in rows if r.get("rss_kb")]
if not rows:
    print("0 0 0 0 0 0 0 0"); raise SystemExit
i = lambda r, k: int(r[k] or 0)
print(max(i(r, "rss_kb") for r in rows),
      max(i(r, "hwm_kb") for r in rows),
      max(i(r, "fds") for r in rows),
      max(i(r, "threads") for r in rows),
      rows[-1]["cpu_sec"],
      i(rows[0], "rss_kb"),
      i(rows[-1], "rss_kb"),
      len(rows))
PY
}

# perf_proc_rss <pid> — VmRSS in kB, or 0.
perf_proc_rss() {
  local line
  while IFS= read -r line; do
    case "$line" in VmRSS:*) echo "${line//[!0-9]/}"; return 0 ;; esac
  done < "/proc/$1/status" 2>/dev/null
  echo 0
}
perf_proc_hwm() {
  local line
  while IFS= read -r line; do
    case "$line" in VmHWM:*) echo "${line//[!0-9]/}"; return 0 ;; esac
  done < "/proc/$1/status" 2>/dev/null
  echo 0
}
perf_proc_cpu_sec() {
  local stat rest fields
  stat="$(< "/proc/$1/stat")" 2>/dev/null || { echo 0; return 0; }
  rest="${stat#*') '}"
  read -ra fields <<< "$rest"
  awk -v u="${fields[11]:-0}" -v s="${fields[12]:-0}" -v t="$PERF_CLK_TCK" 'BEGIN{printf "%.3f", (u+s)/t}'
}
perf_proc_fds() {
  local entries
  entries=(/proc/"$1"/fd/*)
  if [ -e "${entries[0]}" ]; then echo "${#entries[@]}"; else echo 0; fi
}

# --- API driving -----------------------------------------------------------

PERF_ULID_ALPHABET=0123456789ABCDEFGHJKMNPQRSTVWXYZ

# A syntactically valid ULID (26 Crockford base32 chars, first char <= '7' so
# the 48-bit timestamp cannot overflow). The daemon only validates the shape.
perf_ulid() {
  local out="${PERF_ULID_ALPHABET:$((RANDOM % 8)):1}" i
  for ((i = 0; i < 25; i++)); do out+="${PERF_ULID_ALPHABET:$((RANDOM % 32)):1}"; done
  printf '%s' "$out"
}

perf_api() { # perf_api <method> <path> [json-body]
  local method="$1" path="$2" body="${3:-}"
  if [ -n "$body" ]; then
    curl -sS -m "${PERF_HTTP_TIMEOUT:-300}" -X "$method" \
      -H "Authorization: Bearer $PERF_TOKEN" -H 'content-type: application/json' \
      -d "$body" "$PERF_ENDPOINT$path"
  else
    curl -sS -m "${PERF_HTTP_TIMEOUT:-300}" -X "$method" \
      -H "Authorization: Bearer $PERF_TOKEN" "$PERF_ENDPOINT$path"
  fi
}

# perf_timed_get <path> <outfile> — GET timed with curl's own clock.
# Prints "wall_ns http_code bytes curl_total_s".
perf_timed_get() {
  local path="$1" out="${2:-/dev/null}" t0 t1 line
  perf_mark t0
  line="$(curl -sS -m "${PERF_HTTP_TIMEOUT:-300}" -o "$out" \
    -w '%{http_code} %{size_download} %{time_total}' \
    -H "Authorization: Bearer $PERF_TOKEN" "$PERF_ENDPOINT$path" 2>/dev/null)"
  perf_mark t1
  printf '%s %s\n' "$((t1 - t0))" "$line"
}

# perf_submit <intent> [cwd] [backend] — one synchronous submit; prints the
# work id (empty on failure). Body is built before the clock starts.
perf_submit() {
  local intent="$1" cwd="${2:-${PERF_REPO:-}}" backend="${3:-fake}" body resp
  body="$(jq -nc --arg cid "$(perf_ulid)" --arg intent "$intent" --arg cwd "$cwd" --arg b "$backend" \
    '{command_id:$cid, intent:$intent, backend:$b, origin:{client:"cli", cwd:$cwd}}')"
  resp="$(perf_api POST /v1/work "$body")"
  jq -r '.work.id // empty' <<< "$resp"
}

# perf_respond <work-id> <input> — prints the resulting work state.
perf_respond() {
  local id="$1" input="$2" body resp
  body="$(jq -nc --arg cid "$(perf_ulid)" --arg i "$input" '{command_id:$cid, input:$i}')"
  resp="$(perf_api POST "/v1/work/$id/input" "$body")"
  jq -r '.work.state // .error.code // "?"' <<< "$resp"
}

perf_cancel() {
  local id="$1" body
  body="$(jq -nc --arg cid "$(perf_ulid)" '{command_id:$cid}')"
  perf_api POST "/v1/work/$id/cancel" "$body" > /dev/null
}

perf_work_state() { perf_api GET "/v1/work/$1" | jq -r '.work.state // "?"'; }

perf_journal_head() { perf_api GET /v1/system | jq -r '.journal_head // 0'; }

perf_state_counts() { # "state:count" lines over the whole fleet
  perf_api GET /v1/work | jq -r '[.works[].state] | group_by(.) | map({(.[0]): length}) | add // {} | to_entries[] | "\(.key):\(.value)"'
}

perf_count_state() { perf_api GET /v1/work | jq --arg s "$1" '[.works[] | select(.state == $s)] | length'; }

perf_journal_events() { # events on disk, independent of the daemon
  local dd="${1:-$PERF_DATA_DIR}"
  cat "$dd"/journal/*.ndjson 2>/dev/null | wc -l | tr -d ' '
}

# perf_burst <n> <parallel> <calls-dir> [intent-prefix]
#
# n concurrent submits through the HTTP API (one curl per call, xargs -P for
# fan-out) writing one CSV line per call into <calls-dir>. The submit path
# runs the whole workflow inside the request with the fake backend, so a call's
# latency is a work's end-to-end latency.
perf_burst() {
  local n="$1" par="$2" dir="$3" prefix="${4:-perf}"
  mkdir -p "$dir"
  seq 1 "$n" | xargs -P "$par" -I@ \
    "$PERF_DIR/_submit-one.sh" "$PERF_ENDPOINT" "$PERF_TOKEN" "${PERF_REPO:-}" "$prefix-@" "$dir/@.csv"
}

# perf_collect_calls <calls-dir> <out-csv> — concatenate per-call CSVs.
perf_collect_calls() {
  local dir="$1" out="$2"
  echo 't_start_ns,wall_ns,curl_total_s,http_code,work_id,state' > "$out"
  cat "$dir"/*.csv 2>/dev/null >> "$out" || true
}

# --- statistics ------------------------------------------------------------

# perf_pctl — numbers on stdin, prints "count min p50 p95 p99 max mean".
#
# `python3 -c`, not a heredoc: a heredoc *is* python's stdin, which would eat
# the numbers being piped in (this cost one silent run of all-zero latency
# columns before it was caught).
perf_pctl() {
  python3 -c '
import sys
xs = sorted(float(x) for x in sys.stdin.read().split() if x.strip())
if not xs:
    print("0 0 0 0 0 0 0"); raise SystemExit
def p(q):
    if len(xs) == 1: return xs[0]
    k = q * (len(xs) - 1)
    lo, hi = int(k), min(int(k) + 1, len(xs) - 1)
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)
print(len(xs), "%.3f" % xs[0], "%.3f" % p(.5), "%.3f" % p(.95), "%.3f" % p(.99),
      "%.3f" % xs[-1], "%.3f" % (sum(xs) / len(xs)))
'
}

perf_ms() { awk -v n="$1" 'BEGIN{printf "%.2f", n/1000000}'; }
perf_s()  { awk -v n="$1" 'BEGIN{printf "%.3f", n/1000000000}'; }

# perf_latency_kv <prefix> <csv> <column-index-1based>
# Emits <prefix>_{count,min_ms,p50_ms,p95_ms,p99_ms,max_ms,mean_ms} from a
# calls CSV whose column holds nanoseconds.
perf_latency_kv() {
  local prefix="$1" csv="$2" col="${3:-2}" stats
  stats="$(tail -n +2 "$csv" | cut -d, -f"$col" | grep -E '^[0-9]+$' | awk '{printf "%.3f\n", $1/1000000}' | perf_pctl)"
  read -r c mn p50 p95 p99 mx mean <<< "$stats"
  perf_kv "${prefix}_count" "$c"
  perf_kv "${prefix}_min_ms" "$mn"
  perf_kv "${prefix}_p50_ms" "$p50"
  perf_kv "${prefix}_p95_ms" "$p95"
  perf_kv "${prefix}_p99_ms" "$p99"
  perf_kv "${prefix}_max_ms" "$mx"
  perf_kv "${prefix}_mean_ms" "$mean"
}

# --- disk accounting -------------------------------------------------------

perf_bytes() { du -sb "$1" 2>/dev/null | cut -f1 || echo 0; }

# perf_disk_kv <prefix> <data-dir> — journal/blobs/surfaces/total bytes are
# read straight off the filesystem and are accurate at any moment, live
# daemon or not. `<prefix>_duckdb_bytes` is not: DuckDB only checkpoints the
# analytics projection to disk at clean shutdown (`src/runtime/analytics.rs`
# — the journal is the source of truth and the projection is disposable), so
# a call against a *running* daemon reads whatever was on disk from the last
# checkpoint, not the projection's current size. Issue #13's measured
# example: S2's `post_disk_duckdb_bytes` read 12,288 B mid-run against a true
# post-shutdown size of 536,576 B (~100% underreport at that scenario's
# work count). **Callers wanting an accurate `duckdb_bytes` must call this
# after `perf_daemon_stop`**, not before it — every scenario in this
# directory does now (fixed alongside this comment; s6-crash.sh's own
# `perf_disk_kv` call was already correctly ordered and is what this fix's
# other call sites were brought in line with).
perf_disk_kv() { # perf_disk_kv <prefix> <data-dir>
  local prefix="$1" dd="$2"
  perf_kv "${prefix}_total_bytes" "$(perf_bytes "$dd")"
  perf_kv "${prefix}_journal_bytes" "$(perf_bytes "$dd/journal")"
  perf_kv "${prefix}_blobs_bytes" "$(perf_bytes "$dd/blobs")"
  perf_kv "${prefix}_surfaces_bytes" "$(perf_bytes "$dd/surfaces")"
  perf_kv "${prefix}_duckdb_bytes" "$(stat -c %s "$dd/projections/sergeant.duckdb" 2>/dev/null || echo 0)"
}

# --- hygiene sweep ---------------------------------------------------------

# perf_sgt_processes — sgt processes visible right now, one per line, with this
# harness's own shell and pgrep invocations filtered out. Never fails: pgrep
# and grep both exit 1 on "no matches", which under `set -o pipefail` would
# otherwise abort the caller.
perf_sgt_processes() {
  { pgrep -fa "release/sgt" 2>/dev/null; pgrep -fa "debug/sgt" 2>/dev/null; } \
    | grep -v "bash -c" | grep -v "scripts/perf" | grep -v "pgrep" || true
}

perf_sgt_process_count() {
  local list; list="$(perf_sgt_processes)"
  if [ -z "$list" ]; then echo 0; else printf '%s\n' "$list" | wc -l | tr -d ' '; fi
}


# perf_hygiene <label> [repo...] — the sweep P1-PERF ends every scenario with.
# Writes hygiene-<label>.txt and prints the verdict. Never exits nonzero: a
# dirty sweep is data the scenario must report, not a reason to abort.
perf_hygiene() {
  local label="$1"; shift
  local file="${PERF_OUT:-.}/hygiene-${label}.txt"
  local orphans strays worktrees=0 repo verdict=clean
  # Repos may arrive both as arguments and in PERF_REPOS; count each once.
  local repos=() seen existing
  for repo in "$@" "${PERF_REPOS[@]:-}"; do
    [ -n "$repo" ] && [ -d "$repo/.git" ] || continue
    seen=no
    for existing in "${repos[@]:-}"; do [ "$existing" = "$repo" ] && seen=yes; done
    [ "$seen" = no ] && repos+=("$repo")
  done
  # pgrep's own invocation is wrapped in a `bash -c` whose command line holds
  # the pattern; that self-match is filtered, nothing else is.
  orphans="$(perf_sgt_processes)"
  strays="$(ls -d /tmp/.tmp* /tmp/sgt-demo-* "/tmp/tmux-$(id -u)"/sgtperf-* 2>/dev/null || true)"
  {
    echo "hygiene sweep: $label   ($(date -u +%Y-%m-%dT%H:%M:%SZ))"
    echo "-- sgt processes"
    if [ -n "$orphans" ]; then echo "$orphans"; else echo "(none)"; fi
    echo "-- /tmp residue (.tmp*, sgt-demo-*, stale sgtperf tmux sockets)"
    if [ -n "$strays" ]; then echo "$strays"; else echo "(none)"; fi
    echo "-- worktrees"
    echo "   (a worktree under <data-dir>/surfaces/<work-id>/ belongs to that"
    echo "    work and is expected while the work is non-terminal: teardown"
    echo "    retires a surface only when the work reaches a terminal state.)"
    for repo in "${repos[@]:-}"; do
      echo "[$repo]"
      git -C "$repo" worktree list 2>/dev/null || true
    done
    echo "-- surfaces dirs"
    if [ -n "${PERF_DATA_DIR:-}" ] && [ -d "$PERF_DATA_DIR/surfaces" ]; then
      find "$PERF_DATA_DIR/surfaces" -mindepth 1 -maxdepth 1 2>/dev/null || true
    else
      echo "(no surfaces dir)"
    fi
  } > "$file"
  local n
  for repo in "${repos[@]:-}"; do
    n="$(git -C "$repo" worktree list 2>/dev/null | tail -n +2 | wc -l | tr -d ' ')"
    worktrees=$((worktrees + n))
  done
  local nproc_left=0
  [ -n "$orphans" ] && nproc_left="$(printf '%s\n' "$orphans" | wc -l | tr -d ' ')"
  # Surfaces are counted in two buckets, because they mean different things:
  # a *populated* surface dir is a worktree that outlived its work (dirty), an
  # *empty* one is a directory teardown did not remove. Measured separately so
  # the second is reportable as its own finding instead of being lost inside a
  # "dirty" verdict.
  local nsurf=0 nsurf_nonempty=0 d
  if [ -n "${PERF_DATA_DIR:-}" ] && [ -d "$PERF_DATA_DIR/surfaces" ]; then
    while IFS= read -r d; do
      [ -n "$d" ] || continue
      nsurf=$((nsurf + 1))
      [ -n "$(ls -A "$d" 2>/dev/null)" ] && nsurf_nonempty=$((nsurf_nonempty + 1))
    done < <(find "$PERF_DATA_DIR/surfaces" -mindepth 1 -maxdepth 1 -type d 2>/dev/null)
  fi
  [ "$nproc_left" -eq 0 ] && [ "$worktrees" -eq 0 ] && [ "$nsurf_nonempty" -eq 0 ] || verdict=dirty
  perf_say "hygiene[$label]: $verdict — sgt procs=$nproc_left stray worktrees=$worktrees populated surfaces=$nsurf_nonempty empty surface dirs=$((nsurf - nsurf_nonempty)) (detail: $file)"
  perf_kv "hygiene_${label}_verdict" "$verdict"
  perf_kv "hygiene_${label}_sgt_processes" "$nproc_left"
  perf_kv "hygiene_${label}_worktrees" "$worktrees"
  perf_kv "hygiene_${label}_surface_dirs" "$nsurf"
  perf_kv "hygiene_${label}_surface_dirs_populated" "$nsurf_nonempty"
  perf_kv "hygiene_${label}_surface_dirs_empty_leftover" "$((nsurf - nsurf_nonempty))"
  local nstray=0
  [ -n "$strays" ] && nstray="$(printf '%s\n' "$strays" | wc -l | tr -d ' ')"
  perf_kv "hygiene_${label}_tmp_residue" "$nstray"
  return 0
}

# --- environment record ----------------------------------------------------

perf_environment() { # perf_environment <outfile>
  local out="$1"
  {
    echo "commit: ${PERF_COMMIT:-unknown}"
    echo "binary: $SGT_BIN"
    echo "binary mtime: ${PERF_BIN_MTIME:-0} ($(date -u -d "@${PERF_BIN_MTIME:-0}" +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || echo '?'))"
    echo "sgt --version: $("$SGT_BIN" --version 2>&1 | head -1)"
    echo "nproc: $(nproc)"
    echo "cgroup cpu.max: $(cat /sys/fs/cgroup/cpu.max 2>/dev/null || echo 'absent (cgroup v1)')"
    echo "cgroup v1 cfs_quota_us: $(cat /sys/fs/cgroup/cpu/cpu.cfs_quota_us 2>/dev/null || echo absent)"
    echo "cgroup v1 cfs_period_us: $(cat /sys/fs/cgroup/cpu/cpu.cfs_period_us 2>/dev/null || echo absent)"
    echo "MemTotal: $(awk '/MemTotal/{print $2" kB"}' /proc/meminfo)"
    echo "kernel: $(uname -sr)"
    echo "CLK_TCK: $PERF_CLK_TCK"
    echo "df (output mount):"
    df -h "${PERF_OUT:-.}" | sed 's/^/  /'
    echo "ulimit -n: $(ulimit -n)"
  } | tee "$out"
}
