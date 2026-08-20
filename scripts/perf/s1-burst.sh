#!/usr/bin/env bash
# S1 — burst submissions.
#
# Concurrent `POST /v1/work` bursts of 1, 5, 10, 20, 50 against one daemon on a
# fresh data dir with the completing fake script. Per burst: wall time,
# per-call latency min/median/p95/max, daemon RSS before/peak/after, daemon CPU
# time, journal events appended, and whether every work reached `completed`.
#
#   scripts/perf/s1-burst.sh <outdir>
#
# Knobs: PERF_S1_BURSTS (default "1 5 10 20 50"), PERF_S1_SETTLE (2 s).
#
# Bursts share one daemon deliberately — "RSS before/peak/after" per burst is
# only readable as a series if the process is the same one throughout, and a
# climb across the series is itself the leak signal S2 goes on to test.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s1-burst.sh <outdir>}"
perf_init "$OUT" s1-burst

BURSTS="${PERF_S1_BURSTS:-1 5 10 20 50}"
SETTLE="${PERF_S1_SETTLE:-2}"

DD="$PERF_SCRATCH/s1/data"
ESTATE="$PERF_SCRATCH/s1/estate"
rm -rf "$PERF_SCRATCH/s1"
mkdir -p "$DD"
perf_estate_scaffold "$ESTATE"
REPO="$PERF_REPO"
perf_daemon_start "$DD"
perf_say "daemon pid $PERF_DAEMON_PID at $PERF_ENDPOINT (ready in ${PERF_DAEMON_READY_MS} ms)"
perf_kv bursts "$BURSTS"
perf_kv daemon_ready_ms "$PERF_DAEMON_READY_MS"
perf_kv idle_rss_kb "$(perf_proc_rss "$PERF_DAEMON_PID")"
perf_kv idle_fds "$(perf_proc_fds "$PERF_DAEMON_PID")"

for n in $BURSTS; do
  perf_step "burst of $n"
  calls="$PERF_SCRATCH/s1/calls-$n"
  rm -rf "$calls"; mkdir -p "$calls"
  sample="$PERF_OUT/s1-sample-$n.csv"
  csv="$PERF_OUT/s1-calls-$n.csv"

  rss_before="$(perf_proc_rss "$PERF_DAEMON_PID")"
  fds_before="$(perf_proc_fds "$PERF_DAEMON_PID")"
  cpu_before="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
  head_before="$(perf_journal_head)"

  perf_sampler_start "$PERF_DAEMON_PID" "$sample" 0.1 "burst$n"
  perf_mark t0
  perf_burst "$n" "$n" "$calls" "s1b$n"
  perf_mark t1
  perf_sampler_stop
  sleep "$SETTLE"

  rss_after="$(perf_proc_rss "$PERF_DAEMON_PID")"
  fds_after="$(perf_proc_fds "$PERF_DAEMON_PID")"
  cpu_after="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
  head_after="$(perf_journal_head)"

  perf_collect_calls "$calls" "$csv"
  read -r peak_rss peak_hwm max_fds max_thr _ _ _ nsamples <<< "$(perf_sample_stats "$sample")"
  ok="$(tail -n +2 "$csv" | awk -F, '$4 == 201' | wc -l | tr -d ' ')"
  completed="$(tail -n +2 "$csv" | awk -F, '$6 == "completed"' | wc -l | tr -d ' ')"
  ids="$(tail -n +2 "$csv" | cut -d, -f5 | grep -c '[A-Z0-9]' || true)"

  # Confirm from the projection, not only from the submit response.
  confirmed=0
  while read -r id; do
    [ -n "$id" ] || continue
    [ "$(perf_work_state "$id")" = "completed" ] && confirmed=$((confirmed + 1))
  done < <(tail -n +2 "$csv" | cut -d, -f5)

  perf_kv "b${n}_wall_s" "$(perf_s $((t1 - t0)))"
  perf_kv "b${n}_throughput_per_s" "$(awk -v n="$n" -v ns="$((t1 - t0))" 'BEGIN{printf "%.2f", n/(ns/1000000000)}')"
  perf_latency_kv "b${n}_lat" "$csv" 2
  perf_kv "b${n}_curl_total_max_s" "$(tail -n +2 "$csv" | cut -d, -f3 | sort -g | tail -1)"
  perf_kv "b${n}_http_201" "$ok/$n"
  perf_kv "b${n}_work_ids" "$ids"
  perf_kv "b${n}_completed_in_response" "$completed"
  perf_kv "b${n}_completed_confirmed" "$confirmed"
  perf_kv "b${n}_rss_before_kb" "$rss_before"
  perf_kv "b${n}_rss_peak_kb" "$peak_rss"
  perf_kv "b${n}_rss_after_kb" "$rss_after"
  perf_kv "b${n}_hwm_kb" "$peak_hwm"
  perf_kv "b${n}_fds_before" "$fds_before"
  perf_kv "b${n}_fds_peak" "$max_fds"
  perf_kv "b${n}_fds_after" "$fds_after"
  perf_kv "b${n}_threads_peak" "$max_thr"
  perf_kv "b${n}_cpu_delta_s" "$(awk -v a="$cpu_after" -v b="$cpu_before" 'BEGIN{printf "%.3f", a-b}')"
  perf_kv "b${n}_journal_events" "$((head_after - head_before))"
  perf_kv "b${n}_events_per_work" "$(awk -v e="$((head_after - head_before))" -v n="$n" 'BEGIN{printf "%.1f", e/n}')"
  perf_kv "b${n}_samples" "$nsamples"
  perf_say "wall $(perf_s $((t1 - t0)))s · 201s $ok/$n · completed $confirmed/$n · rss ${rss_before}→peak ${peak_rss}→${rss_after} kB · +$((head_after - head_before)) events"
done

perf_step "fleet and disk"
perf_kv fleet_total "$(perf_api GET /v1/work | jq '.works | length')"
perf_kv fleet_states "$(perf_state_counts | tr '\n' ' ')"
perf_kv journal_events_on_disk "$(perf_journal_events "$DD")"

perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` (see its doc in
# common.sh) — `duckdb_bytes` only reflects reality post-checkpoint.
perf_disk_kv disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s1 "$REPO"
perf_summary
