#!/usr/bin/env bash
# S2 — sustained churn + leak watch.
#
# 200 works in waves of 10 against one daemon. VmRSS / VmHWM / fd count / CPU
# are sampled once per wave (and continuously in the background), then the run
# settles and is sampled again: RSS must come back near the pre-run mark. A
# monotonic climb across the waves is the finding this scenario exists to see.
# Journal, blob and surface bytes are accounted before and after, per work.
#
#   scripts/perf/s2-churn.sh <outdir>
#
# Knobs: PERF_S2_TOTAL (200), PERF_S2_WAVE (10), PERF_S2_SETTLE (120 s),
#        PERF_S2_SAMPLE_INTERVAL (1 s).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s2-churn.sh <outdir>}"
perf_init "$OUT" s2-churn

TOTAL="${PERF_S2_TOTAL:-200}"
WAVE="${PERF_S2_WAVE:-10}"
SETTLE="${PERF_S2_SETTLE:-120}"
INTERVAL="${PERF_S2_SAMPLE_INTERVAL:-1}"

DD="$PERF_SCRATCH/s2/data"
ESTATE="$PERF_SCRATCH/s2/estate"
rm -rf "$PERF_SCRATCH/s2"
mkdir -p "$DD"
perf_estate_scaffold "$ESTATE"
REPO="$PERF_REPO"
perf_daemon_start "$DD"
perf_say "daemon pid $PERF_DAEMON_PID · $TOTAL works in waves of $WAVE · settle ${SETTLE}s"

perf_kv total_works "$TOTAL"
perf_kv wave_size "$WAVE"
perf_kv settle_s "$SETTLE"

# The pre-run mark every later reading is judged against. Sampled after the
# daemon has answered a few requests so it is a warm idle, not a cold one.
perf_api GET /v1/work > /dev/null
sleep 2
RSS0="$(perf_proc_rss "$PERF_DAEMON_PID")"
HWM0="$(perf_proc_hwm "$PERF_DAEMON_PID")"
FDS0="$(perf_proc_fds "$PERF_DAEMON_PID")"
CPU0="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
perf_kv pre_rss_kb "$RSS0"
perf_kv pre_hwm_kb "$HWM0"
perf_kv pre_fds "$FDS0"
perf_disk_kv pre_disk "$DD"
HEAD0="$(perf_journal_head)"

WAVES_CSV="$PERF_OUT/s2-waves.csv"
echo 'wave,works_done,wall_ns,rss_kb,hwm_kb,fds,threads,cpu_sec,journal_head,disk_total_bytes' > "$WAVES_CSV"
CONT="$PERF_OUT/s2-continuous.csv"
perf_sampler_start "$PERF_DAEMON_PID" "$CONT" "$INTERVAL" churn

CALLS="$PERF_SCRATCH/s2/calls"
rm -rf "$CALLS"; mkdir -p "$CALLS"

waves=$(( (TOTAL + WAVE - 1) / WAVE ))
done_n=0
perf_mark run0
for ((w = 1; w <= waves; w++)); do
  n=$WAVE
  [ $((done_n + n)) -gt "$TOTAL" ] && n=$((TOTAL - done_n))
  [ "$n" -gt 0 ] || break
  wdir="$CALLS/w$w"
  perf_mark wt0
  perf_burst "$n" "$n" "$wdir" "s2w$w"
  perf_mark wt1
  done_n=$((done_n + n))
  rss="$(perf_proc_rss "$PERF_DAEMON_PID")"
  hwm="$(perf_proc_hwm "$PERF_DAEMON_PID")"
  fds="$(perf_proc_fds "$PERF_DAEMON_PID")"
  thr="$(awk '/Threads:/{print $2}' "/proc/$PERF_DAEMON_PID/status" 2>/dev/null || echo 0)"
  cpu="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
  head="$(perf_journal_head)"
  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$w" "$done_n" "$((wt1 - wt0))" "$rss" "$hwm" "$fds" "$thr" "$cpu" "$head" "$(perf_bytes "$DD")" >> "$WAVES_CSV"
  printf '   wave %2d/%d  %3d works  %6s s  rss %8s kB  fds %3s  cpu %8s s\n' \
    "$w" "$waves" "$done_n" "$(perf_s $((wt1 - wt0)))" "$rss" "$fds" "$cpu"
done
perf_mark run1

CALLS_CSV="$PERF_OUT/s2-calls.csv"
echo 't_start_ns,wall_ns,curl_total_s,http_code,work_id,state' > "$CALLS_CSV"
cat "$CALLS"/w*/*.csv 2>/dev/null >> "$CALLS_CSV" || true

perf_step "settling ${SETTLE}s"
sleep "$SETTLE"
perf_sampler_stop

RSS1="$(perf_proc_rss "$PERF_DAEMON_PID")"
HWM1="$(perf_proc_hwm "$PERF_DAEMON_PID")"
FDS1="$(perf_proc_fds "$PERF_DAEMON_PID")"
CPU1="$(perf_proc_cpu_sec "$PERF_DAEMON_PID")"
HEAD1="$(perf_journal_head)"

read -r peak_rss peak_hwm max_fds max_thr last_cpu _ _ nsamples <<< "$(perf_sample_stats "$CONT")"

perf_kv run_wall_s "$(perf_s $((run1 - run0)))"
perf_kv works_submitted "$done_n"
perf_kv http_201 "$(tail -n +2 "$CALLS_CSV" | awk -F, '$4 == 201' | wc -l | tr -d ' ')"
perf_kv completed_in_response "$(tail -n +2 "$CALLS_CSV" | awk -F, '$6 == "completed"' | wc -l | tr -d ' ')"
perf_kv fleet_states "$(perf_state_counts | tr '\n' ' ')"
perf_latency_kv lat "$CALLS_CSV" 2
perf_kv peak_rss_kb "$peak_rss"
perf_kv peak_hwm_kb "$peak_hwm"
perf_kv post_settle_rss_kb "$RSS1"
perf_kv post_settle_hwm_kb "$HWM1"
perf_kv rss_delta_kb "$((RSS1 - RSS0))"
perf_kv rss_growth_pct "$(awk -v a="$RSS0" -v b="$RSS1" 'BEGIN{printf "%.1f", a? (b-a)*100.0/a : 0}')"
perf_kv rss_kb_per_work "$(awk -v d="$((RSS1 - RSS0))" -v n="$done_n" 'BEGIN{printf "%.2f", n? d/n : 0}')"
perf_kv fds_pre "$FDS0"
perf_kv fds_peak "$max_fds"
perf_kv fds_post "$FDS1"
perf_kv threads_peak "$max_thr"
perf_kv cpu_total_s "$(awk -v a="$CPU1" -v b="$CPU0" 'BEGIN{printf "%.3f", a-b}')"
perf_kv cpu_ms_per_work "$(awk -v a="$CPU1" -v b="$CPU0" -v n="$done_n" 'BEGIN{printf "%.1f", n? (a-b)*1000/n : 0}')"
perf_kv samples "$nsamples"
perf_kv journal_events "$((HEAD1 - HEAD0))"
perf_kv events_per_work "$(awk -v e="$((HEAD1 - HEAD0))" -v n="$done_n" 'BEGIN{printf "%.1f", n? e/n : 0}')"

python3 - "$WAVES_CSV" "$PERF_OUT/s2-trend.json" <<'PY'
# The leak question is "does RSS trend up across waves", so fit a line to it:
# a slope near zero with a bounded residual is a plateau, a positive slope that
# survives the settle is a climb. Reported, not adjudicated.
import csv, json, sys
rows = list(csv.DictReader(open(sys.argv[1])))
xs = [int(r["works_done"]) for r in rows]
ys = [int(r["rss_kb"]) for r in rows]
n = len(xs)
out = {"waves": n}
if n >= 2:
    mx, my = sum(xs) / n, sum(ys) / n
    den = sum((x - mx) ** 2 for x in xs)
    slope = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / den if den else 0.0
    out.update({
        "rss_kb_per_work_slope": round(slope, 4),
        "rss_first_wave_kb": ys[0], "rss_last_wave_kb": ys[-1],
        "rss_min_kb": min(ys), "rss_max_kb": max(ys),
        "monotonic_nondecreasing": all(b >= a for a, b in zip(ys, ys[1:])),
    })
json.dump(out, open(sys.argv[2], "w"), indent=2)
print("   rss trend:", json.dumps(out))
PY
perf_kv rss_trend_json "$PERF_OUT/s2-trend.json"

perf_daemon_stop
# Issue #13: moved from before `perf_daemon_stop` — DuckDB only checkpoints
# to disk at clean shutdown, so `post_disk_duckdb_bytes` read here used to be
# whatever was on disk from the last checkpoint (measured: 12,288 B) rather
# than the projection's real post-run size (536,576 B). `perf_disk_kv`'s own
# doc in common.sh has the full measurement.
perf_disk_kv post_disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s2 "$REPO"
perf_summary
