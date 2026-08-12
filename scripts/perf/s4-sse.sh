#!/usr/bin/env bash
# S4 — SSE fan-out.
#
# 25 concurrent SSE subscribers held open through a 20-work burst: every
# subscriber must see the burst's terminal events; daemon fd count is measured
# during and after; then the burst is repeated with 10 subscribers killed
# mid-flight to prove a disconnect does not wedge the daemon.
#
#   scripts/perf/s4-sse.sh <outdir>
#
# Knobs: PERF_S4_SUBS (25), PERF_S4_BURST (20), PERF_S4_KILL (10).
#
# Subscribers attach with `?from=<journal head>` rather than at the live tail,
# so "did this subscriber see the burst" is not a race against how fast curl
# got its socket up: the resume-by-seq contract is what guarantees delivery,
# and it is exactly what is being measured.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s4-sse.sh <outdir>}"
perf_init "$OUT" s4-sse

SUBS="${PERF_S4_SUBS:-25}"
BURST="${PERF_S4_BURST:-20}"
KILLN="${PERF_S4_KILL:-10}"

DD="$PERF_SCRATCH/s4/data"
REPO="$PERF_SCRATCH/s4/repo"
rm -rf "$PERF_SCRATCH/s4"
mkdir -p "$DD"
perf_seed_repo "$REPO"
perf_daemon_start "$DD"
perf_say "daemon pid $PERF_DAEMON_PID · $SUBS subscribers · burst $BURST"
perf_kv subscribers "$SUBS"
perf_kv burst "$BURST"
perf_kv killed_mid_flight "$KILLN"

STREAM_DIR="$PERF_SCRATCH/s4/streams"
mkdir -p "$STREAM_DIR"
SUB_PIDS=()

sse_attach() { # sse_attach <index> <from> <phase>
  local i="$1" from="$2" phase="$3"
  curl -sS --no-buffer -m 900 -H "Authorization: Bearer $PERF_TOKEN" \
    "$PERF_ENDPOINT/v1/events/stream?from=$from" > "$STREAM_DIR/$phase-$i.sse" 2>/dev/null &
  SUB_PIDS+=($!)
}

sse_kinds() { # count of one event kind in a stream capture
  local file="$1" kind="$2"
  sed -n 's/^data: //p' "$file" 2>/dev/null | jq -r 'select(.kind == "'"$kind"'") | .kind' 2>/dev/null | wc -l | tr -d ' '
}

kill_subs() {
  local pid
  for pid in "${SUB_PIDS[@]:-}"; do [ -n "$pid" ] && kill "$pid" 2>/dev/null || true; done
  for pid in "${SUB_PIDS[@]:-}"; do [ -n "$pid" ] && wait "$pid" 2>/dev/null || true; done
  SUB_PIDS=()
}

FDS_IDLE="$(perf_proc_fds "$PERF_DAEMON_PID")"
RSS_IDLE="$(perf_proc_rss "$PERF_DAEMON_PID")"
perf_kv fds_idle "$FDS_IDLE"
perf_kv rss_idle_kb "$RSS_IDLE"

# ---------------------------------------------------------------- phase 1
perf_step "phase 1 — $SUBS subscribers through a $BURST-work burst"
HEAD="$(perf_journal_head)"
for ((i = 1; i <= SUBS; i++)); do sse_attach "$i" "$HEAD" p1; done
sleep 2
FDS_ATTACHED="$(perf_proc_fds "$PERF_DAEMON_PID")"
perf_kv fds_with_subscribers "$FDS_ATTACHED"
perf_kv fds_per_subscriber "$(awk -v a="$FDS_IDLE" -v b="$FDS_ATTACHED" -v n="$SUBS" 'BEGIN{printf "%.2f", (b-a)/n}')"

SAMPLE1="$PERF_OUT/s4-sample-p1.csv"
perf_sampler_start "$PERF_DAEMON_PID" "$SAMPLE1" 0.2 p1
CALLS="$PERF_SCRATCH/s4/calls-p1"
perf_mark b0
perf_burst "$BURST" "$BURST" "$CALLS" s4p1
perf_mark b1
sleep 3
perf_sampler_stop
FDS_DURING="$(perf_proc_fds "$PERF_DAEMON_PID")"
read -r peak_rss peak_hwm max_fds max_thr last_cpu _ _ _ <<< "$(perf_sample_stats "$SAMPLE1")"

perf_collect_calls "$CALLS" "$PERF_OUT/s4-calls-p1.csv"
perf_kv p1_burst_wall_s "$(perf_s $((b1 - b0)))"
perf_latency_kv p1_lat "$PERF_OUT/s4-calls-p1.csv" 2
perf_kv p1_rss_peak_kb "$peak_rss"
perf_kv p1_fds_peak "$max_fds"
perf_kv p1_threads_peak "$max_thr"
perf_kv p1_cpu_s "$last_cpu"

kill_subs
sleep 1

full=0; short=0; min_seen=999999; max_seen=0
for ((i = 1; i <= SUBS; i++)); do
  n="$(sse_kinds "$STREAM_DIR/p1-$i.sse" work.completed)"
  [ "$n" -ge "$BURST" ] && full=$((full + 1)) || short=$((short + 1))
  [ "$n" -lt "$min_seen" ] && min_seen="$n"
  [ "$n" -gt "$max_seen" ] && max_seen="$n"
done
perf_kv p1_subscribers_seeing_all "$full/$SUBS"
perf_kv p1_subscribers_short "$short"
perf_kv p1_completed_seen_min "$min_seen"
perf_kv p1_completed_seen_max "$max_seen"
perf_kv p1_stream_bytes_max "$(du -b "$STREAM_DIR"/p1-*.sse 2>/dev/null | cut -f1 | sort -n | tail -1)"
perf_say "phase 1: $full/$SUBS subscribers saw all $BURST work.completed events (min seen $min_seen)"

sleep 3
FDS_AFTER1="$(perf_proc_fds "$PERF_DAEMON_PID")"
perf_kv p1_fds_after_disconnect "$FDS_AFTER1"
perf_kv p1_fds_returned_to_idle "$([ "$FDS_AFTER1" -le $((FDS_IDLE + 2)) ] && echo yes || echo "no (idle $FDS_IDLE)")"
perf_kv p1_rss_after_kb "$(perf_proc_rss "$PERF_DAEMON_PID")"

# ---------------------------------------------------------------- phase 2
perf_step "phase 2 — same burst, $KILLN subscribers killed mid-flight"
HEAD2="$(perf_journal_head)"
for ((i = 1; i <= SUBS; i++)); do sse_attach "$i" "$HEAD2" p2; done
sleep 2
SAMPLE2="$PERF_OUT/s4-sample-p2.csv"
perf_sampler_start "$PERF_DAEMON_PID" "$SAMPLE2" 0.2 p2
CALLS2="$PERF_SCRATCH/s4/calls-p2"
perf_mark c0
perf_burst "$BURST" "$BURST" "$CALLS2" s4p2 &
BURST_JOB=$!
# Kill the first KILLN subscribers while the burst is in flight — an abrupt
# client disconnect is the case that must not wedge the writer behind it.
sleep 0.3
killed=0
for ((i = 0; i < KILLN && i < ${#SUB_PIDS[@]}; i++)); do
  kill -9 "${SUB_PIDS[$i]}" 2>/dev/null && killed=$((killed + 1)) || true
done
wait "$BURST_JOB" || true
perf_mark c1
sleep 3
perf_sampler_stop
perf_kv p2_subscribers_killed "$killed"
perf_kv p2_burst_wall_s "$(perf_s $((c1 - c0)))"
perf_collect_calls "$CALLS2" "$PERF_OUT/s4-calls-p2.csv"
perf_latency_kv p2_lat "$PERF_OUT/s4-calls-p2.csv" 2
perf_kv p2_http_201 "$(tail -n +2 "$PERF_OUT/s4-calls-p2.csv" | awk -F, '$4 == 201' | wc -l | tr -d ' ')/$BURST"

read -r peak_rss2 peak_hwm2 max_fds2 max_thr2 last_cpu2 _ _ _ <<< "$(perf_sample_stats "$SAMPLE2")"
perf_kv p2_rss_peak_kb "$peak_rss2"
perf_kv p2_fds_peak "$max_fds2"

kill_subs
sleep 2

survivors_full=0
for ((i = KILLN + 1; i <= SUBS; i++)); do
  n="$(sse_kinds "$STREAM_DIR/p2-$i.sse" work.completed)"
  [ "$n" -ge "$BURST" ] && survivors_full=$((survivors_full + 1))
done
perf_kv p2_survivors_seeing_all "$survivors_full/$((SUBS - KILLN))"

# The daemon must still be a daemon: healthy, answering, and able to take work.
HEALTH="$(curl -sS -m 5 -o /dev/null -w '%{http_code}' "$PERF_ENDPOINT/healthz" 2>/dev/null || echo 000)"
perf_kv post_kill_healthz "$HEALTH"
PROBE="$(perf_submit "S4 post-disconnect liveness probe")"
perf_kv post_kill_submit_work_id "${PROBE:-none}"
perf_kv post_kill_submit_state "$([ -n "$PROBE" ] && perf_work_state "$PROBE" || echo none)"
perf_kv post_kill_wedged "$([ -n "$PROBE" ] && echo no || echo YES)"

sleep 3
FDS_FINAL="$(perf_proc_fds "$PERF_DAEMON_PID")"
perf_kv fds_final "$FDS_FINAL"
perf_kv fds_leaked_vs_idle "$((FDS_FINAL - FDS_IDLE))"
perf_kv rss_final_kb "$(perf_proc_rss "$PERF_DAEMON_PID")"
perf_kv rss_delta_vs_idle_kb "$(($(perf_proc_rss "$PERF_DAEMON_PID") - RSS_IDLE))"
perf_kv fleet_states "$(perf_state_counts | tr '\n' ' ')"

perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` (see its doc in
# common.sh) — `duckdb_bytes` only reflects reality post-checkpoint.
perf_disk_kv disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s4 "$REPO"
perf_summary
