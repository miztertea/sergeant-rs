#!/usr/bin/env bash
# S3 — deep work + graph load.
#
# One work is driven through needs_input→respond cycles until its journal trail
# is at least 1,000 events, then the read paths are loaded at that depth:
# 100 sequential `GET /v1/graph/work/{id}`, 5 rounds of 20 concurrent, plus
# `work show` (endpoint and CLI) and the events-tail endpoints.
#
#   scripts/perf/s3-deep.sh <outdir>
#
# Knobs: PERF_S3_TARGET_EVENTS (1000), PERF_S3_MAX_CYCLES (500),
#        PERF_S3_SEQ_READS (100), PERF_S3_CONC (20), PERF_S3_ROUNDS (5),
#        PERF_S3_REPS (1) — repeat the read-burst block (sequential + concurrent
#        graph reads, work show) this many times before the daemon stops, so a
#        held-open daemon's read paths can be loaded well past a single pass
#        without a second invocation of this script. At the default REPS=1
#        every output is byte-identical to before this knob existed; above 1,
#        the per-request CSVs and the latency/count `perf_kv` keys they feed
#        become cumulative across reps (more samples, same percentile math),
#        and a new s3-reps.csv (rep, rss_kb) tracks RSS at each rep boundary
#        — repeated reads that grow RSS are exactly what this knob exists to
#        surface. The orchestrator runs the 12-rep measurement pass; this
#        worktree only adds the knob.
#
# The fake backend's script is a single queue shared by the whole daemon: one
# step is consumed per execution START and one per input SEND. With exactly one
# work in flight that is deterministic — a script of N `needs_input` steps
# yields N respond cycles before the queue runs dry and the default `complete`
# step ends the stage. That is why this scenario runs one work alone.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s3-deep.sh <outdir>}"
perf_init "$OUT" s3-deep

TARGET="${PERF_S3_TARGET_EVENTS:-1000}"
MAXC="${PERF_S3_MAX_CYCLES:-500}"
SEQ_READS="${PERF_S3_SEQ_READS:-100}"
CONC="${PERF_S3_CONC:-20}"
ROUNDS="${PERF_S3_ROUNDS:-5}"
REPS="${PERF_S3_REPS:-1}"

DD="$PERF_SCRATCH/s3/data"
REPO="$PERF_SCRATCH/s3/repo"
rm -rf "$PERF_SCRATCH/s3"
mkdir -p "$DD"
perf_seed_repo "$REPO"

# A script of MAXC needs_input steps: the loop below stops responding as soon
# as the depth target is met, so the tail of the script is simply unused.
SCRIPT=""
for ((i = 1; i <= MAXC; i++)); do SCRIPT+="needs_input:deep question $i;"; done
perf_daemon_start "$DD" "$SCRIPT"
perf_say "daemon pid $PERF_DAEMON_PID · script of $MAXC needs_input steps · target ${TARGET} events"

perf_kv target_events "$TARGET"
perf_kv max_cycles "$MAXC"

perf_step "driving one work deep"
WORK="$(perf_submit "S3 deep work: drive this to $TARGET journal events")"
[ -n "$WORK" ] || perf_die "submit returned no work id"
perf_kv work_id "$WORK"
STATE="$(perf_work_state "$WORK")"
perf_kv state_after_submit "$STATE"

events_for_work() { perf_api GET "/v1/events?work_id=$WORK" | jq '.events | length'; }

CYCLES_CSV="$PERF_OUT/s3-cycles.csv"
echo 'cycle,wall_ns,state,events' > "$CYCLES_CSV"
cycles=0
events="$(events_for_work)"
events_after_submit="$events"
while [ "$events" -lt "$TARGET" ] && [ "$cycles" -lt "$MAXC" ]; do
  [ "$STATE" = "needs_input" ] || break
  perf_mark c0
  STATE="$(perf_respond "$WORK" "answer $((cycles + 1))")"
  perf_mark c1
  cycles=$((cycles + 1))
  if [ $((cycles % 25)) -eq 0 ] || [ "$STATE" != "needs_input" ]; then
    events="$(events_for_work)"
    printf '   cycle %3d  state %-12s events %5s\n' "$cycles" "$STATE" "$events"
  fi
  printf '%s,%s,%s,%s\n' "$cycles" "$((c1 - c0))" "$STATE" "$events" >> "$CYCLES_CSV"
done
events="$(events_for_work)"

perf_kv events_after_submit "$events_after_submit"
perf_kv respond_cycles "$cycles"
perf_kv final_state "$STATE"
perf_kv work_events "$events"
perf_kv events_per_cycle "$(awk -v e="$events" -v s="$events_after_submit" -v c="$cycles" 'BEGIN{printf "%.2f", c? (e-s)/c : 0}')"
perf_kv depth_target_met "$([ "$events" -ge "$TARGET" ] && echo yes || echo "no (stopped at $events)")"
perf_latency_kv respond "$CYCLES_CSV" 2
perf_kv journal_head "$(perf_journal_head)"

RSS_BEFORE="$(perf_proc_rss "$PERF_DAEMON_PID")"
FDS_BEFORE="$(perf_proc_fds "$PERF_DAEMON_PID")"
SAMPLE="$PERF_OUT/s3-sample.csv"
perf_sampler_start "$PERF_DAEMON_PID" "$SAMPLE" 0.2 reads

# #8: the read-burst block below (sequential + concurrent graph reads, work
# show) repeats REPS times. Every CSV it writes is initialized once here, kept
# open across reps (each rep appends), so `perf_latency_kv`/the `_http` counts
# it feeds are cumulative over every rep by the time the loop ends — more
# samples behind the same percentile math, not a second, separate summary. At
# the default REPS=1 the loop body runs exactly once and every artifact below
# is byte-identical to before this knob existed.
SEQ_CSV="$PERF_OUT/s3-graph-seq.csv"
echo 't_start_ns,wall_ns,curl_total_s,http_code,bytes' > "$SEQ_CSV"
CONC_DIR="$PERF_SCRATCH/s3/conc"
CONC_CSV="$PERF_OUT/s3-graph-conc.csv"
echo 't_start_ns,wall_ns,curl_total_s,http_code,bytes' > "$CONC_CSV"
SHOW_CSV="$PERF_OUT/s3-show.csv"
echo 't_start_ns,wall_ns,curl_total_s,http_code,bytes' > "$SHOW_CSV"
REPS_CSV="$PERF_OUT/s3-reps.csv"
echo 'rep,rss_kb' > "$REPS_CSV"

for ((rep = 1; rep <= REPS; rep++)); do
  [ "$REPS" -gt 1 ] && perf_step "read-burst rep $rep/$REPS"

  perf_step "$SEQ_READS sequential graph reads"
  for ((i = 1; i <= SEQ_READS; i++)); do
    perf_mark g0
    line="$(curl -sS -m 300 -o "$PERF_SCRATCH/s3/graph.json" \
      -w '%{http_code} %{size_download} %{time_total}' \
      -H "Authorization: Bearer $PERF_TOKEN" "$PERF_ENDPOINT/v1/graph/work/$WORK" 2>/dev/null)"
    perf_mark g1
    read -r code bytes total <<< "$line"
    printf '%s,%s,%s,%s,%s\n' "$g0" "$((g1 - g0))" "$total" "$code" "$bytes" >> "$SEQ_CSV"
  done
  perf_latency_kv graph_seq "$SEQ_CSV" 2
  perf_kv graph_seq_http "$(tail -n +2 "$SEQ_CSV" | awk -F, '$4 == 200' | wc -l | tr -d ' ')/$((SEQ_READS * rep))"
  perf_kv graph_bytes "$(tail -n +2 "$SEQ_CSV" | cut -d, -f5 | sort -n | tail -1)"
  perf_kv graph_nodes "$(jq '.nodes | length' "$PERF_SCRATCH/s3/graph.json" 2>/dev/null || echo 0)"
  perf_kv graph_edges "$(jq '.edges | length' "$PERF_SCRATCH/s3/graph.json" 2>/dev/null || echo 0)"

  perf_step "$ROUNDS rounds of $CONC concurrent graph reads"
  rm -rf "$CONC_DIR"; mkdir -p "$CONC_DIR"
  round_walls=()
  for ((r = 1; r <= ROUNDS; r++)); do
    rdir="$CONC_DIR/r$r"; mkdir -p "$rdir"
    perf_mark r0
    seq 1 "$CONC" | xargs -P "$CONC" -I@ \
      "$PERF_DIR/_get-one.sh" "$PERF_ENDPOINT" "$PERF_TOKEN" "/v1/graph/work/$WORK" "$rdir/@.csv"
    perf_mark r1
    round_walls+=("$((r1 - r0))")
    cat "$rdir"/*.csv >> "$CONC_CSV"
    perf_say "round $r: $(perf_s $((r1 - r0)))s wall for $CONC concurrent"
  done
  perf_latency_kv graph_conc "$CONC_CSV" 2
  perf_kv graph_conc_http "$(tail -n +2 "$CONC_CSV" | awk -F, '$4 == 200' | wc -l | tr -d ' ')/$((CONC * ROUNDS * rep))"
  perf_kv graph_conc_round_walls_s "$(for w in "${round_walls[@]}"; do printf '%s ' "$(perf_s "$w")"; done)"

  perf_step "work show at depth"
  for ((i = 1; i <= 20; i++)); do
    read -r w code bytes total <<< "$(perf_timed_get "/v1/work/$WORK" "$PERF_SCRATCH/s3/show.json")"
    printf '%s,%s,%s,%s,%s\n' "$(perf_now_ns)" "$w" "$total" "$code" "$bytes" >> "$SHOW_CSV"
  done
  perf_latency_kv work_show "$SHOW_CSV" 2
  perf_kv work_show_bytes "$(stat -c %s "$PERF_SCRATCH/s3/show.json" 2>/dev/null || echo 0)"

  printf '%s,%s\n' "$rep" "$(perf_proc_rss "$PERF_DAEMON_PID")" >> "$REPS_CSV"
done

perf_kv rss_first_rep_kb "$(sed -n '2p' "$REPS_CSV" | cut -d, -f2)"
perf_kv rss_last_rep_kb "$(tail -n 1 "$REPS_CSV" | cut -d, -f2)"

perf_step "the events tail at depth"
TAIL_CSV="$PERF_OUT/s3-events-tail.csv"
echo 't_start_ns,wall_ns,curl_total_s,http_code,bytes,variant' > "$TAIL_CSV"
for variant in "limit=50" "limit=200" "all"; do
  case "$variant" in
    all) path="/v1/events?work_id=$WORK" ;;
    *)   path="/v1/events?work_id=$WORK&$variant" ;;
  esac
  for ((i = 1; i <= 20; i++)); do
    read -r w code bytes total <<< "$(perf_timed_get "$path" "$PERF_SCRATCH/s3/tail.json")"
    printf '%s,%s,%s,%s,%s,%s\n' "$(perf_now_ns)" "$w" "$total" "$code" "$bytes" "$variant" >> "$TAIL_CSV"
  done
  stats="$(grep ",$variant\$" "$TAIL_CSV" | cut -d, -f2 | awk '{printf "%.3f\n", $1/1000000}' | perf_pctl)"
  read -r c mn p50 p95 p99 mx mean <<< "$stats"
  perf_kv "events_tail_${variant//=/}_p50_ms" "$p50"
  perf_kv "events_tail_${variant//=/}_p95_ms" "$p95"
  perf_kv "events_tail_${variant//=/}_max_ms" "$mx"
  perf_kv "events_tail_${variant//=/}_bytes" "$(grep ",$variant\$" "$TAIL_CSV" | cut -d, -f5 | sort -n | tail -1)"
done

# The full-history tail with no work filter, which is what a fresh client asks
# for: the cost of the unbounded scan at this depth.
read -r w code bytes total <<< "$(perf_timed_get "/v1/events" "$PERF_SCRATCH/s3/all.json")"
perf_kv events_all_ms "$(perf_ms "$w")"
perf_kv events_all_bytes "$bytes"
perf_kv events_all_http "$code"

perf_step "the same reads through the CLI (process-per-call cost)"
perf_mark cli0
"$SGT_BIN" --data-dir "$DD" --json work show "$WORK" > "$PERF_SCRATCH/s3/cli-show.json" 2>/dev/null || true
perf_mark cli1
perf_kv cli_work_show_ms "$(perf_ms $((cli1 - cli0)))"
perf_mark cli2
"$SGT_BIN" --data-dir "$DD" --json work show "$WORK" --graph > "$PERF_SCRATCH/s3/cli-graph.json" 2>/dev/null || true
perf_mark cli3
perf_kv cli_work_graph_ms "$(perf_ms $((cli3 - cli2)))"

perf_sampler_stop
read -r peak_rss peak_hwm max_fds max_thr last_cpu _ _ _ <<< "$(perf_sample_stats "$SAMPLE")"
perf_kv rss_before_reads_kb "$RSS_BEFORE"
perf_kv rss_peak_reads_kb "$peak_rss"
perf_kv rss_after_reads_kb "$(perf_proc_rss "$PERF_DAEMON_PID")"
perf_kv rss_delta_reads_kb "$(($(perf_proc_rss "$PERF_DAEMON_PID") - RSS_BEFORE))"
perf_kv hwm_kb "$peak_hwm"
perf_kv fds_before_reads "$FDS_BEFORE"
perf_kv fds_peak_reads "$max_fds"
perf_kv fds_after_reads "$(perf_proc_fds "$PERF_DAEMON_PID")"
perf_kv cpu_s "$last_cpu"

# The work is still parked in needs_input holding a live surface; cancel it so
# the sweep below is a real sweep and not a known-dirty one. Needs the daemon
# alive, so it stays before `perf_daemon_stop` even though `perf_disk_kv`
# below has moved after it.
[ "$STATE" = "needs_input" ] && perf_cancel "$WORK"
sleep 1
perf_kv state_at_teardown "$(perf_work_state "$WORK")"

perf_daemon_stop
# Issue #13: `perf_disk_kv` moved after `perf_daemon_stop` (see its doc in
# common.sh) — `duckdb_bytes` only reflects reality post-checkpoint.
perf_disk_kv disk "$DD"
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s3 "$REPO"
perf_summary
