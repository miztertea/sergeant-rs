#!/usr/bin/env bash
# S5 — journal scale: rebuild + analytics.
#
# One data dir is grown in waves to each mark (10k / 25k / 50k events). At every
# mark the daemon is stopped and restarted, and the cold start is timed: spawn
# to first answered request is journal replay + registry fold + DuckDB rebuild +
# listener bind, which is the whole "rebuild-on-start is the only population
# path" cost. Peak RSS, every canned analytics question's latency, and the
# DuckDB file size are recorded at each mark.
#
#   scripts/perf/s5-journal.sh <outdir>
#
# Knobs: PERF_S5_MARKS ("10000 25000 50000"), PERF_S5_WAVE (25),
#        PERF_S5_READY_TIMEOUT (600).
#
# The §21 rebuild budget claim (M5 ledger: ~15k events/s) is reported as a
# measured events/s at each mark; whether it holds is the runner's finding, not
# this script's assertion.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s5-journal.sh <outdir>}"
perf_init "$OUT" s5-journal

MARKS="${PERF_S5_MARKS:-10000 25000 50000}"
WAVE="${PERF_S5_WAVE:-25}"
READY_TIMEOUT="${PERF_S5_READY_TIMEOUT:-600}"

DD="$PERF_SCRATCH/s5/data"
REPO="$PERF_SCRATCH/s5/repo"
rm -rf "$PERF_SCRATCH/s5"
mkdir -p "$DD"
perf_seed_repo "$REPO"
perf_daemon_start "$DD" "" "$READY_TIMEOUT"
perf_kv marks "$MARKS"
perf_kv wave_size "$WAVE"
PERF_FIRST_START_MS="$PERF_DAEMON_READY_MS"
perf_kv first_start_empty_ms "$PERF_FIRST_START_MS"

GROWTH_CSV="$PERF_OUT/s5-growth.csv"
echo 'wave,works_total,journal_head,wall_ns,rss_kb,disk_bytes' > "$GROWTH_CSV"
MARK_CSV="$PERF_OUT/s5-marks.csv"
echo 'mark,events_on_disk,segments,restart_ms,rebuild_events_per_s,rss_after_start_kb,hwm_after_start_kb,duckdb_bytes,journal_bytes,data_dir_bytes' > "$MARK_CSV"
ANALYTICS_CSV="$PERF_OUT/s5-analytics.csv"
echo 'mark,query,wall_ns,http_code,bytes,rows' > "$ANALYTICS_CSV"

QUERIES="$(perf_api GET /v1/analytics | jq -r '.queries[].name' | tr '\n' ' ')"
perf_kv canned_queries "$QUERIES"

wave=0
works=0
for mark in $MARKS; do
  perf_step "growing to $mark events"
  while true; do
    head="$(perf_journal_head)"
    [ "$head" -ge "$mark" ] && break
    wave=$((wave + 1))
    wdir="$PERF_SCRATCH/s5/calls/w$wave"
    perf_mark w0
    perf_burst "$WAVE" "$WAVE" "$wdir" "s5w$wave"
    perf_mark w1
    works=$((works + WAVE))
    rm -rf "$wdir"
    head="$(perf_journal_head)"
    printf '%s,%s,%s,%s,%s,%s\n' "$wave" "$works" "$head" "$((w1 - w0))" \
      "$(perf_proc_rss "$PERF_DAEMON_PID")" "$(perf_bytes "$DD")" >> "$GROWTH_CSV"
    if [ $((wave % 10)) -eq 0 ]; then
      printf '   wave %4d  works %5d  head %6s  rss %8s kB\n' \
        "$wave" "$works" "$head" "$(perf_proc_rss "$PERF_DAEMON_PID")"
    fi
  done

  perf_step "mark $mark — cold start"
  perf_daemon_stop
  events="$(perf_journal_events "$DD")"
  segments="$(find "$DD/journal" -name '*.ndjson' 2>/dev/null | wc -l | tr -d ' ')"
  # Measured here, with nothing else running, so the subtraction below is
  # against this machine at this moment rather than a sample from minutes ago.
  overhead_ms="$(perf_empty_start_ms "$PERF_SCRATCH/s5" 2)"
  perf_daemon_start "$DD" "" "$READY_TIMEOUT"
  restart_ms="$PERF_DAEMON_READY_MS"
  rss_start="$(perf_proc_rss "$PERF_DAEMON_PID")"
  hwm_start="$(perf_proc_hwm "$PERF_DAEMON_PID")"
  # Two rates, because the §21 budget is about replay and a process start is
  # not replay: the raw rate over the whole cold start, and the net rate after
  # subtracting the empty-data-dir start measured at the top of this scenario
  # (exec + bind + descriptor, the fixed overhead every start pays).
  rate="$(awk -v e="$events" -v ms="$restart_ms" 'BEGIN{ if (ms > 0) printf "%.0f", e / (ms / 1000); else printf "0" }')"
  net_ms="$(awk -v a="$restart_ms" -v b="$overhead_ms" 'BEGIN{ printf "%.2f", a - b }')"
  # A net that is not comfortably positive means the empty-start overhead and
  # the loaded start are within each other's noise: report that instead of a
  # rate computed by dividing by nearly zero.
  net_rate="$(awk -v e="$events" -v ms="$net_ms" 'BEGIN{ if (ms >= 10) printf "%.0f", e / (ms / 1000); else printf "n/a (net %.2f ms is within start-overhead noise)", ms }')"
  duckdb="$(stat -c %s "$DD/projections/sergeant.duckdb" 2>/dev/null || echo 0)"
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$mark" "$events" "$segments" "$restart_ms" "$rate" "$rss_start" "$hwm_start" \
    "$duckdb" "$(perf_bytes "$DD/journal")" "$(perf_bytes "$DD")" >> "$MARK_CSV"

  perf_kv "m${mark}_events_on_disk" "$events"
  perf_kv "m${mark}_segments" "$segments"
  perf_kv "m${mark}_cold_start_ms" "$restart_ms"
  perf_kv "m${mark}_rebuild_events_per_s" "$rate"
  perf_kv "m${mark}_empty_start_overhead_ms" "$overhead_ms"
  perf_kv "m${mark}_cold_start_net_ms" "$net_ms"
  perf_kv "m${mark}_rebuild_events_per_s_net" "$net_rate"
  perf_kv "m${mark}_rss_after_start_kb" "$rss_start"
  perf_kv "m${mark}_hwm_after_start_kb" "$hwm_start"
  perf_kv "m${mark}_duckdb_bytes" "$duckdb"
  perf_kv "m${mark}_journal_bytes" "$(perf_bytes "$DD/journal")"
  perf_kv "m${mark}_data_dir_bytes" "$(perf_bytes "$DD")"
  perf_say "cold start ${restart_ms} ms for $events events — ${rate} events/s raw; net of a ${overhead_ms} ms empty start: ${net_rate} events/s · duckdb $duckdb B"

  for q in $QUERIES; do
    # First call after a restart pays any lazy catch-up; both are recorded.
    for attempt in cold warm; do
      read -r w code bytes total <<< "$(perf_timed_get "/v1/analytics/$q" "$PERF_SCRATCH/s5/a.json")"
      rows="$(jq '.rows | length' "$PERF_SCRATCH/s5/a.json" 2>/dev/null || echo 0)"
      printf '%s,%s,%s,%s,%s,%s\n' "$mark" "$q-$attempt" "$w" "$code" "$bytes" "$rows" >> "$ANALYTICS_CSV"
      perf_kv "m${mark}_analytics_${q}_${attempt}_ms" "$(perf_ms "$w")"
      [ "$attempt" = warm ] && perf_kv "m${mark}_analytics_${q}_rows" "$rows"
    done
  done
  # `sgt analytics <q>` through the CLI once, so the process-per-call cost of
  # the shipped client is on the record next to the endpoint's.
  perf_mark ca0
  "$SGT_BIN" --data-dir "$DD" --json analytics "${QUERIES%% *}" > /dev/null 2>&1 || true
  perf_mark ca1
  perf_kv "m${mark}_analytics_cli_ms" "$(perf_ms $((ca1 - ca0)))"
done

perf_kv works_submitted "$works"
perf_kv final_journal_head "$(perf_journal_head)"
perf_kv fleet_total "$(perf_api GET /v1/work | jq '.works | length')"
perf_kv fleet_states "$(perf_state_counts | tr '\n' ' ')"
perf_kv bytes_per_event "$(awk -v b="$(perf_bytes "$DD/journal")" -v e="$(perf_journal_events "$DD")" 'BEGIN{printf "%.1f", e? b/e : 0}')"

perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` (see its doc in
# common.sh) — `duckdb_bytes` only reflects reality post-checkpoint (the
# S5 50k mark cited in the issue: 8.1 MiB right after start vs 31.8 MiB
# after clean stop).
perf_disk_kv final_disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s5 "$REPO"
perf_summary
