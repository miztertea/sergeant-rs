#!/usr/bin/env bash
# Idle daemon baseline — recorded before any scenario (P1-PERF measurement rule
# 3): a freshly started daemon on an empty data dir, doing nothing, sampled for
# two minutes. RSS, fds and CPU over that window are the floor every other
# number in the baseline is read against.
#
#   scripts/perf/idle-baseline.sh <outdir>
#
# Knobs: PERF_IDLE_SECS (120), PERF_IDLE_INTERVAL (1).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: idle-baseline.sh <outdir>}"
perf_init "$OUT" idle-baseline

SECS="${PERF_IDLE_SECS:-120}"
INTERVAL="${PERF_IDLE_INTERVAL:-1}"

perf_environment "$PERF_OUT/environment.txt" > /dev/null

DD="$PERF_SCRATCH/idle/data"
ESTATE="$PERF_SCRATCH/idle/estate"
rm -rf "$PERF_SCRATCH/idle"
mkdir -p "$DD"
# `sgt daemon` refuses to start outside an exact estate root (§4.1) even when
# nothing is ever submitted, so this baseline scaffolds one too — idle
# behavior on a fresh data dir, not idle behavior with no estate at all.
perf_estate_scaffold "$ESTATE"
perf_daemon_start "$DD"
perf_say "daemon pid $PERF_DAEMON_PID · sampling ${SECS}s at ${INTERVAL}s"
perf_kv settle_s "$SECS"
perf_kv sample_interval_s "$INTERVAL"
perf_kv cold_start_empty_ms "$PERF_DAEMON_READY_MS"

CSV="$PERF_OUT/idle-sample.csv"
cpu0="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
perf_sampler_start "$PERF_DAEMON_PID" "$CSV" "$INTERVAL" idle
sleep "$SECS"
perf_sampler_stop
cpu1="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"

read -r peak_rss peak_hwm max_fds max_thr last_cpu first_rss last_rss nsamples <<< "$(perf_sample_stats "$CSV")"
perf_kv rss_first_kb "$first_rss"
perf_kv rss_last_kb "$last_rss"
perf_kv rss_peak_kb "$peak_rss"
perf_kv rss_drift_kb "$((last_rss - first_rss))"
perf_kv hwm_kb "$peak_hwm"
perf_kv fds "$max_fds"
perf_kv threads "$max_thr"
perf_kv cpu_total_s "$(awk -v a="$cpu0" -v b="$cpu1" 'BEGIN{printf "%.3f", b-a}')"
perf_kv cpu_pct "$(awk -v a="$cpu0" -v b="$cpu1" -v t="$SECS" 'BEGIN{printf "%.3f", (b-a)*100/t}')"
perf_kv samples "$nsamples"
perf_kv journal_events "$(perf_journal_events "$DD")"

perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` — its
# `duckdb_bytes` reads DuckDB's on-disk checkpoint, which only happens at
# clean shutdown (see the doc on `perf_disk_kv` in common.sh).
perf_disk_kv disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene idle
perf_summary
