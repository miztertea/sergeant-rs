#!/usr/bin/env bash
# P1-PERF — the whole matrix, in order, one command.
#
#   scripts/perf/run-all.sh <outdir>
#
# Records the environment, takes the idle-daemon baseline, then runs S1..S7
# strictly sequentially — never overlapping another scenario's measurement
# window, each into its own subdirectory of <outdir>, each against its own
# fresh data dir. A scenario that fails is recorded as failed and the run
# continues: "a scenario that cannot hit its target records where it stopped
# and why — that is data, not failure."
#
# Every scenario is also runnable alone: scripts/perf/s3-deep.sh <outdir>.
#
# Knobs: PERF_ONLY ("s1 s4" to run a subset), PERF_SKIP_IDLE=1.
# Scenario-level knobs are documented in each scenario's header and pass
# through this script unchanged.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

OUT="${1:?usage: run-all.sh <outdir>}"
mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
perf_require_binary

LOG="$OUT/run-all.log"
STATUS="$OUT/run-all-status.tsv"
: > "$STATUS"

{
  echo "P1-PERF full run"
  echo "started: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "outdir:  $OUT"
  echo
  PERF_OUT="$OUT" perf_environment "$OUT/environment.txt"
} | tee "$LOG"

ONLY="${PERF_ONLY:-}"
wanted() {
  [ -z "$ONLY" ] && return 0
  case " $ONLY " in *" $1 "*) return 0 ;; *) return 1 ;; esac
}

run_stage() { # run_stage <name> <script> <subdir>
  local name="$1" script="$2" sub="$3"
  wanted "$name" || { printf '%s\tskipped\t0\n' "$name" >> "$STATUS"; return 0; }
  printf '\n\033[1m######## %s ########\033[0m\n' "$name" | tee -a "$LOG"
  local t0 t1 rc
  perf_mark t0
  "$script" "$OUT/$sub" 2>&1 | tee -a "$LOG"
  rc="${PIPESTATUS[0]}"
  perf_mark t1
  printf '%s\t%s\t%s\n' "$name" "$([ "$rc" -eq 0 ] && echo ok || echo "failed(rc=$rc)")" "$(perf_s $((t1 - t0)))" >> "$STATUS"
  # Between scenarios: nothing of this one may still be running.
  sleep 3
}

if [ "${PERF_SKIP_IDLE:-0}" != 1 ]; then
  run_stage idle "$HERE/idle-baseline.sh" idle
fi
run_stage s1 "$HERE/s1-burst.sh"   s1
run_stage s2 "$HERE/s2-churn.sh"   s2
run_stage s3 "$HERE/s3-deep.sh"    s3
run_stage s4 "$HERE/s4-sse.sh"     s4
run_stage s5 "$HERE/s5-journal.sh" s5
run_stage s6 "$HERE/s6-crash.sh"   s6
run_stage s7 "$HERE/s7-clients.sh" s7

printf '\n\033[1m######## run complete ########\033[0m\n' | tee -a "$LOG"
{
  printf '%-8s %-16s %s\n' scenario outcome wall_s
  while IFS=$'\t' read -r name outcome wall; do
    printf '%-8s %-16s %s\n' "$name" "$outcome" "$wall"
  done < "$STATUS"
} | tee -a "$LOG"

# Final sweep across everything the run created.
PERF_OUT="$OUT" PERF_SCENARIO=run-all PERF_SUMMARY_TSV="$OUT/run-all-summary.tsv" PERF_REPOS=()
: > "$OUT/run-all-summary.tsv"
# Each scenario's scratch estate mounts its one repository at
# <scenario>/scratch/<scenario>/estate/repos/<name> (perf_estate_scaffold);
# idle-baseline's own scaffold has no work driven through it but still
# mounts one, so it is picked up here the same way.
for repo in "$OUT"/*/scratch/*/estate/repos/*; do [ -d "$repo/.git" ] && PERF_REPOS+=("$repo"); done
perf_hygiene final | tee -a "$LOG"

python3 - "$OUT" <<'PY' | tee -a "$LOG"
# One index of every distilled summary the run produced, so the baseline doc
# has a single place to read the matrix out of.
import glob, json, os, sys
out = sys.argv[1]
index = {}
for path in sorted(glob.glob(os.path.join(out, "*", "*-summary.json"))):
    try:
        index[os.path.relpath(path, out)] = json.load(open(path))
    except Exception as e:
        index[os.path.relpath(path, out)] = {"error": str(e)}
target = os.path.join(out, "all-summaries.json")
json.dump(index, open(target, "w"), indent=2)
print(f"\nindex of {len(index)} scenario summaries: {target}")
PY

echo "log: $LOG"
