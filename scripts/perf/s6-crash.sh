#!/usr/bin/env bash
# S6 — crash + recovery under load.
#
# Three times: 10 submits in flight, `kill -9` the daemon mid-burst, restart,
# and check that (a) the journal tail is recovered, (b) every work lands in a
# legal state — fail-closed `blocked` is acceptable, silent loss or duplicate
# execution is not, (c) the killed daemon left no orphan processes, and (d) a
# second restart is idempotent (same fleet, byte for byte).
#
#   scripts/perf/s6-crash.sh <outdir>
#
# Knobs: PERF_S6_CYCLES (3), PERF_S6_INFLIGHT (10), PERF_S6_KILL_DELAY (auto),
#        PERF_S6_KILL_FACTOR (1.0 — the kill lands one single-submit latency
#        into the burst, so some calls have answered and some have not).
#
# "Silent loss" is decided against the client's own evidence: a submit that
# returned 201 named a work id, and that work must exist after the restart.
# Submits that got no answer are counted separately — they are the crash
# window, and either outcome for them is legal.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: s6-crash.sh <outdir>}"
perf_init "$OUT" s6-crash

CYCLES="${PERF_S6_CYCLES:-3}"
INFLIGHT="${PERF_S6_INFLIGHT:-10}"
KILL_DELAY="${PERF_S6_KILL_DELAY:-auto}"

DD="$PERF_SCRATCH/s6/data"
REPO="$PERF_SCRATCH/s6/repo"
rm -rf "$PERF_SCRATCH/s6"
mkdir -p "$DD"
perf_seed_repo "$REPO"

perf_kv cycles "$CYCLES"
perf_kv inflight "$INFLIGHT"

CYCLE_CSV="$PERF_OUT/s6-cycles.csv"
echo 'cycle,kill_delay_s,submitted,http_201,no_answer,events_before_kill,events_after_kill,restart1_ms,restart2_ms,works_after,lost,legal_states,illegal_states,duplicate_executions,orphans,idempotent' > "$CYCLE_CSV"

LEGAL='pending|running|needs_input|waiting|blocked|failed|completed|canceled'

perf_daemon_start "$DD"
# One warm submit to learn the latency the kill has to land inside of.
perf_mark p0
WARM="$(perf_submit "S6 warm-up")"
perf_mark p1
WARM_MS="$(perf_ms $((p1 - p0)))"
perf_kv warmup_submit_ms "$WARM_MS"
perf_kv warmup_work "${WARM:-none}"
if [ "$KILL_DELAY" = auto ]; then
  # Half a single submit's latency: late enough that work is genuinely in
  # flight, early enough that the burst has not already finished.
  KILL_DELAY="$(awk -v ms="$WARM_MS" -v f="${PERF_S6_KILL_FACTOR:-1.0}" \
    'BEGIN{ d = ms * f / 1000; if (d < 0.05) d = 0.05; if (d > 2) d = 2; printf "%.3f", d }')"
fi
perf_kv kill_delay_s "$KILL_DELAY"

for ((c = 1; c <= CYCLES; c++)); do
  perf_step "cycle $c — $INFLIGHT in flight, kill -9 after ${KILL_DELAY}s"
  [ -n "${PERF_DAEMON_PID:-}" ] || perf_daemon_start "$DD"
  pid="$PERF_DAEMON_PID"
  events_before="$(perf_journal_events "$DD")"

  calls="$PERF_SCRATCH/s6/calls-$c"
  rm -rf "$calls"; mkdir -p "$calls"
  perf_burst "$INFLIGHT" "$INFLIGHT" "$calls" "s6c$c" &
  burst_job=$!
  sleep "$KILL_DELAY"
  perf_daemon_kill9 "$pid"
  wait "$burst_job" 2>/dev/null || true

  csv="$PERF_OUT/s6-calls-$c.csv"
  perf_collect_calls "$calls" "$csv"
  submitted="$(tail -n +2 "$csv" | wc -l | tr -d ' ')"
  ok201="$(tail -n +2 "$csv" | awk -F, '$4 == 201' | wc -l | tr -d ' ')"
  noanswer="$(tail -n +2 "$csv" | awk -F, '$4 != 201' | wc -l | tr -d ' ')"
  events_after="$(perf_journal_events "$DD")"

  # Any process the killed daemon left behind (the fake backend runs
  # in-process, so the honest expectation is zero; a real adapter would not
  # be, which is why this is measured rather than assumed).
  orphans="$(perf_sgt_process_count)"

  perf_step "cycle $c — restart 1"
  perf_daemon_start "$DD"
  r1="$PERF_DAEMON_READY_MS"
  fleet1="$PERF_OUT/s6-fleet-$c-a.json"
  perf_api GET /v1/work | jq -S '.' > "$fleet1"
  works_after="$(jq '.works | length' "$fleet1")"

  # Silent loss: a 201 named a work id; that work must be there.
  lost=0
  while read -r id; do
    [ -n "$id" ] || continue
    jq -e --arg id "$id" '.works[] | select(.id == $id)' "$fleet1" > /dev/null 2>&1 || lost=$((lost + 1))
  done < <(tail -n +2 "$csv" | awk -F, '$4 == 201 {print $5}')

  legal="$(jq -r '[.works[].state] | map(select(test("^('"$LEGAL"')$"))) | length' "$fleet1")"
  illegal="$(jq -r '[.works[].state] | map(select(test("^('"$LEGAL"')$") | not)) | length' "$fleet1")"
  jq -r '[.works[].state] | group_by(.) | map({(.[0]): length}) | add' "$fleet1" > "$PERF_OUT/s6-states-$c.json"

  # Duplicate execution: more execution.started events for a work than the
  # workflow has stages (2), counted straight off the journal.
  dup="$(python3 - "$DD" <<'PY'
import glob, json, os, sys, collections
counts = collections.Counter()
for path in sorted(glob.glob(os.path.join(sys.argv[1], "journal", "*.ndjson"))):
    for line in open(path, errors="replace"):
        line = line.strip()
        if not line:
            continue
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("kind") == "execution.started" and e.get("work_id"):
            counts[e["work_id"]] += 1
print(sum(1 for _, n in counts.items() if n > 2))
PY
)"

  perf_step "cycle $c — restart 2 (idempotence)"
  perf_daemon_stop
  perf_daemon_start "$DD"
  r2="$PERF_DAEMON_READY_MS"
  fleet2="$PERF_OUT/s6-fleet-$c-b.json"
  perf_api GET /v1/work | jq -S '.' > "$fleet2"
  if diff -q "$fleet1" "$fleet2" > /dev/null 2>&1; then idem=yes; else idem=no; diff -u "$fleet1" "$fleet2" > "$PERF_OUT/s6-fleet-diff-$c.txt" || true; fi

  printf '%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s\n' \
    "$c" "$KILL_DELAY" "$submitted" "$ok201" "$noanswer" "$events_before" "$events_after" \
    "$r1" "$r2" "$works_after" "$lost" "$legal" "$illegal" "$dup" "$orphans" "$idem" >> "$CYCLE_CSV"

  perf_kv "c${c}_http_201" "$ok201/$submitted"
  perf_kv "c${c}_no_answer" "$noanswer"
  perf_kv "c${c}_events_added" "$((events_after - events_before))"
  perf_kv "c${c}_restart1_ms" "$r1"
  perf_kv "c${c}_restart2_ms" "$r2"
  perf_kv "c${c}_works_after" "$works_after"
  perf_kv "c${c}_silently_lost" "$lost"
  perf_kv "c${c}_illegal_states" "$illegal"
  perf_kv "c${c}_duplicate_executions" "$dup"
  perf_kv "c${c}_orphan_processes" "$orphans"
  perf_kv "c${c}_second_restart_idempotent" "$idem"
  perf_kv "c${c}_states" "$(tr -d '\n ' < "$PERF_OUT/s6-states-$c.json")"
  perf_say "cycle $c: 201=$ok201/$submitted lost=$lost illegal=$illegal dup=$dup orphans=$orphans idempotent=$idem restart ${r1}/${r2} ms"
done

# The journal must still be readable end to end by the shipped validator.
perf_step "journal validation after $CYCLES crash cycles"
perf_daemon_stop
DOCTOR="$PERF_OUT/s6-doctor.json"
"$SGT_BIN" --data-dir "$DD" --json doctor > "$DOCTOR" 2>/dev/null || true
perf_kv doctor_healthy "$(jq -r '.healthy // "?"' "$DOCTOR" 2>/dev/null || echo '?')"
perf_kv doctor_journal "$(jq -r '.checks[] | select(.name | test("journal")) | .status' "$DOCTOR" 2>/dev/null | tr '\n' ' ')"
perf_kv doctor_failing "$(jq -r '[.checks[] | select(.status != "ok") | .name] | join(",")' "$DOCTOR" 2>/dev/null || echo '?')"
perf_kv journal_events_final "$(perf_journal_events "$DD")"
perf_kv journal_unparseable_lines "$(python3 - "$DD" <<'PY'
import glob, json, os, sys
bad = 0
for path in sorted(glob.glob(os.path.join(sys.argv[1], "journal", "*.ndjson"))):
    for line in open(path, errors="replace"):
        line = line.strip()
        if not line:
            continue
        try:
            json.loads(line)
        except Exception:
            bad += 1
print(bad)
PY
)"
perf_disk_kv disk "$DD"
# A crash legitimately leaves works non-terminal, and a non-terminal work keeps
# its surface. Counted here so the sweep's worktree count below is readable as
# "expected residue of N blocked works" rather than as a leak.
LAST_FLEET="$PERF_OUT/s6-fleet-${CYCLES}-b.json"
if [ -f "$LAST_FLEET" ]; then
  perf_kv nonterminal_works "$(jq -r '[.works[] | select(.state | test("^(pending|running|needs_input|waiting|blocked)$"))] | length' "$LAST_FLEET")"
  perf_kv nonterminal_states "$(jq -r '[.works[].state] | group_by(.) | map({(.[0]): length}) | add | tostring' "$LAST_FLEET")"
fi
perf_kv daemon_force_killed "$PERF_FORCE_KILLED"
perf_hygiene s6 "$REPO"
perf_summary
