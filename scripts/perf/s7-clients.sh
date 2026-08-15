#!/usr/bin/env bash
# S7 — client-process hygiene.
#
# The TUI needs a pty, so it is driven under tmux (on its own socket, so this
# never touches another tmux server). Measured here:
#
#   * TUI CPU while idle-but-live — a busy render loop is a finding;
#   * the SSE tail stays live through a burst (the fleet screen sees new work);
#   * `doctor` / `status` latency under load;
#   * `q` exits, and the pane's exit status says it exited cleanly;
#   * the 2026-08-09 orphan observation: a TUI whose pty is destroyed
#     (`tmux kill-server`) must die on SIGTERM — reproduced or refuted, with
#     the time to exit and whether SIGKILL was needed.
#
#   scripts/perf/s7-clients.sh <outdir>
#
# Knobs: PERF_S7_IDLE (30 s), PERF_S7_BURST (20).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s7-clients.sh <outdir>}"
perf_init "$OUT" s7-clients

command -v tmux >/dev/null || perf_die "S7 needs tmux (the TUI needs a pty)"

IDLE="${PERF_S7_IDLE:-30}"
BURST="${PERF_S7_BURST:-20}"
SOCK_MAIN="sgtperf-main-$$"
SOCK_ORPHAN="sgtperf-orphan-$$"

DD="$PERF_SCRATCH/s7/data"
REPO="$PERF_SCRATCH/s7/repo"
rm -rf "$PERF_SCRATCH/s7"
mkdir -p "$DD"
perf_seed_repo "$REPO"
perf_daemon_start "$DD"
perf_kv idle_window_s "$IDLE"
perf_kv burst "$BURST"

TMUX_SOCK_DIR="/tmp/tmux-$(id -u)"
# kill-server does not always unlink the socket, and this scenario's own
# sockets must be gone before the sweep runs or the sweep reports its own
# harness as /tmp residue.
s7_kill_tmux() {
  tmux -L "$SOCK_MAIN" kill-server 2>/dev/null || true
  tmux -L "$SOCK_ORPHAN" kill-server 2>/dev/null || true
  rm -f "$TMUX_SOCK_DIR/$SOCK_MAIN" "$TMUX_SOCK_DIR/$SOCK_ORPHAN" 2>/dev/null || true
}
s7_cleanup() { s7_kill_tmux; perf_teardown; }
trap s7_cleanup EXIT

# Something for the fleet screen to show before the TUI attaches.
SEED_WORK="$(perf_submit "S7 seed work so the fleet screen is not empty")"
perf_kv seed_work "${SEED_WORK:-none}"

# ---------------------------------------------------------------- attach
perf_step "attaching the TUI under tmux"
tmux -L "$SOCK_MAIN" new-session -d -s tui -x 200 -y 50 \
  "exec '$SGT_BIN' --data-dir '$DD' tui"
tmux -L "$SOCK_MAIN" set-option -t tui remain-on-exit on >/dev/null 2>&1 || true
sleep 3
TUI_PID="$(tmux -L "$SOCK_MAIN" list-panes -t tui -F '#{pane_pid}' 2>/dev/null | head -1)"
[ -n "$TUI_PID" ] && kill -0 "$TUI_PID" 2>/dev/null || perf_die "the TUI did not start under tmux"
perf_kv tui_pid "$TUI_PID"
perf_kv tui_argv "$(tr '\0' ' ' < "/proc/$TUI_PID/cmdline" 2>/dev/null)"
tmux -L "$SOCK_MAIN" capture-pane -t tui -p > "$PERF_OUT/s7-pane-attached.txt" 2>/dev/null || true
perf_kv tui_pane_nonempty "$([ -s "$PERF_OUT/s7-pane-attached.txt" ] && echo yes || echo no)"
perf_kv tui_pane_shows_seed_work "$(grep -qF "${SEED_WORK:-__none__}" "$PERF_OUT/s7-pane-attached.txt" && echo yes || echo no)"
perf_kv daemon_fds_with_tui "$(perf_proc_fds "$PERF_DAEMON_PID")"

# ---------------------------------------------------------------- idle CPU
perf_step "TUI CPU over ${IDLE}s idle-but-live"
TUI_SAMPLE="$PERF_OUT/s7-tui-idle.csv"
perf_sampler_start "$TUI_PID" "$TUI_SAMPLE" 1 idle
cpu0="$(perf_proc_cpu_sec "$TUI_PID")"
rss0="$(perf_proc_rss "$TUI_PID")"
sleep "$IDLE"
cpu1="$(perf_proc_cpu_sec "$TUI_PID")"
rss1="$(perf_proc_rss "$TUI_PID")"
perf_sampler_stop
IDLE_CPU_PCT="$(awk -v a="$cpu0" -v b="$cpu1" -v t="$IDLE" 'BEGIN{printf "%.2f", (b-a)*100/t}')"
perf_kv tui_idle_cpu_s "$(awk -v a="$cpu0" -v b="$cpu1" 'BEGIN{printf "%.3f", b-a}')"
perf_kv tui_idle_cpu_pct "$IDLE_CPU_PCT"
perf_kv tui_idle_rss_start_kb "$rss0"
perf_kv tui_idle_rss_end_kb "$rss1"
perf_kv tui_idle_rss_delta_kb "$((rss1 - rss0))"
perf_kv tui_idle_verdict "$(awk -v p="$IDLE_CPU_PCT" 'BEGIN{ if (p > 5.0) print "busy-loop-suspect"; else print "quiet" }')"
perf_say "TUI idle CPU ${IDLE_CPU_PCT}% over ${IDLE}s"

DAEMON_CPU_IDLE="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
perf_kv daemon_cpu_during_tui_idle_s "$DAEMON_CPU_IDLE"

# ---------------------------------------------------------------- burst
perf_step "burst of $BURST with the TUI attached"
FLEET_BEFORE="$(perf_api GET /v1/work | jq '.works | length')"
tui_cpu_before="$(perf_proc_cpu_sec "$TUI_PID")"
tui_rss_before="$(perf_proc_rss "$TUI_PID")"
perf_sampler_start "$TUI_PID" "$PERF_OUT/s7-tui-burst.csv" 0.5 burst

calls="$PERF_SCRATCH/s7/calls"
perf_mark b0
perf_burst "$BURST" "$BURST" "$calls" s7b &
burst_job=$!

# The diagnostic commands, concurrent with the burst.
sleep 0.3
perf_mark st0; "$SGT_BIN" --data-dir "$DD" --json status > "$PERF_OUT/s7-status.json" 2>/dev/null || true; perf_mark st1
perf_kv sgt_status_ms "$(perf_ms $((st1 - st0)))"
perf_mark d0; "$SGT_BIN" --data-dir "$DD" --json doctor > "$PERF_OUT/s7-doctor.json" 2>/dev/null || true; perf_mark d1
perf_kv sgt_doctor_ms "$(perf_ms $((d1 - d0)))"
perf_kv sgt_doctor_healthy "$(jq -r '.healthy // "?"' "$PERF_OUT/s7-doctor.json" 2>/dev/null || echo '?')"

wait "$burst_job" 2>/dev/null || true
perf_mark b1
perf_sampler_stop
perf_collect_calls "$calls" "$PERF_OUT/s7-calls.csv"
perf_kv burst_wall_s "$(perf_s $((b1 - b0)))"
perf_latency_kv burst_lat "$PERF_OUT/s7-calls.csv" 2

tui_cpu_after="$(perf_proc_cpu_sec "$TUI_PID")"
perf_kv tui_burst_cpu_s "$(awk -v a="$tui_cpu_before" -v b="$tui_cpu_after" 'BEGIN{printf "%.3f", b-a}')"
perf_kv tui_burst_rss_delta_kb "$(($(perf_proc_rss "$TUI_PID") - tui_rss_before))"
perf_kv tui_alive_after_burst "$(kill -0 "$TUI_PID" 2>/dev/null && echo yes || echo NO)"

sleep 3
tmux -L "$SOCK_MAIN" capture-pane -t tui -p > "$PERF_OUT/s7-pane-after-burst.txt" 2>/dev/null || true
FLEET_AFTER="$(perf_api GET /v1/work | jq '.works | length')"
NEW_ID="$(tail -n +2 "$PERF_OUT/s7-calls.csv" | awk -F, '$4 == 201 {print $5; exit}')"
perf_kv fleet_before "$FLEET_BEFORE"
perf_kv fleet_after "$FLEET_AFTER"
perf_kv tui_pane_shows_new_work "$([ -n "$NEW_ID" ] && grep -qF "$NEW_ID" "$PERF_OUT/s7-pane-after-burst.txt" && echo yes || echo no)"
# The fleet screen truncates ids; a live SSE tail is also visible as a changed
# pane and a count that moved. Both are recorded, neither is asserted here.
perf_kv tui_pane_changed "$(cmp -s "$PERF_OUT/s7-pane-attached.txt" "$PERF_OUT/s7-pane-after-burst.txt" && echo no || echo yes)"

# ---------------------------------------------------------------- q exits
perf_step "q exits and restores the terminal"
tmux -L "$SOCK_MAIN" send-keys -t tui q
qgone=no
perf_mark q0
for i in $(seq 1 100); do
  kill -0 "$TUI_PID" 2>/dev/null || { qgone=yes; break; }
  sleep 0.05
done
perf_mark q1
perf_kv tui_q_exited "$qgone"
perf_kv tui_q_exit_ms "$(perf_ms $((q1 - q0)))"
perf_kv tui_q_pane_dead "$(tmux -L "$SOCK_MAIN" list-panes -t tui -F '#{pane_dead}' 2>/dev/null | head -1)"
perf_kv tui_q_exit_status "$(tmux -L "$SOCK_MAIN" list-panes -t tui -F '#{pane_dead_status}' 2>/dev/null | head -1)"
if [ "$qgone" != yes ]; then
  perf_warn "TUI did not exit on q — killing"
  kill -9 "$TUI_PID" 2>/dev/null || true
fi
tmux -L "$SOCK_MAIN" kill-server 2>/dev/null || true
sleep 1
perf_kv daemon_fds_after_tui "$(perf_proc_fds "$PERF_DAEMON_PID")"

# ------------------------------------------------- orphan SIGTERM repro
perf_step "orphan repro: tmux kill-server, then SIGTERM the surviving TUI"
tmux -L "$SOCK_ORPHAN" new-session -d -s orphan -x 200 -y 50 \
  "exec '$SGT_BIN' --data-dir '$DD' tui"
sleep 3
ORPHAN_PID="$(tmux -L "$SOCK_ORPHAN" list-panes -t orphan -F '#{pane_pid}' 2>/dev/null | head -1)"
perf_kv orphan_tui_pid "${ORPHAN_PID:-none}"
if [ -z "${ORPHAN_PID:-}" ] || ! kill -0 "$ORPHAN_PID" 2>/dev/null; then
  perf_kv orphan_repro "blocked: second TUI never started"
else
  tmux -L "$SOCK_ORPHAN" kill-server 2>/dev/null || true
  sleep 1
  if kill -0 "$ORPHAN_PID" 2>/dev/null; then
    perf_kv orphan_survived_pty_destruction yes
    perf_kv orphan_state_after_kill_server "$(awk '/^State:/{print $2, $3}' "/proc/$ORPHAN_PID/status" 2>/dev/null)"
    # Process state is recorded on both sides of the signal: "still there" is
    # only a finding if it is a live process, and a `Z` would mean it did exit
    # and simply was not reaped.
    perf_kv orphan_wchan_before_sigterm "$(cat "/proc/$ORPHAN_PID/wchan" 2>/dev/null || echo unknown)"
    kill -TERM "$ORPHAN_PID" 2>/dev/null || true
    perf_mark o0
    gone=no
    for i in $(seq 1 40); do   # 2 s at 50 ms
      kill -0 "$ORPHAN_PID" 2>/dev/null || { gone=yes; break; }
      sleep 0.05
    done
    perf_mark o1
    perf_kv orphan_sigterm_exit_ms "$(perf_ms $((o1 - o0)))"
    perf_kv orphan_state_after_sigterm "$(awk '/^State:/{print $2, $3}' "/proc/$ORPHAN_PID/status" 2>/dev/null || echo gone)"
    # Caught-signal mask, kept as a hex string: whether SIGTERM (bit 0x4000)
    # is in it separates "the default action never ran" from "a handler is
    # installed and did not finish".
    perf_kv orphan_sigcgt_after_sigterm "0x$(awk '/^SigCgt:/{print $2}' "/proc/$ORPHAN_PID/status" 2>/dev/null || echo gone)"
    if [ "$gone" = yes ]; then
      perf_kv orphan_repro "refuted: SIGTERM reaped it within 2s"
      perf_kv orphan_needed_sigkill no
    else
      perf_kv orphan_repro "REPRODUCED: alive 2s after SIGTERM"
      kill -9 "$ORPHAN_PID" 2>/dev/null || true
      k=no
      for i in $(seq 1 100); do   # 5 s at 50 ms
        kill -0 "$ORPHAN_PID" 2>/dev/null || { k=yes; break; }
        sleep 0.05
      done
      perf_kv orphan_needed_sigkill yes
      perf_kv orphan_dead_after_sigkill "$k"
      perf_kv orphan_state_after_sigkill "$(awk '/^State:/{print $2, $3}' "/proc/$ORPHAN_PID/status" 2>/dev/null || echo gone)"
      perf_warn "orphan TUI $ORPHAN_PID needed SIGKILL — reproduced the 2026-08-09 observation"
    fi
  else
    perf_kv orphan_survived_pty_destruction no
    perf_kv orphan_repro "refuted: kill-server alone reaped the TUI"
    perf_kv orphan_needed_sigkill no
  fi
fi
perf_say "orphan repro: $(grep -m1 '^orphan_repro' "$PERF_SUMMARY_TSV" | cut -f2-)"

perf_kv fleet_states "$(perf_state_counts | tr '\n' ' ')"
s7_kill_tmux
perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` (see its doc in
# common.sh) — `duckdb_bytes` only reflects reality post-checkpoint.
perf_disk_kv disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s7 "$REPO"
perf_summary
