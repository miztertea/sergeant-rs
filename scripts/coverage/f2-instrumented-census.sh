#!/usr/bin/env bash
# F2 — the instrumented arm of the flake census: N full passes of C1–C3.
#
#   scripts/coverage/f2-instrumented-census.sh [N]   # N defaults to 3
#
# The question this arm answers is not "what is the coverage" — C4 answered
# that — but whether instrumentation's slowdown (~1.5–3×) brings back the
# M3-era parallel-flake class. So it runs the same pipeline the baseline was
# collected with, three times, in the same default parallel harness, and
# records what failed by name and how long each pass took. The wall-time
# ratio against F1 is the other output: it is the number that tells the next
# program what a coverage lane costs.
#
# RUN THIS AFTER C4, NEVER BEFORE. Each pass starts by removing every profraw
# (`cargo llvm-cov clean --profraw-only`, R-S0-6 — measured: it removes the
# .profraw files and leaves the build tree and the existing .profdata alone,
# so a pass costs no rebuild). Census passes must not pool into the baseline's
# profdata, and the baseline must already be exported before the first clean.
#
# Knobs: COV_CENSUS_CEILING_S (default 14400).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
. "$HERE/common.sh"

N="${1:-3}"
CEILING="${COV_CENSUS_CEILING_S:-14400}"
BASE_ARTIFACTS="$COV_ARTIFACTS"

cov_stage_begin f2
# The one stage in this harness that deletes profraws on purpose (the
# `clean --profraw-only` at the top of each pass, above). Declared so the
# accounting records the loss with its reason instead of failing the stage —
# every other stage's loss is a real defect and does fail. See
# `cov_expect_profraw_loss` in common.sh.
cov_expect_profraw_loss "each census pass opens with cargo llvm-cov clean --profraw-only, \
so the baseline's profdata is not polluted by census runs (R-S0-6)"
RUNS="$COV_ARTIFACTS/f2-runs.tsv"
printf 'run\tstage\toutcome\twall_s\n' > "$RUNS"
printf 'run\tstage\ttest\n' > "$COV_ARTIFACTS/f2-failures.tsv"

started="$(date +%s)"
completed=0
for run in $(seq 1 "$N"); do
  elapsed=$(( $(date +%s) - started ))
  if [ "$run" -gt 1 ] && [ "$elapsed" -ge "$CEILING" ]; then
    cov_say "ceiling reached after $completed pass(es) (${elapsed}s ≥ ${CEILING}s) — stopping short"
    break
  fi

  cov_run cargo llvm-cov clean --profraw-only || cov_fail "cleaning profraws before pass $run failed"

  pass_t0="$(date +%s)"
  pass_rc=0
  for stage in c1-lib c2-suites c3-spawning-suites; do
    t0="$(date +%s)"
    set +e
    COV_ARTIFACTS="$BASE_ARTIFACTS/f2-run-$run" "$HERE/$stage.sh" \
      > "$BASE_ARTIFACTS/f2-run-$run-$stage.log" 2>&1
    rc=$?
    set -e
    wall=$(( $(date +%s) - t0 ))
    if [ "$rc" -eq 0 ]; then
      printf '%s\t%s\tok\t%s\n' "$run" "$stage" "$wall" >> "$RUNS"
      rm -f "$BASE_ARTIFACTS/f2-run-$run-$stage.log"
    else
      pass_rc=1
      printf '%s\t%s\tfailed\t%s\n' "$run" "$stage" "$wall" >> "$RUNS"
      cov_say "pass $run stage $stage FAILED — transcript kept"
      grep -h '^test .* FAILED$' "$BASE_ARTIFACTS/f2-run-$run/"*.log 2>/dev/null \
        | sed "s/^test /$run\t$stage\t/; s/ \.\.\. FAILED$//" \
        >> "$BASE_ARTIFACTS/f2-failures.tsv" || true
    fi
  done
  completed=$((completed + 1))
  cov_say "pass $run: rc=$pass_rc wall=$(( $(date +%s) - pass_t0 ))s"
done

TOTAL=$(( $(date +%s) - started ))
{
  printf 'arm\tinstrumented\n'
  printf 'requested\t%s\ncompleted\t%s\nceiling_s\t%s\ntotal_s\t%s\n' \
    "$N" "$completed" "$CEILING" "$TOTAL"
  if [ -f "$BASE_ARTIFACTS/f1-census.tsv" ]; then
    f1_total="$(sed -n 's/^total_s\t//p' "$BASE_ARTIFACTS/f1-census.tsv")"
    f1_runs="$(sed -n 's/^completed\t//p' "$BASE_ARTIFACTS/f1-census.tsv")"
    if [ "${f1_runs:-0}" -gt 0 ] && [ "$completed" -gt 0 ]; then
      printf 'control_mean_s\t%s\ninstrumented_mean_s\t%s\nratio\t%s\n' \
        "$(( f1_total / f1_runs ))" "$(( TOTAL / completed ))" \
        "$(awk -v a="$TOTAL" -v b="$completed" -v c="$f1_total" -v d="$f1_runs" \
             'BEGIN{ printf "%.2f", (a/b)/(c/d) }')"
    fi
  else
    printf 'ratio\tunknown — F1 has not been run in this artifacts dir\n'
  fi
} > "$COV_ARTIFACTS/f2-census.tsv"
[ "$completed" -eq "$N" ] \
  || cov_say "SHORTFALL: $completed of $N passes completed — recorded, not hidden (R-S0-7)"

# Each pass accounted for its own stages; this closing pass re-checks every
# profraw the last one left behind (each must still be mergeable) and runs the
# final hygiene sweep. The floor is 0 because the profiles belong to the C1–C3
# stages this script invokes, not to the script itself.
cov_stage_end 0 "the last pass's profiles are re-checked here, not produced here"
